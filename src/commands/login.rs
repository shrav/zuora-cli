use anyhow::Result;
use colored::Colorize;

use crate::client::auth;
use crate::config::store::ConfigStore;
use crate::config::Profile;

/// All available Zuora environments, in order shown to the user.
const ENVIRONMENTS: &[(&str, &str)] = &[
    ("https://rest.na.zuora.com",         "US Production Cloud 1"),
    ("https://rest.zuora.com",            "US Production Cloud 2"),
    ("https://rest.eu.zuora.com",         "EU Production"),
    ("https://rest.ap.zuora.com",         "APAC Production"),
    ("https://rest.test.zuora.com",       "US Sandbox (Developer & Test Drive)"),
    ("https://rest.sandbox.na.zuora.com", "US API Sandbox Cloud 1"),
    ("https://rest.apisandbox.zuora.com", "US API Sandbox Cloud 2"),
    ("https://rest.test.eu.zuora.com",    "EU Sandbox"),
    ("https://rest.sandbox.eu.zuora.com", "EU API Sandbox"),
    ("https://rest.test.ap.zuora.com",    "APAC Sandbox"),
];

/// Interactive login — prompts for environment, credentials, validates, saves.
pub async fn run(
    profile_name: &str,
    client_id: Option<&str>,
    client_secret: Option<&str>,
    base_url: Option<&str>,
) -> Result<()> {
    let store = ConfigStore::new()?;

    eprintln!("{}", "Zuora CLI Login".bold());
    eprintln!();

    // Resolve base URL — flag > env > interactive picker
    let base_url = match base_url {
        Some(url) => url.to_string(),
        None => match std::env::var("ZUORA_BASE_URL") {
            Ok(url) => {
                eprintln!("  Using ZUORA_BASE_URL from environment: {}", url.dimmed());
                url
            }
            Err(_) => pick_environment()?,
        },
    };

    // Get credentials — from flags, env, or interactive prompt
    let client_id = match client_id {
        Some(id) => id.to_string(),
        None => match std::env::var("ZUORA_CLIENT_ID") {
            Ok(id) => {
                eprintln!("  Using ZUORA_CLIENT_ID from environment");
                id
            }
            Err(_) => {
                eprintln!();
                eprintln!("Enter your OAuth credentials from:");
                eprintln!("  Zuora UI > {} > {} > {}",
                    "Settings".bold(),
                    "Administration".bold(),
                    "Manage OAuth Clients".bold(),
                );
                eprintln!();
                eprint!("  Client ID: ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let trimmed = input.trim().to_string();
                if trimmed.is_empty() {
                    anyhow::bail!("Client ID cannot be empty");
                }
                trimmed
            }
        },
    };

    let client_secret = match client_secret {
        Some(s) => s.to_string(),
        None => match std::env::var("ZUORA_CLIENT_SECRET") {
            Ok(s) => {
                eprintln!("  Using ZUORA_CLIENT_SECRET from environment");
                s
            }
            Err(_) => {
                let secret = rpassword::prompt_password("  Client Secret: ")?;
                if secret.is_empty() {
                    anyhow::bail!("Client Secret cannot be empty");
                }
                secret
            }
        },
    };

    // Validate credentials by fetching a token
    eprintln!();
    eprintln!("Authenticating with {}...", base_url.dimmed());

    let token_resp = auth::fetch_token(&base_url, &client_id, &client_secret).await?;

    // Save profile and token
    let profile = Profile {
        client_id: Some(client_id),
        client_secret: Some(client_secret),
        base_url: Some(base_url.clone()),
    };
    store.save_profile(profile_name, &profile)?;
    store.save_token(profile_name, &token_resp.access_token, token_resp.expires_in)?;

    eprintln!();
    eprintln!("{} Done! Profile '{}' saved to ~/.zuora/config.toml", "✓".green(), profile_name);
    eprintln!();
    eprintln!("  Environment: {}", base_url);
    eprintln!("  Token:       valid for {} seconds", token_resp.expires_in);
    eprintln!();
    eprintln!("Try it out:");
    eprintln!("  {} {}", "$".dimmed(), "zuora whoami".bold());
    eprintln!("  {} {}", "$".dimmed(), "zuora accounts list --limit 5".bold());

    Ok(())
}

/// Interactive environment picker — shows numbered list, returns the URL.
fn pick_environment() -> Result<String> {
    eprintln!("Select your Zuora environment:");
    eprintln!();
    for (i, (url, desc)) in ENVIRONMENTS.iter().enumerate() {
        let num = format!("  {:>2})", i + 1);
        eprintln!("{} {} {}", num.bold(), desc, url.dimmed());
    }
    eprintln!();
    eprint!("  Enter number [1]: ");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    let idx = if trimmed.is_empty() {
        0 // Default to first option (US Production Cloud 1)
    } else {
        let n: usize = trimmed.parse().map_err(|_| {
            anyhow::anyhow!("Invalid selection: '{trimmed}'. Enter a number 1-{}", ENVIRONMENTS.len())
        })?;
        if n == 0 || n > ENVIRONMENTS.len() {
            anyhow::bail!("Invalid selection: {n}. Enter a number 1-{}", ENVIRONMENTS.len());
        }
        n - 1
    };

    let (url, desc) = ENVIRONMENTS[idx];
    eprintln!();
    eprintln!("  Selected: {} ({})", desc, url);
    Ok(url.to_string())
}

/// Resolve a base URL from string — used for validation in other modules
pub fn environment_name(base_url: &str) -> &str {
    ENVIRONMENTS
        .iter()
        .find(|(url, _)| *url == base_url)
        .map(|(_, desc)| *desc)
        .unwrap_or("Custom")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_name_known() {
        assert_eq!(environment_name("https://rest.na.zuora.com"), "US Production Cloud 1");
        assert_eq!(environment_name("https://rest.eu.zuora.com"), "EU Production");
        assert_eq!(environment_name("https://rest.test.zuora.com"), "US Sandbox (Developer & Test Drive)");
        assert_eq!(environment_name("https://rest.ap.zuora.com"), "APAC Production");
    }

    #[test]
    fn environment_name_unknown() {
        assert_eq!(environment_name("https://custom.zuora.com"), "Custom");
    }

    #[test]
    fn environments_list_has_10_entries() {
        assert_eq!(ENVIRONMENTS.len(), 10);
    }

    #[test]
    fn all_environment_urls_start_with_https() {
        for (url, _) in ENVIRONMENTS {
            assert!(url.starts_with("https://"), "URL doesn't start with https: {url}");
        }
    }

    #[test]
    fn default_environment_is_us_production() {
        assert_eq!(ENVIRONMENTS[0].0, "https://rest.na.zuora.com");
    }
}
