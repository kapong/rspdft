//! Integration tests for pdf-core
//!
//! These tests verify end-to-end functionality with real PDF operations.

use lopdf::dictionary;
use pdf_core::{Align, PdfDocument, PdfError};

/// Create a minimal valid PDF for testing
///
/// This creates a simple one-page PDF with A4 dimensions.
fn create_test_pdf() -> Vec<u8> {
    // Create a new document
    let mut doc = lopdf::Document::new();

    // Create pages dictionary
    let pages_id = doc.add_object(lopdf::Object::Dictionary(lopdf::dictionary! {
        "Type" => "Pages",
        "Count" => 1,
        "Kids" => vec![], // Will be updated below
    }));

    // Create Contents stream as a separate object
    let contents_id = doc.add_object(lopdf::Object::Stream(lopdf::Stream::new(
        lopdf::dictionary! {},
        vec![],
    )));

    // Create page dictionary
    let page_id = doc.add_object(lopdf::Object::Dictionary(lopdf::dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "MediaBox" => vec![0.into(), 0.into(), 595.28.into(), 841.89.into()],
        "Resources" => lopdf::dictionary! {},
        "Contents" => contents_id,
    }));

    // Update pages dictionary to include the page
    let mut pages_dict = doc.get_object(pages_id).unwrap().as_dict().unwrap().clone();
    pages_dict.set("Kids", lopdf::Object::Array(vec![page_id.into()]));
    doc.objects.insert(pages_id, pages_dict.into());

    // Create catalog dictionary
    let catalog_id = doc.add_object(lopdf::Object::Dictionary(lopdf::dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    }));

    // Set the trailer
    doc.trailer.set("Root", catalog_id);

    // Save to bytes
    let mut buffer = Vec::new();
    doc.save_to(&mut buffer).unwrap();
    buffer
}

/// Create a minimal valid PDF with multiple pages for testing
fn create_test_pdf_with_pages(page_count: usize) -> Vec<u8> {
    let mut doc = lopdf::Document::new();

    // Create pages dictionary
    let pages_id = doc.add_object(lopdf::Object::Dictionary(lopdf::dictionary! {
        "Type" => "Pages",
        "Count" => page_count as i32,
        "Kids" => vec![], // Will be updated below
    }));

    // Create page dictionaries
    let mut page_ids = Vec::new();
    for _ in 0..page_count {
        // Create Contents stream as a separate object
        let contents_id = doc.add_object(lopdf::Object::Stream(lopdf::Stream::new(
            lopdf::dictionary! {},
            vec![],
        )));

        let page_id = doc.add_object(lopdf::Object::Dictionary(lopdf::dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.28.into(), 841.89.into()],
            "Resources" => lopdf::dictionary! {},
            "Contents" => contents_id,
        }));
        page_ids.push(page_id);
    }

    // Update pages dictionary to include all pages
    let mut pages_dict = doc.get_object(pages_id).unwrap().as_dict().unwrap().clone();
    pages_dict.set(
        "Kids",
        lopdf::Object::Array(page_ids.into_iter().map(|id| id.into()).collect()),
    );
    doc.objects.insert(pages_id, pages_dict.into());

    // Create catalog dictionary
    let catalog_id = doc.add_object(lopdf::Object::Dictionary(lopdf::dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    }));

    // Set the trailer
    doc.trailer.set("Root", catalog_id);

    // Save to bytes
    let mut buffer = Vec::new();
    doc.save_to(&mut buffer).unwrap();
    buffer
}

/// Create a minimal TTF font for testing
///
/// This returns a very small subset of a font with basic glyphs.
/// For production use, you would use actual font files.
fn get_test_font_data() -> Vec<u8> {
    // Use an actual font file from the project
    std::fs::read("../../fonts/THSarabunNew.ttf").expect("Failed to read test font file")
}

/// Create a minimal JPEG image for testing
fn create_test_jpeg() -> Vec<u8> {
    // Minimal JPEG with SOI, SOF0, and EOI markers
    vec![
        0xFF, 0xD8, // SOI marker
        0xFF, 0xC0, // SOF0 marker (baseline DCT)
        0x00, 0x11, // Length (17 bytes)
        0x08, // Precision (8 bits)
        0x00, 0x10, // Height (16 pixels)
        0x00, 0x10, // Width (16 pixels)
        0x03, // Number of components (RGB)
        0x01, 0x22, 0x00, // Component 1 (Y, subsampling 2x2)
        0x02, 0x11, 0x01, // Component 2 (Cb, subsampling 2x1)
        0x03, 0x11, 0x01, // Component 3 (Cr, subsampling 2x1)
        0xFF, 0xD9, // EOI marker
    ]
}

/// Create a minimal PNG image for testing
fn create_test_png() -> Vec<u8> {
    // Create a simple 16x16 grayscale PNG using the image crate
    use image::{ImageBuffer, Luma};

    let img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(16, 16);
    let mut buffer = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buffer),
        image::ImageFormat::Png,
    )
    .expect("Failed to create PNG");
    buffer
}

#[test]
fn test_open_save_roundtrip() {
    // Create a test PDF
    let pdf_data = create_test_pdf();

    // Open the PDF
    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
    assert_eq!(doc.page_count(), 1);

    // Save to bytes
    let saved_data = doc.to_bytes().expect("Failed to save PDF");

    // Open again and verify
    let doc2 = PdfDocument::open_from_bytes(&saved_data).expect("Failed to re-open PDF");
    assert_eq!(doc2.page_count(), 1);
}

#[test]
fn test_insert_text_basic() {
    // Create a test PDF
    let pdf_data = create_test_pdf();
    let font_data = get_test_font_data();

    // Open and add font
    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
    doc.add_font("test", &font_data)
        .expect("Failed to add font");
    doc.set_font("test", 12.0).expect("Failed to set font");

    // Insert text
    doc.insert_text("Hello", 1, 100.0, 700.0, Align::Left)
        .expect("Failed to insert text");

    // Save and verify
    let saved_data = doc.to_bytes().expect("Failed to save PDF");
    assert!(!saved_data.is_empty());
}

#[test]
fn test_insert_text_alignment() {
    let pdf_data = create_test_pdf();
    let font_data = get_test_font_data();

    // Test left alignment
    {
        let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
        doc.add_font("test", &font_data)
            .expect("Failed to add font");
        doc.set_font("test", 12.0).expect("Failed to set font");
        doc.insert_text("Left", 1, 100.0, 700.0, Align::Left)
            .expect("Failed to insert text");
        let _ = doc.to_bytes().expect("Failed to save PDF");
    }

    // Test center alignment
    {
        let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
        doc.add_font("test", &font_data)
            .expect("Failed to add font");
        doc.set_font("test", 12.0).expect("Failed to set font");
        doc.insert_text("Center", 1, 100.0, 700.0, Align::Center)
            .expect("Failed to insert text");
        let _ = doc.to_bytes().expect("Failed to save PDF");
    }

    // Test right alignment
    {
        let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
        doc.add_font("test", &font_data)
            .expect("Failed to add font");
        doc.set_font("test", 12.0).expect("Failed to set font");
        doc.insert_text("Right", 1, 100.0, 700.0, Align::Right)
            .expect("Failed to insert text");
        let _ = doc.to_bytes().expect("Failed to save PDF");
    }
}

#[test]
fn test_insert_image_jpeg() {
    let pdf_data = create_test_pdf();
    let jpeg_data = create_test_jpeg();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
    doc.insert_image(&jpeg_data, 1, 100.0, 700.0, 50.0, 50.0)
        .expect("Failed to insert JPEG image");

    let saved_data = doc.to_bytes().expect("Failed to save PDF");
    assert!(!saved_data.is_empty());
}

#[test]
fn test_insert_image_png() {
    let pdf_data = create_test_pdf();
    let png_data = create_test_png();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
    doc.insert_image(&png_data, 1, 100.0, 700.0, 50.0, 50.0)
        .expect("Failed to insert PNG image");

    let saved_data = doc.to_bytes().expect("Failed to save PDF");
    assert!(!saved_data.is_empty());
}

#[test]
fn test_page_duplication() {
    let pdf_data = create_test_pdf();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
    assert_eq!(doc.page_count(), 1);

    let new_page = doc.duplicate_page(1).expect("Failed to duplicate page");
    assert_eq!(new_page, 2);
    assert_eq!(doc.page_count(), 2);

    let saved_data = doc.to_bytes().expect("Failed to save PDF");
    assert!(!saved_data.is_empty());
}

#[test]
fn test_multiple_fonts() {
    let pdf_data = create_test_pdf();
    let font_data = get_test_font_data();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");

    // Add multiple fonts
    doc.add_font("font1", &font_data)
        .expect("Failed to add font1");
    doc.add_font("font2", &font_data)
        .expect("Failed to add font2");

    // Use different fonts
    doc.set_font("font1", 12.0).expect("Failed to set font1");
    doc.insert_text("Font 1", 1, 100.0, 700.0, Align::Left)
        .expect("Failed to insert text with font1");

    doc.set_font("font2", 14.0).expect("Failed to set font2");
    doc.insert_text("Font 2", 1, 100.0, 680.0, Align::Left)
        .expect("Failed to insert text with font2");

    let saved_data = doc.to_bytes().expect("Failed to save PDF");
    assert!(!saved_data.is_empty());
}

#[test]
fn test_text_on_multiple_pages() {
    // Create a PDF with multiple pages
    let buffer = create_test_pdf_with_pages(2);

    // Now test adding text to multiple pages
    let font_data = get_test_font_data();
    let mut pdf_doc = PdfDocument::open_from_bytes(&buffer).expect("Failed to open PDF");
    assert_eq!(pdf_doc.page_count(), 2);

    pdf_doc
        .add_font("test", &font_data)
        .expect("Failed to add font");
    pdf_doc.set_font("test", 12.0).expect("Failed to set font");

    // Add text to page 1
    pdf_doc
        .insert_text("Page 1", 1, 100.0, 700.0, Align::Left)
        .expect("Failed to insert text on page 1");

    // Add text to page 2
    pdf_doc
        .insert_text("Page 2", 2, 100.0, 700.0, Align::Left)
        .expect("Failed to insert text on page 2");

    let saved_data = pdf_doc.to_bytes().expect("Failed to save PDF");
    assert!(!saved_data.is_empty());
}

#[test]
fn test_empty_text() {
    let pdf_data = create_test_pdf();
    let font_data = get_test_font_data();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
    doc.add_font("test", &font_data)
        .expect("Failed to add font");
    doc.set_font("test", 12.0).expect("Failed to set font");

    // Insert empty text - should work
    doc.insert_text("", 1, 100.0, 700.0, Align::Left)
        .expect("Failed to insert empty text");

    let saved_data = doc.to_bytes().expect("Failed to save PDF");
    assert!(!saved_data.is_empty());
}

#[test]
fn test_very_long_text() {
    let pdf_data = create_test_pdf();
    let font_data = get_test_font_data();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
    doc.add_font("test", &font_data)
        .expect("Failed to add font");
    doc.set_font("test", 12.0).expect("Failed to set font");

    // Create a very long text string
    let long_text = "Hello ".repeat(1000);

    doc.insert_text(&long_text, 1, 100.0, 700.0, Align::Left)
        .expect("Failed to insert long text");

    let saved_data = doc.to_bytes().expect("Failed to save PDF");
    assert!(!saved_data.is_empty());
}

#[test]
fn test_thai_characters() {
    let pdf_data = create_test_pdf();
    let font_data = get_test_font_data();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
    doc.add_font("test", &font_data)
        .expect("Failed to add font");
    doc.set_font("test", 12.0).expect("Failed to set font");

    // Thai text
    let thai_text = "สวัสดีชาวโลก";

    doc.insert_text(thai_text, 1, 100.0, 700.0, Align::Left)
        .expect("Failed to insert Thai text");

    let saved_data = doc.to_bytes().expect("Failed to save PDF");
    assert!(!saved_data.is_empty());
}

#[test]
fn test_invalid_page_number() {
    let pdf_data = create_test_pdf();
    let font_data = get_test_font_data();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
    doc.add_font("test", &font_data)
        .expect("Failed to add font");
    doc.set_font("test", 12.0).expect("Failed to set font");

    // Try to insert text on invalid page
    let result = doc.insert_text("Test", 999, 100.0, 700.0, Align::Left);
    assert!(result.is_err());

    match result {
        Err(PdfError::InvalidPage(page, total)) => {
            assert_eq!(page, 999);
            assert_eq!(total, 1);
        }
        _ => panic!("Expected InvalidPage error"),
    }
}

#[test]
fn test_font_not_found() {
    let pdf_data = create_test_pdf();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");

    // Try to set font that wasn't added
    let result = doc.set_font("nonexistent", 12.0);
    assert!(result.is_err());

    match result {
        Err(PdfError::FontNotFound(name)) => {
            assert_eq!(name, "nonexistent");
        }
        _ => panic!("Expected FontNotFound error"),
    }
}

#[test]
fn test_no_font_set() {
    let pdf_data = create_test_pdf();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");

    // Try to insert text without setting font
    let result = doc.insert_text("Test", 1, 100.0, 700.0, Align::Left);
    assert!(result.is_err());

    match result {
        Err(PdfError::FontNotFound(_)) => {}
        _ => panic!("Expected FontNotFound error"),
    }
}

#[test]
fn test_duplicate_page_invalid() {
    let pdf_data = create_test_pdf();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");

    // Try to duplicate invalid page
    let result = doc.duplicate_page(999);
    assert!(result.is_err());

    match result {
        Err(PdfError::InvalidPage(page, total)) => {
            assert_eq!(page, 999);
            assert_eq!(total, 1);
        }
        _ => panic!("Expected InvalidPage error"),
    }
}

#[test]
fn test_image_deduplication() {
    let pdf_data = create_test_pdf();
    let jpeg_data = create_test_jpeg();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");

    // Insert the same image twice
    doc.insert_image(&jpeg_data, 1, 100.0, 700.0, 50.0, 50.0)
        .expect("Failed to insert image 1");
    doc.insert_image(&jpeg_data, 1, 200.0, 700.0, 50.0, 50.0)
        .expect("Failed to insert image 2");

    let saved_data = doc.to_bytes().expect("Failed to save PDF");
    assert!(!saved_data.is_empty());
}

#[test]
fn test_get_page_ids() {
    let pdf_data = create_test_pdf();

    let doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");
    let page_ids = doc.get_page_ids();

    assert_eq!(page_ids.len(), 1);
}

#[test]
fn test_inner_document_access() {
    let pdf_data = create_test_pdf();

    let mut doc = PdfDocument::open_from_bytes(&pdf_data).expect("Failed to open PDF");

    // Test inner() access
    let inner = doc.inner();
    assert_eq!(inner.get_pages().len(), 1);

    // Test inner_mut() access
    let inner_mut = doc.inner_mut();
    assert_eq!(inner_mut.get_pages().len(), 1);
}
