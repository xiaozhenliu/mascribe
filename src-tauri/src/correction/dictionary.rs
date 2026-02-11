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
