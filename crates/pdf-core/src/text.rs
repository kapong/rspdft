//! Text rendering utilities

use crate::document::Color;
use crate::Align;

/// Context for rendering text
pub struct TextRenderContext {
    /// PDF font resource name (e.g., "F1")
    pub font_name: String,
    /// Font size in points
    pub font_size: f32,
    /// Text width in points (for alignment)
    pub text_width: f64,
    /// Text color (RGB)
    pub color: Color,
}

/// Calculate X offset for text alignment
///
/// # Arguments
/// * `text_width` - Width of text in points
/// * `container_width` - Available width for alignment
/// * `align` - Desired alignment
#[allow(dead_code)]
pub fn calculate_x_offset(text_width: f64, container_width: f64, align: Align) -> f64 {
    match align {
        Align::Left => 0.0,
        Align::Center => (container_width - text_width) / 2.0,
        Align::Right => container_width - text_width,
    }
}

/// Generate PDF operators for text insertion
///
/// Creates the proper PDF text operators (BT, Tf, Td, Tj, ET) to render text
/// at a specific position with alignment support.
///
/// # Arguments
/// * `text_hex` - Hex-encoded text (e.g., "<0041004200>")
/// * `x` - X coordinate in points (PDF coordinates, from left)
/// * `y` - Y coordinate in points (PDF coordinates, from bottom)
/// * `align` - Text alignment
/// * `ctx` - Text rendering context
///
/// # Returns
/// Vector of bytes containing the PDF operators
pub fn generate_text_operators(
    text_hex: &str,
    x: f64,
    y: f64,
    align: Align,
    ctx: &TextRenderContext,
) -> Vec<u8> {
    let mut ops = String::new();

    // Calculate X offset for alignment
    let x_offset = match align {
        Align::Left => 0.0,
        Align::Center => -ctx.text_width / 2.0,
        Align::Right => -ctx.text_width,
    };

    let final_x = x + x_offset;

    // Begin Text
    ops.push_str("BT\n");

    // Set text color (rg operator for non-stroking color)
    ops.push_str(&format!(
        "{} {} {} rg\n",
        ctx.color.r, ctx.color.g, ctx.color.b
    ));

    // Set font and size: /F1 12 Tf
    ops.push_str(&format!("/{} {} Tf\n", ctx.font_name, ctx.font_size));

    // Move to position: x y Td
    ops.push_str(&format!("{final_x} {y} Td\n"));

    // Show text: <hex> Tj
    ops.push_str(&format!("{text_hex} Tj\n"));

    // End Text
    ops.push_str("ET\n");

    ops.into_bytes()
}

/// Split text into lines based on maximum width
///
/// This is a simple implementation that splits on spaces.
/// For Thai text, use the thai-text crate's word_wrap function.
///
/// # Arguments
/// * `text` - Text to split
/// * `max_chars` - Maximum characters per line
pub fn simple_word_wrap(text: &str, max_chars: usize) -> Vec<String> {
    if max_chars == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_chars {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x_offset_left() {
        let offset = calculate_x_offset(100.0, 500.0, Align::Left);
        assert_eq!(offset, 0.0);
    }

    #[test]
    fn test_x_offset_center() {
        let offset = calculate_x_offset(100.0, 500.0, Align::Center);
        assert_eq!(offset, 200.0);
    }

    #[test]
    fn test_x_offset_right() {
        let offset = calculate_x_offset(100.0, 500.0, Align::Right);
        assert_eq!(offset, 400.0);
    }

    #[test]
    fn test_simple_word_wrap() {
        let text = "Hello world this is a test";
        let lines = simple_word_wrap(text, 12);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Hello world");
        assert_eq!(lines[1], "this is a");
        assert_eq!(lines[2], "test");
    }

    #[test]
    fn test_word_wrap_single_line() {
        let text = "Short";
        let lines = simple_word_wrap(text, 100);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Short");
    }

    #[test]
    fn test_word_wrap_zero_max() {
        let text = "Hello world";
        let lines = simple_word_wrap(text, 0);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello world");
    }

    #[test]
    fn test_generate_text_operators_left() {
        let ctx = TextRenderContext {
            font_name: "F1".to_string(),
            font_size: 12.0,
            text_width: 100.0,
            color: Color::black(),
        };

        let ops =
            generate_text_operators("<00480065006C006C006F>", 100.0, 700.0, Align::Left, &ctx);
        let ops_str = String::from_utf8(ops).unwrap();

        assert!(ops_str.contains("BT"));
        assert!(ops_str.contains("/F1 12 Tf"));
        assert!(ops_str.contains("100 700 Td")); // No offset for left align
        assert!(ops_str.contains("<00480065006C006C006F> Tj"));
        assert!(ops_str.contains("ET"));
    }

    #[test]
    fn test_generate_text_operators_center() {
        let ctx = TextRenderContext {
            font_name: "F2".to_string(),
            font_size: 14.0,
            text_width: 100.0,
            color: Color::black(),
        };

        let ops = generate_text_operators("<0054006500730074>", 200.0, 600.0, Align::Center, &ctx);
        let ops_str = String::from_utf8(ops).unwrap();

        assert!(ops_str.contains("BT"));
        assert!(ops_str.contains("/F2 14 Tf"));
        assert!(ops_str.contains("150 600 Td")); // 200 - 50 (half of 100)
        assert!(ops_str.contains("<0054006500730074> Tj"));
        assert!(ops_str.contains("ET"));
    }

    #[test]
    fn test_generate_text_operators_right() {
        let ctx = TextRenderContext {
            font_name: "F3".to_string(),
            font_size: 16.0,
            text_width: 80.0,
            color: Color::black(),
        };

        let ops =
            generate_text_operators("<00520069006700680074>", 300.0, 500.0, Align::Right, &ctx);
        let ops_str = String::from_utf8(ops).unwrap();

        assert!(ops_str.contains("BT"));
        assert!(ops_str.contains("/F3 16 Tf"));
        assert!(ops_str.contains("220 500 Td")); // 300 - 80
        assert!(ops_str.contains("<00520069006700680074> Tj"));
        assert!(ops_str.contains("ET"));
    }

    #[test]
    fn test_generate_text_operators_empty_text() {
        let ctx = TextRenderContext {
            font_name: "F1".to_string(),
            font_size: 12.0,
            text_width: 0.0,
            color: Color::black(),
        };

        let ops = generate_text_operators("<>", 100.0, 700.0, Align::Left, &ctx);
        let ops_str = String::from_utf8(ops).unwrap();

        assert!(ops_str.contains("BT"));
        assert!(ops_str.contains("/F1 12 Tf"));
        assert!(ops_str.contains("100 700 Td"));
        assert!(ops_str.contains("<> Tj"));
        assert!(ops_str.contains("ET"));
    }

    #[test]
    fn test_generate_text_operators_zero_width() {
        let ctx = TextRenderContext {
            font_name: "F1".to_string(),
            font_size: 12.0,
            text_width: 0.0,
            color: Color::black(),
        };

        let ops = generate_text_operators("<0041>", 100.0, 700.0, Align::Center, &ctx);
        let ops_str = String::from_utf8(ops).unwrap();

        // With zero width, center alignment should not change X position
        assert!(ops_str.contains("100 700 Td"));
    }

    #[test]
    fn test_generate_text_operators_large_font() {
        let ctx = TextRenderContext {
            font_name: "F1".to_string(),
            font_size: 72.0,
            text_width: 100.0,
            color: Color::black(),
        };

        let ops = generate_text_operators("<0041>", 100.0, 700.0, Align::Left, &ctx);
        let ops_str = String::from_utf8(ops).unwrap();

        assert!(ops_str.contains("/F1 72 Tf"));
    }

    #[test]
    fn test_simple_word_wrap_empty() {
        let text = "";
        let lines = simple_word_wrap(text, 10);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "");
    }

    #[test]
    fn test_simple_word_wrap_whitespace_only() {
        let text = "   ";
        let lines = simple_word_wrap(text, 10);
        // Should return empty since split_whitespace() removes all whitespace
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "");
    }

    #[test]
    fn test_simple_word_wrap_single_word() {
        let text = "Hello";
        let lines = simple_word_wrap(text, 100);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello");
    }

    #[test]
    fn test_simple_word_wrap_long_word() {
        let text = "Supercalifragilisticexpialidocious";
        let lines = simple_word_wrap(text, 10);
        // Word is longer than max_chars, so it should be on its own line
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Supercalifragilisticexpialidocious");
    }

    #[test]
    fn test_simple_word_wrap_multiple_spaces() {
        let text = "Hello    world";
        let lines = simple_word_wrap(text, 20);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello world");
    }

    #[test]
    fn test_simple_word_wrap_exact_fit() {
        let text = "Hello world";
        let lines = simple_word_wrap(text, 11);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello world");
    }

    #[test]
    fn test_simple_word_wrap_just_over() {
        let text = "Hello world";
        let lines = simple_word_wrap(text, 10);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Hello");
        assert_eq!(lines[1], "world");
    }

    #[test]
    fn test_simple_word_wrap_thai() {
        // Thai text doesn't use spaces between words
        // This simple implementation will treat the whole string as one "word"
        let text = "สวัสดีชาวโลก";
        let lines = simple_word_wrap(text, 10);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "สวัสดีชาวโลก");
    }

    #[test]
    fn test_text_render_context() {
        let ctx = TextRenderContext {
            font_name: "F1".to_string(),
            font_size: 12.0,
            text_width: 100.0,
            color: Color::red(),
        };

        assert_eq!(ctx.font_name, "F1");
        assert_eq!(ctx.font_size, 12.0);
        assert_eq!(ctx.text_width, 100.0);
        assert_eq!(ctx.color, Color::red());
    }

    #[test]
    fn test_generate_text_operators_with_color() {
        let ctx = TextRenderContext {
            font_name: "F1".to_string(),
            font_size: 12.0,
            text_width: 100.0,
            color: Color::red(),
        };

        let ops = generate_text_operators("<0041>", 100.0, 700.0, Align::Left, &ctx);
        let ops_str = String::from_utf8(ops).unwrap();

        // Should contain red color (1 0 0 rg)
        assert!(ops_str.contains("1 0 0 rg"));
    }
}
