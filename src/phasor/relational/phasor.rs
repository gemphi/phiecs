#[allow(unused_imports)]
use super::*;

impl RelationalPhasor {
    /// A word's starting position: hash-seeded angles, zero evidence.
    ///
    /// Magnitudes start at zero rather than one. A word has no claim to any
    /// relation until it is seen in one, and starting every word with full
    /// magnitude everywhere would reproduce the flat representation this module
    /// exists to replace.
    pub fn seeded(word: &str) -> Self {
        // The one seed (docs/159), shared with every other phasor.
        Self { theta: phistd::seed_angles(word, PHASE_CHANNELS), mag: vec![0.0; PHASE_CHANNELS] }
    }

    /// Normalised complex inner product, magnitude included.
    ///
    /// A channel where either word has no evidence contributes nothing, so two
    /// words are compared only on the relations they both participate in. That
    /// is the behaviour the shipped `resonance` cannot have: averaging 64
    /// cosines gives a channel the same vote whether the word was ever seen
    /// there or not.
    pub fn similarity(&self, other: &Self) -> f64 {
        let (mut dot, mut na, mut nb) = (0.0f64, 0.0f64, 0.0f64);
        for k in 0..PHASE_CHANNELS {
            let (a, b) = (self.mag[k], other.mag[k]);
            dot += a * b * (self.theta[k] - other.theta[k]).cos();
            na += a * a;
            nb += b * b;
        }
        match na > 1e-12 && nb > 1e-12 {
            true => (dot / (na.sqrt() * nb.sqrt())).clamp(-1.0, 1.0),
            false => 0.0,
        }
    }

    /// Agreement on one relation alone, weighted by the evidence for it.
    pub fn on(&self, other: &Self, k: usize) -> f64 {
        let w = (self.mag[k] * other.mag[k]).sqrt();
        match w > 1e-12 {
            true => (self.theta[k] - other.theta[k]).cos(),
            false => 0.0,
        }
    }

    /// Total evidence, for reporting how concentrated a word's relations are.
    pub fn evidence(&self) -> f64 {
        self.mag.iter().sum()
    }
}
