use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn create(client: &mut ZuoraClient, body_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(body_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.post_json("/v1/billing-preview-runs", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn get(client: &mut ZuoraClient, id: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/billing-preview-runs/{id}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}
