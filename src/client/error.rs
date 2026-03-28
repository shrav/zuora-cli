use serde::Deserialize;

/// Unified error parser for all 5 Zuora API error response formats.
///
/// Format 1 — REST API (CommonResponse / ErrorResponse):
///   `{ "success": false, "reasons": [{ "code": "...", "message": "..." }] }`
///
/// Format 2 — CRUD Proxy API (PascalCase):
///   `{ "Success": false, "Errors": [{ "Code": "...", "Message": "..." }] }`
///
/// Format 3 — Action API v1 (nested faults array):
///   `{ "faults": [{ "faultCode": "...", "faultMessage": "..." }] }`
///
/// Format 4 — Action API v2 (top-level FaultCode):
///   `{ "FaultCode": "MALFORMED_QUERY", "FaultMessage": "..." }`
///
/// Format 5 — ProxyUnauthorizedResponse:
///   `{ "message": "Authentication error" }`
pub fn parse_error_body(body: &str) -> String {
    if body.is_empty() {
        return "(empty response)".to_string();
    }

    if let Ok(err) = serde_json::from_str::<ZuoraUnifiedError>(body) {
        // Format 1: REST API reasons (lowercase)
        if let Some(ref reasons) = err.reasons {
            if !reasons.is_empty() {
                let msg = format_reasons(reasons);
                return with_hint(&msg, reasons);
            }
        }

        // Format 2: CRUD Proxy Errors (PascalCase)
        if let Some(ref errors) = err.errors {
            if !errors.is_empty() {
                return errors.iter().map(|e| {
                    let code = e.code.as_deref().unwrap_or("UNKNOWN");
                    let msg = e.message.as_deref().unwrap_or("No message");
                    format!("[{code}] {msg}")
                }).collect::<Vec<_>>().join("; ");
            }
        }

        // Format 3: Action API faults array
        if let Some(ref faults) = err.faults {
            if !faults.is_empty() {
                return faults.iter().map(|f| {
                    let code = f.fault_code.as_deref().unwrap_or("UNKNOWN");
                    let msg = f.fault_message.as_deref().unwrap_or("No message");
                    format!("[{code}] {msg}")
                }).collect::<Vec<_>>().join("; ");
            }
        }

        // Format 4: Top-level FaultCode/FaultMessage
        if let Some(ref code) = err.fault_code {
            let msg = err.fault_message.as_deref().unwrap_or("No message");
            let formatted = format!("[{code}] {msg}");
            return with_zoql_hint(&formatted, code);
        }

        // Format 5: Simple message field
        if let Some(ref message) = err.message {
            return with_auth_hint(message);
        }

        // Has processId or requestId but no error details — still useful
        if let Some(ref process_id) = err.process_id {
            return format!("Request failed (processId: {process_id})");
        }
    }

    // Not valid JSON or unrecognized structure — return as-is, truncated
    if body.len() > 500 {
        format!("{}... (truncated)", &body[..500])
    } else {
        body.to_string()
    }
}

/// All possible fields across Zuora's error formats, merged into one struct
#[derive(Debug, Deserialize)]
struct ZuoraUnifiedError {
    // Format 1: REST API
    reasons: Option<Vec<Reason>>,
    // Format 2: CRUD Proxy
    #[serde(rename = "Errors")]
    errors: Option<Vec<ProxyError>>,
    // Format 3: Action API faults array
    faults: Option<Vec<Fault>>,
    // Format 4: Action API top-level fault
    #[serde(rename = "FaultCode")]
    fault_code: Option<String>,
    #[serde(rename = "FaultMessage")]
    fault_message: Option<String>,
    // Format 5: Simple message
    message: Option<String>,
    // Metadata
    #[serde(rename = "processId")]
    process_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Reason {
    code: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProxyError {
    #[serde(rename = "Code")]
    code: Option<String>,
    #[serde(rename = "Message")]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Fault {
    #[serde(rename = "faultCode")]
    fault_code: Option<String>,
    #[serde(rename = "faultMessage")]
    fault_message: Option<String>,
}

fn format_reasons(reasons: &[Reason]) -> String {
    reasons.iter().map(|r| {
        let code = r.code.as_deref().unwrap_or("UNKNOWN");
        let msg = r.message.as_deref().unwrap_or("No message");
        format!("[{code}] {msg}")
    }).collect::<Vec<_>>().join("; ")
}

/// Add actionable hints for common REST API error codes
fn with_hint(msg: &str, reasons: &[Reason]) -> String {
    let code = reasons.first().and_then(|r| r.code.as_deref()).unwrap_or("");
    let hint = match code {
        "INVALID_VALUE" => Some("Check that the ID or key you provided exists and is correctly formatted."),
        "MISSING_REQUIRED_VALUE" => Some("A required field is missing. Use --output json to see full details, or check the API docs."),
        "REQUEST_REJECTED" => Some("The request was rejected by Zuora's business rules. Check field values and account state."),
        "TRANSACTION_FAILED" => Some("The transaction could not be completed. Check the account and payment method status."),
        "LOCK_COMPETITION" => Some("Another operation is in progress on this object. Wait and retry."),
        "OBJECT_NOT_FOUND" | "ObjectNotFound" => Some("The requested object was not found. Verify the ID or key."),
        "API_DISABLED" => Some("This API feature is not enabled for your tenant. Contact Zuora support."),
        "CANNOT_DELETE" => Some("This object cannot be deleted in its current state."),
        _ => None,
    };
    match hint {
        Some(h) => format!("{msg}\n  Hint: {h}"),
        None => msg.to_string(),
    }
}

/// Add hints for ZOQL-specific errors
fn with_zoql_hint(msg: &str, code: &str) -> String {
    let hint = match code {
        "MALFORMED_QUERY" => Some("Check your ZOQL syntax. Common issues: unsupported ORDER BY fields, missing quotes around string values, or invalid field names. Use `zuora describe <Object>` to see available fields."),
        "INVALID_FIELD" => Some("One or more fields in your query don't exist on this object. Use `zuora describe <Object>` to list valid fields."),
        "QUERY_TIMEOUT" => Some("The query took too long. Add filters (WHERE clause) or reduce the number of fields selected."),
        _ => None,
    };
    match hint {
        Some(h) => format!("{msg}\n  Hint: {h}"),
        None => msg.to_string(),
    }
}

/// Add hints for auth errors
fn with_auth_hint(message: &str) -> String {
    if message.contains("Authentication error") || message.contains("Unauthorized") {
        format!("{message}\n  Hint: Your token may have expired. Run `zuora login` to re-authenticate.")
    } else if message.contains("Failed to get user info") {
        format!("{message}\n  Hint: Transient error. Retry the request.")
    } else {
        message.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Format 1: REST API reasons
    #[test]
    fn parse_rest_api_error() {
        let body = r#"{"success":false,"reasons":[{"code":"INVALID_VALUE","message":"The account ID is invalid"}]}"#;
        let result = parse_error_body(body);
        assert!(result.contains("[INVALID_VALUE] The account ID is invalid"));
        assert!(result.contains("Hint:"));
    }

    #[test]
    fn parse_rest_api_multiple_reasons() {
        let body = r#"{"success":false,"reasons":[{"code":"ERR1","message":"First"},{"code":"ERR2","message":"Second"}]}"#;
        let result = parse_error_body(body);
        assert!(result.contains("[ERR1] First"));
        assert!(result.contains("[ERR2] Second"));
    }

    #[test]
    fn parse_object_not_found_has_hint() {
        let body = r#"{"success":false,"reasons":[{"code":"OBJECT_NOT_FOUND","message":"Account not found"}]}"#;
        let result = parse_error_body(body);
        assert!(result.contains("Hint:"));
        assert!(result.contains("Verify the ID"));
    }

    #[test]
    fn parse_missing_required_has_hint() {
        let body = r#"{"success":false,"reasons":[{"code":"MISSING_REQUIRED_VALUE","message":"name is required"}]}"#;
        let result = parse_error_body(body);
        assert!(result.contains("Hint:"));
    }

    // Format 2: CRUD Proxy (PascalCase)
    #[test]
    fn parse_proxy_error() {
        let body = r#"{"Success":false,"Errors":[{"Code":"INVALID_VALUE","Message":"The account number 123 is invalid."}]}"#;
        let result = parse_error_body(body);
        assert_eq!(result, "[INVALID_VALUE] The account number 123 is invalid.");
    }

    #[test]
    fn parse_proxy_error_multiple() {
        let body = r#"{"Success":false,"Errors":[{"Code":"A","Message":"first"},{"Code":"B","Message":"second"}]}"#;
        let result = parse_error_body(body);
        assert!(result.contains("[A] first"));
        assert!(result.contains("[B] second"));
    }

    // Format 3: Action API faults array
    #[test]
    fn parse_action_faults() {
        let body = r#"{"faults":[{"faultCode":"MALFORMED_QUERY","faultMessage":"syntax error"}]}"#;
        let result = parse_error_body(body);
        assert_eq!(result, "[MALFORMED_QUERY] syntax error");
    }

    // Format 4: Top-level FaultCode
    #[test]
    fn parse_top_level_fault() {
        let body = r#"{"FaultCode":"MALFORMED_QUERY","FaultMessage":"You have an error in your ZOQL syntax"}"#;
        let result = parse_error_body(body);
        assert!(result.contains("[MALFORMED_QUERY]"));
        assert!(result.contains("Hint:"), "ZOQL errors should have hints");
        assert!(result.contains("zuora describe"));
    }

    #[test]
    fn parse_invalid_field_fault() {
        let body = r#"{"FaultCode":"INVALID_FIELD","FaultMessage":"Field 'bogus' does not exist"}"#;
        let result = parse_error_body(body);
        assert!(result.contains("Hint:"));
        assert!(result.contains("zuora describe"));
    }

    // Format 5: Simple message
    #[test]
    fn parse_auth_error_message() {
        let body = r#"{"message":"Authentication error"}"#;
        let result = parse_error_body(body);
        assert!(result.contains("Authentication error"));
        assert!(result.contains("zuora login"));
    }

    #[test]
    fn parse_transient_error_message() {
        let body = r#"{"message":"Failed to get user info"}"#;
        let result = parse_error_body(body);
        assert!(result.contains("Retry"));
    }

    // Edge cases
    #[test]
    fn parse_empty_body() {
        assert_eq!(parse_error_body(""), "(empty response)");
    }

    #[test]
    fn parse_non_json() {
        assert_eq!(parse_error_body("Internal Server Error"), "Internal Server Error");
    }

    #[test]
    fn parse_json_no_error_fields() {
        let body = r#"{"success":false}"#;
        let result = parse_error_body(body);
        assert_eq!(result, r#"{"success":false}"#);
    }

    #[test]
    fn parse_with_process_id() {
        let body = r#"{"processId":"abc-123"}"#;
        let result = parse_error_body(body);
        assert!(result.contains("abc-123"));
    }

    #[test]
    fn parse_reasons_null_fields() {
        let body = r#"{"reasons":[{"code":null,"message":null}]}"#;
        let result = parse_error_body(body);
        assert!(result.contains("[UNKNOWN] No message"));
    }

    #[test]
    fn reasons_take_precedence_over_faults() {
        let body = r#"{"reasons":[{"code":"A","message":"from reasons"}],"faults":[{"faultCode":"B","faultMessage":"from faults"}]}"#;
        let result = parse_error_body(body);
        assert!(result.contains("[A] from reasons"));
    }

    #[test]
    fn long_body_truncated() {
        let body = "x".repeat(1000);
        let result = parse_error_body(&body);
        assert!(result.len() < 600);
        assert!(result.contains("truncated"));
    }
}
