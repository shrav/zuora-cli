use std::fs;
use anyhow::Result;

use crate::client::ZuoraClient;
use crate::output::formatter::*;

const COLUMNS: &[ColumnDef] = &[
    ColumnDef { header: "Id", json_path: "Id" },
    ColumnDef { header: "Invoice #", json_path: "InvoiceNumber" },
    ColumnDef { header: "Date", json_path: "InvoiceDate" },
    ColumnDef { header: "Due Date", json_path: "DueDate" },
    ColumnDef { header: "Amount", json_path: "Amount" },
    ColumnDef { header: "Balance", json_path: "Balance" },
    ColumnDef { header: "Status", json_path: "Status" },
];

pub async fn list(
    client: &mut ZuoraClient,
    account: &str,
    format: OutputFormat,
    status: Option<&str>,
    limit: Option<usize>,
) -> Result<()> {
    let limit_val = limit.unwrap_or(20);
    let mut zoql = format!(
        "SELECT Id, InvoiceNumber, InvoiceDate, DueDate, Amount, Balance, Status \
         FROM Invoice WHERE AccountId = '{account}'"
    );
    if let Some(s) = status {
        zoql.push_str(&format!(" AND Status = '{s}'"));
    }
    zoql.push_str(&format!(" ORDER BY InvoiceDate DESC LIMIT {limit_val}"));

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
    invoice_id: &str,
    format: OutputFormat,
) -> Result<()> {
    let value: serde_json::Value = client
        .get_json(&format!("/v1/invoices/{invoice_id}"))
        .await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn pdf(
    client: &mut ZuoraClient,
    invoice_id: &str,
    output_file: Option<&str>,
) -> Result<()> {
    let bytes = client
        .download(&format!("/v1/invoices/{invoice_id}/files"))
        .await?;

    let file_path = output_file
        .map(String::from)
        .unwrap_or_else(|| format!("invoice-{invoice_id}.pdf"));

    fs::write(&file_path, &bytes)?;
    eprintln!("Saved invoice PDF to {file_path} ({} bytes)", bytes.len());
    Ok(())
}
