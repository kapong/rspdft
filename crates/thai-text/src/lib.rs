//! Thai Text - Thai language text processing
//!
//! This crate provides:
//! - Thai word segmentation (dictionary-based longest matching)
//! - Thai-aware line breaking
//! - Thai number formatting (หนึ่ง, สอง, สาม...)
//! - Thai currency formatting (บาท, สตางค์)
//! - Thai date formatting (Buddhist calendar)
//!
//! # Example
//!
//! ```ignore
//! use thai_text::{ThaiWordcut, ThaiFormatter};
//!
//! // Word segmentation
//! let wordcut = ThaiWordcut::from_file("dict.txt")?;
//! let words = wordcut.segment("สวัสดีครับ");
//!
//! // Word wrapping
//! let lines = wordcut.word_wrap("ข้อความยาวๆ ที่ต้องการตัดบรรทัด", 20);
//!
//! // Number formatting
//! let text = ThaiFormatter::format_number(42);  // "สี่สิบสอง"
//! let baht = ThaiFormatter::format_baht(100.50); // "หนึ่งร้อยบาทห้าสิบสตางค์"
//! ```

mod dictionary;
mod formatter;
mod linebreak;
mod wordcut;

pub use dictionary::{Dictionary, EMBEDDED_DICT};
pub use formatter::ThaiFormatter;
pub use wordcut::ThaiWordcut;

// Re-export commonly used formatting functions
pub use formatter::{
    format_thai_baht, format_thai_date_long, format_thai_date_short, format_thai_number,
    format_thai_year, render_float,
};

use thiserror::Error;

/// Errors that can occur during Thai text processing
#[derive(Debug, Error)]
pub enum ThaiTextError {
    #[error("Failed to load dictionary: {0}")]
    DictionaryLoadError(String),

    #[error("Invalid date: {0}")]
    InvalidDate(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for Thai text operations
pub type Result<T> = std::result::Result<T, ThaiTextError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_thai_number() {
        assert_eq!(format_thai_number(0), "ศูนย์");
        assert_eq!(format_thai_number(1), "หนึ่ง");
        assert_eq!(format_thai_number(10), "สิบ");
        assert_eq!(format_thai_number(11), "สิบเอ็ด");
        assert_eq!(format_thai_number(21), "ยี่สิบเอ็ด");
        assert_eq!(format_thai_number(100), "หนึ่งร้อย");
    }

    #[test]
    fn test_format_thai_baht() {
        assert_eq!(format_thai_baht(0.0), "-");
        assert_eq!(format_thai_baht(1.0), "หนึ่งบาทถ้วน");
        assert_eq!(format_thai_baht(0.50), "ห้าสิบสตางค์");
    }
}
