//! The eye — active vision as rows, beside the body, the track and the
//! device. It lived as its own crate (phioml) until the rule "one crate,
//! one thing" brought it home: what a persona sees is told the way what it
//! does is told.
//!
//! Ported from `gemphi/phi/crates/phi-oml` (docs/167), the portable
//! replacement for Apple's CoreML/MLCompute/Metal, and re-based on
//! phistd's `c64` so the eye and the language share one number:
//!
//! - [`Aoc`] — PhiFly's AOC signal: phase + amplitude, interference by
//!   complex addition. It is one channel of a phasor, said in vision's words.
//! - [`Aoc::resonance`] — phi-oml's `cpu_resonance`: a complex multiply by a
//!   micro-jitter, O(N), twelve flops a signal, no backprop anywhere.
//! - [`Flow`] — the flow field: a scene of named signals twitched twice;
//!   the phase difference under the jitter is parallax, parallax is depth,
//!   and the scene is **told as rows** (`Flow::rows`), so an agent watches
//!   a scene the way it watches the kit.

pub mod flow;
pub mod signal;

pub use flow::Flow;
pub use signal::Aoc;
