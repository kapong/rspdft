//! Integration tests for template rendering

use serde_json::json;
use template::{parse_template, Block, FormatType};

#[test]
fn test_parse_simple_template() {
    let template_json = r#"{
        "version": "2.0",
        "template": {
            "source": "base64:..."
        },
        "fonts": [
            {
                "id": "sarabun",
                "source": "fonts/THSarabunNew.ttf"
            }
        ],
        "blocks": [
            {
                "type": "text",
                "bind": "$.customer.name",
                "position": { "x": 100, "y": 50 },
                "font": { "family": "sarabun", "size": 14 },
                "align": "left"
            }
        ]
    }"#;

    let template = parse_template(template_json).unwrap();

    assert_eq!(template.version, "2.0");
    assert_eq!(template.fonts.len(), 1);
    assert_eq!(template.blocks.len(), 1);
}

#[test]
fn test_parse_template_with_all_block_types() {
    let template_json = r#"{
        "version": "2.0",
        "template": { "source": "test.pdf" },
        "fonts": [],
        "blocks": [
            {
                "type": "text",
                "text": "Static text",
                "position": { "x": 100, "y": 50 }
            },
            {
                "type": "fieldform",
                "bind": "$.idNumber",
                "position": { "x": 100, "y": 100 },
                "charSpacing": [15, 15, 15, 15]
            },
            {
                "type": "table",
                "bind": "$.items",
                "position": { "x": 50, "y": 200 },
                "rowHeight": 15,
                "columns": [
                    { "field": "name", "x": 0, "align": "left" },
                    { "field": "price", "x": 200, "align": "right", "format": "numeric" }
                ]
            },
            {
                "type": "qrcode",
                "bind": "$.qr_code",
                "position": { "x": 400, "y": 50 },
                "size": { "width": 80, "height": 80 }
            }
        ]
    }"#;

    let template = parse_template(template_json).unwrap();
    assert_eq!(template.blocks.len(), 4);
}

#[test]
fn test_template_with_conditional_rendering() {
    let template_json = r#"{
        "version": "2.0",
        "template": { "source": "test.pdf" },
        "fonts": [],
        "blocks": [
            {
                "type": "text",
                "text": "Always visible",
                "position": { "x": 100, "y": 50 }
            },
            {
                "type": "text",
                "text": "Conditional text",
                "position": { "x": 100, "y": 100 },
                "enable": "$.showExtra"
            }
        ]
    }"#;

    let template = parse_template(template_json).unwrap();
    assert_eq!(template.blocks.len(), 2);

    // First block has no enable
    assert!(template.blocks[0].enable().is_none());

    // Second block has enable binding
    assert_eq!(template.blocks[1].enable(), Some("$.showExtra"));
}

#[test]
fn test_template_with_duplicate() {
    let template_json = r#"{
        "version": "2.0",
        "template": {
            "source": "test.pdf",
            "duplicate": { "x": 0, "y": 400 }
        },
        "fonts": [],
        "blocks": [
            {
                "type": "text",
                "text": "Receipt copy",
                "position": { "x": 100, "y": 50 }
            }
        ]
    }"#;

    let template = parse_template(template_json).unwrap();

    assert!(template.template.duplicate.is_some());
    let dup = template.template.duplicate.as_ref().unwrap();
    assert_eq!(dup.x, 0.0);
    assert_eq!(dup.y, 400.0);
}

#[test]
fn test_template_with_word_wrap() {
    let template_json = r#"{
        "version": "2.0",
        "template": { "source": "test.pdf" },
        "fonts": [],
        "blocks": [
            {
                "type": "text",
                "bind": "$.description",
                "position": { "x": 50, "y": 100 },
                "wordWrap": {
                    "maxChars": 40,
                    "lineHeight": 14
                }
            }
        ]
    }"#;

    let template = parse_template(template_json).unwrap();

    if let Block::Text(block) = &template.blocks[0] {
        assert!(block.word_wrap.is_some());
        let wrap = block.word_wrap.as_ref().unwrap();
        assert_eq!(wrap.max_chars, 40);
        assert_eq!(wrap.line_height, 14.0);
    } else {
        panic!("Expected TextBlock");
    }
}

#[test]
fn test_template_with_format_types() {
    let template_json = r#"{
        "version": "2.0",
        "template": { "source": "test.pdf" },
        "fonts": [],
        "blocks": [
            {
                "type": "text",
                "bind": "$.amount",
                "position": { "x": 100, "y": 50 },
                "formatType": "thai-baht"
            },
            {
                "type": "text",
                "bind": "$.date",
                "position": { "x": 100, "y": 100 },
                "formatType": "thai-date-long"
            }
        ]
    }"#;

    let template = parse_template(template_json).unwrap();

    if let Block::Text(block) = &template.blocks[0] {
        assert_eq!(block.format_type, Some(FormatType::ThaiBaht));
    }

    if let Block::Text(block) = &template.blocks[1] {
        assert_eq!(block.format_type, Some(FormatType::ThaiDateLong));
    }
}

#[test]
fn test_data_binding_resolution() {
    use template::parser::resolve_binding;

    let data = json!({
        "customer": {
            "name": "สมชาย",
            "address": {
                "city": "กรุงเทพ"
            }
        },
        "items": [
            { "name": "สินค้า 1", "price": 100 },
            { "name": "สินค้า 2", "price": 200 }
        ],
        "total": 300.50
    });

    // Simple field
    assert_eq!(
        resolve_binding("$.customer.name", &data),
        Some(&json!("สมชาย"))
    );

    // Nested field
    assert_eq!(
        resolve_binding("$.customer.address.city", &data),
        Some(&json!("กรุงเทพ"))
    );

    // Array access
    assert_eq!(
        resolve_binding("$.items[0].name", &data),
        Some(&json!("สินค้า 1"))
    );

    // Number field
    assert_eq!(resolve_binding("$.total", &data), Some(&json!(300.50)));

    // Missing field
    assert_eq!(resolve_binding("$.missing", &data), None);
}
