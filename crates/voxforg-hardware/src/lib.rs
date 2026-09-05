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
}
