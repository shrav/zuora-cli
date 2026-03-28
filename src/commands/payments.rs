use anyhow::Result;

use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Id", json_path: "Id" },
    ColumnDef { header: "Payment #", json_path: "PaymentNumber" },
    ColumnDef { header: "Amount", json_path: "Amount" },
    ColumnDef { header: "Date", json_path: "EffectiveDate" },
    ColumnDef { header: "Status", json_path: "Status" },
    ColumnDef { header: "Gateway Response", json_path: "GatewayResponse" },
];

pub async fn list(
    client: &mut ZuoraClient,
    account: &str,
    format: OutputFormat,
) -> Result<()> {
    let zoql = format!(
        "SELECT Id, PaymentNumber, Amount, EffectiveDate, Status, GatewayResponse \
         FROM Payment WHERE AccountId = '{account}' ORDER BY EffectiveDate DESC"
    );
    let resp = client.query(&zoql).await?;
    let records = resp.records.unwrap_or_default();

    match format {
        OutputFormat::Table => println!("{}", format_list_as_table(&records, COLUMNS)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(records))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&records)?),
    }
    Ok(())
}

pub async fn get(
    client: &mut ZuoraClient,
    payment_id: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client
        .get_json(&format!("/v1/payments/{payment_id}"))
        .await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(
    client: &mut ZuoraClient,
    account_id: &str,
    amount: f64,
    payment_method_id: &str,
    format: OutputFormat,
) -> Result<()> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let body = serde_json::json!({
        "accountId": account_id,
        "amount": amount,
        "paymentMethodId": payment_method_id,
        "effectiveDate": today,
        "type": "External",
    });
    let result: serde_json::Value = client.post_json("/v1/payments", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
