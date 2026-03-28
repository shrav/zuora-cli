use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Id", json_path: "id" },
    ColumnDef { header: "Status", json_path: "status" },
    ColumnDef { header: "Target Date", json_path: "targetDate" },
    ColumnDef { header: "Created", json_path: "createdDate" },
];

pub async fn list(client: &mut ZuoraClient, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json("/v1/payment-runs").await?;
    let items = value.get("paymentRuns").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_list_as_table(&items, COLUMNS)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}

pub async fn get(client: &mut ZuoraClient, key: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/payment-runs/{key}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(client: &mut ZuoraClient, body_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(body_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.post_json("/v1/payment-runs", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn summary(client: &mut ZuoraClient, key: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/payment-runs/{key}/summary")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn delete(client: &mut ZuoraClient, key: &str, format: OutputFormat) -> Result<()> {
    let result = client.delete_req(&format!("/v1/payment-runs/{key}")).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
