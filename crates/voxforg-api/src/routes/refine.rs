use axum::{extract::State, http::StatusCode, Json};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

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
        let filler_regex = Regex::new(
            r"(?i)\b(um|uh|er|ah|like|you know|sort of|kind of|i mean|basically|actually)\b[,.]?",
        )
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        fillers_count = filler_regex.find_iter(&refined).count();
        refined = filler_regex.replace_all(&refined, "").to_string();
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
        let space_regex = Regex::new(r"\s+").unwrap();
        refined = space_regex.replace_all(&refined, " ").trim().to_string();

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
