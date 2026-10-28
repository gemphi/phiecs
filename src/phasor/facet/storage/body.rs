#[allow(unused_imports)]
use super::*;

// ── v1: parsed by hand (see `Storage::read_v1_lexicon`) ────────────────────

pub(super) struct V1Body {
    pub(super) lexicon: HashMap<String, LegacyPhasor>,
    pub(super) bigrams: HashMap<String, HashMap<String, u32>>,
    pub(super) trigrams: HashMap<String, HashMap<String, u32>>,
    pub(super) phase_lags: HashMap<String, HashMap<String, f64>>,
    pub(super) grounded_version: u32,
}

/// A cursor over a bincode body with fixed-int encoding, little-endian.
pub(super) struct V1Reader<'a> {
    pub(super) body: &'a [u8],
    pub(super) o: usize,
}

/// Storage - facade for persisting and loading facets to/from disk.
pub struct Storage;
