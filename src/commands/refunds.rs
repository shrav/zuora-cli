use anyhow::Result;

use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Id", json_path: "id" },
    ColumnDef { header: "Refund #", json_path: "number" },
    ColumnDef { header: "Amount", json_path: "amount" },
    ColumnDef { header: "Status", json_path: "status" },
    ColumnDef { header: "Date", json_path: "refundDate" },
];

pub async fn list(
    client: &mut ZuoraClient,
    account: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client
        .get_json(&format!("/v1/refunds?accountId={account}"))
        .await?;

    let refunds = value.get("refunds")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    match format {
        OutputFormat::Table => println!("{}", format_list_as_table(&refunds, COLUMNS)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(refunds))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&refunds)?),
    }
    Ok(())
}

pub async fn get(
    client: &mut ZuoraClient,
    refund_id: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client
        .get_json(&format!("/v1/refunds/{refund_id}"))
        .await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(
    client: &mut ZuoraClient,
    payment_id: &str,
    amount: f64,
    format: OutputFormat,
) -> Result<()> {
    let body = serde_json::json!({
        "paymentId": payment_id,
        "amount": amount,
        "type": "External",
    });
    let result: serde_json::Value = client.post_json("/v1/refunds", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
