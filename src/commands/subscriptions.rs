use anyhow::Result;

use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Id", json_path: "Id" },
    ColumnDef { header: "Name", json_path: "Name" },
    ColumnDef { header: "Status", json_path: "Status" },
    ColumnDef { header: "Start Date", json_path: "TermStartDate" },
    ColumnDef { header: "End Date", json_path: "TermEndDate" },
];

pub async fn list(
    client: &mut ZuoraClient,
    account: &str,
    format: OutputFormat,
    status: Option<&str>,
) -> Result<()> {
    let mut zoql = format!(
        "SELECT Id, Name, Status, TermStartDate, TermEndDate \
         FROM Subscription WHERE AccountId = '{account}'"
    );
    if let Some(s) = status {
        zoql.push_str(&format!(" AND Status = '{s}'"));
    }

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
    subscription_key: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client
        .get_json(&format!("/v1/subscriptions/{subscription_key}"))
        .await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn cancel(
    client: &mut ZuoraClient,
    subscription_key: &str,
    format: OutputFormat,
) -> Result<()> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let body = serde_json::json!({
        "cancellationPolicy": "EndOfCurrentTerm",
        "cancellationEffectiveDate": today,
    });
    let result: serde_json::Value = client
        .put_json(&format!("/v1/subscriptions/{subscription_key}/cancel"), body)
        .await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
