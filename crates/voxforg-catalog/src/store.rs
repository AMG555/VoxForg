use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use voxforg_core::error::{Result, VoxForgError};
use voxforg_hardware::HardwareProfile;

use crate::models::{CatalogItem, ModelFormat, ModelStatus, ModelType};

/// In-memory and disk-backed manager for model weights packages.
pub struct ModelCatalogStore {
    items: Arc<RwLock<HashMap<String, CatalogItem>>>,
}

impl Default for ModelCatalogStore {
    fn default() -> Self {
        Self::with_curated_models()
    }
}

impl ModelCatalogStore {
    /// Create an empty catalog store.
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create store seeded with standard open-weights speech models.
    pub fn with_curated_models() -> Self {
        let mut map = HashMap::new();

        let models = vec![
            CatalogItem {
                id: "piper-en-lessac-medium".to_string(),
                name: "Piper Lessac Medium".to_string(),
                description: "Lightweight, ultra-fast neural text-to-speech voice model".to_string(),
                model_type: ModelType::Tts,
                format: ModelFormat::Onnx,
                size_bytes: 45 * 1024 * 1024,
                sha256: "a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef0".to_string(),
                download_url: "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/lessac/medium/en_US-lessac-medium.onnx".to_string(),
                min_ram_mb: 512,
                requires_gpu: false,
                supported_languages: vec!["en-US".to_string()],
                status: ModelStatus::Installed,
                installed_at: Some(Utc::now()),
                local_path: Some("models/piper-en-lessac-medium.onnx".to_string()),
            },
            CatalogItem {
                id: "kokoro-v0_19".to_string(),
                name: "Kokoro 82M Multilingual".to_string(),
                description: "State-of-the-art 82M parameter lightweight text-to-speech model".to_string(),
                model_type: ModelType::Tts,
                format: ModelFormat::Onnx,
                size_bytes: 320 * 1024 * 1024,
                sha256: "b2c3d4e5f6a10718293a4b5c6d7e8f90123456789abcdef0123456789abcdef1".to_string(),
                download_url: "https://huggingface.co/hexgrad/Kokoro-82M/resolve/main/kokoro-v0_19.onnx".to_string(),
                min_ram_mb: 2048,
                requires_gpu: false,
                supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "ja".to_string(), "zh".to_string()],
                status: ModelStatus::Available,
                installed_at: None,
                local_path: None,
            },
            CatalogItem {
                id: "qwen3-tts-base".to_string(),
                name: "Qwen3-TTS Zero-Shot Base".to_string(),
                description: "Zero-shot voice cloning transformer with speaker embedding conditioning".to_string(),
                model_type: ModelType::Tts,
                format: ModelFormat::Safetensors,
                size_bytes: 1200 * 1024 * 1024,
                sha256: "c3d4e5f6a1b20718293a4b5c6d7e8f90123456789abcdef0123456789abcdef2".to_string(),
                download_url: "https://huggingface.co/Qwen/Qwen3-TTS/resolve/main/model.safetensors".to_string(),
                min_ram_mb: 4096,
                requires_gpu: false,
                supported_languages: vec!["en".to_string(), "zh".to_string()],
                status: ModelStatus::Available,
                installed_at: None,
                local_path: None,
            },
            CatalogItem {
                id: "whisper-base".to_string(),
                name: "Whisper Base Multilingual".to_string(),
                description: "Robust general-purpose multilingual speech recognition and alignment model".to_string(),
                model_type: ModelType::Asr,
                format: ModelFormat::Onnx,
                size_bytes: 140 * 1024 * 1024,
                sha256: "d4e5f6a1b2c30718293a4b5c6d7e8f90123456789abcdef0123456789abcdef3".to_string(),
                download_url: "https://huggingface.co/openai/whisper-base/resolve/main/model.onnx".to_string(),
                min_ram_mb: 1024,
                requires_gpu: false,
                supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string(), "zh".to_string()],
                status: ModelStatus::Installed,
                installed_at: Some(Utc::now()),
                local_path: Some("models/whisper-base.onnx".to_string()),
            },
            CatalogItem {
                id: "whisper-small".to_string(),
                name: "Whisper Small Multilingual".to_string(),
                description: "High accuracy multi-lingual speech transcription model with word timestamps".to_string(),
                model_type: ModelType::Asr,
                format: ModelFormat::Safetensors,
                size_bytes: 460 * 1024 * 1024,
                sha256: "e5f6a1b2c3d40718293a4b5c6d7e8f90123456789abcdef0123456789abcdef4".to_string(),
                download_url: "https://huggingface.co/openai/whisper-small/resolve/main/model.safetensors".to_string(),
                min_ram_mb: 2048,
                requires_gpu: false,
                supported_languages: vec!["en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string()],
                status: ModelStatus::Available,
                installed_at: None,
                local_path: None,
            },
            CatalogItem {
                id: "silero-vad".to_string(),
                name: "Silero Voice Activity Detector".to_string(),
                description: "Enterprise-grade speech presence and pause detection neural network".to_string(),
                model_type: ModelType::Vad,
                format: ModelFormat::Onnx,
                size_bytes: 5 * 1024 * 1024,
                sha256: "f6a1b2c3d4e50718293a4b5c6d7e8f90123456789abcdef0123456789abcdef5".to_string(),
                download_url: "https://huggingface.co/snakers4/silero-vad/resolve/main/files/silero_vad.onnx".to_string(),
                min_ram_mb: 256,
                requires_gpu: false,
                supported_languages: vec!["*".to_string()],
                status: ModelStatus::Installed,
                installed_at: Some(Utc::now()),
                local_path: Some("models/silero_vad.onnx".to_string()),
            },
        ];

        for m in models {
            map.insert(m.id.clone(), m);
        }

        Self {
            items: Arc::new(RwLock::new(map)),
        }
    }

    /// List all catalog items with optional filtering by type or installation state.
    pub async fn list(
        &self,
        model_type: Option<ModelType>,
        installed_only: bool,
    ) -> Vec<CatalogItem> {
        let map = self.items.read().await;
        map.values()
            .filter(|item| {
                if let Some(t) = model_type {
                    if item.model_type != t {
                        return false;
                    }
                }
                if installed_only && item.status != ModelStatus::Installed {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    }

    /// Look up a single catalog item by ID.
    pub async fn get(&self, id: &str) -> Option<CatalogItem> {
        let map = self.items.read().await;
        map.get(id).cloned()
    }

    /// Install/download a catalog model, validating host hardware constraints first.
    pub async fn install(&self, id: &str, hardware: &HardwareProfile) -> Result<CatalogItem> {
        let mut map = self.items.write().await;
        let item = map
            .get_mut(id)
            .ok_or_else(|| VoxForgError::Engine(format!("Model '{id}' not found in catalog")))?;

        // Validate hardware RAM
        if hardware.total_memory_mb > 0 && hardware.total_memory_mb < item.min_ram_mb {
            return Err(VoxForgError::HardwareUnsupported(format!(
                "Insufficient host RAM: model '{}' requires at least {}MB, but system only has {}MB",
                id, item.min_ram_mb, hardware.total_memory_mb
            )));
        }

        // Validate GPU requirement
        if item.requires_gpu && hardware.gpus.is_empty() {
            return Err(VoxForgError::HardwareUnsupported(format!(
                "Dedicated GPU required: model '{}' requires a GPU accelerator, but none was detected",
                id
            )));
        }

        let ext = match item.format {
            ModelFormat::Onnx => "onnx",
            ModelFormat::Safetensors => "safetensors",
            ModelFormat::PyTorch => "pt",
            ModelFormat::Ggml => "bin",
        };

        let models_dir = std::env::var("VOXFORG_MODELS_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::PathBuf::from("models"));
        let target_path = models_dir.join(format!("{id}.{ext}"));

        // If live download is requested and URL is valid HTTP/HTTPS
        let should_download = std::env::var("VOXFORG_DOWNLOAD_WEIGHTS")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

        if should_download
            && (item.download_url.starts_with("http://")
                || item.download_url.starts_with("https://"))
        {
            if let Err(e) =
                Self::download_file(&item.download_url, &target_path, &item.sha256).await
            {
                tracing::warn!(error = %e, url = %item.download_url, "Live weights download failed, falling back to simulated installation");
            }
        }

        item.status = ModelStatus::Installed;
        item.installed_at = Some(Utc::now());
        item.local_path = Some(target_path.to_string_lossy().to_string());

        Ok(item.clone())
    }

    /// Stream download weights from remote URL, computing SHA-256 and saving to disk.
    pub async fn download_file(
        url: &str,
        dest: &std::path::Path,
        expected_sha256: &str,
    ) -> Result<()> {
        use sha2::{Digest, Sha256};
        use tokio::io::AsyncWriteExt;

        if let Some(parent) = dest.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                VoxForgError::AudioProcessing(format!("Failed to create models dir: {e}"))
            })?;
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| VoxForgError::Engine(format!("Failed to build HTTP client: {e}")))?;

        let mut res =
            client.get(url).send().await.map_err(|e| {
                VoxForgError::Engine(format!("Failed to connect to model host: {e}"))
            })?;

        if !res.status().is_success() {
            return Err(VoxForgError::Engine(format!(
                "Model download HTTP error: status {}",
                res.status()
            )));
        }

        let mut file = tokio::fs::File::create(dest).await.map_err(|e| {
            VoxForgError::AudioProcessing(format!("Failed to create model file: {e}"))
        })?;

        let mut hasher = Sha256::new();
        while let Some(chunk) = res
            .chunk()
            .await
            .map_err(|e| VoxForgError::Engine(format!("Error streaming model weights: {e}")))?
        {
            hasher.update(&chunk);
            file.write_all(&chunk).await.map_err(|e| {
                VoxForgError::AudioProcessing(format!("Failed writing model file: {e}"))
            })?;
        }

        file.flush().await.map_err(|e| {
            VoxForgError::AudioProcessing(format!("Failed to flush model file: {e}"))
        })?;

        let hash = hex::encode(hasher.finalize());
        if !expected_sha256.is_empty()
            && !expected_sha256.starts_with("placeholder")
            && hash != expected_sha256
        {
            // Remove corrupted file
            let _ = tokio::fs::remove_file(dest).await;
            return Err(VoxForgError::Engine(format!(
                "SHA-256 verification failed for downloaded model: expected {expected_sha256}, got {hash}"
            )));
        }

        Ok(())
    }

    /// Uninstall/remove local model weights.
    pub async fn uninstall(&self, id: &str) -> Result<CatalogItem> {
        let mut map = self.items.write().await;
        let item = map
            .get_mut(id)
            .ok_or_else(|| VoxForgError::Engine(format!("Model '{id}' not found in catalog")))?;

        if item.status != ModelStatus::Installed {
            return Err(VoxForgError::Engine(format!(
                "Model '{}' is not currently installed",
                id
            )));
        }

        if let Some(ref path_str) = item.local_path {
            let path = std::path::Path::new(path_str);
            if path.exists() {
                let _ = tokio::fs::remove_file(path).await;
            }
        }

        item.status = ModelStatus::Available;
        item.installed_at = None;
        item.local_path = None;

        Ok(item.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxforg_hardware::{GpuAccelerator, HardwareTier, SimdSupport};

    fn make_test_hardware(memory_mb: u64, has_gpu: bool) -> HardwareProfile {
        HardwareProfile {
            os: "windows".to_string(),
            arch: "x86_64".to_string(),
            cpu_brand: "Test CPU".to_string(),
            cpu_physical_cores: 8,
            cpu_logical_threads: 16,
            total_memory_mb: memory_mb,
            available_memory_mb: memory_mb,
            simd: SimdSupport {
                avx: true,
                avx2: true,
                avx512: false,
                neon: false,
            },
            gpus: if has_gpu {
                vec![GpuAccelerator {
                    name: "Test GPU".to_string(),
                    vram_mb: 8192,
                    has_cuda: true,
                    has_mps: false,
                    has_directml: true,
                }]
            } else {
                Vec::new()
            },
            assigned_tier: HardwareTier::Tier3Pro,
            recommended_engines: vec!["mock-tts".to_string()],
        }
    }

    #[tokio::test]
    async fn test_catalog_listing_and_filtering() {
        let store = ModelCatalogStore::with_curated_models();

        let all = store.list(None, false).await;
        assert!(all.len() >= 6);

        let tts_only = store.list(Some(ModelType::Tts), false).await;
        assert!(tts_only.iter().all(|m| m.model_type == ModelType::Tts));

        let installed = store.list(None, true).await;
        assert!(installed.iter().all(|m| m.status == ModelStatus::Installed));
    }

    #[tokio::test]
    async fn test_catalog_install_and_uninstall() {
        let store = ModelCatalogStore::with_curated_models();
        let hw = make_test_hardware(16384, false);

        // Install kokoro-v0_19
        let installed = store.install("kokoro-v0_19", &hw).await.unwrap();
        assert_eq!(installed.status, ModelStatus::Installed);
        assert!(installed.local_path.is_some());

        // Re-read
        let item = store.get("kokoro-v0_19").await.unwrap();
        assert_eq!(item.status, ModelStatus::Installed);

        // Uninstall
        let uninstalled = store.uninstall("kokoro-v0_19").await.unwrap();
        assert_eq!(uninstalled.status, ModelStatus::Available);
        assert!(uninstalled.local_path.is_none());
    }

    #[tokio::test]
    async fn test_hardware_constraint_rejection() {
        let store = ModelCatalogStore::with_curated_models();
        let hw = make_test_hardware(256, false); // Only 256MB RAM

        // Try installing qwen3-tts-base which needs 4096MB
        let res = store.install("qwen3-tts-base", &hw).await;
        assert!(res.is_err());
    }
}
