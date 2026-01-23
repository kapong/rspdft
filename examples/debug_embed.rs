//! Debug font embedding

use pdf_core::PdfDocument;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = fs::read("assets/boj45.pdf")?;
    let mut doc = PdfDocument::open_from_bytes(&original)?;

    // Add fonts like the template does
    let fonts = [
        ("sarabun", "fonts/THSarabunNew.ttf"),
        ("sarabun-bold", "fonts/THSarabunNew Bold.ttf"),
        ("sarabun-italic", "fonts/THSarabunNew Italic.ttf"),
        ("sarabun-bold-italic", "fonts/THSarabunNew BoldItalic.ttf"),
        ("symbols", "fonts/NotoSansSymbols2-Regular.ttf"),
    ];

    let mut added_fonts = Vec::new();
    for (name, path) in &fonts {
        let data = fs::read(path)?;
        doc.add_font(name, &data)?;
        println!("Added font: {}", name);
        added_fonts.push(name.to_string());
    }

    // Set font and insert minimal text
    doc.set_font("sarabun", 12.0)?;
    doc.insert_text("X", 1, 100.0, 100.0, pdf_core::Align::Left)?;

    // List what fonts we added
    println!("\n=== Fonts added to document ===");
    for font in &added_fonts {
        println!("  {}", font);
    }
    println!("Total: {} fonts", added_fonts.len());

    // Save
    let output = doc.to_bytes()?;
    fs::write("output/debug_embed.pdf", &output)?;
    println!("\nOutput: {} bytes", output.len());

    // Check embedded fonts in output
    let fontfile_count = output.windows(9).filter(|w| w == b"FontFile2").count();
    println!("FontFile2 in output: {}", fontfile_count);

    // Also check for font names in the output
    println!("\n=== Checking for font subsets in output ===");
    for name in &added_fonts {
        let name_bytes = name.as_bytes();
        let count = output
            .windows(name_bytes.len())
            .filter(|w| *w == name_bytes)
            .count();
        println!("  '{}' occurrences: {}", name, count);
    }

    // Check for /Type /Font entries
    let type_font = b"/Type /Font";
    let type_font_count = output
        .windows(type_font.len())
        .filter(|w| *w == type_font)
        .count();
    println!("\n/Type /Font entries: {}", type_font_count);

    // Check for /Subtype /TrueType
    let truetype = b"/Subtype /TrueType";
    let truetype_count = output
        .windows(truetype.len())
        .filter(|w| *w == truetype)
        .count();
    println!("/Subtype /TrueType entries: {}", truetype_count);

    // Check for /Subtype /CIDFontType2
    let cidtype2 = b"/Subtype /CIDFontType2";
    let cidtype2_count = output
        .windows(cidtype2.len())
        .filter(|w| *w == cidtype2)
        .count();
    println!("/Subtype /CIDFontType2 entries: {}", cidtype2_count);

    Ok(())
}
