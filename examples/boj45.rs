//! BOJ45 Form Generator
//!
//! Generates BOJ45 tax form using template and input data.
//!
//! Run with: cargo run --example boj45

use std::path::Path;
use template::{Align, Color, FontStyle, TemplateRenderer};

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
    let mut renderer = TemplateRenderer::new(&template_json, pdf_bytes, Some(Path::new(".")))?;

    // === MODIFY TEMPLATE ===
    // Add "(COPY)" label on the right side only using fluent API
    renderer
        .template_mut()
        .set_font("sarabun", 10)
        .set_font_style(FontStyle::Bold)
        .set_text_color(Color::red())
        .insert_text("(สำเนา / COPY)", 1, 820.0, 15.0, Align::Right);

    // Load input data
    let input_json = std::fs::read_to_string("input/boj45_input.json")?;
    let data: serde_json::Value = serde_json::from_str(&input_json)?;

    // Render to bytes
    let pdf_bytes = renderer.render(&data)?;

    // Save output
    let output_path = "output/boj45.pdf";
    std::fs::write(output_path, pdf_bytes)?;

    println!("Generated: {output_path}");

    Ok(())
}
