//! Template JSON parsing

use crate::{Result, Template, TemplateError};

/// Parse a template from JSON string
pub fn parse_template(json: &str) -> Result<Template> {
    serde_json::from_str(json).map_err(|e| TemplateError::ParseError(e.to_string()))
}

/// Resolve a JSONPath-like binding expression against data
///
/// Supports simple paths like:
/// - `$.field` - Root field
/// - `$.object.field` - Nested field
/// - `$.array[0]` - Array index
/// - `$.array[0].field` - Array element field
pub fn resolve_binding<'a>(
    path: &str,
    data: &'a serde_json::Value,
) -> Option<&'a serde_json::Value> {
    if !path.starts_with("$.") {
        return None;
    }

    let path = &path[2..]; // Remove "$."
    let mut current = data;

    for segment in path.split('.') {
        // Check for array index
        if let Some(bracket_pos) = segment.find('[') {
            let field = &segment[..bracket_pos];
            let index_str = &segment[bracket_pos + 1..segment.len() - 1];
            let index: usize = index_str.parse().ok()?;

            if !field.is_empty() {
                current = current.get(field)?;
            }
            current = current.get(index)?;
        } else {
            current = current.get(segment)?;
        }
    }

    Some(current)
}

/// Convert a JSON value to string for rendering
pub fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolve_simple_field() {
        let data = json!({ "name": "John" });
        let result = resolve_binding("$.name", &data);
        assert_eq!(result, Some(&json!("John")));
    }

    #[test]
    fn test_resolve_nested_field() {
        let data = json!({
            "customer": {
                "name": "Jane"
            }
        });
        let result = resolve_binding("$.customer.name", &data);
        assert_eq!(result, Some(&json!("Jane")));
    }

    #[test]
    fn test_resolve_array_index() {
        let data = json!({
            "items": ["a", "b", "c"]
        });
        let result = resolve_binding("$.items[1]", &data);
        assert_eq!(result, Some(&json!("b")));
    }

    #[test]
    fn test_resolve_array_object() {
        let data = json!({
            "items": [
                { "name": "Item 1" },
                { "name": "Item 2" }
            ]
        });
        let result = resolve_binding("$.items[0].name", &data);
        assert_eq!(result, Some(&json!("Item 1")));
    }

    #[test]
    fn test_resolve_missing_field() {
        let data = json!({ "name": "John" });
        let result = resolve_binding("$.missing", &data);
        assert_eq!(result, None);
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(value_to_string(&json!("hello")), "hello");
        assert_eq!(value_to_string(&json!(42)), "42");
        assert_eq!(
            value_to_string(&json!(std::f64::consts::PI)),
            "3.141592653589793"
        );
        assert_eq!(value_to_string(&json!(true)), "true");
        assert_eq!(value_to_string(&json!(null)), "");
    }

    #[test]
    fn test_parse_template() {
        let json = r#"{
            "version": "2.0",
            "template": {
                "source": "test.pdf"
            },
            "fonts": [],
            "blocks": []
        }"#;

        let template = parse_template(json).unwrap();
        assert_eq!(template.version, "2.0");
        assert_eq!(template.template.source, "test.pdf");
    }
}
