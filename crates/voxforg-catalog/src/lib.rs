//! VoxForg Model Catalog and Weights Lifecycle Manager.
//!
//! Provides curated registries of open-source TTS, ASR, and VAD model weights,
//! hardware compatibility matching, SHA-256 integrity validation, and installation
//! tracking.

pub mod models;
pub mod store;

pub use models::{CatalogItem, ModelFormat, ModelStatus, ModelType};
pub use store::ModelCatalogStore;
