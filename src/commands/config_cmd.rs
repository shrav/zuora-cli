use anyhow::Result;
use colored::Colorize;

use crate::config::store::ConfigStore;

pub fn run_set(profile_name: &str, key: &str, value: &str) -> Result<()> {
    let store = ConfigStore::new()?;
    store.set_value(profile_name, key, value)?;
    eprintln!("{} Set {}.{} = {}", "✓".green(), profile_name, key, value);
    Ok(())
}

pub fn run_get(profile_name: &str, key: &str) -> Result<()> {
    let store = ConfigStore::new()?;
    match store.get_value(profile_name, key)? {
        Some(value) => {
            // Mask secrets
            let display = if key == "client_secret" {
                let len = value.len();
                if len > 4 {
                    format!("{}...{}", &value[..2], &value[len - 2..])
                } else {
                    "****".to_string()
                }
            } else {
                value
            };
            println!("{display}");
        }
        None => {
            eprintln!("{} Key '{key}' not set for profile '{profile_name}'", "!".yellow());
        }
    }
    Ok(())
}

pub fn run_list(profile_name: &str) -> Result<()> {
    let store = ConfigStore::new()?;
    let profiles = store.read_profiles()?;

    if profiles.is_empty() {
        eprintln!("No profiles configured. Run `zuora login` to get started.");
        return Ok(());
    }

    if let Some(profile) = profiles.get(profile_name) {
        println!("{}", format!("[{profile_name}]").bold());
        if let Some(ref id) = profile.client_id {
            println!("  client_id = {id}");
        }
        if let Some(ref secret) = profile.client_secret {
            let len = secret.len();
            let masked = if len > 4 {
                format!("{}...{}", &secret[..2], &secret[len - 2..])
            } else {
                "****".to_string()
            };
            println!("  client_secret = {masked}");
        }
        if let Some(ref url) = profile.base_url {
            println!("  base_url = {url}");
        }
    } else {
        eprintln!("Profile '{profile_name}' not found. Available profiles:");
        for name in profiles.keys() {
            eprintln!("  - {name}");
        }
    }

    Ok(())
}
