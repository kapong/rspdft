# 001: Batch Processing & WASM Examples

**Branch**: `feature/001-batch-wasm`
**Status**: planning

## Objective

Improve the PDF template filling library with batch processing capabilities for rendering multiple PDFs efficiently, then expose this functionality via WebAssembly bindings with comprehensive examples for both browser and Node.js environments.

### Goals
1. **Batch Processing**: Render multiple documents from a single template+font setup without reloading resources
2. **WASM Integration**: Expose batch processing API to JavaScript
3. **Examples**: Provide working examples for browser and Node.js usage

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Template Crate                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ BatchRenderer                                        │   │
│  │  - Holds template, fonts, wordcut                    │   │
│  │  - render_batch(data: Vec<Value>) -> Vec<Vec<u8>>   │   │
│  │  - render_single(data: &Value) -> Vec<u8>           │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                      WASM Crate                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ PdfBatchRenderer (wasm_bindgen)                      │   │
│  │  - new(template_json, pdf_bytes, fonts)             │   │
│  │  - renderBatch(data_array) -> Uint8Array[]          │   │
│  │  - renderSingle(data) -> Uint8Array                 │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│   Browser Example       │     │   Node.js Example       │
│   - index.html          │     │   - render.mjs          │
│   - Web Workers support │     │   - CLI batch processing│
└─────────────────────────┘     └─────────────────────────┘
```

## Steps

### 1. Add Batch Renderer to Template Crate

- [ ] Create `BatchRenderer` struct that holds template, fonts, and optional wordcut
- [ ] Implement `render_single(&self, data: &Value) -> Result<Vec<u8>>`
- [ ] Implement `render_batch(&self, data: Vec<Value>) -> Result<Vec<Vec<u8>>>`
- [ ] Support for loading fonts from bytes (not just file paths) for WASM compatibility
- [ ] Add builder pattern for configuration

> Commit: `feat(template): add BatchRenderer for efficient multi-document rendering`

### 2. Add WASM Batch Rendering API

- [ ] Create `PdfBatchRenderer` struct with wasm_bindgen
- [ ] Implement `new(template_json, pdf_bytes)` constructor
- [ ] Implement `loadFont(name, bytes)` method
- [ ] Implement `setWordcut(wordcut: ThaiWordcut)` method
- [ ] Implement `renderSingle(data) -> Uint8Array` method
- [ ] Implement `renderBatch(data_array) -> Array<Uint8Array>` method
- [ ] Add proper error handling and JS-friendly error messages

> Commit: `feat(wasm): add PdfBatchRenderer for batch PDF generation`

### 3. Create Browser Example

- [ ] Create `examples/web/` directory structure
- [ ] Create `index.html` with file upload UI for template, PDF, fonts
- [ ] Create `main.js` with WASM initialization and batch rendering
- [ ] Add sample template JSON and demo data
- [ ] Support downloading generated PDFs as ZIP
- [ ] Add simple styling with CSS

> Commit: `feat(examples): add browser-based batch rendering demo`

### 4. Create Node.js Example

- [ ] Create `examples/node/` directory structure
- [ ] Create `render.mjs` CLI script for batch processing
- [ ] Create `package.json` with wasm-pack generated module dependency
- [ ] Support reading template, PDF, fonts, and data from files
- [ ] Output multiple PDFs to specified directory
- [ ] Add usage documentation in README

> Commit: `feat(examples): add Node.js batch rendering CLI example`

### 5. Documentation & Build Scripts

- [ ] Update main README with batch processing usage examples
- [ ] Add WASM build instructions
- [ ] Create build script for WASM package (`scripts/build-wasm.sh`)
- [ ] Document API in rustdoc comments

> Commit: `docs: add batch processing and WASM documentation`

## API Design

### Template Crate - BatchRenderer

```rust
/// Batch renderer for efficient multi-document PDF generation
pub struct BatchRenderer<'a> {
    template: Template,
    pdf_bytes: Vec<u8>,
    fonts: HashMap<String, Vec<u8>>,
    wordcut: Option<&'a ThaiWordcut>,
}

impl<'a> BatchRenderer<'a> {
    /// Create a new batch renderer
    pub fn new(template: Template, pdf_bytes: Vec<u8>) -> Self;

    /// Add a font (from bytes, not file path)
    pub fn add_font(self, name: &str, data: Vec<u8>) -> Self;

    /// Set Thai wordcut for word wrapping
    pub fn with_wordcut(self, wordcut: &'a ThaiWordcut) -> Self;

    /// Render a single document
    pub fn render(&self, data: &serde_json::Value) -> Result<Vec<u8>>;

    /// Render multiple documents
    pub fn render_batch(&self, data: Vec<serde_json::Value>) -> Result<Vec<Vec<u8>>>;
}
```

### WASM Crate - PdfBatchRenderer

```javascript
// JavaScript API
class PdfBatchRenderer {
    // Create from template JSON
    static fromJson(templateJson: string): PdfBatchRenderer;

    // Load base PDF template
    loadBasePdf(data: Uint8Array): void;

    // Load font
    loadFont(name: string, data: Uint8Array): void;

    // Set Thai wordcut (optional)
    setWordcut(wordcut: ThaiWordcut): void;

    // Render single PDF
    render(data: object): Uint8Array;

    // Render batch of PDFs
    renderBatch(dataArray: object[]): Uint8Array[];
}
```

## File Structure

```
examples/
├── web/
│   ├── index.html       # Main HTML page
│   ├── main.js          # JavaScript for WASM interaction
│   ├── style.css        # Basic styling
│   └── sample/
│       ├── template.json
│       └── data.json
└── node/
    ├── package.json
    ├── render.mjs       # CLI script
    └── README.md
```

## Tests

- [ ] Unit tests for BatchRenderer in template crate
- [ ] Test single document rendering matches existing TemplateRenderer
- [ ] Test batch rendering with multiple data sets
- [ ] WASM tests with wasm-bindgen-test
- [ ] Integration test for Node.js example

## Performance Considerations

1. **Font Reuse**: Fonts are loaded and parsed once, reused across all renders
2. **Template Parsing**: Template JSON is parsed once
3. **Memory Management**: Each rendered PDF is returned immediately, no accumulation
4. **WASM Memory**: Careful with large batch sizes to avoid OOM in browser

## Notes

- The existing `TemplateRenderer` uses file paths for fonts which doesn't work in WASM
- New `BatchRenderer` will accept font bytes directly for WASM compatibility
- Browser example should work offline (all assets bundled)
- Node.js example should be practical for CLI batch processing workflows
- Consider adding progress callback for batch rendering in future iteration