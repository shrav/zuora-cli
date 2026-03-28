use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn get(client: &mut ZuoraClient, ramp_number: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/ramps/{ramp_number}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn metrics(client: &mut ZuoraClient, ramp_number: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/ramps/{ramp_number}/ramp-metrics")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}
