//! One channel per relation, and a magnitude that can carry something.
//!
//! It was phiano's `relational` module; it lives with the other phasor
//! forms, kept whole so `examples/relational` and `examples/marked` still
//! reproduce, and phiano's `flow` reads it.
//!
//! **Promoted.** What this module found is now in the lexicon type itself:
//! [`crate::phasor::SpectralPhasor`] carries per-channel evidence
//! (`evidence`, `deposit`, `similarity_with`, `on`, `shared_relation_with`),
//! [`crate::phasor::facet::Facet`] carries the relation inventory (`relations`,
//! `learn_relations`, `channel_balance`) and
//! phiano's `Trainer::couple_relations` deposits evidence on the
//! channel of the relation each pair was observed under, with a contrastive
//! term this module's attraction-only step lacked. [`RelationalPhasor`] and
//! [`RelationalFacet`] stay as the isolated experiment `bin/relational` runs;
//! `bin/promoted` measures the promotion against the shipped trainer.
//!
//! `bin/dimensions` established two things about the shipped representation.
//!
//! **The channels cannot specialise.** Training is symmetric in them: seeds are
//! hashes, the Kuramoto step treats every channel alike, and `channel_kernel`
//! varies only decay and frequency. Nothing distinguishes channel 17 from channel
//! 42, so nothing can push channel 17 toward being *the gender channel*. Scoring
//! a pair by its best dimension came out at 43.9% against a 50% chance line,
//! because there are no dimensions to be best at.
//!
//! **The magnitude cannot matter.** Amplitude is one number per word, shared
//! across all 64 channels, so it factors out of a normalised inner product
//! exactly. Phase-only and full-complex scoring agreed to the decimal place in
//! every arm of `bin/constructed` — not approximately, identically. Half of every
//! phasor in this project is mathematically incapable of influencing a result.
//!
//! [`RelationalPhasor`] fixes both, and they are the same fix. A word carries a
//! phase *and a magnitude per channel*, and channel `r` belongs to relation `r`:
//! the angle says where the word sits under that relation, the magnitude says how
//! much evidence it has for standing in it at all. A word never observed under
//! relation `r` has magnitude near zero there and contributes nothing to
//! comparisons along it — which is what makes the complex number do work that the
//! angle alone cannot.
//!
//! # Where the relations come from
//!
//! Not from a list I wrote. Two observable, label-free sources, packed into the
//! 64 channels:
//!
//! * **Adjacency.** The relative offset between two words in a sentence, 1 to
//!   [`OFFSET_CHANNELS`]. "Next to" is a relation, and it is the one relation
//!   always available.
//! * **Markers.** The most frequent word standing *between* two others names the
//!   relation between them — `of`, `in`, `as`, `to`. This is the observation
//!   behind `roles::RELATIONAL_PREPOSITIONS`, except that the markers are taken
//!   by corpus frequency rather than chosen, so the language supplies the
//!   inventory.
//!
//! # Why this breaks the symmetry
//!
//! Training couples two words **on one channel only** — the channel of the
//! relation under which they were observed. A pair seen under `of` moves the
//! `of` channel and leaves the other 63 alone. After enough text, channel `r`
//! holds an arrangement of the vocabulary specific to relation `r`, which is
//! precisely what a symmetric update can never produce.
//!
//! # What it measured, and why it loses
//!
//! Built and run on WikiText-2 prose, 36,577 words, 56 marker channels plus 8
//! offset channels:
//!
//! ```text
//! similarity                                pair>random
//! shipped: mean of 64 channels                    61.5%
//! relational: evidence-weighted, all              50.1%
//! relational: best single dimension               38.8%
//! relational: mean of best 8                      41.6%
//! ```
//!
//! **The symmetry break works mechanically and loses on the benchmark.** Both
//! fixes do what they were built to do — the tests below prove an observation
//! moves one channel and that per-channel magnitude changes a similarity, neither
//! of which the shipped representation can do. And the result is at chance, ten
//! points behind the representation it was meant to improve.
//!
//! The reason is sparsity, and it is worth stating precisely because it is not a
//! bug. Two arbitrary words almost never co-occur under `of`. Of the ten relation
//! families, every one's best channel is an *offset* channel covering 100% of its
//! pairs, and the marker channels cover almost none — so the only dimensions on
//! which two arbitrary words can be compared are the positional ones, which carry
//! where a word sits rather than what it means. Only gender (+0.094) and agent
//! (+0.058) clear the -0.004 noise floor.
//!
//! **Relation-specific channels make comparison sparse, and sparse comparison
//! loses to dense averaging on a similarity task.** That is a real trade, not a
//! failure of the idea: this architecture puts each relation somewhere definite,
//! which is what *composition* needs — deliberately working within one relation —
//! and takes away the everything-at-once blend that *similarity* wants. The
//! benchmark here is a similarity benchmark, and it is measuring the cost of the
//! trade without measuring the benefit.
//!
//! Two failures found on the way, both caught by the measurement rather than by
//! reading the code. The first run scored 35.9%, *below* chance, because channel
//! traffic is wildly unequal — every adjacent pair feeds an offset channel while a
//! marker channel sees only pairs joined by that word — so the offset channels
//! outvoted everything by two orders of magnitude. [`RelationalFacet::balance`]
//! equalises the vote and recovered 14 points. The second was in the measurement:
//! averaging a channel's agreement over the whole family scores it zero wherever a
//! pair was never seen, so only universally-populated channels could ever come
//! first. Fixed, and the conclusion was unchanged — the offset channels really do
//! cover every pair.

mod phasor;
mod facet;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use phasor::*;
#[allow(unused_imports)]
use facet::*;

use crate::relation::RelationVocab;
use phiton::units::{PHASE_CHANNELS, TWO_PI};
use std::collections::HashMap;

/// How far apart two words may be and still count as related.
pub const WINDOW: usize = 8;

/// Coupling strength per observation.
pub const RATE: f64 = 0.06;

/// Below this circular dispersion a channel has collapsed and is muted.
///
/// The same reasoning as `core::grounding::DISPERSION_FLOOR`, applied per
/// channel because that is where this representation collapses.
pub const CHANNEL_DISPERSION_FLOOR: f64 = 0.35;

/// Below this vocabulary the collapse guard does not run.
pub const GUARD_MIN_VOCAB: usize = 200;

/// A word as 64 complex numbers: one per relation.
#[derive(Debug, Clone)]
pub struct RelationalPhasor {
    /// Angle per relation channel.
    pub theta: Vec<f64>,
    /// Evidence per relation channel. This is the magnitude, and unlike the
    /// shipped representation it varies across channels — which is the only way
    /// a magnitude can survive normalisation and affect a comparison.
    pub mag: Vec<f64>,
}

/// The manifold: one relational phasor per word.
#[derive(Debug, Default)]
pub struct RelationalFacet {
    pub words: HashMap<String, RelationalPhasor>,
    pub counts: HashMap<String, u32>,
    /// Channels that survived the collapse guard.
    pub live: usize,
}
