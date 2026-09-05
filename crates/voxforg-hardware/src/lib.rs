pub mod probe;
pub mod profile;

pub use probe::HardwareProbe;
pub use profile::{GpuAccelerator, HardwareProfile, HardwareTier, SimdSupport};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_probe_execution() {
        let profile = HardwareProbe::probe();
        assert!(!profile.os.is_empty());
        assert!(!profile.arch.is_empty());
        assert!(profile.cpu_physical_cores >= 1);
        assert!(profile.cpu_logical_threads >= 1);
        assert!(profile.total_memory_mb > 0);
        assert!(!profile.recommended_engines.is_empty());
    }

    #[test]
    fn test_hardware_tier_ordering() {
        assert!(HardwareTier::Tier1Minimal < HardwareTier::Tier2Standard);
        assert!(HardwareTier::Tier2Standard < HardwareTier::Tier3Pro);
        assert!(HardwareTier::Tier3Pro < HardwareTier::Tier4Enterprise);
    }

    #[test]
    fn test_hardware_tier_display() {
        assert_eq!(HardwareTier::Tier1Minimal.to_string(), "TIER_1_MINIMAL");
        assert_eq!(HardwareTier::Tier2Standard.to_string(), "TIER_2_STANDARD");
        assert_eq!(HardwareTier::Tier3Pro.to_string(), "TIER_3_PRO");
        assert_eq!(HardwareTier::Tier4Enterprise.to_string(), "TIER_4_ENTERPRISE");
    }

    #[test]
    fn test_profile_serialization_roundtrip() {
        let profile = HardwareProfile {
            os: "Linux".to_string(),
            arch: "x86_64".to_string(),
            cpu_brand: "AMD Ryzen".to_string(),
            cpu_physical_cores: 8,
            cpu_logical_threads: 16,
            total_memory_mb: 32768,
            available_memory_mb: 24000,
            simd: SimdSupport {
                avx: true,
                avx2: true,
                avx512: false,
                neon: false,
            },
            gpus: vec![GpuAccelerator {
                name: "NVIDIA RTX 4090".to_string(),
                vram_mb: 24576,
                has_cuda: true,
                has_mps: false,
                has_directml: false,
            }],
            assigned_tier: HardwareTier::Tier4Enterprise,
            recommended_engines: vec!["qwen3-tts".to_string()],
        };

        let json = serde_json::to_string(&profile).expect("Serialization failed");
        let decoded: HardwareProfile = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(decoded.os, "Linux");
        assert_eq!(decoded.assigned_tier, HardwareTier::Tier4Enterprise);
        assert_eq!(decoded.gpus.len(), 1);
        assert!(decoded.gpus[0].has_cuda);
    }
}
