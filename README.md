# rspdft

A cross-platform PDF template filling library in Rust, with WebAssembly support.

## Features

- Open existing PDF templates and inject content
- Text insertion with Thai language support (word segmentation, line breaking)
- Image insertion (JPEG, PNG)
- QR code generation and insertion
- Field form support (character-by-character spacing)
- Table rendering
- Thai number and date formatting
- **Embedded Thai dictionary** - no external files needed
- Compiles to native binary and WebAssembly

## AI Should Read This Part

This section provides essential context for AI agents working with this codebase.

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         rspdft                               │
├─────────────────────────────────────────────────────────────┤
│  crates/pdf-core/    - Low-level PDF manipulation           │
│    • PdfDocument: Open, modify, save PDFs                   │
│    • FontData: TrueType font embedding with subsetting      │
│    • Image handling (JPEG, PNG)                             │
├─────────────────────────────────────────────────────────────┤
│  crates/thai-text/   - Thai language processing             │
│    • ThaiWordcut: Word segmentation (embedded dictionary)   │
│    • Thai number/currency/date formatting                   │
├─────────────────────────────────────────────────────────────┤
│  crates/template/    - Template parsing and rendering       │
│    • TemplateRenderer: Load once, render many times         │
│    • Block types: text, fieldform, table, qrcode            │
│    • Data binding with JSONPath syntax                      │
├─────────────────────────────────────────────────────────────┤
│  crates/wasm/        - WebAssembly bindings                 │
│    • PdfTemplate: JS-friendly API wrapper                   │
│    • ThaiWordcut, ThaiFormatter exports                     │
└─────────────────────────────────────────────────────────────┘
```

### Template JSON Structure

```json
{
  "template": {
    "source": "path/to/base.pdf",
    "duplicate": {
      "page": 2,
      "additionalItems": [
        {
          "type": "text",
          "text": "(COPY)",
          "position": { "x": 550, "y": 15 },
          "font": { "family": "sarabun", "size": 10, "color": { "r": 255, "g": 0, "b": 0 } },
          "align": "right",
          "page": 2
        }
      ]
    }
  },
  "fonts": [
    { "id": "sarabun", "regular": "fonts/THSarabunNew.ttf", "bold": "fonts/THSarabunNew-Bold.ttf" }
  ],
  "blocks": [
    {
      "type": "text",
      "position": { "x": 100, "y": 200 },
      "font": { "family": "sarabun", "size": 14 },
      "bind": "$.customer.name",
      "pages": [1]
    },
    {
      "type": "fieldform",
      "position": { "x": 50, "y": 300 },
      "font": { "family": "sarabun", "size": 12 },
      "bind": "$.taxId",
      "charSpacing": [15, 15, 15],
      "pages": [1]
    },
    {
      "type": "qrcode",
      "position": { "x": 400, "y": 100 },
      "size": 80,
      "bind": "$.qrData",
      "pages": [1]
    }
  ]
}
```

The `duplicate` section allows:
- **page**: Duplicate all blocks to another page
- **x/y**: Offset for duplicated blocks
- **additionalItems**: Extra items (like "(COPY)" labels) rendered after duplication

### Key API Patterns

**Rust (Native)**:
```rust
// TemplateRenderer: load once, render many
let renderer = TemplateRenderer::new(&json, pdf_bytes, Some(Path::new(".")))?;
let output = renderer.render(&data)?;  // Returns PDF bytes directly

// Or render to document for post-render modifications
let mut doc = renderer.render_to_document(&data)?;
doc.set_font("sarabun", 10.0)?
    .set_font_weight(PdfFontWeight::Bold)?
    .set_text_color(PdfColor::red())
    .insert_text("(COPY)", 2, 550.0, 15.0, PdfAlign::Right)?;
let output = doc.to_bytes()?;
```

**JavaScript (WASM)**:
```javascript
// PdfTemplate: fluent API
const template = PdfTemplate.fromJson(json);
template.loadBasePdf(pdfBytes);
template.loadFont('sarabun', fontBytes);
template.setWordcut(ThaiWordcut.embedded());
const output = template.render(data);  // Returns PDF bytes directly

// Or render to document for post-render modifications
const doc = template.renderToDocument(data);
doc.setFont('sarabun', 10);
doc.setFontWeight('bold');
doc.setTextColor(255, 0, 0);  // Red
doc.insertText('(COPY)', 2, 550, 15, 'right');
const output = doc.toBytes();
```

### Data Binding

- `$.field` - Root level field
- `$.object.property` - Nested property
- `$.array[0]` - Array index
- `$.items[*].name` - Array iteration (in tables)

### Font Handling

Fonts are embedded as subsets (only used glyphs) to minimize PDF size.
Font IDs in template JSON must match the `id` used in `loadFont()` or auto-loaded from paths.

### Error Handling

All operations return `Result<T, E>`:
- Rust: Use `?` operator or `.expect()`
- JavaScript: Methods throw on error, wrap in try/catch

### Testing

```bash
cargo test --all        # Run all tests
cargo run --example render_form -- assets/approve_wh3.json input/approve_wh3_input.json
./scripts/build-wasm.sh    # Build WASM
```

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
rspdft = "0.1"
```

### JavaScript (npm)

```bash
npm install @rspdft/wasm
```

## Quick Start

### Rust

```rust
use std::path::Path;
use template::{PdfAlign, PdfColor, PdfFontWeight, TemplateRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load template and PDF
    let template_json = std::fs::read_to_string("template.json")?;
    let pdf_bytes = std::fs::read("base.pdf")?;
    let renderer = TemplateRenderer::new(&template_json, pdf_bytes, Some(Path::new(".")))?;

    // Data for rendering
    let data = serde_json::json!({ "name": "บริษัท ตัวอย่าง จำกัด" });

    // Option 1: Simple render (returns PDF bytes)
    let output = renderer.render(&data)?;
    std::fs::write("output.pdf", output)?;

    // Option 2: Render to document for post-render modifications
    let mut doc = renderer.render_to_document(&data)?;
    doc.set_font("sarabun", 10.0)?
        .set_font_weight(PdfFontWeight::Bold)?
        .set_text_color(PdfColor::red())
        .insert_text("(COPY)", 1, 550.0, 15.0, PdfAlign::Right)?;
    let output = doc.to_bytes()?;
    std::fs::write("output_with_label.pdf", output)?;

    Ok(())
}
```

### JavaScript (Browser)

```javascript
import init, { PdfTemplate, ThaiWordcut } from '@rspdft/wasm';

async function generatePdf() {
    // Initialize WASM module
    await init();
    
    // Use embedded dictionary (no fetch needed)
    const wordcut = ThaiWordcut.embedded();
    
    // Load template
    const templateResponse = await fetch('/template.json');
    const templateJson = await templateResponse.text();
    const template = PdfTemplate.fromJson(templateJson);
    
    // Load base PDF
    const pdfResponse = await fetch('/base-template.pdf');
    const pdfBytes = new Uint8Array(await pdfResponse.arrayBuffer());
    template.loadBasePdf(pdfBytes);
    
    // Load font
    const fontResponse = await fetch('/fonts/THSarabunNew.ttf');
    const fontBytes = new Uint8Array(await fontResponse.arrayBuffer());
    template.loadFont('sarabun', fontBytes);
    
    // Render with data
    const data = {
        customer: { name: "บริษัท ตัวอย่าง จำกัด" },
        amount: 40000.50
    };
    
    // Option 1: Simple render (returns PDF bytes)
    const outputPdf = template.render(data);
    
    // Option 2: Render to document for post-render modifications
    const doc = template.renderToDocument(data);
    doc.setFont('sarabun', 10);
    doc.setFontWeight('bold');
    doc.setTextColor(255, 0, 0);  // Red
    doc.insertText('(COPY)', 1, 550, 15, 'right');
    const outputPdf = doc.toBytes();
    
    // Download
    const blob = new Blob([outputPdf], { type: 'application/pdf' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'output.pdf';
    a.click();
}
```

### Node.js

```javascript
import { readFileSync, writeFileSync } from 'fs';
import { PdfTemplate, ThaiWordcut } from '@rspdft/wasm';

async function main() {
    // Use embedded dictionary
    const wordcut = ThaiWordcut.embedded();
    
    const templateJson = readFileSync('template.json', 'utf-8');
    const template = PdfTemplate.fromJson(templateJson);
    
    const pdfBytes = readFileSync('base-template.pdf');
    template.loadBasePdf(new Uint8Array(pdfBytes));
    
    const fontBytes = readFileSync('fonts/THSarabunNew.ttf');
    template.loadFont('sarabun', new Uint8Array(fontBytes));
    
    // Set wordcut for Thai text wrapping
    template.setWordcut(wordcut);
    
    const data = { customer: { name: "Test" }, amount: 100 };
    
    // Option 1: Simple render
    const output = template.render(data);
    writeFileSync('output.pdf', Buffer.from(output));
    
    // Option 2: Render to document for post-render modifications
    const doc = template.renderToDocument(data);
    doc.setFont('sarabun', 10);
    doc.setFontWeight('bold');
    doc.setTextColor(255, 0, 0);  // Red
    doc.insertText('(COPY)', 1, 550, 15, 'right');
    const modifiedOutput = doc.toBytes();
    writeFileSync('output_with_label.pdf', Buffer.from(modifiedOutput));
}

main();
```

## Batch Rendering (Rust)

The `TemplateRenderer` supports efficient batch rendering - load resources once, render many times:

```rust
use std::path::Path;
use template::{PdfAlign, PdfColor, PdfFontWeight, TemplateRenderer};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load once (fonts auto-loaded from template paths)
    let template_json = std::fs::read_to_string("template.json")?;
    let pdf_bytes = std::fs::read("base.pdf")?;
    let renderer = TemplateRenderer::new(&template_json, pdf_bytes, Some(Path::new(".")))?;

    // 2. Render multiple times (each independent, no data mixing)
    let records = vec![
        json!({"name": "Alice", "amount": 1000.00}),
        json!({"name": "Bob", "amount": 2000.00}),
        json!({"name": "Charlie", "amount": 3000.00}),
    ];

    for (i, data) in records.iter().enumerate() {
        // Simple render
        let pdf = renderer.render(&data)?;
        std::fs::write(format!("output_{}.pdf", i + 1), pdf)?;

        // Or with post-render modifications
        let mut doc = renderer.render_to_document(&data)?;
        doc.set_font("sarabun", 10.0)?
            .set_font_weight(PdfFontWeight::Bold)?
            .set_text_color(PdfColor::red())
            .insert_text("(COPY)", 1, 550.0, 15.0, PdfAlign::Right)?;
        std::fs::write(format!("output_{}_copy.pdf", i + 1), doc.to_bytes()?)?;
    }

    Ok(())
}
```

For manual font loading (e.g., when fonts are not in template paths):

```rust
let mut renderer = TemplateRenderer::new(&template_json, pdf_bytes, None)?;
renderer.add_font("sarabun", std::fs::read("fonts/THSarabunNew.ttf")?);
```

## Post-Render Modifications

The `render_to_document()` / `renderToDocument()` API returns a document object that allows modifications after template rendering. This is useful for:

- Adding page-specific labels (e.g., "(COPY)" on duplicate pages)
- Adding watermarks
- Dynamic content based on page count

### PdfDocument Methods (Rust)

| Method | Description |
|--------|-------------|
| `set_font(id, size)` | Set font for subsequent text (returns `&mut Self`) |
| `set_font_weight(weight)` | Set weight: `Regular` or `Bold` |
| `set_font_style(style)` | Set style: `Normal` or `Italic` |
| `set_text_color(color)` | Set RGB color |
| `insert_text(text, page, x, y, align)` | Insert text at position |
| `page_count()` | Get number of pages |
| `to_bytes()` | Convert to PDF bytes |

### WasmPdfDocument Methods (JavaScript)

| Method | Description |
|--------|-------------|
| `setFont(id, size)` | Set font for subsequent text |
| `setFontWeight(weight)` | `"regular"` or `"bold"` |
| `setFontStyle(style)` | `"normal"` or `"italic"` |
| `setTextColor(r, g, b)` | RGB values (0-255) |
| `insertText(text, page, x, y, align)` | Insert text (`align`: "left", "center", "right") |
| `pageCount()` | Get number of pages |
| `toBytes()` | Convert to PDF bytes (Uint8Array) |

### Method Chaining (Rust)

`set_font`, `set_font_weight`, `set_font_style`, and `set_text_color` return `&mut Self` for fluent API:

```rust
doc.set_font("sarabun", 10.0)?
    .set_font_weight(PdfFontWeight::Bold)?
    .set_text_color(PdfColor::red())
    .insert_text("(COPY)", 1, 550.0, 15.0, PdfAlign::Right)?;
```

## Examples

### CLI Form Renderer

```bash
# Render any form with template and input JSON
cargo run --example render_form -- <template.json> <input.json> [output.pdf]

# Examples:
cargo run --example render_form -- assets/approve_wh3.json input/approve_wh3_input.json
cargo run --example render_form -- assets/boj45_template.json input/boj45_input.json
```

### Other Examples

- **Browser**: See `examples/web/` for interactive browser demo
- **Node.js**: See `examples/node/` for CLI batch processing

## Building

### Native

```bash
cargo build --release
```

### WASM (using build script)

```bash
./scripts/build-wasm.sh
```

This builds both web and Node.js targets and sets up example symlinks.

### WASM (manual)

```bash
cd crates/wasm
wasm-pack build --target web      # For browsers (ES modules)
wasm-pack build --target nodejs   # For Node.js (CommonJS)
```

## Project Structure

```
crates/
├── pdf-core/     # Low-level PDF manipulation
├── thai-text/    # Thai language processing (with embedded dictionary)
├── template/     # Template parsing and rendering (with embedded schema)
└── wasm/         # WebAssembly bindings
```

## Template Format

The template schema is embedded in the library and can be accessed via `template::TEMPLATE_SCHEMA`.

### Block Types

- **text**: Simple text at a position
- **fieldform**: Character-by-character with custom spacing (e.g., tax ID boxes)
- **table**: Multi-row data tables
- **qrcode**: QR code images

### Data Binding

Use JSONPath-like syntax:
- `$.customer.name` - Object property
- `$.items[0].price` - Array element

## Thai Language Support

This library includes full Thai language support with an **embedded dictionary** (~50,000 words):

- **Word Segmentation**: Uses dictionary-based longest matching
- **Line Breaking**: Thai-aware word wrapping
- **Number Formatting**: Thai numerals (หนึ่ง, สอง, สาม...)
- **Currency**: Thai baht text (สามหมื่นบาทถ้วน)
- **Dates**: Thai Buddhist calendar (25 ม.ค. 68)

## Credits & Acknowledgments

### Thai Dictionary

The embedded Thai dictionary is from the **LibreOffice/Hunspell** project.

- **Source**: [LibreOffice Dictionaries - th_TH](https://github.com/LibreOffice/dictionaries/tree/master/th_TH)
- **License**: LGPL-3.0
- **Words**: ~50,000 Thai words

### Fonts (included in `fonts/`)

| Font | License | Source |
|------|---------|--------|
| **TH Sarabun New** | Public Domain (SIPA) | [SIPA Fonts](https://www.sipa.or.th/) - Software Industry Promotion Agency (Thailand) |
| **Noto Sans Symbols 2** | OFL 1.1 | [Google Fonts](https://fonts.google.com/noto/specimen/Noto+Sans+Symbols+2) |

## License

MIT
