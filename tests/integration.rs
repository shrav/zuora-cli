/// Integration tests using a real HTTP server (wiremock).
///
/// wiremock starts a real TCP listener on a random port — no trait mocking,
/// no stubs. The ZuoraClient sends real HTTP requests over the network.
use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::matchers::{body_string_contains, header, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn zuora() -> Command {
    Command::cargo_bin("zuora").unwrap()
}

/// Helper: set up a mock OAuth token endpoint on the given server
async fn mount_oauth(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "test-token-abc",
            "token_type": "bearer",
            "expires_in": 3600
        })))
        .mount(server)
        .await;
}

// ============================================================
// Auth & Login
// ============================================================

#[tokio::test]
async fn login_succeeds_with_valid_credentials() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args([
            "login",
            "--client-id", "test-id",
            "--client-secret", "test-secret",
            "--base-url", &server.uri(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Done!"));
}

#[tokio::test]
async fn login_fails_with_bad_credentials() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid_client"))
        .mount(&server)
        .await;

    zuora()
        .args([
            "login",
            "--client-id", "bad",
            "--client-secret", "bad",
            "--base-url", &server.uri(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Authentication failed"));
}

#[tokio::test]
async fn login_saves_profile_and_token() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args([
            "login",
            "--profile", "staging",
            "--client-id", "stg-id",
            "--client-secret", "stg-secret",
            "--base-url", &server.uri(),
        ])
        .assert()
        .success();

    // Verify config.toml was written
    let config_path = dir.path().join(".zuora").join("config.toml");
    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("[staging]"));
    assert!(content.contains("stg-id"));

    // Verify tokens.json was written
    let tokens_path = dir.path().join(".zuora").join("tokens.json");
    let tokens_content = std::fs::read_to_string(&tokens_path).unwrap();
    assert!(tokens_content.contains("test-token-abc"));
}

// ============================================================
// Config
// ============================================================

#[tokio::test]
async fn config_set_and_get() {
    let dir = tempfile::tempdir().unwrap();

    // Set
    zuora()
        .env("HOME", dir.path())
        .args(["config", "set", "client_id", "my-test-id"])
        .assert()
        .success();

    // Get
    zuora()
        .env("HOME", dir.path())
        .args(["config", "get", "client_id"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my-test-id"));
}

#[tokio::test]
async fn config_set_invalid_key() {
    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["config", "set", "bogus_key", "value"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown config key"));
}

#[tokio::test]
async fn config_list_shows_profile() {
    let dir = tempfile::tempdir().unwrap();

    // First login to create a profile
    let server = MockServer::start().await;
    mount_oauth(&server).await;
    zuora()
        .env("HOME", dir.path())
        .args([
            "login",
            "--client-id", "list-test-id",
            "--client-secret", "list-test-secret",
            "--base-url", &server.uri(),
        ])
        .assert()
        .success();

    // List
    zuora()
        .env("HOME", dir.path())
        .args(["config", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[default]"))
        .stdout(predicate::str::contains("list-test-id"));
}

// ============================================================
// ZOQL Query
// ============================================================

#[tokio::test]
async fn query_returns_table_output() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .and(body_string_contains("SELECT"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true,
            "size": 2,
            "records": [
                {"Id": "acc-1", "Name": "Acme Corp", "Status": "Active"},
                {"Id": "acc-2", "Name": "Beta Inc", "Status": "Draft"},
            ]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    // Login first
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "secret", "--base-url", &server.uri()])
        .assert()
        .success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "query", "SELECT Id, Name, Status FROM Account"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Acme Corp"))
        .stdout(predicate::str::contains("Beta Inc"));
}

#[tokio::test]
async fn query_json_output() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true,
            "size": 1,
            "records": [{"Id": "acc-1", "Name": "Test"}]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert()
        .success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "query", "SELECT Id FROM Account"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"Id\": \"acc-1\""));
}

#[tokio::test]
async fn query_raw_output() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true,
            "size": 1,
            "records": [{"Id": "acc-1"}]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert()
        .success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "raw", "query", "SELECT Id FROM Account"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"{"Id":"acc-1"}"#));
}

#[tokio::test]
async fn query_with_limit() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true,
            "size": 3,
            "records": [
                {"Id": "1"}, {"Id": "2"}, {"Id": "3"}
            ]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert()
        .success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "raw", "query", "--limit", "1", "SELECT Id FROM Account"])
        .assert()
        .success()
        .stderr(predicate::str::contains("1 record(s)"));
}

#[tokio::test]
async fn query_empty_results() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true,
            "size": 0,
            "records": []
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert()
        .success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "query", "SELECT Id FROM Account WHERE 1=0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results"))
        .stderr(predicate::str::contains("0 record(s)"));
}

// ============================================================
// Accounts
// ============================================================

#[tokio::test]
async fn accounts_list() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .and(body_string_contains("Account"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true,
            "size": 1,
            "records": [{"Id": "a1", "Name": "Acme", "AccountNumber": "A-001", "Status": "Active", "Balance": 100.0, "Currency": "USD"}]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "accounts", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Acme"))
        .stdout(predicate::str::contains("A-001"));
}

#[tokio::test]
async fn accounts_list_with_status_filter() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .and(body_string_contains("Active"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 0, "records": []
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "accounts", "list", "--status", "Active"])
        .assert()
        .success();
}

#[tokio::test]
async fn accounts_get() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc-123"))
        .and(header("Authorization", "Bearer test-token-abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "basicInfo": {"id": "acc-123", "name": "Acme Corp"},
            "billingAndPayment": {"currency": "USD"}
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "accounts", "get", "acc-123"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Acme Corp"));
}

#[tokio::test]
async fn accounts_create() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/accounts"))
        .and(body_string_contains("NewCo"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true, "accountId": "new-acc-1"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "accounts", "create", "--name", "NewCo"])
        .assert()
        .success()
        .stdout(predicate::str::contains("new-acc-1"));
}

#[tokio::test]
async fn accounts_update() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("PUT"))
        .and(path("/v1/accounts/acc-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "accounts", "update", "acc-123", "--fields", r#"{"name":"Updated"}"#])
        .assert()
        .success();
}

// ============================================================
// Subscriptions
// ============================================================

#[tokio::test]
async fn subscriptions_list() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .and(body_string_contains("Subscription"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 1,
            "records": [{"Id": "sub-1", "Name": "S-001", "Status": "Active", "TermStartDate": "2024-01-01", "TermEndDate": "2025-01-01"}]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "subscriptions", "list", "--account", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("S-001"));
}

#[tokio::test]
async fn subscriptions_get() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/subscriptions/sub-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "sub-1", "subscriptionNumber": "S-001", "status": "Active"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "subscriptions", "get", "sub-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("S-001"));
}

#[tokio::test]
async fn subscriptions_cancel() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("PUT"))
        .and(path("/v1/subscriptions/sub-1/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true, "subscriptionId": "sub-1"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "subscriptions", "cancel", "sub-1"])
        .assert()
        .success();
}

// ============================================================
// Invoices
// ============================================================

#[tokio::test]
async fn invoices_list() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .and(body_string_contains("Invoice"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 1,
            "records": [{"Id": "inv-1", "InvoiceNumber": "INV-001", "InvoiceDate": "2024-06-01", "DueDate": "2024-07-01", "Amount": 500.0, "Balance": 500.0, "Status": "Posted"}]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "invoices", "list", "--account", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("INV-001"));
}

#[tokio::test]
async fn invoices_get() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/invoices/inv-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "inv-1", "invoiceNumber": "INV-001", "amount": 500.0
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "invoices", "get", "inv-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("INV-001"));
}

#[tokio::test]
async fn invoices_pdf_download() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    // Return fake PDF bytes
    Mock::given(method("GET"))
        .and(path("/v1/invoices/inv-1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"%PDF-1.4 fake content".to_vec()))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    let output_path = dir.path().join("test-invoice.pdf");
    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "invoices", "pdf", "inv-1", "--output-file", output_path.to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("Saved invoice PDF"));

    let content = std::fs::read(&output_path).unwrap();
    assert!(content.starts_with(b"%PDF"));
}

// ============================================================
// Payments
// ============================================================

#[tokio::test]
async fn payments_list() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .and(body_string_contains("Payment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 1,
            "records": [{"Id": "pay-1", "PaymentNumber": "P-001", "Amount": 100.0, "EffectiveDate": "2024-06-01", "Status": "Processed", "GatewayResponse": "Approved"}]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "payments", "list", "--account", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("P-001"));
}

#[tokio::test]
async fn payments_create() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/payments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true, "id": "pay-new"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "payments", "create", "--account", "acc-1", "--amount", "50.00", "--payment-method", "pm-1"])
        .assert()
        .success();
}

// ============================================================
// Payment Methods
// ============================================================

#[tokio::test]
async fn payment_methods_list() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .and(body_string_contains("PaymentMethod"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 1,
            "records": [{"Id": "pm-1", "Type": "CreditCard", "CreditCardMaskNumber": "****1234", "BankName": null, "PaymentMethodStatus": "Active"}]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "payment-methods", "list", "--account", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CreditCard"));
}

#[tokio::test]
async fn payment_methods_get() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/object/payment-method/pm-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Id": "pm-1", "Type": "ACH", "BankName": "Wells Fargo"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "payment-methods", "get", "pm-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Wells Fargo"));
}

#[tokio::test]
async fn payment_methods_delete() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("DELETE"))
        .and(path("/v1/payment-methods/pm-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "payment-methods", "delete", "pm-1"])
        .assert()
        .success();
}

// ============================================================
// Orders
// ============================================================

#[tokio::test]
async fn orders_list() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/orders/subscriptionOwner/A-001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orders": [
                {"orderNumber": "O-001", "orderDate": "2024-06-01", "status": "Completed", "accountNumber": "A-001", "description": "New sub"}
            ]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "orders", "list", "--account", "A-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("O-001"));
}

#[tokio::test]
async fn orders_get() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/orders/O-001"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orderNumber": "O-001", "status": "Completed"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "orders", "get", "O-001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Completed"));
}

#[tokio::test]
async fn orders_create_from_file() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/orders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true, "orderNumber": "O-NEW"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    // Write order JSON to a file
    let order_file = dir.path().join("order.json");
    std::fs::write(&order_file, r#"{"existingAccountNumber": "A-001", "orderDate": "2024-06-01", "subscriptions": []}"#).unwrap();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "orders", "create", "--file", order_file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("O-NEW"));
}

#[tokio::test]
async fn orders_cancel() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("PUT"))
        .and(path("/v1/orders/O-001/cancel"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "orders", "cancel", "O-001"])
        .assert()
        .success();
}

// ============================================================
// Credit Memos
// ============================================================

#[tokio::test]
async fn credit_memos_list() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path_regex("/v1/credit-memos.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "creditMemos": [
                {"id": "cm-1", "number": "CM-001", "amount": 50.0, "balance": 50.0, "status": "Draft", "reasonCode": "Goodwill"}
            ]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "credit-memos", "list", "--account", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CM-001"));
}

#[tokio::test]
async fn credit_memos_create() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/credit-memos"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true, "id": "cm-new"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "credit-memos", "create", "--account", "acc-1", "--amount", "25.00", "--reason", "Billing Error"])
        .assert()
        .success();
}

// ============================================================
// Refunds
// ============================================================

#[tokio::test]
async fn refunds_list() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path_regex("/v1/refunds.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "refunds": [
                {"id": "ref-1", "number": "R-001", "amount": 25.0, "status": "Processed", "refundDate": "2024-06-15"}
            ]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "refunds", "list", "--account", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("R-001"));
}

#[tokio::test]
async fn refunds_create() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/refunds"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true, "id": "ref-new"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "refunds", "create", "--payment", "pay-1", "--amount", "25.00"])
        .assert()
        .success();
}

// ============================================================
// Error handling
// ============================================================

#[tokio::test]
async fn api_error_shows_zuora_message() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/accounts/bad-id"))
        .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
            "success": false,
            "reasons": [{"code": "OBJECT_NOT_FOUND", "message": "Account not found"}]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "accounts", "get", "bad-id"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Account not found"));
}

#[tokio::test]
async fn rate_limit_shows_retry_message() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "10")
                .set_body_string("rate limited")
        )
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "query", "SELECT Id FROM Account"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Rate limited"));
}

#[tokio::test]
async fn no_credentials_shows_helpful_error() {
    let dir = tempfile::tempdir().unwrap();
    // No login, no env vars
    zuora()
        .env("HOME", dir.path())
        .env_remove("ZUORA_CLIENT_ID")
        .env_remove("ZUORA_CLIENT_SECRET")
        .args(["query", "SELECT Id FROM Account"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("zuora login").or(predicate::str::contains("ZUORA_CLIENT_ID")));
}

// ============================================================
// Verbose mode
// ============================================================

#[tokio::test]
async fn verbose_shows_request_details() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 0, "records": []
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--verbose", "query", "SELECT Id FROM Account"])
        .assert()
        .success()
        .stderr(predicate::str::contains("POST"));
}

// ============================================================
// Multi-profile support
// ============================================================

#[tokio::test]
async fn multi_profile_isolation() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    let dir = tempfile::tempdir().unwrap();

    // Login with default profile
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "prod-id", "--client-secret", "prod-secret", "--base-url", &server.uri()])
        .assert().success();

    // Login with staging profile
    zuora()
        .env("HOME", dir.path())
        .args(["--profile", "staging", "login", "--client-id", "stg-id", "--client-secret", "stg-secret", "--base-url", &server.uri()])
        .assert().success();

    // Verify default profile
    zuora()
        .env("HOME", dir.path())
        .args(["config", "get", "client_id"])
        .assert()
        .success()
        .stdout(predicate::str::contains("prod-id"));

    // Verify staging profile
    zuora()
        .env("HOME", dir.path())
        .args(["--profile", "staging", "config", "get", "client_id"])
        .assert()
        .success()
        .stdout(predicate::str::contains("stg-id"));
}

// ============================================================
// Whoami
// ============================================================

#[tokio::test]
async fn whoami_shows_profile_info() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .arg("whoami")
        .assert()
        .success()
        .stdout(predicate::str::contains("Profile"))
        .stdout(predicate::str::contains("default"))
        .stdout(predicate::str::contains("valid"));
}

#[tokio::test]
async fn whoami_no_profile_shows_guidance() {
    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .arg("whoami")
        .assert()
        .success()
        .stdout(predicate::str::contains("zuora login"));
}

// ============================================================
// Status
// ============================================================

#[tokio::test]
async fn status_reports_reachable() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path_regex("/v1/catalog/products.*"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "products": []
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("reachable"));
}

// ============================================================
// Dry Run
// ============================================================

#[tokio::test]
async fn dry_run_does_not_send_mutation() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    // Do NOT mount a POST /v1/accounts handler — if dry-run leaks, it will 404
    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--dry-run", "accounts", "create", "--name", "DryRunCo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("DRY RUN"))
        .stderr(predicate::str::contains("DryRunCo"));
}

#[tokio::test]
async fn dry_run_allows_reads() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/accounts/acc-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "acc-1", "name": "ReadOnly"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    // --dry-run should still allow GET requests
    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--dry-run", "--output", "json", "accounts", "get", "acc-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ReadOnly"));
}

#[tokio::test]
async fn dry_run_payments_create() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--dry-run", "payments", "create", "--account", "a1", "--amount", "99.99", "--payment-method", "pm-1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("DRY RUN"))
        .stderr(predicate::str::contains("99.99"));
}

#[tokio::test]
async fn dry_run_order_cancel() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--dry-run", "orders", "cancel", "O-001"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("DRY RUN"));
}

// ============================================================
// Coverage: payment-methods create/update
// ============================================================

#[tokio::test]
async fn payment_methods_create() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/payment-methods"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true, "id": "pm-new"
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "payment-methods", "create", "--account", "acc-1", "--body", r#"{"type":"CreditCard"}"#])
        .assert()
        .success()
        .stdout(predicate::str::contains("pm-new"));
}

#[tokio::test]
async fn payment_methods_update() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("PUT"))
        .and(path("/v1/object/payment-method/pm-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "payment-methods", "update", "pm-1", "--fields", r#"{"TokenId":"tok_new"}"#])
        .assert()
        .success();
}

// ============================================================
// Coverage: get commands for remaining resources
// ============================================================

#[tokio::test]
async fn payments_get() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/payments/pay-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "pay-1", "paymentNumber": "P-001", "amount": 100.0
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "payments", "get", "pay-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("P-001"));
}

#[tokio::test]
async fn credit_memos_get() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/credit-memos/cm-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "cm-1", "number": "CM-001", "amount": 50.0
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "credit-memos", "get", "cm-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("CM-001"));
}

#[tokio::test]
async fn refunds_get() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path("/v1/refunds/ref-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "ref-1", "number": "R-001", "amount": 25.0
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--output", "json", "refunds", "get", "ref-1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("R-001"));
}

// ============================================================
// Coverage: status error paths, verbose on mutation
// ============================================================

#[tokio::test]
async fn status_reports_auth_invalid() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("GET"))
        .and(path_regex("/v1/catalog/products.*"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "success": false, "reasons": [{"code": "UNAUTHORIZED", "message": "Invalid token"}]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("reachable"));
}

// ============================================================
// Coverage: config list masking, verbose on queries
// ============================================================

#[tokio::test]
async fn config_list_masks_secret() {
    let dir = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "my-client-id", "--client-secret", "super-long-secret-value", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["config", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("su...ue"))  // masked secret
        .stdout(predicate::str::contains("my-client-id"));
}

#[tokio::test]
async fn verbose_query_shows_body() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 0, "records": []
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "--verbose", "query", "SELECT Id FROM Account"])
        .assert()
        .success()
        .stderr(predicate::str::contains("POST"))
        .stderr(predicate::str::contains("queryString"));
}

// ============================================================
// Coverage: accounts update with bad JSON
// ============================================================

#[tokio::test]
async fn accounts_update_bad_json_errors() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "accounts", "update", "acc-1", "--fields", "not-json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid JSON"));
}

// ============================================================
// Coverage: env var auth (no login command)
// ============================================================

#[tokio::test]
async fn env_var_auth_without_login() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 1,
            "records": [{"Id": "env-acc"}]
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    // No login — use env vars directly
    zuora()
        .env("HOME", dir.path())
        .env("ZUORA_CLIENT_ID", "env-id")
        .env("ZUORA_CLIENT_SECRET", "env-secret")
        .env("ZUORA_BASE_URL", &server.uri())
        .args(["--output", "raw", "query", "SELECT Id FROM Account"])
        .assert()
        .success()
        .stdout(predicate::str::contains("env-acc"));
}

// ============================================================
// Coverage: invoices list with status + limit
// ============================================================

#[tokio::test]
async fn invoices_list_with_status_and_limit() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .and(body_string_contains("Posted"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 0, "records": []
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "invoices", "list", "--account", "a1", "--status", "Posted", "--limit", "3"])
        .assert()
        .success();
}

// ============================================================
// Coverage: subscriptions list with status filter
// ============================================================

#[tokio::test]
async fn subscriptions_list_with_status() {
    let server = MockServer::start().await;
    mount_oauth(&server).await;

    Mock::given(method("POST"))
        .and(path("/v1/action/query"))
        .and(body_string_contains("Active"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "done": true, "size": 0, "records": []
        })))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    zuora()
        .env("HOME", dir.path())
        .args(["login", "--client-id", "id", "--client-secret", "s", "--base-url", &server.uri()])
        .assert().success();

    zuora()
        .env("HOME", dir.path())
        .args(["--base-url", &server.uri(), "subscriptions", "list", "--account", "a1", "--status", "Active"])
        .assert()
        .success();
}
