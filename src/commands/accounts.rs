use anyhow::Result;

use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Id", json_path: "Id" },
    ColumnDef { header: "Name", json_path: "Name" },
    ColumnDef { header: "Number", json_path: "AccountNumber" },
    ColumnDef { header: "Status", json_path: "Status" },
    ColumnDef { header: "Balance", json_path: "Balance" },
    ColumnDef { header: "Currency", json_path: "Currency" },
];

pub async fn list(
    client: &mut ZuoraClient,
    format: OutputFormat,
    status: Option<&str>,
    limit: Option<usize>,
) -> Result<()> {
    let limit_val = limit.unwrap_or(20);
    let mut zoql =
        "SELECT Id, Name, AccountNumber, Status, Balance, Currency FROM Account".to_string();
    if let Some(s) = status {
        zoql.push_str(&format!(" WHERE Status = '{s}'"));
    }
    zoql.push_str(&format!(" LIMIT {limit_val}"));

    let resp = client.query(&zoql).await?;
    let records = resp.records.unwrap_or_default();

    match format {
        OutputFormat::Table => println!("{}", format_list_as_table(&records, COLUMNS)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(records))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&records)?),
    }
    Ok(())
}

pub async fn get(
    client: &mut ZuoraClient,
    account_key: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/accounts/{account_key}")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(
    client: &mut ZuoraClient,
    name: &str,
    currency: &str,
    format: OutputFormat,
) -> Result<()> {
    let body = serde_json::json!({
        "name": name,
        "currency": currency,
        "billCycleDay": 1,
        "autoPay": false,
    });
    let result: serde_json::Value = client.post_json("/v1/accounts", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn update(
    client: &mut ZuoraClient,
    account_key: &str,
    fields_json: &str,
    format: OutputFormat,
) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(fields_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON for --fields: {e}"))?;
    let result: serde_json::Value = client.put_json(&format!("/v1/accounts/{account_key}"), body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
