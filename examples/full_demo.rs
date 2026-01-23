//! Full Demo - Demonstrates the rspdft font family API
//!
//! This example shows:
//! - Font family registration with variants (regular, bold, italic, bold-italic)
//! - Font weight and style switching
//! - Font fallback for symbols
//! - Image and QR code insertion
//! - Text alignment
//! - Thai text rendering
//!
//! Run with: cargo run --example full_demo -p pdf-core

use lopdf::dictionary;
use pdf_core::{Align, FontFamilyBuilder, FontStyle, FontWeight, ImageScaleMode, PdfDocument};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    std::fs::create_dir_all("output")?;

    // Create a blank PDF document
    let pdf_data = create_blank_pdf();
    let mut doc = PdfDocument::open_from_bytes(&pdf_data)?;

    // ========================================
    // 1. Register Font Families
    // ========================================

    // Register Sarabun font family with all variants
    doc.register_font_family(
        "sarabun",
        FontFamilyBuilder::new()
            .regular(std::fs::read("fonts/THSarabunNew.ttf")?)
            .bold(std::fs::read("fonts/THSarabunNew Bold.ttf")?)
            .italic(std::fs::read("fonts/THSarabunNew Italic.ttf")?)
            .bold_italic(std::fs::read("fonts/THSarabunNew BoldItalic.ttf")?),
    )?;

    // Register Noto Sans Symbols 2 for Unicode symbols
    doc.register_font_family(
        "symbols",
        FontFamilyBuilder::new().regular(std::fs::read("fonts/NotoSansSymbols2-Regular.ttf")?),
    )?;

    // Set fallback chain: sarabun -> symbols
    doc.set_font_fallback("sarabun", &["symbols".to_string()])?;

    // ========================================
    // 2. Page 1: Font Variants Demo
    // ========================================

    // Title
    doc.set_font("sarabun", 28.0)?;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text("rspdft Font Demo", 1, 297.5, 50.0, Align::Center)?;

    // Subtitle
    doc.set_font("sarabun", 14.0)?; // Resets to Regular/Normal
    doc.insert_text(
        "Demonstrating Font Family API",
        1,
        297.5,
        75.0,
        Align::Center,
    )?;

    // Section: Font Variants
    let mut y = 120.0;
    doc.set_font("sarabun", 18.0)?;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text("1. Font Variants", 1, 50.0, y, Align::Left)?;

    y += 30.0;
    doc.set_font("sarabun", 14.0)?; // Regular
    doc.insert_text(
        "Regular: The quick brown fox jumps over the lazy dog.",
        1,
        50.0,
        y,
        Align::Left,
    )?;

    y += 25.0;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text(
        "Bold: The quick brown fox jumps over the lazy dog.",
        1,
        50.0,
        y,
        Align::Left,
    )?;

    y += 25.0;
    doc.set_font_weight(FontWeight::Regular)?;
    doc.set_font_style(FontStyle::Italic)?;
    doc.insert_text(
        "Italic: The quick brown fox jumps over the lazy dog.",
        1,
        50.0,
        y,
        Align::Left,
    )?;

    y += 25.0;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.set_font_style(FontStyle::Italic)?;
    doc.insert_text(
        "Bold Italic: The quick brown fox jumps over the lazy dog.",
        1,
        50.0,
        y,
        Align::Left,
    )?;

    // Section: Thai Text
    y += 40.0;
    doc.set_font("sarabun", 18.0)?;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text("2. Thai Text", 1, 50.0, y, Align::Left)?;

    y += 30.0;
    doc.set_font("sarabun", 14.0)?;
    doc.insert_text(
        "Regular: สวัสดีชาวโลก ยินดีต้อนรับสู่ rspdft",
        1,
        50.0,
        y,
        Align::Left,
    )?;

    y += 25.0;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text("Bold: สวัสดีชาวโลก ยินดีต้อนรับสู่ rspdft", 1, 50.0, y, Align::Left)?;

    y += 25.0;
    doc.set_font_weight(FontWeight::Regular)?;
    doc.set_font_style(FontStyle::Italic)?;
    doc.insert_text(
        "Italic: สวัสดีชาวโลก ยินดีต้อนรับสู่ rspdft",
        1,
        50.0,
        y,
        Align::Left,
    )?;

    // Section: Font Fallback (Symbols)
    y += 40.0;
    doc.set_font("sarabun", 18.0)?;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text("3. Font Fallback (Symbols)", 1, 50.0, y, Align::Left)?;

    y += 30.0;
    doc.set_font("sarabun", 14.0)?;
    doc.insert_text(
        "Checkmark: ✓  Cross: ✗  Star: ★  Heart: ♥  Arrow: ➡",
        1,
        50.0,
        y,
        Align::Left,
    )?;

    y += 25.0;
    doc.insert_text(
        "Mixed: Pass ✓ | Fail ✗ | Rating: ★★★★☆",
        1,
        50.0,
        y,
        Align::Left,
    )?;

    // Section: Text Alignment
    y += 40.0;
    doc.set_font("sarabun", 18.0)?;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text("4. Text Alignment", 1, 50.0, y, Align::Left)?;

    y += 30.0;
    doc.set_font("sarabun", 14.0)?;
    doc.insert_text("Left aligned text", 1, 50.0, y, Align::Left)?;

    y += 25.0;
    doc.insert_text("Center aligned text", 1, 297.5, y, Align::Center)?;

    y += 25.0;
    doc.insert_text("Right aligned text", 1, 545.0, y, Align::Right)?;

    // Section: Font Size Changes
    y += 40.0;
    doc.set_font("sarabun", 18.0)?;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text("5. Font Size Changes", 1, 50.0, y, Align::Left)?;

    y += 30.0;
    doc.set_font("sarabun", 10.0)?;
    doc.insert_text("10pt: Small text for footnotes", 1, 50.0, y, Align::Left)?;

    y += 20.0;
    doc.set_font_size(14.0)?; // Just change size, keep Regular style
    doc.insert_text("14pt: Normal body text", 1, 50.0, y, Align::Left)?;

    y += 25.0;
    doc.set_font_size(20.0)?;
    doc.insert_text("20pt: Larger heading text", 1, 50.0, y, Align::Left)?;

    y += 35.0;
    doc.set_font_size(28.0)?;
    doc.insert_text("28pt: Title text", 1, 50.0, y, Align::Left)?;

    // ========================================
    // 3. Page 2: Images and QR Code Demo
    // ========================================

    let page2 = doc.add_blank_page()?;
    println!("Added page 2: {page2}");

    // Title for page 2
    doc.set_font("sarabun", 28.0)?;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text("Images & QR Code Demo", page2, 297.5, 50.0, Align::Center)?;

    // Create a simple test image (gradient)
    let test_image = create_test_image()?;

    // Section: Image Scaling
    y = 100.0;
    doc.set_font("sarabun", 18.0)?;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text(
        "1. Image with Different Scaling Modes",
        page2,
        50.0,
        y,
        Align::Left,
    )?;

    y += 30.0;
    doc.set_font("sarabun", 12.0)?;
    doc.insert_text("Stretch (100x80)", page2, 50.0, y, Align::Left)?;
    doc.insert_image_scaled(
        &test_image,
        page2,
        50.0,
        y + 15.0,
        100.0,
        80.0,
        ImageScaleMode::Stretch,
    )?;

    doc.insert_text("FitWidth (100x80)", page2, 200.0, y, Align::Left)?;
    doc.insert_image_scaled(
        &test_image,
        page2,
        200.0,
        y + 15.0,
        100.0,
        80.0,
        ImageScaleMode::FitWidth,
    )?;

    doc.insert_text("FitBox (100x80)", page2, 350.0, y, Align::Left)?;
    doc.insert_image_scaled(
        &test_image,
        page2,
        350.0,
        y + 15.0,
        100.0,
        80.0,
        ImageScaleMode::FitBox,
    )?;

    // Section: QR Code
    y = 280.0;
    doc.set_font("sarabun", 18.0)?;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.insert_text("2. QR Code", page2, 50.0, y, Align::Left)?;

    y += 30.0;
    let qr_image = create_qr_code("https://github.com/kapong/rspdft")?;
    doc.insert_image(&qr_image, page2, 50.0, y, 100.0, 100.0)?;

    doc.set_font("sarabun", 12.0)?;
    doc.insert_text(
        "Scan me! ➡ github.com/kapong/rspdft",
        page2,
        160.0,
        y + 50.0,
        Align::Left,
    )?;

    // Footer
    doc.set_font("sarabun", 10.0)?;
    doc.insert_text("Generated by rspdft", page2, 297.5, 800.0, Align::Center)?;

    // ========================================
    // 4. Save the document
    // ========================================

    doc.save("output/full_demo.pdf")?;
    println!("Saved to output/full_demo.pdf");
    println!("  - Page 1: Font variants, Thai text, symbols, alignment");
    println!("  - Page 2: Images, QR code");

    Ok(())
}

/// Create a minimal blank PDF document
fn create_blank_pdf() -> Vec<u8> {
    let mut doc = lopdf::Document::new();

    // Create pages dictionary
    let pages_id = doc.add_object(lopdf::Object::Dictionary(lopdf::dictionary! {
        "Type" => "Pages",
        "Count" => 1,
        "Kids" => vec![],
    }));

    // Create Contents stream
    let contents_id = doc.add_object(lopdf::Object::Stream(lopdf::Stream::new(
        lopdf::dictionary! {},
        vec![],
    )));

    // Create page dictionary (A4 size)
    let page_id = doc.add_object(lopdf::Object::Dictionary(lopdf::dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.28.into(), 841.89.into()],
        "Resources" => lopdf::dictionary! {},
        "Contents" => contents_id,
    }));

    // Update pages dictionary
    let mut pages_dict = doc.get_object(pages_id).unwrap().as_dict().unwrap().clone();
    pages_dict.set("Kids", lopdf::Object::Array(vec![page_id.into()]));
    doc.objects.insert(pages_id, pages_dict.into());

    // Create catalog
    let catalog_id = doc.add_object(lopdf::Object::Dictionary(lopdf::dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    }));

    doc.trailer.set("Root", catalog_id);

    let mut buffer = Vec::new();
    doc.save_to(&mut buffer).unwrap();
    buffer
}

/// Create a test image (colored gradient)
fn create_test_image() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use image::{ImageBuffer, Rgb};

    // Create a 100x100 RGB image with a gradient
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(100, 100, |x, y| {
        Rgb([
            (x * 255 / 100) as u8, // Red gradient
            (y * 255 / 100) as u8, // Green gradient
            128,                   // Blue constant
        ])
    });

    let mut buffer = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buffer),
        image::ImageFormat::Jpeg,
    )?;

    Ok(buffer)
}

/// Create a QR code image
fn create_qr_code(content: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use image::{GrayImage, ImageBuffer, Luma};
    use qrcode::QrCode;

    let code = QrCode::new(content.as_bytes())?;
    let image = code.render::<Luma<u8>>().build();

    // Scale up the QR code for better visibility
    let scale = 4;
    let scaled: GrayImage =
        ImageBuffer::from_fn(image.width() * scale, image.height() * scale, |x, y| {
            *image.get_pixel(x / scale, y / scale)
        });

    let mut buffer = Vec::new();
    scaled.write_to(
        &mut std::io::Cursor::new(&mut buffer),
        image::ImageFormat::Jpeg,
    )?;

    Ok(buffer)
}
