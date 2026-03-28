use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn list(client: &mut ZuoraClient, account: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/billing-documents?accountId={account}")).await?;
    let items = value.get("billingDocuments").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&items)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}
