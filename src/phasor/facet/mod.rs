//! `Facet`: the lexicon of spectral phasors, a word to its 64 channels,
//! with the interned n-gram tables and the relation inventory beside it.
//! It was phiano's `facet`; phiano reads it from here.

mod vocab;
mod learn;
mod fit;
mod under;
mod rebuild;
mod centroid;
mod pmi;
#[allow(unused_imports)]
use vocab::*;
#[allow(unused_imports)]
use learn::*;
#[allow(unused_imports)]
use fit::*;
#[allow(unused_imports)]
use under::*;
pub use under::Reading;
#[allow(unused_imports)]
use rebuild::*;
#[allow(unused_imports)]
use centroid::*;
#[allow(unused_imports)]
use pmi::*;

mod spectral;
pub mod constants;
pub use constants::*;

#[cfg(test)]
mod tests;

use phiton::units::PHASE_CHANNELS;
use crate::phasor::SpectralPhasor;
use crate::relation::RelationVocab;
use crate::phasor::wave::Wave;
use phistd::c64;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A word's interned identifier.
pub type WordId = u32;

/// String-to-id interner for the n-gram tables.
///
/// The tables previously keyed on owned `String`s: every bigram follower stored
/// a full copy of the word, every trigram key stored two, and `phase_lags`
/// duplicated the whole bigram key set a third time. On a real model that was
/// the overwhelming majority of a 92 MB artifact, against a documented 2–12 MB
/// target — and `trigram_candidates` allocated a fresh `format!("{} {}", a, b)`
/// on every *lookup*, once per candidate per generated token.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Vocab {
    pub(super) ids: HashMap<String, WordId>,
    pub(super) words: Vec<String>,
}

/// Tally needed to score one n-gram continuation.
#[derive(Debug, Clone, Copy)]
pub struct NgramStat {
    /// Times this exact continuation was seen.
    pub count: u32,
    /// Times the context was seen at all.
    pub total: u32,
    /// Distinct continuations observed for the context.
    pub types: u32,
}

/// Sorted association list of `(id, count)`.
///
/// A sorted `Vec` with binary search costs 8 bytes per entry against roughly a
/// hundred for a nested `HashMap<String, u32>` entry, and it has no per-context
/// hash-table allocation.
type Followers = Vec<(WordId, u32)>;

/// Facet - the core lexicon mapping words to complex phasors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facet {
    /// The word-to-phasor lexicon.
    pub lexicon: HashMap<String, SpectralPhasor>,
    /// Interner shared by every n-gram table.
    #[serde(default)]
    pub vocab: Vocab,
    /// Bigram transitions: id(a) → sorted [(id(b), count)].
    #[serde(default)]
    pub bigrams: HashMap<WordId, Followers>,
    /// Trigram transitions: (id(a), id(b)) → sorted [(id(c), count)].
    #[serde(default)]
    pub trigrams: HashMap<(WordId, WordId), Followers>,
    /// Learned Kuramoto-Sakaguchi phase lags β_ij for a → b.
    #[serde(default)]
    pub phase_lags: HashMap<(WordId, WordId), f32>,
    /// Version of the definition-grounding pass already applied to this facet.
    #[serde(default)]
    pub grounded_version: u32,
    /// Which channel each observable relation occupies.
    ///
    /// Offsets 1..=8 always have channels 0..8; the marker channels above them
    /// are filled by [`Facet::learn_relations`] from the corpus. Until that runs
    /// only adjacency deposits evidence, which is the one relation always
    /// available.
    #[serde(default)]
    pub relations: RelationVocab,
    /// Flat vocabulary list used for frequency-biased negative sampling.
    #[serde(skip)]
    pub sample_pool: Vec<String>,
    /// Negatives for the relation step, drawn in proportion to count^0.75
    /// with no cap: the words that meet the most partners must also be the
    /// ones most often pushed apart (Mikolov et al. 2013).
    pub relation_pool: Vec<String>,
}
