use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn invoice_collect(client: &mut ZuoraClient, body_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(body_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.post_json("/v1/operations/invoice-collect", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn job_status(client: &mut ZuoraClient, job_id: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/operations/jobs/{job_id}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}
