//! Thai word segmentation using longest matching algorithm

use crate::{Dictionary, Result, ThaiTextError};
use std::path::Path;
use std::str::FromStr;

/// Thai word segmentation utility
#[derive(Debug, Clone)]
pub struct ThaiWordcut {
    /// Word dictionary
    dict: Dictionary,
}

impl ThaiWordcut {
    /// Create a new wordcut instance with the given dictionary
    pub fn new(dict: Dictionary) -> Self {
        Self { dict }
    }

    /// Create wordcut with embedded Thai dictionary (recommended)
    ///
    /// This uses the built-in Chulalongkorn University TNC 2017 dictionary
    /// with ~40,000 Thai words. No external file needed.
    ///
    /// # Example
    /// ```
    /// use thai_text::ThaiWordcut;
    ///
    /// let wordcut = ThaiWordcut::embedded().unwrap();
    /// let words = wordcut.segment("สวัสดีครับ");
    /// ```
    pub fn embedded() -> Result<Self> {
        let dict = Dictionary::embedded()?;
        Ok(Self::new(dict))
    }

    /// Load wordcut from a dictionary file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let dict = Dictionary::from_file(path)?;
        Ok(Self::new(dict))
    }

    /// Load wordcut from dictionary content string
    pub fn from_str_content(content: &str) -> Result<Self> {
        let dict = Dictionary::from_str_content(content)?;
        Ok(Self::new(dict))
    }

    /// Segment Thai text into words using longest matching
    ///
    /// # Arguments
    /// * `text` - Thai text to segment
    ///
    /// # Returns
    /// Vector of words/segments
    pub fn segment(&self, text: &str) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut result = Vec::new();
        let mut pos = 0;

        while pos < chars.len() {
            // Try longest match first
            let max_len = std::cmp::min(self.dict.max_word_len(), chars.len() - pos);
            let mut matched = false;

            for len in (1..=max_len).rev() {
                let word: String = chars[pos..pos + len].iter().collect();
                if self.dict.contains(&word) {
                    result.push(word);
                    pos += len;
                    matched = true;
                    break;
                }
            }

            // If no match found, take single character
            if !matched {
                result.push(chars[pos].to_string());
                pos += 1;
            }
        }

        result
    }

    /// Word wrap Thai text with maximum character limit per line
    ///
    /// # Arguments
    /// * `text` - Thai text to wrap
    /// * `max_chars` - Maximum characters per line
    ///
    /// # Returns
    /// Vector of lines
    pub fn word_wrap(&self, text: &str, max_chars: usize) -> Vec<String> {
        if max_chars == 0 {
            return vec![text.to_string()];
        }

        let words = self.segment(text);
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_len = 0;

        for word in words {
            let word_len = word.chars().count();

            if current_len == 0 {
                // First word on line
                current_line = word;
                current_len = word_len;
            } else if current_len + word_len <= max_chars {
                // Word fits on current line
                current_line.push_str(&word);
                current_len += word_len;
            } else {
                // Word doesn't fit, start new line
                lines.push(current_line);
                current_line = word;
                current_len = word_len;
            }
        }

        // Don't forget the last line
        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// Get reference to the dictionary
    pub fn dictionary(&self) -> &Dictionary {
        &self.dict
    }
}

impl FromStr for ThaiWordcut {
    type Err = ThaiTextError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_str_content(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_wordcut() -> ThaiWordcut {
        let dict_content = "สวัสดี\nครับ\nค่ะ\nประเทศ\nไทย\nกรุงเทพ\nมหา\nนคร\n";
        ThaiWordcut::from_str_content(dict_content).unwrap()
    }

    #[test]
    fn test_segment_basic() {
        let wordcut = create_test_wordcut();
        let words = wordcut.segment("สวัสดีครับ");

        assert_eq!(words, vec!["สวัสดี", "ครับ"]);
    }

    #[test]
    fn test_segment_unknown_chars() {
        let wordcut = create_test_wordcut();
        let words = wordcut.segment("สวัสดีXYZครับ");

        // Unknown characters should be split individually
        assert_eq!(words.len(), 5); // สวัสดี, X, Y, Z, ครับ
    }

    #[test]
    fn test_segment_longest_match() {
        let wordcut = create_test_wordcut();
        let words = wordcut.segment("ประเทศไทย");

        assert_eq!(words, vec!["ประเทศ", "ไทย"]);
    }

    #[test]
    fn test_word_wrap() {
        let wordcut = create_test_wordcut();
        let lines = wordcut.word_wrap("สวัสดีครับประเทศไทย", 10);

        // Each line should be <= 10 chars
        for line in &lines {
            assert!(line.chars().count() <= 10);
        }
    }

    #[test]
    fn test_word_wrap_zero_max() {
        let wordcut = create_test_wordcut();
        let lines = wordcut.word_wrap("สวัสดี", 0);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "สวัสดี");
    }

    #[test]
    fn test_word_wrap_empty() {
        let wordcut = create_test_wordcut();
        let lines = wordcut.word_wrap("", 10);

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "");
    }
}
