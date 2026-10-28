//! `storage` — the facet on disk: the `.chroma` format, a header (the format
//! version and the fine-structure constant it was written under), the
//! lexicon and the keyed n-gram tables. It was phiano's `storage`.

mod types;
mod body;
mod disk;
mod save;
mod load;
mod keyed;
#[cfg(test)]
mod tests;
pub use types::*;
pub use body::*;
#[allow(unused_imports)]
use disk::*;
#[allow(unused_imports)]
use save::*;
#[allow(unused_imports)]
use load::*;
#[allow(unused_imports)]
use keyed::*;

use phiton::units::ALPHA;
use crate::phasor::facet::{Facet, Vocab, WordId};
use crate::phasor::spectral::{widen_evidence, SpectralPhasor, EVIDENCE_WORDS, EVIDENCE_WORDS_V5};
use crate::relation::RelationVocab;
use serde::{Deserialize, Serialize};

/// On-disk format version. Bumped when the serialized layout changes.
pub const FORMAT_VERSION: u32 = 6;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Result};
