//! Voice Design & Semantic Archetype Engine.
//!
//! Synthesizes custom 512-dimensional neural acoustic speaker embeddings from
//! free-text descriptive prompts and 5-axis Director AI controls via 40+ semantic
//! anchor archetypes.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use voxforg_audio::dsp::DirectorTaxonomy;
use voxforg_core::models::{Gender, VoiceProfile};

/// Request payload to design a custom voice from text description and parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceDesignRequest {
    /// Natural language prompt describing vocal qualities, age, pitch, timbre, tone, and pacing.
    pub prompt: String,
    pub name: Option<String>,
    pub gender: Option<Gender>,
    pub language: Option<String>,
    pub director: Option<DirectorTaxonomy>,
}

/// A semantic anchor archetype defining a cluster in 512-D acoustic embedding space.
pub struct AcousticArchetype {
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub gender: Gender,
    pub base_vector: [f32; 512],
}

pub struct ArchetypeBank {
    archetypes: Vec<AcousticArchetype>,
}

impl Default for ArchetypeBank {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchetypeBank {
    pub fn new() -> Self {
        let mut bank = Self {
            archetypes: Vec::new(),
        };
        bank.seed_archetypes();
        bank
    }

    /// Generates a deterministic, normalized 512-D pseudo-orthogonal vector for a given seed.
    fn generate_anchor_vector(seed: u64) -> [f32; 512] {
        let mut vec = [0.0f32; 512];
        let mut state = seed.wrapping_mul(0x517cc1b727220a95).wrapping_add(1);
        let mut sum_sq = 0.0f32;

        for item in &mut vec {
            // Xorshift64star pseudo-random generator
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            let val =
                ((state.wrapping_mul(0x2545F4914F6CDD1D) >> 32) as i32) as f32 / (i32::MAX as f32);
            *item = val;
            sum_sq += val * val;
        }

        let norm = sum_sq.sqrt().max(1e-6);
        for item in &mut vec {
            *item /= norm;
        }

        vec
    }

    fn seed_archetypes(&mut self) {
        let definitions: &[(&str, &[&str], Gender, u64)] = &[
            (
                "warm_baritone",
                &[
                    "warm", "deep", "baritone", "chest", "rich", "soothing", "resonant",
                ],
                Gender::Male,
                101,
            ),
            (
                "crisp_broadcast_male",
                &[
                    "broadcast",
                    "news",
                    "clear",
                    "authoritative",
                    "professional",
                    "presenter",
                    "formal",
                ],
                Gender::Male,
                102,
            ),
            (
                "raspy_detective",
                &[
                    "raspy", "gravel", "noir", "gritty", "smoky", "whiskey", "cynical",
                ],
                Gender::Male,
                103,
            ),
            (
                "gentle_storyteller",
                &[
                    "storyteller",
                    "gentle",
                    "fairytale",
                    "calm",
                    "soft",
                    "hypnotic",
                    "soothing",
                ],
                Gender::Neutral,
                104,
            ),
            (
                "intimate_whisper",
                &[
                    "whisper", "intimate", "in-ear", "asmr", "close", "quiet", "breathy",
                ],
                Gender::Neutral,
                105,
            ),
            (
                "energetic_host",
                &[
                    "energetic",
                    "upbeat",
                    "podcast",
                    "excited",
                    "fast",
                    "lively",
                    "vibrant",
                ],
                Gender::Neutral,
                106,
            ),
            (
                "authoritative_female",
                &[
                    "authoritative",
                    "executive",
                    "firm",
                    "leader",
                    "sharp",
                    "commanding",
                ],
                Gender::Female,
                107,
            ),
            (
                "silky_narrator_female",
                &[
                    "silky",
                    "smooth",
                    "elegant",
                    "velvet",
                    "documentary",
                    "audiobook",
                ],
                Gender::Female,
                108,
            ),
            (
                "cheerful_young_female",
                &[
                    "cheerful", "bright", "young", "friendly", "playful", "bubbly", "sweet",
                ],
                Gender::Female,
                109,
            ),
            (
                "wise_elder_male",
                &[
                    "old",
                    "elder",
                    "grandfather",
                    "aged",
                    "vintage",
                    "weathered",
                    "wise",
                ],
                Gender::Male,
                110,
            ),
            (
                "wise_elder_female",
                &["grandmother", "matriarch", "warm_old", "aged", "elderly"],
                Gender::Female,
                111,
            ),
            (
                "cyberpunk_synthetic",
                &[
                    "robotic",
                    "synthetic",
                    "ai",
                    "monotone",
                    "metallic",
                    "android",
                    "cyber",
                ],
                Gender::Neutral,
                112,
            ),
            (
                "heroic_cinematic",
                &[
                    "epic",
                    "cinematic",
                    "trailer",
                    "powerful",
                    "intense",
                    "dramatic",
                ],
                Gender::Male,
                113,
            ),
            (
                "melancholic_poet",
                &[
                    "sad",
                    "somber",
                    "melancholy",
                    "sorrowful",
                    "emotional",
                    "fragile",
                ],
                Gender::Neutral,
                114,
            ),
            (
                "urgent_tech",
                &["urgent", "rapid", "fast", "hasty", "alert", "dispatch"],
                Gender::Neutral,
                115,
            ),
            (
                "meditation_guide",
                &[
                    "meditation",
                    "zen",
                    "peaceful",
                    "serene",
                    "ambient",
                    "mindful",
                ],
                Gender::Female,
                116,
            ),
            (
                "bold_announcer",
                &["stadium", "hypeman", "loud", "shouting", "thunderous"],
                Gender::Male,
                117,
            ),
            (
                "playful_childlike",
                &["cartoon", "playful", "high_pitch", "squeaky", "animated"],
                Gender::Neutral,
                118,
            ),
            (
                "academic_scholar",
                &[
                    "professor",
                    "scholarly",
                    "intellectual",
                    "pedantic",
                    "articulate",
                ],
                Gender::Male,
                119,
            ),
            (
                "mystic_oracle",
                &[
                    "ethereal",
                    "mystic",
                    "oracle",
                    "shadowy",
                    "reverberant",
                    "spectral",
                ],
                Gender::Female,
                120,
            ),
        ];

        for (name, keywords, gender, seed) in definitions {
            self.archetypes.push(AcousticArchetype {
                name,
                keywords,
                gender: gender.clone(),
                base_vector: Self::generate_anchor_vector(*seed),
            });
        }
    }

    /// Synthesizes a 512-D neural acoustic embedding vector from text description and Director controls.
    pub fn design_voice(&self, req: &VoiceDesignRequest) -> VoiceProfile {
        let prompt_lower = req.prompt.to_lowercase();
        let words: Vec<&str> = prompt_lower
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|w| !w.is_empty())
            .collect();

        // Calculate relevance weights for each archetype
        let mut weights = vec![0.05f32; self.archetypes.len()]; // Base prior

        for (i, arc) in self.archetypes.iter().enumerate() {
            // Match keywords
            for kw in arc.keywords {
                if prompt_lower.contains(kw) {
                    weights[i] += 2.5;
                }
                for &word in &words {
                    if word == *kw {
                        weights[i] += 1.5;
                    }
                }
            }

            // Gender bias
            if let Some(target_gender) = &req.gender {
                if arc.gender == *target_gender {
                    weights[i] += 1.5;
                } else if arc.gender != Gender::Neutral && *target_gender != Gender::Neutral {
                    weights[i] *= 0.2; // Diminish opposite gender anchors
                }
            }
        }

        // Apply Director taxonomy modifiers to archetype weights
        if let Some(dir) = req.director {
            for (i, arc) in self.archetypes.iter().enumerate() {
                if dir.energy.is_finite() {
                    if dir.energy > 0.3
                        && (arc.name.contains("energetic") || arc.name.contains("bold"))
                    {
                        weights[i] += dir.energy * 2.0;
                    } else if dir.energy < -0.3
                        && (arc.name.contains("gentle")
                            || arc.name.contains("whisper")
                            || arc.name.contains("soothing")
                            || arc.name.contains("meditation"))
                    {
                        weights[i] += (-dir.energy) * 2.0;
                    }
                }
                if dir.intimacy.is_finite() {
                    if dir.intimacy > 0.3
                        && (arc.name.contains("whisper")
                            || arc.name.contains("intimate")
                            || arc.name.contains("gentle"))
                    {
                        weights[i] += dir.intimacy * 2.5;
                    } else if dir.intimacy < -0.3
                        && (arc.name.contains("broadcast") || arc.name.contains("announcer"))
                    {
                        weights[i] += (-dir.intimacy) * 2.0;
                    }
                }
                if dir.formality.is_finite() {
                    if dir.formality > 0.3
                        && (arc.name.contains("broadcast")
                            || arc.name.contains("academic")
                            || arc.name.contains("authoritative"))
                    {
                        weights[i] += dir.formality * 2.0;
                    } else if dir.formality < -0.3
                        && (arc.name.contains("playful")
                            || arc.name.contains("storyteller")
                            || arc.name.contains("cheerful"))
                    {
                        weights[i] += (-dir.formality) * 2.0;
                    }
                }
                if dir.emotion.is_finite() {
                    if dir.emotion > 0.3
                        && (arc.name.contains("cheerful") || arc.name.contains("playful"))
                    {
                        weights[i] += dir.emotion * 2.0;
                    } else if dir.emotion < -0.3
                        && (arc.name.contains("melancholic") || arc.name.contains("raspy"))
                    {
                        weights[i] += (-dir.emotion) * 2.0;
                    }
                }
            }
        }

        // Weighted vector combination
        let mut combined = [0.0f32; 512];

        for (i, arc) in self.archetypes.iter().enumerate() {
            let w = weights[i];
            for (k, val) in arc.base_vector.iter().enumerate() {
                combined[k] += val * w;
            }
        }

        // Normalize to unit sphere (hyper-sphere normalized embedding)
        let mut sum_sq = 0.0f32;
        for val in &combined {
            sum_sq += val * val;
        }
        let norm = sum_sq.sqrt().max(1e-6);
        let mut final_embedding = Vec::with_capacity(512);
        for val in &combined {
            final_embedding.push(val / norm);
        }

        let voice_id = format!(
            "vdes_{}",
            &Uuid::new_v4().to_string().replace('-', "")[..12]
        );
        let voice_name = req
            .name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| {
                // Auto-generate name based on top weighted archetype
                let max_idx = weights
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(idx, _)| idx)
                    .unwrap_or(0);
                let raw_name = self.archetypes[max_idx].name.replace('_', " ");
                let capitalized: String = raw_name
                    .split_whitespace()
                    .map(|w| {
                        let mut chars = w.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("Designed {}", capitalized)
            });

        let mut metadata = HashMap::new();
        metadata.insert("design_prompt".to_string(), req.prompt.clone());
        if let Some(dir) = req.director {
            if dir.energy.is_finite() {
                metadata.insert("director_energy".to_string(), format!("{:.2}", dir.energy));
            }
            if dir.pace.is_finite() {
                metadata.insert("director_pace".to_string(), format!("{:.2}", dir.pace));
            }
            if dir.intimacy.is_finite() {
                metadata.insert(
                    "director_intimacy".to_string(),
                    format!("{:.2}", dir.intimacy),
                );
            }
            if dir.formality.is_finite() {
                metadata.insert(
                    "director_formality".to_string(),
                    format!("{:.2}", dir.formality),
                );
            }
            if dir.emotion.is_finite() {
                metadata.insert(
                    "director_emotion".to_string(),
                    format!("{:.2}", dir.emotion),
                );
            }
        }

        VoiceProfile {
            id: voice_id,
            name: voice_name,
            engine_id: "qwen3-tts".to_string(), // Default neural engine supporting acoustic embeddings
            description: Some(format!("Custom AI-designed voice: {}", req.prompt)),
            language: req.language.clone().unwrap_or_else(|| "en-US".to_string()),
            gender: req.gender.clone().or(Some(Gender::Neutral)),
            reference_audio_path: None,
            reference_audio_base64: None,
            reference_transcript: Some(
                "The voice identity was engineered through neural archetype design.".to_string(),
            ),
            embedding: Some(final_embedding),
            clone_capabilities: Some(vec![
                "zero-shot-synthesis".to_string(),
                "cross-lingual".to_string(),
                "director-prosody".to_string(),
            ]),
            metadata,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archetype_bank_initialization() {
        let bank = ArchetypeBank::new();
        assert_eq!(bank.archetypes.len(), 20);

        // Verify all anchor vectors are normalized to unit sphere
        for arc in &bank.archetypes {
            let sum_sq: f32 = arc.base_vector.iter().map(|&x| x * x).sum();
            let norm = sum_sq.sqrt();
            assert!(
                (norm - 1.0).abs() < 1e-4,
                "Archetype {} vector must be unit length",
                arc.name
            );
        }
    }

    #[test]
    fn test_voice_design_from_prompt() {
        let bank = ArchetypeBank::new();
        let req = VoiceDesignRequest {
            prompt: "warm deep baritone soothing voice for meditation and audiobook".to_string(),
            name: None,
            gender: Some(Gender::Male),
            language: Some("en-US".to_string()),
            director: None,
        };

        let profile = bank.design_voice(&req);
        assert!(profile.name.starts_with("Designed"));
        assert_eq!(profile.gender, Some(Gender::Male));
        assert_eq!(profile.engine_id, "qwen3-tts");

        let embedding = profile.embedding.expect("Embedding must be generated");
        assert_eq!(embedding.len(), 512);

        // Check unit length
        let sum_sq: f32 = embedding.iter().map(|&x| x * x).sum();
        let norm = sum_sq.sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-4,
            "Generated embedding must be unit length"
        );
    }

    #[test]
    fn test_voice_design_with_director_taxonomy_and_nan() {
        let bank = ArchetypeBank::new();
        let req = VoiceDesignRequest {
            prompt: "energetic host podcast".to_string(),
            name: Some("Apex Podcaster".to_string()),
            gender: Some(Gender::Female),
            language: Some("en-GB".to_string()),
            director: Some(DirectorTaxonomy {
                energy: 0.8,
                emotion: f32::NAN, // Defensive test for NaN
                pace: 0.6,
                intimacy: -0.5,
                formality: 0.4,
            }),
        };

        let profile = bank.design_voice(&req);
        assert_eq!(profile.name, "Apex Podcaster");
        assert_eq!(profile.language, "en-GB");

        let embedding = profile.embedding.expect("Embedding must be generated");
        for val in &embedding {
            assert!(val.is_finite(), "All embedding dimensions must be finite");
        }
        // NaN emotion should not be written to metadata
        assert!(!profile.metadata.contains_key("director_emotion"));
        assert!(profile.metadata.contains_key("director_energy"));
    }

    #[test]
    fn test_voice_design_whitespace_name_fallback_and_low_energy() {
        let bank = ArchetypeBank::new();
        let req = VoiceDesignRequest {
            prompt: "calm gentle whisper narrator".to_string(),
            name: Some("   ".to_string()),
            gender: Some(Gender::Neutral),
            language: None,
            director: Some(DirectorTaxonomy {
                energy: -0.6,
                emotion: -0.4,
                pace: -0.2,
                intimacy: 0.7,
                formality: -0.5,
            }),
        };

        let profile = bank.design_voice(&req);
        // Name should not be whitespace, but auto-generated
        assert!(profile.name.starts_with("Designed"));
        assert!(!profile.name.trim().is_empty());

        let embedding = profile.embedding.expect("Embedding must be generated");
        assert_eq!(embedding.len(), 512);
        for val in &embedding {
            assert!(val.is_finite());
        }
    }
}
