# AGENT.md - rspdft

## Project Overview

This is a Rust implementation of a PDF template filling system, compiled to both 
native binary and WebAssembly (WASM). It handles Thai text properly with word 
segmentation and line breaking.

## Architecture

- **pdf-core**: Low-level PDF manipulation using `lopdf`
- **thai-text**: Thai word segmentation and line breaking
- **template**: JSON template parsing and block rendering
- **wasm**: WebAssembly bindings for browser/Node.js

## Key Features

1. Open existing PDF templates
2. Inject text at specific coordinates
3. Inject images (JPEG/PNG)
4. Generate and inject QR codes
5. Thai text word-wrapping
6. Number/date formatting (Thai locale)
7. Support for fieldform (character-spaced input)
8. Table rendering

## Development Guidelines

### Building

```bash
# Native build
cargo build --release

# WASM build
cd crates/wasm && wasm-pack build --target web
```

### Testing

```bash
cargo test --all
```

### Thai Dictionary

The Thai dictionary is **embedded** in the `thai-text` crate at compile time:
- Located at `crates/thai-text/data/chula-tnc-2017.txt`
- ~40,000 words from Chulalongkorn University TNC 2017
- Access via `ThaiWordcut::embedded()` or `Dictionary::embedded()`
- No external file needed at runtime

### Template Format

The template schema is **embedded** in the `template` crate:
- Located at `crates/template/data/template-schema.json`
- Access via `template::TEMPLATE_SCHEMA`

Key block types:
- `text`: Simple text at position
- `fieldform`: Character-by-character with custom spacing
- `table`: Multi-row data table
- `qrcode`: QR code image

### Data Binding

Use JSONPath syntax for data binding:
- `$.customer.name` - Access nested object
- `$.items[0].price` - Access array element

## File Organization

```
crates/
├── pdf-core/src/
│   ├── lib.rs          # Public exports
│   ├── document.rs     # PDF document wrapper
│   ├── page.rs         # Page operations
│   ├── text.rs         # Text injection
│   ├── image.rs        # Image injection
│   └── font.rs         # Font embedding
│
├── thai-text/src/
│   ├── lib.rs          # Public exports
│   ├── dictionary.rs   # Word dictionary loading
│   ├── wordcut.rs      # Word segmentation
│   ├── linebreak.rs    # Line breaking logic
│   └── formatter.rs    # Thai number/date formatting
│
├── template/src/
│   ├── lib.rs          # Public exports
│   ├── schema.rs       # Template schema types
│   ├── parser.rs       # JSON parsing
│   ├── renderer.rs     # Block to PDF rendering
│   └── blocks/
│       ├── mod.rs
│       ├── text.rs
│       ├── fieldform.rs
│       ├── table.rs
│       └── qrcode.rs
│
└── wasm/src/
    ├── lib.rs          # WASM entry point
    └── api.rs          # JavaScript API
```

## MCP Integration

This library is designed to work with MCP (Model Context Protocol) for data 
fetching. A separate MCP server can:

1. Connect to PostgreSQL
2. Execute queries
3. Output JSON in the data-input format
4. Pass to this library for PDF generation

## Performance Notes

- Dictionary loading: ~50-100ms for 40k words
- PDF generation: ~10-50ms per page
- WASM binary size: ~500KB-1MB (gzipped)

## Error Handling

Use `thiserror` for error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum PdfError {
    #[error("Failed to open PDF: {0}")]
    OpenError(String),
    #[error("Font not found: {0}")]
    FontNotFound(String),
    // ...
}
```

## Testing Strategy

1. Unit tests in each crate
2. Integration tests in workspace root
3. Visual regression tests comparing output PDFs
4. WASM tests in browser environment

## Original Go Project Reference

This project is inspired by the Go-based `gopdf` project. Key mappings:

| Go Component | Rust Equivalent |
|--------------|-----------------|
| `github.com/kapong/pdft` | `pdf-core` crate with `lopdf` |
| `github.com/kapong/mapkha` | `thai-text` crate |
| `github.com/boombuler/barcode` | `qrcode` crate |
| `block.go`, `textblock.go`, etc. | `template` crate |

See `MIGRATION_GUIDE.md` for detailed type and pattern mappings.

## Assets

### Font Files (`fonts/`)

| Font File | Description |
|-----------|-------------|
| `THSarabunNew.ttf` | TH Sarabun New Regular |
| `THSarabunNew Bold.ttf` | TH Sarabun New Bold |
| `THSarabunNew Italic.ttf` | TH Sarabun New Italic |
| `THSarabunNew BoldItalic.ttf` | TH Sarabun New Bold Italic |
| `NotoSansSymbols2-Regular.ttf` | Unicode symbols font (checkmarks, etc.) |

**Font Usage:**
- `THSarabunNew` - Primary Thai font, used for most text
- `THSarabunNew Bold` - Headers, important values
- `THSarabunNew Italic` / `BoldItalic` - Amount text (Thai baht)
- `NotoSansSymbols2` - Fallback for Unicode symbols (checkmarks, arrows, etc.)

**Important:** These are TTF (TrueType) fonts. The Rust implementation must:
1. Parse TTF using `ttf-parser` crate
2. Embed font subsets into PDF
3. Handle Thai character encoding correctly

### Embedded Data

Data files are embedded directly in the library at compile time:

- **Thai Dictionary** (`crates/thai-text/data/chula-tnc-2017.txt`)
  - ~40,000 words from Chulalongkorn University Thai National Corpus 2017
  - Access: `thai_text::EMBEDDED_DICT` or `ThaiWordcut::embedded()`
  
- **Template Schema** (`crates/template/data/template-schema.json`)
  - JSON schema for template validation
  - Access: `template::TEMPLATE_SCHEMA`
