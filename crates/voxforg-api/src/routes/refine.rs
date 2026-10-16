use std::sync::LazyLock;

use axum::{extract::State, http::StatusCode, Json};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

static FILLER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(um|uh|er|ah|like|you know|sort of|kind of|i mean|basically|actually)\b[,.]?",
    )
    .expect("Valid filler regex")
});

static SPACE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+").expect("Valid space regex"));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineTextRequest {
    pub text: String,
    #[serde(default = "default_true")]
    pub strip_fillers: bool,
    #[serde(default = "default_true")]
    pub fix_punctuation: bool,
    #[serde(default = "default_true")]
    pub strip_hallucinations: bool,
    #[serde(default)]
    pub persona_prompt: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefineTextResponse {
    pub original_text: String,
    pub refined_text: String,
    pub fillers_removed: usize,
    pub hallucination_loops_removed: usize,
}

/// Refine dictated or transcribed text by stripping verbal fillers ("um", "uh"),
/// filtering repetitive hallucination loops, and formatting punctuation.
pub async fn refine_text(
    State(_state): State<AppState>,
    Json(payload): Json<RefineTextRequest>,
) -> Result<Json<RefineTextResponse>, (StatusCode, String)> {
    let mut refined = payload.text.clone();
    let mut fillers_count = 0;
    let mut loop_count = 0;

    // 1. Strip conversational filler words if requested
    if payload.strip_fillers {
        fillers_count = FILLER_REGEX.find_iter(&refined).count();
        refined = FILLER_REGEX.replace_all(&refined, "").to_string();
    }

    // 2. Strip Whisper hallucination repetition loops (loops of identical words)
    if payload.strip_hallucinations {
        let words: Vec<&str> = refined.split_whitespace().collect();
        if words.len() >= 4 {
            let mut cleaned_words = Vec::new();
            let mut i = 0;
            while i < words.len() {
                cleaned_words.push(words[i]);
                let mut run = 0;
                while i + 1 < words.len() && words[i].eq_ignore_ascii_case(words[i + 1]) {
                    run += 1;
                    i += 1;
                }
                if run >= 2 {
                    loop_count += 1;
                }
                i += 1;
            }
            refined = cleaned_words.join(" ");
        }
    }

    // 3. Fix punctuation and whitespace cleanup
    if payload.fix_punctuation {
        refined = SPACE_REGEX.replace_all(&refined, " ").trim().to_string();

        let mut chars = refined.chars();
        if let Some(first) = chars.next() {
            refined = first.to_uppercase().collect::<String>() + chars.as_str();
        }
    }

    Ok(Json(RefineTextResponse {
        original_text: payload.text,
        refined_text: refined,
        fillers_removed: fillers_count,
        hallucination_loops_removed: loop_count,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> AppState {
        let registry = std::sync::Arc::new(voxforg_engine::EngineRegistry::new());
        let store = std::sync::Arc::new(voxforg_core::store::memory::MemoryStore::new());
        let hardware = voxforg_hardware::HardwareProbe::probe();
        AppState::new(registry, store, hardware, None)
    }

    #[tokio::test]
    async fn test_refine_text_strips_fillers() {
        let state = test_state();
        let req = RefineTextRequest {
            text: "Um, so basically, we need to, like, ship the neural audio model.".to_string(),
            strip_fillers: true,
            fix_punctuation: true,
            strip_hallucinations: true,
            persona_prompt: None,
        };

        let Json(res) = refine_text(State(state), Json(req)).await.unwrap();
        assert!(res.fillers_removed >= 3);
        assert!(!res.refined_text.contains("basically"));
        assert!(!res.refined_text.to_lowercase().starts_with("um"));
    }

    #[tokio::test]
    async fn test_refine_text_strips_hallucination_loops() {
        let state = test_state();
        let req = RefineTextRequest {
            text: "The audio graph model model model model was executing properly.".to_string(),
            strip_fillers: false,
            fix_punctuation: true,
            strip_hallucinations: true,
            persona_prompt: None,
        };

        let Json(res) = refine_text(State(state), Json(req)).await.unwrap();
        assert!(res.hallucination_loops_removed >= 1);
        assert!(!res.refined_text.contains("model model"));
    }

    #[tokio::test]
    async fn test_refine_text_clean_noop() {
        let state = test_state();
        let req = RefineTextRequest {
            text: "Neural synthesis operates with high precision.".to_string(),
            strip_fillers: true,
            fix_punctuation: true,
            strip_hallucinations: true,
            persona_prompt: None,
        };

        let Json(res) = refine_text(State(state), Json(req)).await.unwrap();
        assert_eq!(res.fillers_removed, 0);
        assert_eq!(res.hallucination_loops_removed, 0);
        assert_eq!(res.refined_text, res.original_text);
    }
}
