//! Portable Persona Bundles (`.voxpersona`).
//!
//! Encapsulates complete vocal identity assets:
//! 1. Speaker metadata and 512-D neural acoustic embedding vectors.
//! 2. Reference WAV audio samples for zero-shot conditioning.
//! 3. Default Director AI prosody baseline parameters.
//! 4. Optional visual avatar icon / picture.

use serde::{Deserialize, Serialize};
use voxforg_audio::dsp::DirectorTaxonomy;
use voxforg_core::models::VoiceProfile;

/// Self-contained portable voice persona bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaBundle {
    pub format_version: u32,
    pub profile: VoiceProfile,
    #[serde(default)]
    pub prosody_defaults: DirectorTaxonomy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_base64: Option<String>,
}

impl PersonaBundle {
    /// Creates a persona bundle from an existing voice profile.
    pub fn from_profile(profile: VoiceProfile) -> Self {
        let energy = profile
            .metadata
            .get("director_energy")
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.0);
        let pace = profile
            .metadata
            .get("director_pace")
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.0);
        let intimacy = profile
            .metadata
            .get("director_intimacy")
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.0);
        let formality = profile
            .metadata
            .get("director_formality")
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.0);
        let emotion = profile
            .metadata
            .get("director_emotion")
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.0);

        Self {
            format_version: 1,
            profile,
            prosody_defaults: DirectorTaxonomy {
                energy,
                emotion,
                pace,
                intimacy,
                formality,
            },
            avatar_base64: None,
        }
    }

    /// Serializes bundle to UTF-8 JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parses bundle from UTF-8 JSON.
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    /// Unpacks bundle directly into a registered VoiceProfile.
    pub fn unpack_profile(self) -> VoiceProfile {
        self.profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_persona_from_profile_with_defaults() {
        let profile = VoiceProfile {
            id: "vp_test".to_string(),
            name: "Test Speaker".to_string(),
            engine_id: "qwen3".to_string(),
            description: None,
            language: "en-US".to_string(),
            gender: None,
            reference_audio_path: None,
            reference_audio_base64: None,
            reference_transcript: None,
            embedding: Some(vec![0.1; 512]),
            clone_capabilities: None,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        };

        let bundle = PersonaBundle::from_profile(profile.clone());
        assert_eq!(bundle.format_version, 1);
        assert_eq!(bundle.prosody_defaults.energy, 0.0);
        assert_eq!(bundle.prosody_defaults.pace, 0.0);
        assert_eq!(bundle.profile.id, "vp_test");
        assert_eq!(bundle.unpack_profile().id, "vp_test");
    }

    #[test]
    fn test_persona_from_profile_with_director_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("director_energy".to_string(), "0.75".to_string());
        metadata.insert("director_pace".to_string(), "1.2".to_string());
        metadata.insert("director_emotion".to_string(), "0.5".to_string());
        metadata.insert("director_intimacy".to_string(), "0.3".to_string());
        metadata.insert("director_formality".to_string(), "-0.2".to_string());

        let profile = VoiceProfile {
            id: "vp_custom".to_string(),
            name: "Custom Persona".to_string(),
            engine_id: "qwen3".to_string(),
            description: Some("Custom voice".to_string()),
            language: "en-US".to_string(),
            gender: None,
            reference_audio_path: None,
            reference_audio_base64: Some("data:audio/wav;base64,UklGRg==".to_string()),
            reference_transcript: None,
            embedding: Some(vec![0.2; 512]),
            clone_capabilities: None,
            metadata,
            created_at: chrono::Utc::now(),
        };

        let bundle = PersonaBundle::from_profile(profile);
        assert_eq!(bundle.prosody_defaults.energy, 0.75);
        assert_eq!(bundle.prosody_defaults.pace, 1.2);
        assert_eq!(bundle.prosody_defaults.emotion, 0.5);
        assert_eq!(bundle.prosody_defaults.intimacy, 0.3);
        assert_eq!(bundle.prosody_defaults.formality, -0.2);

        // Serialization roundtrip
        let json = bundle.to_json().expect("Serialization failed");
        let parsed = PersonaBundle::from_json(&json).expect("Deserialization failed");
        assert_eq!(parsed.profile.id, "vp_custom");
        assert_eq!(parsed.prosody_defaults.energy, 0.75);
    }
}


