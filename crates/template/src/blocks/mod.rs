//! Block rendering implementations
//!
//! This module contains implementations for rendering different block types.
//! The main rendering logic is in the `renderer` module, but this module
//! can be used for block-specific utilities.

// Re-export block types from schema
pub use crate::schema::{Block, FieldFormBlock, QRCodeBlock, TableBlock, TableColumn, TextBlock};

/// Trait for blocks that can provide their text content
pub trait TextContent {
    /// Get the text content (either static or bound)
    fn get_text(&self, data: &serde_json::Value) -> Option<String>;
}

impl TextContent for TextBlock {
    fn get_text(&self, data: &serde_json::Value) -> Option<String> {
        if let Some(bind) = &self.bind {
            crate::parser::resolve_binding(bind, data).map(crate::parser::value_to_string)
        } else {
            self.text.clone()
        }
    }
}

impl TextContent for FieldFormBlock {
    fn get_text(&self, data: &serde_json::Value) -> Option<String> {
        if let Some(bind) = &self.bind {
            crate::parser::resolve_binding(bind, data).map(crate::parser::value_to_string)
        } else {
            self.text.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Position;
    use serde_json::json;

    #[test]
    fn test_text_content_with_bind() {
        let block = TextBlock {
            id: None,
            bind: Some("$.name".to_string()),
            text: None,
            position: Position { x: 0.0, y: 0.0 },
            font: None,
            align: crate::schema::Align::Left,
            word_wrap: None,
            format: None,
            format_type: None,
            pages: None,
            enable: None,
        };

        let data = json!({ "name": "Test" });
        let text = block.get_text(&data);
        assert_eq!(text, Some("Test".to_string()));
    }

    #[test]
    fn test_text_content_static() {
        let block = TextBlock {
            id: None,
            bind: None,
            text: Some("Static text".to_string()),
            position: Position { x: 0.0, y: 0.0 },
            font: None,
            align: crate::schema::Align::Left,
            word_wrap: None,
            format: None,
            format_type: None,
            pages: None,
            enable: None,
        };

        let data = json!({});
        let text = block.get_text(&data);
        assert_eq!(text, Some("Static text".to_string()));
    }
}
