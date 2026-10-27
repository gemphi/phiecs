#[allow(unused_imports)]
use super::*;

/// Multi-frequency Torus Phasor (T^D).
///
/// Reads the phasor's actual independent channels. Previously every harmonic was
/// a deterministic function of the single `phase` field, so the torus carried
/// exactly one number's worth of information — a 1-D curve embedded in T^D.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorusPhasor {
    pub base_phase: f64,
    pub amplitude: f64,
    pub harmonics: [f64; TORUS_HARMONICS_COUNT],
}
