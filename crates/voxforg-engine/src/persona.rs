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
