use crate::profile::{GpuAccelerator, HardwareProfile, HardwareTier, SimdSupport};
use sysinfo::System;

pub struct HardwareProbe;

impl HardwareProbe {
    pub fn probe() -> HardwareProfile {
        let mut sys = System::new_all();
        sys.refresh_all();

        let os = System::name().unwrap_or_else(|| std::env::consts::OS.to_string());
        let arch = std::env::consts::ARCH.to_string();

        let cpus = sys.cpus();
        let cpu_brand = cpus
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let cpu_physical_cores = sys.physical_core_count().unwrap_or(1);
        let cpu_logical_threads = cpus.len();

        let total_memory_mb = sys.total_memory() / (1024 * 1024);
        let available_memory_mb = sys.available_memory() / (1024 * 1024);

        let simd = Self::detect_simd();
        let gpus = Self::detect_gpus(&os);
        let assigned_tier = Self::determine_tier(total_memory_mb, cpu_physical_cores, &simd, &gpus);
        let recommended_engines = Self::recommend_engines(assigned_tier);

        HardwareProfile {
            os,
            arch,
            cpu_brand,
            cpu_physical_cores,
            cpu_logical_threads,
            total_memory_mb,
            available_memory_mb,
            simd,
            gpus,
            assigned_tier,
            recommended_engines,
        }
    }

    fn detect_simd() -> SimdSupport {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            SimdSupport {
                avx: is_x86_feature_detected!("avx"),
                avx2: is_x86_feature_detected!("avx2"),
                avx512: is_x86_feature_detected!("avx512f"),
                neon: false,
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            SimdSupport {
                avx: false,
                avx2: false,
                avx512: false,
                neon: true,
            }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        {
            SimdSupport {
                avx: false,
                avx2: false,
                avx512: false,
                neon: false,
            }
        }
    }

    fn detect_gpus(_os: &str) -> Vec<GpuAccelerator> {
        let mut gpus = Vec::new();

        // Check for CUDA via environment or nvidia-smi presence
        if let Ok(path) = std::env::var("CUDA_PATH") {
            gpus.push(GpuAccelerator {
                name: format!("NVIDIA CUDA Runtime ({})", path),
                vram_mb: 8192,
                has_cuda: true,
                has_mps: false,
                has_directml: false,
            });
        }

        #[cfg(target_os = "macos")]
        if std::env::consts::ARCH == "aarch64" {
            gpus.push(GpuAccelerator {
                name: "Apple Silicon Neural Engine / Metal".to_string(),
                vram_mb: 16384,
                has_cuda: false,
                has_mps: true,
                has_directml: false,
            });
        }

        #[cfg(target_os = "windows")]
        if gpus.is_empty() {
            // DirectML fallback capability on Windows
            gpus.push(GpuAccelerator {
                name: "DirectML GPU Hardware Adapter".to_string(),
                vram_mb: 4096,
                has_cuda: false,
                has_mps: false,
                has_directml: true,
            });
        }

        gpus
    }

    fn determine_tier(
        total_ram_mb: u64,
        _cores: usize,
        simd: &SimdSupport,
        gpus: &[GpuAccelerator],
    ) -> HardwareTier {
        let has_cuda_or_mps = gpus.iter().any(|g| g.has_cuda || g.has_mps);
        let max_vram = gpus.iter().map(|g| g.vram_mb).max().unwrap_or(0);

        if has_cuda_or_mps && max_vram >= 8192 && total_ram_mb >= 16384 {
            HardwareTier::Tier4Enterprise
        } else if (has_cuda_or_mps && max_vram >= 4096) || total_ram_mb >= 12288 {
            HardwareTier::Tier3Pro
        } else if simd.avx2 || simd.neon || total_ram_mb >= 6144 {
            HardwareTier::Tier2Standard
        } else {
            HardwareTier::Tier1Minimal
        }
    }

    fn recommend_engines(tier: HardwareTier) -> Vec<String> {
        match tier {
            HardwareTier::Tier4Enterprise => vec![
                "qwen3-tts".to_string(),
                "kokoro-82m-cuda".to_string(),
                "piper-onnx".to_string(),
                "edge-tts".to_string(),
            ],
            HardwareTier::Tier3Pro => vec![
                "kokoro-82m-gpu".to_string(),
                "kittentts".to_string(),
                "piper-onnx".to_string(),
                "edge-tts".to_string(),
            ],
            HardwareTier::Tier2Standard => vec![
                "piper-onnx".to_string(),
                "kokoro-82m-cpu".to_string(),
                "edge-tts".to_string(),
            ],
            HardwareTier::Tier1Minimal => vec!["edge-tts".to_string(), "piper-low-cpu".to_string()],
        }
    }
}
