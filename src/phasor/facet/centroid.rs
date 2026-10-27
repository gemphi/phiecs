#[allow(unused_imports)]
use super::*;

impl Facet {

    /// Computes the centroid wave - the sum of all phasor complex representations.
    pub fn centroid(&self) -> c64 {
        if self.lexicon.is_empty() {
            return Wave::zero();
        }
        Wave::from_sum(self.lexicon.values().map(|p| p.to_complex()))
    }

    // ── n-gram statistics ──────────────────────────────────────────────────

    /// Records a bigram (word_a, word_b) co-occurrence.
    pub fn record_bigram(&mut self, word_a: &str, word_b: &str) {
        if word_a == word_b {
            return;
        }
        let (a, b) = (self.vocab.intern(word_a), self.vocab.intern(word_b));
        bump(self.bigrams.entry(a).or_default(), b);
    }

    /// Records a trigram (word_a, word_b, word_c) co-occurrence.
    pub fn record_trigram(&mut self, word_a: &str, word_b: &str, word_c: &str) {
        if word_a == word_c || word_b == word_c {
            return;
        }
        let a = self.vocab.intern(word_a);
        let b = self.vocab.intern(word_b);
        let c = self.vocab.intern(word_c);
        bump(self.trigrams.entry((a, b)).or_default(), c);
    }

    /// Candidate next words given a current word, with their counts.
    /// Only returns words that exist in the lexicon.
    pub fn next_word_candidates(&self, current: &str) -> Vec<(String, u32)> {
        let id = match self.vocab.id(current) {
            Some(i) => i,
            None => return Vec::new(),
        };
        match self.bigrams.get(&id) {
            None => Vec::new(),
            Some(list) => list
                .iter()
                .filter_map(|(wid, c)| self.vocab.word(*wid).map(|w| (w, *c)))
                .filter(|(w, _)| self.lexicon.contains_key(*w))
                .map(|(w, c)| (w.to_string(), c))
                .collect(),
        }
    }

    /// Candidate next words given two preceding words.
    ///
    /// The context key is a `(u32, u32)` tuple, so this no longer allocates a
    /// joined `String` on every call — which, in the decode loop, was one heap
    /// allocation per candidate per generated token.
    pub fn trigram_candidates(&self, word_a: &str, word_b: &str) -> Vec<(String, u32)> {
        let key = match (self.vocab.id(word_a), self.vocab.id(word_b)) {
            (Some(a), Some(b)) => (a, b),
            _ => return Vec::new(),
        };
        match self.trigrams.get(&key) {
            None => Vec::new(),
            Some(list) => list
                .iter()
                .filter_map(|(wid, c)| self.vocab.word(*wid).map(|w| (w, *c)))
                .filter(|(w, _)| self.lexicon.contains_key(*w))
                .map(|(w, c)| (w.to_string(), c))
                .collect(),
        }
    }

    /// Raw maximum-likelihood trigram probability. Returns 0.0 for unseen
    /// continuations; prefer [`Facet::trigram_discounted`] for scoring.
    pub fn trigram_probability(&self, word_a: &str, word_b: &str, word_c: &str) -> f64 {
        match self.trigram_stats(word_a, word_b, word_c) {
            Some(s) if s.total > 0 => s.count as f64 / s.total as f64,
            _ => 0.0,
        }
    }

    /// Raw maximum-likelihood bigram probability.
    pub fn bigram_probability(&self, word_a: &str, word_b: &str) -> f64 {
        match self.bigram_stats(word_a, word_b) {
            Some(s) if s.total > 0 => s.count as f64 / s.total as f64,
            _ => 0.0,
        }
    }

    /// Tally for one bigram continuation, or `None` if the context is unseen.
    pub fn bigram_stats(&self, word_a: &str, word_b: &str) -> Option<NgramStat> {
        let a = self.vocab.id(word_a)?;
        let list = self.bigrams.get(&a)?;
        let b = self.vocab.id(word_b);
        Some(NgramStat {
            count: b.map(|b| lookup(list, b)).unwrap_or(0),
            total: total_of(list),
            types: list.len() as u32,
        })
    }

    /// Tally for one trigram continuation, or `None` if the context is unseen.
    pub fn trigram_stats(&self, word_a: &str, word_b: &str, word_c: &str) -> Option<NgramStat> {
        let key = (self.vocab.id(word_a)?, self.vocab.id(word_b)?);
        let list = self.trigrams.get(&key)?;
        let c = self.vocab.id(word_c);
        Some(NgramStat {
            count: c.map(|c| lookup(list, c)).unwrap_or(0),
            total: total_of(list),
            types: list.len() as u32,
        })
    }

    /// Absolute discount used by the smoothed n-gram estimators.
    const DISCOUNT: f64 = 0.75;

    /// Smoothed bigram probability as `(discounted, backoff_mass)`.
    ///
    /// The raw estimator returns exactly 0.0 for an unseen continuation, which
    /// makes held-out likelihood infinite and the model brittle off
    /// distribution. Callers distribute the back-off mass over whatever base
    /// distribution they prefer.
    pub fn bigram_discounted(&self, word_a: &str, word_b: &str) -> (f64, f64) {
        match self.bigram_stats(word_a, word_b) {
            Some(s) if s.total > 0 => {
                let lambda = Self::DISCOUNT * s.types as f64 / s.total as f64;
                (((s.count as f64 - Self::DISCOUNT).max(0.0)) / s.total as f64, lambda)
            }
            _ => (0.0, 1.0),
        }
    }

    /// Smoothed trigram probability as `(discounted, backoff_mass)`.
    pub fn trigram_discounted(&self, word_a: &str, word_b: &str, word_c: &str) -> (f64, f64) {
        match self.trigram_stats(word_a, word_b, word_c) {
            Some(s) if s.total > 0 => {
                let lambda = Self::DISCOUNT * s.types as f64 / s.total as f64;
                (((s.count as f64 - Self::DISCOUNT).max(0.0)) / s.total as f64, lambda)
            }
            _ => (0.0, 1.0),
        }
    }

    /// Drops n-grams seen only once, and any context left empty.
    ///
    /// **This is a size/quality trade, not a free win.** Measured on the Rust
    /// Book corpus (7,757 sentences, 6,016 vocabulary): the table shrinks 80.7%
    /// — 136,807 entries to 26,338 — and held-out perplexity worsens 81%,
    /// from 148.92 to 269.89.
    ///
    /// The reason is corpus size. On a small corpus most n-grams *are*
    /// singletons and they carry most of the coverage, so discarding them
    /// discards the model. Prefer vocabulary interning for footprint, which is
    /// lossless.
    ///
    /// Returns `(bigrams_dropped, trigrams_dropped)`.
    pub fn prune_singletons(&mut self) -> (usize, usize) {
        let mut bi_dropped = 0usize;
        for list in self.bigrams.values_mut() {
            let before = list.len();
            list.retain(|(_, c)| *c > 1);
            bi_dropped += before - list.len();
        }
        self.bigrams.retain(|_, l| !l.is_empty());

        let mut tri_dropped = 0usize;
        for list in self.trigrams.values_mut() {
            let before = list.len();
            list.retain(|(_, c)| *c > 1);
            tri_dropped += before - list.len();
        }
        self.trigrams.retain(|_, l| !l.is_empty());

        self.phase_lags.retain(|(a, _), _| self.bigrams.contains_key(a));

        (bi_dropped, tri_dropped)
    }
}
