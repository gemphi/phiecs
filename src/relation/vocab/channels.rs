#[allow(unused_imports)]
use super::*;
use crate::relation::Relation;

impl Default for RelationVocab {
    /// The eight offset channels, which every vocabulary holds before any
    /// marker is learned.
    fn default() -> Self {
        let mut inner = Register::new();
        for k in 0..OFFSET_CHANNELS {
            inner.declare(&format!("offset+{}", k + 1));
        }
        Self { inner }
    }
}

impl RelationVocab {
    /// True until [`RelationVocab::learn`] has run: only the offset channels
    /// are available, which is the one relation always observable.
    pub fn is_empty(&self) -> bool {
        self.inner.len() <= OFFSET_CHANNELS
    }

    /// The shared registry this vocabulary is a view of.
    pub fn relations(&self) -> &Register {
        &self.inner
    }

    /// Channels no relation occupies — free for the ranking objective.
    ///
    /// Offsets always hold `0..OFFSET_CHANNELS`; markers hold the next
    /// `markers()` channels. Everything above is unclaimed.
    pub fn unclaimed(&self) -> Vec<usize> {
        (self.inner.len()..PHASE_CHANNELS).collect()
    }

    /// The `(marker, channel)` table, for serialization: markers only, in
    /// channel order. Offsets are implied and placeholders (`chN`) are not
    /// relations.
    pub fn entries(&self) -> Vec<(String, usize)> {
        self.inner
            .iter()
            .map(|(r, n)| (n.to_string(), r.channel()))
            .filter(|(n, c)| *c >= OFFSET_CHANNELS && *n != format!("ch{c}"))
            .collect()
    }

    /// Rebuilds a table from stored entries. Entries are placed at the channel
    /// they were stored with; a gap is held by a `chN` placeholder so no
    /// marker moves channel on the way through disk.
    pub fn from_entries(mut entries: Vec<(String, usize)>) -> Self {
        entries.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        let mut v = Self::default();
        for (name, c) in entries {
            if c < OFFSET_CHANNELS || c >= PHASE_CHANNELS || v.inner.get(&name).is_some() {
                continue;
            }
            while v.inner.len() < c {
                let k = v.inner.len();
                v.inner.declare(&format!("ch{k}"));
            }
            v.inner.declare(&name);
        }
        v
    }

    /// The marker channel for a pair at `i` and `j`, if a word between them
    /// names their relation.
    ///
    /// The rule is: among the words between the pair, the marker is the one that
    /// links *least often* across the corpus. This replaced a filler set, and it
    /// is both simpler and better founded.
    ///
    /// The problem it solves is `a dog eats a donut`. The original rule fired
    /// only when exactly one word separated the pair, so `dog eats donut` was an
    /// `eats` relation and `a dog eats a donut` was an offset-3 one — the rule
    /// found the relation in the sentence a grammarian would write and missed it
    /// in the sentence a person would say. The first fix was a filler list of
    /// frequent words to skip over, and it deleted `of`, `in` and `to`, which are
    /// frequent *and* are the clearest relational markers in the language.
    ///
    /// Rarity settles it without a list. Between `dog` and `donut` stand `eats`
    /// and `a`; `a` links fourteen thousand times and `eats` a handful, so `eats`
    /// is the more specific claim about this pair and it wins. Between
    /// `president` and `states` stand `of` and `the`; `of` links thirty-two
    /// thousand times against `the`'s forty thousand, so `of` wins. The same
    /// comparison, no exceptions, and it needs no extra storage: channels are
    /// handed out in rank order, so a *higher channel index is a rarer marker*
    /// and the rule is a maximum over channel indices.
    pub(super) fn marker(&self, toks: &[String], i: usize, j: usize) -> Option<usize> {
        match j - i <= MARKER_MAX_GAP {
            true => ((i + 1)..j).filter_map(|m| self.channel_of(&toks[m])).max(),
            false => None,
        }
    }

    /// The channel for a pair at `i` and `j` in `toks`, if any.
    ///
    /// A marked relation takes precedence over a bare offset: `bread of wheat`
    /// is an `of` relation first and an offset-2 relation second, and the marker
    /// is the more specific claim.
    pub fn channel(&self, toks: &[String], i: usize, j: usize) -> Option<usize> {
        if let Some(c) = self.marker(toks, i, j) {
            return Some(c);
        }
        let d = j - i;
        match d <= OFFSET_CHANNELS {
            true => Some(d - 1),
            false => None,
        }
    }

    /// The channel `marker` was given, if any. An offset name is not a marker.
    pub fn channel_of(&self, marker: &str) -> Option<usize> {
        self.inner.get(marker).map(Relation::channel).filter(|c| *c >= OFFSET_CHANNELS)
    }

    pub fn markers(&self) -> usize {
        self.inner.len().saturating_sub(OFFSET_CHANNELS)
    }

    /// The marker a channel stands for, for inspection.
    pub fn name(&self, k: usize) -> String {
        self.inner
            .iter()
            .find(|(r, _)| r.channel() == k)
            .map(|(_, n)| n.to_string())
            .unwrap_or_else(|| format!("ch{k}"))
    }
}
