mod compressor;
mod eq;
mod gate;

pub use compressor::Compressor;
pub use eq::Eq3Band;
pub use gate::NoiseGate;

use serde::Serialize;
use std::sync::{Arc, Mutex, OnceLock};

pub struct EffectChain {
    pub gate: NoiseGate,
    pub compressor: Compressor,
    pub eq: Eq3Band,
}

impl Default for EffectChain {
    fn default() -> Self {
        Self {
            gate: NoiseGate::default(),
            compressor: Compressor::default(),
            eq: Eq3Band::default(),
        }
    }
}

impl EffectChain {
    pub fn process(&mut self, samples: &mut [f32]) {
        self.gate.process(samples);
        self.compressor.process(samples);
        self.eq.process(samples);
    }

    pub fn set_sample_rate(&mut self, sr: f32) {
        self.gate.set_sample_rate(sr);
        self.compressor.set_sample_rate(sr);
        self.eq.set_sample_rate(sr);
    }
}

static EFFECT_CHAIN: OnceLock<Arc<Mutex<EffectChain>>> = OnceLock::new();

pub fn get_chain() -> Arc<Mutex<EffectChain>> {
    EFFECT_CHAIN.get_or_init(|| Arc::new(Mutex::new(EffectChain::default()))).clone()
}

// ─── DTOs ───────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct EffectsStateDto {
    pub gate_enabled: bool,
    pub gate_threshold_db: f32,
    pub gate_attack_ms: f32,
    pub gate_release_ms: f32,
    pub comp_enabled: bool,
    pub comp_threshold_db: f32,
    pub comp_ratio: f32,
    pub comp_attack_ms: f32,
    pub comp_release_ms: f32,
    pub comp_makeup_db: f32,
    pub eq_enabled: bool,
    pub eq_low_db: f32,
    pub eq_mid_db: f32,
    pub eq_mid_freq: f32,
    pub eq_high_db: f32,
}

// ─── Tauri commands ─────────────────────────────────────────────────────────

#[tauri::command]
pub fn set_noise_gate(enabled: bool, threshold_db: f32, attack_ms: f32, release_ms: f32) {
    let arc = get_chain();
    let mut c = arc.lock().unwrap();
    c.gate.enabled = enabled;
    c.gate.threshold_db = threshold_db;
    c.gate.attack_ms = attack_ms;
    c.gate.release_ms = release_ms;
}

#[tauri::command]
pub fn set_compressor(
    enabled: bool,
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    makeup_gain_db: f32,
) {
    let arc = get_chain();
    let mut c = arc.lock().unwrap();
    c.compressor.enabled = enabled;
    c.compressor.threshold_db = threshold_db;
    c.compressor.ratio = ratio;
    c.compressor.attack_ms = attack_ms;
    c.compressor.release_ms = release_ms;
    c.compressor.makeup_gain_db = makeup_gain_db;
}

#[tauri::command]
pub fn set_eq(enabled: bool, low_gain_db: f32, mid_gain_db: f32, mid_freq: f32, high_gain_db: f32) {
    let arc = get_chain();
    let mut c = arc.lock().unwrap();
    c.eq.enabled = enabled;
    c.eq.low_gain_db = low_gain_db;
    c.eq.mid_gain_db = mid_gain_db;
    c.eq.mid_freq = mid_freq.clamp(200.0, 6000.0);
    c.eq.high_gain_db = high_gain_db;
    c.eq.update_coefficients();
}

#[tauri::command]
pub fn get_effects_state() -> EffectsStateDto {
    let arc = get_chain();
    let c = arc.lock().unwrap();
    EffectsStateDto {
        gate_enabled: c.gate.enabled,
        gate_threshold_db: c.gate.threshold_db,
        gate_attack_ms: c.gate.attack_ms,
        gate_release_ms: c.gate.release_ms,
        comp_enabled: c.compressor.enabled,
        comp_threshold_db: c.compressor.threshold_db,
        comp_ratio: c.compressor.ratio,
        comp_attack_ms: c.compressor.attack_ms,
        comp_release_ms: c.compressor.release_ms,
        comp_makeup_db: c.compressor.makeup_gain_db,
        eq_enabled: c.eq.enabled,
        eq_low_db: c.eq.low_gain_db,
        eq_mid_db: c.eq.mid_gain_db,
        eq_mid_freq: c.eq.mid_freq,
        eq_high_db: c.eq.high_gain_db,
    }
}
