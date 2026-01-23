//! Template Engine - JSON template parsing and rendering
//!
//! This crate provides:
//! - Template JSON schema types
//! - Template parsing from JSON
//! - Block rendering (text, fieldform, table, qrcode)
//! - Data binding via JSONPath-like expressions
//!
//! # Example
//!
//! ```ignore
//! use template::{Template, TemplateRenderer};
//! use thai_text::ThaiWordcut;
//!
//! let wordcut = ThaiWordcut::from_file("dict.txt")?;
//! let template = Template::from_json(template_json)?;
//! let data: serde_json::Value = serde_json::from_str(data_json)?;
//! let pdf_bytes = template.render(&data, &wordcut)?;
//! ```

pub mod blocks;
pub mod parser;
mod renderer;
mod schema;

pub use parser::parse_template;
pub use renderer::TemplateRenderer;
pub use schema::*;

// Re-export the embedded schema
pub use schema::TEMPLATE_SCHEMA;

use thiserror::Error;

/// Errors that can occur during template processing
#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("Failed to parse template: {0}")]
    ParseError(String),

    #[error("Invalid data binding: {0}")]
    BindingError(String),

    #[error("Render error: {0}")]
    RenderError(String),

    #[error("Block error: {0}")]
    BlockError(String),

    #[error("PDF error: {0}")]
    PdfError(#[from] pdf_core::PdfError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Image error: {0}")]
    ImageError(String),

    #[error("Font error: {0}")]
    FontError(String),
}

/// Result type for template operations
pub type Result<T> = std::result::Result<T, TemplateError>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
