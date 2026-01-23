# Architecture: rspdft

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Client Application                           │
│                    (Browser / Node.js / Native)                      │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          WASM Bindings                               │
│                         (crates/wasm)                                │
│  • JavaScript API                                                    │
│  • Binary data handling                                              │
│  • Error marshalling                                                 │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Template Engine                               │
│                       (crates/template)                              │
│  • JSON template parsing                                             │
│  • Data binding (JSONPath)                                           │
│  • Block rendering orchestration                                     │
│                                                                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐               │
│  │TextBlock │ │TableBlock│ │ QRBlock  │ │FieldForm │               │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘               │
└─────────────────────────────────────────────────────────────────────┘
                        │                       │
                        ▼                       ▼
┌────────────────────────────────┐  ┌─────────────────────────────────┐
│          PDF Core              │  │         Thai Text               │
│        (crates/pdf-core)       │  │       (crates/thai-text)        │
│  • Document loading (lopdf)    │  │  • Dictionary loading           │
│  • Page manipulation           │  │  • Word segmentation            │
│  • Text injection              │  │  • Line breaking                │
│  • Image injection             │  │  • Number formatting            │
│  • Font embedding              │  │  • Date formatting              │
└────────────────────────────────┘  └─────────────────────────────────┘
```

## Data Flow

```
Template JSON ──┐
                ├──▶ Template Parser ──▶ Block List
Data JSON ──────┘
                                            │
                                            ▼
Base PDF ──────────────────────────▶ PDF Document
                                            │
Dictionary ──▶ Thai Wordcut ────────────────┤
                                            │
Fonts ─────────────────────────────────────▶│
                                            ▼
                                    Render Each Block
                                            │
                                            ▼
                                    Output PDF Bytes
```

## Crate Dependencies

```
                    ┌─────────┐
                    │  wasm   │
                    └────┬────┘
                         │
                         ▼
                   ┌──────────┐
                   │ template │
                   └────┬─────┘
                        │
           ┌────────────┼────────────┐
           ▼            ▼            ▼
      ┌─────────┐  ┌─────────┐  ┌─────────┐
      │pdf-core │  │thai-text│  │ qrcode  │
      └─────────┘  └─────────┘  └─────────┘
           │
           ▼
      ┌─────────┐
      │  lopdf  │
      └─────────┘
```

## Key Types

### pdf-core

```rust
pub struct PdfDocument {
    inner: lopdf::Document,
    fonts: HashMap<String, FontData>,
}

pub struct FontData {
    name: String,
    ttf_data: Vec<u8>,
    subset: HashSet<char>,
}

impl PdfDocument {
    pub fn open(path: &str) -> Result<Self>;
    pub fn open_from_bytes(data: &[u8]) -> Result<Self>;
    pub fn save(&self, path: &str) -> Result<()>;
    pub fn to_bytes(&self) -> Result<Vec<u8>>;
    pub fn page_count(&self) -> usize;
    pub fn add_font(&mut self, name: &str, ttf_data: &[u8]) -> Result<()>;
    pub fn set_font(&mut self, name: &str, size: f32) -> Result<()>;
    pub fn insert_text(&mut self, text: &str, page: usize, x: f64, y: f64, align: Align) -> Result<()>;
    pub fn insert_image(&mut self, data: &[u8], page: usize, x: f64, y: f64, w: f64, h: f64) -> Result<()>;
}
```

### thai-text

```rust
pub struct ThaiWordcut {
    dict: HashSet<String>,
    max_word_len: usize,
}

impl ThaiWordcut {
    pub fn from_file(path: &str) -> Result<Self>;
    pub fn from_str(content: &str) -> Result<Self>;
    pub fn segment(&self, text: &str) -> Vec<String>;
    pub fn word_wrap(&self, text: &str, max_chars: usize) -> Vec<String>;
}

pub struct ThaiFormatter;

impl ThaiFormatter {
    pub fn format_number(n: i64) -> String;
    pub fn format_baht(amount: f64) -> String;
    pub fn format_date_short(date: NaiveDate) -> String;
    pub fn format_date_long(date: NaiveDate) -> String;
    pub fn format_year(date: NaiveDate) -> String;
}
```

### template

```rust
#[derive(Debug, Deserialize)]
pub struct Template {
    pub version: String,
    pub template: TemplateSource,
    pub fonts: Vec<FontDef>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Deserialize)]
pub struct TemplateSource {
    pub source: String,
    pub duplicate: Option<Duplicate>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Block {
    Text(TextBlock),
    FieldForm(FieldFormBlock),
    Table(TableBlock),
    QRCode(QRCodeBlock),
}

pub trait BlockRenderer {
    fn render(
        &self, 
        doc: &mut PdfDocument, 
        data: &serde_json::Value, 
        wordcut: &ThaiWordcut
    ) -> Result<(), RenderError>;
}
```

## WASM Interface

```typescript
// TypeScript declarations for the WASM module

export function init(): Promise<void>;

export class PdfTemplate {
    static fromJson(json: string): PdfTemplate;
    loadBasePdf(data: Uint8Array): void;
    loadFont(name: string, data: Uint8Array): void;
    render(data: object, wordcut: ThaiWordcut): Uint8Array;
}

export class ThaiWordcut {
    static fromDict(data: string): ThaiWordcut;
    segment(text: string): string[];
    wordWrap(text: string, maxChars: number): string[];
}

export class ThaiFormatter {
    static formatNumber(n: number): string;
    static formatBaht(amount: number): string;
    static formatDateShort(year: number, month: number, day: number): string;
}
```

## Error Handling

```rust
// Unified error type across crates

#[derive(Debug, thiserror::Error)]
pub enum GopdfError {
    #[error("PDF error: {0}")]
    Pdf(#[from] PdfError),
    
    #[error("Template error: {0}")]
    Template(#[from] TemplateError),
    
    #[error("Thai text error: {0}")]
    ThaiText(#[from] ThaiTextError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Configuration

Runtime configuration options:

```rust
#[derive(Debug, Default)]
pub struct RenderOptions {
    /// Enable debug borders around text blocks
    pub show_borders: bool,
    
    /// Default font size when not specified
    pub default_font_size: u8,
    
    /// DPI for image rendering
    pub image_dpi: u16,
}
```

## Thread Safety

- All core types are `Send + Sync` for native builds
- WASM builds are single-threaded by default
- No global state; all state is encapsulated in structs

## Memory Management

- Use `Vec<u8>` for binary data (PDF, fonts, images)
- Fonts are loaded once and reused
- Dictionary is loaded once and shared (via `Arc` in native, direct reference in WASM)
- PDF documents are processed in memory; streaming not supported

## Future Considerations

1. **Streaming PDF generation** - For very large documents
2. **Font subsetting** - Reduce output size by only including used glyphs
3. **PDF/A compliance** - For archival purposes
4. **Digital signatures** - For document authentication
5. **Incremental updates** - Modify PDFs without full rewrite
