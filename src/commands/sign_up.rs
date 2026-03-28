use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn run(client: &mut ZuoraClient, body_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(body_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.post_json("/v1/sign-up", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
