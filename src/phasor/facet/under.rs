#[allow(unused_imports)]
use super::*;

/// Which channels a directed reading averages over.
///
/// `Offsets` is what `calls_for` has always read: the eight channels every
/// adjacent pair feeds. `Markers` is the relation channels a corpus or a
/// quest names (`of`, `with`, `is-a`). `All` is both. Chosen for a run by
/// `PHIANO_READING=offsets|markers|all` (default `offsets`, the shipped
/// behaviour), so the generator and the dialogue read the same way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reading {
    Offsets,
    Markers,
    All,
}

impl Reading {
    pub fn from_env() -> Self {
        match std::env::var("PHIANO_READING").as_deref() {
            Ok("markers") => Reading::Markers,
            Ok("all") => Reading::All,
            _ => Reading::Offsets,
        }
    }
}

impl Facet {
    /// [`Facet::calls_for`] over the channels `reading` names: the mean
    /// directed agreement from `word` to `goal` over those channels where
    /// both carry evidence.
    pub fn calls_for_with(&self, word: &str, goal: &str, reading: Reading) -> f64 {
        let (a, g) = match (self.lexicon.get(word), self.lexicon.get(goal)) {
            (Some(a), Some(g)) => (a, g),
            _ => return 0.0,
        };
        let offsets = crate::relation::OFFSET_CHANNELS;
        let markers = offsets..offsets + self.relations.markers();
        let channels: Vec<usize> = match reading {
            Reading::Offsets => (0..offsets).collect(),
            Reading::Markers => markers.collect(),
            Reading::All => (0..offsets).chain(markers).collect(),
        };
        let (mut sum, mut n) = (0.0, 0usize);
        for k in channels {
            if a.evidence_count(k) > 0 && g.evidence_count(k) > 0 {
                sum += a.on(g, k);
                n += 1;
            }
        }
        if n == 0 { 0.0 } else { sum / n as f64 }
    }

    /// The channel a marker word was given, if it has one: `of`, `with`, or a
    /// quest's `is-a`. `None` for a word that names no relation here.
    pub fn marker_channel(&self, marker: &str) -> Option<usize> {
        self.relations.channel_of(marker)
    }

    /// How much `word` calls for `goal` **under** `marker` — the directed
    /// reading on that one channel, X ⇒ Y under c. Zero when the marker names
    /// no relation, or either word lacks evidence on it.
    ///
    /// [`Facet::calls_for`] reads the offset channels, which every adjacent
    /// pair feeds and no relation name ever touches; docs/83 measured that
    /// reading at chance on a quest whose every edge was written on a marker
    /// channel. This is the reading that sees them.
    pub fn calls_for_under(&self, word: &str, marker: &str, goal: &str) -> f64 {
        match self.marker_channel(marker) {
            Some(k) => self.on(word, goal, k),
            None => 0.0,
        }
    }

    /// Everything in `pool` that `word` calls for under `marker`, strongest
    /// first, zeros dropped. The quest's own question, put to the facet:
    /// "`word` · `marker` · ?".
    pub fn under(&self, word: &str, marker: &str, pool: &[String]) -> Vec<(String, f64)> {
        let Some(k) = self.marker_channel(marker) else { return Vec::new() };
        let mut out: Vec<(String, f64)> = pool
            .iter()
            .filter(|w| w.as_str() != word)
            .map(|w| (w.clone(), self.on(word, w, k)))
            .filter(|(_, s)| *s > 0.0)
            .collect();
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.0.cmp(&b.0)));
        out
    }
}
