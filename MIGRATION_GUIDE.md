# Go to Rust Migration Guide

This guide documents the mapping from the original Go implementation to Rust.

## Type Mappings

| Go Type | Rust Type | Notes |
|---------|-----------|-------|
| `string` | `String` | Owned string |
| `&str` | `&str` | Borrowed string slice |
| `[]byte` | `Vec<u8>` | Byte vector |
| `&[]byte` | `&[u8]` | Byte slice |
| `float64` | `f64` | 64-bit float |
| `float32` | `f32` | 32-bit float |
| `int` | `i32` or `usize` | Context dependent |
| `int64` | `i64` | 64-bit integer |
| `bool` | `bool` | Boolean |
| `map[string]interface{}` | `serde_json::Value` | Dynamic JSON |
| `map[string]string` | `HashMap<String, String>` | String map |
| `[]string` | `Vec<String>` | String vector |
| `struct` | `struct` | Use `#[derive(Serialize, Deserialize)]` |
| `interface{}` | `trait` or `enum` | Prefer enum for known variants |
| `error` | `Result<T, E>` | Use `thiserror` for error types |
| `time.Time` | `chrono::NaiveDate` | Or `chrono::DateTime` |

## Go Patterns → Rust

### Error Handling

```go
// Go
func doSomething() (result string, err error) {
    data, err := readFile(path)
    if err != nil {
        return "", err
    }
    return process(data), nil
}
```

```rust
// Rust
fn do_something() -> Result<String, Error> {
    let data = read_file(path)?;  // ? operator for early return
    Ok(process(&data))
}
```

### JSON Parsing

```go
// Go
type Config struct {
    Name string  `json:"name"`
    Size int     `json:"size,omitempty"`
    Tags []string `json:"tags,omitempty"`
}

var config Config
err := json.Unmarshal(data, &config)
```

```rust
// Rust
#[derive(Debug, Deserialize)]
struct Config {
    name: String,
    #[serde(default)]
    size: Option<i32>,
    #[serde(default)]
    tags: Vec<String>,
}

let config: Config = serde_json::from_str(data)?;
```

### Interface/Polymorphism

```go
// Go - interface
type Block interface {
    InsertToPDF(pdf *pdft.PDFt, wordwrap func(string, int) []string)
    SetTagetValue(tag string, value interface{})
    ShiftXY(x, y float64)
    Copy() Block
}

type TextBlock struct { /* fields */ }
func (t *TextBlock) InsertToPDF(pdf *pdft.PDFt, wordwrap func(string, int) []string) {
    // implementation
}
```

```rust
// Rust - prefer enum for closed set of types
#[derive(Debug, Clone)]
pub enum Block {
    Text(TextBlock),
    Table(TableBlock),
    QRCode(QRCodeBlock),
    FieldForm(FieldFormBlock),
}

impl Block {
    pub fn render(&self, doc: &mut PdfDocument, wordcut: &ThaiWordcut) -> Result<()> {
        match self {
            Block::Text(b) => b.render(doc, wordcut),
            Block::Table(b) => b.render(doc, wordcut),
            Block::QRCode(b) => b.render(doc, wordcut),
            Block::FieldForm(b) => b.render(doc, wordcut),
        }
    }
    
    pub fn set_value(&mut self, tag: &str, value: &serde_json::Value) {
        match self {
            Block::Text(b) => b.set_value(tag, value),
            // ... etc
        }
    }
    
    pub fn shift_xy(&mut self, x: f64, y: f64) {
        match self {
            Block::Text(b) => b.shift_xy(x, y),
            // ... etc
        }
    }
}
```

### Method Receivers

```go
// Go - pointer receiver
func (t *TextBlock) SetText(text string) {
    t.Text = text
}

// Go - value receiver
func (t TextBlock) GetText() string {
    return t.Text
}
```

```rust
// Rust - mutable reference
impl TextBlock {
    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }
    
    // Rust - immutable reference
    pub fn get_text(&self) -> &str {
        &self.text
    }
}
```

### Nil/Option Handling

```go
// Go
if tb.PagesNum != nil {
    for _, p := range tb.PagesNum {
        // use p
    }
}
```

```rust
// Rust
if let Some(pages) = &self.pages {
    for p in pages {
        // use p
    }
}
// Or with unwrap_or_default
for p in self.pages.as_ref().unwrap_or(&vec![]) {
    // use p
}
```

## Specific Component Migrations

### pdft → lopdf

```go
// Go (pdft)
var ipdf pdft.PDFt
err := ipdf.Open(filepath)
ipdf.AddFont(name, path)
ipdf.SetFont(name, style, size)
ipdf.Insert(text, pageNum, x, y, w, h, align)
ipdf.InsertImg(bytes, pageNum, x, y, w, h)
err = ipdf.Save(target)
```

```rust
// Rust (lopdf + custom wrapper)
let mut doc = PdfDocument::open(filepath)?;
doc.add_font(name, &font_bytes)?;
doc.set_font(name, size)?;
doc.insert_text(text, page_num, x, y, align)?;
doc.insert_image(&image_bytes, page_num, x, y, w, h)?;
doc.save(target)?;
```

### mapkha → thai-text

```go
// Go (mapkha)
dict, err := mapkha.LoadDict("dict.txt")
if err != nil {
    panic(err)
}
wordcut := mapkha.NewWordcut(dict)
lines := wordcut.WordWrap(text, maxChars)
```

```rust
// Rust (thai-text)
let wordcut = ThaiWordcut::from_file("dict.txt")?;
let lines = wordcut.word_wrap(text, max_chars);
```

### barcode → qrcode

```go
// Go (boombuler/barcode)
img, _ := qr.Encode(code, qr.M, qr.Auto)
img, _ = barcode.Scale(img, 500, 500)
buff := new(bytes.Buffer)
jpeg.Encode(buff, img, nil)
```

```rust
// Rust (qrcode + image)
use qrcode::QrCode;
use image::{Luma, DynamicImage};

let code = QrCode::new(data.as_bytes())?;
let image = code.render::<Luma<u8>>().build();
let resized = image::imageops::resize(
    &image, 500, 500, 
    image::imageops::FilterType::Lanczos3
);
let mut bytes: Vec<u8> = Vec::new();
resized.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Jpeg)?;
```

### copier → Clone

```go
// Go (jinzhu/copier)
var newObj TextBlock
copier.Copy(&newObj, &oldObj)
```

```rust
// Rust (Clone derive)
#[derive(Clone)]
struct TextBlock { /* fields */ }

let new_obj = old_obj.clone();
```

### JSON Unmarshaling with Type Field

```go
// Go - custom UnmarshalJSON
func (b *block) UnmarshalJSON(data []byte) error {
    var meta blockMeta
    json.Unmarshal(data, &meta)
    
    switch meta.Type {
    case "text":
        var obj textBlock
        json.Unmarshal(data, &obj)
        b.object = &obj
    case "table":
        // ...
    }
    return nil
}
```

```rust
// Rust - serde tagged enum
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Block {
    Text(TextBlock),
    Table(TableBlock),
    #[serde(rename = "qrcode")]
    QRCode(QRCodeBlock),
    #[serde(rename = "fieldform")]
    FieldForm(FieldFormBlock),
}

// Automatically handles the "type" field!
let block: Block = serde_json::from_str(json)?;
```

## Number Formatting (Thai)

Port from `utils/numberformat.go`:

```rust
// thai-text/src/formatter.rs

const NUMBER_NAMES: [&str; 10] = [
    "ศูนย์", "หนึ่ง", "สอง", "สาม", "สี่",
    "ห้า", "หก", "เจ็ด", "แปด", "เก้า"
];

const UNIT_NAMES: [&str; 6] = [
    "", "สิบ", "ร้อย", "พัน", "หมื่น", "แสน"
];

pub fn format_thai_number(mut n: i64) -> String {
    if n == 0 {
        return NUMBER_NAMES[0].to_string();
    }
    
    let mut result = String::new();
    let mut position = 0;
    let mut has_previous = false;
    
    while n > 0 {
        let digit = (n % 10) as usize;
        let unit_index = position % 6;
        
        if digit > 0 {
            let digit_str = if unit_index == 1 && digit == 2 {
                "ยี่".to_string()
            } else if unit_index == 1 && digit == 1 {
                String::new()
            } else if unit_index == 0 && digit == 1 && has_previous {
                "เอ็ด".to_string()
            } else {
                NUMBER_NAMES[digit].to_string()
            };
            
            result = format!("{}{}{}", digit_str, UNIT_NAMES[unit_index], result);
            has_previous = true;
        }
        
        if unit_index == 5 && n >= 10 {
            result = format!("ล้าน{}", result);
        }
        
        n /= 10;
        position += 1;
    }
    
    result
}

pub fn format_thai_baht(amount: f64) -> String {
    let satang = ((amount * 100.0).round() as i64) % 100;
    let baht = amount.floor() as i64;
    
    match (baht, satang) {
        (0, 0) => "-".to_string(),
        (b, 0) if b > 0 => format!("{}บาทถ้วน", format_thai_number(b)),
        (0, s) if s > 0 => format!("{}สตางค์", format_thai_number(s)),
        (b, s) => format!("{}บาท{}สตางค์", format_thai_number(b), format_thai_number(s)),
    }
}
```

## Date Formatting (Thai)

Port from `utils/timeformat.go`:

```rust
// thai-text/src/formatter.rs

const THAI_MONTHS_SHORT: [&str; 12] = [
    "ม.ค.", "ก.พ.", "มี.ค.", "เม.ย.", "พ.ค.", "มิ.ย.",
    "ก.ค.", "ส.ค.", "ก.ย.", "ต.ค.", "พ.ย.", "ธ.ค."
];

const THAI_MONTHS_LONG: [&str; 12] = [
    "มกราคม", "กุมภาพันธ์", "มีนาคม", "เมษายน", 
    "พฤษภาคม", "มิถุนายน", "กรกฎาคม", "สิงหาคม",
    "กันยายน", "ตุลาคม", "พฤศจิกายน", "ธันวาคม"
];

pub fn format_thai_date_short(year: i32, month: u32, day: u32) -> String {
    let thai_year = year + 543;
    let month_idx = (month - 1) as usize;
    format!("{} {} {}", day, THAI_MONTHS_SHORT[month_idx], thai_year % 100)
}

pub fn format_thai_date_long(year: i32, month: u32, day: u32) -> String {
    let thai_year = year + 543;
    let month_idx = (month - 1) as usize;
    format!("{} {} {}", day, THAI_MONTHS_LONG[month_idx], thai_year)
}

pub fn format_thai_year(year: i32) -> String {
    format!("ปี {}", year + 543)
}
```

## RenderFloat Function

Port the number formatting function:

```rust
pub fn render_float(format: &str, n: f64) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }
    if n.is_infinite() {
        return if n > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    
    // Parse format string to determine:
    // - precision (digits after decimal)
    // - thousand separator
    // - decimal separator
    // - positive/negative prefix
    
    // Default format: #,###.##
    let precision = 2;
    let thousand_sep = ",";
    let decimal_sep = ".";
    
    let abs_n = n.abs();
    let int_part = abs_n.floor() as i64;
    let frac_part = ((abs_n - abs_n.floor()) * 10_f64.powi(precision as i32)).round() as i64;
    
    // Format integer part with thousand separators
    let int_str = format_with_thousands(int_part, thousand_sep);
    
    // Format fractional part
    let frac_str = format!("{:0>width$}", frac_part, width = precision as usize);
    
    let sign = if n < 0.0 { "-" } else { "" };
    
    if precision > 0 {
        format!("{}{}{}{}", sign, int_str, decimal_sep, frac_str)
    } else {
        format!("{}{}", sign, int_str)
    }
}

fn format_with_thousands(n: i64, sep: &str) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, sep.chars().next().unwrap());
        }
        result.insert(0, c);
    }
    result
}
```

## Testing Equivalence

For each migrated function, write tests that verify identical output:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_thai_number_matches_go() {
        // Test cases from Go implementation
        assert_eq!(format_thai_number(0), "ศูนย์");
        assert_eq!(format_thai_number(1), "หนึ่ง");
        assert_eq!(format_thai_number(11), "สิบเอ็ด");
        assert_eq!(format_thai_number(21), "ยี่สิบเอ็ด");
        assert_eq!(format_thai_number(100), "หนึ่งร้อย");
        assert_eq!(format_thai_number(101), "หนึ่งร้อยเอ็ด");
        assert_eq!(format_thai_number(1000000), "หนึ่งล้าน");
    }
    
    #[test]
    fn test_format_thai_baht_matches_go() {
        assert_eq!(format_thai_baht(0.0), "-");
        assert_eq!(format_thai_baht(1.0), "หนึ่งบาทถ้วน");
        assert_eq!(format_thai_baht(0.50), "ห้าสิบสตางค์");
        assert_eq!(format_thai_baht(100.25), "หนึ่งร้อยบาทยี่สิบห้าสตางค์");
    }
}
```
