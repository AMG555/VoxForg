//! # voxforg-pronunciation
//!
//! Pre-synthesis text normalisation and custom pronunciation dictionary for VoxForg.
//!
//! ## Pipeline
//! ```text
//! Raw text
//!   → SymbolNormalizer   (₹, $, %, &, …)
//!   → NumberNormalizer   (1K → "one thousand", 3.14 → "three point one four")
//!   → DictionaryLookup  (user-defined overrides: SQL → "sequel")
//!   → (TTS engine)
//! ```
//!
//! ## Usage
//! ```rust
//! use voxforg_pronunciation::{PronunciationProcessor, ProcessorConfig};
//!
//! let mut config = ProcessorConfig::default();
//! config.dictionary.insert("SQL".to_string(), "sequel".to_string());
//!
//! let processor = PronunciationProcessor::new(config);
//! let normalized = processor.process("Run SQL query for ₹1,000");
//! // → "Run sequel query for 1000 rupees"
//! ```

pub mod dictionary;
pub mod normalizer;
pub mod processor;

pub use dictionary::{DictionaryEntry, PronunciationDictionary};
pub use normalizer::TextNormalizer;
pub use processor::{ProcessorConfig, PronunciationProcessor};
