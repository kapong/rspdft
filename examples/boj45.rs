//! BOJ45 Form Generator
//!
//! Generates BOJ45 tax form using template and input data.
//! Demonstrates render_to_document() for adding custom labels.
//!
//! Run with: cargo run --example boj45

use std::path::Path;
use template::{PdfAlign, PdfColor, PdfFontWeight, TemplateRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    std::fs::create_dir_all("output")?;

    // Load template JSON
    let template_json = std::fs::read_to_string("assets/boj45_template.json")?;

    // Parse template to get the PDF source path
    let temp_template: serde_json::Value = serde_json::from_str(&template_json)?;
    let pdf_source = temp_template["template"]["source"]
        .as_str()
        .ok_or("Missing template.source")?;
    let pdf_bytes = std::fs::read(pdf_source)?;

    // Create renderer - fonts auto-loaded from template paths
    let renderer = TemplateRenderer::new(&template_json, pdf_bytes, Some(Path::new(".")))?;

    // Load input data
    let input_json = std::fs::read_to_string("input/boj45_input.json")?;
    let data: serde_json::Value = serde_json::from_str(&input_json)?;

    // Render to PdfDocument for post-render modifications
    // (This example shows how to add labels not in the template)
    let mut doc = renderer.render_to_document(&data)?;

    // Add "(COPY)" label on the right side using method chaining
    doc.set_font("sarabun", 10.0)?
        .set_font_weight(PdfFontWeight::Bold)?
        .set_text_color(PdfColor::red())
        .insert_text("(สำเนา / COPY)", 1, 820.0, 15.0, PdfAlign::Right)?;

    // Convert to bytes and save
    let pdf_bytes = doc.to_bytes()?;
    let output_path = "output/boj45.pdf";
    std::fs::write(output_path, pdf_bytes)?;

    println!("Generated: {output_path}");

    Ok(())
}
