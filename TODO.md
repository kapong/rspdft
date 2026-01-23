# Implementation Checklist

## Phase 1: Project Setup
- [x] Create Cargo workspace
- [x] Set up crate structure
- [x] Add dependencies to Cargo.toml files
- [x] Configure CI/CD (GitHub Actions)
- [x] Set up testing framework
- [x] Create .gitignore
- [x] Initialize git repository

## Phase 2: pdf-core Crate
- [x] Create `PdfDocument` struct wrapper around lopdf
- [x] Implement `PdfDocument::open()` - open from file path
- [x] Implement `PdfDocument::open_from_bytes()` - open from memory
- [x] Implement `PdfDocument::save()` - save to file
- [x] Implement `PdfDocument::to_bytes()` - save to memory
- [x] Implement `PdfDocument::page_count()`
- [x] Implement TTF font loading and parsing
- [x] Implement font embedding into PDF
- [ ] Implement font subsetting (optional, for smaller output)
- [x] Implement `insert_text()` with X, Y positioning
- [x] Implement text alignment (left, center, right)
- [x] Implement `insert_image()` for JPEG format
- [x] Implement `insert_image()` for PNG format
- [x] Implement page duplication
- [x] Write unit tests for all PDF operations
- [x] Test with Thai fonts (TH Sarabun New)

## Phase 3: thai-text Crate
- [x] Implement dictionary loading from file
- [x] Implement dictionary loading from string
- [x] Implement longest-matching word segmentation algorithm
- [x] Implement line breaking with Thai word awareness
- [x] Implement `word_wrap()` function with max character limit
- [x] Port Thai number formatting (`FormatTHNumberInt`)
- [x] Port Thai baht formatting (`FormatTHBaht`)
- [x] Port Thai date formatting short (`FormatTHDateShort`)
- [x] Port Thai date formatting long (`FormatTHDateLong`)
- [x] Port Thai year formatting (`FormatTHYear`)
- [x] Port number rendering (`RenderFloat`)
- [x] Write unit tests for word segmentation
- [x] Write unit tests for line breaking
- [x] Write unit tests for all formatting functions
- [x] Verify output matches original Go implementation

## Phase 4: template Crate
- [x] Define template JSON schema as Rust types
- [x] Implement template JSON parsing with serde
- [x] Implement data binding with JSONPath-like expressions
- [x] Implement `TextBlock` struct and renderer
- [x] Implement `FieldFormBlock` struct and renderer (character spacing)
- [x] Implement `TableBlock` struct and renderer
- [x] Implement `QRCodeBlock` struct and renderer
- [x] Implement block duplication feature (for duplicate receipts)
- [x] Implement conditional rendering (enable flag)
- [x] Implement number formatting in text blocks
- [x] Implement word wrapping in text blocks
- [x] Write integration tests with sample templates
- [x] Test backward compatibility with sample data

## Phase 5: wasm Crate
- [ ] Set up wasm-bindgen configuration
- [ ] Create `ThaiWordcut` JavaScript class
- [ ] Implement `ThaiWordcut.fromDict()` 
- [ ] Implement `ThaiWordcut.segment()`
- [ ] Implement `ThaiWordcut.wordWrap()`
- [ ] Create `PdfTemplate` JavaScript class
- [ ] Implement `PdfTemplate.fromJson()`
- [ ] Implement `PdfTemplate.loadBasePdf()`
- [ ] Implement `PdfTemplate.loadFont()`
- [ ] Implement `PdfTemplate.render()`
- [ ] Create `ThaiFormatter` JavaScript class for formatting utils
- [ ] Handle Uint8Array ↔ Vec<u8> conversion
- [ ] Handle JSON object ↔ serde_json::Value conversion
- [ ] Proper error handling and JavaScript-friendly error messages
- [ ] Build npm package with wasm-pack
- [ ] Write TypeScript type definitions (.d.ts)
- [ ] Create browser example (HTML + JS)
- [ ] Create Node.js example
- [ ] Test in multiple browsers (Chrome, Firefox, Safari)

## Phase 6: Testing & Quality
- [ ] Set up visual regression tests
- [ ] Create reference PDF outputs for comparison
- [ ] Performance benchmarks (native vs Go)
- [ ] Performance benchmarks (WASM in browser)
- [ ] Memory usage analysis
- [ ] WASM binary size optimization
- [ ] Code coverage analysis
- [ ] Clippy lint checks
- [ ] rustfmt formatting

## Phase 7: Documentation
- [ ] API documentation with rustdoc
- [ ] README.md with quick start guide
- [ ] Usage examples for Rust
- [ ] Usage examples for JavaScript/TypeScript
- [ ] Migration guide from Go version
- [ ] Template format documentation
- [ ] Data binding documentation
- [ ] Publish to crates.io (optional)
- [ ] Publish to npm (optional)

## Phase 8: MCP Integration (Optional)
- [ ] Create MCP server project skeleton
- [ ] Implement PostgreSQL connection
- [ ] Implement parameterized query execution
- [ ] Implement result mapping to JSON
- [ ] Output JSON in data-input format
- [ ] Document MCP server usage
- [ ] Example MCP tool definitions

## Known Challenges & Solutions

### Challenge: Font subsetting for Thai
**Problem**: Thai fonts can be large (2-5MB). Including full font increases PDF size.
**Solution**: 
- Start without subsetting (accept larger files)
- Add subsetting later using `subsetter` crate or custom implementation
- Only subset if output size becomes a problem

### Challenge: PDF content stream manipulation
**Problem**: `lopdf` provides low-level access but text positioning requires understanding PDF operators
**Solution**:
- Study PDF specification for text positioning (Tm, Td operators)
- Look at how existing libraries handle this
- May need to generate content streams manually

### Challenge: WASM binary size
**Problem**: Including all dependencies can result in large WASM files
**Solution**:
- Use `wasm-opt` for optimization
- Enable LTO in release builds
- Consider lazy-loading dictionary
- Split into multiple WASM modules if needed

### Challenge: Cross-platform font rendering consistency
**Problem**: Font metrics might differ slightly across platforms
**Solution**:
- Use `ab_glyph` or `rusttype` for consistent font parsing
- Test extensively on different platforms
- Document any known inconsistencies

## Priority Order

1. **Critical Path** (MVP):
   - pdf-core: open, save, insert_text, add_font
   - thai-text: dictionary, segment, word_wrap
   - template: parse, TextBlock, FieldFormBlock
   - wasm: basic bindings

2. **Important**:
   - pdf-core: insert_image
   - template: TableBlock, QRCodeBlock
   - thai-text: all formatters
   - wasm: full API

3. **Nice to Have**:
   - Font subsetting
   - PDF/A compliance
   - MCP integration
   - Visual regression tests
