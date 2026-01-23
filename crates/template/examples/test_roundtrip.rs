//! Minimal PDF roundtrip test using pdf-core directly
//! Run with: cargo run --example test_roundtrip
//!
//! Tests each step to find where PDF corruption occurs:
//! 1. Open PDF with lopdf
//! 2. Save without modifications
//! 3. Add font (no text)
//! 4. Add text

use pdf_core::{Align, PdfDocument};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PDF Roundtrip Test ===\n");

    // Ensure output directory exists
    fs::create_dir_all("output")?;

    // Step 1: Read base PDF
    let original = fs::read("assets/boj45.pdf")?;
    println!("1. Original PDF: {} bytes", original.len());

    // Step 2: Open with lopdf (via pdf-core)
    let mut doc = PdfDocument::open_from_bytes(&original)?;
    println!("2. Opened PDF: {} pages", doc.page_count());

    // Step 3: Save WITHOUT any modifications
    let output1 = doc.to_bytes()?;
    fs::write("output/test_unchanged.pdf", &output1)?;
    println!(
        "3. Saved unchanged: {} bytes -> output/test_unchanged.pdf",
        output1.len()
    );

    // Step 4: Add a simple font
    let font_bytes = fs::read("fonts/THSarabunNew.ttf")?;
    doc.add_font("sarabun", &font_bytes)?;
    println!("4. Added font");

    // Step 5: Save with font (but no text)
    let output2 = doc.to_bytes()?;
    fs::write("output/test_with_font.pdf", &output2)?;
    println!(
        "5. Saved with font: {} bytes -> output/test_with_font.pdf",
        output2.len()
    );

    // Step 6: Insert text using correct API
    doc.set_font("sarabun", 12.0)?;
    doc.insert_text("Test", 1, 100.0, 100.0, Align::Left)?;
    println!("6. Inserted text");

    // Step 7: Save with text
    let output3 = doc.to_bytes()?;
    fs::write("output/test_with_text.pdf", &output3)?;
    println!(
        "7. Saved with text: {} bytes -> output/test_with_text.pdf",
        output3.len()
    );

    println!("\n=== Done! Check each PDF in Preview ===");
    println!("open output/test_unchanged.pdf");
    println!("open output/test_with_font.pdf");
    println!("open output/test_with_text.pdf");

    Ok(())
}
