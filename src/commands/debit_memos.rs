use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Id", json_path: "id" },
    ColumnDef { header: "Number", json_path: "number" },
    ColumnDef { header: "Amount", json_path: "amount" },
    ColumnDef { header: "Balance", json_path: "balance" },
    ColumnDef { header: "Status", json_path: "status" },
    ColumnDef { header: "Date", json_path: "debitMemoDate" },
];

pub async fn list(client: &mut ZuoraClient, account: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/debit-memos?accountId={account}")).await?;
    let items = value.get("debitMemos").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_list_as_table(&items, COLUMNS)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(items))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&items)?),
    }
    Ok(())
}

pub async fn get(client: &mut ZuoraClient, memo_id: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/debit-memos/{memo_id}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(client: &mut ZuoraClient, body_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(body_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.post_json("/v1/debit-memos", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn cancel(client: &mut ZuoraClient, memo_id: &str, format: OutputFormat) -> Result<()> {
    let result: serde_json::Value = client.put_json(&format!("/v1/debit-memos/{memo_id}/cancel"), serde_json::json!({})).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
