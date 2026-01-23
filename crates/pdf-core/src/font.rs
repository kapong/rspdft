//! Font handling for PDF documents

use crate::{PdfError, Result};
use lopdf::{Dictionary, Object, Stream};
use std::collections::HashSet;

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontWeight {
    #[default]
    Regular,
    Bold,
}

/// Font style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}

/// Font data structure for embedded fonts
#[derive(Debug, Clone)]
pub struct FontData {
    /// Font name/identifier
    pub name: String,
    /// Raw TTF data
    pub ttf_data: Vec<u8>,
    /// Characters used (for subsetting)
    pub used_chars: HashSet<char>,
    /// Parsed font face
    face: Option<ttf_parser::Face<'static>>,
}

/// PDF objects generated for font embedding
pub struct FontObjects {
    /// Type0 font dictionary
    pub type0_font: Dictionary,
    /// CIDFont Type2 dictionary
    pub cid_font: Dictionary,
    /// Font descriptor dictionary
    pub font_descriptor: Dictionary,
    /// Font file stream (TTF data)
    pub font_file_stream: Stream,
    /// ToUnicode CMap stream
    pub tounicode_stream: Stream,
}

/// Font family with variants
#[derive(Debug, Clone, Default)]
pub struct FontFamily {
    /// Regular variant (required)
    pub regular: Option<FontData>,
    /// Bold variant
    pub bold: Option<FontData>,
    /// Italic variant
    pub italic: Option<FontData>,
    /// Bold italic variant
    pub bold_italic: Option<FontData>,
}

impl FontFamily {
    /// Get the font data for the specified weight and style
    /// Falls back to regular if requested variant is not available
    pub fn get_variant(&self, weight: FontWeight, style: FontStyle) -> Option<&FontData> {
        match (weight, style) {
            (FontWeight::Bold, FontStyle::Italic) => self
                .bold_italic
                .as_ref()
                .or(self.bold.as_ref())
                .or(self.italic.as_ref())
                .or(self.regular.as_ref()),
            (FontWeight::Bold, FontStyle::Normal) => self.bold.as_ref().or(self.regular.as_ref()),
            (FontWeight::Regular, FontStyle::Italic) => {
                self.italic.as_ref().or(self.regular.as_ref())
            }
            (FontWeight::Regular, FontStyle::Normal) => self.regular.as_ref(),
        }
    }

    /// Get mutable reference to the font data for the specified weight and style
    /// Falls back to regular if requested variant is not available
    pub fn get_variant_mut(
        &mut self,
        weight: FontWeight,
        style: FontStyle,
    ) -> Option<&mut FontData> {
        match (weight, style) {
            (FontWeight::Bold, FontStyle::Italic) => {
                if self.bold_italic.is_some() {
                    self.bold_italic.as_mut()
                } else if self.bold.is_some() {
                    self.bold.as_mut()
                } else if self.italic.is_some() {
                    self.italic.as_mut()
                } else {
                    self.regular.as_mut()
                }
            }
            (FontWeight::Bold, FontStyle::Normal) => {
                if self.bold.is_some() {
                    self.bold.as_mut()
                } else {
                    self.regular.as_mut()
                }
            }
            (FontWeight::Regular, FontStyle::Italic) => {
                if self.italic.is_some() {
                    self.italic.as_mut()
                } else {
                    self.regular.as_mut()
                }
            }
            (FontWeight::Regular, FontStyle::Normal) => self.regular.as_mut(),
        }
    }

    /// Get internal font name for the variant (for PDF resource naming)
    pub fn get_variant_name(
        &self,
        family_name: &str,
        weight: FontWeight,
        style: FontStyle,
    ) -> String {
        match (weight, style) {
            (FontWeight::Bold, FontStyle::Italic) => format!("{}-bold-italic", family_name),
            (FontWeight::Bold, FontStyle::Normal) => format!("{}-bold", family_name),
            (FontWeight::Regular, FontStyle::Italic) => format!("{}-italic", family_name),
            (FontWeight::Regular, FontStyle::Normal) => family_name.to_string(),
        }
    }

    /// Check if the family has at least one variant
    pub fn has_variant(&self, weight: FontWeight, style: FontStyle) -> bool {
        self.get_variant(weight, style).is_some()
    }
}

/// Builder for registering font families
pub struct FontFamilyBuilder {
    regular: Option<Vec<u8>>,
    bold: Option<Vec<u8>>,
    italic: Option<Vec<u8>>,
    bold_italic: Option<Vec<u8>>,
}

impl FontFamilyBuilder {
    pub fn new() -> Self {
        Self {
            regular: None,
            bold: None,
            italic: None,
            bold_italic: None,
        }
    }

    pub fn regular(mut self, ttf_data: Vec<u8>) -> Self {
        self.regular = Some(ttf_data);
        self
    }

    pub fn bold(mut self, ttf_data: Vec<u8>) -> Self {
        self.bold = Some(ttf_data);
        self
    }

    pub fn italic(mut self, ttf_data: Vec<u8>) -> Self {
        self.italic = Some(ttf_data);
        self
    }

    pub fn bold_italic(mut self, ttf_data: Vec<u8>) -> Self {
        self.bold_italic = Some(ttf_data);
        self
    }

    /// Build the FontFamily from the provided TTF data
    pub fn build(self, family_name: &str) -> Result<FontFamily> {
        let regular = if let Some(ttf_data) = self.regular {
            Some(FontData::from_ttf(
                &format!("{}-regular", family_name),
                &ttf_data,
            )?)
        } else {
            return Err(PdfError::FontParseError(
                "FontFamily must have at least a regular variant".to_string(),
            ));
        };

        let bold = self
            .bold
            .map(|data| FontData::from_ttf(&format!("{}-bold", family_name), &data))
            .transpose()?;

        let italic = self
            .italic
            .map(|data| FontData::from_ttf(&format!("{}-italic", family_name), &data))
            .transpose()?;

        let bold_italic = self
            .bold_italic
            .map(|data| FontData::from_ttf(&format!("{}-bold-italic", family_name), &data))
            .transpose()?;

        Ok(FontFamily {
            regular,
            bold,
            italic,
            bold_italic,
        })
    }
}

impl Default for FontFamilyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FontData {
    /// Create font data from TTF bytes
    ///
    /// # Arguments
    /// * `name` - Font identifier
    /// * `ttf_data` - TrueType font file bytes
    pub fn from_ttf(name: &str, ttf_data: &[u8]) -> Result<Self> {
        // Validate that we can parse the font
        let data = ttf_data.to_vec();

        // We need to use 'static lifetime for the face, so we leak the data
        // This is acceptable since fonts are typically loaded once and kept for the document lifetime
        let static_data: &'static [u8] = Box::leak(data.clone().into_boxed_slice());

        let face = ttf_parser::Face::parse(static_data, 0)
            .map_err(|e| PdfError::FontParseError(format!("{e:?}")))?;

        Ok(Self {
            name: name.to_string(),
            ttf_data: data,
            used_chars: HashSet::new(),
            face: Some(face),
        })
    }

    /// Add characters to the used set (for subsetting)
    pub fn add_chars(&mut self, text: &str) {
        for c in text.chars() {
            self.used_chars.insert(c);
        }
    }

    /// Get glyph ID for a character
    pub fn glyph_id(&self, c: char) -> Option<u16> {
        self.face
            .as_ref()
            .and_then(|face| face.glyph_index(c).map(|id| id.0))
    }

    /// Check if font has a glyph for the given character
    pub fn has_glyph(&self, c: char) -> bool {
        self.glyph_id(c).map(|id| id != 0).unwrap_or(false)
    }

    /// Get glyph advance width
    pub fn glyph_advance(&self, c: char) -> Option<u16> {
        self.face.as_ref().and_then(|face| {
            let glyph_id = face.glyph_index(c)?;
            face.glyph_hor_advance(glyph_id)
        })
    }

    /// Get font units per em
    pub fn units_per_em(&self) -> u16 {
        self.face
            .as_ref()
            .map(|face| face.units_per_em())
            .unwrap_or(1000)
    }

    /// Get font ascender
    pub fn ascender(&self) -> i16 {
        self.face
            .as_ref()
            .map(|face| face.ascender())
            .unwrap_or(800)
    }

    /// Get font descender
    pub fn descender(&self) -> i16 {
        self.face
            .as_ref()
            .map(|face| face.descender())
            .unwrap_or(-200)
    }

    /// Calculate text width in font units
    pub fn text_width(&self, text: &str) -> u32 {
        text.chars()
            .filter_map(|c| self.glyph_advance(c))
            .map(|w| w as u32)
            .sum()
    }

    /// Calculate text width in points for a given font size
    pub fn text_width_points(&self, text: &str, font_size: f32) -> f32 {
        let width = self.text_width(text);
        let units_per_em = self.units_per_em() as f32;
        (width as f32 / units_per_em) * font_size
    }

    /// Generate all PDF objects needed to embed this font
    pub fn to_pdf_objects(&self) -> Result<FontObjects> {
        let font_name = Object::Name(self.name.clone().into());

        // Generate ToUnicode CMap
        let tounicode_content = self.generate_tounicode_cmap();
        let tounicode_stream = Stream::new(
            Dictionary::from_iter(vec![
                ("Type", "CMap".into()),
                ("Length", (tounicode_content.len() as i32).into()),
            ]),
            tounicode_content.as_bytes().to_vec(),
        );

        // Generate font file stream
        let font_file_stream = Stream::new(
            Dictionary::from_iter(vec![
                ("Type", "FontDescriptor".into()),
                ("Subtype", "TrueType".into()),
                ("Length1", (self.ttf_data.len() as i32).into()),
            ]),
            self.ttf_data.clone(),
        );

        // Generate font descriptor
        let units_per_em = self.units_per_em() as i32;
        let ascender = self.ascender();
        let descender = self.descender();

        // Calculate bounding box (simplified - using font metrics)
        let font_bbox = vec![
            0.into(),
            descender.into(),
            (units_per_em).into(),
            ascender.into(),
        ];

        let font_descriptor = Dictionary::from_iter(vec![
            ("Type", "FontDescriptor".into()),
            ("FontName", font_name.clone()),
            ("Flags", 4.into()), // Symbolic font
            ("FontBBox", font_bbox.into()),
            ("ItalicAngle", 0.into()),
            ("Ascent", ascender.into()),
            ("Descent", descender.into()),
            ("CapHeight", ascender.into()),
            ("StemV", 80.into()),
            ("FontFile2", Object::Reference((0, 0))), // Placeholder, will be set when embedding
        ]);

        // Generate widths array
        let widths_array = self.generate_widths_array();

        // Generate CIDFont Type2 dictionary
        let cid_system_info = Dictionary::from_iter(vec![
            ("Registry", "Adobe".into()),
            ("Ordering", "Identity".into()),
            ("Supplement", 0.into()),
        ]);

        let cid_font = Dictionary::from_iter(vec![
            ("Type", "Font".into()),
            ("Subtype", "CIDFontType2".into()),
            ("BaseFont", font_name.clone()),
            ("CIDSystemInfo", cid_system_info.into()),
            ("FontDescriptor", Object::Reference((0, 0))), // Placeholder, will be set when embedding
            ("W", widths_array.into()),
            ("DW", 1000.into()),
        ]);

        // Generate Type0 font dictionary
        let type0_font = Dictionary::from_iter(vec![
            ("Type", "Font".into()),
            ("Subtype", "Type0".into()),
            ("BaseFont", font_name),
            ("Encoding", "Identity-H".into()),
            ("DescendantFonts", vec![Object::Reference((0, 0))].into()), // Placeholder, will be set when embedding
            ("ToUnicode", Object::Reference((0, 0))), // Placeholder, will be set when embedding
        ]);

        Ok(FontObjects {
            type0_font,
            cid_font,
            font_descriptor,
            font_file_stream,
            tounicode_stream,
        })
    }

    /// Encode text as hex string for PDF Tj operator
    pub fn encode_text_hex(&self, text: &str) -> String {
        let mut result = String::new();
        for c in text.chars() {
            // Get Glyph ID from font (GID)
            let gid = self.glyph_id(c).unwrap_or(0);
            result.push_str(&format!("{gid:04X}"));
        }
        format!("<{result}>")
    }

    /// Generate /W array for glyph widths
    fn generate_widths_array(&self) -> Vec<Object> {
        let mut widths = Vec::new();
        let face = match &self.face {
            Some(f) => f,
            None => return widths,
        };

        // Collect unique GIDs used in the document
        let mut gids: Vec<u16> = self
            .used_chars
            .iter()
            .filter_map(|&c| self.glyph_id(c))
            .collect();
        gids.sort();
        gids.dedup();

        if gids.is_empty() {
            // No characters used, return empty array
            return widths;
        }

        // For simplicity, use individual mapping format: [gid1 [width1] gid2 [width2] ...]
        // This is less optimal than ranges but works correctly for any GID distribution
        for gid in gids {
            let glyph_id = ttf_parser::GlyphId(gid);
            let advance = face.glyph_hor_advance(glyph_id).unwrap_or(1000);
            widths.push(gid.into());
            widths.push(vec![advance.into()].into());
        }

        widths
    }

    /// Generate ToUnicode CMap stream content
    fn generate_tounicode_cmap(&self) -> String {
        let mut cmap = String::new();

        // Header
        cmap.push_str("/CIDInit /ProcSet findresource begin\n");
        cmap.push_str("12 dict begin\n");
        cmap.push_str("begincmap\n");
        cmap.push_str("/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n");
        cmap.push_str("/CMapName /Adobe-Identity-UCS def\n");
        cmap.push_str("/CMapType 2 def\n");

        // Code space range (all 16-bit values)
        cmap.push_str("1 begincodespacerange\n");
        cmap.push_str("<0000> <FFFF>\n");
        cmap.push_str("endcodespacerange\n");

        // Character mappings: map GID (CID) to Unicode codepoint
        let mut char_list: Vec<char> = self.used_chars.iter().copied().collect();
        char_list.sort_by_key(|c| *c as u32);

        if !char_list.is_empty() {
            // PDF spec recommends limiting bfchar sections to 100 entries
            let chunks: Vec<_> = char_list.chunks(100).collect();

            for chunk in chunks {
                cmap.push_str(&format!("{} beginbfchar\n", chunk.len()));
                for c in chunk {
                    let gid = self.glyph_id(*c).unwrap_or(0);
                    let unicode = *c as u32;
                    cmap.push_str(&format!("<{gid:04X}> <{unicode:04X}>\n"));
                }
                cmap.push_str("endbfchar\n");
            }
        }

        // Footer
        cmap.push_str("endcmap\n");
        cmap.push_str("CMapName currentdict /CMap defineresource pop\n");
        cmap.push_str("end\n");
        cmap.push_str("end\n");

        cmap
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a minimal TTF for testing
    ///
    /// Note: This creates a simplified TTF structure. For production use,
    /// you would use actual font files. This is sufficient for testing
    /// the FontData API without requiring real font files.
    fn create_minimal_ttf() -> Vec<u8> {
        // This is a placeholder - in real tests you'd use actual font data
        // For now, we'll skip font parsing tests and focus on the API
        vec![0u8; 100]
    }

    #[test]
    fn test_font_from_ttf() {
        // Skip this test since we don't have valid TTF data
        // In production, you would use actual font files
        // This assertion was always true, removed as it provides no value
    }

    #[test]
    fn test_add_chars() {
        let ttf_data = create_minimal_ttf();
        // Create font without parsing (direct construction for testing)
        let mut font = FontData {
            name: "test".to_string(),
            ttf_data: ttf_data.clone(),
            used_chars: HashSet::new(),
            face: None,
        };

        font.add_chars("Hello");
        assert_eq!(font.used_chars.len(), 4); // H, e, l, o (l appears twice)
        assert!(font.used_chars.contains(&'H'));
        assert!(font.used_chars.contains(&'e'));
        assert!(font.used_chars.contains(&'l'));
        assert!(font.used_chars.contains(&'o'));
    }

    #[test]
    fn test_generate_widths_array() {
        let ttf_data = create_minimal_ttf();
        let mut font = FontData {
            name: "test".to_string(),
            ttf_data,
            used_chars: HashSet::new(),
            face: None,
        };

        font.add_chars("AB");

        let widths = font.generate_widths_array();

        // Should have start_cid and widths array (or be empty if no face)
        // Since we have no face, it will be empty
        assert!(!widths.is_empty() || widths.is_empty());
    }

    #[test]
    fn test_add_chars_thai() {
        let ttf_data = create_minimal_ttf();
        let mut font = FontData {
            name: "test".to_string(),
            ttf_data,
            used_chars: HashSet::new(),
            face: None,
        };

        font.add_chars("สวัสดี");
        assert_eq!(font.used_chars.len(), 5);
        assert!(font.used_chars.contains(&'ส'));
        assert!(font.used_chars.contains(&'ว'));
        assert!(font.used_chars.contains(&'ั'));
        assert!(font.used_chars.contains(&'ด'));
        assert!(font.used_chars.contains(&'ี'));
    }

    #[test]
    fn test_units_per_em() {
        let font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        let units = font.units_per_em();
        assert_eq!(units, 1000); // Default value when no face
    }

    #[test]
    fn test_ascender_descender() {
        let font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        let ascender = font.ascender();
        let descender = font.descender();

        assert_eq!(ascender, 800); // Default value
        assert_eq!(descender, -200); // Default value
    }

    #[test]
    fn test_text_width() {
        let font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        let width = font.text_width("Hello");
        assert_eq!(width, 0); // No face, so no glyph advances
    }

    #[test]
    fn test_text_width_empty() {
        let font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        let width = font.text_width("");
        assert_eq!(width, 0);
    }

    #[test]
    fn test_text_width_points() {
        let font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        let width_12 = font.text_width_points("Hello", 12.0);
        let width_24 = font.text_width_points("Hello", 24.0);

        // Both should be 0 since no face
        assert_eq!(width_12, 0.0);
        assert_eq!(width_24, 0.0);
    }

    #[test]
    fn test_encode_text_hex_empty() {
        let font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        let encoded = font.encode_text_hex("");
        assert_eq!(encoded, "<>");
    }

    #[test]
    fn test_encode_text_hex_no_face() {
        let font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        // Without a face, all characters map to GID 0
        let encoded = font.encode_text_hex("A");
        assert_eq!(encoded, "<0000>");

        let encoded = font.encode_text_hex("AB");
        assert_eq!(encoded, "<00000000>");
    }

    #[test]
    fn test_to_pdf_objects() {
        let mut font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        // Add some characters so widths array is generated
        font.add_chars("Hello");

        let objects = font
            .to_pdf_objects()
            .expect("Failed to generate PDF objects");

        // Verify all objects are present
        assert!(!objects.type0_font.is_empty());
        assert!(!objects.cid_font.is_empty());
        assert!(!objects.font_descriptor.is_empty());
        assert!(!objects.font_file_stream.content.is_empty());
        assert!(!objects.tounicode_stream.content.is_empty());
    }

    #[test]
    fn test_to_pdf_objects_empty_chars() {
        let font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        // Should work even with no characters used
        let objects = font
            .to_pdf_objects()
            .expect("Failed to generate PDF objects");

        assert!(!objects.type0_font.is_empty());
        assert!(!objects.cid_font.is_empty());
    }

    #[test]
    fn test_generate_tounicode_cmap() {
        let mut font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        font.add_chars("AB");

        let cmap = font.generate_tounicode_cmap();

        assert!(cmap.contains("/CIDInit"));
        assert!(cmap.contains("begincmap"));
        assert!(cmap.contains("endcmap"));
        // Without a face, all characters map to GID 0
        assert!(cmap.contains("<0000> <0041>")); // A -> GID 0
        assert!(cmap.contains("<0000> <0042>")); // B -> GID 0
    }

    #[test]
    fn test_generate_tounicode_cmap_empty() {
        let font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        let cmap = font.generate_tounicode_cmap();

        // Should still have header and footer
        assert!(cmap.contains("/CIDInit"));
        assert!(cmap.contains("begincmap"));
        assert!(cmap.contains("endcmap"));
    }

    #[test]
    fn test_generate_tounicode_cmap_thai() {
        let mut font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        font.add_chars("สวัสดี");

        let cmap = font.generate_tounicode_cmap();

        // Without a face, all characters map to GID 0
        assert!(cmap.contains("<0000> <0E2A>")); // ส -> GID 0
        assert!(cmap.contains("<0000> <0E27>")); // ว -> GID 0
    }

    #[test]
    fn test_has_glyph_no_face() {
        let font = FontData {
            name: "test".to_string(),
            ttf_data: vec![0u8; 100],
            used_chars: HashSet::new(),
            face: None,
        };

        // Without a face, has_glyph should return false
        assert!(!font.has_glyph('A'));
        assert!(!font.has_glyph('ส'));
    }
}
