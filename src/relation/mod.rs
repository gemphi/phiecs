//! `Relation` — the kind of an edge, and the thing Apple's model has no name
//! for.
//!
//! RealityKit's hierarchy is `Parent` / `Children`: one edge kind, one parent
//! (`reality-foundation/components.rs:113`). A language is a multigraph — a
//! word stands in many relations to many words at once, and the *kind* is the
//! payload. In Phient's terms the kind is the condition: `w` means nothing
//! except what it relates to, under which `c`.

mod edge;
mod set;
pub mod vocab;
pub use edge::{Edge, Graph};
pub use set::{DEFAULT_KINDS, KIND_NAMES};
pub use vocab::{RelationVocab, MARKER_CHANNELS_MAX, MARKER_MAX_GAP, OFFSET_CHANNELS};

/// How many relation kinds one language is assumed to hold. Phiano measures
/// this bound rather than asserting it (docs/types); 64 is the channel count
/// every phasor already carries.
pub const MAX_KINDS: usize = 64;

/// One relation kind, opaque: it is issued by a [`Register`] table, because
/// what a kind *means* lives in the table's evidence, not in the number.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Relation(pub(crate) u32);

impl Relation {
    /// Its channel on the relation circle.
    pub fn channel(self) -> usize {
        self.0 as usize
    }
}

/// The kinds this world knows, with the evidence each was learned from.
#[derive(Default, Clone, Debug)]
pub struct Register {
    names: Vec<String>,
    /// How many pairs were seen under each kind.
    evidence: Vec<u32>,
    /// Kinds declared SYMMETRIC: `a near b` puts the two at the same angle;
    /// every other kind is a turn (docs/187).
    symmetric: Vec<bool>,
    /// Name → kind, so declaring and finding stay O(1) however many
    /// condition channels a world opens (docs/190).
    index: std::collections::HashMap<String, u32>,
}

/// The kinds that are symmetric by their meaning — being beside, being
/// alike, being with — read with no turn.
pub const SYMMETRIC: &[&str] = &["near", "with", "like", "same", "beside", "alongside", "together"];

impl Register {
    pub fn new() -> Self {
        Self::default()
    }

    /// The table every world starts with: the named kinds of [`KIND_NAMES`].
    pub fn named() -> Self {
        let mut r = Self::new();
        for n in KIND_NAMES {
            r.declare(n);
        }
        r
    }

    /// Issue (or find) the kind called `name`. The first [`MAX_KINDS`] are
    /// the base channels every word carries densely; every kind past them
    /// is an EXTENSION channel (docs/188, docs/90 §2) a word pays for only
    /// where it touches it — a condition gets its own channel, unbounded
    /// but for the `u32`.
    pub fn declare(&mut self, name: &str) -> Option<Relation> {
        if let Some(i) = self.index.get(name) {
            return Some(Relation(*i));
        }
        if self.names.len() >= u32::MAX as usize {
            return None;
        }
        self.index.insert(name.to_string(), self.names.len() as u32);
        self.symmetric.push(SYMMETRIC.contains(&name));
        self.names.push(name.to_string());
        self.evidence.push(0);
        Some(Relation(self.names.len() as u32 - 1))
    }

    pub fn get(&self, name: &str) -> Option<Relation> {
        self.index.get(name).map(|i| Relation(*i))
    }

    pub fn name(&self, kind: Relation) -> Option<&str> {
        self.names.get(kind.channel()).map(String::as_str)
    }

    /// Record that one more pair was seen under `kind`.
    pub fn observe(&mut self, kind: Relation) {
        if let Some(e) = self.evidence.get_mut(kind.channel()) {
            *e += 1;
        }
    }

    /// Whether `kind` is symmetric — read and coupled with no turn.
    pub fn is_symmetric(&self, kind: Relation) -> bool {
        self.symmetric.get(kind.channel()).copied().unwrap_or(false)
    }

    pub fn evidence(&self, kind: Relation) -> u32 {
        self.evidence.get(kind.channel()).copied().unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Relation, &str)> {
        self.names.iter().enumerate().map(|(i, n)| (Relation(i as u32), n.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_are_stable_and_extend_past_the_base() {
        let mut r = Register::named();
        let a = r.declare("eats").unwrap();
        assert_eq!(Some(a), r.declare("eats"));
        assert_eq!(r.name(a), Some("eats"));
        r.observe(a);
        assert_eq!(r.evidence(a), 1);
        while r.len() < MAX_KINDS {
            r.declare(&format!("k{}", r.len())).unwrap();
        }
        // Past the base, a kind is an extension channel (docs/188), not an
        // error: a condition gets its own channel.
        let more = r.declare("one more").unwrap();
        assert_eq!(more.channel(), MAX_KINDS);
        assert_eq!(r.name(more), Some("one more"));
    }
}
