//! `Wave`: the facet read as waves - a sentence as one complex number, the
//! ray cast back to words, the sector index over the circle. It was phiano's
//! `wave`; phiano reads it from here.

mod sentence;
mod ray;
mod sector;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use sentence::*;
#[allow(unused_imports)]
use ray::*;
#[allow(unused_imports)]
use sector::*;

use phistd::GOLDEN_ANGLE;
use phiton::units::{PHASE_CHANNELS, TWO_PI};
use crate::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use rayon::prelude::*;
use std::cmp::Ordering;

use phistd::c64;

/// Wave - operations on complex wave representations of text.
pub struct Wave;

/// Coarse spatial index over the phase circle.
///
/// Retrieval was O(V) per query, and generation issues one per token. On a
/// circle, nearest-neighbour search does not need a full scan: bucket words by
/// sector, then scan the query's own sector and its neighbours outward. The
/// expected cost is V/`sector_count` rather than V.
///
/// This is the one place a one-dimensional representation is an advantage, and
/// it was going unused.
pub struct SectorIndex {
    pub(super) buckets: Vec<Vec<String>>,
}
