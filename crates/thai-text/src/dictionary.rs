//! Thai word dictionary for word segmentation

use crate::{Result, ThaiTextError};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// Embedded Thai dictionary from LibreOffice/Hunspell
/// Contains ~50,000 Thai words for word segmentation
/// Source: https://github.com/LibreOffice/dictionaries/blob/master/th_TH/th_TH.dic
pub const EMBEDDED_DICT: &str = include_str!("../data/th_TH.dic");

/// Thai word dictionary
#[derive(Debug, Clone)]
pub struct Dictionary {
    /// Set of known words
    words: HashSet<String>,
    /// Maximum word length in the dictionary
    max_word_len: usize,
}

impl Dictionary {
    /// Create a new empty dictionary
    pub fn new() -> Self {
        Self {
            words: HashSet::new(),
            max_word_len: 0,
        }
    }

    /// Create dictionary with embedded Thai dictionary (recommended)
    ///
    /// This uses the built-in LibreOffice/Hunspell Thai dictionary
    /// with ~50,000 Thai words.
    pub fn embedded() -> Result<Self> {
        Self::from_str_content(EMBEDDED_DICT)
    }

    /// Load dictionary from a file
    ///
    /// The file format is one word per line, UTF-8 encoded.
    ///
    /// # Arguments
    /// * `path` - Path to dictionary file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| ThaiTextError::DictionaryLoadError(e.to_string()))?;

        Self::from_str_content(&content)
    }

    /// Load dictionary from a string
    ///
    /// # Arguments
    /// * `content` - Dictionary content (one word per line)
    pub fn from_str_content(content: &str) -> Result<Self> {
        let mut dict = Self::new();

        for line in content.lines() {
            let word = line.trim();
            if !word.is_empty() && !word.starts_with('#') {
                dict.add_word(word);
            }
        }

        if dict.words.is_empty() {
            return Err(ThaiTextError::DictionaryLoadError(
                "Dictionary is empty".to_string(),
            ));
        }

        Ok(dict)
    }

    /// Add a word to the dictionary
    pub fn add_word(&mut self, word: &str) {
        let len = word.chars().count();
        if len > self.max_word_len {
            self.max_word_len = len;
        }
        self.words.insert(word.to_string());
    }

    /// Check if a word exists in the dictionary
    pub fn contains(&self, word: &str) -> bool {
        self.words.contains(word)
    }

    /// Get the maximum word length
    pub fn max_word_len(&self) -> usize {
        self.max_word_len
    }

    /// Get the number of words in the dictionary
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Check if dictionary is empty
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for Dictionary {
    type Err = ThaiTextError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_str_content(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dictionary_from_str() {
        let content = "สวัสดี\nครับ\nค่ะ\n";
        let dict = Dictionary::from_str_content(content).unwrap();

        assert_eq!(dict.len(), 3);
        assert!(dict.contains("สวัสดี"));
        assert!(dict.contains("ครับ"));
        assert!(dict.contains("ค่ะ"));
        assert!(!dict.contains("ไม่มี"));
    }

    #[test]
    fn test_dictionary_max_len() {
        let content = "ก\nกก\nกกก\n";
        let dict = Dictionary::from_str_content(content).unwrap();

        assert_eq!(dict.max_word_len(), 3);
    }

    #[test]
    fn test_dictionary_ignores_comments() {
        let content = "# This is a comment\nสวัสดี\n# Another comment\nครับ\n";
        let dict = Dictionary::from_str_content(content).unwrap();

        assert_eq!(dict.len(), 2);
        assert!(!dict.contains("# This is a comment"));
    }

    #[test]
    fn test_empty_dictionary_error() {
        let content = "# Only comments\n\n  \n";
        let result = Dictionary::from_str_content(content);

        assert!(result.is_err());
    }
}
