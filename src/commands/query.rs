use anyhow::Result;

use crate::client::ZuoraClient;
use crate::output::formatter::{OutputFormat, format_auto_table, format_json};

pub async fn run(
    client: &mut ZuoraClient,
    zoql: &str,
    format: OutputFormat,
    limit: Option<usize>,
) -> Result<()> {
    // Use auto-pagination to fetch all results
    let mut records = client.query_all(zoql).await?;

    if let Some(limit) = limit {
        records.truncate(limit);
    }

    let count = records.len();

    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&records)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(records))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&records)?),
    }

    eprintln!("{count} record(s) returned");
    Ok(())
}
