//! BOJ45 Form Generator
//!
//! Generates BOJ45 tax form using template and input data.
//!
//! Run with: cargo run --example boj45 -p pdf-core

use pdf_core::{Align, Color, FontWeight, PdfDocument};
use template::{parse_template, TemplateRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    std::fs::create_dir_all("output")?;

    // Load template JSON
    let template_json = std::fs::read_to_string("assets/boj45_template.json")?;
    let template = parse_template(&template_json)?;

    // Load input data
    let input_json = std::fs::read_to_string("input/boj45_input.json")?;
    let data: serde_json::Value = serde_json::from_str(&input_json)?;

    // Open the PDF template
    let mut doc = PdfDocument::open(&template.template.source)?;

    // Load fonts from template using new font family API
    let renderer = TemplateRenderer::new(&template);
    renderer.load_fonts(&mut doc)?;

    // Render template
    renderer.render(&mut doc, &data)?;

    // Add "(COPY)" at top right corner of the right (duplicated) half
    // The duplicate offset is 420.945, so the right half starts there
    // Position: near top-right of the right half
    let duplicate_x = template
        .template
        .duplicate
        .as_ref()
        .map(|d| d.x)
        .unwrap_or(420.945);

    // Use font family with weight and red color
    doc.set_font("sarabun", 14.0)?;
    doc.set_font_weight(FontWeight::Bold)?;
    doc.set_text_color(Color::red());
    doc.insert_text(
        "(สำเนา)",
        1,
        duplicate_x + 380.0, // Right side of the duplicated form
        25.0,                // Near top
        Align::Right,
    )?;
    doc.set_text_color(Color::default()); // Reset color

    // Save output
    let output_path = "output/boj45.pdf";
    doc.save(output_path)?;

    println!("Generated: {}", output_path);

    Ok(())
}
