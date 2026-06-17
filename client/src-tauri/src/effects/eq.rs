use std::f32::consts::PI;

// Biquad direct form I:  y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2] - a1*y[n-1] - a2*y[n-2]
// Coefficients follow Audio EQ Cookbook by Robert Bristow-Johnson.
pub struct BiquadFilter {
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

impl Default for BiquadFilter {
    fn default() -> Self {
        // Identity passthrough
        Self { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0 }
    }
}

impl BiquadFilter {
    fn reset_state(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    // Low shelf, slope S=1 (maximally steep)
    pub fn low_shelf(gain_db: f32, freq: f32, sample_rate: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        // alpha = sin(w0)/2 * sqrt(2) comes from S=1 in the cookbook formula
        let alpha = w0.sin() / 2.0 * 2.0f32.sqrt();
        let sqrt_a = a.sqrt();

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha;

        Self {
            b0: b0 / a0, b1: b1 / a0, b2: b2 / a0,
            a1: a1 / a0, a2: a2 / a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }

    // High shelf, slope S=1
    pub fn high_shelf(gain_db: f32, freq: f32, sample_rate: f32) -> Self {
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / 2.0 * 2.0f32.sqrt();
        let sqrt_a = a.sqrt();

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * sqrt_a * alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
        let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * sqrt_a * alpha;

        Self {
            b0: b0 / a0, b1: b1 / a0, b2: b2 / a0,
            a1: a1 / a0, a2: a2 / a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }

    // Parametric peak/notch EQ
    pub fn peak_eq(gain_db: f32, freq: f32, q: f32, sample_rate: f32) -> Self {
        if gain_db.abs() < 0.01 {
            return Self::default();
        }
        let a = 10f32.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q.max(0.1));

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0, b1: b1 / a0, b2: b2 / a0,
            a1: a1 / a0, a2: a2 / a0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
        }
    }

    #[inline]
    pub fn process_sample(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
              - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

pub struct Eq3Band {
    pub enabled: bool,
    pub low_gain_db: f32,
    pub mid_gain_db: f32,
    pub mid_freq: f32,
    pub high_gain_db: f32,
    sample_rate: f32,
    low_filter: BiquadFilter,
    mid_filter: BiquadFilter,
    high_filter: BiquadFilter,
}

impl Default for Eq3Band {
    fn default() -> Self {
        let mut eq = Self {
            enabled: false,
            low_gain_db: 0.0,
            mid_gain_db: 0.0,
            mid_freq: 1000.0,
            high_gain_db: 0.0,
            sample_rate: 48000.0,
            low_filter: BiquadFilter::default(),
            mid_filter: BiquadFilter::default(),
            high_filter: BiquadFilter::default(),
        };
        eq.update_coefficients();
        eq
    }
}

impl Eq3Band {
    // Call after changing any param or sample_rate
    pub fn update_coefficients(&mut self) {
        self.low_filter = BiquadFilter::low_shelf(self.low_gain_db, 200.0, self.sample_rate);
        self.mid_filter = BiquadFilter::peak_eq(self.mid_gain_db, self.mid_freq, 1.0, self.sample_rate);
        self.high_filter = BiquadFilter::high_shelf(self.high_gain_db, 8000.0, self.sample_rate);
        // Reset state to avoid transients when coefficients change
        self.low_filter.reset_state();
        self.mid_filter.reset_state();
        self.high_filter.reset_state();
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
        self.update_coefficients();
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }
        for s in samples.iter_mut() {
            *s = self.low_filter.process_sample(*s);
            *s = self.mid_filter.process_sample(*s);
            *s = self.high_filter.process_sample(*s);
        }
    }
}
