use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

use sha2::{Digest, Sha256};
use voxforg_core::models::AudioChunk;

#[derive(Clone)]
pub struct AudioCache {
    capacity: usize,
    entries: Arc<RwLock<HashMap<String, AudioChunk>>>,
    lru_order: Arc<RwLock<VecDeque<String>>>,
    hits: Arc<AtomicU64>,
    misses: Arc<AtomicU64>,
}

impl AudioCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: Arc::new(RwLock::new(HashMap::new())),
            lru_order: Arc::new(RwLock::new(VecDeque::new())),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn compute_key(
        engine_id: &str,
        voice_id: &str,
        speed: f32,
        pitch: f32,
        text: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(engine_id.as_bytes());
        hasher.update(b":");
        hasher.update(voice_id.as_bytes());
        hasher.update(b":");
        hasher.update(speed.to_bits().to_be_bytes());
        hasher.update(b":");
        hasher.update(pitch.to_bits().to_be_bytes());
        hasher.update(b":");
        hasher.update(text.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn get(
        &self,
        engine_id: &str,
        voice_id: &str,
        speed: f32,
        pitch: f32,
        text: &str,
    ) -> Option<AudioChunk> {
        let key = Self::compute_key(engine_id, voice_id, speed, pitch, text);
        let entries = self.entries.read().await;

        if let Some(chunk) = entries.get(&key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            let mut lru = self.lru_order.write().await;
            if let Some(pos) = lru.iter().position(|k| k == &key) {
                lru.remove(pos);
            }
            lru.push_back(key);
            Some(chunk.clone())
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    pub async fn insert(
        &self,
        engine_id: &str,
        voice_id: &str,
        speed: f32,
        pitch: f32,
        text: &str,
        chunk: AudioChunk,
    ) {
        let key = Self::compute_key(engine_id, voice_id, speed, pitch, text);
        let mut entries = self.entries.write().await;
        let mut lru = self.lru_order.write().await;

        if entries.contains_key(&key) {
            entries.insert(key.clone(), chunk);
            if let Some(pos) = lru.iter().position(|k| k == &key) {
                lru.remove(pos);
            }
            lru.push_back(key);
            return;
        }

        while entries.len() >= self.capacity {
            if let Some(evicted_key) = lru.pop_front() {
                entries.remove(&evicted_key);
            } else {
                break;
            }
        }

        entries.insert(key.clone(), chunk);
        lru.push_back(key);
    }

    pub fn stats(&self) -> (u64, u64) {
        (
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
        )
    }

    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        let mut lru = self.lru_order.write().await;
        entries.clear();
        lru.clear();
    }

    pub async fn len(&self) -> usize {
        self.entries.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

impl Default for AudioCache {
    fn default() -> Self {
        Self::new(500)
    }
}
