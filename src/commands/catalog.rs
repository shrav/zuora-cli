use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Id", json_path: "id" },
    ColumnDef { header: "Name", json_path: "name" },
    ColumnDef { header: "SKU", json_path: "sku" },
    ColumnDef { header: "Category", json_path: "category" },
    ColumnDef { header: "Description", json_path: "description" },
];

pub async fn list(client: &mut ZuoraClient, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json("/v1/catalog/products").await?;
    let items = value.get("products").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_list_as_table(&items, COLUMNS)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}

pub async fn get(client: &mut ZuoraClient, product_key: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/catalog/products/{product_key}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn rate_plans(client: &mut ZuoraClient, product_key: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/products/{product_key}/product-rate-plans")).await?;
    let items = value.get("productRatePlans").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&items)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}
