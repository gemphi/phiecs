//! The AOC signal (phi-oml `phifly::aoc`, re-based): polar phase and
//! amplitude over [`phistd::c64`]. Interference is complex addition;
//! resonance is complex multiplication — the two moves everything else in
//! this workspace is made of, which is why the port is small.

use phistd::c64;

/// A signal in polar form: one channel of a phasor, in vision's words.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aoc {
    /// Phase angle, radians.
    pub phase: f64,
    /// Non-negative magnitude.
    pub amplitude: f64,
}

impl Aoc {
    pub fn new(phase: f64, amplitude: f64) -> Self {
        Self { phase, amplitude }
    }

    /// Seeded from a word's identity on channel `k`: the one seed
    /// (docs/159), so a signal is born where the word's phasor is.
    pub fn seeded(word: &str, k: usize) -> Self {
        Self { phase: phistd::seed_angles(word, k + 1)[k], amplitude: 1.0 }
    }

    /// The complex form, `amplitude · e^{i·phase}`.
    pub fn to_complex(self) -> c64 {
        c64::from_polar(self.amplitude, self.phase)
    }

    fn from_complex(z: c64) -> Self {
        Self { phase: z.arg(), amplitude: z.norm() }
    }

    /// Vector-addition interference: both to complex, summed, back to polar.
    pub fn interfere(self, other: Aoc) -> Aoc {
        Self::from_complex(self.to_complex() + other.to_complex())
    }

    /// Superposition of many signals.
    pub fn superpose(signals: &[Aoc]) -> Aoc {
        Self::from_complex(signals.iter().map(|s| s.to_complex()).sum())
    }

    /// phi-oml's `cpu_resonance`: the phase of this signal rotated by a
    /// micro-jitter (a complex number), one multiply and one `atan2`.
    pub fn resonance(self, jitter: c64) -> f64 {
        (self.to_complex() * jitter).arg()
    }
}

/// A micro-jitter: the eye's own movement as a unit rotation.
pub fn jitter(angle: f64) -> c64 {
    c64::from_polar(1.0, angle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    #[test]
    fn interference_and_resonance_are_the_complex_moves() {
        let a = Aoc::new(0.0, 1.0);
        let b = Aoc::new(PI, 1.0);
        let cancelled = a.interfere(b);
        assert!(cancelled.amplitude < 1e-12, "opposite phases cancel");
        let c = Aoc::new(FRAC_PI_2, 2.0);
        let built = Aoc::superpose(&[c, c]);
        assert!((built.amplitude - 4.0).abs() < 1e-12 && (built.phase - FRAC_PI_2).abs() < 1e-12);
        // Resonance rotates the phase by the jitter's angle.
        assert!((a.resonance(jitter(0.3)) - 0.3).abs() < 1e-12);
        assert!((c.resonance(jitter(0.3)) - (FRAC_PI_2 + 0.3)).abs() < 1e-12);
        // The seed is the one seed.
        assert_eq!(Aoc::seeded("boat", 3).phase, phistd::seed_angles("boat", 4)[3]);
    }
}
