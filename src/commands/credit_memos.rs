use anyhow::Result;

use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Id", json_path: "id" },
    ColumnDef { header: "Memo #", json_path: "number" },
    ColumnDef { header: "Amount", json_path: "amount" },
    ColumnDef { header: "Balance", json_path: "balance" },
    ColumnDef { header: "Status", json_path: "status" },
    ColumnDef { header: "Reason", json_path: "reasonCode" },
];

pub async fn list(
    client: &mut ZuoraClient,
    account: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client
        .get_json(&format!("/v1/credit-memos?accountId={account}"))
        .await?;

    let memos = value.get("creditMemos")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    match format {
        OutputFormat::Table => println!("{}", format_list_as_table(&memos, COLUMNS)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(memos))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&memos)?),
    }
    Ok(())
}

pub async fn get(
    client: &mut ZuoraClient,
    memo_id: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client
        .get_json(&format!("/v1/credit-memos/{memo_id}"))
        .await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn create(
    client: &mut ZuoraClient,
    account_id: &str,
    amount: f64,
    reason: &str,
    format: OutputFormat,
) -> Result<()> {
    let body = serde_json::json!({
        "accountId": account_id,
        "amount": amount,
        "reasonCode": reason,
    });
    let result: serde_json::Value = client.post_json("/v1/credit-memos", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}
