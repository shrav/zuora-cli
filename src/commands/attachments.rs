use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn list(client: &mut ZuoraClient, object_type: &str, object_key: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/attachments/{object_type}/{object_key}")).await?;
    let items = value.get("attachments").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&items)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}

pub async fn get(client: &mut ZuoraClient, id: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/attachments/{id}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn delete(client: &mut ZuoraClient, id: &str, format: OutputFormat) -> Result<()> {
    let result = client.delete_req(&format!("/v1/attachments/{id}")).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
