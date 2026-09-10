use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct MetricsCollector {
    requests_total: Arc<RwLock<HashMap<(String, u16), u64>>>,
    synthesis_count: Arc<AtomicU64>,
    synthesis_duration_ms_total: Arc<AtomicU64>,
    audio_samples_total: Arc<AtomicU64>,
    active_requests: Arc<AtomicI64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn record_request(&self, endpoint: &str, status: u16) {
        let mut map = self.requests_total.write().await;
        let counter = map.entry((endpoint.to_string(), status)).or_insert(0);
        *counter += 1;
    }

    pub fn record_synthesis(&self, duration_ms: u64, samples_count: usize) {
        self.synthesis_count.fetch_add(1, Ordering::Relaxed);
        self.synthesis_duration_ms_total.fetch_add(duration_ms, Ordering::Relaxed);
        self.audio_samples_total.fetch_add(samples_count as u64, Ordering::Relaxed);
    }

    pub fn inc_active_requests(&self) {
        self.active_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_active_requests(&self) {
        self.active_requests.fetch_sub(1, Ordering::Relaxed);
    }

    pub async fn render_prometheus(&self, cache_hits: u64, cache_misses: u64) -> String {
        let mut out = String::new();

        out.push_str("# HELP voxforg_active_requests Currently in-flight HTTP requests\n");
        out.push_str("# TYPE voxforg_active_requests gauge\n");
        out.push_str(&format!(
            "voxforg_active_requests {}\n\n",
            self.active_requests.load(Ordering::Relaxed).max(0)
        ));

        out.push_str("# HELP voxforg_requests_total Total number of HTTP requests processed by endpoint and status\n");
        out.push_str("# TYPE voxforg_requests_total counter\n");
        let req_map = self.requests_total.read().await;
        if req_map.is_empty() {
            out.push_str("voxforg_requests_total{endpoint=\"none\",status=\"200\"} 0\n");
        } else {
            for ((ep, status), count) in req_map.iter() {
                out.push_str(&format!(
                    "voxforg_requests_total{{endpoint=\"{}\",status=\"{}\"}} {}\n",
                    ep, status, count
                ));
            }
        }
        out.push('\n');

        out.push_str("# HELP voxforg_synthesis_total Total number of completed speech syntheses\n");
        out.push_str("# TYPE voxforg_synthesis_total counter\n");
        out.push_str(&format!(
            "voxforg_synthesis_total {}\n\n",
            self.synthesis_count.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP voxforg_synthesis_duration_seconds_total Total duration in seconds spent synthesizing audio\n");
        out.push_str("# TYPE voxforg_synthesis_duration_seconds_total counter\n");
        let total_secs = self.synthesis_duration_ms_total.load(Ordering::Relaxed) as f64 / 1000.0;
        out.push_str(&format!("voxforg_synthesis_duration_seconds_total {:.4}\n\n", total_secs));

        out.push_str("# HELP voxforg_audio_samples_total Total 16-bit PCM audio samples generated\n");
        out.push_str("# TYPE voxforg_audio_samples_total counter\n");
        out.push_str(&format!(
            "voxforg_audio_samples_total {}\n\n",
            self.audio_samples_total.load(Ordering::Relaxed)
        ));

        out.push_str("# HELP voxforg_cache_hits_total In-memory audio synthesis LRU cache hits\n");
        out.push_str("# TYPE voxforg_cache_hits_total counter\n");
        out.push_str(&format!("voxforg_cache_hits_total {}\n\n", cache_hits));

        out.push_str("# HELP voxforg_cache_misses_total In-memory audio synthesis LRU cache misses\n");
        out.push_str("# TYPE voxforg_cache_misses_total counter\n");
        out.push_str(&format!("voxforg_cache_misses_total {}\n\n", cache_misses));

        out
    }
}
