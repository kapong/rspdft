//! Generic Form Renderer
//!
//! Renders any PDF form from a template JSON and input data.
//! Template-specific settings like "(COPY)" labels are configured in the template's additionalItems.
//!
//! Usage:
//!   cargo run --example render_form -- <template.json> <input.json> [output.pdf]
//!
//! Examples:
//!   cargo run --example render_form -- assets/approve_wh3.json input/approve_wh3_input.json
//!   cargo run --example render_form -- assets/boj45_template.json input/boj45_input.json output/boj45.pdf

use std::path::Path;
use template::TemplateRenderer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!(
            "Usage: {} <template.json> <input.json> [output.pdf]",
            args[0]
        );
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  cargo run --example render_form -- assets/approve_wh3.json input/approve_wh3_input.json");
        eprintln!("  cargo run --example render_form -- assets/boj45_template.json input/boj45_input.json");
        std::process::exit(1);
    }

    let template_path = &args[1];
    let input_path = &args[2];

    // Derive output path from template name if not provided
    let output_path = if args.len() > 3 {
        args[3].clone()
    } else {
        let template_name = Path::new(template_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        // Remove _template suffix if present
        let name = template_name.trim_end_matches("_template");
        format!("output/{}.pdf", name)
    };

    // Create output directory
    if let Some(parent) = Path::new(&output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Load template JSON
    let template_json = std::fs::read_to_string(template_path)
        .map_err(|e| format!("Failed to read template '{}': {}", template_path, e))?;

    // Parse template to get the PDF source path
    let temp_template: serde_json::Value = serde_json::from_str(&template_json)?;
    let pdf_source = temp_template["template"]["source"]
        .as_str()
        .ok_or("Missing template.source in JSON")?;
    let pdf_bytes = std::fs::read(pdf_source)
        .map_err(|e| format!("Failed to read PDF '{}': {}", pdf_source, e))?;

    // Create renderer - fonts auto-loaded from template paths
    let renderer = TemplateRenderer::new(&template_json, pdf_bytes, Some(Path::new(".")))?;

    // Load input data
    let input_json = std::fs::read_to_string(input_path)
        .map_err(|e| format!("Failed to read input '{}': {}", input_path, e))?;
    let data: serde_json::Value = serde_json::from_str(&input_json)?;

    // Render PDF
    let pdf_bytes = renderer.render(&data)?;

    // Save output
    std::fs::write(&output_path, pdf_bytes)?;

    println!("Generated: {}", output_path);

    Ok(())
}
