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
  "pdf": "path/to/base.pdf",
  "fonts": [
    { "id": "sarabun", "path": "fonts/THSarabunNew.ttf" },
    { "id": "sarabun-bold", "path": "fonts/THSarabunNew-Bold.ttf" }
  ],
  "blocks": [
    {
      "type": "text",
      "x": 100,
      "y": 200,
      "page": 1,
      "font": "sarabun",
      "fontSize": 14,
      "binding": "$.customer.name"
    },
    {
      "type": "fieldform",
      "x": 50,
      "y": 300,
      "page": 1,
      "font": "sarabun",
      "fontSize": 12,
      "binding": "$.taxId",
      "spacing": 15,
      "maxLength": 13
    },
    {
      "type": "qrcode",
      "x": 400,
      "y": 100,
      "page": 1,
      "size": 80,
      "binding": "$.qrData"
    }
  ]
}
```

### Key API Patterns

**Rust (Native)**:
```rust
// TemplateRenderer: load once, render many
let mut renderer = TemplateRenderer::new(&json, pdf_bytes, Some(Path::new(".")))?;
renderer.set_wordcut(ThaiWordcut::embedded()?);
let output = renderer.render(&data)?;  // Each call is independent
```

**JavaScript (WASM)**:
```javascript
// PdfTemplate: fluent API
const template = PdfTemplate.fromJson(json);
template.loadBasePdf(pdfBytes);
template.loadFont('sarabun', fontBytes);
template.setWordcut(ThaiWordcut.embedded());
const output = template.render(data);  // Each call is independent
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
cargo run --example boj45  # Run example
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
use thai_text::ThaiWordcut;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use embedded Thai dictionary (recommended - no external file needed)
    let wordcut = ThaiWordcut::embedded()?;
    
    // Or load from custom file if needed:
    // let wordcut = ThaiWordcut::from_file("custom-dict.txt")?;
    
    // Segment Thai text
    let words = wordcut.segment("สวัสดีครับ");
    println!("{:?}", words); // ["สวัสดี", "ครับ"]
    
    // Word wrap
    let lines = wordcut.word_wrap("ข้อความยาวๆ ที่ต้องการตัดบรรทัด", 20);
    
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
    const outputPdf = template.render(data);
    
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
    const output = template.render(data);
    
    writeFileSync('output.pdf', Buffer.from(output));
}

main();
```

## Batch Rendering (Rust)

The `TemplateRenderer` supports efficient batch rendering - load resources once, render many times:

```rust
use template::TemplateRenderer;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load once (fonts auto-loaded from template paths)
    let template_json = std::fs::read_to_string("template.json")?;
    let pdf_bytes = std::fs::read("base.pdf")?;
    let mut renderer = TemplateRenderer::new(&template_json, pdf_bytes, Some(std::path::Path::new(".")))?;

    // Optional: enable Thai word segmentation
    renderer.set_wordcut(thai_text::ThaiWordcut::embedded()?);

    // 2. Render multiple times (each independent, no data mixing)
    let records = vec![
        json!({"name": "Alice", "amount": 1000.00}),
        json!({"name": "Bob", "amount": 2000.00}),
        json!({"name": "Charlie", "amount": 3000.00}),
    ];

    for (i, data) in records.iter().enumerate() {
        let pdf = renderer.render(data)?;
        std::fs::write(format!("output_{}.pdf", i + 1), pdf)?;
    }

    Ok(())
}
```

For manual font loading (e.g., when fonts are not in template paths):

```rust
let mut renderer = TemplateRenderer::new(&template_json, pdf_bytes, None)?;
renderer.add_font("sarabun", std::fs::read("fonts/THSarabunNew.ttf")?);
```

## Examples

- **Browser**: See `examples/web/` for interactive browser demo
- **Node.js**: See `examples/node/` for CLI batch processing
- **Rust**: See `examples/boj45.rs` for native Rust usage

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
