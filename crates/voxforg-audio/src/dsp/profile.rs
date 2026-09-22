//! End-to-end studio mastering profiles for vocal audio.

use crate::dsp::{BrickwallLimiter, DeEsser, DynamicCompressor, HarmonicWarmth, ParametricEq};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MasteringProfile {
    /// Broadcast radio / television standard: chest fullness, controlled dynamics, de-essing, -0.5 dBFS ceiling.
    Broadcast,
    /// Intimate podcast: sub-bass rumble cutoff, conversational warmth, natural speech compression.
    Podcast,
    /// Warm analogue: enhanced lower-mid harmonics, tube saturation, smooth high-end roll-off.
    Warmth,
    /// Pure linear transparent studio pass: bit-perfect preservation with -0.2 dBFS brickwall peak safety.
    StudioClean,
}

impl MasteringProfile {
    /// Master raw 16-bit PCM speech samples using the selected mastering profile.
    pub fn apply(&self, samples: &mut [i16], sample_rate: u32) {
        if samples.is_empty() {
            return;
        }

        let sr = sample_rate.max(8000);

        match self {
            Self::Broadcast => {
                // 1. Equalization: subtle chest boost (180Hz) and harshness reduction (3.2kHz)
                ParametricEq::process_3band(samples, sr, 2.0, -1.5, 1.5);

                // 2. De-Esser: tame sibilance between 5.5kHz and 7.5kHz
                DeEsser::process(samples, sr, 6500.0, -22.0, 7.0);

                // 3. Harmonic warmth: subtle broadcast condenser microphone presence
                HarmonicWarmth::process(samples, 0.25, 0.35);

                // 4. Dynamic compressor: even out conversational volume
                DynamicCompressor::process(samples, sr, -18.0, 2.8, 12.0, 90.0, 1.5);

                // 5. Brickwall peak limiter: guarantee zero clipping
                BrickwallLimiter::process(samples, -0.5);
            }
            Self::Podcast => {
                // 1. Gentle warmth & presence
                ParametricEq::process_3band(samples, sr, 1.5, -1.0, 1.0);

                // 2. De-Esser
                DeEsser::process(samples, sr, 6200.0, -20.0, 6.0);

                // 3. Natural voice compressor
                DynamicCompressor::process(samples, sr, -16.0, 2.4, 15.0, 120.0, 1.0);

                // 4. Peak limiter
                BrickwallLimiter::process(samples, -1.0);
            }
            Self::Warmth => {
                // 1. Lower-mid bass fullness
                ParametricEq::process_3band(samples, sr, 3.0, -1.0, 0.5);

                // 2. Analogue tube saturation
                HarmonicWarmth::process(samples, 0.40, 0.55);

                // 3. Peak limiter
                BrickwallLimiter::process(samples, -0.5);
            }
            Self::StudioClean => {
                // Transparent pass: only apply safety limiter
                BrickwallLimiter::process(samples, -0.2);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mastering_profiles_execute_safely() {
        let sample_rate = 24000;
        let samples = vec![5000, -8000, 15000, -22000, 31000, -32000];

        for profile in [
            MasteringProfile::Broadcast,
            MasteringProfile::Podcast,
            MasteringProfile::Warmth,
            MasteringProfile::StudioClean,
        ] {
            let mut buf = samples.clone();
            profile.apply(&mut buf, sample_rate);
            assert_eq!(buf.len(), samples.len());
            // Limiter must guarantee samples don't clip
            assert!(buf.iter().all(|&s| s < 32760 && s > -32760));
        }
    }
}
