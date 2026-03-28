use anyhow::Result;
use colored::Colorize;

use crate::client::ZuoraClient;
use crate::output::formatter::{OutputFormat, format_json, format_value};

/// `zuora billing-context <account-key>` — Full billing picture for an account.
/// Fetches account details, recent invoices, payment history, payment method
/// status, and recent declines in one command.
pub async fn billing_context(
    client: &mut ZuoraClient,
    account_key: &str,
    format: OutputFormat,
) -> Result<()> {
    if format == OutputFormat::Table {
        eprintln!("{}", "Billing Context".bold());
        eprintln!();
    }

    // 1. Account details
    let account: serde_json::Value = client
        .get_json(&format!("/v1/accounts/{account_key}"))
        .await?;

    let account_id = account
        .get("basicInfo")
        .and_then(|b| b.get("id"))
        .and_then(|v| v.as_str())
        .or_else(|| account.get("id").and_then(|v| v.as_str()))
        .unwrap_or(account_key);

    let account_name = account
        .get("basicInfo")
        .and_then(|b| b.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    if format == OutputFormat::Table {
        eprintln!("  Account: {} ({})", account_name.bold(), account_id);
    }

    // 2. Recent invoices
    let invoices = client
        .query_all(&format!(
            "SELECT InvoiceNumber, Amount, Balance, Status, DueDate \
             FROM Invoice WHERE AccountId = '{account_id}'"
        ))
        .await
        .unwrap_or_default();

    let outstanding: Vec<&serde_json::Value> = invoices
        .iter()
        .filter(|inv| {
            inv.get("Balance")
                .and_then(|b| b.as_f64())
                .unwrap_or(0.0)
                > 0.0
        })
        .collect();

    let total_outstanding: f64 = outstanding
        .iter()
        .map(|inv| inv.get("Balance").and_then(|b| b.as_f64()).unwrap_or(0.0))
        .sum();

    // 3. Payment history + decline check
    let payments = client
        .query_all(&format!(
            "SELECT Amount, EffectiveDate, Status, GatewayResponse, GatewayResponseCode \
             FROM Payment WHERE AccountId = '{account_id}'"
        ))
        .await
        .unwrap_or_default();

    let declines: Vec<&serde_json::Value> = payments
        .iter()
        .filter(|p| {
            matches!(
                p.get("Status").and_then(|s| s.as_str()),
                Some("Error" | "Declined" | "Voided")
            )
        })
        .collect();

    let recent_declines: Vec<&&serde_json::Value> = declines
        .iter()
        .filter(|p| {
            p.get("EffectiveDate")
                .and_then(|d| d.as_str())
                .map(|d| d >= "2024-01-01") // rough "recent" filter
                .unwrap_or(false)
        })
        .collect();

    // 4. Payment method
    let pms = client
        .query_all(&format!(
            "SELECT Id, Type, CreditCardMaskNumber, BankName, PaymentMethodStatus \
             FROM PaymentMethod WHERE AccountId = '{account_id}'"
        ))
        .await
        .unwrap_or_default();

    let active_pm = pms.iter().find(|pm| {
        pm.get("PaymentMethodStatus")
            .and_then(|s| s.as_str())
            == Some("Active")
    });

    // 5. Active subscriptions
    let subs = client
        .query_all(&format!(
            "SELECT Name, Status, TermStartDate, TermEndDate \
             FROM Subscription WHERE AccountId = '{account_id}' AND Status = 'Active'"
        ))
        .await
        .unwrap_or_default();

    // Output
    match format {
        OutputFormat::Json | OutputFormat::Raw => {
            let result = serde_json::json!({
                "account": {
                    "id": account_id,
                    "name": account_name,
                },
                "invoices": {
                    "total": invoices.len(),
                    "outstanding": outstanding.len(),
                    "totalOutstandingBalance": total_outstanding,
                },
                "payments": {
                    "total": payments.len(),
                    "declines": declines.len(),
                    "recentDeclines": recent_declines.len(),
                },
                "paymentMethod": active_pm.map(|pm| serde_json::json!({
                    "type": pm.get("Type"),
                    "card": pm.get("CreditCardMaskNumber"),
                    "bank": pm.get("BankName"),
                    "status": pm.get("PaymentMethodStatus"),
                })),
                "activeSubscriptions": subs.len(),
            });
            if format == OutputFormat::Raw {
                println!("{}", serde_json::to_string(&result)?);
            } else {
                println!("{}", format_json(&result));
            }
        }
        OutputFormat::Table => {
            println!("  Subscriptions: {} active", subs.len().to_string().bold());
            println!(
                "  Invoices:      {} total, {} outstanding (${:.2})",
                invoices.len(),
                outstanding.len().to_string().bold(),
                total_outstanding
            );
            println!(
                "  Payments:      {} total, {} declines ({} recent)",
                payments.len(),
                declines.len(),
                recent_declines.len()
            );

            match active_pm {
                Some(pm) => {
                    let pm_type = pm.get("Type").and_then(|t| t.as_str()).unwrap_or("Unknown");
                    let detail = pm
                        .get("CreditCardMaskNumber")
                        .and_then(|v| v.as_str())
                        .or_else(|| pm.get("BankName").and_then(|v| v.as_str()))
                        .unwrap_or("");
                    println!("  Payment Method: {} {}", pm_type.green(), detail);
                }
                None => {
                    println!("  Payment Method: {}", "none active".red());
                }
            }

            if !recent_declines.is_empty() {
                println!();
                println!(
                    "  {} Recent payment declines detected — payment method may need updating",
                    "!".yellow()
                );
            }

            if total_outstanding > 0.0 && active_pm.is_none() {
                println!();
                println!(
                    "  {} Outstanding balance with no active payment method",
                    "!".red()
                );
            }
        }
    }

    Ok(())
}

/// `zuora collections <account-key>` — Outstanding invoices aged by bucket.
pub async fn collections(
    client: &mut ZuoraClient,
    account_key: &str,
    format: OutputFormat,
) -> Result<()> {
    // Resolve account ID
    let account: serde_json::Value = client
        .get_json(&format!("/v1/accounts/{account_key}"))
        .await?;

    let account_id = account
        .get("basicInfo")
        .and_then(|b| b.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or(account_key);

    let invoices = client
        .query_all(&format!(
            "SELECT InvoiceNumber, InvoiceDate, DueDate, Amount, Balance, Status \
             FROM Invoice WHERE AccountId = '{account_id}' AND Balance > 0"
        ))
        .await?;

    if invoices.is_empty() {
        if format == OutputFormat::Table {
            println!("No outstanding invoices.");
        } else {
            println!("{}", format_json(&serde_json::json!({"outstanding": []})));
        }
        return Ok(());
    }

    let today = chrono::Utc::now().date_naive();
    let mut current: Vec<&serde_json::Value> = Vec::new();
    let mut days_30: Vec<&serde_json::Value> = Vec::new();
    let mut days_60: Vec<&serde_json::Value> = Vec::new();
    let mut days_90: Vec<&serde_json::Value> = Vec::new();
    let mut over_90: Vec<&serde_json::Value> = Vec::new();

    for inv in &invoices {
        let due = inv
            .get("DueDate")
            .and_then(|d| d.as_str())
            .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

        match due {
            Some(due_date) => {
                let days_past = (today - due_date).num_days();
                if days_past <= 0 {
                    current.push(inv);
                } else if days_past <= 30 {
                    days_30.push(inv);
                } else if days_past <= 60 {
                    days_60.push(inv);
                } else if days_past <= 90 {
                    days_90.push(inv);
                } else {
                    over_90.push(inv);
                }
            }
            None => current.push(inv),
        }
    }

    let bucket_total = |items: &[&serde_json::Value]| -> f64 {
        items
            .iter()
            .map(|i| i.get("Balance").and_then(|b| b.as_f64()).unwrap_or(0.0))
            .sum()
    };

    match format {
        OutputFormat::Json | OutputFormat::Raw => {
            let result = serde_json::json!({
                "accountId": account_id,
                "totalOutstanding": bucket_total(&invoices.iter().collect::<Vec<_>>()),
                "buckets": {
                    "current": { "count": current.len(), "amount": bucket_total(&current) },
                    "1-30 days": { "count": days_30.len(), "amount": bucket_total(&days_30) },
                    "31-60 days": { "count": days_60.len(), "amount": bucket_total(&days_60) },
                    "61-90 days": { "count": days_60.len(), "amount": bucket_total(&days_90) },
                    "90+ days": { "count": over_90.len(), "amount": bucket_total(&over_90) },
                },
                "invoices": invoices,
            });
            println!("{}", format_value(&result, format));
        }
        OutputFormat::Table => {
            println!("{}", "Collections Report".bold());
            println!();
            println!(
                "  Total Outstanding: {}",
                format!("${:.2}", bucket_total(&invoices.iter().collect::<Vec<_>>())).bold()
            );
            println!();
            println!("  {:>12}  {:>6}  {:>12}", "Bucket", "Count", "Amount");
            println!("  {:>12}  {:>6}  {:>12}", "------", "-----", "------");

            let print_bucket = |name: &str, items: &[&serde_json::Value]| {
                if !items.is_empty() {
                    println!(
                        "  {:>12}  {:>6}  {:>12}",
                        name,
                        items.len(),
                        format!("${:.2}", bucket_total(items))
                    );
                }
            };

            print_bucket("Current", &current);
            print_bucket("1-30 days", &days_30);
            print_bucket("31-60 days", &days_60);
            print_bucket("61-90 days", &days_90);
            print_bucket("90+ days", &over_90);

            if !over_90.is_empty() {
                println!();
                println!(
                    "  {} {} invoices are 90+ days past due",
                    "!".red(),
                    over_90.len()
                );
            }
        }
    }

    Ok(())
}

/// `zuora customer-health <account-key>` — Overall health score.
pub async fn customer_health(
    client: &mut ZuoraClient,
    account_key: &str,
    format: OutputFormat,
) -> Result<()> {
    let account: serde_json::Value = client
        .get_json(&format!("/v1/accounts/{account_key}"))
        .await?;

    let account_id = account
        .get("basicInfo")
        .and_then(|b| b.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or(account_key);

    let account_name = account
        .get("basicInfo")
        .and_then(|b| b.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let balance = account
        .get("metrics")
        .and_then(|m| m.get("balance"))
        .and_then(|b| b.as_f64())
        .unwrap_or(0.0);

    // Subscriptions
    let subs = client
        .query_all(&format!(
            "SELECT Status FROM Subscription WHERE AccountId = '{account_id}'"
        ))
        .await
        .unwrap_or_default();

    let active_subs = subs
        .iter()
        .filter(|s| s.get("Status").and_then(|v| v.as_str()) == Some("Active"))
        .count();

    // Payment declines in last 90 days
    let payments = client
        .query_all(&format!(
            "SELECT Status FROM Payment WHERE AccountId = '{account_id}'"
        ))
        .await
        .unwrap_or_default();

    let total_payments = payments.len();
    let failed_payments = payments
        .iter()
        .filter(|p| {
            matches!(
                p.get("Status").and_then(|s| s.as_str()),
                Some("Error" | "Declined" | "Voided")
            )
        })
        .count();

    // Payment method
    let pms = client
        .query_all(&format!(
            "SELECT PaymentMethodStatus FROM PaymentMethod WHERE AccountId = '{account_id}'"
        ))
        .await
        .unwrap_or_default();

    let has_active_pm = pms.iter().any(|pm| {
        pm.get("PaymentMethodStatus").and_then(|s| s.as_str()) == Some("Active")
    });

    // Overdue invoices
    let overdue = client
        .query_all(&format!(
            "SELECT Balance FROM Invoice WHERE AccountId = '{account_id}' AND Balance > 0"
        ))
        .await
        .unwrap_or_default();

    // Health scoring
    let mut score = 100i32;
    let mut flags: Vec<String> = Vec::new();

    if active_subs == 0 {
        score -= 30;
        flags.push("No active subscriptions".to_string());
    }
    if !has_active_pm {
        score -= 25;
        flags.push("No active payment method".to_string());
    }
    if !overdue.is_empty() {
        score -= 15;
        flags.push(format!("{} overdue invoices", overdue.len()));
    }
    if balance > 0.0 {
        score -= 10;
        flags.push(format!("Outstanding balance: ${balance:.2}"));
    }
    if failed_payments > 0 {
        let rate = (failed_payments as f64 / total_payments.max(1) as f64) * 100.0;
        if rate > 20.0 {
            score -= 20;
        } else if rate > 5.0 {
            score -= 10;
        }
        flags.push(format!(
            "{failed_payments}/{total_payments} payments failed ({rate:.0}%)"
        ));
    }

    let score = score.max(0);
    let health = if score >= 80 {
        "Healthy"
    } else if score >= 50 {
        "At Risk"
    } else {
        "Critical"
    };

    match format {
        OutputFormat::Json | OutputFormat::Raw => {
            let result = serde_json::json!({
                "accountId": account_id,
                "accountName": account_name,
                "healthScore": score,
                "healthStatus": health,
                "activeSubscriptions": active_subs,
                "hasActivePaymentMethod": has_active_pm,
                "outstandingBalance": balance,
                "overdueInvoices": overdue.len(),
                "failedPayments": failed_payments,
                "totalPayments": total_payments,
                "flags": flags,
            });
            println!("{}", format_value(&result, format));
        }
        OutputFormat::Table => {
            let health_colored = match health {
                "Healthy" => health.green().to_string(),
                "At Risk" => health.yellow().to_string(),
                _ => health.red().to_string(),
            };

            println!("{}", "Customer Health".bold());
            println!();
            println!("  Account:       {} ({})", account_name.bold(), account_id);
            println!("  Health:        {} (score: {}/100)", health_colored, score);
            println!("  Subscriptions: {} active", active_subs);
            println!(
                "  Payment Method: {}",
                if has_active_pm {
                    "active".green().to_string()
                } else {
                    "none".red().to_string()
                }
            );
            println!("  Balance:       ${:.2}", balance);
            println!("  Overdue:       {} invoices", overdue.len());
            println!(
                "  Payments:      {}/{} failed",
                failed_payments, total_payments
            );

            if !flags.is_empty() {
                println!();
                for flag in &flags {
                    println!("  {} {flag}", "!".yellow());
                }
            }
        }
    }

    Ok(())
}
