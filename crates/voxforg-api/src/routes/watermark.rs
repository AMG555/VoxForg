use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use voxforg_audio::dsp::{AudioWatermark, WatermarkDetectionResult, DEFAULT_SIGNATURE_PAYLOAD};
use voxforg_audio::WavEncoder;
use voxforg_core::error::ProblemDetails;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct EmbedWatermarkRequest {
    /// Base64 encoded WAV audio bytes
    pub audio_base64: String,
    /// Optional 16-bit payload (defaults to 0x5658 "VX")
    pub payload: Option<u16>,
    /// Optional embedding strength (0.001 - 0.01)
    pub strength: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct EmbedWatermarkResponse {
    pub audio_base64: String,
    pub payload: u16,
    pub repetitions: usize,
    pub duration_seconds: f32,
}

#[derive(Debug, Deserialize)]
pub struct VerifyWatermarkRequest {
    /// Base64 encoded WAV audio bytes
    pub audio_base64: String,
}

/// `POST /v1/audio/watermark` — embed invisible spread-spectrum watermark
pub async fn embed_watermark_handler(
    State(_state): State<AppState>,
    Json(body): Json<EmbedWatermarkRequest>,
) -> Result<Json<EmbedWatermarkResponse>, (StatusCode, Json<ProblemDetails>)> {
    let wav_bytes = match hex_or_base64_decode(&body.audio_base64) {
        Ok(b) => b,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/invalid-payload".to_string(),
                    title: "Invalid Base64 Audio".to_string(),
                    status: 400,
                    detail: e,
                    instance: "/v1/audio/watermark".to_string(),
                }),
            ));
        }
    };

    let (mut pcm16, sample_rate, channels) =
        WavEncoder::decode_wav_to_pcm16(&wav_bytes).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/wav-decode-failed".to_string(),
                    title: "WAV Decode Error".to_string(),
                    status: 400,
                    detail: e.to_string(),
                    instance: "/v1/audio/watermark".to_string(),
                }),
            )
        })?;

    let payload = body.payload.unwrap_or(DEFAULT_SIGNATURE_PAYLOAD);
    let mut wm = AudioWatermark::new(sample_rate);
    if let Some(s) = body.strength {
        wm = wm.with_strength(s);
    }

    let mut f32_samples: Vec<f32> = pcm16.iter().map(|&s| s as f32 / 32768.0).collect();
    let repetitions = wm.embed(&mut f32_samples, payload);

    for (dst, &src) in pcm16.iter_mut().zip(f32_samples.iter()) {
        *dst = (src * 32768.0).clamp(-32768.0, 32767.0) as i16;
    }

    let out_wav = WavEncoder::encode_pcm16_to_wav(&pcm16, sample_rate, channels).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ProblemDetails {
                problem_type: "https://voxforg.org/errors/wav-encode-failed".to_string(),
                title: "WAV Encode Error".to_string(),
                status: 500,
                detail: e.to_string(),
                instance: "/v1/audio/watermark".to_string(),
            }),
        )
    })?;

    let duration_seconds = pcm16.len() as f32 / (sample_rate as f32 * channels as f32);

    Ok(Json(EmbedWatermarkResponse {
        audio_base64: base64_encode(&out_wav),
        payload,
        repetitions,
        duration_seconds,
    }))
}

/// `POST /v1/audio/verify-watermark` — detect watermark and provenance signature
pub async fn verify_watermark_handler(
    State(_state): State<AppState>,
    Json(body): Json<VerifyWatermarkRequest>,
) -> Result<Json<WatermarkDetectionResult>, (StatusCode, Json<ProblemDetails>)> {
    let wav_bytes = match hex_or_base64_decode(&body.audio_base64) {
        Ok(b) => b,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/invalid-payload".to_string(),
                    title: "Invalid Base64 Audio".to_string(),
                    status: 400,
                    detail: e,
                    instance: "/v1/audio/verify-watermark".to_string(),
                }),
            ));
        }
    };

    let (pcm16, sample_rate, _channels) = WavEncoder::decode_wav_to_pcm16(&wav_bytes)
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ProblemDetails {
                    problem_type: "https://voxforg.org/errors/wav-decode-failed".to_string(),
                    title: "WAV Decode Error".to_string(),
                    status: 400,
                    detail: e.to_string(),
                    instance: "/v1/audio/verify-watermark".to_string(),
                }),
            )
        })?;

    let f32_samples: Vec<f32> = pcm16.iter().map(|&s| s as f32 / 32768.0).collect();
    let wm = AudioWatermark::new(sample_rate);
    let result = wm.detect(&f32_samples);

    Ok(Json(result))
}

fn hex_or_base64_decode(input: &str) -> Result<Vec<u8>, String> {
    // Standard base64 decoding
    let clean = input.trim().replace("\r", "").replace("\n", "");
    // Remove data URL prefix if present
    let data_str = if let Some(idx) = clean.find(";base64,") {
        &clean[idx + 8..]
    } else {
        &clean
    };

    // Use simple custom base64 decoder
    match simple_base64_decode(data_str) {
        Some(bytes) => Ok(bytes),
        None => Err("Failed to decode base64 audio payload".to_string()),
    }
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;

        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn simple_base64_decode(s: &str) -> Option<Vec<u8>> {
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;
    let mut out = Vec::new();

    for &b in s.as_bytes() {
        let val = match b {
            b'A'..=b'Z' => (b - b'A') as u32,
            b'a'..=b'z' => (b - b'a' + 26) as u32,
            b'0'..=b'9' => (b - b'0' + 52) as u32,
            b'+' => 62,
            b'/' => 63,
            b'=' | b' ' | b'\n' | b'\r' => continue,
            _ => return None,
        };
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }
    Some(out)
}
