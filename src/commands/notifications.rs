use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn callout_history(client: &mut ZuoraClient, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json("/v1/notification-history/callout").await?;
    let items = value.get("calloutHistories").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&items)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}

pub async fn email_history(client: &mut ZuoraClient, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json("/v1/notification-history/email").await?;
    let items = value.get("emailHistories").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&items)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}
