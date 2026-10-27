#[allow(unused_imports)]
use super::*;

impl SpectralPhasor {

    /// Computes the multi-frequency harmonic spectrum Z(k) across D frequencies on the torus T^D.
    #[allow(dead_code)]
    pub fn harmonic_spectrum(&self, d: usize) -> Vec<c64> {
        (0..d)
            .map(|k| {
                let harmonic_amp = self.amplitude / (1.0 + 0.1 * k as f64);
                c64::from_polar(harmonic_amp, self.theta(k))
            })
            .collect()
    }

    /// Returns the effective phase including the fine-structure sub-band correction.
    ///
    /// φ_eff = φ + min(n, BAND_N_EFFECTIVE_MAX)·α
    pub fn effective_phase(&self) -> f64 {
        phiton::units::effective_phase(self.phase, self.band_n.min(BAND_N_EFFECTIVE_MAX))
    }

    /// Resolves this phasor to a physics-derived [`PhitonColor`].
    #[allow(dead_code)]
    pub fn to_color(&self) -> PhitonColor {
        PhitonSpectrum::phase_to_color(self.effective_phase(), 0)
    }

    /// Converts this phasor into a [`LightQuantum`] (phiton particle).
    #[allow(dead_code)]
    pub fn to_light_quantum(&self) -> LightQuantum {
        LightQuantum::from_phase(self.phase, self.amplitude, self.band_n)
    }
}

impl TorusPhasor {
    pub fn from_spectral(phasor: &SpectralPhasor) -> Self {
        let mut harmonics = [0.0; TORUS_HARMONICS_COUNT];
        for (k, h) in harmonics.iter_mut().enumerate() {
            *h = phasor.theta(k);
        }
        Self {
            base_phase: phasor.phase,
            amplitude: phasor.amplitude,
            harmonics,
        }
    }

    /// Resonance overlap between two torus phasors across all discrete frequencies.
    pub fn resonance(&self, other: &Self) -> f64 {
        let mut sum = (self.base_phase - other.base_phase).cos();
        for k in 0..TORUS_HARMONICS_COUNT {
            sum += (self.harmonics[k] - other.harmonics[k]).cos();
        }
        (sum / ((TORUS_HARMONICS_COUNT + 1) as f64)).clamp(-1.0, 1.0)
    }
}
