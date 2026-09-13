use serde::{Deserialize, Serialize};

/// Options for configuring an ASR transcription request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionOptions {
    /// Optional ISO-639-1 language code (e.g. "en", "es", "de").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Sampling temperature between 0.0 and 1.0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Optional guiding text prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,

    /// Whether to generate word-level timestamps.
    #[serde(default)]
    pub word_timestamps: bool,

    /// Response serialization format: "json", "text", "srt", "vtt", "verbose_json".
    #[serde(default = "default_format")]
    pub response_format: String,
}

fn default_format() -> String {
    "json".to_string()
}

impl Default for TranscriptionOptions {
    fn default() -> Self {
        Self {
            language: None,
            temperature: None,
            prompt: None,
            word_timestamps: true,
            response_format: default_format(),
        }
    }
}

/// Word-level timestamp information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WordTimestamp {
    pub word: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub probability: f32,
}

/// Timed segment within an audio transcription.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranscriptionSegment {
    pub id: u32,
    pub seek: u32,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub tokens: Vec<u32>,
    pub temperature: f32,
    pub avg_logprob: f32,
    pub compression_ratio: f32,
    pub no_speech_prob: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub words: Vec<WordTimestamp>,
}

/// Complete ASR transcription result payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranscriptionResult {
    pub text: String,
    pub task: String,
    pub language: String,
    pub duration_seconds: f32,
    pub segments: Vec<TranscriptionSegment>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub words: Vec<WordTimestamp>,
}

impl TranscriptionResult {
    /// Format segments as SubRip Text (.srt) format.
    pub fn to_srt(&self) -> String {
        let mut out = String::new();
        for (i, seg) in self.segments.iter().enumerate() {
            let start = format_srt_time(seg.start_ms);
            let end = format_srt_time(seg.end_ms);
            out.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                i + 1,
                start,
                end,
                seg.text.trim()
            ));
        }
        out
    }

    /// Format segments as WebVTT (.vtt) format.
    pub fn to_vtt(&self) -> String {
        let mut out = String::from("WEBVTT\n\n");
        for (i, seg) in self.segments.iter().enumerate() {
            let start = format_vtt_time(seg.start_ms);
            let end = format_vtt_time(seg.end_ms);
            out.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                i + 1,
                start,
                end,
                seg.text.trim()
            ));
        }
        out
    }
}

fn format_srt_time(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let minutes = (ms % 3_600_000) / 60_000;
    let seconds = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    format!("{hours:02}:{minutes:02}:{seconds:02},{millis:03}")
}

fn format_vtt_time(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let minutes = (ms % 3_600_000) / 60_000;
    let seconds = (ms % 60_000) / 1000;
    let millis = ms % 1000;
    format!("{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srt_and_vtt_formatting() {
        let res = TranscriptionResult {
            text: "Hello world. Testing transcription.".to_string(),
            task: "transcribe".to_string(),
            language: "en".to_string(),
            duration_seconds: 3.5,
            segments: vec![
                TranscriptionSegment {
                    id: 0,
                    seek: 0,
                    start_ms: 0,
                    end_ms: 1200,
                    text: "Hello world.".to_string(),
                    tokens: vec![50364, 15496, 995, 50424],
                    temperature: 0.0,
                    avg_logprob: -0.15,
                    compression_ratio: 1.2,
                    no_speech_prob: 0.01,
                    speaker: Some("SPEAKER_00".to_string()),
                    words: vec![],
                },
                TranscriptionSegment {
                    id: 1,
                    seek: 1200,
                    start_ms: 1300,
                    end_ms: 3500,
                    text: "Testing transcription.".to_string(),
                    tokens: vec![50424, 8820, 24653, 50534],
                    temperature: 0.0,
                    avg_logprob: -0.12,
                    compression_ratio: 1.1,
                    no_speech_prob: 0.005,
                    speaker: Some("SPEAKER_00".to_string()),
                    words: vec![],
                },
            ],
            words: vec![],
        };

        let srt = res.to_srt();
        assert!(srt.contains("00:00:00,000 --> 00:00:01,200"));
        assert!(srt.contains("Hello world."));
        assert!(srt.contains("00:00:01,300 --> 00:00:03,500"));

        let vtt = res.to_vtt();
        assert!(vtt.starts_with("WEBVTT\n\n"));
        assert!(vtt.contains("00:00:00.000 --> 00:00:01.200"));
    }
}
