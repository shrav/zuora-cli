/// Integration tests for all new resource commands added from the OpenAPI spec.
/// Each test uses a real wiremock HTTP server — no mocks or stubs.
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

macro_rules! json_ok {
    ($($json:tt)+) => { ResponseTemplate::new(200).set_body_json(serde_json::json!($($json)+)) }
}

// === Catalog ===

#[tokio::test]
async fn catalog_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/catalog/products"))
        .respond_with(json_ok!({"products": [{"id": "p1", "name": "Pro Plan"}]}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "catalog", "list"])
        .assert().success().stdout(predicate::str::contains("Pro Plan"));
}

#[tokio::test]
async fn catalog_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/catalog/products/p1"))
        .respond_with(json_ok!({"id": "p1", "name": "Pro Plan"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "catalog", "get", "p1"])
        .assert().success().stdout(predicate::str::contains("Pro Plan"));
}

#[tokio::test]
async fn catalog_rate_plans() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/products/p1/product-rate-plans"))
        .respond_with(json_ok!({"productRatePlans": [{"id": "rp1", "name": "Monthly"}]}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "catalog", "rate-plans", "p1"])
        .assert().success().stdout(predicate::str::contains("Monthly"));
}

// === Contacts ===

#[tokio::test]
async fn contacts_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/contacts/ct-1"))
        .respond_with(json_ok!({"id": "ct-1", "firstName": "Jane"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "contacts", "get", "ct-1"])
        .assert().success().stdout(predicate::str::contains("Jane"));
}

#[tokio::test]
async fn contacts_create() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("POST")).and(path("/v1/contacts"))
        .respond_with(json_ok!({"success": true, "id": "ct-new"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "contacts", "create", "--body", r#"{"firstName":"Jane"}"#])
        .assert().success();
}

#[tokio::test]
async fn contacts_delete() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("DELETE")).and(path("/v1/contacts/ct-1"))
        .respond_with(json_ok!({"success": true}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "contacts", "delete", "ct-1"])
        .assert().success();
}

// === Debit Memos ===

#[tokio::test]
async fn debit_memos_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path_regex("/v1/debit-memos.*"))
        .respond_with(json_ok!({"debitMemos": [{"id": "dm-1", "number": "DM-001", "amount": 100.0, "balance": 100.0, "status": "Draft", "debitMemoDate": "2024-01-01"}]}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "debit-memos", "list", "--account", "a1"])
        .assert().success().stdout(predicate::str::contains("DM-001"));
}

#[tokio::test]
async fn debit_memos_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/debit-memos/dm-1"))
        .respond_with(json_ok!({"id": "dm-1", "number": "DM-001"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "debit-memos", "get", "dm-1"])
        .assert().success().stdout(predicate::str::contains("DM-001"));
}

// === Bill Runs ===

#[tokio::test]
async fn bill_runs_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/bill-runs/br-1"))
        .respond_with(json_ok!({"id": "br-1", "status": "Completed"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "bill-runs", "get", "br-1"])
        .assert().success().stdout(predicate::str::contains("Completed"));
}

// === Payment Runs ===

#[tokio::test]
async fn payment_runs_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/payment-runs"))
        .respond_with(json_ok!({"paymentRuns": [{"id": "pr-1", "status": "Completed", "targetDate": "2024-01-01", "createdDate": "2024-01-01"}]}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "payment-runs", "list"])
        .assert().success().stdout(predicate::str::contains("Completed"));
}

// === Payment Schedules ===

#[tokio::test]
async fn payment_schedules_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/payment-schedules"))
        .respond_with(json_ok!({"paymentSchedules": []}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "payment-schedules", "list"])
        .assert().success();
}

// === Usage ===

#[tokio::test]
async fn usage_query() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("POST")).and(path("/v1/action/query")).and(body_string_contains("Usage"))
        .respond_with(json_ok!({"done": true, "size": 0, "records": []}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "usage", "query", "--account", "a1"])
        .assert().success();
}

// === Files ===

#[tokio::test]
async fn files_download() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/files/f-1"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"file-content".to_vec()))
        .mount(&s).await;
    let out = d.path().join("downloaded.dat");
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "files", "download", "f-1", "-o", out.to_str().unwrap()])
        .assert().success();
    assert_eq!(std::fs::read(&out).unwrap(), b"file-content");
}

// === Accounting Codes ===

#[tokio::test]
async fn accounting_codes_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/accounting-codes"))
        .respond_with(json_ok!({"accountingCodes": [{"id": "ac-1", "name": "Revenue"}]}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "accounting-codes", "list"])
        .assert().success().stdout(predicate::str::contains("Revenue"));
}

#[tokio::test]
async fn accounting_codes_create() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("POST")).and(path("/v1/accounting-codes"))
        .respond_with(json_ok!({"success": true, "id": "ac-new"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "accounting-codes", "create", "--body", r#"{"name":"Deferred Revenue","type":"AccountsReceivable"}"#])
        .assert().success();
}

// === Accounting Periods ===

#[tokio::test]
async fn accounting_periods_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/accounting-periods"))
        .respond_with(json_ok!({"accountingPeriods": [{"id": "ap-1", "name": "Jan 2024"}]}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "accounting-periods", "list"])
        .assert().success().stdout(predicate::str::contains("Jan 2024"));
}

// === Journal Entries ===

#[tokio::test]
async fn journal_entries_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/journal-entries/JE-001"))
        .respond_with(json_ok!({"number": "JE-001", "status": "Created"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "journal-entries", "get", "JE-001"])
        .assert().success().stdout(predicate::str::contains("JE-001"));
}

// === Journal Runs ===

#[tokio::test]
async fn journal_runs_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/journal-runs/JR-001"))
        .respond_with(json_ok!({"number": "JR-001", "status": "Completed"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "journal-runs", "get", "JR-001"])
        .assert().success().stdout(predicate::str::contains("Completed"));
}

// === Taxation Items ===

#[tokio::test]
async fn taxation_items_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/taxation-items/ti-1"))
        .respond_with(json_ok!({"id": "ti-1", "taxAmount": 5.0}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "taxation-items", "get", "ti-1"])
        .assert().success().stdout(predicate::str::contains("5.0"));
}

// === Adjustments ===

#[tokio::test]
async fn adjustments_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/adjustments"))
        .respond_with(json_ok!({"adjustments": []}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "adjustments", "list"])
        .assert().success();
}

// === Describe ===

#[tokio::test]
async fn describe_object() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/describe/Account"))
        .respond_with(json_ok!({"name": "Account", "fields": [{"name": "Id"}]}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "describe", "Account"])
        .assert().success().stdout(predicate::str::contains("Account"));
}

// === Billing Documents ===

#[tokio::test]
async fn billing_documents_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path_regex("/v1/billing-documents.*"))
        .respond_with(json_ok!({"billingDocuments": []}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "billing-documents", "--account", "a1"])
        .assert().success();
}

// === Billing Preview ===

#[tokio::test]
async fn billing_preview_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/billing-preview-runs/bp-1"))
        .respond_with(json_ok!({"id": "bp-1", "status": "Completed"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "billing-preview", "get", "bp-1"])
        .assert().success().stdout(predicate::str::contains("Completed"));
}

// === Operations ===

#[tokio::test]
async fn operations_job_status() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/operations/jobs/job-1"))
        .respond_with(json_ok!({"id": "job-1", "status": "Completed"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "operations", "job-status", "job-1"])
        .assert().success().stdout(predicate::str::contains("Completed"));
}

// === Fulfillments ===

#[tokio::test]
async fn fulfillments_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/fulfillments/f-1"))
        .respond_with(json_ok!({"id": "f-1", "state": "SentToBilling"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "fulfillments", "get", "f-1"])
        .assert().success().stdout(predicate::str::contains("SentToBilling"));
}

// === Order Line Items ===

#[tokio::test]
async fn order_line_items_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/order-line-items/oli-1"))
        .respond_with(json_ok!({"id": "oli-1", "itemName": "Widget"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "order-line-items", "get", "oli-1"])
        .assert().success().stdout(predicate::str::contains("Widget"));
}

// === Subscription Change Logs ===

#[tokio::test]
async fn subscription_change_logs_by_sub() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/subscription-change-logs/S-001"))
        .respond_with(json_ok!({"changeLogs": [{"type": "NewSubscription"}]}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "subscription-change-logs", "by-subscription", "S-001"])
        .assert().success().stdout(predicate::str::contains("NewSubscription"));
}

// === Ramps ===

#[tokio::test]
async fn ramps_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/ramps/R-001"))
        .respond_with(json_ok!({"rampNumber": "R-001", "name": "Growth Ramp"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "ramps", "get", "R-001"])
        .assert().success().stdout(predicate::str::contains("Growth Ramp"));
}

// === Notifications ===

#[tokio::test]
async fn notifications_callouts() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/notification-history/callout"))
        .respond_with(json_ok!({"calloutHistories": []}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "notifications", "callouts"])
        .assert().success();
}

// === Sequence Sets ===

#[tokio::test]
async fn sequence_sets_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/sequence-sets"))
        .respond_with(json_ok!({"sequenceSets": [{"id": "ss-1", "name": "Default"}]}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "sequence-sets", "list"])
        .assert().success().stdout(predicate::str::contains("Default"));
}

// === Exchange Rates ===

#[tokio::test]
async fn exchange_rates_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/custom-exchange-rates/EUR"))
        .respond_with(json_ok!({"currency": "EUR", "rates": []}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "exchange-rates", "EUR"])
        .assert().success().stdout(predicate::str::contains("EUR"));
}

// === Commitments ===

#[tokio::test]
async fn commitments_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/commitments"))
        .respond_with(json_ok!({"commitments": []}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "commitments", "list"])
        .assert().success();
}

// === Catalog Groups ===

#[tokio::test]
async fn catalog_groups_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/catalog-groups"))
        .respond_with(json_ok!({"catalogGroups": [{"id": "cg-1", "name": "Enterprise"}]}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "catalog-groups", "list"])
        .assert().success().stdout(predicate::str::contains("Enterprise"));
}

// === Attachments ===

#[tokio::test]
async fn attachments_list() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/attachments/Account/a1"))
        .respond_with(json_ok!({"attachments": []}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "attachments", "list", "Account", "a1"])
        .assert().success();
}

// === Invoice Schedules ===

#[tokio::test]
async fn invoice_schedules_get() {
    let s = MockServer::start().await;
    let d = setup(&s).await;
    Mock::given(method("GET")).and(path("/v1/invoice-schedules/is-1"))
        .respond_with(json_ok!({"id": "is-1", "status": "Active"}))
        .mount(&s).await;
    zuora().env("HOME", d.path()).args(["--base-url", &s.uri(), "--output", "json", "invoice-schedules", "get", "is-1"])
        .assert().success().stdout(predicate::str::contains("Active"));
}
