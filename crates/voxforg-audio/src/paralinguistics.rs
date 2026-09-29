use std::f32::consts::PI;

/// Paralinguistic emotion expression tags supported in text inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParalinguisticTag {
    Laugh,
    Chuckle,
    Gasp,
    Sigh,
    Cough,
    Sniff,
    Groan,
    ClearThroat,
    PauseMs(u32),
}

/// Segment in a parsed paralinguistic script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParalinguisticSegment {
    Text(String),
    Emotion(ParalinguisticTag),
}

/// Check if text contains any recognized paralinguistic bracket tags.
pub fn has_paralinguistic_tags(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.contains("[laugh")
        || lower.contains("[chuckle")
        || lower.contains("[gasp")
        || lower.contains("[sigh")
        || lower.contains("[cough")
        || lower.contains("[sniff")
        || lower.contains("[groan")
        || lower.contains("[clear throat")
        || lower.contains("[clearthroat")
        || lower.contains("[pause:")
}

/// Parse paralinguistic tags and text into an ordered sequence of segments.
pub fn parse_paralinguistic_text(input: &str) -> Vec<ParalinguisticSegment> {
    let mut segments = Vec::new();
    let mut cursor = 0;

    while cursor < input.len() {
        if let Some(open_idx) = input[cursor..].find('[') {
            let tag_start = cursor + open_idx;
            if let Some(close_idx) = input[tag_start..].find(']') {
                let tag_end = tag_start + close_idx;
                let text_before = input[cursor..tag_start].trim();
                if !text_before.is_empty() {
                    segments.push(ParalinguisticSegment::Text(text_before.to_string()));
                }

                let tag_str = &input[tag_start + 1..tag_end].trim().to_lowercase();
                if let Some(tag) = parse_tag(tag_str) {
                    segments.push(ParalinguisticSegment::Emotion(tag));
                } else {
                    // Unrecognized tag: preserve as literal text
                    segments.push(ParalinguisticSegment::Text(input[tag_start..=tag_end].to_string()));
                }
                cursor = tag_end + 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if cursor < input.len() {
        let remainder = input[cursor..].trim();
        if !remainder.is_empty() {
            segments.push(ParalinguisticSegment::Text(remainder.to_string()));
        }
    }

    if segments.is_empty() && !input.trim().is_empty() {
        segments.push(ParalinguisticSegment::Text(input.trim().to_string()));
    }

    segments
}

fn parse_tag(tag_str: &str) -> Option<ParalinguisticTag> {
    match tag_str {
        "laugh" | "laughter" => Some(ParalinguisticTag::Laugh),
        "chuckle" => Some(ParalinguisticTag::Chuckle),
        "gasp" => Some(ParalinguisticTag::Gasp),
        "sigh" => Some(ParalinguisticTag::Sigh),
        "cough" => Some(ParalinguisticTag::Cough),
        "sniff" => Some(ParalinguisticTag::Sniff),
        "groan" => Some(ParalinguisticTag::Groan),
        "clear throat" | "clearthroat" => Some(ParalinguisticTag::ClearThroat),
        s if s.starts_with("pause:") => {
            let param = s.trim_start_matches("pause:").trim();
            if let Some(ms_str) = param.strip_suffix("ms") {
                ms_str.parse::<u32>().ok().map(ParalinguisticTag::PauseMs)
            } else if let Some(s_str) = param.strip_suffix('s') {
                s_str.parse::<f32>().ok().map(|sec| ParalinguisticTag::PauseMs((sec * 1000.0) as u32))
            } else {
                param.parse::<u32>().ok().map(ParalinguisticTag::PauseMs)
            }
        }
        _ => None,
    }
}

/// Procedural acoustic synthesizer for paralinguistic and non-verbal speech events.
pub struct ParalinguisticSynthesizer;

impl ParalinguisticSynthesizer {
    /// Synthesize high-fidelity PCM audio for a paralinguistic emotion event.
    pub fn synthesize_event(tag: ParalinguisticTag, sample_rate: u32) -> Vec<i16> {
        match tag {
            ParalinguisticTag::PauseMs(ms) => {
                let duration_s = (ms.min(10_000) as f32) / 1000.0;
                let sample_count = (sample_rate as f32 * duration_s) as usize;
                vec![0i16; sample_count]
            }
            ParalinguisticTag::Gasp => {
                // Inhalation breath: 220ms, rising amplitude and rising high-pass cutoff
                let duration_s = 0.22;
                let sample_count = (sample_rate as f32 * duration_s) as usize;
                let mut pcm = Vec::with_capacity(sample_count);
                let mut rng = 123456789u32;

                for i in 0..sample_count {
                    let t = i as f32 / sample_count as f32; // 0.0 to 1.0
                    // Inhalation envelope: exponential rise, quick release
                    let env = (t * PI * 0.5).sin().powf(2.0) * (1.0 - t).clamp(0.0, 1.0).powf(0.5);
                    // Filtered noise
                    rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                    let white = ((rng as f32 / 4294967296.0) * 2.0 - 1.0) * 0.7;
                    // Voiced inhalation tone
                    let formant = ((i as f32 / sample_rate as f32) * 650.0 * 2.0 * PI).sin() * 0.3;
                    let sample = ((white + formant) * env * 12000.0) as i16;
                    pcm.push(sample);
                }
                pcm
            }
            ParalinguisticTag::Sigh => {
                // Exhalation breath: 650ms, gentle attack, exponential decay with warm body
                let duration_s = 0.65;
                let sample_count = (sample_rate as f32 * duration_s) as usize;
                let mut pcm = Vec::with_capacity(sample_count);
                let mut rng = 987654321u32;

                for i in 0..sample_count {
                    let t = i as f32 / sample_count as f32;
                    let env = if t < 0.15 {
                        t / 0.15
                    } else {
                        (-3.2 * (t - 0.15)).exp()
                    };
                    rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                    let breath = ((rng as f32 / 4294967296.0) * 2.0 - 1.0) * 0.5;
                    let low_body = ((i as f32 / sample_rate as f32) * 220.0 * 2.0 * PI).sin() * 0.25;
                    let sample = ((breath + low_body) * env * 9500.0) as i16;
                    pcm.push(sample);
                }
                pcm
            }
            ParalinguisticTag::Chuckle => {
                // 3 gentle rhythmic glottal pulses: 380ms total
                let duration_s = 0.38;
                let sample_count = (sample_rate as f32 * duration_s) as usize;
                let mut pcm = Vec::with_capacity(sample_count);

                for i in 0..sample_count {
                    let t = i as f32 / sample_rate as f32;
                    // 3 pulses at ~8Hz
                    let pulse_env = (t * 8.0 * PI).sin().abs().powf(3.0);
                    let decay = (1.0 - (i as f32 / sample_count as f32)).powf(0.8);
                    let f0 = 180.0 + (t * 40.0).sin() * 20.0;
                    let tone = (t * f0 * 2.0 * PI).sin() * 0.6
                        + (t * f0 * 2.0 * 2.0 * PI).sin() * 0.3;
                    let sample = (tone * pulse_env * decay * 11000.0) as i16;
                    pcm.push(sample);
                }
                pcm
            }
            ParalinguisticTag::Laugh => {
                // Full laugh burst: 5 rhythmic voiced pulses with vocal timbre, 750ms
                let duration_s = 0.75;
                let sample_count = (sample_rate as f32 * duration_s) as usize;
                let mut pcm = Vec::with_capacity(sample_count);

                for i in 0..sample_count {
                    let t = i as f32 / sample_rate as f32;
                    let burst_env = (t * 6.5 * PI).sin().abs().powf(2.5);
                    let decay = (1.0 - (i as f32 / sample_count as f32)).powf(0.6);
                    let pitch = 220.0 + (t * 5.0).sin() * 35.0;
                    let vowel = (t * pitch * 2.0 * PI).sin() * 0.5
                        + (t * pitch * 2.0 * 2.0 * PI).sin() * 0.25
                        + (t * pitch * 3.0 * 2.0 * PI).sin() * 0.15;
                    let sample = (vowel * burst_env * decay * 14000.0) as i16;
                    pcm.push(sample);
                }
                pcm
            }
            ParalinguisticTag::Cough => {
                // Sharp acoustic pop transient + vocal fold resonance: 260ms
                let duration_s = 0.26;
                let sample_count = (sample_rate as f32 * duration_s) as usize;
                let mut pcm = Vec::with_capacity(sample_count);
                let mut rng = 543216789u32;

                for i in 0..sample_count {
                    let t = i as f32 / sample_rate as f32;
                    let env = if t < 0.02 {
                        t / 0.02
                    } else {
                        (-14.0 * (t - 0.02)).exp()
                    };
                    rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                    let burst = ((rng as f32 / 4294967296.0) * 2.0 - 1.0) * 0.8;
                    let body = (t * 310.0 * 2.0 * PI).sin() * 0.35;
                    let sample = ((burst + body) * env * 16000.0) as i16;
                    pcm.push(sample);
                }
                pcm
            }
            ParalinguisticTag::Sniff => {
                // Sharp nasal intake: 160ms
                let duration_s = 0.16;
                let sample_count = (sample_rate as f32 * duration_s) as usize;
                let mut pcm = Vec::with_capacity(sample_count);
                let mut rng = 777888999u32;

                for i in 0..sample_count {
                    let t = i as f32 / sample_rate as f32;
                    let env = (t * PI / duration_s).sin().powf(2.0);
                    rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                    let noise = (rng as f32 / 4294967296.0) * 2.0 - 1.0;
                    let sample = (noise * env * 10500.0) as i16;
                    pcm.push(sample);
                }
                pcm
            }
            ParalinguisticTag::Groan => {
                // Low vocal fold downward slide: 550ms
                let duration_s = 0.55;
                let sample_count = (sample_rate as f32 * duration_s) as usize;
                let mut pcm = Vec::with_capacity(sample_count);

                for i in 0..sample_count {
                    let t = i as f32 / sample_rate as f32;
                    let env = (t * PI / duration_s).sin();
                    let pitch = 140.0 - t * 50.0; // descending
                    let osc = (t * pitch * 2.0 * PI).sin() * 0.6
                        + (t * pitch * 2.0 * 2.0 * PI).sin() * 0.3;
                    let sample = (osc * env * 12500.0) as i16;
                    pcm.push(sample);
                }
                pcm
            }
            ParalinguisticTag::ClearThroat => {
                // Two consecutive rasping friction bursts: 350ms
                let duration_s = 0.35;
                let sample_count = (sample_rate as f32 * duration_s) as usize;
                let mut pcm = Vec::with_capacity(sample_count);
                let mut rng = 333444555u32;

                for i in 0..sample_count {
                    let t = i as f32 / sample_rate as f32;
                    let burst_env = (t * 5.7 * PI).sin().abs().powf(3.0);
                    rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                    let noise = ((rng as f32 / 4294967296.0) * 2.0 - 1.0) * 0.7;
                    let resonance = (t * 480.0 * 2.0 * PI).sin() * 0.3;
                    let sample = ((noise + resonance) * burst_env * 11500.0) as i16;
                    pcm.push(sample);
                }
                pcm
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_paralinguistic_tags() {
        let input = "Hello world! [laugh] That was funny. [sigh] Moving on.";
        let segments = parse_paralinguistic_text(input);
        assert_eq!(segments.len(), 5);
        assert_eq!(segments[0], ParalinguisticSegment::Text("Hello world!".into()));
        assert_eq!(segments[1], ParalinguisticSegment::Emotion(ParalinguisticTag::Laugh));
        assert_eq!(segments[2], ParalinguisticSegment::Text("That was funny.".into()));
        assert_eq!(segments[3], ParalinguisticSegment::Emotion(ParalinguisticTag::Sigh));
        assert_eq!(segments[4], ParalinguisticSegment::Text("Moving on.".into()));
    }

    #[test]
    fn test_synthesize_paralinguistic_events() {
        let gasp = ParalinguisticSynthesizer::synthesize_event(ParalinguisticTag::Gasp, 24000);
        assert!(!gasp.is_empty());
        assert_eq!(gasp.len(), (24000.0 * 0.22) as usize);

        let sigh = ParalinguisticSynthesizer::synthesize_event(ParalinguisticTag::Sigh, 24000);
        assert!(!sigh.is_empty());

        let pause = ParalinguisticSynthesizer::synthesize_event(ParalinguisticTag::PauseMs(250), 24000);
        assert_eq!(pause.len(), 6000);
        assert!(pause.iter().all(|&s| s == 0));
    }
}
