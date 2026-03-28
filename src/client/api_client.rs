use anyhow::{Context, Result};
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;

use crate::client::auth;
use crate::client::error::parse_error_body;
use crate::config::profile::ResolvedProfile;
use crate::config::store::ConfigStore;
use crate::types::responses::QueryResponse;

pub struct ZuoraClient {
    http: Client,
    profile: ResolvedProfile,
    profile_name: String,
    store: ConfigStore,
    token: Option<String>,
    pub verbose: bool,
    pub dry_run: bool,
}

impl ZuoraClient {
    pub fn new(profile: ResolvedProfile, profile_name: String, store: ConfigStore) -> Self {
        Self {
            http: Client::new(),
            profile,
            profile_name,
            store,
            token: None,
            verbose: false,
            dry_run: false,
        }
    }

    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.profile.base_url
    }

    /// Ensure we have a valid token
    async fn ensure_token(&mut self) -> Result<String> {
        if let Some(ref token) = self.token {
            return Ok(token.clone());
        }
        let token = auth::get_token(
            &self.store,
            &self.profile_name,
            &self.profile.base_url,
            &self.profile.client_id,
            &self.profile.client_secret,
        )
        .await?;
        self.token = Some(token.clone());
        Ok(token)
    }

    /// Force refresh the token (e.g., after a 401)
    async fn refresh_token(&mut self) -> Result<String> {
        let token_resp = auth::fetch_token(
            &self.profile.base_url,
            &self.profile.client_id,
            &self.profile.client_secret,
        )
        .await?;
        self.store.save_token(
            &self.profile_name,
            &token_resp.access_token,
            token_resp.expires_in,
        )?;
        self.token = Some(token_resp.access_token.clone());
        Ok(token_resp.access_token)
    }

    /// Make an authenticated HTTP request. Retries once on 401.
    async fn request(
        &mut self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<Response> {
        let url = format!("{}{}", self.profile.base_url, path);

        // Dry-run: print the request without executing
        if self.dry_run && method != Method::GET {
            eprintln!("DRY RUN — would send:");
            eprintln!("  {} {}", method, url);
            eprintln!("  Authorization: Bearer <token>");
            if let Some(ref b) = body {
                eprintln!("  Body: {}", serde_json::to_string_pretty(b)?);
            }
            anyhow::bail!("dry-run: request not sent");
        }

        let token = self.ensure_token().await?;

        if self.verbose {
            eprintln!("→ {} {}", method, url);
            if let Some(ref b) = body {
                eprintln!("→ Body: {}", serde_json::to_string_pretty(b)?);
            }
        }

        let mut req = self.http.request(method.clone(), &url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json");

        if let Some(ref b) = body {
            req = req.json(b);
        }

        let resp = req.send().await.context("HTTP request failed")?;

        if self.verbose {
            eprintln!("← {} {}", resp.status(), url);
        }

        // Retry once on 401
        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            if self.verbose {
                eprintln!("← 401 — refreshing token and retrying");
            }
            let new_token = self.refresh_token().await?;
            let mut retry_req = self.http.request(method, &url)
                .header("Authorization", format!("Bearer {new_token}"))
                .header("Content-Type", "application/json");

            if let Some(b) = body {
                retry_req = retry_req.json(&b);
            }

            let retry_resp = retry_req.send().await.context("HTTP retry failed")?;
            return Ok(retry_resp);
        }

        // Handle 429 rate limiting
        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = resp
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(5);
            anyhow::bail!(
                "Rate limited by Zuora. Retry after {retry_after} seconds."
            );
        }

        Ok(resp)
    }

    /// GET request, return parsed JSON
    pub async fn get<T: DeserializeOwned>(&mut self, path: &str) -> Result<T> {
        let resp = self.request(Method::GET, path, None).await?;
        let status = resp.status();
        let body = resp.text().await?;

        if self.verbose {
            eprintln!("← Body: {}", &body[..body.len().min(500)]);
        }

        if !status.is_success() {
            let err_msg = parse_error_body(&body);
            anyhow::bail!("GET {path} failed (HTTP {status}): {err_msg}");
        }

        serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse response from GET {path}"))
    }

    /// GET request, return raw JSON value
    pub async fn get_json(&mut self, path: &str) -> Result<serde_json::Value> {
        self.get(path).await
    }

    /// POST request with JSON body, return parsed response
    pub async fn post<T: DeserializeOwned>(
        &mut self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<T> {
        let resp = self.request(Method::POST, path, Some(body)).await?;
        let status = resp.status();
        let body_text = resp.text().await?;

        if self.verbose {
            eprintln!("← Body: {}", &body_text[..body_text.len().min(500)]);
        }

        if !status.is_success() {
            let err_msg = parse_error_body(&body_text);
            anyhow::bail!("POST {path} failed (HTTP {status}): {err_msg}");
        }

        serde_json::from_str(&body_text)
            .with_context(|| format!("Failed to parse response from POST {path}"))
    }

    /// POST request, return raw JSON value
    pub async fn post_json(
        &mut self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.post(path, body).await
    }

    /// PUT request with JSON body
    pub async fn put_json(
        &mut self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp = self.request(Method::PUT, path, Some(body)).await?;
        let status = resp.status();
        let body_text = resp.text().await?;

        if !status.is_success() {
            let err_msg = parse_error_body(&body_text);
            anyhow::bail!("PUT {path} failed (HTTP {status}): {err_msg}");
        }

        serde_json::from_str(&body_text)
            .with_context(|| format!("Failed to parse response from PUT {path}"))
    }

    /// DELETE request
    pub async fn delete_req(&mut self, path: &str) -> Result<serde_json::Value> {
        let resp = self.request(Method::DELETE, path, None).await?;
        let status = resp.status();
        let body_text = resp.text().await?;

        if !status.is_success() {
            let err_msg = parse_error_body(&body_text);
            anyhow::bail!("DELETE {path} failed (HTTP {status}): {err_msg}");
        }

        if body_text.is_empty() {
            return Ok(serde_json::json!({"success": true}));
        }
        serde_json::from_str(&body_text)
            .with_context(|| format!("Failed to parse response from DELETE {path}"))
    }

    /// Execute a ZOQL query (single page)
    pub async fn query(&mut self, zoql: &str) -> Result<QueryResponse> {
        self.post(
            "/v1/action/query",
            serde_json::json!({ "queryString": zoql }),
        )
        .await
    }

    /// Execute a ZOQL query and auto-paginate through all results.
    /// Follows `queryMore` until `done: true`.
    pub async fn query_all(&mut self, zoql: &str) -> Result<Vec<serde_json::Value>> {
        let mut all_records: Vec<serde_json::Value> = Vec::new();
        let mut page = 1;

        let first: QueryResponse = self.post(
            "/v1/action/query",
            serde_json::json!({ "queryString": zoql }),
        ).await?;

        if let Some(records) = first.records {
            all_records.extend(records);
        }

        let mut done = first.done.unwrap_or(true);
        let mut locator = first.query_locator;

        while !done {
            if let Some(ref loc) = locator {
                page += 1;
                if self.verbose {
                    eprintln!("  Fetching page {page}...");
                }
                let more: QueryResponse = self.post(
                    "/v1/action/queryMore",
                    serde_json::json!({ "queryLocator": loc }),
                ).await?;

                if let Some(records) = more.records {
                    all_records.extend(records);
                }
                done = more.done.unwrap_or(true);
                locator = more.query_locator;
            } else {
                break;
            }
        }

        Ok(all_records)
    }

    /// Download binary content (e.g., invoice PDF)
    pub async fn download(&mut self, path: &str) -> Result<Vec<u8>> {
        let token = self.ensure_token().await?;
        let url = format!("{}{}", self.profile.base_url, path);

        let resp = self.http
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await?;
            let err_msg = parse_error_body(&body);
            anyhow::bail!("Download {path} failed (HTTP {status}): {err_msg}");
        }

        Ok(resp.bytes().await?.to_vec())
    }
}
