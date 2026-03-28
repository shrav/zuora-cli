use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn run(client: &mut ZuoraClient, object: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/describe/{object}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}
