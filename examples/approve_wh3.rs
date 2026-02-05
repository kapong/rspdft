//! Approve WH3 Form Generator
//!
//! Generates Approve WH3 tax withholding form using template and input data.
//! This template duplicates page 1 to page 2 for original/copy receipts.
//!
//! Run with: cargo run --example approve_wh3

use std::path::Path;
use template::{PdfAlign, PdfColor, PdfFontWeight, TemplateRenderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory
    std::fs::create_dir_all("output")?;

    // Load template JSON
    let template_json = std::fs::read_to_string("assets/approve_wh3.json")?;

    // Parse template to get the PDF source path
    let temp_template: serde_json::Value = serde_json::from_str(&template_json)?;
    let pdf_source = temp_template["template"]["source"]
        .as_str()
        .ok_or("Missing template.source")?;
    let pdf_bytes = std::fs::read(pdf_source)?;

    // Create renderer - fonts auto-loaded from template paths
    let renderer = TemplateRenderer::new(&template_json, pdf_bytes, Some(Path::new(".")))?;

    // Load input data
    let input_json = std::fs::read_to_string("input/approve_wh3_input.json")?;
    let data: serde_json::Value = serde_json::from_str(&input_json)?;

    // Render to PdfDocument (allows further modification)
    let mut doc = renderer.render_to_document(&data)?;

    // Add "(COPY)" label ONLY on page 2 using method chaining
    doc.set_font("sarabun", 14.0)?
        .set_font_weight(PdfFontWeight::Regular)?
        .set_text_color(PdfColor::red())
        .insert_text("(สำเนา / COPY)", 2, 565.0, 35.0, PdfAlign::Right)?;

    // Convert to bytes and save
    let pdf_bytes = doc.to_bytes()?;
    let output_path = "output/approve_wh3.pdf";
    std::fs::write(output_path, pdf_bytes)?;

    println!("Generated: {output_path}");

    Ok(())
}
