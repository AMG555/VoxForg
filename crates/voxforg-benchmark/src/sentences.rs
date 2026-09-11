//! Canonical benchmark sentence suite.
//!
//! These sentences are deliberately chosen to stress-test different dimensions
//! of TTS quality:
//! - General prose fluency
//! - Acronym handling (SQL, API, URL)
//! - Numbers and currency
//! - Punctuation and pacing
//! - Technical terminology
//! - Multilingual loanwords

/// Canonical sentences used in every benchmark run.
/// Adding sentences here automatically expands all future benchmark runs.
pub const BENCHMARK_SENTENCES: &[(&str, &str)] = &[
    // (id, text)
    ("prose_short",     "The quick brown fox jumps over the lazy dog."),
    ("prose_medium",    "In the beginning was the Word, and the Word was with God, and the Word was God."),
    ("numbers_simple",  "She earned $1,250 last month working remotely."),
    ("numbers_large",   "The market cap reached 2.4 trillion dollars by end of quarter."),
    ("acronyms",        "The REST API returns JSON over HTTPS using OAuth2 tokens."),
    ("technical",       "Configure nginx to proxy WebSocket connections on port 8080."),
    ("punctuation",     "Wait—are you sure? Yes, I am. Absolutely, positively sure!"),
    ("question_answer", "How does machine learning differ from traditional programming? It learns from data."),
    ("list_reading",    "The ingredients are: flour, sugar, butter, eggs, vanilla extract, and salt."),
    ("long_sentence",   "Despite the numerous challenges encountered during the development process, including unexpected hardware failures, software compatibility issues, and tight deadlines, the team managed to deliver a fully functional product ahead of schedule."),
];
