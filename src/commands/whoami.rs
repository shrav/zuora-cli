use anyhow::Result;
use colored::Colorize;

use crate::client::ZuoraClient;
use crate::commands::login::environment_name;
use crate::config::store::ConfigStore;

/// Show current authentication status, account/tenant info.
/// If authenticated, makes a lightweight API call to verify and show context.
pub async fn run(
    profile_name: &str,
    client: Option<&mut ZuoraClient>,
) -> Result<()> {
    let store = ConfigStore::new()?;

    let profile = store.get_profile(profile_name)?;
    let cached_token = store.get_cached_token(profile_name)?;

    println!("{}", "Zuora CLI".bold());
    println!();

    match profile {
        Some(p) => {
            let url = p.base_url.as_deref().unwrap_or("(not set)");
            let env = environment_name(url);

            // Try to fetch account info to verify connection and show context
            let account_info = if let Some(client) = client {
                fetch_account_summary(client).await
            } else {
                None
            };

            if let Some(ref info) = account_info {
                println!("  Account:       {}", info.bold());
            }

            println!("  Profile:       {}", profile_name.bold());
            println!("  Environment:   {} ({})", env, url.dimmed());
            println!(
                "  Client ID:     {}",
                p.client_id.as_deref().unwrap_or("(not set)")
            );
            println!("  Client Secret: {}", "••••••••".dimmed());

            match cached_token {
                Some(token) => {
                    let now = chrono::Utc::now().timestamp();
                    let remaining = token.expires_at - now;
                    if remaining > 0 {
                        println!(
                            "  Token:         {} (expires in {}s)",
                            "valid".green(),
                            remaining
                        );
                    } else {
                        println!(
                            "  Token:         {} (will refresh on next request)",
                            "expired".yellow()
                        );
                    }
                }
                None => {
                    println!("  Token:         {}", "none cached".dimmed());
                }
            }
        }
        None => {
            println!(
                "  Profile '{}' not configured.",
                profile_name
            );
            println!();
            println!("  Run {} to get started.", "`zuora login`".bold());

            let profiles = store.read_profiles()?;
            if !profiles.is_empty() {
                println!();
                println!("  Available profiles:");
                for name in profiles.keys() {
                    println!("    - {name}");
                }
            }
        }
    }

    Ok(())
}

/// Verify auth works and report the number of accounts in the tenant.
/// Zuora doesn't have a /whoami endpoint, so we use a count query.
async fn fetch_account_summary(client: &mut ZuoraClient) -> Option<String> {
    let resp = client
        .query("SELECT Id FROM Account WHERE Status = 'Active' LIMIT 1")
        .await
        .ok()?;

    resp.size.map(|_| "Authenticated".to_string())
}
