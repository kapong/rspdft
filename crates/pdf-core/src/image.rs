//! Image handling for PDF documents

use crate::{PdfError, Result};
use image::{DynamicImage, ImageDecoder, ImageReader};
use lopdf::{Dictionary, Stream};
use std::io::Cursor;

impl From<image::ImageError> for PdfError {
    fn from(err: image::ImageError) -> Self {
        PdfError::ImageError(err.to_string())
    }
}

/// Detected image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

/// Image scaling mode for insert_image
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageScaleMode {
    /// Stretch to exact dimensions (current behavior)
    #[default]
    Stretch,
    /// Scale proportionally based on width, auto-calculate height
    FitWidth,
    /// Scale proportionally based on height, auto-calculate width
    FitHeight,
    /// Fit within bounding box, preserving aspect ratio
    FitBox,
}

/// Calculate display dimensions based on scaling mode
///
/// # Arguments
/// * `original_width` - Original image width in pixels
/// * `original_height` - Original image height in pixels
/// * `target_width` - Target width in points
/// * `target_height` - Target height in points
/// * `mode` - Scaling mode
///
/// # Returns
/// (actual_width, actual_height) in points
pub fn calculate_scaled_dimensions(
    original_width: u32,
    original_height: u32,
    target_width: f64,
    target_height: f64,
    mode: ImageScaleMode,
) -> (f64, f64) {
    match mode {
        ImageScaleMode::Stretch => (target_width, target_height),
        ImageScaleMode::FitWidth => {
            let aspect = original_height as f64 / original_width as f64;
            (target_width, target_width * aspect)
        }
        ImageScaleMode::FitHeight => {
            let aspect = original_width as f64 / original_height as f64;
            (target_height * aspect, target_height)
        }
        ImageScaleMode::FitBox => {
            let width_ratio = target_width / original_width as f64;
            let height_ratio = target_height / original_height as f64;
            let scale = width_ratio.min(height_ratio);
            (
                original_width as f64 * scale,
                original_height as f64 * scale,
            )
        }
    }
}

/// Detect image format from magic bytes
pub fn detect_format(data: &[u8]) -> Result<ImageFormat> {
    if data.len() < 8 {
        return Err(PdfError::ImageError("Image data too short".to_string()));
    }

    // Check for JPEG (starts with FF D8 FF)
    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return Ok(ImageFormat::Jpeg);
    }

    // Check for PNG (starts with 89 50 4E 47 0D 0A 1A 0A)
    if data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Ok(ImageFormat::Png);
    }

    Err(PdfError::ImageError("Unknown image format".to_string()))
}

/// Image dimensions
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

/// JPEG info including dimensions and color components
#[derive(Debug, Clone, Copy)]
struct JpegInfo {
    width: u32,
    height: u32,
    num_components: u8,
}

/// Image XObject for PDF embedding
#[derive(Debug, Clone)]
pub struct ImageXObject {
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Color space ("DeviceRGB", "DeviceGray")
    pub color_space: String,
    /// Bits per component
    pub bits_per_component: u8,
    /// PDF filter ("DCTDecode" for JPEG, "FlateDecode" for PNG)
    pub filter: String,
    /// Raw image data (compressed)
    pub data: Vec<u8>,
}

/// Get image dimensions without fully decoding
///
/// # Arguments
/// * `data` - Image file bytes
#[allow(dead_code)]
pub fn get_dimensions(data: &[u8]) -> Result<ImageDimensions> {
    let format = detect_format(data)?;

    match format {
        ImageFormat::Jpeg => {
            let info = get_jpeg_info(data)?;
            Ok(ImageDimensions {
                width: info.width,
                height: info.height,
            })
        }
        ImageFormat::Png => get_png_dimensions(data),
    }
}

/// Get JPEG info including dimensions and color components
fn get_jpeg_info(data: &[u8]) -> Result<JpegInfo> {
    // Simple JPEG dimension parser
    // Look for SOF0 (0xFFC0) or SOF2 (0xFFC2) marker
    // SOF segment structure:
    // - 2 bytes: marker (0xFF, 0xC0-0xCF)
    // - 2 bytes: segment length
    // - 1 byte: precision
    // - 2 bytes: height
    // - 2 bytes: width
    // - 1 byte: number of components (1=grayscale, 3=RGB/YCbCr)
    let mut i = 2;
    while i + 10 < data.len() {
        if data[i] != 0xFF {
            i += 1;
            continue;
        }

        let marker = data[i + 1];

        // SOF markers (baseline, progressive, etc.)
        if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
            let height = u16::from_be_bytes([data[i + 5], data[i + 6]]) as u32;
            let width = u16::from_be_bytes([data[i + 7], data[i + 8]]) as u32;
            let num_components = data[i + 9];
            return Ok(JpegInfo {
                width,
                height,
                num_components,
            });
        }

        // Skip to next marker
        if i + 4 < data.len() {
            let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
            if length < 2 {
                break;
            }
            i += 2 + length;
        } else {
            break;
        }
    }

    Err(PdfError::ImageError(
        "Could not parse JPEG info".to_string(),
    ))
}

/// Get PNG dimensions from header
#[allow(dead_code)]
fn get_png_dimensions(data: &[u8]) -> Result<ImageDimensions> {
    // PNG IHDR chunk starts at byte 8
    // Format: length (4) + "IHDR" (4) + width (4) + height (4)
    if data.len() < 24 {
        return Err(PdfError::ImageError("PNG data too short".to_string()));
    }

    // Check for IHDR chunk
    if &data[12..16] != b"IHDR" {
        return Err(PdfError::ImageError(
            "Invalid PNG: IHDR not found".to_string(),
        ));
    }

    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);

    Ok(ImageDimensions { width, height })
}

impl ImageXObject {
    /// Create XObject from JPEG data
    ///
    /// JPEG images can be embedded directly with DCTDecode filter.
    pub fn from_jpeg(data: &[u8]) -> Result<Self> {
        let info = get_jpeg_info(data)?;

        let color_space = if info.num_components == 1 {
            "DeviceGray".to_string()
        } else {
            "DeviceRGB".to_string()
        };

        Ok(Self {
            width: info.width,
            height: info.height,
            color_space,
            bits_per_component: 8,
            filter: "DCTDecode".to_string(),
            data: data.to_vec(),
        })
    }

    /// Create XObject from PNG data
    ///
    /// PNG images need to be decoded and re-encoded as RGB data with FlateDecode.
    /// Alpha channels are properly blended with white background.
    pub fn from_png(data: &[u8]) -> Result<Self> {
        let cursor = Cursor::new(data);
        let reader = ImageReader::new(cursor).with_guessed_format()?;
        let decoder = reader.into_decoder()?;

        let dims = decoder.dimensions();
        let color_type = decoder.color_type();

        // Decode the image
        let image = DynamicImage::from_decoder(decoder)?;

        // Process based on color type, handling alpha properly
        let (raw_data, color_space) = match color_type {
            // Pure grayscale - keep as grayscale for smaller size
            image::ColorType::L8 | image::ColorType::L16 => {
                let gray = image.to_luma8();
                (gray.into_raw(), "DeviceGray".to_string())
            }
            // Grayscale with alpha - blend with white, output grayscale
            image::ColorType::La8 | image::ColorType::La16 => {
                let la = image.to_luma_alpha8();
                let mut gray_data = Vec::with_capacity((dims.0 * dims.1) as usize);
                for pixel in la.pixels() {
                    let alpha = pixel[1] as f32 / 255.0;
                    let gray = (pixel[0] as f32 * alpha + 255.0 * (1.0 - alpha)) as u8;
                    gray_data.push(gray);
                }
                (gray_data, "DeviceGray".to_string())
            }
            // RGBA - blend with white background, output RGB
            image::ColorType::Rgba8 | image::ColorType::Rgba16 => {
                let rgba = image.to_rgba8();
                let mut rgb_data = Vec::with_capacity((dims.0 * dims.1 * 3) as usize);
                for pixel in rgba.pixels() {
                    let alpha = pixel[3] as f32 / 255.0;
                    let r = (pixel[0] as f32 * alpha + 255.0 * (1.0 - alpha)) as u8;
                    let g = (pixel[1] as f32 * alpha + 255.0 * (1.0 - alpha)) as u8;
                    let b = (pixel[2] as f32 * alpha + 255.0 * (1.0 - alpha)) as u8;
                    rgb_data.push(r);
                    rgb_data.push(g);
                    rgb_data.push(b);
                }
                (rgb_data, "DeviceRGB".to_string())
            }
            // RGB and other types - convert to RGB
            _ => {
                let rgb = image.to_rgb8();
                (rgb.into_raw(), "DeviceRGB".to_string())
            }
        };

        // Compress with FlateDecode (zlib)
        let compressed =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        let mut encoder = compressed;
        std::io::Write::write_all(&mut encoder, &raw_data)?;
        let data = encoder.finish()?;

        Ok(Self {
            width: dims.0,
            height: dims.1,
            color_space,
            bits_per_component: 8,
            filter: "FlateDecode".to_string(),
            data,
        })
    }

    /// Convert to lopdf Stream object
    pub fn to_pdf_stream(&self) -> Stream {
        let mut dict = Dictionary::new();

        dict.set("Type", lopdf::Object::Name(b"XObject".to_vec()));
        dict.set("Subtype", lopdf::Object::Name(b"Image".to_vec()));
        dict.set("Width", self.width as i64);
        dict.set("Height", self.height as i64);
        dict.set(
            "ColorSpace",
            lopdf::Object::Name(self.color_space.as_bytes().to_vec()),
        );
        dict.set("BitsPerComponent", self.bits_per_component as i64);
        dict.set(
            "Filter",
            lopdf::Object::Name(self.filter.as_bytes().to_vec()),
        );
        dict.set("Length", self.data.len() as i64);

        Stream::new(dict, self.data.clone())
    }
}

/// Generate operators to draw image at position
///
/// # Arguments
/// * `image_name` - Image resource name (e.g., "Im1")
/// * `x` - X coordinate in points
/// * `y` - Y coordinate in points (from bottom, PDF coordinates)
/// * `width` - Image width in points
/// * `height` - Image height in points
///
/// # Returns
/// PDF content stream operators as bytes
pub fn generate_image_operators(
    image_name: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Vec<u8> {
    // PDF image drawing operators:
    // q                    - Save graphics state
    // width 0 0 height x y cm  - Concatenate transformation matrix
    // /Im1 Do             - Draw image
    // Q                    - Restore graphics state

    format!("q\n{width} 0 0 {height} {x} {y} cm\n/{image_name} Do\nQ\n").into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_jpeg() {
        let jpeg_header = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        assert_eq!(detect_format(&jpeg_header).unwrap(), ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_png() {
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_format(&png_header).unwrap(), ImageFormat::Png);
    }

    #[test]
    fn test_detect_unknown() {
        let unknown = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(detect_format(&unknown).is_err());
    }

    #[test]
    fn test_png_dimensions() {
        // Minimal valid PNG header with 100x50 dimensions
        let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        png.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]); // IHDR length
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&100u32.to_be_bytes()); // width
        png.extend_from_slice(&50u32.to_be_bytes()); // height

        let dims = get_png_dimensions(&png).unwrap();
        assert_eq!(dims.width, 100);
        assert_eq!(dims.height, 50);
    }

    #[test]
    fn test_generate_image_operators() {
        let ops = generate_image_operators("Im1", 100.0, 200.0, 50.0, 75.0);
        let ops_str = String::from_utf8(ops).unwrap();

        assert!(ops_str.contains("q"));
        assert!(ops_str.contains("50 0 0 75 100 200 cm"));
        assert!(ops_str.contains("/Im1 Do"));
        assert!(ops_str.contains("Q"));
    }

    #[test]
    fn test_image_xobject_to_pdf_stream() {
        let xobject = ImageXObject {
            width: 100,
            height: 50,
            color_space: "DeviceRGB".to_string(),
            bits_per_component: 8,
            filter: "DCTDecode".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };

        let stream = xobject.to_pdf_stream();
        let dict = stream.dict;

        assert_eq!(dict.get(b"Type").unwrap().as_name().unwrap(), b"XObject");
        assert_eq!(dict.get(b"Subtype").unwrap().as_name().unwrap(), b"Image");
        assert_eq!(dict.get(b"Width").unwrap().as_i64().unwrap(), 100);
        assert_eq!(dict.get(b"Height").unwrap().as_i64().unwrap(), 50);
        // ColorSpace is stored as a name without the leading slash
        assert_eq!(
            dict.get(b"ColorSpace").unwrap().as_name().unwrap(),
            b"DeviceRGB"
        );
        assert_eq!(dict.get(b"BitsPerComponent").unwrap().as_i64().unwrap(), 8);
        assert_eq!(
            dict.get(b"Filter").unwrap().as_name().unwrap(),
            b"DCTDecode"
        );
        assert_eq!(stream.content, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_detect_format_too_short() {
        let data = vec![0x00, 0x00, 0x00];
        assert!(detect_format(&data).is_err());
    }

    #[test]
    fn test_detect_format_jpeg_variants() {
        // JPEG SOI marker is FF D8 FF
        // Third byte can vary (E0, E1, E2, etc.)
        let jpeg1 = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
        let jpeg2 = vec![0xFF, 0xD8, 0xFF, 0xE1, 0x00, 0x10, 0x4A, 0x46];
        let jpeg3 = vec![0xFF, 0xD8, 0xFF, 0xFE, 0x00, 0x10, 0x4A, 0x46];

        assert_eq!(detect_format(&jpeg1).unwrap(), ImageFormat::Jpeg);
        assert_eq!(detect_format(&jpeg2).unwrap(), ImageFormat::Jpeg);
        assert_eq!(detect_format(&jpeg3).unwrap(), ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_format_not_jpeg() {
        // Starts with FF but not JPEG
        let data = vec![0xFF, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(detect_format(&data).is_err());
    }

    #[test]
    fn test_detect_format_not_png() {
        // Starts with 0x89 but not PNG
        let data = vec![0x89, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(detect_format(&data).is_err());
    }

    #[test]
    fn test_get_jpeg_info_invalid() {
        // Invalid JPEG data
        let data = vec![0xFF, 0xD8, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(get_jpeg_info(&data).is_err());
    }

    #[test]
    fn test_get_jpeg_info_too_short() {
        let data = vec![0xFF, 0xD8, 0xFF];
        assert!(get_jpeg_info(&data).is_err());
    }

    #[test]
    fn test_get_png_dimensions_too_short() {
        let data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(get_png_dimensions(&data).is_err());
    }

    #[test]
    fn test_get_png_dimensions_no_ihdr() {
        let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x0D]); // IHDR length
        data.extend_from_slice(b"NOTI"); // Not IHDR
        assert!(get_png_dimensions(&data).is_err());
    }

    #[test]
    fn test_get_dimensions_jpeg() {
        // Create minimal JPEG with known dimensions
        let jpeg = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xC0, // SOF0
            0x00, 0x11, // Length
            0x08, // Precision
            0x00, 0x64, // Height (100)
            0x00, 0xC8, // Width (200)
            0x03, // Components
            0x01, 0x22, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01, 0xFF, 0xD9, // EOI
        ];

        let dims = get_dimensions(&jpeg).unwrap();
        assert_eq!(dims.width, 200);
        assert_eq!(dims.height, 100);
    }

    #[test]
    fn test_get_dimensions_png() {
        let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        png.extend_from_slice(&0x0000000Du32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&150u32.to_be_bytes()); // Width
        png.extend_from_slice(&75u32.to_be_bytes()); // Height
        png.extend_from_slice(&[8, 2, 0, 0, 0]);
        png.extend_from_slice(&0x00000000u32.to_be_bytes()); // CRC placeholder

        let dims = get_dimensions(&png).unwrap();
        assert_eq!(dims.width, 150);
        assert_eq!(dims.height, 75);
    }

    #[test]
    fn test_image_xobject_clone() {
        let xobject = ImageXObject {
            width: 100,
            height: 50,
            color_space: "DeviceRGB".to_string(),
            bits_per_component: 8,
            filter: "DCTDecode".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };

        let cloned = xobject.clone();
        assert_eq!(cloned.width, xobject.width);
        assert_eq!(cloned.height, xobject.height);
        assert_eq!(cloned.color_space, xobject.color_space);
        assert_eq!(cloned.bits_per_component, xobject.bits_per_component);
        assert_eq!(cloned.filter, xobject.filter);
        assert_eq!(cloned.data, xobject.data);
    }

    #[test]
    fn test_image_xobject_debug() {
        let xobject = ImageXObject {
            width: 100,
            height: 50,
            color_space: "DeviceRGB".to_string(),
            bits_per_component: 8,
            filter: "DCTDecode".to_string(),
            data: vec![1, 2, 3],
        };

        let debug_str = format!("{xobject:?}");
        assert!(debug_str.contains("ImageXObject"));
        assert!(debug_str.contains("100"));
        assert!(debug_str.contains("50"));
    }

    #[test]
    fn test_generate_image_operators_zero_size() {
        let ops = generate_image_operators("Im1", 0.0, 0.0, 0.0, 0.0);
        let ops_str = String::from_utf8(ops).unwrap();

        assert!(ops_str.contains("q"));
        assert!(ops_str.contains("0 0 0 0 0 0 cm"));
        assert!(ops_str.contains("/Im1 Do"));
        assert!(ops_str.contains("Q"));
    }

    #[test]
    fn test_generate_image_operators_negative_position() {
        let ops = generate_image_operators("Im1", -50.0, -100.0, 100.0, 200.0);
        let ops_str = String::from_utf8(ops).unwrap();

        assert!(ops_str.contains("-50 -100"));
        assert!(ops_str.contains("100 0 0 200"));
    }

    #[test]
    fn test_generate_image_operators_large_size() {
        let ops = generate_image_operators("Im1", 0.0, 0.0, 1000.0, 2000.0);
        let ops_str = String::from_utf8(ops).unwrap();

        assert!(ops_str.contains("1000 0 0 2000"));
    }

    #[test]
    fn test_image_format_equality() {
        assert_eq!(ImageFormat::Jpeg, ImageFormat::Jpeg);
        assert_eq!(ImageFormat::Png, ImageFormat::Png);
        assert_ne!(ImageFormat::Jpeg, ImageFormat::Png);
    }

    #[test]
    fn test_image_dimensions() {
        let dims = ImageDimensions {
            width: 1920,
            height: 1080,
        };

        assert_eq!(dims.width, 1920);
        assert_eq!(dims.height, 1080);
    }

    #[test]
    fn test_image_xobject_with_gray_colorspace() {
        let xobject = ImageXObject {
            width: 100,
            height: 50,
            color_space: "DeviceGray".to_string(),
            bits_per_component: 8,
            filter: "FlateDecode".to_string(),
            data: vec![1, 2, 3],
        };

        let stream = xobject.to_pdf_stream();
        let dict = stream.dict;

        assert_eq!(
            dict.get(b"ColorSpace").unwrap().as_name().unwrap(),
            b"DeviceGray"
        );
        assert_eq!(
            dict.get(b"Filter").unwrap().as_name().unwrap(),
            b"FlateDecode"
        );
    }

    #[test]
    fn test_image_xobject_empty_data() {
        let xobject = ImageXObject {
            width: 0,
            height: 0,
            color_space: "DeviceRGB".to_string(),
            bits_per_component: 8,
            filter: "DCTDecode".to_string(),
            data: vec![],
        };

        let stream = xobject.to_pdf_stream();
        assert_eq!(stream.content.len(), 0);
    }

    #[test]
    fn test_image_scale_mode_default() {
        assert_eq!(ImageScaleMode::default(), ImageScaleMode::Stretch);
    }

    #[test]
    fn test_image_scale_mode_equality() {
        assert_eq!(ImageScaleMode::Stretch, ImageScaleMode::Stretch);
        assert_eq!(ImageScaleMode::FitWidth, ImageScaleMode::FitWidth);
        assert_eq!(ImageScaleMode::FitHeight, ImageScaleMode::FitHeight);
        assert_eq!(ImageScaleMode::FitBox, ImageScaleMode::FitBox);
        assert_ne!(ImageScaleMode::Stretch, ImageScaleMode::FitBox);
    }

    #[test]
    fn test_calculate_scaled_dimensions_stretch() {
        // Stretch mode should return exact target dimensions
        let (w, h) = calculate_scaled_dimensions(800, 600, 100.0, 200.0, ImageScaleMode::Stretch);
        assert_eq!(w, 100.0);
        assert_eq!(h, 200.0);
    }

    #[test]
    fn test_calculate_scaled_dimensions_fit_width() {
        // 800x600 image (4:3 aspect) scaled to width 100 should have height 75
        let (w, h) = calculate_scaled_dimensions(800, 600, 100.0, 200.0, ImageScaleMode::FitWidth);
        assert_eq!(w, 100.0);
        assert_eq!(h, 75.0);
    }

    #[test]
    fn test_calculate_scaled_dimensions_fit_height() {
        // 800x600 image (4:3 aspect) scaled to height 150 should have width 200
        let (w, h) = calculate_scaled_dimensions(800, 600, 100.0, 150.0, ImageScaleMode::FitHeight);
        assert_eq!(w, 200.0);
        assert_eq!(h, 150.0);
    }

    #[test]
    fn test_calculate_scaled_dimensions_fit_box_width_limited() {
        // 800x600 image in 100x200 box - width is the limiting factor
        // Scale factor = 100/800 = 0.125
        // Result: 100 x 75
        let (w, h) = calculate_scaled_dimensions(800, 600, 100.0, 200.0, ImageScaleMode::FitBox);
        assert_eq!(w, 100.0);
        assert_eq!(h, 75.0);
    }

    #[test]
    fn test_calculate_scaled_dimensions_fit_box_height_limited() {
        // 600x800 image in 200x100 box - height is the limiting factor
        // Scale factor = 100/800 = 0.125
        // Result: 75 x 100
        let (w, h) = calculate_scaled_dimensions(600, 800, 200.0, 100.0, ImageScaleMode::FitBox);
        assert_eq!(w, 75.0);
        assert_eq!(h, 100.0);
    }

    #[test]
    fn test_calculate_scaled_dimensions_fit_box_square() {
        // 400x400 image in 100x100 box - both ratios equal
        let (w, h) = calculate_scaled_dimensions(400, 400, 100.0, 100.0, ImageScaleMode::FitBox);
        assert_eq!(w, 100.0);
        assert_eq!(h, 100.0);
    }
}
