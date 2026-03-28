/// Tests for workflow commands (billing-context, collections, customer-health),
/// ZOQL pagination (queryMore), and MCP server.
use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{body_string_contains, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn zuora() -> Command { Command::cargo_bin("zuora").unwrap() }

async fn setup(server: &MockServer) -> tempfile::TempDir {
    Mock::given(method("POST")).and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "tok", "token_type": "bearer", "expires_in": 3600
        }))).mount(server).await;
    let dir = tempfile::tempdir().unwrap();
    zuora().env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();
    dir
}

async fn mount_account(server: &MockServer) {
    Mock::given(method("GET")).and(path("/v1/accounts/acc-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "basicInfo": { "id": "acc-1", "name": "Acme Corp" },
            "metrics": { "balance": 500.0 }
        }))).mount(server).await;
}

async fn mount_zoql(server: &MockServer) {
    Mock::given(method("POST")).and(path("/v1/action/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 2,
            "records": [
                { "Id": "r1", "InvoiceNumber": "INV-001", "Amount": 100.0, "Balance": 50.0, "Status": "Posted", "DueDate": "2024-01-01", "Name": "Sub-1", "Type": "CreditCard", "PaymentMethodStatus": "Active", "CreditCardMaskNumber": "****1234", "BankName": null, "EffectiveDate": "2024-06-01", "GatewayResponse": "Approved", "GatewayResponseCode": "00", "TermStartDate": "2024-01-01", "TermEndDate": "2025-01-01", "AccountNumber": "A-001" },
                { "Id": "r2", "InvoiceNumber": "INV-002", "Amount": 200.0, "Balance": 0.0, "Status": "Posted", "DueDate": "2024-06-01", "Name": "Sub-2", "Type": "ACH", "PaymentMethodStatus": "Closed", "CreditCardMaskNumber": null, "BankName": "Wells", "EffectiveDate": "2024-07-01", "GatewayResponse": "Declined", "GatewayResponseCode": "05", "TermStartDate": "2024-01-01", "TermEndDate": "2025-01-01", "AccountNumber": "A-001" }
            ]
        }))).mount(server).await;
}

// ============================================================
// Billing Context
// ============================================================

#[tokio::test]
async fn billing_context_table() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    mount_account(&s).await;
    mount_zoql(&s).await;

    zuora().env("HOME", d.path())
        .args(["--base-url", &s.uri(), "billing-context", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Acme Corp").or(predicate::str::contains("Subscriptions")));
}

#[tokio::test]
async fn billing_context_json() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    mount_account(&s).await;
    mount_zoql(&s).await;

    zuora().env("HOME", d.path())
        .args(["--base-url", &s.uri(), "--output", "json", "billing-context", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("activeSubscriptions"));
}

// ============================================================
// Collections
// ============================================================

#[tokio::test]
async fn collections_table() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    mount_account(&s).await;
    mount_zoql(&s).await;

    zuora().env("HOME", d.path())
        .args(["--base-url", &s.uri(), "collections", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Outstanding").or(predicate::str::contains("50")));
}

#[tokio::test]
async fn collections_json() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    mount_account(&s).await;
    mount_zoql(&s).await;

    zuora().env("HOME", d.path())
        .args(["--base-url", &s.uri(), "--output", "json", "collections", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("totalOutstanding").or(predicate::str::contains("outstanding")));
}

// ============================================================
// Customer Health
// ============================================================

#[tokio::test]
async fn customer_health_table() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    mount_account(&s).await;
    mount_zoql(&s).await;

    zuora().env("HOME", d.path())
        .args(["--base-url", &s.uri(), "customer-health", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Health"));
}

#[tokio::test]
async fn customer_health_json() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    mount_account(&s).await;
    mount_zoql(&s).await;

    zuora().env("HOME", d.path())
        .args(["--base-url", &s.uri(), "--output", "json", "customer-health", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("healthScore"));
}

// ============================================================
// ZOQL Pagination (queryMore)
// ============================================================

#[tokio::test]
async fn query_pagination_follows_query_more() {
    let s = MockServer::start().await;
    let d = setup(&s).await;

    // First page: done=false, has queryLocator
    Mock::given(method("POST")).and(path("/v1/action/query")).and(body_string_contains("SELECT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": false, "size": 4,
            "queryLocator": "page2-loc",
            "records": [{"Id": "1"}, {"Id": "2"}]
        }))).mount(&s).await;

    // Second page: done=true
    Mock::given(method("POST")).and(path("/v1/action/queryMore"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 4,
            "records": [{"Id": "3"}, {"Id": "4"}]
        }))).mount(&s).await;

    zuora().env("HOME", d.path())
        .args(["--base-url", &s.uri(), "--output", "raw", "query", "SELECT Id FROM Account"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Id\":\"3\""))
        .stdout(predicate::str::contains("\"Id\":\"4\""))
        .stderr(predicate::str::contains("4 record(s)"));
}

// ============================================================
// MCP Server
// ============================================================

#[tokio::test]
async fn mcp_help_shows_command() {
    zuora()
        .args(["mcp", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("MCP server"))
        .stdout(predicate::str::contains("Claude Desktop"));
}

// ============================================================
// CLI help for new commands
// ============================================================

#[tokio::test]
async fn billing_context_help() {
    zuora()
        .args(["billing-context", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("billing context"));
}

#[tokio::test]
async fn collections_help() {
    zuora()
        .args(["collections", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Collections"));
}

#[tokio::test]
async fn customer_health_help() {
    zuora()
        .args(["customer-health", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("health"));
}
