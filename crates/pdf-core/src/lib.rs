//! PDF Core - Low-level PDF manipulation
//!
//! This crate provides functionality for:
//! - Opening and saving PDF documents
//! - Embedding TrueType fonts
//! - Inserting text at specific coordinates
//! - Inserting images (JPEG, PNG)
//!
//! # Example
//!
//! ```ignore
//! use pdf_core::{PdfDocument, Align};
//!
//! let mut doc = PdfDocument::open("template.pdf")?;
//! doc.add_font("sarabun", include_bytes!("fonts/THSarabunNew.ttf"))?;
//! doc.set_font("sarabun", 12)?;
//! doc.insert_text("Hello, World!", 1, 100.0, 700.0, Align::Left)?;
//! doc.save("output.pdf")?;
//! ```

mod document;
mod font;
mod image;
mod text;

pub use document::{Color, PdfDocument};
pub use font::{FontData, FontFamily, FontFamilyBuilder, FontStyle, FontWeight};
pub use image::ImageScaleMode;
pub use text::{generate_text_operators, simple_word_wrap, TextRenderContext};

use thiserror::Error;

/// Errors that can occur during PDF operations
#[derive(Debug, Error)]
pub enum PdfError {
    #[error("Failed to open PDF: {0}")]
    OpenError(String),

    #[error("Failed to save PDF: {0}")]
    SaveError(String),

    #[error("Font not found: {0}")]
    FontNotFound(String),

    #[error("Font already exists: {0}")]
    FontAlreadyExists(String),

    #[error("Failed to parse font: {0}")]
    FontParseError(String),

    #[error("Font subset error: {0}")]
    FontSubsetError(String),

    #[error("Invalid page number: {0} (document has {1} pages)")]
    InvalidPage(usize, usize),

    #[error("Image error: {0}")]
    ImageError(String),

    #[error("PDF parsing error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Lopdf error: {0}")]
    LopdfError(#[from] lopdf::Error),
}

/// Result type for PDF operations
pub type Result<T> = std::result::Result<T, PdfError>;

/// Text alignment options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// Position constants for alignment (matching original Go implementation)
pub mod position {
    /// Left alignment
    pub const LEFT: i32 = 8; // 001000
    /// Top alignment
    pub const TOP: i32 = 4; // 000100
    /// Right alignment  
    pub const RIGHT: i32 = 2; // 000010
    /// Bottom alignment
    pub const BOTTOM: i32 = 1; // 000001
    /// Center alignment
    pub const CENTER: i32 = 16; // 010000
    /// Middle alignment
    pub const MIDDLE: i32 = 32; // 100000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_default() {
        assert_eq!(Align::default(), Align::Left);
    }

    #[test]
    fn test_position_constants() {
        // Verify position constants match expected values
        assert_eq!(position::LEFT, 8);
        assert_eq!(position::RIGHT, 2);
        assert_eq!(position::CENTER, 16);
    }
}
