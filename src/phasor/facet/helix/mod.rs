//! The helix: a facet as a Linux directory tree, coded like DNA (it was
//! phiano's `helix`; the code per channel is `phical::helix::Codon`).
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c). Phical stores no point: a word is 64 relations, each a
//! phasor on a closed circle. DNA stores no point either: a gene
//! is 64 codons — three bases of four — and what a codon *means* is fixed by
//! where it sits on a helix that puts every position in three-dimensional
//! space. The correspondence is exact in number and, this module argues,
//! in kind: 64 codons, 64 hexagrams (`phistd::hexagram`), 64 spherical
//! harmonics of degree < 8 (`phifno::sphere`), 64 relation channels. So a
//! word is written as a **strand** of 64 codons, one per channel, each
//! carrying the word's source and target angles on that channel (the two
//! strands of the double helix) and its evidence count; and the tree is laid
//! out so that `ls`, `grep` and `cat` are the query language.
//!
//! ```text
//! helix/
//!   README.md               the code table, in words
//!   codons.tsv              k  codon  hexagram  degree  order  x y z  relation
//!   strands/<aa>/<word>.dna one line per codon: codon  src  tgt  evidence  λnm  band  hex
//!   codons/<CODON>/words.tsv  the words with most evidence on that channel
//! ```
//!
//! `codon(k)` is `k` in base four over `ACGT`; the hexagram is `k`'s six
//! bits; `(x, y, z)` is the codon's place on a B-DNA helix (10.5 codons per
//! turn, 0.34 nm rise) — every code a position in space, as asked. `write`
//! and `read` round-trip the lexicon exactly (tested); n-grams are not
//! part of the helix and stay in the `.chroma`. The last three columns are
//! PhiTon's reading of the codon — the source angle as a wavelength, its
//! spectral band and colour — so a strand is a spectrum a person can see,
//! in the project's own units rather than a bare `f64`.

mod read;
mod write;

pub use read::read;
pub use write::write;

#[allow(unused_imports)]
use phical::helix::Codon;
#[allow(unused_imports)]
use phistd::codon::{codon, codon_index, helix_position, hexagram_glyph, BASES, CODONS, PITCH_PER_TURN, RISE};
