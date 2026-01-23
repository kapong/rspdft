//! Template rendering

use crate::parser::{resolve_binding, value_to_string};
use crate::schema::*;
use crate::{Result, TemplateError};
use pdf_core::{FontFamilyBuilder, FontStyle as PdfFontStyle, FontWeight, PdfDocument};
use thai_text::ThaiWordcut;

/// Template renderer
pub struct TemplateRenderer<'a> {
    /// The template to render
    template: &'a Template,
    /// Thai word segmentation (for word wrapping)
    wordcut: Option<&'a ThaiWordcut>,
}

impl<'a> TemplateRenderer<'a> {
    /// Create a new renderer for a template
    pub fn new(template: &'a Template) -> Self {
        Self {
            template,
            wordcut: None,
        }
    }

    /// Set the Thai wordcut for word wrapping
    pub fn with_wordcut(mut self, wordcut: &'a ThaiWordcut) -> Self {
        self.wordcut = Some(wordcut);
        self
    }

    /// Load all fonts from template into the PDF document
    ///
    /// This uses the new font family API with variants and fallback support.
    pub fn load_fonts(&self, doc: &mut PdfDocument) -> Result<()> {
        for font_def in &self.template.fonts {
            self.load_font_family(doc, font_def)?;
        }

        // Set up fallback chains after all fonts are loaded
        for font_def in &self.template.fonts {
            if !font_def.fallback.is_empty() {
                doc.set_font_fallback(&font_def.id, &font_def.fallback)?;
            }
        }

        Ok(())
    }

    /// Load a single font family from definition
    fn load_font_family(&self, doc: &mut PdfDocument, font_def: &FontDef) -> Result<()> {
        let mut builder = FontFamilyBuilder::new();

        // Check for new format (variants) or legacy format (single source)
        let has_variants = font_def.regular.is_some()
            || font_def.bold.is_some()
            || font_def.italic.is_some()
            || font_def.bold_italic.is_some();

        if has_variants {
            // New format: load specified variants
            if let Some(ref path) = font_def.regular {
                let data = std::fs::read(path).map_err(|e| {
                    TemplateError::RenderError(format!("Failed to read font {}: {}", path, e))
                })?;
                builder = builder.regular(data);
            }
            if let Some(ref path) = font_def.bold {
                let data = std::fs::read(path).map_err(|e| {
                    TemplateError::RenderError(format!("Failed to read font {}: {}", path, e))
                })?;
                builder = builder.bold(data);
            }
            if let Some(ref path) = font_def.italic {
                let data = std::fs::read(path).map_err(|e| {
                    TemplateError::RenderError(format!("Failed to read font {}: {}", path, e))
                })?;
                builder = builder.italic(data);
            }
            if let Some(ref path) = font_def.bold_italic {
                let data = std::fs::read(path).map_err(|e| {
                    TemplateError::RenderError(format!("Failed to read font {}: {}", path, e))
                })?;
                builder = builder.bold_italic(data);
            }
        } else if let Some(ref path) = font_def.source {
            // Legacy format: single font as regular variant
            let data = std::fs::read(path).map_err(|e| {
                TemplateError::RenderError(format!("Failed to read font {}: {}", path, e))
            })?;
            builder = builder.regular(data);
        } else {
            return Err(TemplateError::RenderError(format!(
                "Font '{}' has no source defined",
                font_def.id
            )));
        }

        doc.register_font_family(&font_def.id, builder)?;
        Ok(())
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

    /// Render the template with data
    ///
    /// # Arguments
    /// * `doc` - PDF document to render into
    /// * `data` - Data for binding
    pub fn render(&self, doc: &mut PdfDocument, data: &serde_json::Value) -> Result<()> {
        // Render all blocks
        for block in &self.template.blocks {
            self.render_block(doc, block, data)?;
        }

        // Handle block duplication if configured
        if let Some(duplicate) = &self.template.template.duplicate {
            if duplicate.x != 0.0 || duplicate.y != 0.0 {
                for block in &self.template.blocks {
                    let mut dup_block = block.clone();
                    dup_block.shift_position(duplicate.x, duplicate.y);
                    self.render_block(doc, &dup_block, data)?;
                }
            }
        }

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
        }

        // Handle word wrapping
        let lines = if let Some(wrap) = &block.word_wrap {
            if let Some(wordcut) = self.wordcut {
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

                        let lines = if let Some(wordcut) = self.wordcut {
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
