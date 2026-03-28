#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Generic wrapper for Zuora REST API list responses
#[derive(Debug, Deserialize)]
pub struct ZuoraListResponse<T> {
    #[serde(default)]
    pub success: Option<bool>,
    #[serde(default)]
    pub records: Option<Vec<T>>,
    /// Used by some endpoints instead of `records`
    #[serde(default, alias = "invoices", alias = "payments", alias = "creditMemos")]
    pub data: Option<Vec<T>>,
    #[serde(default, rename = "nextPage")]
    pub next_page: Option<String>,
}

impl<T> ZuoraListResponse<T> {
    pub fn into_items(self) -> Vec<T> {
        self.records
            .or(self.data)
            .unwrap_or_default()
    }
}

/// ZOQL query response
#[derive(Debug, Deserialize)]
pub struct QueryResponse {
    pub done: Option<bool>,
    pub size: Option<i64>,
    pub records: Option<Vec<serde_json::Value>>,
    #[serde(rename = "queryLocator")]
    pub query_locator: Option<String>,
}

// Error types moved to client/error.rs — unified parser handles all 5 Zuora formats

/// OAuth token response
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// Cached token stored on disk
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CachedToken {
    pub access_token: String,
    pub expires_at: i64,
    pub profile: String,
}

// --- Resource types ---

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Account {
    pub id: Option<String>,
    pub name: Option<String>,
    pub account_number: Option<String>,
    pub status: Option<String>,
    pub balance: Option<f64>,
    pub currency: Option<String>,
    #[serde(rename = "DefaultPaymentMethodId")]
    pub default_payment_method_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Subscription {
    pub id: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub account_id: Option<String>,
    pub term_start_date: Option<String>,
    pub term_end_date: Option<String>,
    pub contract_effective_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Invoice {
    pub id: Option<String>,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<String>,
    pub due_date: Option<String>,
    pub amount: Option<f64>,
    pub balance: Option<f64>,
    pub status: Option<String>,
    pub account_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Payment {
    pub id: Option<String>,
    pub payment_number: Option<String>,
    pub amount: Option<f64>,
    pub effective_date: Option<String>,
    pub status: Option<String>,
    pub account_id: Option<String>,
    pub payment_method_id: Option<String>,
    pub gateway_response: Option<String>,
    pub gateway_response_code: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PaymentMethod {
    pub id: Option<String>,
    #[serde(rename = "Type")]
    pub method_type: Option<String>,
    pub credit_card_mask_number: Option<String>,
    pub credit_card_expiration_month: Option<i32>,
    pub credit_card_expiration_year: Option<i32>,
    pub bank_name: Option<String>,
    #[serde(rename = "PaymentMethodStatus")]
    pub payment_method_status: Option<String>,
    pub account_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub order_number: Option<String>,
    pub order_date: Option<String>,
    pub status: Option<String>,
    pub account_number: Option<String>,
    pub description: Option<String>,
    pub created_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreditMemo {
    pub id: Option<String>,
    #[serde(rename = "CreditMemoNumber")]
    pub credit_memo_number: Option<String>,
    pub amount: Option<f64>,
    pub balance: Option<f64>,
    pub status: Option<String>,
    pub reason_code: Option<String>,
    pub account_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Refund {
    pub id: Option<String>,
    pub refund_number: Option<String>,
    pub amount: Option<f64>,
    pub status: Option<String>,
    pub refund_date: Option<String>,
    pub payment_id: Option<String>,
    pub account_id: Option<String>,
}
