# PDF Text Insertion Implementation Summary (Phase 2.1)

## Overview
Implemented PDF text insertion functionality that allows adding text to PDF pages at specific coordinates with proper font embedding and alignment support.

## Files Modified

### 1. `crates/pdf-core/src/text.rs`
Added text rendering infrastructure:

- **`TextRenderContext` struct**: Holds rendering state including:
  - `font_name`: PDF font resource name (e.g., "F1")
  - `font_size`: Font size in points
  - `text_width`: Text width in points for alignment calculations

- **`generate_text_operators()` function**: Generates proper PDF text operators:
  - `BT` - Begin Text
  - `Tf` - Set font and size
  - `Td` - Move to position (with alignment offset)
  - `Tj` - Show text (hex-encoded UTF-16BE)
  - `ET` - End Text

- **Alignment support**: Calculates X offset based on alignment:
  - Left: No offset
  - Center: `-text_width / 2`
  - Right: `-text_width`

### 2. `crates/pdf-core/src/document.rs`
Implemented text insertion methods:

- **`insert_text()`**: Main public API for inserting text:
  - Validates page number
  - Tracks characters used in font for subsetting
  - Gets or creates font reference for the page
  - Calculates text width for alignment
  - Converts Y coordinate from top-origin to PDF bottom-origin
  - Encodes text as hex string
  - Generates and appends PDF operators

- **`get_page_height()`**: Extracts page height from MediaBox or CropBox:
  - Parses page dictionary
  - Extracts MediaBox array [x1, y1, x2, y2]
  - Returns height as y2 - y1

- **`append_to_content_stream()`**: Appends content to page's content stream:
  - Handles single stream or array of streams
  - Decompresses compressed streams if needed
  - Concatenates existing content with new operators
  - Creates new stream and updates page dictionary

- **`get_text_width()`**: Calculates text width in points:
  - Uses current font's metrics
  - Scales by font size

### 3. `crates/pdf-core/src/lib.rs`
Exported new public API:
- `TextRenderContext`
- `generate_text_operators`

## Key Features

### Coordinate System Conversion
- **Input**: Y from top of page (like web/screen coordinates)
- **PDF**: Y from bottom of page
- **Conversion**: `pdf_y = page_height - input_y`

### Text Encoding
- Uses UTF-16BE hex encoding for PDF Tj operator
- Handles characters beyond BMP with surrogate pairs
- Example: "Hello" → `<00480065006C006C006F>`

### Font Resource Management
- Automatically creates font resource names (F1, F2, etc.)
- Tracks font resources per page
- Adds font to page's Resources dictionary

### Content Stream Handling
- Supports both compressed and uncompressed streams
- Handles single stream or array of streams
- Properly decompresses before appending

## Example Usage

```rust
use pdf_core::{PdfDocument, Align};

let mut doc = PdfDocument::open("template.pdf")?;
doc.add_font("sarabun", include_bytes!("THSarabunNew.ttf"))?;
doc.set_font("sarabun", 14.0)?;

// Insert text 50pt from top, 100pt from left
doc.insert_text("Hello สวัสดี", 1, 100.0, 50.0, Align::Left)?;

// Insert centered text
doc.insert_text("Centered", 1, 300.0, 100.0, Align::Center)?;

// Insert right-aligned text
doc.insert_text("Right", 1, 500.0, 150.0, Align::Right)?;

doc.save("output.pdf")?;
```

## Tests Added

### Text Operator Generation Tests
- `test_generate_text_operators_left`: Verifies left alignment (no offset)
- `test_generate_text_operators_center`: Verifies center alignment (half-width offset)
- `test_generate_text_operators_right`: Verifies right alignment (full-width offset)

All tests verify:
- Proper operator sequence (BT, Tf, Td, Tj, ET)
- Correct font resource name and size
- Correct position with alignment offset
- Proper hex-encoded text

## Technical Details

### PDF Text Operators Generated
```
BT              % Begin Text
/F1 12 Tf       % Set font F1 size 12
100 700 Td      % Move to position (100, 700)
<0041004200> Tj % Show text (hex encoded UTF-16BE)
ET              % End Text
```

### Error Handling
- Invalid page number → `PdfError::InvalidPage`
- Font not set → `PdfError::FontNotFound`
- Missing MediaBox → `PdfError::ParseError`
- Invalid MediaBox format → `PdfError::ParseError`

## Next Steps

This implementation provides the foundation for text insertion. Future enhancements could include:
- Multi-line text support
- Text rotation
- Text color and styling
- Word wrapping for Thai text
- Text clipping
- Character spacing and kerning