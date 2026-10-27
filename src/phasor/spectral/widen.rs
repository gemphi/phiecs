#[allow(unused_imports)]
use super::*;

/// Expands a 16-bit-lane evidence block (formats v4/v5) into the 32-bit layout.
pub fn widen_evidence(old: [u64; EVIDENCE_WORDS_V5]) -> [u64; EVIDENCE_WORDS] {
    let mut out = [0u64; EVIDENCE_WORDS];
    for k in 0..PHASE_CHANNELS {
        let c = (old[k / 4] >> (16 * (k % 4))) & 0xFFFF;
        out[k / 2] |= c << (32 * (k % 2));
    }
    out
}

/// FNV-1a over a spelling: phinet's, so every seed in the family is one hash.
pub use phistd::fnv1a;

/// splitmix64's finaliser: avalanches every input bit into every output bit.
///
/// FNV-1a does not avalanche in a few rounds. Over three bytes its high bits are
/// still dominated by the first byte, and its low bit is the parity of all of
/// them — the multiply by an odd prime preserves it. Neither end is safe to slice
/// bits out of, and this project has been bitten by both:
///
/// * `seeded` took `h >> 11` as a word's phase, so `cow`, `cod` and `cot`
///   (`f604f819…`, `f604eb19…`, `f604fb19…`) received *the same* phase on 63 of
///   64 channels and resonated at 1.0000. Every word's starting position was a
///   function of its spelling prefix, which is exactly the thing the manifold is
///   supposed to learn its way out of.
/// * `kinds::split_side` took the low bit, so `h_x` and `h_y` landed in
///   opposite halves of a fit/held split without exception.
///
/// Anything deriving a number from a word hash goes through this first.
pub use phistd::mix64;
