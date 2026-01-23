//! Debug font embedding

use pdf_core::PdfDocument;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = fs::read("assets/boj45.pdf")?;
    let mut doc = PdfDocument::open_from_bytes(&original)?;

    // Add fonts like the template renderer does
    let fonts = [
        ("sarabun", "fonts/THSarabunNew.ttf"),
        ("sarabun-bold", "fonts/THSarabunNew Bold.ttf"),
    ];

    for (name, path) in fonts {
        let data = fs::read(path)?;
        println!("Adding font: {} ({} bytes)", name, data.len());
        doc.add_font(name, &data)?;
    }

    // Set font and insert text
    doc.set_font("sarabun", 12.0)?;
    doc.insert_text("Test", 1, 100.0, 100.0, pdf_core::Align::Left)?;

    // Save
    let output = doc.to_bytes()?;
    fs::write("output/debug_fonts.pdf", &output)?;
    println!("\nOutput: {} bytes", output.len());

    Ok(())
}
