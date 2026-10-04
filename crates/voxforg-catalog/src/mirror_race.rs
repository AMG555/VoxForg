use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Candidate mirror endpoint for model and weights distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorCandidate {
    pub name: String,
    pub base_url: String,
    pub priority: u32,
}

/// Result of probing an individual mirror
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorProbeResult {
    pub name: String,
    pub base_url: String,
    pub is_available: bool,
    pub latency_ms: u64,
    pub error: Option<String>,
}

/// Overall winner and benchmark details of mirror racing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorRaceResult {
    pub selected_mirror: String,
    pub selected_base_url: String,
    pub latency_ms: u64,
    pub probes: Vec<MirrorProbeResult>,
}

/// Multi-Endpoint Mirror Racer: Races primary endpoints against CDN/mirrors
#[derive(Debug, Clone)]
pub struct MirrorRaceManager {
    client: Client,
    candidates: Vec<MirrorCandidate>,
    probe_timeout: Duration,
}

impl Default for MirrorRaceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MirrorRaceManager {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        let candidates = vec![
            MirrorCandidate {
                name: "HuggingFace Official".to_string(),
                base_url: "https://huggingface.co".to_string(),
                priority: 1,
            },
            MirrorCandidate {
                name: "HF-Mirror Regional CDN".to_string(),
                base_url: "https://hf-mirror.com".to_string(),
                priority: 2,
            },
            MirrorCandidate {
                name: "VoxForg Edge CDN (Cloudflare R2)".to_string(),
                base_url: "https://cdn.voxforg.org/models".to_string(),
                priority: 3,
            },
        ];

        Self {
            client,
            candidates,
            probe_timeout: Duration::from_millis(1500),
        }
    }

    pub fn with_candidates(mut self, candidates: Vec<MirrorCandidate>) -> Self {
        self.candidates = candidates;
        self
    }

    /// Race all candidate mirrors with concurrent HEAD/GET probes and select fastest
    pub async fn race(&self, probe_path: &str) -> MirrorRaceResult {
        let mut tasks = Vec::new();

        for candidate in &self.candidates {
            let client = self.client.clone();
            let name = candidate.name.clone();
            let base_url = candidate.base_url.clone();
            let probe_url = format!("{}{}", base_url.trim_end_matches('/'), probe_path);
            let timeout = self.probe_timeout;

            tasks.push(tokio::spawn(async move {
                let start = Instant::now();
                match tokio::time::timeout(timeout, client.head(&probe_url).send()).await {
                    Ok(Ok(resp)) => {
                        let latency = start.elapsed().as_millis() as u64;
                        let is_available = resp.status().is_success() || resp.status().is_redirection();
                        MirrorProbeResult {
                            name,
                            base_url,
                            is_available,
                            latency_ms: latency,
                            error: if is_available {
                                None
                            } else {
                                Some(format!("HTTP {}", resp.status()))
                            },
                        }
                    }
                    Ok(Err(e)) => MirrorProbeResult {
                        name,
                        base_url,
                        is_available: false,
                        latency_ms: start.elapsed().as_millis() as u64,
                        error: Some(e.to_string()),
                    },
                    Err(_) => MirrorProbeResult {
                        name,
                        base_url,
                        is_available: false,
                        latency_ms: timeout.as_millis() as u64,
                        error: Some("Timeout exceeded".to_string()),
                    },
                }
            }));
        }

        let mut probe_results = Vec::new();
        for task in tasks {
            if let Ok(res) = task.await {
                probe_results.push(res);
            }
        }

        // Pick fastest available mirror, or fallback to first candidate
        let winner = probe_results
            .iter()
            .filter(|p| p.is_available)
            .min_by_key(|p| p.latency_ms);

        if let Some(best) = winner {
            debug!(
                "Mirror race won by '{}' ({} ms latency)",
                best.name, best.latency_ms
            );
            MirrorRaceResult {
                selected_mirror: best.name.clone(),
                selected_base_url: best.base_url.clone(),
                latency_ms: best.latency_ms,
                probes: probe_results,
            }
        } else {
            warn!("All mirror race probes timed out/failed; falling back to default mirror");
            let fallback = self.candidates.first().cloned().unwrap_or(MirrorCandidate {
                name: "Default".to_string(),
                base_url: "https://huggingface.co".to_string(),
                priority: 1,
            });
            MirrorRaceResult {
                selected_mirror: fallback.name,
                selected_base_url: fallback.base_url,
                latency_ms: 0,
                probes: probe_results,
            }
        }
    }

    /// Resolve full URL by prepending the fastest mirror
    pub async fn resolve_url(&self, relative_path: &str) -> String {
        let race = self.race("").await;
        format!(
            "{}/{}",
            race.selected_base_url.trim_end_matches('/'),
            relative_path.trim_start_matches('/')
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mirror_race_manager_fallback() {
        let manager = MirrorRaceManager::new().with_candidates(vec![
            MirrorCandidate {
                name: "Unreachable Port".to_string(),
                base_url: "http://127.0.0.1:59999".to_string(),
                priority: 1,
            },
        ]);

        let result = manager.race("/test").await;
        assert_eq!(result.selected_mirror, "Unreachable Port");
        assert_eq!(result.probes.len(), 1);
        assert!(!result.probes[0].is_available);
    }
}
