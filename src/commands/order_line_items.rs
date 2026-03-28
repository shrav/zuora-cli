use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn get(client: &mut ZuoraClient, item_id: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/order-line-items/{item_id}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn update(client: &mut ZuoraClient, item_id: &str, fields_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(fields_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.put_json(&format!("/v1/order-line-items/{item_id}"), body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
