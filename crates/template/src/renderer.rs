//! Template rendering

use crate::parser::{parse_template, resolve_binding, value_to_string};
use crate::schema::*;
use crate::{Result, TemplateError};
use pdf_core::{FontStyle as PdfFontStyle, FontWeight, PdfDocument};
use std::collections::HashMap;
use thai_text::ThaiWordcut;

/// Template renderer with owned resources for reusable rendering
///
/// # Example
/// ```ignore
/// // Native Rust - auto-load fonts from template paths
/// let renderer = TemplateRenderer::new(&template_json, pdf_bytes, Some(Path::new(".")))?;
///
/// // WASM or manual font loading
/// let mut renderer = TemplateRenderer::new(&template_json, pdf_bytes, None)?;
/// renderer.add_font("sarabun", font_bytes);
/// renderer.set_wordcut(wordcut);
///
/// // Render multiple times - each call is independent
/// let pdf1 = renderer.render(&data1)?;
/// let pdf2 = renderer.render(&data2)?;
/// ```
pub struct TemplateRenderer {
    /// The template (owned)
    template: Template,
    /// Base PDF bytes
    pdf_bytes: Vec<u8>,
    /// Fonts loaded from bytes (font_id -> font_bytes)
    fonts: HashMap<String, Vec<u8>>,
    /// Thai word segmentation (owned)
    wordcut: Option<ThaiWordcut>,
}

impl TemplateRenderer {
    /// Create renderer from template JSON and base PDF bytes
    ///
    /// If `base_path` is provided (native Rust only), fonts defined in the template
    /// will be automatically loaded from disk. Pass `None` for WASM or when you want
    /// to load fonts manually via `add_font()`.
    ///
    /// # Example
    /// ```ignore
    /// // Native Rust - auto-load fonts from template paths
    /// let renderer = TemplateRenderer::new(&template_json, pdf_bytes, Some(Path::new(".")))?;
    ///
    /// // WASM or manual font loading
    /// let mut renderer = TemplateRenderer::new(&template_json, pdf_bytes, None)?;
    /// renderer.add_font("sarabun", font_bytes);
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(
        template_json: &str,
        pdf_bytes: Vec<u8>,
        base_path: Option<&std::path::Path>,
    ) -> Result<Self> {
        let template = parse_template(template_json)?;
        let mut renderer = Self {
            template,
            pdf_bytes,
            fonts: HashMap::new(),
            wordcut: None,
        };

        // Auto-load fonts if base_path provided
        if let Some(path) = base_path {
            renderer.load_fonts_internal(path)?;
        }

        Ok(renderer)
    }

    /// WASM version - no filesystem access, fonts must be added manually
    #[cfg(target_arch = "wasm32")]
    pub fn new(template_json: &str, pdf_bytes: Vec<u8>) -> Result<Self> {
        let template = parse_template(template_json)?;
        Ok(Self {
            template,
            pdf_bytes,
            fonts: HashMap::new(),
            wordcut: None,
        })
    }

    /// Add font from bytes
    pub fn add_font(&mut self, name: &str, data: Vec<u8>) {
        self.fonts.insert(name.to_string(), data);
    }

    /// Set Thai wordcut for word wrapping
    pub fn set_wordcut(&mut self, wordcut: ThaiWordcut) {
        self.wordcut = Some(wordcut);
    }

    /// Load fonts from file paths defined in the template
    ///
    /// For native Rust use - reads font files from disk based on paths in template JSON.
    /// The base_path is prepended to relative font paths.
    ///
    /// # Example
    /// ```ignore
    /// let mut renderer = TemplateRenderer::new(&template_json, pdf_bytes, None)?;
    /// renderer.load_fonts_from_template(Path::new("."))?;  // Load from current dir
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_fonts_from_template(&mut self, base_path: &std::path::Path) -> Result<()> {
        self.load_fonts_internal(base_path)
    }

    /// Internal method to load fonts from template paths
    #[cfg(not(target_arch = "wasm32"))]
    fn load_fonts_internal(&mut self, base_path: &std::path::Path) -> Result<()> {
        for font_def in &self.template.fonts {
            // Load regular variant (or legacy source)
            if let Some(ref path) = font_def.regular {
                let full_path = base_path.join(path);
                let data = std::fs::read(&full_path).map_err(|e| {
                    TemplateError::FontError(format!("Failed to load font {path}: {e}"))
                })?;
                self.fonts.insert(font_def.id.clone(), data);
            } else if let Some(ref path) = font_def.source {
                let full_path = base_path.join(path);
                let data = std::fs::read(&full_path).map_err(|e| {
                    TemplateError::FontError(format!("Failed to load font {path}: {e}"))
                })?;
                self.fonts.insert(font_def.id.clone(), data);
            }

            // Load bold variant
            if let Some(ref path) = font_def.bold {
                let full_path = base_path.join(path);
                let data = std::fs::read(&full_path).map_err(|e| {
                    TemplateError::FontError(format!("Failed to load font {path}: {e}"))
                })?;
                self.fonts.insert(format!("{}-bold", font_def.id), data);
            }

            // Load italic variant
            if let Some(ref path) = font_def.italic {
                let full_path = base_path.join(path);
                let data = std::fs::read(&full_path).map_err(|e| {
                    TemplateError::FontError(format!("Failed to load font {path}: {e}"))
                })?;
                self.fonts.insert(format!("{}-italic", font_def.id), data);
            }

            // Load bold-italic variant
            if let Some(ref path) = font_def.bold_italic {
                let full_path = base_path.join(path);
                let data = std::fs::read(&full_path).map_err(|e| {
                    TemplateError::FontError(format!("Failed to load font {path}: {e}"))
                })?;
                self.fonts
                    .insert(format!("{}-bold-italic", font_def.id), data);
            }
        }
        Ok(())
    }

    /// Get template (read-only)
    pub fn template(&self) -> &Template {
        &self.template
    }

    /// Get template (mutable for modifications)
    pub fn template_mut(&mut self) -> &mut Template {
        &mut self.template
    }

    /// Render with data - clones base PDF, applies data, returns bytes
    ///
    /// Each call creates a fresh PdfDocument from stored pdf_bytes,
    /// adds fonts, renders template blocks, and returns the output bytes.
    /// No state is retained between calls.
    ///
    /// For more control over the output, use `render_to_document()` instead,
    /// which returns a PdfDocument that can be further modified before
    /// calling `to_bytes()` on it.
    pub fn render(&self, data: &serde_json::Value) -> Result<Vec<u8>> {
        let mut doc = self.render_to_document(data)?;
        doc.to_bytes()
            .map_err(|e| TemplateError::RenderError(format!("Failed to save PDF: {e}")))
    }

    /// Render with data - returns a PdfDocument for further modification
    ///
    /// This allows you to make additional modifications to the document
    /// (e.g., adding watermarks, labels only on specific pages) before
    /// converting to bytes.
    ///
    /// # Example
    /// ```ignore
    /// let mut doc = renderer.render_to_document(&data)?;
    /// // Add a "COPY" label only on page 2
    /// doc.draw_text("(COPY)", 2, 500.0, 50.0)?;
    /// let bytes = doc.to_bytes()?;
    /// ```
    pub fn render_to_document(&self, data: &serde_json::Value) -> Result<PdfDocument> {
        // 1. Clone base PDF -> fresh document
        let mut doc = PdfDocument::open_from_bytes(&self.pdf_bytes)
            .map_err(|e| TemplateError::RenderError(format!("Failed to open PDF: {e}")))?;

        // 2. Prepare pages for duplication if configured
        if let Some(duplicate) = &self.template.template.duplicate {
            if let Some(target_page) = duplicate.page {
                // Ensure target page exists by duplicating page 1
                while doc.page_count() < target_page as usize {
                    doc.duplicate_page(1).map_err(|e| {
                        TemplateError::RenderError(format!("Failed to duplicate page: {e}"))
                    })?;
                }
            }
        }

        // 3. Add fonts from stored bytes
        for (name, font_data) in &self.fonts {
            doc.add_font(name, font_data).map_err(|e| {
                TemplateError::RenderError(format!("Failed to add font {name}: {e}"))
            })?;
        }

        // 4. Render all blocks
        self.render_blocks(&mut doc, data)?;

        Ok(doc)
    }

    /// Internal: render all blocks to document
    fn render_blocks(&self, doc: &mut PdfDocument, data: &serde_json::Value) -> Result<()> {
        // Render all blocks
        for block in &self.template.blocks {
            self.render_block(doc, block, data)?;
        }

        // Handle block duplication if configured
        if let Some(duplicate) = &self.template.template.duplicate {
            let has_offset = duplicate.x != 0.0 || duplicate.y != 0.0;
            let has_page = duplicate.page.is_some();

            if has_offset || has_page {
                for block in &self.template.blocks {
                    let mut dup_block = block.clone();

                    // Apply position offset if configured
                    if has_offset {
                        dup_block.shift_position(duplicate.x, duplicate.y);
                    }

                    // Change target page if configured
                    if let Some(target_page) = duplicate.page {
                        dup_block.set_pages(vec![target_page as usize]);
                    }

                    self.render_block(doc, &dup_block, data)?;
                }
            }

            // Render additional items (e.g., "(COPY)" label)
            for item in &duplicate.additional_items {
                self.render_additional_item(doc, item)?;
            }
        }

        Ok(())
    }

    /// Render an additional item (from duplicate configuration)
    fn render_additional_item(
        &self,
        doc: &mut PdfDocument,
        item: &crate::schema::AdditionalItem,
    ) -> Result<()> {
        if item.item_type != "text" {
            return Ok(()); // Only text type supported for now
        }

        let text = item.text.as_deref().unwrap_or("");
        if text.is_empty() {
            return Ok(());
        }

        let page = item.page.unwrap_or(1);

        // Set font if specified
        if let Some(font) = &item.font {
            let font_weight = match font.style {
                crate::schema::FontStyle::Bold | crate::schema::FontStyle::BoldItalic => {
                    pdf_core::FontWeight::Bold
                }
                _ => pdf_core::FontWeight::Regular,
            };
            let font_style = match font.style {
                crate::schema::FontStyle::Italic | crate::schema::FontStyle::BoldItalic => {
                    pdf_core::FontStyle::Italic
                }
                _ => pdf_core::FontStyle::Normal,
            };

            doc.set_font(&font.family, font.size as f32)
                .map_err(|e| TemplateError::RenderError(format!("Font error: {e}")))?;
            doc.set_font_weight(font_weight)
                .map_err(|e| TemplateError::RenderError(format!("Font weight error: {e}")))?;
            doc.set_font_style(font_style)
                .map_err(|e| TemplateError::RenderError(format!("Font style error: {e}")))?;

            // Set color if specified
            if let Some(color) = &font.color {
                // Color values in JSON are 0-255, convert to pdf_core::Color
                let pdf_color =
                    pdf_core::Color::from_rgb(color.r as u8, color.g as u8, color.b as u8);
                doc.set_text_color(pdf_color);
            }
        }

        // Convert alignment
        let align = match item.align {
            crate::schema::Align::Left => pdf_core::Align::Left,
            crate::schema::Align::Center => pdf_core::Align::Center,
            crate::schema::Align::Right => pdf_core::Align::Right,
        };

        doc.insert_text(text, page, item.position.x, item.position.y, align)
            .map_err(|e| TemplateError::RenderError(format!("Insert text error: {e}")))?;

        Ok(())
    }

    /// Render a single block
    fn render_block(
        &self,
        doc: &mut PdfDocument,
        block: &Block,
        data: &serde_json::Value,
    ) -> Result<()> {
        // Check if block is enabled
        if !self.is_block_enabled(block, data) {
            return Ok(());
        }

        match block {
            Block::Text(b) => self.render_text_block(doc, b, data),
            Block::FieldForm(b) => self.render_fieldform_block(doc, b, data),
            Block::Table(b) => self.render_table_block(doc, b, data),
            Block::QRCode(b) => self.render_qrcode_block(doc, b, data),
        }
    }

    /// Check if a block is enabled based on its enable binding
    fn is_block_enabled(&self, block: &Block, data: &serde_json::Value) -> bool {
        match block.enable() {
            None => true, // No enable binding = always enabled
            Some(bind) => {
                match resolve_binding(bind, data) {
                    None => false, // Binding not found = disabled
                    Some(value) => is_truthy(value),
                }
            }
        }
    }

    /// Render a text block
    fn render_text_block(
        &self,
        doc: &mut PdfDocument,
        block: &TextBlock,
        data: &serde_json::Value,
    ) -> Result<()> {
        // Resolve text content
        let text = if let Some(bind) = &block.bind {
            resolve_binding(bind, data)
                .map(value_to_string)
                .unwrap_or_default()
        } else {
            block.text.clone().unwrap_or_default()
        };

        if text.is_empty() {
            return Ok(());
        }

        // Apply formatting if specified
        let formatted_text = self.format_text(&text, block.format.as_deref(), block.format_type)?;

        // Set font if specified
        if let Some(font) = &block.font {
            self.set_font(doc, font)?;

            // Set text color from font (or default to black)
            let color = font.color.unwrap_or_default();
            doc.set_text_color(pdf_core::Color::rgb(
                color.r as f32,
                color.g as f32,
                color.b as f32,
            ));
        } else {
            // No font specified, reset to default black
            doc.set_text_color(pdf_core::Color::black());
        }

        // Handle word wrapping
        let lines = if let Some(wrap) = &block.word_wrap {
            if let Some(wordcut) = &self.wordcut {
                wordcut.word_wrap(&formatted_text, wrap.max_chars)
            } else {
                pdf_core::simple_word_wrap(&formatted_text, wrap.max_chars)
            }
        } else {
            vec![formatted_text]
        };

        // Determine pages to render on
        let pages = self.resolve_pages(block.pages.as_deref(), doc.page_count());

        // Render text on each page
        let line_height = block
            .word_wrap
            .as_ref()
            .map(|w| w.line_height)
            .unwrap_or(13.5);
        let align = convert_align(block.align);

        for page in pages {
            let mut y = block.position.y;
            for line in &lines {
                doc.insert_text(line, page, block.position.x, y, align)?;
                y += line_height;
            }
        }

        Ok(())
    }

    /// Render a field form block
    fn render_fieldform_block(
        &self,
        doc: &mut PdfDocument,
        block: &FieldFormBlock,
        data: &serde_json::Value,
    ) -> Result<()> {
        // Resolve text content
        let text = if let Some(bind) = &block.bind {
            resolve_binding(bind, data)
                .map(value_to_string)
                .unwrap_or_default()
        } else {
            block.text.clone().unwrap_or_default()
        };

        if text.is_empty() {
            return Ok(());
        }

        // Set font if specified
        if let Some(font) = &block.font {
            self.set_font(doc, font)?;

            // Set text color from font (or default to black)
            let color = font.color.unwrap_or_default();
            doc.set_text_color(pdf_core::Color::rgb(
                color.r as f32,
                color.g as f32,
                color.b as f32,
            ));
        } else {
            // No font specified, reset to default black
            doc.set_text_color(pdf_core::Color::black());
        }

        // Determine pages to render on
        let pages = self.resolve_pages(block.pages.as_deref(), doc.page_count());

        // Render each character with spacing
        let chars: Vec<char> = text.chars().collect();
        let spacing = &block.char_spacing;

        for page in pages {
            let mut x = block.position.x;
            for (i, ch) in chars.iter().enumerate() {
                doc.insert_text(
                    &ch.to_string(),
                    page,
                    x,
                    block.position.y,
                    pdf_core::Align::Center,
                )?;

                // Apply spacing for next character
                if i < spacing.len() {
                    x += spacing[i];
                } else {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Render a table block
    fn render_table_block(
        &self,
        doc: &mut PdfDocument,
        block: &TableBlock,
        data: &serde_json::Value,
    ) -> Result<()> {
        // Resolve rows data
        let rows = if let Some(bind) = &block.bind {
            resolve_binding(bind, data)
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        if rows.is_empty() {
            return Ok(());
        }

        // Set font if specified
        if let Some(font) = &block.font {
            self.set_font(doc, font)?;
        }

        // Determine pages to render on
        let pages = self.resolve_pages(block.pages.as_deref(), doc.page_count());

        // Render rows
        for page in pages {
            let mut y = block.position.y;

            for row in &rows {
                let mut max_lines = 1;

                // First pass: determine maximum lines needed for word wrapping
                for col in &block.columns {
                    if let Some(max_chars) = col.word_wrap {
                        let cell_text =
                            row.get(&col.field).map(value_to_string).unwrap_or_default();

                        let lines = if let Some(wordcut) = &self.wordcut {
                            wordcut.word_wrap(&cell_text, max_chars)
                        } else {
                            pdf_core::simple_word_wrap(&cell_text, max_chars)
                        };
                        max_lines = max_lines.max(lines.len());
                    }
                }

                // Second pass: render cells
                for col in &block.columns {
                    let cell_text = row.get(&col.field).map(value_to_string).unwrap_or_default();

                    let formatted = if let Some(format) = &col.format {
                        self.format_text(&cell_text, Some(format), None)?
                    } else {
                        cell_text
                    };

                    let x = block.position.x + col.x;
                    let align = convert_align(col.align);

                    doc.insert_text(&formatted, page, x, y, align)?;
                }

                y += block.row_height * max_lines as f64;
            }
        }

        Ok(())
    }

    /// Render a QR code block
    fn render_qrcode_block(
        &self,
        doc: &mut PdfDocument,
        block: &QRCodeBlock,
        data: &serde_json::Value,
    ) -> Result<()> {
        // Resolve QR data
        let qr_data = if let Some(bind) = &block.bind {
            resolve_binding(bind, data)
                .map(value_to_string)
                .unwrap_or_default()
        } else {
            block.data.clone().unwrap_or_default()
        };

        if qr_data.is_empty() {
            return Ok(());
        }

        // Generate QR code image
        let qr_image = generate_qr_image(&qr_data, block.error_correction)?;

        // Determine pages to render on
        let pages = self.resolve_pages(block.pages.as_deref(), doc.page_count());

        // Insert image on each page
        for page in pages {
            doc.insert_image(
                &qr_image,
                page,
                block.position.x,
                block.position.y,
                block.size.width,
                block.size.height,
            )?;
        }

        Ok(())
    }

    /// Format text with optional format pattern or special format type
    fn format_text(
        &self,
        text: &str,
        format: Option<&str>,
        format_type: Option<FormatType>,
    ) -> Result<String> {
        // Handle special format types first
        if let Some(ft) = format_type {
            return match ft {
                FormatType::Number => {
                    let n: f64 = text.parse().unwrap_or(0.0);
                    Ok(thai_text::render_float(format.unwrap_or("#,###.##"), n))
                }
                FormatType::ThaiBaht => {
                    let n: f64 = text.parse().unwrap_or(0.0);
                    Ok(thai_text::format_thai_baht(n))
                }
                FormatType::ThaiDateShort => {
                    // Expects YYYY-MM-DD format
                    parse_and_format_date(text, |y, m, d| {
                        thai_text::format_thai_date_short(y, m, d)
                    })
                }
                FormatType::ThaiDateLong => {
                    parse_and_format_date(text, thai_text::format_thai_date_long)
                }
                FormatType::ThaiYear => {
                    let year: i32 = text.parse().unwrap_or(2000);
                    Ok(thai_text::format_thai_year(year))
                }
            };
        }

        // Handle number format pattern
        if let Some(format_pattern) = format {
            if let Ok(n) = text.parse::<f64>() {
                return Ok(thai_text::render_float(format_pattern, n));
            }
        }

        Ok(text.to_string())
    }

    /// Resolve pages to render on
    fn resolve_pages(&self, pages: Option<&[usize]>, total_pages: usize) -> Vec<usize> {
        match pages {
            Some(p) if !p.is_empty() => p.to_vec(),
            _ => (1..=total_pages).collect(),
        }
    }

    /// Set font on document based on Font specification
    fn set_font(&self, doc: &mut PdfDocument, font: &Font) -> Result<()> {
        doc.set_font(&font.family, font.size as f32)?;

        // Set weight and style based on FontStyle enum
        let (weight, style) = match font.style {
            FontStyle::Regular => (FontWeight::Regular, PdfFontStyle::Normal),
            FontStyle::Bold => (FontWeight::Bold, PdfFontStyle::Normal),
            FontStyle::Italic => (FontWeight::Regular, PdfFontStyle::Italic),
            FontStyle::BoldItalic => (FontWeight::Bold, PdfFontStyle::Italic),
        };

        doc.set_font_weight(weight)?;
        doc.set_font_style(style)?;

        Ok(())
    }
}

/// Convert schema Align to pdf_core Align
fn convert_align(align: Align) -> pdf_core::Align {
    match align {
        Align::Left => pdf_core::Align::Left,
        Align::Center => pdf_core::Align::Center,
        Align::Right => pdf_core::Align::Right,
    }
}

/// Check if a JSON value is truthy
fn is_truthy(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        serde_json::Value::String(s) => !s.is_empty(),
        serde_json::Value::Array(a) => !a.is_empty(),
        serde_json::Value::Object(o) => !o.is_empty(),
    }
}

/// Generate QR code image as JPEG bytes
fn generate_qr_image(data: &str, ec: ErrorCorrection) -> Result<Vec<u8>> {
    use image::Luma;
    use qrcode::EcLevel;
    use qrcode::QrCode;

    let ec_level = match ec {
        ErrorCorrection::L => EcLevel::L,
        ErrorCorrection::M => EcLevel::M,
        ErrorCorrection::Q => EcLevel::Q,
        ErrorCorrection::H => EcLevel::H,
    };

    let code = QrCode::with_error_correction_level(data.as_bytes(), ec_level)
        .map_err(|e| TemplateError::ImageError(e.to_string()))?;

    // Render QR code at larger size (200x200 pixels minimum)
    let image = code.render::<Luma<u8>>().min_dimensions(200, 200).build();

    // Convert to JPEG
    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut bytes);

    image::DynamicImage::ImageLuma8(image)
        .write_to(&mut cursor, image::ImageFormat::Jpeg)
        .map_err(|e| TemplateError::ImageError(e.to_string()))?;

    Ok(bytes)
}

/// Parse ISO date string and format using provided function
fn parse_and_format_date<F>(text: &str, format_fn: F) -> Result<String>
where
    F: Fn(i32, u32, u32) -> String,
{
    // Expected format: YYYY-MM-DD
    let parts: Vec<&str> = text.split('-').collect();
    if parts.len() != 3 {
        return Err(TemplateError::RenderError(format!(
            "Invalid date format: {text}. Expected YYYY-MM-DD"
        )));
    }

    let year: i32 = parts[0]
        .parse()
        .map_err(|_| TemplateError::RenderError(format!("Invalid year: {}", parts[0])))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| TemplateError::RenderError(format!("Invalid month: {}", parts[1])))?;
    let day: u32 = parts[2]
        .parse()
        .map_err(|_| TemplateError::RenderError(format!("Invalid day: {}", parts[2])))?;

    Ok(format_fn(year, month, day))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_format_date() {
        let result = parse_and_format_date("2025-01-22", |y, m, d| format!("{y}-{m}-{d}")).unwrap();
        assert_eq!(result, "2025-1-22");
    }

    #[test]
    fn test_parse_date_invalid() {
        let result = parse_and_format_date("invalid", |_, _, _| String::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_is_truthy() {
        assert!(!is_truthy(&serde_json::json!(null)));
        assert!(!is_truthy(&serde_json::json!(false)));
        assert!(is_truthy(&serde_json::json!(true)));
        assert!(!is_truthy(&serde_json::json!(0)));
        assert!(is_truthy(&serde_json::json!(1)));
        assert!(!is_truthy(&serde_json::json!("")));
        assert!(is_truthy(&serde_json::json!("hello")));
        assert!(!is_truthy(&serde_json::json!([])));
        assert!(is_truthy(&serde_json::json!([1])));
        assert!(!is_truthy(&serde_json::json!({})));
        assert!(is_truthy(&serde_json::json!({"key": "value"})));
    }
}
