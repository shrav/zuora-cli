use colored::Colorize;
use comfy_table::{Cell, Table, presets::UTF8_FULL_CONDENSED};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Json,
    Table,
    Raw,
}

impl OutputFormat {
    pub fn from_str_opt(s: Option<&str>) -> Self {
        match s {
            Some("json") => Self::Json,
            Some("table") => Self::Table,
            Some("raw") => Self::Raw,
            _ => Self::Table,
        }
    }
}

/// Column definition for table output
pub struct ColumnDef {
    pub header: &'static str,
    pub json_path: &'static str,
}

/// Format a single JSON value for display
pub fn format_value(value: &serde_json::Value, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => {
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
        }
        OutputFormat::Raw => {
            serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
        }
        OutputFormat::Table => {
            // For single objects, display as key-value pairs
            if let Some(obj) = value.as_object() {
                let mut table = Table::new();
                table.load_preset(UTF8_FULL_CONDENSED);
                table.set_header(vec![
                    Cell::new("Field".bold().to_string()),
                    Cell::new("Value".bold().to_string()),
                ]);
                for (key, val) in obj {
                    let display_val = match val {
                        serde_json::Value::Null => "null".dimmed().to_string(),
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        other => other.to_string(),
                    };
                    table.add_row(vec![key.clone(), display_val]);
                }
                table.to_string()
            } else {
                serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
            }
        }
    }
}

/// Format a list of JSON objects as a table using column definitions
pub fn format_list_as_table(
    items: &[serde_json::Value],
    columns: &[ColumnDef],
) -> String {
    if items.is_empty() {
        return "No results found.".dimmed().to_string();
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);

    // Headers
    let headers: Vec<Cell> = columns
        .iter()
        .map(|c| Cell::new(c.header.bold().to_string()))
        .collect();
    table.set_header(headers);

    // Rows
    for item in items {
        let row: Vec<String> = columns
            .iter()
            .map(|col| extract_field(item, col.json_path))
            .collect();
        table.add_row(row);
    }

    table.to_string()
}

/// Format a list of items with auto-detected columns (for ZOQL queries)
pub fn format_auto_table(items: &[serde_json::Value]) -> String {
    if items.is_empty() {
        return "No results found.".dimmed().to_string();
    }

    // Detect columns from the first record's keys
    let first = &items[0];
    let keys: Vec<String> = if let Some(obj) = first.as_object() {
        obj.keys().cloned().collect()
    } else {
        return serde_json::to_string_pretty(items).unwrap_or_default();
    };

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);

    let headers: Vec<Cell> = keys
        .iter()
        .map(|k| Cell::new(k.bold().to_string()))
        .collect();
    table.set_header(headers);

    for item in items {
        let row: Vec<String> = keys
            .iter()
            .map(|k| extract_field(item, k))
            .collect();
        table.add_row(row);
    }

    table.to_string()
}

/// Format a JSON response for display
pub fn format_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}

/// Extract a nested field value as a display string
pub(crate) fn extract_field(value: &serde_json::Value, path: &str) -> String {
    let mut current = value;
    for segment in path.split('.') {
        current = match current.get(segment) {
            Some(v) => v,
            None => return String::new(),
        };
    }
    match current {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- OutputFormat ---

    #[test]
    fn output_format_from_str_opt() {
        assert_eq!(OutputFormat::from_str_opt(Some("json")), OutputFormat::Json);
        assert_eq!(OutputFormat::from_str_opt(Some("table")), OutputFormat::Table);
        assert_eq!(OutputFormat::from_str_opt(Some("raw")), OutputFormat::Raw);
        assert_eq!(OutputFormat::from_str_opt(None), OutputFormat::Table);
        assert_eq!(OutputFormat::from_str_opt(Some("garbage")), OutputFormat::Table);
    }

    // --- extract_field ---

    #[test]
    fn extract_field_string() {
        let val = json!({"name": "Acme Corp"});
        assert_eq!(extract_field(&val, "name"), "Acme Corp");
    }

    #[test]
    fn extract_field_number() {
        let val = json!({"balance": 42.5});
        assert_eq!(extract_field(&val, "balance"), "42.5");
    }

    #[test]
    fn extract_field_bool() {
        let val = json!({"active": true});
        assert_eq!(extract_field(&val, "active"), "true");
    }

    #[test]
    fn extract_field_null_returns_empty() {
        let val = json!({"status": null});
        assert_eq!(extract_field(&val, "status"), "");
    }

    #[test]
    fn extract_field_missing_returns_empty() {
        let val = json!({"name": "Acme"});
        assert_eq!(extract_field(&val, "nonexistent"), "");
    }

    #[test]
    fn extract_field_nested_path() {
        let val = json!({"billing": {"address": {"city": "SF"}}});
        assert_eq!(extract_field(&val, "billing.address.city"), "SF");
    }

    #[test]
    fn extract_field_nested_path_missing_intermediate() {
        let val = json!({"billing": {"name": "test"}});
        assert_eq!(extract_field(&val, "billing.address.city"), "");
    }

    #[test]
    fn extract_field_array_value() {
        let val = json!({"tags": ["a", "b"]});
        let result = extract_field(&val, "tags");
        assert_eq!(result, r#"["a","b"]"#);
    }

    // --- format_value ---

    #[test]
    fn format_value_json_mode() {
        let val = json!({"id": "123", "name": "Acme"});
        let result = format_value(&val, OutputFormat::Json);
        assert!(result.contains("\"id\": \"123\""));
        assert!(result.contains("\"name\": \"Acme\""));
    }

    #[test]
    fn format_value_raw_mode_compact() {
        let val = json!({"id": "123"});
        let result = format_value(&val, OutputFormat::Raw);
        assert!(!result.contains('\n'));
        assert!(result.contains("\"id\":\"123\""));
    }

    #[test]
    fn format_value_table_mode_object() {
        let val = json!({"Name": "Acme", "Status": "Active"});
        let result = format_value(&val, OutputFormat::Table);
        assert!(result.contains("Name"));
        assert!(result.contains("Acme"));
        assert!(result.contains("Status"));
        assert!(result.contains("Active"));
    }

    #[test]
    fn format_value_table_mode_non_object_falls_back() {
        let val = json!("just a string");
        let result = format_value(&val, OutputFormat::Table);
        assert!(result.contains("just a string"));
    }

    #[test]
    fn format_value_table_handles_all_types() {
        let val = json!({
            "str": "hello",
            "num": 42,
            "bool_val": true,
            "null_val": null,
            "nested": {"a": 1}
        });
        let result = format_value(&val, OutputFormat::Table);
        assert!(result.contains("hello"));
        assert!(result.contains("42"));
        assert!(result.contains("true"));
    }

    // --- format_list_as_table ---

    #[test]
    fn format_list_as_table_with_data() {
        let items = vec![
            json!({"Id": "1", "Name": "Acme"}),
            json!({"Id": "2", "Name": "Beta"}),
        ];
        let cols = &[
            ColumnDef { header: "ID", json_path: "Id" },
            ColumnDef { header: "Name", json_path: "Name" },
        ];
        let result = format_list_as_table(&items, cols);
        assert!(result.contains("Acme"));
        assert!(result.contains("Beta"));
        assert!(result.contains("ID"));
        assert!(result.contains("Name"));
    }

    #[test]
    fn format_list_as_table_empty() {
        let cols = &[ColumnDef { header: "ID", json_path: "Id" }];
        let result = format_list_as_table(&[], cols);
        assert!(result.contains("No results"));
    }

    #[test]
    fn format_list_as_table_missing_fields() {
        let items = vec![json!({"Id": "1"})];
        let cols = &[
            ColumnDef { header: "ID", json_path: "Id" },
            ColumnDef { header: "Missing", json_path: "nope" },
        ];
        let result = format_list_as_table(&items, cols);
        assert!(result.contains("1"));
    }

    // --- format_auto_table ---

    #[test]
    fn format_auto_table_detects_columns() {
        let items = vec![
            json!({"Col1": "a", "Col2": "b"}),
            json!({"Col1": "c", "Col2": "d"}),
        ];
        let result = format_auto_table(&items);
        assert!(result.contains("Col1"));
        assert!(result.contains("Col2"));
        assert!(result.contains("a"));
        assert!(result.contains("d"));
    }

    #[test]
    fn format_auto_table_empty() {
        let result = format_auto_table(&[]);
        assert!(result.contains("No results"));
    }

    #[test]
    fn format_auto_table_non_objects_fallback() {
        let items = vec![json!("not an object")];
        let result = format_auto_table(&items);
        assert!(result.contains("not an object"));
    }

    // --- format_json ---

    #[test]
    fn format_json_pretty_prints() {
        let val = json!({"a": 1});
        let result = format_json(&val);
        assert!(result.contains('\n'));
        assert!(result.contains("\"a\": 1"));
    }
}
