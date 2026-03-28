use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn get(client: &mut ZuoraClient, id: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/taxation-items/{id}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn update(client: &mut ZuoraClient, id: &str, fields_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(fields_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.put_json(&format!("/v1/taxation-items/{id}"), body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn delete(client: &mut ZuoraClient, id: &str, format: OutputFormat) -> Result<()> {
    let result = client.delete_req(&format!("/v1/taxation-items/{id}")).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
