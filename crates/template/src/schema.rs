//! Template JSON schema types

use serde::{Deserialize, Serialize};

/// Embedded JSON Schema for template validation
/// This schema can be used by IDEs and validators for template authoring
pub const TEMPLATE_SCHEMA: &str = include_str!("../data/template-schema.json");

/// Root template structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Schema version
    pub version: String,

    /// Template source configuration
    pub template: TemplateSource,

    /// Font definitions
    #[serde(default)]
    pub fonts: Vec<FontDef>,

    /// Content blocks
    pub blocks: Vec<Block>,
}

/// Template source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSource {
    /// Path to base PDF or base64-encoded data
    pub source: String,

    /// Duplicate blocks with offset (for duplicate receipts)
    #[serde(default)]
    pub duplicate: Option<Duplicate>,
}

/// Duplicate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duplicate {
    /// X offset for duplicated blocks
    #[serde(default)]
    pub x: f64,

    /// Y offset for duplicated blocks
    #[serde(default)]
    pub y: f64,
}

/// Font family definition (new format with variants)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontDef {
    /// Font family identifier (used in blocks)
    pub id: String,

    /// Font family display name (for documentation)
    #[serde(default)]
    pub family: Option<String>,

    /// Path to regular TTF file (legacy: single font source)
    #[serde(default)]
    pub source: Option<String>,

    /// Regular variant source
    #[serde(default)]
    pub regular: Option<String>,

    /// Bold variant source
    #[serde(default)]
    pub bold: Option<String>,

    /// Italic variant source
    #[serde(default)]
    pub italic: Option<String>,

    /// Bold-italic variant source
    #[serde(rename = "boldItalic")]
    #[serde(default)]
    pub bold_italic: Option<String>,

    /// Fallback font family IDs (for missing glyphs)
    #[serde(default)]
    pub fallback: Vec<String>,
}

/// Content block (tagged union)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Block {
    /// Text block
    Text(TextBlock),

    /// Field form block (character-by-character)
    #[serde(rename = "fieldform")]
    FieldForm(FieldFormBlock),

    /// Table block
    Table(TableBlock),

    /// QR code block
    #[serde(rename = "qrcode")]
    QRCode(QRCodeBlock),
}

/// Position in PDF coordinates
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    /// X coordinate in points
    pub x: f64,

    /// Y coordinate in points (from top)
    pub y: f64,
}

/// Font specification for a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Font {
    /// Font family ID
    pub family: String,

    /// Font size in points
    #[serde(default = "default_font_size")]
    pub size: u8,

    /// Font style
    #[serde(default)]
    pub style: FontStyle,
}

fn default_font_size() -> u8 {
    12
}

/// Font style
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    #[default]
    Regular,
    Bold,
    Italic,
    #[serde(rename = "bold-italic")]
    BoldItalic,
}

/// Text alignment
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    #[default]
    Left,
    Center,
    Right,
}

/// Word wrap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordWrap {
    /// Maximum characters per line
    #[serde(rename = "maxChars")]
    pub max_chars: usize,

    /// Line height in points
    #[serde(rename = "lineHeight")]
    pub line_height: f64,
}

/// Special format types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FormatType {
    Number,
    ThaiBaht,
    ThaiDateShort,
    ThaiDateLong,
    ThaiYear,
}

/// Text block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    /// Block identifier
    #[serde(default)]
    pub id: Option<String>,

    /// Data binding path (JSONPath-like)
    #[serde(default)]
    pub bind: Option<String>,

    /// Static text (used if bind is not specified)
    #[serde(default)]
    pub text: Option<String>,

    /// Position
    pub position: Position,

    /// Font specification
    #[serde(default)]
    pub font: Option<Font>,

    /// Text alignment
    #[serde(default)]
    pub align: Align,

    /// Word wrap configuration
    #[serde(rename = "wordWrap")]
    #[serde(default)]
    pub word_wrap: Option<WordWrap>,

    /// Number format pattern
    #[serde(default)]
    pub format: Option<String>,

    /// Special format type
    #[serde(rename = "formatType")]
    #[serde(default)]
    pub format_type: Option<FormatType>,

    /// Pages to render on (1-indexed)
    #[serde(default)]
    pub pages: Option<Vec<usize>>,

    /// Optional enable flag - if set, evaluates binding to determine if block is rendered
    /// If the bound value is falsy (null, false, 0, empty string), block is not rendered
    #[serde(default)]
    pub enable: Option<String>,
}

/// Field form block (character-by-character with spacing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldFormBlock {
    /// Block identifier
    #[serde(default)]
    pub id: Option<String>,

    /// Data binding path
    #[serde(default)]
    pub bind: Option<String>,

    /// Static text
    #[serde(default)]
    pub text: Option<String>,

    /// Position
    pub position: Position,

    /// Font specification
    #[serde(default)]
    pub font: Option<Font>,

    /// Character spacing (X offset for each character after the first)
    #[serde(rename = "charSpacing")]
    pub char_spacing: Vec<f64>,

    /// Pages to render on
    #[serde(default)]
    pub pages: Option<Vec<usize>>,

    /// Optional enable flag - if set, evaluates binding to determine if block is rendered
    /// If the bound value is falsy (null, false, 0, empty string), block is not rendered
    #[serde(default)]
    pub enable: Option<String>,
}

fn default_row_height() -> f64 {
    13.5
}

/// Table block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableBlock {
    /// Block identifier
    #[serde(default)]
    pub id: Option<String>,

    /// Data binding path (to array of rows)
    #[serde(default)]
    pub bind: Option<String>,

    /// Position
    pub position: Position,

    /// Font specification
    #[serde(default)]
    pub font: Option<Font>,

    /// Row height in points
    #[serde(rename = "rowHeight")]
    #[serde(default = "default_row_height")]
    pub row_height: f64,

    /// Column definitions
    pub columns: Vec<TableColumn>,

    /// Pages to render on
    #[serde(default)]
    pub pages: Option<Vec<usize>>,

    /// Optional enable flag - if set, evaluates binding to determine if block is rendered
    /// If the bound value is falsy (null, false, 0, empty string), block is not rendered
    #[serde(default)]
    pub enable: Option<String>,
}

/// Table column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    /// Field name in row object
    pub field: String,

    /// X offset from table position
    pub x: f64,

    /// Text alignment
    #[serde(default)]
    pub align: Align,

    /// Word wrap max characters
    #[serde(rename = "wordWrap")]
    #[serde(default)]
    pub word_wrap: Option<usize>,

    /// Number format pattern
    #[serde(default)]
    pub format: Option<String>,
}

/// QR code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeBlock {
    /// Block identifier
    #[serde(default)]
    pub id: Option<String>,

    /// Data binding path
    #[serde(default)]
    pub bind: Option<String>,

    /// Static QR data
    #[serde(default)]
    pub data: Option<String>,

    /// Position
    pub position: Position,

    /// Size
    pub size: Size,

    /// Error correction level
    #[serde(rename = "errorCorrection")]
    #[serde(default)]
    pub error_correction: ErrorCorrection,

    /// Pages to render on
    #[serde(default)]
    pub pages: Option<Vec<usize>>,

    /// Optional enable flag - if set, evaluates binding to determine if block is rendered
    /// If the bound value is falsy (null, false, 0, empty string), block is not rendered
    #[serde(default)]
    pub enable: Option<String>,
}

/// Size specification
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// QR code error correction level
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorCorrection {
    L,
    #[default]
    M,
    Q,
    H,
}

impl Block {
    /// Get the block ID if present
    pub fn id(&self) -> Option<&str> {
        match self {
            Block::Text(b) => b.id.as_deref(),
            Block::FieldForm(b) => b.id.as_deref(),
            Block::Table(b) => b.id.as_deref(),
            Block::QRCode(b) => b.id.as_deref(),
        }
    }

    /// Get the data binding path if present
    pub fn bind(&self) -> Option<&str> {
        match self {
            Block::Text(b) => b.bind.as_deref(),
            Block::FieldForm(b) => b.bind.as_deref(),
            Block::Table(b) => b.bind.as_deref(),
            Block::QRCode(b) => b.bind.as_deref(),
        }
    }

    /// Get the enable binding if present
    pub fn enable(&self) -> Option<&str> {
        match self {
            Block::Text(b) => b.enable.as_deref(),
            Block::FieldForm(b) => b.enable.as_deref(),
            Block::Table(b) => b.enable.as_deref(),
            Block::QRCode(b) => b.enable.as_deref(),
        }
    }

    /// Get the position
    pub fn position(&self) -> Position {
        match self {
            Block::Text(b) => b.position,
            Block::FieldForm(b) => b.position,
            Block::Table(b) => b.position,
            Block::QRCode(b) => b.position,
        }
    }

    /// Shift the block position
    pub fn shift_position(&mut self, dx: f64, dy: f64) {
        match self {
            Block::Text(b) => {
                b.position.x += dx;
                b.position.y += dy;
            }
            Block::FieldForm(b) => {
                b.position.x += dx;
                b.position.y += dy;
            }
            Block::Table(b) => {
                b.position.x += dx;
                b.position.y += dy;
            }
            Block::QRCode(b) => {
                b.position.x += dx;
                b.position.y += dy;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_text_block() {
        let json = r#"{
            "type": "text",
            "bind": "$.name",
            "position": { "x": 100, "y": 200 },
            "font": { "family": "sarabun", "size": 12 },
            "align": "left"
        }"#;

        let block: Block = serde_json::from_str(json).unwrap();

        match block {
            Block::Text(b) => {
                assert_eq!(b.bind, Some("$.name".to_string()));
                assert_eq!(b.position.x, 100.0);
                assert_eq!(b.align, Align::Left);
            }
            _ => panic!("Expected TextBlock"),
        }
    }

    #[test]
    fn test_parse_qrcode_block() {
        let json = r#"{
            "type": "qrcode",
            "bind": "$.qrData",
            "position": { "x": 100, "y": 200 },
            "size": { "width": 50, "height": 50 }
        }"#;

        let block: Block = serde_json::from_str(json).unwrap();

        match block {
            Block::QRCode(b) => {
                assert_eq!(b.bind, Some("$.qrData".to_string()));
                assert_eq!(b.size.width, 50.0);
                assert_eq!(b.error_correction, ErrorCorrection::M);
            }
            _ => panic!("Expected QRCodeBlock"),
        }
    }

    #[test]
    fn test_parse_text_block_with_enable() {
        let json = r#"{
            "type": "text",
            "bind": "$.name",
            "position": { "x": 100, "y": 200 },
            "enable": "$.showName"
        }"#;

        let block: Block = serde_json::from_str(json).unwrap();
        assert_eq!(block.enable(), Some("$.showName"));
    }
}
