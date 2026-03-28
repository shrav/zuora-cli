use anyhow::Result;
use colored::Colorize;

use crate::client::ZuoraClient;

/// Check Zuora API health by making a lightweight request.
pub async fn run(client: &mut ZuoraClient) -> Result<()> {
    println!("{}", "Zuora API Status".bold());
    println!();

    // Test auth by fetching a simple query
    let start = std::time::Instant::now();
    let result: Result<serde_json::Value> = client.get_json("/v1/catalog/products?pageSize=1").await;
    let elapsed = start.elapsed();

    match result {
        Ok(_) => {
            println!(
                "  API:    {} ({}ms)",
                "reachable".green(),
                elapsed.as_millis()
            );
            println!("  Auth:   {}", "valid".green());
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("401") || msg.contains("Unauthorized") {
                println!("  API:    {}", "reachable".green());
                println!("  Auth:   {} — run `zuora login`", "invalid".red());
            } else {
                println!(
                    "  API:    {} — {}",
                    "unreachable".red(),
                    msg
                );
            }
        }
    }

    println!("  Latency: {}ms", elapsed.as_millis());
    Ok(())
}
