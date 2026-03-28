use anyhow::{Context, Result};

use crate::config::store::ConfigStore;
use crate::types::responses::TokenResponse;

/// Fetch a fresh OAuth token from Zuora
pub async fn fetch_token(
    base_url: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<TokenResponse> {
    let http = reqwest::Client::new();
    let resp = http
        .post(format!("{base_url}/oauth/token"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(format!(
            "client_id={}&client_secret={}&grant_type=client_credentials",
            urlencoded(client_id),
            urlencoded(client_secret),
        ))
        .send()
        .await
        .context("Failed to connect to Zuora OAuth endpoint")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Authentication failed (HTTP {status}): {body}");
    }

    resp.json::<TokenResponse>()
        .await
        .context("Failed to parse OAuth token response")
}

/// Get a valid bearer token — from cache or by fetching fresh
pub async fn get_token(
    store: &ConfigStore,
    profile_name: &str,
    base_url: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<String> {
    // Check cache first
    if let Some(cached) = store.get_cached_token(profile_name)? {
        return Ok(cached.access_token);
    }

    // Fetch fresh
    let token_resp = fetch_token(base_url, client_id, client_secret).await?;
    store.save_token(profile_name, &token_resp.access_token, token_resp.expires_in)?;
    Ok(token_resp.access_token)
}

pub(crate) fn urlencoded(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencoded_passthrough_safe_chars() {
        assert_eq!(urlencoded("abcXYZ019"), "abcXYZ019");
        assert_eq!(urlencoded("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn urlencoded_encodes_special_chars() {
        assert_eq!(urlencoded("a b"), "a%20b");
        assert_eq!(urlencoded("a=b&c"), "a%3Db%26c");
        assert_eq!(urlencoded("hello/world"), "hello%2Fworld");
    }

    #[test]
    fn urlencoded_encodes_zuora_style_secrets() {
        // Zuora secrets often contain = and /
        let secret = "s3cr3t/with=special+chars";
        let encoded = urlencoded(secret);
        assert!(encoded.contains("%2F"));
        assert!(encoded.contains("%3D"));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
    }

    #[test]
    fn urlencoded_empty_string() {
        assert_eq!(urlencoded(""), "");
    }

    #[tokio::test]
    async fn fetch_token_bad_url_returns_error() {
        let result = fetch_token("http://127.0.0.1:1", "id", "secret").await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Failed to connect") || err.contains("error"),
            "Unexpected error message: {err}"
        );
    }

    #[tokio::test]
    async fn get_token_uses_cache_when_valid() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::with_dir(dir.path().to_path_buf());

        // Save a token that expires far in the future
        store.save_token("test", "cached-token-123", 7200).unwrap();

        let result = get_token(&store, "test", "http://127.0.0.1:1", "id", "secret").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "cached-token-123");
    }

    #[tokio::test]
    async fn get_token_fetches_fresh_when_no_cache() {
        let dir = tempfile::tempdir().unwrap();
        let store = ConfigStore::with_dir(dir.path().to_path_buf());

        // No cached token, and the URL is invalid — should fail trying to fetch
        let result = get_token(&store, "test", "http://127.0.0.1:1", "id", "secret").await;
        assert!(result.is_err());
    }
}
