use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn upload(client: &mut ZuoraClient, body_json: &str, format: OutputFormat) -> Result<()> {
    let body: serde_json::Value = serde_json::from_str(body_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON: {e}"))?;
    let result: serde_json::Value = client.post_json("/v1/usage", body).await?;
    println!("{}", format_value(&result, format));
    Ok(())
}

pub async fn query(client: &mut ZuoraClient, account: &str, format: OutputFormat) -> Result<()> {
    let zoql = format!(
        "SELECT Id, AccountId, ChargeId, Quantity, StartDateTime, EndDateTime, UOM \
         FROM Usage WHERE AccountId = '{account}'"
    );
    let resp = client.query(&zoql).await?;
    let records = resp.records.unwrap_or_default();
    match format {
        OutputFormat::Table => println!("{}", format_auto_table(&records)),
        OutputFormat::Json => println!("{}", format_json(&serde_json::Value::Array(records))),
        OutputFormat::Raw => println!("{}", serde_json::to_string(&records)?),
    }
    Ok(())
}
