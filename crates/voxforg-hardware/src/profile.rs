use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HardwareTier {
    Tier1Minimal,
    Tier2Standard,
    Tier3Pro,
    Tier4Enterprise,
}

impl std::fmt::Display for HardwareTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tier1Minimal => write!(f, "TIER_1_MINIMAL"),
            Self::Tier2Standard => write!(f, "TIER_2_STANDARD"),
            Self::Tier3Pro => write!(f, "TIER_3_PRO"),
            Self::Tier4Enterprise => write!(f, "TIER_4_ENTERPRISE"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimdSupport {
    pub avx: bool,
    pub avx2: bool,
    pub avx512: bool,
    pub neon: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuAccelerator {
    pub name: String,
    pub vram_mb: u64,
    pub has_cuda: bool,
    pub has_mps: bool,
    pub has_directml: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub os: String,
    pub arch: String,
    pub cpu_brand: String,
    pub cpu_physical_cores: usize,
    pub cpu_logical_threads: usize,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub simd: SimdSupport,
    pub gpus: Vec<GpuAccelerator>,
    pub assigned_tier: HardwareTier,
    pub recommended_engines: Vec<String>,
}
