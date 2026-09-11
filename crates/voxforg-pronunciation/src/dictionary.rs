//! In-memory pronunciation dictionary with thread-safe concurrent access.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A single pronunciation override entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DictionaryEntry {
    /// The term to match (case-sensitive by default).
    pub term: String,
    /// The phonetic replacement text.
    pub replacement: String,
    /// Optional note about the override (e.g. "acronym", "brand name").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Thread-safe, runtime-editable pronunciation dictionary.
///
/// Loaded from a static seed map at startup; entries can be added/removed at
/// runtime via the API without a server restart.
#[derive(Clone)]
pub struct PronunciationDictionary {
    entries: Arc<RwLock<HashMap<String, DictionaryEntry>>>,
}

impl Default for PronunciationDictionary {
    fn default() -> Self {
        let mut map = HashMap::new();
        // Seed with common tech/domain terms that TTS engines pronounce poorly
        let seeds = [
            ("SQL", "sequel", Some("database query language")),
            ("NoSQL", "No sequel", Some("database paradigm")),
            ("APIs", "A P I s", Some("plural of API")),
            ("UI", "U I", None),
            ("CLI", "C L I", None),
            ("YAML", "yammel", None),
            ("GIF", "jif", Some("pronounced like peanut butter brand")),
            ("nginx", "engine X", None),
            ("AWS", "A W S", None),
            ("GCP", "G C P", None),
        ];
        for (term, replacement, note) in seeds {
            map.insert(
                term.to_string(),
                DictionaryEntry {
                    term: term.to_string(),
                    replacement: replacement.to_string(),
                    note: note.map(|s| s.to_string()),
                },
            );
        }
        Self {
            entries: Arc::new(RwLock::new(map)),
        }
    }
}

impl PronunciationDictionary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or update an entry. Returns the previous entry if it existed.
    pub fn upsert(&self, entry: DictionaryEntry) -> Option<DictionaryEntry> {
        let mut lock = self
            .entries
            .write()
            .expect("dictionary write lock poisoned");
        lock.insert(entry.term.clone(), entry)
    }

    /// Remove an entry by term. Returns the removed entry, or None if absent.
    pub fn remove(&self, term: &str) -> Option<DictionaryEntry> {
        let mut lock = self
            .entries
            .write()
            .expect("dictionary write lock poisoned");
        lock.remove(term)
    }

    /// Get a snapshot of all entries, sorted alphabetically.
    pub fn list(&self) -> Vec<DictionaryEntry> {
        let lock = self.entries.read().expect("dictionary read lock poisoned");
        let mut entries: Vec<_> = lock.values().cloned().collect();
        entries.sort_by(|a, b| a.term.cmp(&b.term));
        entries
    }

    /// Look up the replacement for `term`. Returns `None` if not in dictionary.
    pub fn lookup(&self, term: &str) -> Option<String> {
        let lock = self.entries.read().expect("dictionary read lock poisoned");
        lock.get(term).map(|e| e.replacement.clone())
    }

    /// Apply all dictionary substitutions to `text`.
    /// Matches whole-word occurrences of each term (case-sensitive).
    pub fn apply(&self, text: &str) -> String {
        let lock = self.entries.read().expect("dictionary read lock poisoned");

        // Sort by descending term length to avoid partial substitution (e.g. "NoSQL" before "SQL")
        let mut terms: Vec<(&String, &DictionaryEntry)> = lock.iter().collect();
        terms.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

        let mut result = text.to_string();
        for (term, entry) in terms {
            result = replace_whole_word(&result, term, &entry.replacement);
        }
        result
    }
}

/// Replace whole-word occurrences of `term` in `text` with `replacement`.
/// Word boundary is determined by non-alphanumeric characters.
fn replace_whole_word(text: &str, term: &str, replacement: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if text[i..].starts_with(term) {
            let before_is_boundary = i == 0
                || !text[..i]
                    .chars()
                    .last()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_');
            let after_pos = i + term.len();
            let after_is_boundary = after_pos >= text.len()
                || !text[after_pos..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_');

            if before_is_boundary && after_is_boundary {
                result.push_str(replacement);
                // Skip the rest of the matched term
                for _ in 1..term.chars().count() {
                    chars.next();
                }
                continue;
            }
        }
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_seeded_term_returns_replacement() {
        let dict = PronunciationDictionary::new();
        assert_eq!(dict.lookup("SQL"), Some("sequel".to_string()));
    }

    #[test]
    fn upsert_adds_new_entry() {
        let dict = PronunciationDictionary::new();
        dict.upsert(DictionaryEntry {
            term: "VoxForg".to_string(),
            replacement: "Vox Forge".to_string(),
            note: None,
        });
        assert_eq!(dict.lookup("VoxForg"), Some("Vox Forge".to_string()));
    }

    #[test]
    fn remove_deletes_entry() {
        let dict = PronunciationDictionary::new();
        dict.upsert(DictionaryEntry {
            term: "TMP".to_string(),
            replacement: "temp".to_string(),
            note: None,
        });
        let removed = dict.remove("TMP");
        assert!(removed.is_some());
        assert!(dict.lookup("TMP").is_none());
    }

    #[test]
    fn apply_replaces_sql_in_sentence() {
        let dict = PronunciationDictionary::new();
        let result = dict.apply("Run the SQL query");
        assert!(
            result.contains("sequel"),
            "expected 'sequel', got: {result}"
        );
        assert!(!result.contains("SQL"), "SQL should be replaced");
    }

    #[test]
    fn apply_does_not_replace_partial_word() {
        let dict = PronunciationDictionary::new();
        // "NoSQL" should not be replaced as just "sequel" (it has its own entry)
        let result = dict.apply("Use NoSQL databases");
        assert!(result.contains("No sequel"), "got: {result}");
    }

    #[test]
    fn list_returns_sorted_entries() {
        let dict = PronunciationDictionary::new();
        let entries = dict.list();
        let terms: Vec<_> = entries.iter().map(|e| &e.term).collect();
        let mut sorted = terms.clone();
        sorted.sort();
        assert_eq!(terms, sorted);
    }
}
