use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn list(client: &mut ZuoraClient, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json("/v1/accounting-codes").await?;
    let items = value.get("accountingCodes").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&items)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}

pub async fn get(client: &mut ZuoraClient, id: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/accounting-codes/{id}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(client: &mut ZuoraClient, body_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(body_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.post_json("/v1/accounting-codes", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn activate(client: &mut ZuoraClient, id: &str, format: OutputFormat) -> Result<()> {
    let result: serde_json::Value = client.put_json(&format!("/v1/accounting-codes/{id}/activate"), serde_json::json!({})).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn deactivate(client: &mut ZuoraClient, id: &str, format: OutputFormat) -> Result<()> {
    let result: serde_json::Value = client.put_json(&format!("/v1/accounting-codes/{id}/deactivate"), serde_json::json!({})).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn delete(client: &mut ZuoraClient, id: &str, format: OutputFormat) -> Result<()> {
    let result = client.delete_req(&format!("/v1/accounting-codes/{id}")).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
