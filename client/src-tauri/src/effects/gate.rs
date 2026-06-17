pub struct NoiseGate {
    pub enabled: bool,
    pub threshold_db: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    envelope: f32,
    gain: f32,
    sample_rate: f32,
}

impl Default for NoiseGate {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_db: -40.0,
            attack_ms: 5.0,
            release_ms: 200.0,
            envelope: 0.0,
            gain: 1.0,
            sample_rate: 48000.0,
        }
    }
}

impl NoiseGate {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }
        let threshold = db_to_lin(self.threshold_db);
        let att = time_coef(self.attack_ms, self.sample_rate);
        let rel = time_coef(self.release_ms, self.sample_rate);

        for s in samples.iter_mut() {
            let level = s.abs();
            self.envelope = if level > self.envelope {
                att * self.envelope + (1.0 - att) * level
            } else {
                rel * self.envelope + (1.0 - rel) * level
            };
            let target = if self.envelope >= threshold { 1.0f32 } else { 0.0 };
            self.gain = if target > self.gain {
                att * self.gain + (1.0 - att) * target
            } else {
                rel * self.gain + (1.0 - rel) * target
            };
            *s *= self.gain;
        }
    }
}

fn db_to_lin(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

// One-pole smoothing coefficient: larger ms = coefficient closer to 1.0 = slower
fn time_coef(ms: f32, sr: f32) -> f32 {
    if ms <= 0.0 || sr <= 0.0 {
        return 0.0;
    }
    (-1000.0 / (sr * ms)).exp()
}
