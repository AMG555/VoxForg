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
        let seeds: &[(&str, &str, Option<&str>)] = &[
            // Tech terms & acronyms
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
            ("LLM", "L L M", None),
            ("TTS", "T T S", None),
            ("ASR", "A S R", None),
            ("DSP", "D S P", None),
            // English Colloquial & Informal Pacing
            ("gonna", "going to", Some("informal future")),
            ("wanna", "want to", Some("informal volition")),
            ("gotta", "got to", Some("informal obligation")),
            ("lemme", "let me", Some("colloquial request")),
            ("gimme", "give me", Some("colloquial imperative")),
            ("kinda", "kind of", Some("hedging particle")),
            ("sorta", "sort of", Some("hedging particle")),
            ("dunno", "don't know", Some("casual negation")),
            ("y'all", "you all", Some("Southern American plural")),
            ("innit", "isn't it", Some("British tag question")),
            ("blimey", "blimey", Some("British surprise exclamation")),
            (
                "no worries",
                "no worries",
                Some("Australian / casual assurance"),
            ),
            ("arvo", "afternoon", Some("Australian slang")),
            ("brekkie", "breakfast", Some("Australian / British slang")),
            // Hinglish / Indian English Colloquial
            (
                "yaar",
                "yaar",
                Some("Hinglish friend / conversational address"),
            ),
            ("jugaad", "jugaad", Some("Hinglish creative hack")),
            ("bindaas", "bindaas", Some("Hinglish carefree / awesome")),
            ("accha", "accha", Some("Hinglish okay / I see")),
            ("chalo", "chalo", Some("Hinglish come on / let's go")),
            ("theek hai", "theek hai", Some("Hinglish all right")),
            ("fatafat", "fatafat", Some("Hinglish quickly")),
            // Malayalam (മലയാളം - Kerala Colloquial & Youth Slang)
            (
                "adipoli",
                "adipoli",
                Some("Malayalam slang superb / awesome"),
            ),
            ("kidilam", "kidilam", Some("Malayalam slang epic / killer")),
            (
                "ente ponno",
                "ente ponno",
                Some("Malayalam exclamation of awe / my goodness"),
            ),
            ("machane", "machane", Some("Malayalam casual dude / bro")),
            (
                "chunke",
                "chunke",
                Some("Malayalam slang best friend / dear"),
            ),
            ("pwoli", "pwoli", Some("Malayalam slang lit / rocking")),
            (
                "sheriyaa",
                "sheriyaa",
                Some("Malayalam colloquial that's right / okay"),
            ),
            (
                "scene aanu",
                "scene aanu",
                Some("Malayalam slang intense vibe"),
            ),
            (
                "set aayi",
                "set aayi",
                Some("Malayalam slang sorted / fixed"),
            ),
            (
                "oru rakshayum illa",
                "oru rakshayum illa",
                Some("Malayalam expression breathtaking / unmatched"),
            ),
            (
                "enthokke undu",
                "enthokke undu",
                Some("Malayalam greeting what's up"),
            ),
            ("nannayi", "nannayi", Some("Malayalam well done")),
            ("kollam", "kollam", Some("Malayalam colloquial nice")),
            ("അടിപൊളി", "അടിപൊളി", Some("Malayalam script adipoli")),
            ("കിടിലം", "കിടിലം", Some("Malayalam script kidilam")),
            ("പൊളി", "പൊളി", Some("Malayalam script pwoli")),
            ("മച്ചാനേ", "മച്ചാനേ", Some("Malayalam script machane")),
            (
                "എന്റെ പൊന്നോ",
                "എന്റെ പൊന്നോ",
                Some("Malayalam script ente ponno"),
            ),
            // Tamil (தமிழ் - Chennai / Colloquial)
            ("machan", "machan", Some("Tamil casual dude / brother")),
            ("thalaiva", "thalaiva", Some("Tamil leader / boss")),
            ("semma", "semma", Some("Tamil slang super / awesome")),
            ("veralevel", "vera level", Some("Tamil next level")),
            ("vera level", "vera level", Some("Tamil slang top tier")),
            ("kandippa", "kandippa", Some("Tamil definitely")),
            ("apdiya", "apdiya", Some("Tamil is that so")),
            // Telugu & Kannada (తెలుగు & ಕನ್ನಡ)
            ("mowa", "mowa", Some("Telugu casual bro / dude")),
            ("keka", "keka", Some("Telugu slang awesome / rocking")),
            ("bava", "bava", Some("Telugu colloquial bro")),
            ("maga", "maga", Some("Kannada casual bro / dude")),
            (
                "channagide",
                "channagide",
                Some("Kannada it is good / nice"),
            ),
            ("sakkath", "sakkath", Some("Kannada slang superb / amazing")),
            // Spanish (Spain & Latin America)
            ("pa'", "para", Some("Spanish casual contraction")),
            ("pa'l", "para el", Some("Spanish casual contraction")),
            ("ta' bien", "está bien", Some("Spanish casual all right")),
            ("chido", "chido", Some("Mexican slang cool")),
            ("güey", "wey", Some("Mexican colloquial mate / dude")),
            ("che", "che", Some("Argentine address")),
            ("boludo", "boludo", Some("Argentine colloquial address")),
            ("vale", "vale", Some("Castilian Spanish all right")),
            ("guay", "guay", Some("Castilian Spanish cool")),
            (
                "de una",
                "de una",
                Some("Latin American Spanish definitely"),
            ),
            // French (France & Quebec)
            ("t'sais", "tu sais", Some("French conversational filler")),
            ("y'a", "il y a", Some("French contraction there is")),
            ("ouais", "ouais", Some("French casual yes")),
            ("boulot", "boulot", Some("French slang work")),
            ("ouf", "ouf", Some("French verlan crazy / relief")),
            ("nickel", "nickel", Some("French slang perfect")),
            ("chum", "chum", Some("Quebec French boyfriend / friend")),
            ("blonde", "blonde", Some("Quebec French girlfriend")),
            ("pantoute", "pas du tout", Some("Quebec French not at all")),
            // German (Casual & Colloquial)
            ("mach's", "mach es", Some("German contraction do it")),
            (
                "geht's",
                "geht es",
                Some("German contraction how is it going"),
            ),
            ("krass", "krass", Some("German slang sick / awesome")),
            ("alter", "alter", Some("German casual dude")),
            ("alles klar", "alles klar", Some("German all right")),
            ("kein ding", "kein ding", Some("German no problem")),
            // Japanese (Casual & Colloquial)
            ("ヤバい", "やばい", Some("Japanese colloquial wow / crazy")),
            ("マジで", "まじで", Some("Japanese colloquial really")),
            ("ウケる", "うける", Some("Japanese slang hilarious")),
            (
                "お疲れ様",
                "おつかれさま",
                Some("Japanese greeting good work"),
            ),
            ("よろしく", "よろしく", Some("Japanese casual plea")),
            ("めっちゃ", "めっちゃ", Some("Kansai Japanese slang very")),
            // Chinese / Mandarin (Colloquial & Net Slang)
            ("给力", "给力", Some("Chinese slang awesome")),
            ("牛逼", "牛逼", Some("Chinese slang badass / incredible")),
            ("好家伙", "好家伙", Some("Chinese colloquial good heavens")),
            ("杠杠的", "杠杠的", Some("Northeastern Chinese top notch")),
            // Italian (Casual & Fillers)
            ("dai", "dai", Some("Italian conversational come on")),
            ("boh", "boh", Some("Italian filler who knows")),
            ("figata", "figata", Some("Italian slang cool thing")),
            (
                "magari",
                "magari",
                Some("Italian expression if only / I wish"),
            ),
            // Portuguese (Brazil & Portugal)
            ("pra", "para", Some("Portuguese contraction for / to")),
            ("pro", "para o", Some("Portuguese contraction for the")),
            ("tá", "está", Some("Portuguese contraction is")),
            ("beleza", "beleza", Some("Brazilian slang cool / all good")),
            ("valeu", "valeu", Some("Brazilian casual thanks")),
            ("tranquilo", "tranquilo", Some("Portuguese no worries")),
            // Arabic (Dialectal Fillers & Conversational)
            ("يلا", "يلا", Some("Arabic colloquial let's go")),
            ("حبيبي", "حبيبي", Some("Arabic term of endearment my dear")),
            ("ماشي", "ماشي", Some("Arabic colloquial all right")),
            ("تمام", "تمام", Some("Arabic colloquial great / fine")),
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
