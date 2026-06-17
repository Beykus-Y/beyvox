pub struct Compressor {
    pub enabled: bool,
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_gain_db: f32,
    envelope_db: f32,
    sample_rate: f32,
}

impl Default for Compressor {
    fn default() -> Self {
        Self {
            enabled: false,
            threshold_db: -18.0,
            ratio: 4.0,
            attack_ms: 5.0,
            release_ms: 100.0,
            makeup_gain_db: 0.0,
            envelope_db: -120.0,
            sample_rate: 48000.0,
        }
    }
}

impl Compressor {
    pub fn set_sample_rate(&mut self, sr: f32) {
        self.sample_rate = sr;
    }

    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled {
            return;
        }
        let att = time_coef(self.attack_ms, self.sample_rate);
        let rel = time_coef(self.release_ms, self.sample_rate);
        let makeup = db_to_lin(self.makeup_gain_db);
        let ratio = self.ratio.max(1.0);

        for s in samples.iter_mut() {
            let level_db = 20.0 * s.abs().max(1e-10_f32).log10();
            self.envelope_db = if level_db > self.envelope_db {
                att * self.envelope_db + (1.0 - att) * level_db
            } else {
                rel * self.envelope_db + (1.0 - rel) * level_db
            };
            let gain_reduction_db = if self.envelope_db > self.threshold_db {
                (self.threshold_db - self.envelope_db) * (1.0 - 1.0 / ratio)
            } else {
                0.0
            };
            *s = (*s * db_to_lin(gain_reduction_db) * makeup).clamp(-1.0, 1.0);
        }
    }
}

fn db_to_lin(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

fn time_coef(ms: f32, sr: f32) -> f32 {
    if ms <= 0.0 || sr <= 0.0 {
        return 0.0;
    }
    (-1000.0 / (sr * ms)).exp()
}
