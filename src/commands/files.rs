use std::fs;
use anyhow::Result;
use crate::client::ZuoraClient;
use crate::output::formatter::*;

pub async fn get(client: &mut ZuoraClient, file_id: &str, format: OutputFormat) -> Result<()> {
    let value: serde_json::Value = client.get_json(&format!("/v1/files/{file_id}/status")).await?;
    println!("{}", format_value(&value, format));
    Ok(())
}

pub async fn download(client: &mut ZuoraClient, file_id: &str, output_file: Option<&str>) -> Result<()> {
    let bytes = client.download(&format!("/v1/files/{file_id}")).await?;
    let file_path = output_file
        .map(String::from)
        .unwrap_or_else(|| format!("file-{file_id}"));
    fs::write(&file_path, &bytes)?;
    eprintln!("Downloaded to {file_path} ({} bytes)", bytes.len());
    Ok(())
}
