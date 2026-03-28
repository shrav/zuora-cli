use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn by_subscription(client: &mut ZuoraClient, sub_number: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/subscription-change-logs/{sub_number}")).await?;
    let items = value.get("changeLogs").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&items)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}

pub async fn by_order(client: &mut ZuoraClient, order_number: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/subscription-change-logs/orders/{order_number}")).await?;
    let items = value.get("changeLogs").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&items)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}
