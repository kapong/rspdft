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
//!
//! // Render
//! const output = template.render({ name: "Test" }, wordcut);
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
    /// Uses the built-in Chulalongkorn University TNC 2017 dictionary
    /// with ~40,000 Thai words. No external file needed.
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
    template: Option<template::Template>,
    pdf_bytes: Option<Vec<u8>>,
    fonts: std::collections::HashMap<String, Vec<u8>>,
}

#[wasm_bindgen]
impl PdfTemplate {
    /// Create a new empty template instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> PdfTemplate {
        PdfTemplate {
            template: None,
            pdf_bytes: None,
            fonts: std::collections::HashMap::new(),
        }
    }

    /// Create template from JSON
    ///
    /// @param json - Template JSON string
    /// @returns PdfTemplate instance
    #[wasm_bindgen(js_name = fromJson)]
    pub fn from_json(json: &str) -> Result<PdfTemplate, JsValue> {
        let template =
            template::parse_template(json).map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(PdfTemplate {
            template: Some(template),
            pdf_bytes: None,
            fonts: std::collections::HashMap::new(),
        })
    }

    /// Load base PDF
    ///
    /// @param data - PDF file bytes (Uint8Array)
    #[wasm_bindgen(js_name = loadBasePdf)]
    pub fn load_base_pdf(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.pdf_bytes = Some(data.to_vec());
        Ok(())
    }

    /// Load font
    ///
    /// @param name - Font identifier
    /// @param data - TTF file bytes (Uint8Array)
    #[wasm_bindgen(js_name = loadFont)]
    pub fn load_font(&mut self, name: &str, data: &[u8]) -> Result<(), JsValue> {
        self.fonts.insert(name.to_string(), data.to_vec());
        Ok(())
    }

    /// Render PDF with data
    ///
    /// @param data - Data object for binding
    /// @param wordcut - ThaiWordcut instance (optional)
    /// @returns PDF bytes (Uint8Array)
    pub fn render(&self, data: JsValue, wordcut: Option<ThaiWordcut>) -> Result<Vec<u8>, JsValue> {
        let template = self
            .template
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Template not loaded"))?;

        let pdf_bytes = self
            .pdf_bytes
            .as_ref()
            .ok_or_else(|| JsValue::from_str("Base PDF not loaded"))?;

        // Parse data from JavaScript object
        let data_value: serde_json::Value = serde_wasm_bindgen::from_value(data)?;

        // Open PDF document
        let mut doc = pdf_core::PdfDocument::open_from_bytes(pdf_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Add fonts
        for (name, font_data) in &self.fonts {
            doc.add_font(name, font_data)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }

        // Create renderer
        let mut renderer = template::TemplateRenderer::new(template);

        // Add wordcut if provided
        if let Some(wc) = &wordcut {
            renderer = renderer.with_wordcut(&wc.inner);
        }

        // Render
        renderer
            .render(&mut doc, &data_value)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Get output bytes
        doc.to_bytes()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

impl Default for PdfTemplate {
    fn default() -> Self {
        Self::new()
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
