use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::Profile;
use crate::types::responses::CachedToken;

/// Manages ~/.zuora/config.toml and ~/.zuora/tokens.json
pub struct ConfigStore {
    config_dir: PathBuf,
}

impl ConfigStore {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        let config_dir = home.join(".zuora");
        Ok(Self { config_dir })
    }

    /// Create a ConfigStore with a custom directory (for testing)
    pub fn with_dir(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    fn config_path(&self) -> PathBuf {
        self.config_dir.join("config.toml")
    }

    fn tokens_path(&self) -> PathBuf {
        self.config_dir.join("tokens.json")
    }

    fn ensure_dir(&self) -> Result<()> {
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir)
                .context("Failed to create ~/.zuora directory")?;
        }
        Ok(())
    }

    /// Read all profiles from config.toml
    pub fn read_profiles(&self) -> Result<HashMap<String, Profile>> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = fs::read_to_string(&path)
            .context("Failed to read ~/.zuora/config.toml")?;
        let profiles: HashMap<String, Profile> = toml::from_str(&content)
            .context("Failed to parse ~/.zuora/config.toml")?;
        Ok(profiles)
    }

    /// Get a specific profile
    pub fn get_profile(&self, name: &str) -> Result<Option<Profile>> {
        let profiles = self.read_profiles()?;
        Ok(profiles.get(name).cloned())
    }

    /// Save a profile (creates or updates)
    pub fn save_profile(&self, name: &str, profile: &Profile) -> Result<()> {
        self.ensure_dir()?;
        let mut profiles = self.read_profiles().unwrap_or_default();
        profiles.insert(name.to_string(), profile.clone());
        let content = toml::to_string_pretty(&profiles)
            .context("Failed to serialize config")?;
        fs::write(self.config_path(), content)
            .context("Failed to write ~/.zuora/config.toml")?;
        Ok(())
    }

    /// Set a single config value within a profile
    pub fn set_value(&self, profile_name: &str, key: &str, value: &str) -> Result<()> {
        let mut profile = self.get_profile(profile_name)?.unwrap_or_default();
        match key {
            "client_id" => profile.client_id = Some(value.to_string()),
            "client_secret" => profile.client_secret = Some(value.to_string()),
            "base_url" => profile.base_url = Some(value.to_string()),
            _ => anyhow::bail!("Unknown config key: {key}. Valid keys: client_id, client_secret, base_url"),
        }
        self.save_profile(profile_name, &profile)
    }

    /// Get a single config value
    pub fn get_value(&self, profile_name: &str, key: &str) -> Result<Option<String>> {
        let profile = self.get_profile(profile_name)?;
        let profile = match profile {
            Some(p) => p,
            None => return Ok(None),
        };
        Ok(match key {
            "client_id" => profile.client_id,
            "client_secret" => profile.client_secret,
            "base_url" => profile.base_url,
            _ => anyhow::bail!("Unknown config key: {key}. Valid keys: client_id, client_secret, base_url"),
        })
    }

    // --- Token cache ---

    /// Read all cached tokens
    fn read_tokens(&self) -> Result<HashMap<String, CachedToken>> {
        let path = self.tokens_path();
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let content = fs::read_to_string(&path)
            .context("Failed to read tokens cache")?;
        let tokens: HashMap<String, CachedToken> = serde_json::from_str(&content)
            .unwrap_or_default();
        Ok(tokens)
    }

    /// Get a cached token for a profile (returns None if expired)
    pub fn get_cached_token(&self, profile_name: &str) -> Result<Option<CachedToken>> {
        let tokens = self.read_tokens()?;
        if let Some(token) = tokens.get(profile_name) {
            let now = chrono::Utc::now().timestamp();
            // Consider expired 60 seconds early to avoid edge cases
            if token.expires_at > now + 60 {
                return Ok(Some(token.clone()));
            }
        }
        Ok(None)
    }

    /// Cache a token for a profile
    pub fn save_token(&self, profile_name: &str, access_token: &str, expires_in: u64) -> Result<()> {
        self.ensure_dir()?;
        let mut tokens = self.read_tokens().unwrap_or_default();
        let expires_at = chrono::Utc::now().timestamp() + expires_in as i64;
        tokens.insert(profile_name.to_string(), CachedToken {
            access_token: access_token.to_string(),
            expires_at,
            profile: profile_name.to_string(),
        });
        let content = serde_json::to_string_pretty(&tokens)?;
        fs::write(self.tokens_path(), content)
            .context("Failed to write token cache")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store() -> (tempfile::TempDir, ConfigStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::with_dir(dir.path().to_path_buf());
        (dir, store)
    }

    // --- Profile CRUD ---

    #[test]
    fn read_profiles_returns_empty_when_no_file() {
        let (_dir, store) = temp_store();
        let profiles = store.read_profiles().unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn save_and_read_profile() {
        let (_dir, store) = temp_store();
        let profile = Profile {
            client_id: Some("test-id".into()),
            client_secret: Some("test-secret".into()),
            base_url: Some("https://test.zuora.com".into()),
        };
        store.save_profile("default", &profile).unwrap();

        let loaded = store.get_profile("default").unwrap().unwrap();
        assert_eq!(loaded.client_id.unwrap(), "test-id");
        assert_eq!(loaded.client_secret.unwrap(), "test-secret");
        assert_eq!(loaded.base_url.unwrap(), "https://test.zuora.com");
    }

    #[test]
    fn save_multiple_profiles() {
        let (_dir, store) = temp_store();
        let prod = Profile {
            client_id: Some("prod-id".into()),
            client_secret: Some("prod-secret".into()),
            base_url: Some("https://rest.na.zuora.com".into()),
        };
        let staging = Profile {
            client_id: Some("staging-id".into()),
            client_secret: Some("staging-secret".into()),
            base_url: Some("https://rest.test.zuora.com".into()),
        };
        store.save_profile("prod", &prod).unwrap();
        store.save_profile("staging", &staging).unwrap();

        let profiles = store.read_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles["prod"].client_id.as_deref(), Some("prod-id"));
        assert_eq!(profiles["staging"].client_id.as_deref(), Some("staging-id"));
    }

    #[test]
    fn save_profile_overwrites_existing() {
        let (_dir, store) = temp_store();
        let p1 = Profile {
            client_id: Some("old-id".into()),
            client_secret: Some("old-secret".into()),
            base_url: None,
        };
        store.save_profile("default", &p1).unwrap();

        let p2 = Profile {
            client_id: Some("new-id".into()),
            client_secret: Some("new-secret".into()),
            base_url: Some("https://new.com".into()),
        };
        store.save_profile("default", &p2).unwrap();

        let loaded = store.get_profile("default").unwrap().unwrap();
        assert_eq!(loaded.client_id.unwrap(), "new-id");
    }

    #[test]
    fn get_nonexistent_profile_returns_none() {
        let (_dir, store) = temp_store();
        assert!(store.get_profile("nonexistent").unwrap().is_none());
    }

    // --- set_value / get_value ---

    #[test]
    fn set_and_get_value() {
        let (_dir, store) = temp_store();
        store.set_value("default", "client_id", "my-id").unwrap();
        store.set_value("default", "client_secret", "my-secret").unwrap();
        store.set_value("default", "base_url", "https://test.com").unwrap();

        assert_eq!(store.get_value("default", "client_id").unwrap(), Some("my-id".into()));
        assert_eq!(store.get_value("default", "client_secret").unwrap(), Some("my-secret".into()));
        assert_eq!(store.get_value("default", "base_url").unwrap(), Some("https://test.com".into()));
    }

    #[test]
    fn set_value_unknown_key_errors() {
        let (_dir, store) = temp_store();
        let result = store.set_value("default", "unknown_key", "value");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown config key"));
    }

    #[test]
    fn get_value_unknown_key_errors() {
        let (_dir, store) = temp_store();
        store.set_value("default", "client_id", "id").unwrap();
        let result = store.get_value("default", "bad_key");
        assert!(result.is_err());
    }

    #[test]
    fn get_value_missing_profile_returns_none() {
        let (_dir, store) = temp_store();
        assert_eq!(store.get_value("nope", "client_id").unwrap(), None);
    }

    // --- Token cache ---

    #[test]
    fn save_and_get_cached_token() {
        let (_dir, store) = temp_store();
        store.save_token("default", "tok-abc", 3600).unwrap();

        let cached = store.get_cached_token("default").unwrap().unwrap();
        assert_eq!(cached.access_token, "tok-abc");
        assert_eq!(cached.profile, "default");
    }

    #[test]
    fn expired_token_returns_none() {
        let (_dir, store) = temp_store();
        // Save a token that "expires" in 30 seconds — but we consider expired 60s early
        store.save_token("default", "tok-expired", 30).unwrap();

        let cached = store.get_cached_token("default").unwrap();
        assert!(cached.is_none(), "Token that expires within 60s buffer should be treated as expired");
    }

    #[test]
    fn get_cached_token_missing_profile_returns_none() {
        let (_dir, store) = temp_store();
        assert!(store.get_cached_token("nope").unwrap().is_none());
    }

    #[test]
    fn save_token_creates_dir_if_missing() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("deep").join("nested");
        let store = ConfigStore::with_dir(nested);
        store.save_token("default", "tok-123", 3600).unwrap();

        let cached = store.get_cached_token("default").unwrap().unwrap();
        assert_eq!(cached.access_token, "tok-123");
    }

    #[test]
    fn multiple_profile_tokens() {
        let (_dir, store) = temp_store();
        store.save_token("prod", "prod-tok", 3600).unwrap();
        store.save_token("staging", "staging-tok", 3600).unwrap();

        assert_eq!(store.get_cached_token("prod").unwrap().unwrap().access_token, "prod-tok");
        assert_eq!(store.get_cached_token("staging").unwrap().unwrap().access_token, "staging-tok");
    }

    // --- Config file format ---

    #[test]
    fn config_toml_roundtrips_correctly() {
        let (_dir, store) = temp_store();
        let profile = Profile {
            client_id: Some("id-with-special=chars/here".into()),
            client_secret: Some("secret+with&things".into()),
            base_url: Some("https://rest.eu.zuora.com".into()),
        };
        store.save_profile("eu-prod", &profile).unwrap();

        // Read back raw TOML to verify format
        let content = fs::read_to_string(store.config_path()).unwrap();
        assert!(content.contains("[eu-prod]"));
        assert!(content.contains("id-with-special=chars/here"));

        // Read back through API
        let loaded = store.get_profile("eu-prod").unwrap().unwrap();
        assert_eq!(loaded.client_id.unwrap(), "id-with-special=chars/here");
    }
}
