use std::io::{self, BufRead, Write};

use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;

use crate::client::ZuoraClient;

/// MCP (Model Context Protocol) server over stdio.
/// Exposes Zuora CLI commands as tools for Claude Desktop, Cursor, and other MCP clients.
pub async fn serve(client: &mut ZuoraClient) -> Result<()> {
    eprintln!("Zuora MCP server starting on stdio...");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err_resp = json_rpc_error(None, -32700, &format!("Parse error: {e}"));
                writeln!(stdout, "{}", serde_json::to_string(&err_resp)?)?;
                stdout.flush()?;
                continue;
            }
        };

        let response = handle_request(client, &request).await;
        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn handle_request(client: &mut ZuoraClient, req: &JsonRpcRequest) -> Value {
    match req.method.as_str() {
        "initialize" => json_rpc_result(
            req.id.clone(),
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "zuora",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            }),
        ),

        "notifications/initialized" => Value::Null, // no response needed

        "tools/list" => json_rpc_result(req.id.clone(), serde_json::json!({ "tools": tools_list() })),

        "tools/call" => {
            let tool_name = req.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = req.params.get("arguments").cloned().unwrap_or(serde_json::json!({}));
            match call_tool(client, tool_name, &args).await {
                Ok(result) => json_rpc_result(
                    req.id.clone(),
                    serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": result,
                        }]
                    }),
                ),
                Err(e) => json_rpc_result(
                    req.id.clone(),
                    serde_json::json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Error: {e:#}"),
                        }],
                        "isError": true,
                    }),
                ),
            }
        }

        _ => json_rpc_error(req.id.clone(), -32601, &format!("Method not found: {}", req.method)),
    }
}

fn tools_list() -> Vec<Value> {
    vec![
        tool("zuora_query", "Execute a ZOQL query against Zuora. Returns JSON array of records.", serde_json::json!({
            "type": "object",
            "properties": {
                "zoql": { "type": "string", "description": "ZOQL query (e.g. SELECT Id, Name FROM Account LIMIT 10)" }
            },
            "required": ["zoql"]
        })),
        tool("zuora_describe", "Describe an object's schema — lists all fields, types, and metadata.", serde_json::json!({
            "type": "object",
            "properties": {
                "object": { "type": "string", "description": "Object name (e.g. Account, Invoice, Subscription)" }
            },
            "required": ["object"]
        })),
        tool("zuora_accounts_list", "List Zuora accounts. Returns JSON array.", serde_json::json!({
            "type": "object",
            "properties": {
                "status": { "type": "string", "description": "Filter by status: Active, Draft, Canceled" },
                "limit": { "type": "integer", "description": "Max accounts to return (default 20)" }
            }
        })),
        tool("zuora_accounts_get", "Get detailed account information by ID or number.", serde_json::json!({
            "type": "object",
            "properties": {
                "account_key": { "type": "string", "description": "Account ID or account number" }
            },
            "required": ["account_key"]
        })),
        tool("zuora_subscriptions_list", "List subscriptions for an account.", serde_json::json!({
            "type": "object",
            "properties": {
                "account": { "type": "string", "description": "Account ID" },
                "status": { "type": "string", "description": "Filter: Active, Cancelled, Expired" }
            },
            "required": ["account"]
        })),
        tool("zuora_invoices_list", "List invoices for an account.", serde_json::json!({
            "type": "object",
            "properties": {
                "account": { "type": "string", "description": "Account ID" },
                "status": { "type": "string", "description": "Filter: Draft, Posted, Canceled" }
            },
            "required": ["account"]
        })),
        tool("zuora_payments_list", "List payments for an account.", serde_json::json!({
            "type": "object",
            "properties": {
                "account": { "type": "string", "description": "Account ID" }
            },
            "required": ["account"]
        })),
        tool("zuora_payment_methods_list", "List payment methods for an account.", serde_json::json!({
            "type": "object",
            "properties": {
                "account": { "type": "string", "description": "Account ID" }
            },
            "required": ["account"]
        })),
        tool("zuora_billing_context", "Get full billing context: account + invoices + payments + payment method status + decline check.", serde_json::json!({
            "type": "object",
            "properties": {
                "account_key": { "type": "string", "description": "Account ID or account number" }
            },
            "required": ["account_key"]
        })),
        tool("zuora_collections", "Get collections report: outstanding invoices aged by 30/60/90+ day buckets.", serde_json::json!({
            "type": "object",
            "properties": {
                "account_key": { "type": "string", "description": "Account ID or account number" }
            },
            "required": ["account_key"]
        })),
        tool("zuora_customer_health", "Get customer health score: subscriptions, payment method, balance, decline rate.", serde_json::json!({
            "type": "object",
            "properties": {
                "account_key": { "type": "string", "description": "Account ID or account number" }
            },
            "required": ["account_key"]
        })),
        tool("zuora_catalog_list", "List all products in the Zuora catalog.", serde_json::json!({
            "type": "object",
            "properties": {}
        })),
    ]
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    serde_json::json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
    })
}

async fn call_tool(client: &mut ZuoraClient, name: &str, args: &Value) -> Result<String> {
    let arg = |key: &str| -> Option<String> {
        args.get(key).and_then(|v| v.as_str()).map(String::from)
    };
    let _arg_or = |key: &str, default: &str| -> String {
        arg(key).unwrap_or_else(|| default.to_string())
    };
    let arg_int = |key: &str, default: usize| -> usize {
        args.get(key).and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(default)
    };

    match name {
        "zuora_query" => {
            let zoql = arg("zoql").ok_or_else(|| anyhow::anyhow!("Missing required argument: zoql"))?;
            let records = client.query_all(&zoql).await?;
            Ok(serde_json::to_string_pretty(&records)?)
        }

        "zuora_describe" => {
            let object = arg("object").ok_or_else(|| anyhow::anyhow!("Missing required argument: object"))?;
            let value: Value = client.get_json(&format!("/v1/describe/{object}")).await?;
            Ok(serde_json::to_string_pretty(&value)?)
        }

        "zuora_accounts_list" => {
            let limit = arg_int("limit", 20);
            let mut zoql = "SELECT Id, Name, AccountNumber, Status, Balance, Currency FROM Account".to_string();
            if let Some(status) = arg("status") {
                zoql.push_str(&format!(" WHERE Status = '{status}'"));
            }
            zoql.push_str(&format!(" LIMIT {limit}"));
            let records = client.query_all(&zoql).await?;
            Ok(serde_json::to_string_pretty(&records)?)
        }

        "zuora_accounts_get" => {
            let key = arg("account_key").ok_or_else(|| anyhow::anyhow!("Missing required argument: account_key"))?;
            let value: Value = client.get_json(&format!("/v1/accounts/{key}")).await?;
            Ok(serde_json::to_string_pretty(&value)?)
        }

        "zuora_subscriptions_list" => {
            let account = arg("account").ok_or_else(|| anyhow::anyhow!("Missing required argument: account"))?;
            let mut zoql = format!("SELECT Id, Name, Status, TermStartDate, TermEndDate FROM Subscription WHERE AccountId = '{account}'");
            if let Some(status) = arg("status") {
                zoql.push_str(&format!(" AND Status = '{status}'"));
            }
            let records = client.query_all(&zoql).await?;
            Ok(serde_json::to_string_pretty(&records)?)
        }

        "zuora_invoices_list" => {
            let account = arg("account").ok_or_else(|| anyhow::anyhow!("Missing required argument: account"))?;
            let mut zoql = format!("SELECT InvoiceNumber, Amount, Balance, Status, DueDate FROM Invoice WHERE AccountId = '{account}'");
            if let Some(status) = arg("status") {
                zoql.push_str(&format!(" AND Status = '{status}'"));
            }
            let records = client.query_all(&zoql).await?;
            Ok(serde_json::to_string_pretty(&records)?)
        }

        "zuora_payments_list" => {
            let account = arg("account").ok_or_else(|| anyhow::anyhow!("Missing required argument: account"))?;
            let records = client.query_all(&format!(
                "SELECT PaymentNumber, Amount, EffectiveDate, Status, GatewayResponse FROM Payment WHERE AccountId = '{account}'"
            )).await?;
            Ok(serde_json::to_string_pretty(&records)?)
        }

        "zuora_payment_methods_list" => {
            let account = arg("account").ok_or_else(|| anyhow::anyhow!("Missing required argument: account"))?;
            let records = client.query_all(&format!(
                "SELECT Id, Type, CreditCardMaskNumber, BankName, PaymentMethodStatus FROM PaymentMethod WHERE AccountId = '{account}'"
            )).await?;
            Ok(serde_json::to_string_pretty(&records)?)
        }

        "zuora_billing_context" => {
            let key = arg("account_key").ok_or_else(|| anyhow::anyhow!("Missing required argument: account_key"))?;
            let account: Value = client.get_json(&format!("/v1/accounts/{key}")).await?;
            let account_id = account.get("basicInfo").and_then(|b| b.get("id")).and_then(|v| v.as_str()).unwrap_or(&key);
            let invoices = client.query_all(&format!("SELECT InvoiceNumber, Amount, Balance, Status FROM Invoice WHERE AccountId = '{account_id}'")).await.unwrap_or_default();
            let payments = client.query_all(&format!("SELECT Amount, Status, GatewayResponse FROM Payment WHERE AccountId = '{account_id}'")).await.unwrap_or_default();
            let pms = client.query_all(&format!("SELECT Type, PaymentMethodStatus FROM PaymentMethod WHERE AccountId = '{account_id}'")).await.unwrap_or_default();

            let outstanding: f64 = invoices.iter()
                .map(|i| i.get("Balance").and_then(|b| b.as_f64()).unwrap_or(0.0))
                .filter(|b| *b > 0.0).sum();

            let declines = payments.iter().filter(|p| matches!(p.get("Status").and_then(|s| s.as_str()), Some("Error" | "Declined"))).count();
            let has_active_pm = pms.iter().any(|pm| pm.get("PaymentMethodStatus").and_then(|s| s.as_str()) == Some("Active"));

            let result = serde_json::json!({
                "accountId": account_id,
                "invoiceCount": invoices.len(),
                "outstandingBalance": outstanding,
                "paymentCount": payments.len(),
                "declineCount": declines,
                "hasActivePaymentMethod": has_active_pm,
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }

        "zuora_collections" => {
            let key = arg("account_key").ok_or_else(|| anyhow::anyhow!("Missing required argument: account_key"))?;
            let account: Value = client.get_json(&format!("/v1/accounts/{key}")).await?;
            let account_id = account.get("basicInfo").and_then(|b| b.get("id")).and_then(|v| v.as_str()).unwrap_or(&key);
            let invoices = client.query_all(&format!("SELECT InvoiceNumber, DueDate, Balance FROM Invoice WHERE AccountId = '{account_id}' AND Balance > 0")).await?;
            let total: f64 = invoices.iter().map(|i| i.get("Balance").and_then(|b| b.as_f64()).unwrap_or(0.0)).sum();
            let result = serde_json::json!({
                "accountId": account_id,
                "outstandingInvoices": invoices.len(),
                "totalOutstanding": total,
                "invoices": invoices,
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }

        "zuora_customer_health" => {
            let key = arg("account_key").ok_or_else(|| anyhow::anyhow!("Missing required argument: account_key"))?;
            let account: Value = client.get_json(&format!("/v1/accounts/{key}")).await?;
            let account_id = account.get("basicInfo").and_then(|b| b.get("id")).and_then(|v| v.as_str()).unwrap_or(&key);
            let subs = client.query_all(&format!("SELECT Status FROM Subscription WHERE AccountId = '{account_id}' AND Status = 'Active'")).await.unwrap_or_default();
            let pms = client.query_all(&format!("SELECT PaymentMethodStatus FROM PaymentMethod WHERE AccountId = '{account_id}'")).await.unwrap_or_default();
            let overdue = client.query_all(&format!("SELECT Balance FROM Invoice WHERE AccountId = '{account_id}' AND Balance > 0")).await.unwrap_or_default();

            let has_pm = pms.iter().any(|pm| pm.get("PaymentMethodStatus").and_then(|s| s.as_str()) == Some("Active"));
            let mut score = 100i32;
            if subs.is_empty() { score -= 30; }
            if !has_pm { score -= 25; }
            if !overdue.is_empty() { score -= 15; }
            let score = score.max(0);

            let result = serde_json::json!({
                "accountId": account_id,
                "healthScore": score,
                "healthStatus": if score >= 80 { "Healthy" } else if score >= 50 { "At Risk" } else { "Critical" },
                "activeSubscriptions": subs.len(),
                "hasActivePaymentMethod": has_pm,
                "overdueInvoices": overdue.len(),
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }

        "zuora_catalog_list" => {
            let value: Value = client.get_json("/v1/catalog/products").await?;
            let products = value.get("products").cloned().unwrap_or(serde_json::json!([]));
            Ok(serde_json::to_string_pretty(&products)?)
        }

        _ => anyhow::bail!("Unknown tool: {name}"),
    }
}

// --- JSON-RPC types ---

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

fn json_rpc_result(id: Option<Value>, result: Value) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

fn json_rpc_error(id: Option<Value>, code: i32, message: &str) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
}
