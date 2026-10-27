//! The spectral phasor's geometry and amplitude rule. They were phiano's
//! `config` constants; phiano's `config` re-exports them from here.

// ── MULTI-CHANNEL PHASE REPRESENTATION (D-dimensional torus) ───────────────

/// The channels per word (D = 64, one byte each) and their quanta (256): the
/// P64 geometry, defined once in phiton.
pub use phiton::units::{CHANNEL_QUANTA, PHASE_CHANNELS};

/// Number of discrete harmonics evaluated on the multi-frequency torus (T^D).
pub const TORUS_HARMONICS_COUNT: usize = 32;

/// Log-frequency amplitude scale: A = 1 + ln(count) / AMPLITUDE_LOG_SCALE.
/// ln(1e6)/14 ≈ 0.99, so a millionfold range maps into [1.0, 2.0].
pub const AMPLITUDE_LOG_SCALE: f64 = 14.0;

/// Maximum amplitude - the familiarity ceiling.
pub const AMPLITUDE_MAX: f64 = 2.0;
