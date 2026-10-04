//! Advanced SubStation Alpha (.ass) Timed Karaoke Subtitle Exporter.
//!
//! Formats word-level timestamps with `{\k<centiseconds>}` karaoke tags
//! for video players and video editing timelines.

use crate::types::{TranscriptionResult, WordTimestamp};

pub struct AssSubtitleExporter;

impl AssSubtitleExporter {
    /// Formats transcription segments and word timestamps into a complete .ass file with karaoke tags.
    pub fn export_karaoke_ass(result: &TranscriptionResult, title: &str) -> String {
        let mut out = String::new();

        // Script Info header
        out.push_str("[Script Info]\n");
        out.push_str(&format!("Title: {}\n", title));
        out.push_str("ScriptType: v4.00+\n");
        out.push_str("WrapStyle: 0\n");
        out.push_str("ScaledBorderAndShadow: yes\n");
        out.push_str("YCbCr Matrix: TV.709\n");
        out.push_str("PlayResX: 1920\n");
        out.push_str("PlayResY: 1080\n\n");

        // Styles
        out.push_str("[V4+ Styles]\n");
        out.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
        // Style: PrimaryColour = &H00FFFFFF (White), SecondaryColour = &H0000D7FF (Amber Gold for Karaoke highlight)
        out.push_str("Style: Karaoke,Arial,48,&H00FFFFFF,&H0000D7FF,&H00000000,&H80000000,1,0,0,0,100,100,0,0,1,3,2,2,40,40,60,1\n\n");

        // Events
        out.push_str("[Events]\n");
        out.push_str("Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");

        for seg in &result.segments {
            let start_ts = Self::format_ass_time(seg.start_ms);
            let end_ts = Self::format_ass_time(seg.end_ms);

            let text = if let Some(words) = &seg.words {
                if !words.is_empty() {
                    Self::format_karaoke_words(words, seg.start_ms)
                } else {
                    seg.text.trim().to_string()
                }
            } else {
                seg.text.trim().to_string()
            };

            out.push_str(&format!(
                "Dialogue: 0,{},{},Karaoke,,0,0,0,,{}\n",
                start_ts, end_ts, text
            ));
        }

        out
    }

    /// Formats milliseconds into ASS timestamp `H:MM:SS.cc` (centiseconds).
    pub fn format_ass_time(ms: u64) -> String {
        let total_seconds = ms / 1000;
        let centis = (ms % 1000) / 10;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        format!("{}:{:02}:{:02}.{:02}", hours, minutes, seconds, centis)
    }

    /// Generates `{\k<centiseconds>}word` tag sequence.
    fn format_karaoke_words(words: &[WordTimestamp], segment_start_ms: u64) -> String {
        let mut text = String::new();
        let mut cur_ms = segment_start_ms;

        for w in words {
            // Gap before word
            if w.start_ms > cur_ms {
                let gap_cs = (w.start_ms - cur_ms) / 10;
                if gap_cs > 0 {
                    text.push_str(&format!("{{\\k{}}} ", gap_cs));
                }
            }

            let word_duration_ms = w.end_ms.saturating_sub(w.start_ms).max(20);
            let word_cs = (word_duration_ms / 10).max(1);
            text.push_str(&format!("{{\\k{}}}{} ", word_cs, w.word.trim()));
            cur_ms = w.end_ms;
        }

        text.trim_end().to_string()
    }
}
