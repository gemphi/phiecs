#[allow(unused_imports)]
use super::*;

impl Facet {

    /// Pointwise mutual information of an adjacent pair, in nats.
    ///
    /// High PMI marks a pair whose meaning is not the sum of its parts —
    /// `hot dog`, `borrow checker` — which is exactly where an additive
    /// sentence representation fails and a multiplicative binding is wanted.
    pub fn pmi(&self, word_a: &str, word_b: &str) -> f64 {
        let joint = self.bigram_stats(word_a, word_b).map(|s| s.count).unwrap_or(0) as f64;
        if joint <= 0.0 {
            return f64::NEG_INFINITY;
        }
        let total: f64 = self
            .bigrams
            .values()
            .map(|l| l.iter().map(|(_, c)| *c as f64).sum::<f64>())
            .sum();
        if total <= 0.0 {
            return f64::NEG_INFINITY;
        }
        let ca = self.bigram_stats(word_a, word_a).map(|s| s.total).unwrap_or(0) as f64;
        let cb: f64 = self
            .vocab
            .id(word_b)
            .map(|bid| {
                self.bigrams
                    .values()
                    .map(|l| lookup(l, bid) as f64)
                    .sum::<f64>()
            })
            .unwrap_or(0.0);
        if ca <= 0.0 || cb <= 0.0 {
            return f64::NEG_INFINITY;
        }
        ((joint / total) / ((ca / total) * (cb / total))).ln()
    }

    /// Total number of stored n-gram entries, across both tables.
    pub fn ngram_entries(&self) -> usize {
        self.bigrams.values().map(|l| l.len()).sum::<usize>()
            + self.trigrams.values().map(|l| l.len()).sum::<usize>()
    }

    /// Directional asymmetry of a word pair, from counts alone, in [-1, 1].
    ///
    /// `+1` means `b` only ever follows `a`; `-1` the reverse; `0` that the two
    /// orders are equally common. This is the syntactic fact β is meant to
    /// encode, and it is a property of the corpus rather than of the manifold.
    pub fn order_asymmetry(&self, word_a: &str, word_b: &str) -> f64 {
        let fwd = self.bigram_stats(word_a, word_b).map(|s| s.count).unwrap_or(0) as f64;
        let rev = self.bigram_stats(word_b, word_a).map(|s| s.count).unwrap_or(0) as f64;
        match fwd + rev > 0.0 {
            true => (fwd - rev) / (fwd + rev),
            false => 0.0,
        }
    }

    /// The lag β_ij *should* take for this pair, anchored outside the geometry.
    ///
    /// The observed lag was previously measured as `θ_b − θ_a` — the difference
    /// between the very phases the β term is pushing around. β chased the
    /// phases, the phases were moved by β, and nothing outside the loop held it
    /// in place, so it converged on zero: the collapse fixed point expressed in
    /// the syntax layer. Anchoring to count asymmetry gives it something to
    /// converge *to*.
    pub fn target_phase_lag(&self, word_a: &str, word_b: &str) -> f64 {
        crate::phasor::facet::constants::SYNTACTIC_LAG_BETA * self.order_asymmetry(word_a, word_b)
    }

    /// Records an observed word-order lag and blends it into β_ij.
    pub fn record_phase_lag(&mut self, word_a: &str, word_b: &str, observed: f64) {
        if word_a == word_b {
            return;
        }
        let (a, b) = (self.vocab.intern(word_a), self.vocab.intern(word_b));
        let entry = self
            .phase_lags
            .entry((a, b))
            .or_insert(crate::phasor::facet::constants::SYNTACTIC_LAG_BETA as f32);
        let rate = crate::phasor::facet::constants::SYNTAX_LAG_LEARN_RATE;
        *entry = ((*entry as f64 * (1.0 - rate) + observed * rate)
            .rem_euclid(phiton::units::TWO_PI)) as f32;
    }

    /// Returns the learned syntactic lag β_ij, or the default subject→verb lag.
    pub fn phase_lag(&self, word_a: &str, word_b: &str) -> f64 {
        match (self.vocab.id(word_a), self.vocab.id(word_b)) {
            (Some(a), Some(b)) => self
                .phase_lags
                .get(&(a, b))
                .map(|v| *v as f64)
                .unwrap_or(crate::phasor::facet::constants::SYNTACTIC_LAG_BETA),
            _ => crate::phasor::facet::constants::SYNTACTIC_LAG_BETA,
        }
    }
}

impl Default for Facet {
    fn default() -> Self {
        Self::new()
    }
}
