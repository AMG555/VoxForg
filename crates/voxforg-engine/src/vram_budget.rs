//! VRAM Memory Budget & Idle Eviction Manager.
//!
//! Monitored background reaper that enforces hardware memory limits across local
//! neural models (Qwen, Kokoro, Piper) and evicts idle engines to avoid CUDA OOM crashes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMemoryFootprint {
    pub model_id: String,
    pub engine_id: String,
    pub vram_usage_mb: u64,
    pub ram_usage_mb: u64,
    pub is_loaded: bool,
    pub last_used_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VramBudgetConfig {
    pub max_vram_mb: u64,
    pub idle_ttl_seconds: u64,
    pub auto_evict_enabled: bool,
}

impl Default for VramBudgetConfig {
    fn default() -> Self {
        Self {
            max_vram_mb: 6144, // 6 GB default budget ceiling
            idle_ttl_seconds: 300, // 5 minutes idle TTL
            auto_evict_enabled: true,
        }
    }
}

#[derive(Clone)]
pub struct VramBudgetManager {
    config: Arc<RwLock<VramBudgetConfig>>,
    models: Arc<RwLock<HashMap<String, ModelMemoryFootprint>>>,
}

impl Default for VramBudgetManager {
    fn default() -> Self {
        Self::new(VramBudgetConfig::default())
    }
}

impl VramBudgetManager {
    pub fn new(config: VramBudgetConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a model as loaded with estimated memory footprint.
    pub async fn record_load(&self, model_id: &str, engine_id: &str, vram_mb: u64, ram_mb: u64) {
        let mut models = self.models.write().await;
        models.insert(
            model_id.to_string(),
            ModelMemoryFootprint {
                model_id: model_id.to_string(),
                engine_id: engine_id.to_string(),
                vram_usage_mb: vram_mb,
                ram_usage_mb: ram_mb,
                is_loaded: true,
                last_used_at: Utc::now(),
            },
        );
    }

    /// Touch a model to refresh its last-used timestamp.
    pub async fn touch(&self, model_id: &str) {
        let mut models = self.models.write().await;
        if let Some(m) = models.get_mut(model_id) {
            m.last_used_at = Utc::now();
        }
    }

    /// Check total currently allocated VRAM.
    pub async fn total_vram_used_mb(&self) -> u64 {
        let models = self.models.read().await;
        models.values().filter(|m| m.is_loaded).map(|m| m.vram_usage_mb).sum()
    }

    /// Find candidates for eviction based on idle TTL and LRU ordering.
    pub async fn find_eviction_candidates(&self) -> Vec<String> {
        let config = self.config.read().await;
        if !config.auto_evict_enabled {
            return Vec::new();
        }

        let now = Utc::now();
        let models = self.models.read().await;
        let mut loaded: Vec<&ModelMemoryFootprint> = models.values().filter(|m| m.is_loaded).collect();

        // Sort by last_used_at ascending (LRU first)
        loaded.sort_by_key(|m| m.last_used_at);

        let mut candidates = Vec::new();
        let mut current_vram: u64 = loaded.iter().map(|m| m.vram_usage_mb).sum();

        for m in loaded {
            let idle_duration_sec = now.signed_duration_since(m.last_used_at).num_seconds().max(0) as u64;
            let is_idle_expired = idle_duration_sec >= config.idle_ttl_seconds;
            let is_over_budget = current_vram > config.max_vram_mb;

            if is_idle_expired || is_over_budget {
                candidates.push(m.model_id.clone());
                current_vram = current_vram.saturating_sub(m.vram_usage_mb);
            }
        }

        candidates
    }

    /// Mark model as unloaded.
    pub async fn record_unload(&self, model_id: &str) {
        let mut models = self.models.write().await;
        if let Some(m) = models.get_mut(model_id) {
            m.is_loaded = false;
        }
    }
}
