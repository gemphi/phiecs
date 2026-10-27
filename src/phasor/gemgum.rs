//! Gemgum — Golden-ratio Elastic Matrix: GUm for word Mapping.
//!
//! The binding layer that couples the phiton/chiton light architecture
//! to the linguistic word model. Gemgum uses the golden ratio φ to
//! create a natural, self-organizing mapping between:
//!
//! - **Phase angles** (the existing facet model)
//! - **Color wavelengths** (phiton's spectral domain)
//! - **Temporal frequencies** (chiton's wave domain)
//! - **Word meanings** (the lexicon)
//!
//! # How It Works
//!
//! The golden ratio φ ≈ 1.618 provides the optimal distribution of
//! semantic points across the color-space-time manifold. Gemgum acts as
//! the "gum" that binds words to their physically-derived positions:
//!
//! 1. A word's `SpectralPhasor` (phase, amplitude, band_n) is mapped
//!    through phical to a `ColorSpaceTimePoint` on the manifold.
//! 2. The golden ratio determines the word's natural position within
//!    its spectral band (sub-band distribution).
//! 3. Gemgum computes the binding strength between a word and its
//!    manifold position, which determines how "settled" the word is.
//!
//! It was phiano's `gemgum` module; it lives beside the spectral phasor it
//! reads, and phiano's `facet::spectral` and `compose` call it.
//!
//! This is where "natural linguistic functions and word mapping make
//! intelligence plausible" — words follow the physics of light.

use super::SpectralPhasor;
use phiton::chiton::Chiton;
use phiton::physics::{ColorSpaceTimePoint, Physics, PhysicsOp};
use phiton::types::LightQuantum;
use phiton::units::{ALPHA, PHI, PHI_CONJUGATE, TWO_PI};
use phiton::{Phiton, PhitonColor, PhitonSpectrum};
use serde::Serialize;
use std::f64::consts::PI;

/// Gemgum — the golden-ratio binding engine.
pub struct Gemgum;

impl Gemgum {
    /// Binds a word's phasor to a color-space-time manifold point.
    ///
    /// This is the core mapping: phasor → color → time → meaning.
    /// The golden ratio φ weights the sub-band position to create
    /// a natural, non-uniform distribution of words within each band.
    pub fn bind(phasor: &SpectralPhasor) -> ColorSpaceTimePoint {
        ColorSpaceTimePoint::from_phase(phasor.phase, phasor.band_n)
    }

    /// Computes the binding strength of a word at its current position.
    ///
    /// A word is strongly bound when its phase aligns with a golden-ratio
    /// node on the manifold. Binding strength ∈ [0, 1].
    pub fn binding_strength(phasor: &SpectralPhasor) -> f64 {
        let node_phase = PhysicsOp::bind_to_node(phasor.phase, phasor.band_n);
        let eff_phase = Physics::effective_phase(phasor.phase, phasor.band_n);

        let mut delta = (eff_phase - node_phase).abs();
        if delta > PI {
            delta = TWO_PI - delta;
        }
        1.0 - delta / PI
    }

    /// Maps a word's phasor to a [`PhitonColor`].
    ///
    /// This replaces the hard-coded color array in `SectorPalette`.
    /// The color is derived from the word's phase and band level
    /// through the physics of light.
    pub fn phasor_to_color(phasor: &SpectralPhasor) -> PhitonColor {
        let eff_phase = Physics::effective_phase(phasor.phase, phasor.band_n);
        PhitonSpectrum::phase_to_color(eff_phase, 0)
    }

    /// Maps a sector index to a color name string.
    ///
    /// Drop-in replacement for the old `sector_color` function,
    /// but using physics-derived spectral bands instead of hard-coded names.
    pub fn sector_color_name(sector: u16, sector_count: u16) -> String {
        let color = Phiton::sector_color(sector, sector_count, 0);
        color.name().to_string()
    }

    /// Maps a sector index to a full [`PhitonColor`] with RGB.
    pub fn sector_color(sector: u16, sector_count: u16) -> PhitonColor {
        Phiton::sector_color(sector, sector_count, 0)
    }

    /// Computes the golden-ratio-weighted semantic distance between two words.
    ///
    /// This is the gemgum replacement for the raw phase delta used in
    /// the existing model. It combines:
    /// - Color distance (wavelength difference)
    /// - Time distance (frequency difference)
    /// - Quantum distance (band level difference)
    ///
    /// All three are weighted by the golden ratio φ.
    pub fn semantic_distance(a: &SpectralPhasor, b: &SpectralPhasor) -> f64 {
        let pa = ColorSpaceTimePoint::from_phase(a.phase, a.band_n);
        let pb = ColorSpaceTimePoint::from_phase(b.phase, b.band_n);

        let color_dist = (pa.wavelength_nm - pb.wavelength_nm).abs();
        let time_dist = (pa.frequency_thz - pb.frequency_thz).abs();
        let quantum_dist = (a.band_n as f64 - b.band_n as f64) * ALPHA;

        (color_dist * color_dist
            + PHI * time_dist * time_dist
            + PHI_CONJUGATE * quantum_dist * quantum_dist)
            .sqrt()
    }

    /// Computes the natural word ordering within a spectral band.
    ///
    /// Uses the golden angle (2π/φ²) to distribute words within each
    /// band, creating the most uniform coverage of the semantic space.
    /// This is the same principle used by sunflower seeds and spiral
    /// galaxies — nature's optimal packing.
    pub fn band_position(phasor: &SpectralPhasor) -> f64 {
        let color = Self::phasor_to_color(phasor);
        let band = color.band;
        let pos = (color.wavelength_nm - band.min_nm) / (band.max_nm - band.min_nm);
        (pos * PHI_CONJUGATE).rem_euclid(1.0)
    }

    /// Creates a standing-wave coupling between two words.
    ///
    /// The coupling strength determines how strongly two words interact
    /// in the linguistic model. Words with strong coupling reinforce
    /// each other's meanings; weak coupling means they are independent.
    pub fn word_coupling(a: &SpectralPhasor, b: &SpectralPhasor) -> f64 {
        let qa = LightQuantum::from_phase(a.phase, a.amplitude, a.band_n);
        let qb = LightQuantum::from_phase(b.phase, b.amplitude, b.band_n);
        let ca = Chiton::from_quantum(&qa);
        let cb = Chiton::from_quantum(&qb);

        let interference = PhysicsOp::interference(a.phase, a.band_n, b.phase, b.band_n);
        let wave_resonance = ca.resonance(&cb);
        let spectral_coupling = PhitonSpectrum::coupling(&qa, &qb);

        interference * wave_resonance * (1.0 + spectral_coupling)
    }

    /// Resolves a word's phasor to its full spectral identity.
    ///
    /// Returns the light quantum (particle), chiton (wave), and color
    /// — the complete 2-in-1 representation of the word in the
    /// phiton/chiton architecture.
    pub fn spectral_identity(phasor: &SpectralPhasor) -> SpectralIdentity {
        let quantum = LightQuantum::from_phase(phasor.phase, phasor.amplitude, phasor.band_n);
        let chiton = Chiton::from_quantum(&quantum);
        let color = PhitonSpectrum::phase_to_color(
            Physics::effective_phase(phasor.phase, phasor.band_n),
            0,
        );
        SpectralIdentity { quantum, chiton, color }
    }
}

/// The complete spectral identity of a word — its 2-in-1 light representation.
///
/// This is the "P64 model like GPT" (`phiton::p64`) — each word is represented as
/// a quantum of light with both particle (phiton) and wave (chiton)
/// aspects, plus its derived color.
#[derive(Debug, Clone, Serialize)]
pub struct SpectralIdentity {
    /// The particle/roll aspect (phiton).
    pub quantum: LightQuantum,
    /// The wave/oscillation aspect (chiton).
    pub chiton: Chiton,
    /// The derived color from the visible spectrum.
    pub color: PhitonColor,
}

impl SpectralIdentity {
    /// Returns the human-readable color name.
    pub fn color_name(&self) -> &'static str {
        self.color.name()
    }

    /// Returns the hex color code.
    pub fn hex(&self) -> String {
        self.color.hex()
    }

    /// Returns the frequency in THz.
    pub fn frequency_thz(&self) -> f64 {
        self.quantum.frequency_thz()
    }

    /// Returns the photon energy in eV.
    pub fn energy_ev(&self) -> f64 {
        self.quantum.energy_ev()
    }
}
