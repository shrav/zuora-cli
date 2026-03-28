use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn get(client: &mut ZuoraClient, currency: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/custom-exchange-rates/{currency}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}
