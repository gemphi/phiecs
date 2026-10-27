//! `RelationVocab`: which channel each observable relation occupies - the
//! eight offsets, then the markers learned from a corpus - over the one
//! channel [`Register`]. It was phiano's `relational::RelationVocab`; the
//! facet carries it as its relation inventory.

mod channels;
mod learn;
mod store;
#[cfg(test)]
mod tests;

use crate::relation::Register;
use phiton::units::PHASE_CHANNELS;
use std::collections::HashMap;

/// Channels reserved for relative offset. The rest go to markers.
pub const OFFSET_CHANNELS: usize = 8;

/// The widest gap at which a word between a pair may still name their relation.
///
/// Three means at most two words stand between them, which covers `dog eats
/// donut` and `a dog eats a donut` and stops well short of picking some rare
/// word six positions away purely because it is rare.
pub const MARKER_MAX_GAP: usize = 3;

/// Marker channels the relation inventory may claim, above the 8 offsets.
///
/// Channel ownership: the ranking objective moves only channels no relation
/// claims (`RelationVocab::unclaimed`), so this decides the split between
/// relation channels and prediction channels. 56 hands every channel to a
/// relation and the ranking objective falls back to all 64, which is the
/// arrangement `bin/promoted` first measured — both objectives fighting over
/// the same angles. 24 leaves 32 channels to prediction.
pub const MARKER_CHANNELS_MAX: usize = 24;

/// Which channel each observable relation occupies.
///
/// Built from the corpus, capped at [`PHASE_CHANNELS`]: offsets first, then the
/// most frequent between-word markers until the channels run out. The bound is
/// the channel count.
///
/// Since docs/159 this is a thin view over the one channel registry of the
/// workspace, [`Register`]: the eight offset channels are
/// declared first under the names `offset+1..=8`, and every marker after them
/// in rank order, so a channel index means the same thing to a `Facet` and to
/// a `World`. The public API and the `.chroma` entry format
/// are unchanged.
#[derive(Debug, Clone)]
pub struct RelationVocab {
    pub(super) inner: Register,
}
