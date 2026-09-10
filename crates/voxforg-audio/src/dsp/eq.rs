use std::f32::consts::PI;

#[derive(Debug, Clone)]
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

impl Biquad {
    fn new() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    fn low_shelf(sample_rate: f32, cutoff_hz: f32, gain_db: f32) -> Self {
        let mut b = Self::new();
        if gain_db.abs() < 0.01 {
            return b;
        }

        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * (cutoff_hz / sample_rate);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / 2.0 * 2.0_f32.sqrt();

        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;
        b.b0 = (a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha)) / a0;
        b.b1 = (2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0)) / a0;
        b.b2 = (a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha)) / a0;
        b.a1 = (-2.0 * ((a - 1.0) + (a + 1.0) * cos_w0)) / a0;
        b.a2 = ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha) / a0;
        b
    }

    fn peaking(sample_rate: f32, center_hz: f32, gain_db: f32, q: f32) -> Self {
        let mut b = Self::new();
        if gain_db.abs() < 0.01 {
            return b;
        }

        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * (center_hz / sample_rate);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q.max(0.1));

        let a0 = 1.0 + alpha / a;
        b.b0 = (1.0 + alpha * a) / a0;
        b.b1 = (-2.0 * cos_w0) / a0;
        b.b2 = (1.0 - alpha * a) / a0;
        b.a1 = (-2.0 * cos_w0) / a0;
        b.a2 = (1.0 - alpha / a) / a0;
        b
    }

    fn high_shelf(sample_rate: f32, cutoff_hz: f32, gain_db: f32) -> Self {
        let mut b = Self::new();
        if gain_db.abs() < 0.01 {
            return b;
        }

        let a = 10.0f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * (cutoff_hz / sample_rate);
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / 2.0 * 2.0_f32.sqrt();

        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;
        b.b0 = (a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha)) / a0;
        b.b1 = (-2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0)) / a0;
        b.b2 = (a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha)) / a0;
        b.a1 = (2.0 * ((a - 1.0) - (a + 1.0) * cos_w0)) / a0;
        b.a2 = ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha) / a0;
        b
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = if output.is_finite() { output } else { 0.0 };

        self.y1
    }
}

pub struct ParametricEq;

impl ParametricEq {
    pub fn process_3band(
        samples: &mut [i16],
        sample_rate: u32,
        low_gain_db: f32,
        mid_gain_db: f32,
        high_gain_db: f32,
    ) {
        if samples.is_empty() {
            return;
        }

        let sr = (sample_rate.max(8000)) as f32;
        let safe_low = if low_gain_db.is_finite() {
            low_gain_db.clamp(-24.0, 24.0)
        } else {
            0.0
        };
        let safe_mid = if mid_gain_db.is_finite() {
            mid_gain_db.clamp(-24.0, 24.0)
        } else {
            0.0
        };
        let safe_high = if high_gain_db.is_finite() {
            high_gain_db.clamp(-24.0, 24.0)
        } else {
            0.0
        };

        if safe_low.abs() < 0.01 && safe_mid.abs() < 0.01 && safe_high.abs() < 0.01 {
            return;
        }

        // Low shelf at 250Hz, Peaking mid at 1000Hz (Q=1.0), High shelf at 4000Hz
        let mut low_filter = Biquad::low_shelf(sr, 250.0, safe_low);
        let mut mid_filter = Biquad::peaking(sr, 1000.0, safe_mid, 1.0);
        let mut high_filter = Biquad::high_shelf(sr, 4000.0, safe_high);

        for sample in samples.iter_mut() {
            let in_val = *sample as f32;
            let out_low = low_filter.process(in_val);
            let out_mid = mid_filter.process(out_low);
            let out_high = high_filter.process(out_mid);

            if out_high.is_finite() {
                *sample = out_high.round().clamp(-32768.0, 32767.0) as i16;
            }
        }
    }
}
