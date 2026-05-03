use std::collections::HashMap;
use std::path::Path;

pub struct CorrectionDictionary {
    entries: Vec<(String, String)>,
}

impl CorrectionDictionary {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let mut entries: Vec<(String, String)> = if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let map: HashMap<String, String> = serde_json::from_str(&content)?;
            map.into_iter().collect()
        } else {
            Vec::new()
        };
        // Sort by key length descending for longest-match-first
        entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        Ok(Self { entries })
    }

    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Get all entries as (from, to) pairs for frontend display.
    pub fn entries(&self) -> &[(String, String)] {
        &self.entries
    }

    /// Create from a list of (from, to) pairs. Sorts by key length for longest-match-first.
    pub fn from_entries(mut entries: Vec<(String, String)>) -> Self {
        entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        Self { entries }
    }

    /// Persist entries to a JSON file (HashMap format).
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let map: HashMap<String, String> = self.entries.iter().cloned().collect();
        let json = serde_json::to_string_pretty(&map)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn apply(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (from, to) in &self.entries {
            // Case-insensitive replacement
            let lower_result = result.to_lowercase();
            let lower_from = from.to_lowercase();
            if lower_result.contains(&lower_from) {
                // Simple replacement (case-insensitive by rebuilding)
                let mut new_result = String::new();
                let mut remaining = result.as_str();
                while let Some(pos) = remaining.to_lowercase().find(&lower_from) {
                    new_result.push_str(&remaining[..pos]);
                    new_result.push_str(to);
                    remaining = &remaining[pos + from.len()..];
                }
                new_result.push_str(remaining);
                result = new_result;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_replaces_exact_match() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("hello".to_string(), "world".to_string()),
        ]);
        assert_eq!(dict.apply("hello"), "world");
    }

    #[test]
    fn apply_is_case_insensitive() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("hello".to_string(), "world".to_string()),
        ]);
        assert_eq!(dict.apply("HELLO"), "world");
        assert_eq!(dict.apply("Hello"), "world");
        assert_eq!(dict.apply("hELLo"), "world");
    }

    #[test]
    fn apply_preserves_surrounding_text() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("foo".to_string(), "bar".to_string()),
        ]);
        assert_eq!(dict.apply("before foo after"), "before bar after");
    }

    #[test]
    fn apply_replaces_multiple_occurrences() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("cat".to_string(), "dog".to_string()),
        ]);
        assert_eq!(dict.apply("cat and cat"), "dog and dog");
    }

    #[test]
    fn empty_dictionary_returns_original() {
        let dict = CorrectionDictionary::empty();
        assert_eq!(dict.apply("unchanged text"), "unchanged text");
    }

    #[test]
    fn longest_match_first() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("ab".to_string(), "XY".to_string()),
            ("abc".to_string(), "Z".to_string()),
        ]);
        // "abc" should match before "ab" because it's longer
        assert_eq!(dict.apply("abc"), "Z");
    }

    #[test]
    fn from_entries_sorts_by_length_descending() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("a".to_string(), "1".to_string()),
            ("abc".to_string(), "3".to_string()),
            ("ab".to_string(), "2".to_string()),
        ]);
        let entries = dict.entries();
        assert_eq!(entries[0].0, "abc");
        assert_eq!(entries[1].0, "ab");
        assert_eq!(entries[2].0, "a");
    }

    #[test]
    fn empty_dictionary_has_no_entries() {
        let dict = CorrectionDictionary::empty();
        assert!(dict.entries().is_empty());
    }

    #[test]
    fn apply_with_chinese_text() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("你好".to_string(), "您好".to_string()),
        ]);
        assert_eq!(dict.apply("你好世界"), "您好世界");
    }

    #[test]
    fn apply_no_match_returns_unchanged() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("foo".to_string(), "bar".to_string()),
        ]);
        assert_eq!(dict.apply("no match here"), "no match here");
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dict = CorrectionDictionary::from_entries(vec![
            ("hello".to_string(), "world".to_string()),
            ("foo".to_string(), "bar".to_string()),
        ]);
        let tmp = std::env::temp_dir().join("mascribe_test_dict.json");
        dict.save(&tmp).unwrap();

        let loaded = CorrectionDictionary::load(&tmp).unwrap();
        // After load (which sorts by length), entries should contain both pairs
        assert_eq!(loaded.entries().len(), 2);
        assert_eq!(dict.apply("hello"), "world");
        assert_eq!(dict.apply("foo"), "bar");

        // Cleanup
        let _ = std::fs::remove_file(&tmp);
    }
}
