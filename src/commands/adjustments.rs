use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn list(client: &mut ZuoraClient, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json("/v1/adjustments").await?;
    let items = value.get("adjustments").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&items)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}

pub async fn get(client: &mut ZuoraClient, key: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/adjustments/{key}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(client: &mut ZuoraClient, body_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(body_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.post_json("/v1/adjustments", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn cancel(client: &mut ZuoraClient, id: &str, format: OutputFormat) -> Result<()> {
    let result: serde_json::Value = client.put_json(&format!("/v1/adjustments/{id}/cancel"), serde_json::json!({})).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
