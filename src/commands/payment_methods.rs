use anyhow::Result;

use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Id", json_path: "Id" },
    ColumnDef { header: "Type", json_path: "Type" },
    ColumnDef { header: "Card #", json_path: "CreditCardMaskNumber" },
    ColumnDef { header: "Bank", json_path: "BankName" },
    ColumnDef { header: "Status", json_path: "PaymentMethodStatus" },
];

pub async fn list(
    client: &mut ZuoraClient,
    account: &str,
    format: OutputFormat,
) -> Result<()> {
    let zoql = format!(
        "SELECT Id, Type, CreditCardMaskNumber, BankName, PaymentMethodStatus \
         FROM PaymentMethod WHERE AccountId = '{account}'"
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
    pm_id: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client
        .get_json(&format!("/v1/object/payment-method/{pm_id}"))
        .await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(
    client: &mut ZuoraClient,
    account_id: &str,
    body_json: &str,
    format: OutputFormat,
) -> Result<()> {
    let mut body: serde_json::Value = serde_json::from_str(body_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    if let Some(obj) = body.as_object_mut() {
        obj.insert("accountId".to_string(), serde_json::json!(account_id));
    }
    let result: serde_json::Value = client.post_json("/v1/payment-methods", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn update(
    client: &mut ZuoraClient,
    pm_id: &str,
    fields_json: &str,
    format: OutputFormat,
) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(fields_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON for --fields: {e}"))?;
    let result: serde_json::Value = client
        .put_json(&format!("/v1/object/payment-method/{pm_id}"), body)
        .await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn delete(
    client: &mut ZuoraClient,
    pm_id: &str,
    format: OutputFormat,
) -> Result<()> {
    let result = client
        .delete_req(&format!("/v1/payment-methods/{pm_id}"))
        .await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
