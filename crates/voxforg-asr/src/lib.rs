//! VoxForg Automated Speech Recognition (ASR) layer.
//!
//! Provides abstract engine contracts, standard OpenAI-compatible transcription
//! types, word-level timestamping, and multi-engine registry routing.

pub mod mock;
pub mod registry;
pub mod traits;
pub mod types;
pub mod whisper;

pub use mock::MockAsrEngine;
pub use registry::AsrRegistry;
pub use traits::{AsrEngine, AsrEngineInfo};
pub use types::{TranscriptionOptions, TranscriptionResult, TranscriptionSegment, WordTimestamp};
