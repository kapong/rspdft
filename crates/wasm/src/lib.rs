//! WASM bindings for rspdft
//!
//! This crate provides JavaScript-friendly API for:
//! - Loading PDF templates
//! - Thai word segmentation (with embedded dictionary)
//! - Rendering PDFs with data
//!
//! # Example (JavaScript)
//!
//! ```javascript
//! import init, { PdfTemplate, ThaiWordcut, ThaiFormatter } from 'rspdft-wasm';
//!
//! await init();
//!
//! // Use embedded dictionary (recommended - no fetch needed)
//! const wordcut = ThaiWordcut.embedded();
//!
//! // Or load custom dictionary
//! // const wordcut = ThaiWordcut.fromDict(dictText);
//!
//! // Load template
//! const template = PdfTemplate.fromJson(templateJson);
//! template.loadBasePdf(pdfBytes);
//! template.loadFont('sarabun', fontBytes);
//! template.setWordcut(wordcut);
//!
//! // Render
//! const output = template.render({ name: "Test" });
//! ```

use wasm_bindgen::prelude::*;

// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Thai word segmentation utility
#[wasm_bindgen]
pub struct ThaiWordcut {
    inner: thai_text::ThaiWordcut,
}

#[wasm_bindgen]
impl ThaiWordcut {
    /// Create wordcut with embedded Thai dictionary (recommended)
    ///
    /// Uses the built-in LibreOffice/Hunspell Thai dictionary
    /// with ~50,000 Thai words. No external file needed.
    ///
    /// @returns ThaiWordcut instance
    pub fn embedded() -> Result<ThaiWordcut, JsValue> {
        let inner =
            thai_text::ThaiWordcut::embedded().map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(ThaiWordcut { inner })
    }

    /// Load dictionary from string content
    ///
    /// @param dictContent - Dictionary content (one word per line)
    /// @returns ThaiWordcut instance
    #[wasm_bindgen(js_name = fromDict)]
    pub fn from_dict(dict_content: &str) -> Result<ThaiWordcut, JsValue> {
        let inner = thai_text::ThaiWordcut::from_str_content(dict_content)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(ThaiWordcut { inner })
    }

    /// Segment Thai text into words
    ///
    /// @param text - Thai text to segment
    /// @returns Array of words
    pub fn segment(&self, text: &str) -> Vec<JsValue> {
        self.inner
            .segment(text)
            .into_iter()
            .map(|s| JsValue::from_str(&s))
            .collect()
    }

    /// Word wrap Thai text
    ///
    /// @param text - Thai text to wrap
    /// @param maxChars - Maximum characters per line
    /// @returns Array of lines
    #[wasm_bindgen(js_name = wordWrap)]
    pub fn word_wrap(&self, text: &str, max_chars: usize) -> Vec<JsValue> {
        self.inner
            .word_wrap(text, max_chars)
            .into_iter()
            .map(|s| JsValue::from_str(&s))
            .collect()
    }
}

/// Thai text formatting utilities
#[wasm_bindgen]
pub struct ThaiFormatter;

#[wasm_bindgen]
impl ThaiFormatter {
    /// Format a number as Thai text
    ///
    /// @param n - Number to format
    /// @returns Thai text (e.g., "สี่สิบสอง")
    #[wasm_bindgen(js_name = formatNumber)]
    pub fn format_number(n: i64) -> String {
        thai_text::format_thai_number(n)
    }

    /// Format an amount as Thai Baht text
    ///
    /// @param amount - Amount in Baht
    /// @returns Thai text (e.g., "หนึ่งร้อยบาทถ้วน")
    #[wasm_bindgen(js_name = formatBaht)]
    pub fn format_baht(amount: f64) -> String {
        thai_text::format_thai_baht(amount)
    }

    /// Format a date in short Thai format
    ///
    /// @param year - Gregorian year
    /// @param month - Month (1-12)
    /// @param day - Day
    /// @returns Thai date (e.g., "22 ม.ค. 68")
    #[wasm_bindgen(js_name = formatDateShort)]
    pub fn format_date_short(year: i32, month: u32, day: u32) -> String {
        thai_text::format_thai_date_short(year, month, day)
    }

    /// Format a date in long Thai format
    ///
    /// @param year - Gregorian year
    /// @param month - Month (1-12)
    /// @param day - Day
    /// @returns Thai date (e.g., "22 มกราคม 2568")
    #[wasm_bindgen(js_name = formatDateLong)]
    pub fn format_date_long(year: i32, month: u32, day: u32) -> String {
        thai_text::format_thai_date_long(year, month, day)
    }

    /// Format a year in Thai Buddhist calendar
    ///
    /// @param year - Gregorian year
    /// @returns Thai year (e.g., "ปี 2568")
    #[wasm_bindgen(js_name = formatYear)]
    pub fn format_year(year: i32) -> String {
        thai_text::format_thai_year(year)
    }

    /// Render a float with formatting pattern
    ///
    /// @param format - Format pattern (e.g., "#,###.##")
    /// @param n - Number to format
    /// @returns Formatted string
    #[wasm_bindgen(js_name = renderFloat)]
    pub fn render_float(format: &str, n: f64) -> String {
        thai_text::render_float(format, n)
    }
}

/// PDF Template renderer
#[wasm_bindgen]
pub struct PdfTemplate {
    renderer: Option<template::TemplateRenderer>,
    template_json: Option<String>,
    pdf_bytes: Option<Vec<u8>>,
    fonts: std::collections::HashMap<String, Vec<u8>>,
    wordcut: Option<thai_text::ThaiWordcut>,
}

#[wasm_bindgen]
impl PdfTemplate {
    /// Create a new empty template instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> PdfTemplate {
        PdfTemplate {
            renderer: None,
            template_json: None,
            pdf_bytes: None,
            fonts: std::collections::HashMap::new(),
            wordcut: None,
        }
    }

    /// Create template from JSON
    ///
    /// @param json - Template JSON string
    /// @returns PdfTemplate instance
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<PdfTemplate, JsValue> {
        Ok(PdfTemplate {
            renderer: None,
            template_json: Some(json.to_string()),
            pdf_bytes: None,
            fonts: std::collections::HashMap::new(),
            wordcut: None,
        })
    }

    /// Load base PDF
    ///
    /// @param data - PDF file bytes (Uint8Array)
    #[wasm_bindgen(js_name = loadBasePdf)]
    pub fn load_base_pdf(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.pdf_bytes = Some(data.to_vec());
        self.rebuild_renderer()?;
        Ok(())
    }

    /// Load font
    ///
    /// @param name - Font identifier
    /// @param data - TTF file bytes (Uint8Array)
    #[wasm_bindgen(js_name = loadFont)]
    pub fn load_font(&mut self, name: &str, data: &[u8]) -> Result<(), JsValue> {
        self.fonts.insert(name.to_string(), data.to_vec());
        // Update renderer if it exists
        if let Some(ref mut renderer) = self.renderer {
            renderer.add_font(name, data.to_vec());
        }
        Ok(())
    }

    /// Set Thai wordcut for word wrapping
    ///
    /// @param wordcut - ThaiWordcut instance
    #[wasm_bindgen(js_name = setWordcut)]
    pub fn set_wordcut(&mut self, wordcut: &ThaiWordcut) -> Result<(), JsValue> {
        // Clone the inner wordcut
        self.wordcut = Some(wordcut.inner.clone());
        // Update renderer if it exists
        if let Some(ref mut renderer) = self.renderer {
            if let Some(ref wc) = self.wordcut {
                renderer.set_wordcut(wc.clone());
            }
        }
        Ok(())
    }

    /// Rebuild the internal renderer when template_json and pdf_bytes are both available
    fn rebuild_renderer(&mut self) -> Result<(), JsValue> {
        if let (Some(json), Some(pdf)) = (&self.template_json, &self.pdf_bytes) {
            #[cfg(target_arch = "wasm32")]
            let mut renderer = template::TemplateRenderer::new(json, pdf.clone())
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            #[cfg(not(target_arch = "wasm32"))]
            let mut renderer = template::TemplateRenderer::new(json, pdf.clone(), None)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

            // Add all fonts
            for (name, data) in &self.fonts {
                renderer.add_font(name, data.clone());
            }

            // Set wordcut if available
            if let Some(ref wc) = self.wordcut {
                renderer.set_wordcut(wc.clone());
            }

            self.renderer = Some(renderer);
        }
        Ok(())
    }

    /// Render PDF with data
    ///
    /// @param data - Data object for binding
    /// @returns PDF bytes (Uint8Array)
    pub fn render(&self, data: JsValue) -> Result<Vec<u8>, JsValue> {
        let renderer = self.renderer.as_ref().ok_or_else(|| {
            JsValue::from_str(
                "Template or PDF not loaded. Call fromJson() and loadBasePdf() first.",
            )
        })?;

        let data_value: serde_json::Value = serde_wasm_bindgen::from_value(data)?;

        renderer
            .render(&data_value)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Render PDF with data, returning a document for further modification
    ///
    /// Use this when you need to add page-specific content after rendering,
    /// such as a "(COPY)" label only on page 2.
    ///
    /// @param data - Data object for binding
    /// @returns WasmPdfDocument instance for further modification
    #[wasm_bindgen(js_name = renderToDocument)]
    pub fn render_to_document(&self, data: JsValue) -> Result<WasmPdfDocument, JsValue> {
        let renderer = self.renderer.as_ref().ok_or_else(|| {
            JsValue::from_str(
                "Template or PDF not loaded. Call fromJson() and loadBasePdf() first.",
            )
        })?;

        let data_value: serde_json::Value = serde_wasm_bindgen::from_value(data)?;

        let doc = renderer
            .render_to_document(&data_value)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(WasmPdfDocument { inner: doc })
    }

    /// Set font for subsequent text insertions (pre-render template modification)
    ///
    /// @param fontId - Font identifier (must be loaded via loadFont)
    /// @param size - Font size in points (1-255)
    #[wasm_bindgen(js_name = setFont)]
    pub fn set_font(&mut self, font_id: &str, size: u8) -> Result<(), JsValue> {
        let renderer = self.renderer.as_mut().ok_or_else(|| {
            JsValue::from_str("Template not loaded. Call fromJson() and loadBasePdf() first.")
        })?;
        renderer.template_mut().set_font(font_id, size);
        Ok(())
    }

    /// Set font style for subsequent text insertions
    ///
    /// @param style - Style: "regular", "bold", "italic", "bold-italic"
    #[wasm_bindgen(js_name = setFontStyle)]
    pub fn set_font_style(&mut self, style: &str) -> Result<(), JsValue> {
        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| JsValue::from_str("Template not loaded."))?;
        let style_enum = match style {
            "bold" => template::FontStyle::Bold,
            "italic" => template::FontStyle::Italic,
            "bold-italic" => template::FontStyle::BoldItalic,
            _ => template::FontStyle::Regular,
        };
        renderer.template_mut().set_font_style(style_enum);
        Ok(())
    }

    /// Insert text at a specific position
    ///
    /// Useful for adding labels that should only appear on specific pages
    /// or positions (e.g., "(COPY)" label on duplicate side of form).
    ///
    /// @param text - Text to insert
    /// @param page - Page number (1-indexed)
    /// @param x - X position in points
    /// @param y - Y position in points
    /// @param align - Alignment: "left", "center", "right"
    #[wasm_bindgen(js_name = insertText)]
    pub fn insert_text(
        &mut self,
        text: &str,
        page: usize,
        x: f64,
        y: f64,
        align: &str,
    ) -> Result<(), JsValue> {
        let renderer = self.renderer.as_mut().ok_or_else(|| {
            JsValue::from_str("Template not loaded. Call fromJson() and loadBasePdf() first.")
        })?;

        let align_enum = match align {
            "right" => template::Align::Right,
            "center" => template::Align::Center,
            _ => template::Align::Left,
        };

        renderer
            .template_mut()
            .insert_text(text, page, x, y, align_enum);
        Ok(())
    }
}

impl Default for PdfTemplate {
    fn default() -> Self {
        Self::new()
    }
}

/// PDF Document for post-render modifications
///
/// Returned by `PdfTemplate.renderToDocument()`. Use this to make
/// page-specific modifications after rendering, such as adding
/// a "(COPY)" label only on page 2.
///
/// # Example (JavaScript)
/// ```javascript
/// const doc = template.renderToDocument(data);
/// doc.setFont("sarabun", 10);
/// doc.setFontWeight("bold");
/// doc.setTextColor(255, 0, 0); // Red
/// doc.insertText("(COPY)", 2, 550, 15, "right");
/// const pdfBytes = doc.toBytes();
/// ```
#[wasm_bindgen]
pub struct WasmPdfDocument {
    inner: pdf_core::PdfDocument,
}

#[wasm_bindgen]
impl WasmPdfDocument {
    /// Set font for subsequent text insertions
    ///
    /// @param fontId - Font identifier (e.g., "sarabun")
    /// @param size - Font size in points
    #[wasm_bindgen(js_name = setFont)]
    pub fn set_font(&mut self, font_id: &str, size: f32) -> Result<(), JsValue> {
        self.inner
            .set_font(font_id, size)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Set font weight
    ///
    /// @param weight - "regular" or "bold"
    #[wasm_bindgen(js_name = setFontWeight)]
    pub fn set_font_weight(&mut self, weight: &str) -> Result<(), JsValue> {
        let weight_enum = match weight {
            "bold" => pdf_core::FontWeight::Bold,
            _ => pdf_core::FontWeight::Regular,
        };
        self.inner
            .set_font_weight(weight_enum)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Set font style
    ///
    /// @param style - "normal" or "italic"
    #[wasm_bindgen(js_name = setFontStyle)]
    pub fn set_font_style(&mut self, style: &str) -> Result<(), JsValue> {
        let style_enum = match style {
            "italic" => pdf_core::FontStyle::Italic,
            _ => pdf_core::FontStyle::Normal,
        };
        self.inner
            .set_font_style(style_enum)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Set text color (RGB values 0-255)
    ///
    /// @param r - Red component (0-255)
    /// @param g - Green component (0-255)
    /// @param b - Blue component (0-255)
    #[wasm_bindgen(js_name = setTextColor)]
    pub fn set_text_color(&mut self, r: u8, g: u8, b: u8) {
        self.inner.set_text_color(pdf_core::Color::from_rgb(r, g, b));
    }

    /// Insert text at a specific position
    ///
    /// @param text - Text to insert
    /// @param page - Page number (1-indexed)
    /// @param x - X position in points
    /// @param y - Y position in points (from top)
    /// @param align - Alignment: "left", "center", "right"
    #[wasm_bindgen(js_name = insertText)]
    pub fn insert_text(
        &mut self,
        text: &str,
        page: usize,
        x: f64,
        y: f64,
        align: &str,
    ) -> Result<(), JsValue> {
        let align_enum = match align {
            "right" => pdf_core::Align::Right,
            "center" => pdf_core::Align::Center,
            _ => pdf_core::Align::Left,
        };

        self.inner
            .insert_text(text, page, x, y, align_enum)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get page count
    ///
    /// @returns Number of pages in the document
    #[wasm_bindgen(js_name = pageCount)]
    pub fn page_count(&self) -> usize {
        self.inner.page_count()
    }

    /// Convert document to PDF bytes
    ///
    /// @returns PDF bytes (Uint8Array)
    #[wasm_bindgen(js_name = toBytes)]
    pub fn to_bytes(&mut self) -> Result<Vec<u8>, JsValue> {
        self.inner
            .to_bytes()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_thai_formatter() {
        assert_eq!(ThaiFormatter::format_number(42), "สี่สิบสอง");
        assert_eq!(ThaiFormatter::format_baht(100.0), "หนึ่งร้อยบาทถ้วน");
    }

    #[wasm_bindgen_test]
    fn test_render_float() {
        assert_eq!(ThaiFormatter::render_float("#,###.##", 1234.56), "1,234.56");
    }
}
