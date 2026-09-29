pub struct AudioMerger;

impl AudioMerger {
    pub fn concatenate_with_pause(
        segments: &[&[i16]],
        sample_rate: u32,
        pause_duration_ms: u32,
    ) -> Vec<i16> {
        // Bound pause duration to maximum 30 seconds to prevent OOM
        let safe_pause_ms = pause_duration_ms.min(30_000);
        let safe_sample_rate = sample_rate.min(192_000);
        let pause_samples_count =
            ((safe_sample_rate as f32) * (safe_pause_ms as f32 / 1000.0)) as usize;

        let total_len: usize = segments.iter().map(|s| s.len()).sum::<usize>()
            + (segments.len().saturating_sub(1) * pause_samples_count);

        let mut output = Vec::with_capacity(total_len);

        for (idx, segment) in segments.iter().enumerate() {
            output.extend_from_slice(segment);
            if idx + 1 < segments.len() && pause_samples_count > 0 {
                output.resize(output.len() + pause_samples_count, 0i16);
            }
        }

        output
    }

    pub fn crossfade(
        segment_a: &[i16],
        segment_b: &[i16],
        sample_rate: u32,
        crossfade_ms: u32,
    ) -> Vec<i16> {
        // Bound crossfade duration to maximum 10 seconds to prevent OOM
        let safe_crossfade_ms = crossfade_ms.min(10_000);
        let safe_sample_rate = sample_rate.min(192_000);
        let fade_len = ((safe_sample_rate as f32) * (safe_crossfade_ms as f32 / 1000.0)) as usize;
        if segment_a.len() < fade_len || segment_b.len() < fade_len || fade_len == 0 {
            let mut out = Vec::with_capacity(segment_a.len() + segment_b.len());
            out.extend_from_slice(segment_a);
            out.extend_from_slice(segment_b);
            return out;
        }

        let a_body = &segment_a[..segment_a.len() - fade_len];
        let a_tail = &segment_a[segment_a.len() - fade_len..];
        let b_head = &segment_b[..fade_len];
        let b_body = &segment_b[fade_len..];

        let mut out = Vec::with_capacity(segment_a.len() + segment_b.len() - fade_len);
        out.extend_from_slice(a_body);

        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            let sample_a = a_tail[i] as f32 * (1.0 - t);
            let sample_b = b_head[i] as f32 * t;
            let blended = (sample_a + sample_b).round().clamp(-32768.0, 32767.0) as i16;
            out.push(blended);
        }

        out.extend_from_slice(b_body);
        out
    }

    /// High-fidelity equal-power crossfade between two audio segments.
    /// Uses quarter-sine/cosine curves (cos^2 + sin^2 = 1) to eliminate
    /// center-dip acoustic attenuation and clicks at chunk seams.
    pub fn equal_power_crossfade(
        segment_a: &[i16],
        segment_b: &[i16],
        sample_rate: u32,
        crossfade_ms: u32,
    ) -> Vec<i16> {
        let safe_crossfade_ms = crossfade_ms.clamp(1, 10_000);
        let safe_sample_rate = sample_rate.min(192_000);
        let fade_len = ((safe_sample_rate as f32) * (safe_crossfade_ms as f32 / 1000.0)) as usize;

        if segment_a.len() < fade_len || segment_b.len() < fade_len || fade_len == 0 {
            let mut out = Vec::with_capacity(segment_a.len() + segment_b.len());
            out.extend_from_slice(segment_a);
            out.extend_from_slice(segment_b);
            return out;
        }

        let a_body = &segment_a[..segment_a.len() - fade_len];
        let a_tail = &segment_a[segment_a.len() - fade_len..];
        let b_head = &segment_b[..fade_len];
        let b_body = &segment_b[fade_len..];

        let mut out = Vec::with_capacity(segment_a.len() + segment_b.len() - fade_len);
        out.extend_from_slice(a_body);

        let half_pi = std::f32::consts::FRAC_PI_2;
        for i in 0..fade_len {
            let t = i as f32 / fade_len as f32;
            let angle = t * half_pi;
            let gain_a = angle.cos();
            let gain_b = angle.sin();

            let sample_a = a_tail[i] as f32 * gain_a;
            let sample_b = b_head[i] as f32 * gain_b;
            let blended = (sample_a + sample_b).round().clamp(-32768.0, 32767.0) as i16;
            out.push(blended);
        }

        out.extend_from_slice(b_body);
        out
    }

    /// Concatenate an array of audio chunks using equal-power crossfading at every seam.
    pub fn concatenate_with_equal_power_crossfade(
        chunks: &[&[i16]],
        sample_rate: u32,
        crossfade_ms: u32,
    ) -> Vec<i16> {
        if chunks.is_empty() {
            return Vec::new();
        }
        if chunks.len() == 1 {
            return chunks[0].to_vec();
        }

        let mut accumulated = chunks[0].to_vec();
        for &next_chunk in &chunks[1..] {
            accumulated =
                Self::equal_power_crossfade(&accumulated, next_chunk, sample_rate, crossfade_ms);
        }
        accumulated
    }
}
