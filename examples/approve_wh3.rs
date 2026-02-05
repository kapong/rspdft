//! Approve WH3 Form Generator
//!
//! Generates Approve WH3 tax withholding form using template and input data.
//! This template duplicates page 1 to page 2 for original/copy receipts.
//! The "(COPY)" label is configured in the template's additionalItems.
//!
//! Run with: cargo run --example approve_wh3

use std::path::Path;
use template::TemplateRenderer;

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

    // Render PDF - additionalItems in template handles the "(COPY)" label
    let pdf_bytes = renderer.render(&data)?;

    // Save output
    let output_path = "output/approve_wh3.pdf";
    std::fs::write(output_path, pdf_bytes)?;

    println!("Generated: {output_path}");

    Ok(())
}
