use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn get(client: &mut ZuoraClient, key: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/fulfillments/{key}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(client: &mut ZuoraClient, body_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(body_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.post_json("/v1/fulfillments", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn delete(client: &mut ZuoraClient, key: &str, format: OutputFormat) -> Result<()> {
    let result = client.delete_req(&format!("/v1/fulfillments/{key}")).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
