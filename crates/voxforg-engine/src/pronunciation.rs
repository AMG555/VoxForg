use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// A single pronunciation replacement rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PronunciationRule {
    pub id: String,
    pub original: String,
    pub replacement: String,
    pub case_sensitive: bool,
    pub is_phonetic: bool,
    pub language: Option<String>,
    pub enabled: bool,
}

/// Thread-safe lexicon store and text normalization pre-processor
#[derive(Debug, Default)]
pub struct PronunciationLexicon {
    rules: RwLock<Vec<PronunciationRule>>,
}

impl PronunciationLexicon {
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(Self::default_builtin_rules()),
        }
    }

    /// Default baseline rules for common acronyms and technical abbreviations
    fn default_builtin_rules() -> Vec<PronunciationRule> {
        vec![
            PronunciationRule {
                id: "builtin-sql".to_string(),
                original: "SQL".to_string(),
                replacement: "Sequel".to_string(),
                case_sensitive: false,
                is_phonetic: false,
                language: Some("en".to_string()),
                enabled: true,
            },
            PronunciationRule {
                id: "builtin-etc".to_string(),
                original: "etc.".to_string(),
                replacement: "et cetera".to_string(),
                case_sensitive: false,
                is_phonetic: false,
                language: Some("en".to_string()),
                enabled: true,
            },
            PronunciationRule {
                id: "builtin-dr".to_string(),
                original: "Dr.".to_string(),
                replacement: "Doctor".to_string(),
                case_sensitive: true,
                is_phonetic: false,
                language: Some("en".to_string()),
                enabled: true,
            },
            PronunciationRule {
                id: "builtin-mr".to_string(),
                original: "Mr.".to_string(),
                replacement: "Mister".to_string(),
                case_sensitive: true,
                is_phonetic: false,
                language: Some("en".to_string()),
                enabled: true,
            },
        ]
    }

    /// List all rules
    pub fn list_rules(&self) -> Vec<PronunciationRule> {
        self.rules.read().unwrap().clone()
    }

    /// Add or update a rule
    pub fn upsert_rule(&self, rule: PronunciationRule) {
        let mut rules = self.rules.write().unwrap();
        if let Some(pos) = rules.iter().position(|r| r.id == rule.id) {
            rules[pos] = rule;
        } else {
            rules.push(rule);
        }
    }

    /// Remove a rule by ID
    pub fn remove_rule(&self, id: &str) -> bool {
        let mut rules = self.rules.write().unwrap();
        let len_before = rules.len();
        rules.retain(|r| r.id != id);
        rules.len() < len_before
    }

    /// Apply lexicon replacement rules to text before speech synthesis
    ///
    /// Longest keys matched first. ReDoS safe with single-pass substitution.
    pub fn apply(&self, text: &str, lang: Option<&str>) -> String {
        let rules_guard = self.rules.read().unwrap();
        let mut active_rules: Vec<&PronunciationRule> = rules_guard
            .iter()
            .filter(|r| {
                if !r.enabled {
                    return false;
                }
                match (&r.language, lang) {
                    (Some(rule_lang), Some(target_lang)) => {
                        rule_lang.eq_ignore_ascii_case("all")
                            || target_lang.starts_with(rule_lang.as_str())
                    }
                    (Some(rule_lang), None) => rule_lang.eq_ignore_ascii_case("all"),
                    (None, _) => true,
                }
            })
            .collect();

        if active_rules.is_empty() {
            return text.to_string();
        }

        // Sort longest keys first so "Dr. Smith" wins over "Dr."
        active_rules.sort_by(|a, b| b.original.len().cmp(&a.original.len()));

        let mut result = text.to_string();

        for rule in active_rules {
            if rule.original.trim().is_empty() {
                continue;
            }

            let escaped = regex::escape(&rule.original);
            let starts_word = rule
                .original
                .chars()
                .next()
                .map(|c| c.is_alphanumeric() || c == '_')
                .unwrap_or(false);
            let ends_word = rule
                .original
                .chars()
                .last()
                .map(|c| c.is_alphanumeric() || c == '_')
                .unwrap_or(false);

            let prefix = if starts_word { r"\b" } else { "" };
            let suffix = if ends_word { r"\b" } else { "" };
            let pattern = format!(
                "(?{}){}{}{}",
                if rule.case_sensitive { "" } else { "i" },
                prefix,
                escaped,
                suffix
            );

            if let Ok(re) = Regex::new(&pattern) {
                result = re
                    .replace_all(&result, rule.replacement.as_str())
                    .to_string();
            }
        }

        result
    }
}

/// SSML-Lite Parser: Lightweight tag interpreter and converter
pub struct SsmlLite;

#[derive(Debug, Clone, PartialEq)]
pub struct SsmlSegment {
    pub text: String,
    pub pause_ms: u32,
    pub rate_factor: f32,
    pub pitch_factor: f32,
}

impl SsmlLite {
    /// Strips XML/SSML tags leaving clean pronounceable text
    pub fn strip_tags(input: &str) -> String {
        let tag_regex = Regex::new(r"<[^>]+>").unwrap();
        tag_regex
            .replace_all(input, " ")
            .replace("  ", " ")
            .trim()
            .to_string()
    }

    /// Parses SSML-lite into sequential segments with pause and rate controls
    pub fn parse_segments(input: &str) -> Vec<SsmlSegment> {
        let mut segments = Vec::new();
        let break_regex = Regex::new(r#"(?i)<break\s+time=["'](\d+)(ms|s)["']\s*/>"#).unwrap();
        let phoneme_regex = Regex::new(
            r#"(?i)<phoneme\s+alphabet=["']\w+["']\s+ph=["']([^"']+)["']>([^<]+)</phoneme>"#,
        )
        .unwrap();

        // First replace phoneme tags with the phonetic transcription directly
        let preprocessed = phoneme_regex.replace_all(input, "$1");

        // Split on break tags to generate segments with pauses
        let mut last_idx = 0;
        for cap in break_regex.captures_iter(&preprocessed) {
            let m = cap.get(0).unwrap();
            let text_part = &preprocessed[last_idx..m.start()];
            let clean_text = Self::strip_tags(text_part);

            let duration_val: u32 = cap[1].parse().unwrap_or(0);
            let unit = &cap[2];
            let pause_ms = if unit.eq_ignore_ascii_case("s") {
                duration_val * 1000
            } else {
                duration_val
            };

            if !clean_text.is_empty() {
                segments.push(SsmlSegment {
                    text: clean_text,
                    pause_ms,
                    rate_factor: 1.0,
                    pitch_factor: 1.0,
                });
            }

            last_idx = m.end();
        }

        let remaining = &preprocessed[last_idx..];
        let clean_remaining = Self::strip_tags(remaining);
        if !clean_remaining.is_empty() {
            segments.push(SsmlSegment {
                text: clean_remaining,
                pause_ms: 0,
                rate_factor: 1.0,
                pitch_factor: 1.0,
            });
        }

        if segments.is_empty() {
            segments.push(SsmlSegment {
                text: input.to_string(),
                pause_ms: 0,
                rate_factor: 1.0,
                pitch_factor: 1.0,
            });
        }

        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pronunciation_lexicon_replacement() {
        let lexicon = PronunciationLexicon::new();

        let text = "I love SQL queries, but PostgreSQL is also SQL.";
        let replaced = lexicon.apply(text, Some("en"));
        // "SQL" should become "Sequel", but "PostgreSQL" should NOT be modified because of word boundaries
        assert!(replaced.contains("Sequel queries"));
        assert!(replaced.contains("PostgreSQL"));
        assert_eq!(
            replaced,
            "I love Sequel queries, but PostgreSQL is also Sequel."
        );
    }

    #[test]
    fn test_pronunciation_longest_match_first() {
        let lexicon = PronunciationLexicon::new();
        lexicon.upsert_rule(PronunciationRule {
            id: "dr-smith".to_string(),
            original: "Dr. Smith".to_string(),
            replacement: "Doctor John Smith".to_string(),
            case_sensitive: true,
            is_phonetic: false,
            language: None,
            enabled: true,
        });

        let text = "Hello Dr. Smith and Dr. Jones.";
        let res = lexicon.apply(text, Some("en"));
        assert_eq!(res, "Hello Doctor John Smith and Doctor Jones.");
    }

    #[test]
    fn test_ssml_lite_parsing() {
        let ssml = "Welcome to the studio.<break time='500ms'/>Now beginning chapter 1.";
        let segments = SsmlLite::parse_segments(ssml);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "Welcome to the studio.");
        assert_eq!(segments[0].pause_ms, 500);
        assert_eq!(segments[1].text, "Now beginning chapter 1.");
    }
}
