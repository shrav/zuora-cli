use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub base_url: Option<String>,
}

impl Profile {
    /// Resolve a complete profile by merging: CLI flags → env vars → config file
    pub fn resolve(
        config_profile: Option<&Profile>,
        env_client_id: Option<String>,
        env_client_secret: Option<String>,
        env_base_url: Option<String>,
        flag_base_url: Option<&str>,
    ) -> anyhow::Result<ResolvedProfile> {
        let base = config_profile.cloned().unwrap_or_default();

        let client_id = env_client_id
            .or(base.client_id)
            .ok_or_else(|| anyhow::anyhow!(
                "No client_id found. Run `zuora login` or set ZUORA_CLIENT_ID"
            ))?;

        let client_secret = env_client_secret
            .or(base.client_secret)
            .ok_or_else(|| anyhow::anyhow!(
                "No client_secret found. Run `zuora login` or set ZUORA_CLIENT_SECRET"
            ))?;

        let base_url = flag_base_url
            .map(String::from)
            .or(env_base_url)
            .or(base.base_url)
            .unwrap_or_else(|| "https://rest.na.zuora.com".to_string());

        Ok(ResolvedProfile {
            client_id,
            client_secret,
            base_url,
        })
    }
}

/// A fully resolved profile with all required fields present
#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    pub client_id: String,
    pub client_secret: String,
    pub base_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(id: &str, secret: &str, url: &str) -> Profile {
        Profile {
            client_id: Some(id.into()),
            client_secret: Some(secret.into()),
            base_url: Some(url.into()),
        }
    }

    #[test]
    fn resolve_from_config_profile() {
        let p = profile("cfg-id", "cfg-secret", "https://example.com");
        let resolved = Profile::resolve(Some(&p), None, None, None, None).unwrap();
        assert_eq!(resolved.client_id, "cfg-id");
        assert_eq!(resolved.client_secret, "cfg-secret");
        assert_eq!(resolved.base_url, "https://example.com");
    }

    #[test]
    fn resolve_env_overrides_config() {
        let p = profile("cfg-id", "cfg-secret", "https://cfg.com");
        let resolved = Profile::resolve(
            Some(&p),
            Some("env-id".into()),
            Some("env-secret".into()),
            Some("https://env.com".into()),
            None,
        )
        .unwrap();
        assert_eq!(resolved.client_id, "env-id");
        assert_eq!(resolved.client_secret, "env-secret");
        assert_eq!(resolved.base_url, "https://env.com");
    }

    #[test]
    fn resolve_flag_overrides_env_for_base_url() {
        let resolved = Profile::resolve(
            None,
            Some("id".into()),
            Some("secret".into()),
            Some("https://env.com".into()),
            Some("https://flag.com"),
        )
        .unwrap();
        assert_eq!(resolved.base_url, "https://flag.com");
    }

    #[test]
    fn resolve_default_base_url_when_none() {
        let resolved = Profile::resolve(
            None,
            Some("id".into()),
            Some("secret".into()),
            None,
            None,
        )
        .unwrap();
        assert_eq!(resolved.base_url, "https://rest.na.zuora.com");
    }

    #[test]
    fn resolve_missing_client_id_errors() {
        let result = Profile::resolve(None, None, Some("secret".into()), None, None);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("client_id"));
    }

    #[test]
    fn resolve_missing_client_secret_errors() {
        let result = Profile::resolve(None, Some("id".into()), None, None, None);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("client_secret"));
    }

    #[test]
    fn resolve_empty_config_with_no_env_errors() {
        let result = Profile::resolve(None, None, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_partial_config_filled_by_env() {
        let p = Profile {
            client_id: Some("cfg-id".into()),
            client_secret: None,
            base_url: None,
        };
        let resolved = Profile::resolve(
            Some(&p),
            None,
            Some("env-secret".into()),
            None,
            None,
        )
        .unwrap();
        assert_eq!(resolved.client_id, "cfg-id");
        assert_eq!(resolved.client_secret, "env-secret");
        assert_eq!(resolved.base_url, "https://rest.na.zuora.com");
    }
}
