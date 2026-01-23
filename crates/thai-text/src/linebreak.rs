//! Line breaking utilities for Thai text

/// Check if a character is a Thai character
#[allow(dead_code)]
pub fn is_thai_char(c: char) -> bool {
    // Thai Unicode range: U+0E00 to U+0E7F
    ('\u{0E00}'..='\u{0E7F}').contains(&c)
}

/// Check if a character is a Thai vowel that comes before a consonant
#[allow(dead_code)]
pub fn is_leading_vowel(c: char) -> bool {
    matches!(c, 'เ' | 'แ' | 'โ' | 'ไ' | 'ใ')
}

/// Check if a character is a Thai tone mark or vowel modifier
#[allow(dead_code)]
pub fn is_above_below_mark(c: char) -> bool {
    matches!(c,
        '\u{0E31}' |         // Mai Han-Akat
        '\u{0E34}'..='\u{0E3A}' | // Upper vowels and marks
        '\u{0E47}'..='\u{0E4E}'   // Tone marks and other marks
    )
}

/// Check if breaking between two characters is allowed
///
/// Returns true if a line break is allowed between `left` and `right`.
#[allow(dead_code)]
pub fn can_break_between(left: char, right: char) -> bool {
    // Don't break if right char is a mark that belongs to left
    if is_above_below_mark(right) {
        return false;
    }

    // Don't break if right char is a leading vowel (it needs the next consonant)
    if is_leading_vowel(right) {
        return false;
    }

    // Don't break after leading vowel
    if is_leading_vowel(left) {
        return false;
    }

    // Allow break between Thai words (this is a simplification)
    // Real implementation would use dictionary-based word boundaries
    true
}

/// Find safe break points in Thai text
///
/// Returns indices where line breaks are allowed.
#[allow(dead_code)]
pub fn find_break_points(text: &str) -> Vec<usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut break_points = Vec::new();

    // Break at index 0 is always allowed (start of text)
    break_points.push(0);

    for i in 1..chars.len() {
        if can_break_between(chars[i - 1], chars[i]) {
            break_points.push(i);
        }
    }

    // End of text is always a valid break point
    break_points.push(chars.len());

    break_points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_thai_char() {
        assert!(is_thai_char('ก'));
        assert!(is_thai_char('ฮ'));
        assert!(is_thai_char('๐'));
        assert!(!is_thai_char('A'));
        assert!(!is_thai_char('1'));
    }

    #[test]
    fn test_is_leading_vowel() {
        assert!(is_leading_vowel('เ'));
        assert!(is_leading_vowel('แ'));
        assert!(is_leading_vowel('โ'));
        assert!(!is_leading_vowel('ก'));
        assert!(!is_leading_vowel('า'));
    }

    #[test]
    fn test_no_break_after_leading_vowel() {
        // Should not break between เ and ก
        assert!(!can_break_between('เ', 'ก'));
    }

    #[test]
    fn test_break_between_consonants() {
        // Can break between consonants (simplified)
        assert!(can_break_between('ก', 'ข'));
    }
}
