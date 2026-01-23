//! PDF Document wrapper

use crate::image::{
    calculate_scaled_dimensions, generate_image_operators, ImageScaleMode, ImageXObject,
};
use crate::text::{generate_text_operators, TextRenderContext};
use crate::{Align, FontData, FontFamily, FontFamilyBuilder, PdfError, Result};
use crate::{FontStyle, FontWeight};
use lopdf::{Dictionary, Document, Object, ObjectId, Stream};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// A segment of text with its associated font
struct TextSegment {
    text: String,
    font_name: String,
}

/// A buffered text operation for deferred encoding
///
/// Text is buffered during rendering and encoded during save,
/// after fonts have been subsetted and glyph IDs remapped.
#[derive(Debug, Clone)]
struct BufferedTextOp {
    /// The text to render
    text: String,
    /// Font name (e.g., "sarabun-bold")
    font_name: String,
    /// Font resource name (e.g., "F1")
    font_resource_name: String,
    /// Page number (1-indexed)
    page: usize,
    /// X coordinate (in PDF coordinates, already converted)
    x: f64,
    /// Y coordinate (in PDF coordinates, already converted)
    y: f64,
    /// Font size in points
    font_size: f32,
    /// Text color
    color: Color,
}

/// RGB Color (values 0.0 - 1.0)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    /// Create a new RGB color (values 0.0 - 1.0)
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// Create color from RGB values (0-255)
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
        }
    }

    /// Black color
    pub fn black() -> Self {
        Self::rgb(0.0, 0.0, 0.0)
    }

    /// White color
    pub fn white() -> Self {
        Self::rgb(1.0, 1.0, 1.0)
    }

    /// Red color
    pub fn red() -> Self {
        Self::rgb(1.0, 0.0, 0.0)
    }

    /// Green color
    pub fn green() -> Self {
        Self::rgb(0.0, 1.0, 0.0)
    }

    /// Blue color
    pub fn blue() -> Self {
        Self::rgb(0.0, 0.0, 1.0)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::black()
    }
}

/// PDF Document wrapper providing high-level operations
pub struct PdfDocument {
    /// The underlying lopdf document
    inner: Document,
    /// Loaded fonts (legacy - for backward compatibility)
    fonts: HashMap<String, FontData>,
    /// Registered font families
    font_families: HashMap<String, FontFamily>,
    /// Current font family name
    current_family: Option<String>,
    /// Current font weight
    current_weight: FontWeight,
    /// Current font style
    current_style: FontStyle,
    /// Current font size
    current_font_size: f32,
    /// Current text color
    current_text_color: Color,
    /// Embedded fonts (font name -> PDF object ID)
    embedded_fonts: HashMap<String, ObjectId>,
    /// Page font resources (page number -> font name -> resource name)
    page_font_resources: HashMap<usize, HashMap<String, String>>,
    /// Next font resource number
    next_font_resource: u32,
    /// Embedded images (data hash -> PDF object ID)
    embedded_images: HashMap<u64, ObjectId>,
    /// Page image resources (page number -> image name -> object ID)
    page_image_resources: HashMap<usize, HashMap<String, ObjectId>>,
    /// Next image resource number
    next_image_resource: u32,
    /// Font fallback chains (family -> list of fallback families)
    font_fallbacks: HashMap<String, Vec<String>>,
    /// Buffered content operators per page (page number -> operators)
    page_content_buffer: HashMap<usize, Vec<u8>>,
    /// Buffered text operations (encoded during save after font subsetting)
    buffered_text_ops: Vec<BufferedTextOp>,
}

impl PdfDocument {
    /// Open a PDF document from a file path
    ///
    /// # Arguments
    /// * `path` - Path to the PDF file
    ///
    /// # Example
    /// ```ignore
    /// let doc = PdfDocument::open("template.pdf")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let inner = Document::load(path).map_err(|e| PdfError::OpenError(e.to_string()))?;

        Ok(Self {
            inner,
            fonts: HashMap::new(),
            font_families: HashMap::new(),
            current_family: None,
            current_weight: FontWeight::default(),
            current_style: FontStyle::default(),
            current_font_size: 12.0,
            current_text_color: Color::default(),
            embedded_fonts: HashMap::new(),
            page_font_resources: HashMap::new(),
            next_font_resource: 1,
            embedded_images: HashMap::new(),
            page_image_resources: HashMap::new(),
            next_image_resource: 1,
            font_fallbacks: HashMap::new(),
            page_content_buffer: HashMap::new(),
            buffered_text_ops: Vec::new(),
        })
    }

    /// Open a PDF document from bytes
    ///
    /// # Arguments
    /// * `data` - PDF file bytes
    pub fn open_from_bytes(data: &[u8]) -> Result<Self> {
        let inner = Document::load_mem(data).map_err(|e| PdfError::OpenError(e.to_string()))?;

        Ok(Self {
            inner,
            fonts: HashMap::new(),
            font_families: HashMap::new(),
            current_family: None,
            current_weight: FontWeight::default(),
            current_style: FontStyle::default(),
            current_font_size: 12.0,
            current_text_color: Color::default(),
            embedded_fonts: HashMap::new(),
            page_font_resources: HashMap::new(),
            next_font_resource: 1,
            embedded_images: HashMap::new(),
            page_image_resources: HashMap::new(),
            next_image_resource: 1,
            font_fallbacks: HashMap::new(),
            page_content_buffer: HashMap::new(),
            buffered_text_ops: Vec::new(),
        })
    }

    /// Get the number of pages in the document
    pub fn page_count(&self) -> usize {
        self.inner.get_pages().len()
    }

    /// Add a TrueType font to the document
    ///
    /// # Arguments
    /// * `name` - Font identifier (used in set_font)
    /// * `ttf_data` - TrueType font file bytes
    ///
    /// # Note
    /// This is the legacy API - it creates a single-variant font family.
    /// For new code, prefer `register_font_family` with `FontFamilyBuilder`.
    pub fn add_font(&mut self, name: &str, ttf_data: &[u8]) -> Result<()> {
        if self.fonts.contains_key(name) || self.font_families.contains_key(name) {
            return Err(PdfError::FontAlreadyExists(name.to_string()));
        }

        let font_data = FontData::from_ttf(name, ttf_data)?;
        self.fonts.insert(name.to_string(), font_data.clone());

        // Also create a single-variant font family for new API compatibility
        let family = FontFamily {
            regular: Some(font_data),
            bold: None,
            italic: None,
            bold_italic: None,
        };
        self.font_families.insert(name.to_string(), family);

        Ok(())
    }

    /// Add a fallback font for a primary font
    ///
    /// # Arguments
    /// * `primary` - Primary font identifier
    /// * `fallback` - Fallback font identifier (must be added with add_font first)
    pub fn add_font_fallback(&mut self, primary: &str, fallback: &str) -> Result<()> {
        if !self.fonts.contains_key(primary) {
            return Err(PdfError::FontNotFound(primary.to_string()));
        }
        if !self.fonts.contains_key(fallback) {
            return Err(PdfError::FontNotFound(fallback.to_string()));
        }

        self.font_fallbacks
            .entry(primary.to_string())
            .or_default()
            .push(fallback.to_string());

        Ok(())
    }

    /// Set font with fallback chain
    ///
    /// # Arguments
    /// * `name` - Primary font identifier
    /// * `size` - Font size in points
    /// * `fallbacks` - List of fallback font identifiers
    pub fn set_font_with_fallback(
        &mut self,
        name: &str,
        size: f32,
        fallbacks: &[String],
    ) -> Result<()> {
        self.set_font(name, size)?;

        // Clear existing fallbacks for this font and set new ones
        self.font_fallbacks.remove(name);
        for fallback in fallbacks {
            self.add_font_fallback(name, fallback)?;
        }

        Ok(())
    }

    /// Register a font family with its variants
    ///
    /// # Arguments
    /// * `name` - Font family name
    /// * `builder` - FontFamilyBuilder with variant data
    ///
    /// # Example
    /// ```ignore
    /// doc.register_font_family("sarabun",
    ///     FontFamilyBuilder::new()
    ///         .regular(std::fs::read("THSarabunNew.ttf")?)
    ///         .bold(std::fs::read("THSarabunNew Bold.ttf")?)
    ///         .italic(std::fs::read("THSarabunNew Italic.ttf")?)
    ///         .bold_italic(std::fs::read("THSarabunNew BoldItalic.ttf")?)
    /// )?;
    /// ```
    pub fn register_font_family(&mut self, name: &str, builder: FontFamilyBuilder) -> Result<()> {
        if self.font_families.contains_key(name) {
            return Err(PdfError::FontAlreadyExists(name.to_string()));
        }

        let family = builder.build(name)?;
        self.font_families.insert(name.to_string(), family);

        Ok(())
    }

    /// Set the current font family and size (new API)
    ///
    /// # Arguments
    /// * `family` - Font family name
    /// * `size` - Font size in points
    ///
    /// # Example
    /// ```ignore
    /// doc.register_font_family("sarabun", FontFamilyBuilder::new().regular(data))?;
    /// doc.set_font("sarabun", 12.0)?;  // Regular 12pt
    /// doc.set_font_weight(FontWeight::Bold)?;  // Now bold 12pt
    /// doc.set_font_size(16.0)?;  // Now bold 16pt
    /// ```
    pub fn set_font(&mut self, family: &str, size: f32) -> Result<()> {
        if !self.font_families.contains_key(family) && !self.fonts.contains_key(family) {
            return Err(PdfError::FontNotFound(family.to_string()));
        }

        self.current_family = Some(family.to_string());
        self.current_font_size = size;

        Ok(())
    }

    /// Set only the font size (keeps current family/weight/style)
    ///
    /// # Arguments
    /// * `size` - Font size in points
    pub fn set_font_size(&mut self, size: f32) -> Result<()> {
        if self.current_family.is_none() {
            return Err(PdfError::FontNotFound("No font family set".to_string()));
        }

        self.current_font_size = size;
        Ok(())
    }

    /// Set the font weight (keeps current family/size/style)
    ///
    /// # Arguments
    /// * `weight` - Font weight (Regular or Bold)
    pub fn set_font_weight(&mut self, weight: FontWeight) -> Result<()> {
        if self.current_family.is_none() {
            return Err(PdfError::FontNotFound("No font family set".to_string()));
        }

        self.current_weight = weight;
        Ok(())
    }

    /// Set the font style (keeps current family/size/weight)
    ///
    /// # Arguments
    /// * `style` - Font style (Normal or Italic)
    pub fn set_font_style(&mut self, style: FontStyle) -> Result<()> {
        if self.current_family.is_none() {
            return Err(PdfError::FontNotFound("No font family set".to_string()));
        }

        self.current_style = style;
        Ok(())
    }

    /// Set the text color
    ///
    /// # Arguments
    /// * `color` - RGB color
    ///
    /// # Example
    /// ```ignore
    /// doc.set_text_color(Color::red())?;
    /// doc.set_text_color(Color::rgb(0.5, 0.5, 0.5))?; // Gray
    /// doc.set_text_color(Color::from_rgb(255, 128, 0))?; // Orange
    /// ```
    pub fn set_text_color(&mut self, color: Color) {
        self.current_text_color = color;
    }

    /// Set font fallback chain for a family
    ///
    /// # Arguments
    /// * `family` - Font family name
    /// * `fallbacks` - List of fallback family names
    pub fn set_font_fallback(&mut self, family: &str, fallbacks: &[String]) -> Result<()> {
        if !self.font_families.contains_key(family) && !self.fonts.contains_key(family) {
            return Err(PdfError::FontNotFound(family.to_string()));
        }

        // Validate all fallback fonts exist
        for fallback in fallbacks {
            if !self.font_families.contains_key(fallback) && !self.fonts.contains_key(fallback) {
                return Err(PdfError::FontNotFound(fallback.clone()));
            }
        }

        // Set the fallback chain
        self.font_fallbacks
            .insert(family.to_string(), fallbacks.to_vec());

        Ok(())
    }

    /// Get the current active font name (for internal use)
    fn get_current_font_name(&self) -> Result<String> {
        let family_name = self
            .current_family
            .as_ref()
            .ok_or_else(|| PdfError::FontNotFound("No font family set".to_string()))?;

        // First try font families (new API)
        if let Some(family) = self.font_families.get(family_name) {
            let variant_name =
                family.get_variant_name(family_name, self.current_weight, self.current_style);
            return Ok(variant_name);
        }

        // Fall back to legacy fonts
        Ok(family_name.clone())
    }

    /// Insert text at a specific position
    ///
    /// # Arguments
    /// * `text` - Text to insert
    /// * `page` - Page number (1-indexed)
    /// * `x` - X coordinate in points
    /// * `y` - Y coordinate in points (from top)
    /// * `align` - Text alignment
    pub fn insert_text(
        &mut self,
        text: &str,
        page: usize,
        x: f64,
        y: f64,
        align: Align,
    ) -> Result<()> {
        let page_count = self.page_count();
        if page == 0 || page > page_count {
            return Err(PdfError::InvalidPage(page, page_count));
        }

        // Skip empty text - nothing to render
        if text.is_empty() {
            return Ok(());
        }

        // Get the current font family name
        let family_name = self
            .current_family
            .as_ref()
            .ok_or_else(|| PdfError::FontNotFound("No font family set".to_string()))?
            .clone();

        // Get the actual font name (variant) to use
        let font_name = self.get_current_font_name()?;

        // Check if fallbacks are configured for this font
        let has_fallbacks = self.font_fallbacks.contains_key(&family_name);

        // Segment text by font availability if fallbacks are configured
        let segments = if has_fallbacks {
            self.segment_text_by_font(text, &family_name, &font_name)
        } else {
            // No fallbacks, treat entire text as single segment
            vec![TextSegment {
                text: text.to_string(),
                font_name: font_name.clone(),
            }]
        };

        // Calculate total text width for alignment
        let mut total_width = 0.0f64;
        for segment in &segments {
            let font_data = self.get_font_data(&segment.font_name)?;
            total_width +=
                font_data.text_width_points(&segment.text, self.current_font_size) as f64;
        }

        // Convert Y coordinate from top-origin to PDF bottom-origin
        let page_height = self.get_page_height(page)?;
        let pdf_y = page_height - y;

        // Calculate starting x position based on alignment
        let start_x = match align {
            Align::Left => x,
            Align::Center => x - (total_width / 2.0),
            Align::Right => x - total_width,
        };

        // Render each segment
        let mut current_x = start_x;
        for segment in &segments {
            // Track characters used in font for subsetting
            {
                let font_data = self.get_font_data_mut(&segment.font_name)?;
                font_data.add_chars(&segment.text);
            }

            // Get or create font reference for this page
            let font_resource_name = self.get_or_create_font_ref(&segment.font_name, page)?;

            // Get segment text width
            let segment_width = {
                let font_data = self.get_font_data(&segment.font_name)?;
                font_data.text_width_points(&segment.text, self.current_font_size) as f64
            };

            // Buffer text operation for deferred encoding (after font subsetting)
            self.buffered_text_ops.push(BufferedTextOp {
                text: segment.text.clone(),
                font_name: segment.font_name.clone(),
                font_resource_name: font_resource_name.clone(),
                page,
                x: current_x,
                y: pdf_y,
                font_size: self.current_font_size,
                color: self.current_text_color,
            });

            // Move to next segment position
            current_x += segment_width;
        }

        Ok(())
    }

    /// Get font data by name (searches both families and legacy fonts)
    fn get_font_data(&self, name: &str) -> Result<&FontData> {
        // First try font families
        for family in self.font_families.values() {
            for variant in [
                &family.regular,
                &family.bold,
                &family.italic,
                &family.bold_italic,
            ]
            .into_iter()
            .flatten()
            {
                if variant.name == name {
                    return Ok(variant);
                }
            }
        }

        // Fall back to legacy fonts
        self.fonts
            .get(name)
            .ok_or_else(|| PdfError::FontNotFound(name.to_string()))
    }

    /// Get mutable font data by name (searches both families and legacy fonts)
    fn get_font_data_mut(&mut self, name: &str) -> Result<&mut FontData> {
        // First try font families
        for family in self.font_families.values_mut() {
            for variant in [
                &mut family.regular,
                &mut family.bold,
                &mut family.italic,
                &mut family.bold_italic,
            ]
            .into_iter()
            .flatten()
            {
                if variant.name == name {
                    return Ok(variant);
                }
            }
        }

        // Fall back to legacy fonts
        self.fonts
            .get_mut(name)
            .ok_or_else(|| PdfError::FontNotFound(name.to_string()))
    }

    /// Segment text by font availability, using fallbacks when needed
    ///
    /// For each character in the text:
    /// 1. Check if primary font (with current variant) has the glyph
    /// 2. If not, check fallback fonts in order
    /// 3. Group consecutive characters with same font into segments
    ///
    /// # Arguments
    /// * `text` - Text to segment
    /// * `family_name` - Font family name
    /// * `variant_name` - Current font variant name (for the primary font)
    fn segment_text_by_font(
        &self,
        text: &str,
        family_name: &str,
        variant_name: &str,
    ) -> Vec<TextSegment> {
        let mut segments = Vec::new();
        let mut current_segment = String::new();
        let mut current_font = variant_name.to_string();
        let mut first_char = true;

        // Get fallback fonts for the primary font family
        let fallbacks = self.font_fallbacks.get(family_name);

        for c in text.chars() {
            // Find the best font for this character
            let font_for_char = if let Ok(font_data) = self.get_font_data(variant_name) {
                if font_data.has_glyph(c) {
                    variant_name.to_string()
                } else {
                    // Try fallback fonts (at family level)
                    let mut found_font = None;
                    if let Some(fallback_list) = fallbacks {
                        for fallback_family in fallback_list {
                            // Get the variant for the fallback family
                            if let Some(fallback_family_data) =
                                self.font_families.get(fallback_family)
                            {
                                if let Some(fallback_variant) = fallback_family_data
                                    .get_variant(self.current_weight, self.current_style)
                                {
                                    if fallback_variant.has_glyph(c) {
                                        found_font = Some(fallback_variant.name.clone());
                                        break;
                                    }
                                }
                            } else if let Some(legacy_font) = self.fonts.get(fallback_family) {
                                // Try legacy fonts too
                                if legacy_font.has_glyph(c) {
                                    found_font = Some(fallback_family.to_string());
                                    break;
                                }
                            }
                        }
                    }
                    found_font.unwrap_or_else(|| variant_name.to_string())
                }
            } else {
                variant_name.to_string()
            };

            if first_char {
                current_font = font_for_char;
                current_segment.push(c);
                first_char = false;
            } else if font_for_char == current_font {
                // Same font, just add to current segment
                current_segment.push(c);
            } else {
                // Different font, start a new segment
                if !current_segment.is_empty() {
                    segments.push(TextSegment {
                        text: current_segment,
                        font_name: current_font,
                    });
                }
                current_font = font_for_char;
                current_segment = c.to_string();
            }
        }

        // Don't forget the last segment
        if !current_segment.is_empty() {
            segments.push(TextSegment {
                text: current_segment,
                font_name: current_font,
            });
        }

        segments
    }

    /// Insert an image at a specific position
    ///
    /// # Arguments
    /// * `data` - Image file bytes (JPEG or PNG)
    /// * `page` - Page number (1-indexed)
    /// * `x` - X coordinate in points
    /// * `y` - Y coordinate in points (from top)
    /// * `width` - Image width in points
    /// * `height` - Image height in points
    pub fn insert_image(
        &mut self,
        data: &[u8],
        page: usize,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Result<()> {
        self.insert_image_scaled(data, page, x, y, width, height, ImageScaleMode::Stretch)
    }

    /// Insert an image with scaling mode
    ///
    /// # Arguments
    /// * `data` - Image file bytes (JPEG or PNG)
    /// * `page` - Page number (1-indexed)
    /// * `x` - X coordinate in points
    /// * `y` - Y coordinate in points (from top)
    /// * `width` - Target width in points
    /// * `height` - Target height in points
    /// * `mode` - Scaling mode
    #[allow(clippy::too_many_arguments)]
    pub fn insert_image_scaled(
        &mut self,
        data: &[u8],
        page: usize,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        mode: ImageScaleMode,
    ) -> Result<()> {
        let page_count = self.page_count();
        if page == 0 || page > page_count {
            return Err(PdfError::InvalidPage(page, page_count));
        }

        // Get or create image resource reference (now returns dimensions too)
        let (image_resource_name, orig_width, orig_height) =
            self.get_or_create_image_ref(data, page)?;

        // Calculate actual display dimensions based on mode
        let (actual_width, actual_height) =
            calculate_scaled_dimensions(orig_width, orig_height, width, height, mode);

        // Convert Y coordinate from top-origin to PDF bottom-origin
        let page_height = self.get_page_height(page)?;
        let pdf_y = page_height - y - actual_height;

        // Generate PDF image drawing operators
        let operators =
            generate_image_operators(&image_resource_name, x, pdf_y, actual_width, actual_height);

        // Buffer content operators (will be flushed at save time)
        self.buffer_content(page, &operators);

        Ok(())
    }

    /// Save the document to a file
    ///
    /// # Arguments
    /// * `path` - Output file path
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        // 1. Subset fonts (creates subsets with only used glyphs)
        self.subset_fonts()?;

        // 2. Encode buffered text with remapped glyph IDs
        self.encode_buffered_text()?;

        // 3. Flush buffered content streams to pages
        self.flush_content_buffers()?;

        // 4. Embed subsetted fonts into PDF
        self.embed_fonts()?;

        self.inner
            .save(path)
            .map_err(|e| PdfError::SaveError(e.to_string()))?;
        Ok(())
    }

    /// Save the document to bytes
    pub fn to_bytes(&mut self) -> Result<Vec<u8>> {
        // 1. Subset fonts (creates subsets with only used glyphs)
        self.subset_fonts()?;

        // 2. Encode buffered text with remapped glyph IDs
        self.encode_buffered_text()?;

        // 3. Flush buffered content streams to pages
        self.flush_content_buffers()?;

        // 4. Embed subsetted fonts into PDF
        self.embed_fonts()?;

        let mut buffer = Vec::new();
        self.inner
            .save_to(&mut buffer)
            .map_err(|e| PdfError::SaveError(e.to_string()))?;

        Ok(buffer)
    }

    /// Create subsets for all fonts that have been used
    ///
    /// This should be called before embed_fonts() to reduce font size.
    /// Only glyphs that were used (tracked via add_chars) will be included.
    fn subset_fonts(&mut self) -> Result<()> {
        // Collect font names that need subsetting
        let mut font_names: Vec<String> = Vec::new();

        // From font families
        for family in self.font_families.values() {
            for font_data in [
                &family.regular,
                &family.bold,
                &family.italic,
                &family.bold_italic,
            ]
            .into_iter()
            .flatten()
            {
                // Only subset fonts that have been used
                if !font_data.used_chars.is_empty() {
                    font_names.push(font_data.name.clone());
                }
            }
        }

        // From legacy fonts
        for (name, font_data) in &self.fonts {
            if !font_data.used_chars.is_empty() {
                font_names.push(name.clone());
            }
        }

        // Deduplicate
        font_names.sort();
        font_names.dedup();

        // Create subset for each font
        for font_name in font_names {
            // Get mutable reference and create subset
            let font_data = self.get_font_data_mut(&font_name)?;
            font_data.create_subset()?;
        }

        Ok(())
    }

    /// Encode buffered text operations and add to content buffers
    ///
    /// This should be called after subset_fonts() to use remapped glyph IDs.
    /// Processes all buffered text ops, encodes them with remapped GIDs,
    /// and adds the resulting operators to the page content buffers.
    fn encode_buffered_text(&mut self) -> Result<()> {
        // Take ownership of buffered ops to avoid borrow issues
        let text_ops: Vec<BufferedTextOp> = std::mem::take(&mut self.buffered_text_ops);

        for op in text_ops {
            // Get font data and encode text with remapped GIDs
            let text_hex = {
                let font_data = self.get_font_data(&op.font_name)?;
                font_data.encode_text_hex_remapped(&op.text)
            };

            // Calculate text width for alignment (already calculated as Left in insert_text)
            let text_width = {
                let font_data = self.get_font_data(&op.font_name)?;
                font_data.text_width_points(&op.text, op.font_size) as f64
            };

            // Create text rendering context
            let ctx = TextRenderContext {
                font_name: op.font_resource_name,
                font_size: op.font_size,
                text_width,
                color: op.color,
            };

            // Generate PDF text operators (position already calculated, use Left)
            let operators = generate_text_operators(&text_hex, op.x, op.y, Align::Left, &ctx);

            // Add to page content buffer
            self.buffer_content(op.page, &operators);
        }

        Ok(())
    }

    /// Embed all added fonts into the PDF
    fn embed_fonts(&mut self) -> Result<()> {
        // Clear embedded fonts to force re-embedding with complete character sets
        self.embedded_fonts.clear();

        // Collect all font names from families and legacy fonts
        let mut font_names: Vec<String> = Vec::new();

        // Add fonts from families (only those with used characters)
        for family in self.font_families.values() {
            for font_data in [
                &family.regular,
                &family.bold,
                &family.italic,
                &family.bold_italic,
            ]
            .into_iter()
            .flatten()
            {
                if !font_data.used_chars.is_empty() {
                    font_names.push(font_data.name.clone());
                }
            }
        }

        // Add legacy fonts (only those with used characters)
        for (font_name, font_data) in &self.fonts {
            if !font_data.used_chars.is_empty() {
                font_names.push(font_name.clone());
            }
        }

        // Deduplicate
        font_names.sort();
        font_names.dedup();

        // Embed each font
        for font_name in font_names {
            self.embed_font_object(&font_name)?;
        }

        // Now add font references to all pages that use them
        self.finalize_page_font_resources()?;

        Ok(())
    }

    /// Embed a single font object into the PDF
    fn embed_font_object(&mut self, font_name: &str) -> Result<ObjectId> {
        let font_data = self.get_font_data(font_name)?;

        // Generate all PDF objects for the font
        let font_objects = font_data.to_pdf_objects()?;

        // Add font file stream
        let font_file_id = self.inner.add_object(font_objects.font_file_stream);

        // Update font descriptor with font file reference
        let mut font_descriptor = font_objects.font_descriptor;
        font_descriptor.set("FontFile2", Object::Reference(font_file_id));
        let font_descriptor_id = self.inner.add_object(font_descriptor);

        // Update CIDFont with font descriptor reference
        let mut cid_font = font_objects.cid_font;
        cid_font.set("FontDescriptor", Object::Reference(font_descriptor_id));
        let cid_font_id = self.inner.add_object(cid_font);

        // Update Type0 font with CIDFont and ToUnicode references
        let mut type0_font = font_objects.type0_font;
        type0_font.set(
            "DescendantFonts",
            Object::Array(vec![Object::Reference(cid_font_id)]),
        );

        // Add ToUnicode stream
        let tounicode_id = self.inner.add_object(font_objects.tounicode_stream);
        type0_font.set("ToUnicode", Object::Reference(tounicode_id));

        let type0_font_id = self.inner.add_object(type0_font);

        // Store the reference
        self.embedded_fonts
            .insert(font_name.to_string(), type0_font_id);

        Ok(type0_font_id)
    }

    /// Get or create a font reference for a specific page
    ///
    /// Returns the resource name (e.g., "F1", "F2") for use in content streams
    pub fn get_or_create_font_ref(&mut self, font_name: &str, page: usize) -> Result<String> {
        // DON'T embed font here - just track the resource name
        // Font will be embedded at save time when all characters are known

        // Check if font is already registered for this page
        let page_resources = self.page_font_resources.entry(page).or_default();

        if let Some(resource_name) = page_resources.get(font_name) {
            return Ok(resource_name.clone());
        }

        // Create new resource name
        let resource_name = format!("F{}", self.next_font_resource);
        self.next_font_resource += 1;

        // Store the mapping (font will be added to page resources at save time)
        page_resources.insert(font_name.to_string(), resource_name.clone());

        Ok(resource_name)
    }

    /// Finalize page font resources after all fonts are embedded
    ///
    /// This is called during save/to_bytes to add font references to page resources
    /// after all fonts have been embedded with complete character sets.
    fn finalize_page_font_resources(&mut self) -> Result<()> {
        // Clone the page_font_resources to avoid borrow issues
        let page_resources: Vec<(usize, Vec<(String, String)>)> = self
            .page_font_resources
            .iter()
            .map(|(&page, fonts)| {
                let font_list: Vec<_> = fonts
                    .iter()
                    .map(|(font_name, resource_name)| (font_name.clone(), resource_name.clone()))
                    .collect();
                (page, font_list)
            })
            .collect();

        for (page, fonts) in page_resources {
            if !fonts.is_empty() {
                self.add_fonts_to_page_resources(page, &fonts)?;
            }
        }

        Ok(())
    }

    /// Add multiple fonts to a page's Resources dictionary in a single operation
    fn add_fonts_to_page_resources(
        &mut self,
        page: usize,
        fonts: &[(String, String)],
    ) -> Result<()> {
        let pages = self.inner.get_pages();
        let page_id = *pages
            .get(&(page as u32))
            .ok_or(PdfError::InvalidPage(page, pages.len()))?;

        // Get the page object
        let page_obj = self.inner.get_object(page_id)?;
        let page_dict = page_obj
            .as_dict()
            .map_err(|_| PdfError::SaveError("Page object is not a dictionary".to_string()))?;

        // Get or create Resources dictionary
        let mut resources_dict = match page_dict.get(b"Resources") {
            Ok(resources) => match resources.as_dict() {
                Ok(dict) => dict.clone(),
                Err(_) => Dictionary::new(),
            },
            Err(_) => Dictionary::new(),
        };

        // Get or create Font dictionary in Resources
        let font_dict = match resources_dict.get(b"Font") {
            Ok(font) => match font.as_dict() {
                Ok(dict) => dict.clone(),
                Err(_) => Dictionary::new(),
            },
            Err(_) => Dictionary::new(),
        };

        // Add all font references
        let mut new_font_dict = font_dict.clone();
        for (font_name, resource_name) in fonts {
            let font_ref = self
                .embedded_fonts
                .get(font_name)
                .ok_or_else(|| PdfError::FontNotFound(font_name.to_string()))?;
            new_font_dict.set(resource_name.as_bytes(), Object::Reference(*font_ref));
        }

        // Update Resources dictionary with the new Font dictionary
        resources_dict.set(b"Font", Object::Dictionary(new_font_dict));

        // Update page dictionary
        let mut new_page_dict = page_dict.clone();
        new_page_dict.set(b"Resources", Object::Dictionary(resources_dict));

        // Replace page object by creating a new one
        self.inner.objects.insert(page_id, new_page_dict.into());

        Ok(())
    }

    /// Get a reference to the underlying lopdf document
    pub fn inner(&self) -> &Document {
        &self.inner
    }

    /// Get a mutable reference to the underlying lopdf document
    pub fn inner_mut(&mut self) -> &mut Document {
        &mut self.inner
    }

    /// Get page height in points
    ///
    /// Extracts the page height from the MediaBox or CropBox.
    /// Handles inherited MediaBox from parent Pages node.
    fn get_page_height(&self, page: usize) -> Result<f64> {
        let pages = self.inner.get_pages();
        let page_id = *pages
            .get(&(page as u32))
            .ok_or(PdfError::InvalidPage(page, pages.len()))?;

        // Try to get MediaBox, following parent chain if needed
        let media_box = self.get_inherited_media_box(page_id)?;

        self.extract_height_from_media_box(&media_box)
    }

    /// Get MediaBox, following parent inheritance chain if needed
    fn get_inherited_media_box(&self, page_id: ObjectId) -> Result<Vec<Object>> {
        let mut current_id = page_id;

        // Follow parent chain up to 10 levels (safety limit)
        for _ in 0..10 {
            let obj = self.inner.get_object(current_id)?;
            let dict = obj
                .as_dict()
                .map_err(|_| PdfError::ParseError("Object is not a dictionary".to_string()))?;

            // Check for MediaBox or CropBox in current dictionary
            if let Ok(media_box) = dict.get(b"MediaBox").or_else(|_| dict.get(b"CropBox")) {
                // Handle both direct array and reference
                let media_box_array = match media_box {
                    Object::Array(arr) => arr.clone(),
                    Object::Reference(ref_id) => {
                        let referred = self.inner.get_object(*ref_id)?;
                        referred
                            .as_array()
                            .map_err(|_| {
                                PdfError::ParseError(
                                    "MediaBox reference is not an array".to_string(),
                                )
                            })?
                            .clone()
                    }
                    _ => return Err(PdfError::ParseError("MediaBox is not an array".to_string())),
                };
                return Ok(media_box_array);
            }

            // Follow Parent reference
            if let Ok(Object::Reference(parent_id)) = dict.get(b"Parent") {
                current_id = *parent_id;
                continue;
            }

            // No parent, break
            break;
        }

        // Fallback: assume A4 page size
        Ok(vec![
            Object::Integer(0),
            Object::Integer(0),
            Object::Real(595.28), // A4 width
            Object::Real(841.89), // A4 height
        ])
    }

    /// Extract height from MediaBox array
    fn extract_height_from_media_box(&self, media_box_array: &[Object]) -> Result<f64> {
        if media_box_array.len() >= 4 {
            let y1 = media_box_array[1]
                .as_f32()
                .map(|v| v as f64)
                .ok()
                .or_else(|| media_box_array[1].as_i64().ok().map(|v| v as f64))
                .ok_or_else(|| PdfError::ParseError("Invalid MediaBox y1".to_string()))?;
            let y2 = media_box_array[3]
                .as_f32()
                .map(|v| v as f64)
                .ok()
                .or_else(|| media_box_array[3].as_i64().ok().map(|v| v as f64))
                .ok_or_else(|| PdfError::ParseError("Invalid MediaBox y2".to_string()))?;
            return Ok(y2 - y1);
        }

        Err(PdfError::ParseError("Invalid MediaBox format".to_string()))
    }

    /// Buffer content operators for a page (written at save time)
    ///
    /// Instead of immediately appending to content stream (which creates orphan objects),
    /// this buffers the operators and flushes them all at once during save.
    fn buffer_content(&mut self, page: usize, content: &[u8]) {
        self.page_content_buffer
            .entry(page)
            .or_default()
            .extend_from_slice(content);
    }

    /// Flush all buffered content to page streams
    ///
    /// Called once during save/to_bytes. Reads each page's existing content stream,
    /// appends all buffered operators, and writes a single new stream object per page.
    fn flush_content_buffers(&mut self) -> Result<()> {
        // Take ownership of buffer to avoid borrow issues
        let buffers: Vec<(usize, Vec<u8>)> = self.page_content_buffer.drain().collect();

        for (page, content) in buffers {
            if !content.is_empty() {
                self.append_to_content_stream(page, &content)?;
            }
        }

        Ok(())
    }

    /// Append content to a page's content stream
    ///
    /// Handles both compressed and uncompressed content streams.
    fn append_to_content_stream(&mut self, page: usize, content: &[u8]) -> Result<()> {
        let pages = self.inner.get_pages();
        let page_id = *pages
            .get(&(page as u32))
            .ok_or(PdfError::InvalidPage(page, pages.len()))?;

        // First pass: extract page dict and gather info about content refs
        // We need to clone data to avoid borrowing issues
        let (existing_content, page_dict_clone) = {
            let page_obj = self.inner.get_object(page_id)?;
            let page_dict = page_obj
                .as_dict()
                .map_err(|_| PdfError::ParseError("Page object is not a dictionary".to_string()))?;

            // Clone the page dict for later modification
            let page_dict_clone = page_dict.clone();

            // Get existing content stream
            let existing_content = match page_dict.get(b"Contents") {
                Ok(contents) => {
                    match contents {
                        Object::Stream(stream) => {
                            // Single stream - decompress if needed
                            stream
                                .decompressed_content()
                                .unwrap_or_else(|_| stream.content.clone())
                        }
                        Object::Reference(ref_id) => {
                            // Contents is a reference to a stream object
                            if let Ok(Object::Stream(stream)) = self.inner.get_object(*ref_id) {
                                stream
                                    .decompressed_content()
                                    .unwrap_or_else(|_| stream.content.clone())
                            } else {
                                Vec::new()
                            }
                        }
                        Object::Array(arr) => {
                            // Array of streams or references - concatenate them
                            let mut combined = Vec::new();
                            for obj in arr {
                                match obj {
                                    Object::Reference(ref_id) => {
                                        if let Ok(Object::Stream(stream)) =
                                            self.inner.get_object(*ref_id)
                                        {
                                            let data = stream
                                                .decompressed_content()
                                                .unwrap_or_else(|_| stream.content.clone());
                                            combined.extend_from_slice(&data);
                                        }
                                    }
                                    Object::Stream(stream) => {
                                        let data = stream
                                            .decompressed_content()
                                            .unwrap_or_else(|_| stream.content.clone());
                                        combined.extend_from_slice(&data);
                                    }
                                    _ => {}
                                }
                            }
                            combined
                        }
                        _ => Vec::new(),
                    }
                }
                Err(_) => Vec::new(),
            };

            (existing_content, page_dict_clone)
        };

        // Append new content
        let mut new_content = existing_content;
        new_content.extend_from_slice(content);

        // Create new stream and add as indirect object
        let new_stream = Stream::new(Dictionary::new(), new_content);
        let stream_id = self.inner.add_object(new_stream);

        // Update page dictionary with reference to stream
        let mut new_page_dict = page_dict_clone;
        new_page_dict.set(b"Contents", Object::Reference(stream_id));

        // Replace page object
        self.inner.objects.insert(page_id, new_page_dict.into());

        Ok(())
    }

    /// Get current font's text width for a string
    ///
    /// Calculates the text width in points based on the current font and size.
    ///
    /// # Arguments
    /// * `text` - The text to measure
    ///
    /// # Returns
    /// Width in points
    ///
    /// # Example
    /// ```ignore
    /// doc.set_font("sarabun", 12.0)?;
    /// let width = doc.get_text_width("Hello")?;
    /// ```
    pub fn get_text_width(&self, text: &str) -> Result<f64> {
        let font_name = self.get_current_font_name()?;
        let font_data = self.get_font_data(&font_name)?;

        Ok(font_data.text_width_points(text, self.current_font_size) as f64)
    }

    /// Get or create an image reference for a specific page
    ///
    /// Returns the resource name (e.g., "Im1", "Im2") and original dimensions.
    /// Images are deduplicated by hash of their data.
    fn get_or_create_image_ref(&mut self, data: &[u8], page: usize) -> Result<(String, u32, u32)> {
        // Calculate hash of image data for deduplication
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let data_hash = hasher.finish();

        // Check if image is already embedded
        if !self.embedded_images.contains_key(&data_hash) {
            // Create XObject from image data
            let xobject = ImageXObject::from_jpeg(data)
                .or_else(|_| ImageXObject::from_png(data))
                .map_err(|e| {
                    PdfError::ImageError(format!("Failed to create image XObject: {e}"))
                })?;

            // Convert to PDF stream and add to document
            let stream = xobject.to_pdf_stream();
            let object_id = self.inner.add_object(stream);

            // Store the reference
            self.embedded_images.insert(data_hash, object_id);
        }

        // Get the object ID
        let object_id = self.embedded_images[&data_hash];

        // Get the image dimensions from the XObject
        let xobject_stream = self.inner.get_object(object_id)?;
        let xobject_dict = &xobject_stream
            .as_stream()
            .map_err(|_| PdfError::ParseError("Image object is not a stream".to_string()))?
            .dict;
        let width = xobject_dict
            .get(b"Width")
            .ok()
            .and_then(|v| v.as_i64().ok())
            .map(|v| v as u32)
            .ok_or_else(|| PdfError::ParseError("Image missing Width".to_string()))?;
        let height = xobject_dict
            .get(b"Height")
            .ok()
            .and_then(|v| v.as_i64().ok())
            .map(|v| v as u32)
            .ok_or_else(|| PdfError::ParseError("Image missing Height".to_string()))?;

        // Check if image is already registered for this page
        let page_resources = self.page_image_resources.entry(page).or_default();

        // Look for existing resource name for this object ID
        for (name, id) in page_resources.iter() {
            if *id == object_id {
                return Ok((name.clone(), width, height));
            }
        }

        // Create new resource name
        let resource_name = format!("Im{}", self.next_image_resource);
        self.next_image_resource += 1;

        // Store the mapping
        page_resources.insert(resource_name.clone(), object_id);

        // Add image to page's Resources dictionary
        self.add_image_to_page_resources(page, &resource_name, object_id)?;

        Ok((resource_name, width, height))
    }

    /// Add image to a specific page's Resources dictionary
    fn add_image_to_page_resources(
        &mut self,
        page: usize,
        resource_name: &str,
        object_id: ObjectId,
    ) -> Result<()> {
        let pages = self.inner.get_pages();
        let page_id = *pages
            .get(&(page as u32))
            .ok_or(PdfError::InvalidPage(page, pages.len()))?;

        // Get the page object
        let page_obj = self.inner.get_object(page_id)?;
        let page_dict = page_obj
            .as_dict()
            .map_err(|_| PdfError::SaveError("Page object is not a dictionary".to_string()))?;

        // Get or create Resources dictionary
        let resources_dict = match page_dict.get(b"Resources") {
            Ok(resources) => match resources.as_dict() {
                Ok(dict) => dict.clone(),
                Err(_) => Dictionary::new(),
            },
            Err(_) => Dictionary::new(),
        };

        // Get or create XObject dictionary in Resources
        let xobject_dict = match resources_dict.get(b"XObject") {
            Ok(xobject) => match xobject.as_dict() {
                Ok(dict) => dict.clone(),
                Err(_) => Dictionary::new(),
            },
            Err(_) => Dictionary::new(),
        };

        // Add image reference
        let mut new_xobject_dict = xobject_dict.clone();
        new_xobject_dict.set(resource_name.as_bytes(), Object::Reference(object_id));

        // Update Resources dictionary
        let mut new_resources = resources_dict.clone();
        new_resources.set(b"XObject", Object::Dictionary(new_xobject_dict));

        // Update page dictionary
        let mut new_page_dict = page_dict.clone();
        new_page_dict.set(b"Resources", Object::Dictionary(new_resources));

        // Replace page object by creating a new one
        self.inner.objects.insert(page_id, new_page_dict.into());

        Ok(())
    }

    /// Add a blank page to the document
    ///
    /// Creates a new blank A4 page (595.28 x 841.89 points) with empty content.
    ///
    /// # Returns
    /// New page number (1-indexed)
    ///
    /// # Example
    /// ```ignore
    /// let mut doc = PdfDocument::open("single-page.pdf")?;
    /// assert_eq!(doc.page_count(), 1);
    ///
    /// let new_page = doc.add_blank_page()?;  // Add blank page
    /// assert_eq!(new_page, 2);
    /// assert_eq!(doc.page_count(), 2);
    ///
    /// doc.save("two-pages.pdf")?;
    /// ```
    pub fn add_blank_page(&mut self) -> Result<usize> {
        // Create empty content stream
        let contents_id = self
            .inner
            .add_object(Object::Stream(Stream::new(Dictionary::new(), vec![])));

        // Get the current page count (this will be the new page number)
        let page_count = self.page_count();

        // Create new page dictionary with A4 MediaBox
        let mut page_dict = Dictionary::new();
        page_dict.set(b"Type", Object::Name(b"Page".to_vec()));
        page_dict.set(
            b"MediaBox",
            Object::Array(vec![
                Object::Real(0.0),
                Object::Real(0.0),
                Object::Real(595.28), // A4 width
                Object::Real(841.89), // A4 height
            ]),
        );
        page_dict.set(b"Resources", Object::Dictionary(Dictionary::new()));
        page_dict.set(b"Contents", Object::Reference(contents_id));

        // Create the new page object
        let new_page_id = self.inner.add_object(Object::Dictionary(page_dict));

        // Get the root Pages object
        let trailer =
            self.inner.trailer.get(b"Root").map_err(|_| {
                PdfError::ParseError("Document trailer missing Root entry".to_string())
            })?;
        let catalog_id = trailer
            .as_reference()
            .map_err(|_| PdfError::ParseError("Root is not a reference".to_string()))?;
        let catalog_obj = self.inner.get_object(catalog_id)?;
        let catalog_dict = catalog_obj
            .as_dict()
            .map_err(|_| PdfError::ParseError("Catalog is not a dictionary".to_string()))?;
        let pages_ref = catalog_dict
            .get(b"Pages")
            .map_err(|_| PdfError::ParseError("Catalog missing Pages entry".to_string()))?;
        let pages_id = pages_ref
            .as_reference()
            .map_err(|_| PdfError::ParseError("Pages is not a reference".to_string()))?;

        // Get the Pages object and update its Kids array
        let pages_obj = self.inner.get_object(pages_id)?;
        let pages_dict = pages_obj
            .as_dict()
            .map_err(|_| PdfError::ParseError("Pages object is not a dictionary".to_string()))?;

        // Get the current Kids array
        let kids = pages_dict
            .get(b"Kids")
            .map_err(|_| PdfError::ParseError("Pages object missing Kids array".to_string()))?;
        let mut kids_array = kids
            .as_array()
            .map_err(|_| PdfError::ParseError("Kids is not an array".to_string()))?
            .clone();

        // Add the new page to the Kids array
        kids_array.push(Object::Reference(new_page_id));

        // Update the Count in the Pages object
        let count = pages_dict
            .get(b"Count")
            .map_err(|_| PdfError::ParseError("Pages object missing Count".to_string()))?;
        let current_count = count
            .as_i64()
            .map_err(|_| PdfError::ParseError("Count is not an integer".to_string()))?;

        // Create updated Pages dictionary
        let mut new_pages_dict = pages_dict.clone();
        new_pages_dict.set(b"Kids", Object::Array(kids_array));
        new_pages_dict.set(b"Count", Object::Integer(current_count + 1));

        // Replace the Pages object
        self.inner.objects.insert(pages_id, new_pages_dict.into());

        // Return the new page number (1-indexed)
        Ok(page_count + 1)
    }

    /// Duplicate a page and return the new page number
    ///
    /// # Arguments
    /// * `page` - Page number to duplicate (1-indexed)
    ///
    /// # Returns
    /// New page number (1-indexed)
    ///
    /// # Example
    /// ```ignore
    /// let mut doc = PdfDocument::open("single-page.pdf")?;
    /// assert_eq!(doc.page_count(), 1);
    ///
    /// let new_page = doc.duplicate_page(1)?;  // Duplicate page 1
    /// assert_eq!(new_page, 2);
    /// assert_eq!(doc.page_count(), 2);
    ///
    /// doc.save("two-pages.pdf")?;
    /// ```
    pub fn duplicate_page(&mut self, page: usize) -> Result<usize> {
        let pages = self.inner.get_pages();
        let page_count = pages.len();

        // Validate page number
        if page == 0 || page > page_count {
            return Err(PdfError::InvalidPage(page, page_count));
        }

        // Get the source page object ID
        let source_page_id = *pages
            .get(&(page as u32))
            .ok_or(PdfError::InvalidPage(page, page_count))?;

        // Get the source page object
        let source_page_obj = self.inner.get_object(source_page_id)?;
        let source_page_dict = source_page_obj
            .as_dict()
            .map_err(|_| PdfError::ParseError("Page object is not a dictionary".to_string()))?;

        // Clone the page dictionary
        let mut new_page_dict = source_page_dict.clone();

        // Clone the content stream if it exists
        if let Ok(contents) = source_page_dict.get(b"Contents") {
            match contents {
                Object::Stream(stream) => {
                    // Clone the stream content
                    let new_stream = Stream::new(stream.dict.clone(), stream.content.clone());
                    new_page_dict.set(b"Contents", new_stream);
                }
                Object::Array(arr) => {
                    // Clone array of streams - collect stream data first to avoid borrow issues
                    let mut streams_to_add = Vec::new();
                    for obj in arr {
                        if let Object::Reference(ref_id) = obj {
                            if let Ok(Object::Stream(stream)) = self.inner.get_object(*ref_id) {
                                let new_stream =
                                    Stream::new(stream.dict.clone(), stream.content.clone());
                                streams_to_add.push(new_stream);
                            }
                        }
                    }

                    // Now add the streams and build the new array
                    let mut new_arr = Vec::new();
                    for stream in streams_to_add {
                        let new_stream_id = self.inner.add_object(stream);
                        new_arr.push(Object::Reference(new_stream_id));
                    }
                    new_page_dict.set(b"Contents", Object::Array(new_arr));
                }
                _ => {}
            }
        }

        // Create the new page object
        let new_page_id = self.inner.add_object(new_page_dict.clone());

        // Get the root Pages object
        let trailer =
            self.inner.trailer.get(b"Root").map_err(|_| {
                PdfError::ParseError("Document trailer missing Root entry".to_string())
            })?;
        let catalog_id = trailer
            .as_reference()
            .map_err(|_| PdfError::ParseError("Root is not a reference".to_string()))?;
        let catalog_obj = self.inner.get_object(catalog_id)?;
        let catalog_dict = catalog_obj
            .as_dict()
            .map_err(|_| PdfError::ParseError("Catalog is not a dictionary".to_string()))?;
        let pages_ref = catalog_dict
            .get(b"Pages")
            .map_err(|_| PdfError::ParseError("Catalog missing Pages entry".to_string()))?;
        let pages_id = pages_ref
            .as_reference()
            .map_err(|_| PdfError::ParseError("Pages is not a reference".to_string()))?;

        // Get the Pages object and update its Kids array
        let pages_obj = self.inner.get_object(pages_id)?;
        let pages_dict = pages_obj
            .as_dict()
            .map_err(|_| PdfError::ParseError("Pages object is not a dictionary".to_string()))?;

        // Get the current Kids array
        let kids = pages_dict
            .get(b"Kids")
            .map_err(|_| PdfError::ParseError("Pages object missing Kids array".to_string()))?;
        let mut kids_array = kids
            .as_array()
            .map_err(|_| PdfError::ParseError("Kids is not an array".to_string()))?
            .clone();

        // Add the new page to the Kids array
        kids_array.push(Object::Reference(new_page_id));

        // Update the Count in the Pages object
        let count = pages_dict
            .get(b"Count")
            .map_err(|_| PdfError::ParseError("Pages object missing Count".to_string()))?;
        let current_count = count
            .as_i64()
            .map_err(|_| PdfError::ParseError("Count is not an integer".to_string()))?;

        // Create updated Pages dictionary
        let mut new_pages_dict = pages_dict.clone();
        new_pages_dict.set(b"Kids", Object::Array(kids_array));
        new_pages_dict.set(b"Count", Object::Integer(current_count + 1));

        // Replace the Pages object
        self.inner.objects.insert(pages_id, new_pages_dict.into());

        // Copy font resource mappings from source page to new page
        // This ensures the cloned content stream's font references remain valid
        if let Some(source_font_resources) = self.page_font_resources.get(&page).cloned() {
            self.page_font_resources
                .insert(page_count + 1, source_font_resources);
        }

        // Also copy image resource mappings
        if let Some(source_image_resources) = self.page_image_resources.get(&page).cloned() {
            self.page_image_resources
                .insert(page_count + 1, source_image_resources);
        }

        // Return the new page number (1-indexed)
        Ok(page_count + 1)
    }

    /// Get all page object IDs in order
    ///
    /// Returns a vector of ObjectId values representing all pages in the document.
    ///
    /// # Example
    /// ```ignore
    /// let doc = PdfDocument::open("document.pdf")?;
    /// let page_ids = doc.get_page_ids();
    /// println!("Document has {} pages", page_ids.len());
    /// ```
    pub fn get_page_ids(&self) -> Vec<ObjectId> {
        let pages = self.inner.get_pages();
        pages.values().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        // Test will need actual PDF file
        // For now, just verify the struct compiles
        let _align = Align::Left;
    }
}
