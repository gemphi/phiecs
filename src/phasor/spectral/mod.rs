//! `SpectralPhasor`: a word packed for a lexicon. `PHASE_CHANNELS` angles a
//! byte each, the evidence behind each channel, and the target side the
//! word presents when it is related to (docs/69). It was phiano's
//! `phasor` module; phiano stores its lexicon in it and re-exports it.

mod types;
mod torus;
mod widen;
mod channels;
mod count;
mod nudge;
mod dither;
mod spectrum;
#[cfg(test)]
mod tests;
pub use types::*;
pub use torus::*;
pub use widen::*;
#[allow(unused_imports)]
use channels::*;
#[allow(unused_imports)]
use count::*;
#[allow(unused_imports)]
use nudge::*;
pub use dither::Rounding;
// The dithered quantiser itself, for phiano's extension channels (docs/90).
pub use dither::quantise;
#[allow(unused_imports)]
use spectrum::*;

mod constants;
pub use constants::*;

use phistd::{c64, GOLDEN_ANGLE};
use phiton::units::{BAND_N_EFFECTIVE_MAX, TWO_PI};
use phiton::{LightQuantum, PhitonColor, PhitonSpectrum};
use serde::{Deserialize, Serialize};
