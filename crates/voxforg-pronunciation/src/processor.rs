//! `PronunciationProcessor` — the top-level entry point that chains all
//! normalization passes: symbol normalization, then dictionary substitution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::dictionary::{DictionaryEntry, PronunciationDictionary};
use crate::normalizer::TextNormalizer;

/// Static configuration for the processor.
/// The dictionary field seeds the runtime-editable [`PronunciationDictionary`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// Custom term → replacement overrides (merged with built-in seed entries).
    #[serde(default)]
    pub dictionary: HashMap<String, String>,
    /// If true, also apply symbol/number normalization before dictionary lookup.
    #[serde(default = "default_true")]
    pub normalize_symbols: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            dictionary: HashMap::new(),
            normalize_symbols: true,
        }
    }
}

/// The full pre-synthesis text processor.
///
/// Thread-safe via `Arc` internals on `PronunciationDictionary`.
/// Clone is cheap — `Dictionary` is reference-counted.
#[derive(Clone)]
pub struct PronunciationProcessor {
    dictionary: PronunciationDictionary,
    normalize_symbols: bool,
}

impl PronunciationProcessor {
    /// Create a processor from config.
    pub fn new(config: ProcessorConfig) -> Self {
        let dict = PronunciationDictionary::new();
        for (term, replacement) in config.dictionary {
            dict.upsert(DictionaryEntry {
                term,
                replacement,
                note: None,
            });
        }
        Self {
            dictionary: dict,
            normalize_symbols: config.normalize_symbols,
        }
    }

    /// Return a reference to the live dictionary for runtime updates.
    pub fn dictionary(&self) -> &PronunciationDictionary {
        &self.dictionary
    }

    /// Run the full processing pipeline on `text`.
    ///
    /// 1. Symbol / number normalization (if enabled)
    /// 2. Dictionary substitution (longest-term-first to avoid partial matches)
    pub fn process(&self, text: &str) -> String {
        let text = if self.normalize_symbols {
            TextNormalizer::normalize(text)
        } else {
            text.to_string()
        };
        self.dictionary.apply(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_sql_and_currency() {
        let mut config = ProcessorConfig::default();
        config
            .dictionary
            .insert("SQL".to_string(), "sequel".to_string());

        let processor = PronunciationProcessor::new(config);
        let result = processor.process("Run SQL query for ₹1,000");
        assert!(
            result.contains("sequel"),
            "SQL should be replaced: {result}"
        );
        assert!(
            result.contains("1000 rupees"),
            "currency should be normalized: {result}"
        );
    }

    #[test]
    fn runtime_dictionary_update_takes_effect() {
        let processor = PronunciationProcessor::new(ProcessorConfig::default());
        processor.dictionary().upsert(DictionaryEntry {
            term: "VoxForg".to_string(),
            replacement: "Vox Forge".to_string(),
            note: None,
        });
        let result = processor.process("Welcome to VoxForg");
        assert!(
            result.contains("Vox Forge"),
            "runtime insert should apply: {result}"
        );
    }

    #[test]
    fn symbol_normalization_disabled_leaves_symbols() {
        let config = ProcessorConfig {
            normalize_symbols: false,
            ..Default::default()
        };
        let processor = PronunciationProcessor::new(config);
        let result = processor.process("costs $100");
        assert!(result.contains('$'), "symbol should remain when disabled");
    }
}
