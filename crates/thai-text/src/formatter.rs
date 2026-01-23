//! Thai number, currency, and date formatting

/// Thai number names (0-9)
const NUMBER_NAMES: [&str; 10] = [
    "ศูนย์",
    "หนึ่ง",
    "สอง",
    "สาม",
    "สี่",
    "ห้า",
    "หก",
    "เจ็ด",
    "แปด",
    "เก้า",
];

/// Thai unit names for each position
const UNIT_NAMES: [&str; 6] = ["", "สิบ", "ร้อย", "พัน", "หมื่น", "แสน"];

/// Thai month names (short)
const THAI_MONTHS_SHORT: [&str; 12] = [
    "ม.ค.",
    "ก.พ.",
    "มี.ค.",
    "เม.ย.",
    "พ.ค.",
    "มิ.ย.",
    "ก.ค.",
    "ส.ค.",
    "ก.ย.",
    "ต.ค.",
    "พ.ย.",
    "ธ.ค.",
];

/// Thai month names (long)
const THAI_MONTHS_LONG: [&str; 12] = [
    "มกราคม",
    "กุมภาพันธ์",
    "มีนาคม",
    "เมษายน",
    "พฤษภาคม",
    "มิถุนายน",
    "กรกฎาคม",
    "สิงหาคม",
    "กันยายน",
    "ตุลาคม",
    "พฤศจิกายน",
    "ธันวาคม",
];

/// Thai text formatting utilities
pub struct ThaiFormatter;

impl ThaiFormatter {
    /// Format a number as Thai text
    pub fn format_number(n: i64) -> String {
        format_thai_number(n)
    }

    /// Format an amount as Thai Baht text
    pub fn format_baht(amount: f64) -> String {
        format_thai_baht(amount)
    }

    /// Format a date in short Thai format
    pub fn format_date_short(year: i32, month: u32, day: u32) -> String {
        format_thai_date_short(year, month, day)
    }

    /// Format a date in long Thai format
    pub fn format_date_long(year: i32, month: u32, day: u32) -> String {
        format_thai_date_long(year, month, day)
    }

    /// Format a year in Thai Buddhist calendar
    pub fn format_year(year: i32) -> String {
        format_thai_year(year)
    }
}

/// Format an integer as Thai text
///
/// # Examples
/// ```
/// use thai_text::format_thai_number;
/// assert_eq!(format_thai_number(0), "ศูนย์");
/// assert_eq!(format_thai_number(21), "ยี่สิบเอ็ด");
/// assert_eq!(format_thai_number(100), "หนึ่งร้อย");
/// ```
pub fn format_thai_number(n: i64) -> String {
    if n == 0 {
        return NUMBER_NAMES[0].to_string();
    }

    let mut n = n.abs();
    let mut result = String::new();
    let mut position = 0;

    while n > 0 {
        let digit = (n % 10) as usize;
        let unit_index = position % 6;
        let remaining = n / 10;

        if digit > 0 {
            let digit_str = if unit_index == 1 && digit == 2 {
                // สิบ position with 2 = ยี่สิบ
                "ยี่".to_string()
            } else if unit_index == 1 && digit == 1 {
                // สิบ position with 1 = สิบ (not หนึ่งสิบ)
                String::new()
            } else if unit_index == 0 && digit == 1 && remaining > 0 {
                // หน่วย position with 1 when there are higher non-zero digits = เอ็ด
                "เอ็ด".to_string()
            } else {
                NUMBER_NAMES[digit].to_string()
            };

            result = format!("{}{}{}", digit_str, UNIT_NAMES[unit_index], result);
        }

        // Add ล้าน (million) marker when crossing million boundary
        if unit_index == 5 && n >= 10 {
            result = format!("ล้าน{result}");
        }

        n /= 10;
        position += 1;
    }

    result
}

/// Format an amount as Thai Baht text
///
/// # Examples
/// ```
/// use thai_text::format_thai_baht;
/// assert_eq!(format_thai_baht(0.0), "-");
/// assert_eq!(format_thai_baht(100.0), "หนึ่งร้อยบาทถ้วน");
/// assert_eq!(format_thai_baht(100.50), "หนึ่งร้อยบาทห้าสิบสตางค์");
/// ```
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

/// Format a date in short Thai format (e.g., "25 ม.ค. 68")
///
/// # Arguments
/// * `year` - Gregorian year (will be converted to Buddhist year)
/// * `month` - Month (1-12)
/// * `day` - Day of month
pub fn format_thai_date_short(year: i32, month: u32, day: u32) -> String {
    let thai_year = year + 543;
    let month_idx = (month.saturating_sub(1)) as usize;
    let month_name = THAI_MONTHS_SHORT.get(month_idx).unwrap_or(&"");
    format!("{} {} {}", day, month_name, thai_year % 100)
}

/// Format a date in long Thai format (e.g., "25 มกราคม 2568")
///
/// # Arguments
/// * `year` - Gregorian year (will be converted to Buddhist year)
/// * `month` - Month (1-12)
/// * `day` - Day of month
pub fn format_thai_date_long(year: i32, month: u32, day: u32) -> String {
    let thai_year = year + 543;
    let month_idx = (month.saturating_sub(1)) as usize;
    let month_name = THAI_MONTHS_LONG.get(month_idx).unwrap_or(&"");
    format!("{day} {month_name} {thai_year}")
}

/// Format a year in Thai Buddhist calendar (e.g., "ปี 2568")
///
/// # Arguments
/// * `year` - Gregorian year
pub fn format_thai_year(year: i32) -> String {
    format!("ปี {}", year + 543)
}

/// Render a float with formatting pattern
///
/// Supports patterns like "#,###.##" for thousand separators and decimal places.
///
/// # Arguments
/// * `format` - Format pattern
/// * `n` - Number to format
pub fn render_float(format: &str, n: f64) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }
    if n.is_infinite() {
        return if n > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }

    // Parse format to determine precision and separators
    let (precision, thousand_sep, decimal_sep) = parse_format(format);

    let abs_n = n.abs();
    let multiplier = 10_f64.powi(precision as i32);
    let rounded = (abs_n * multiplier).round() / multiplier;

    let int_part = rounded.floor() as i64;
    let frac_part = ((rounded - rounded.floor()) * multiplier).round() as i64;

    // Format integer part with thousand separators
    let int_str = format_with_thousands(int_part, thousand_sep);

    // Format fractional part
    let frac_str = if precision > 0 {
        format!("{:0>width$}", frac_part, width = precision as usize)
    } else {
        String::new()
    };

    let sign = if n < -0.000000001 { "-" } else { "" };

    if precision > 0 {
        format!("{sign}{int_str}{decimal_sep}{frac_str}")
    } else {
        format!("{sign}{int_str}")
    }
}

/// Parse format pattern to extract precision and separators
fn parse_format(format: &str) -> (u8, &str, &str) {
    if format.is_empty() {
        return (2, ",", ".");
    }

    // Find decimal point position
    let decimal_pos = format.rfind(['.', ',']);

    let precision = match decimal_pos {
        Some(pos) => {
            let after_decimal: String = format[pos + 1..]
                .chars()
                .filter(|c| *c == '#' || *c == '0')
                .collect();
            after_decimal.len() as u8
        }
        None => 0,
    };

    // Determine separators (simplified - assumes standard format)
    let thousand_sep = if format.contains(',') && format.rfind(',') != decimal_pos {
        ","
    } else {
        ""
    };
    let decimal_sep = if precision > 0 { "." } else { "" };

    (precision, thousand_sep, decimal_sep)
}

/// Format integer with thousand separators
fn format_with_thousands(n: i64, sep: &str) -> String {
    if sep.is_empty() {
        return n.to_string();
    }

    let s = n.to_string();
    let mut result = String::new();

    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, sep.chars().next().unwrap_or(','));
        }
        result.insert(0, c);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_thai_number_basic() {
        assert_eq!(format_thai_number(0), "ศูนย์");
        assert_eq!(format_thai_number(1), "หนึ่ง");
        assert_eq!(format_thai_number(2), "สอง");
        assert_eq!(format_thai_number(9), "เก้า");
    }

    #[test]
    fn test_format_thai_number_tens() {
        assert_eq!(format_thai_number(10), "สิบ");
        assert_eq!(format_thai_number(11), "สิบเอ็ด");
        assert_eq!(format_thai_number(20), "ยี่สิบ");
        assert_eq!(format_thai_number(21), "ยี่สิบเอ็ด");
        assert_eq!(format_thai_number(30), "สามสิบ");
    }

    #[test]
    fn test_format_thai_number_hundreds() {
        assert_eq!(format_thai_number(100), "หนึ่งร้อย");
        assert_eq!(format_thai_number(101), "หนึ่งร้อยเอ็ด");
        assert_eq!(format_thai_number(111), "หนึ่งร้อยสิบเอ็ด");
        assert_eq!(format_thai_number(999), "เก้าร้อยเก้าสิบเก้า");
    }

    #[test]
    fn test_format_thai_number_large() {
        assert_eq!(format_thai_number(1000), "หนึ่งพัน");
        assert_eq!(format_thai_number(10000), "หนึ่งหมื่น");
        assert_eq!(format_thai_number(100000), "หนึ่งแสน");
        assert_eq!(format_thai_number(1000000), "หนึ่งล้าน");
    }

    #[test]
    fn test_format_thai_baht() {
        assert_eq!(format_thai_baht(0.0), "-");
        assert_eq!(format_thai_baht(1.0), "หนึ่งบาทถ้วน");
        assert_eq!(format_thai_baht(0.25), "ยี่สิบห้าสตางค์");
        assert_eq!(format_thai_baht(0.50), "ห้าสิบสตางค์");
        assert_eq!(format_thai_baht(100.50), "หนึ่งร้อยบาทห้าสิบสตางค์");
    }

    #[test]
    fn test_format_thai_date_short() {
        assert_eq!(format_thai_date_short(2025, 1, 22), "22 ม.ค. 68");
        assert_eq!(format_thai_date_short(2025, 12, 31), "31 ธ.ค. 68");
    }

    #[test]
    fn test_format_thai_date_long() {
        assert_eq!(format_thai_date_long(2025, 1, 22), "22 มกราคม 2568");
    }

    #[test]
    fn test_format_thai_year() {
        assert_eq!(format_thai_year(2025), "ปี 2568");
    }

    #[test]
    fn test_render_float() {
        assert_eq!(render_float("#,###.##", 1234.56), "1,234.56");
        assert_eq!(render_float("#,###.##", 1000000.0), "1,000,000.00");
        assert_eq!(render_float("#,###.##", -100.5), "-100.50");
    }

    #[test]
    fn test_render_float_special() {
        assert_eq!(render_float("", f64::NAN), "NaN");
        assert_eq!(render_float("", f64::INFINITY), "Infinity");
        assert_eq!(render_float("", f64::NEG_INFINITY), "-Infinity");
    }

    #[test]
    fn test_format_with_thousands() {
        assert_eq!(format_with_thousands(1000, ","), "1,000");
        assert_eq!(format_with_thousands(1000000, ","), "1,000,000");
        assert_eq!(format_with_thousands(100, ","), "100");
    }
}
