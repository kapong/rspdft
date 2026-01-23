# 001: Batch Processing & WASM Examples

**Branch**: `feature/001-batch-wasm`
**Status**: completed

## Objective

Refactor `TemplateRenderer` to support a load-modify-render workflow. Resources are loaded once, can be modified, then `render()` can be called multiple times - each call clones the base state and applies fresh data with no mixing between calls.

### Goals
1. **Load Once**: Template, PDF, and fonts loaded once
2. **Modify**: Allow template/document modifications after loading  
3. **Render Multiple Times**: Each `render(data)` clones base state, applies data, returns bytes
4. **No Data Mixing**: Previous render data never affects next render
5. **WASM Support**: Expose full workflow to JavaScript

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     TemplateRenderer                         │
├─────────────────────────────────────────────────────────────┤
│  LOAD PHASE (once)                                           │
│    new(template_json, pdf_bytes) -> TemplateRenderer         │
│    add_font(name, bytes)                                     │
│    set_wordcut(wordcut)                                      │
├─────────────────────────────────────────────────────────────┤
│  MODIFY PHASE (optional, anytime before render)              │
│    template_mut() -> &mut Template                           │
├─────────────────────────────────────────────────────────────┤
│  RENDER PHASE (call multiple times, each independent)        │
│    render(data) -> Vec<u8>                                   │
│      └─> clones PDF, applies data, returns bytes             │
│    render(data) -> Vec<u8>   // fresh clone, no mixing       │
│    render(data) -> Vec<u8>   // fresh clone, no mixing       │
└─────────────────────────────────────────────────────────────┘
```

## Design

### Key Principle: Clone on Render

```rust
// Each render() call:
pub fn render(&self, data: &Value) -> Result<Vec<u8>> {
    // 1. Clone base PDF bytes -> new PdfDocument
    let mut doc = PdfDocument::open_from_bytes(&self.pdf_bytes)?;
    
    // 2. Add fonts to this fresh document
    for (name, font_data) in &self.fonts {
        doc.add_font(name, font_data)?;
    }
    
    // 3. Render template with data
    // ... apply blocks to doc ...
    
    // 4. Return bytes (doc is dropped, no state retained)
    doc.to_bytes()
}
```

Each call is completely independent - no state from previous renders.

## Steps

### 1a. Refactor TemplateRenderer

- [x] Change `TemplateRenderer` to own `Template` (not reference)
- [x] Add `pdf_bytes: Vec<u8>` field
- [x] Add `fonts: HashMap<String, Vec<u8>>` field
- [x] Add `wordcut: Option<ThaiWordcut>` field (owned)
- [x] Create `new(template_json: &str, pdf_bytes: Vec<u8>) -> Result<Self>`
- [x] Add `add_font(&mut self, name: &str, data: Vec<u8>)`
- [x] Add `set_wordcut(&mut self, wordcut: ThaiWordcut)`
- [x] Add `template(&self) -> &Template`
- [x] Add `template_mut(&mut self) -> &mut Template`
- [x] Implement `render(&self, data: &Value) -> Result<Vec<u8>>` (clones, applies, returns)
- [x] Remove old `render(&self, doc: &mut PdfDocument, data)` method

> Commit: `feat(template): refactor TemplateRenderer for reusable rendering`

### 1b. Update Existing Examples

- [x] Update `examples/full_demo.rs` to use new TemplateRenderer API
- [x] Update `examples/boj45.rs` to use new TemplateRenderer API
- [x] Update integration tests in `crates/template/tests/`
- [x] Ensure `cargo test` passes

> Commit: `refactor(examples): update examples to use new TemplateRenderer API`

### 2. Update WASM PdfTemplate

- [x] Refactor `PdfTemplate` to use new `TemplateRenderer` internally
- [x] Keep `fromJson()`, `loadBasePdf()`, `loadFont()` API
- [x] Update `render(data, wordcut)` to use new renderer
- [x] Each JS `render()` call is independent, no data mixing

> Commit: `feat(wasm): update PdfTemplate to use refactored renderer`

### 3. Create Browser Example

- [x] Create `examples/web/` directory structure
- [x] Create `index.html` with UI for:
  - Upload template JSON, base PDF, fonts
  - Input data (JSON textarea or form)
  - Render button -> download PDF
  - Support rendering multiple times with different data
- [x] Create `main.js` with WASM initialization
- [x] Add sample template JSON and demo data
- [x] Add simple styling with CSS

> Commit: `feat(examples): add browser-based rendering demo`

### 4. Create Node.js Example

- [x] Create `examples/node/` directory structure
- [x] Create `render.mjs` CLI script
  - Load template, PDF, fonts once
  - Read data from JSON file (array of records)
  - Call render() for each record
  - Save each PDF to output directory
- [x] Create `package.json`
- [x] Add README with usage instructions

> Commit: `feat(examples): add Node.js rendering CLI example`

### 5. Documentation & Build Scripts

- [x] Update main README with new API usage
- [x] Add WASM build instructions
- [x] Create `scripts/build-wasm.sh`
- [x] Document API in rustdoc comments

> Commit: `docs: add batch processing and WASM documentation`

## API Design

### Template Crate - TemplateRenderer

```rust
pub struct TemplateRenderer {
    template: Template,
    pdf_bytes: Vec<u8>,
    fonts: HashMap<String, Vec<u8>>,
    wordcut: Option<ThaiWordcut>,
}

impl TemplateRenderer {
    /// Create from template JSON and base PDF
    pub fn new(template_json: &str, pdf_bytes: Vec<u8>) -> Result<Self>;
    
    /// Add font from bytes
    pub fn add_font(&mut self, name: &str, data: Vec<u8>);
    
    /// Set Thai wordcut
    pub fn set_wordcut(&mut self, wordcut: ThaiWordcut);
    
    /// Get template (read-only)
    pub fn template(&self) -> &Template;
    
    /// Get template (mutable for modifications)
    pub fn template_mut(&mut self) -> &mut Template;
    
    /// Render with data - clones base PDF, applies data, returns bytes
    /// Can be called multiple times, each call is independent
    pub fn render(&self, data: &serde_json::Value) -> Result<Vec<u8>>;
}
```

### Usage Example (Rust)

```rust
// Load once
let mut renderer = TemplateRenderer::new(template_json, pdf_bytes)?;
renderer.add_font("sarabun", font_bytes);
renderer.set_wordcut(wordcut);

// Optionally modify template
renderer.template_mut().blocks[0].set_text("Custom Header");

// Render multiple times - each is independent
let pdf1 = renderer.render(&json!({"name": "Alice", "amount": 1000}))?;
let pdf2 = renderer.render(&json!({"name": "Bob", "amount": 2000}))?;
let pdf3 = renderer.render(&json!({"name": "Charlie", "amount": 3000}))?;

// pdf1, pdf2, pdf3 are completely independent
std::fs::write("alice.pdf", pdf1)?;
std::fs::write("bob.pdf", pdf2)?;
std::fs::write("charlie.pdf", pdf3)?;
```

### WASM / JavaScript Usage

```javascript
// Load once
const template = PdfTemplate.fromJson(templateJson);
template.loadBasePdf(pdfBytes);
template.loadFont('sarabun', fontBytes);
template.setWordcut(ThaiWordcut.embedded());

// Render multiple times
const records = [
    { name: 'Alice', amount: 1000 },
    { name: 'Bob', amount: 2000 },
    { name: 'Charlie', amount: 3000 },
];

for (const record of records) {
    const pdf = template.render(record);  // Each call is fresh
    download(pdf, `${record.name}.pdf`);
}
```

## File Structure

```
examples/
├── web/
│   ├── index.html
│   ├── main.js
│   ├── style.css
│   └── sample/
│       ├── template.json
│       └── data.json
└── node/
    ├── package.json
    ├── render.mjs
    └── README.md

scripts/
└── build-wasm.sh
```

## Tests

- [ ] Test render() returns valid PDF bytes
- [ ] Test multiple render() calls with different data are independent
- [ ] Test template_mut() changes affect subsequent renders
- [ ] Test fonts are correctly applied
- [ ] WASM tests with wasm-bindgen-test

## Notes

- Breaking change: removes old `render(&mut doc, data)` 
- Each `render()` clones PDF bytes -> fresh PdfDocument
- No `render_batch()` - just call `render()` in a loop
- Fonts stored as bytes, added to each cloned document
- Wordcut is owned (not borrowed) for simpler lifetime management