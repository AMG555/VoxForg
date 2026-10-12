use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voice {
    pub id: String,
    pub name: String,
    pub engine_id: String,
    pub language: String,
    pub gender: Gender,
    pub sample_rate_hz: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub is_local: bool,
    pub default_sample_rate_hz: u32,
    pub supported_formats: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum VoiceQuality {
    #[default]
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoiceEngineMapping {
    pub engine_id: String,
    pub voice_id: String,
    #[serde(default)]
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceIdentity {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub quality: VoiceQuality,
    #[serde(default)]
    pub style: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gender: Option<Gender>,
    #[serde(default)]
    pub engine_mappings: Vec<VoiceEngineMapping>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gender_serde_roundtrip() {
        assert_eq!(serde_json::to_string(&Gender::Male).unwrap(), "\"male\"");
        assert_eq!(serde_json::to_string(&Gender::Female).unwrap(), "\"female\"");
        assert_eq!(serde_json::to_string(&Gender::Neutral).unwrap(), "\"neutral\"");

        assert_eq!(serde_json::from_str::<Gender>("\"male\"").unwrap(), Gender::Male);
        assert_eq!(serde_json::from_str::<Gender>("\"female\"").unwrap(), Gender::Female);
        assert_eq!(serde_json::from_str::<Gender>("\"neutral\"").unwrap(), Gender::Neutral);
    }

    #[test]
    fn test_voice_quality_default_and_serde() {
        assert_eq!(VoiceQuality::default(), VoiceQuality::High);
        assert_eq!(serde_json::to_string(&VoiceQuality::High).unwrap(), "\"high\"");
        assert_eq!(serde_json::to_string(&VoiceQuality::Medium).unwrap(), "\"medium\"");
        assert_eq!(serde_json::to_string(&VoiceQuality::Low).unwrap(), "\"low\"");
    }

    #[test]
    fn test_voice_identity_serialization() {
        let identity = VoiceIdentity {
            id: "vid_drake".to_string(),
            display_name: "Julian Drake".to_string(),
            quality: VoiceQuality::High,
            style: "conversational".to_string(),
            accent: Some("american".to_string()),
            gender: Some(Gender::Male),
            engine_mappings: vec![VoiceEngineMapping {
                engine_id: "edge-tts".to_string(),
                voice_id: "en-US-GuyNeural".to_string(),
                priority: 1,
            }],
        };

        let json_str = serde_json::to_string(&identity).unwrap();
        let decoded: VoiceIdentity = serde_json::from_str(&json_str).unwrap();
        assert_eq!(identity, decoded);
    }
}
