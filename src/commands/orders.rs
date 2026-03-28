use std::io::Read;
use anyhow::Result;

use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Order #", json_path: "orderNumber" },
    ColumnDef { header: "Date", json_path: "orderDate" },
    ColumnDef { header: "Status", json_path: "status" },
    ColumnDef { header: "Account", json_path: "accountNumber" },
    ColumnDef { header: "Description", json_path: "description" },
];

pub async fn list(
    client: &mut ZuoraClient,
    account: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client
        .get_json(&format!("/v1/orders/subscriptionOwner/{account}"))
        .await?;

    let orders = value.get("orders")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    match format {
        OutputFormat::Table => println!("{}", format_list_as_table(&orders, COLUMNS)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(orders))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&orders)?),
    }
    Ok(())
}

pub async fn get(
    client: &mut ZuoraClient,
    order_number: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client
        .get_json(&format!("/v1/orders/{order_number}"))
        .await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(
    client: &mut ZuoraClient,
    file: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let body_str = match file {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read {path}: {e}"))?,
        None => {
            eprintln!("Reading order JSON from stdin...");
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };

    let body: serde_json::Value = serde_json::from_str(&body_str)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.post_json("/v1/orders", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn cancel(
    client: &mut ZuoraClient,
    order_number: &str,
    format: OutputFormat,
) -> Result<()> {
    let result: serde_json::Value = client
        .put_json(&format!("/v1/orders/{order_number}/cancel"), serde_json::json!({}))
        .await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
