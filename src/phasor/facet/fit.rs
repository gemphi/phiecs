#[allow(unused_imports)]
use super::*;

impl Facet {

    /// The Liskov reading: how well `cand` stands in at position `i` of
    /// `toks` — read against every neighbour on both sides, each under the
    /// condition the sentence actually puts between them.
    ///
    /// Liskov's substitution principle says a thing may stand in for another
    /// wherever every relation the context demands still holds. For words the
    /// demands are the pairs: for a neighbour before `i` the candidate's
    /// target side is read against the neighbour's source side; for one after,
    /// the candidate's source side against the neighbour's target side.
    /// `relational_score` is this reading at the last position. Returns the
    /// mean agreement over evidenced pairs and their number.
    pub fn relational_fit(&self, toks: &[String], i: usize, cand: &str) -> (f64, usize) {
        let cp = match self.lexicon.get(cand) {
            Some(p) => p,
            None => return (0.0, 0),
        };
        let mut t: Vec<String> = toks.to_vec();
        if i >= t.len() {
            return (0.0, 0);
        }
        t[i] = cand.to_string();
        let (mut sum, mut n) = (0.0f64, 0usize);
        let lo_j = i.saturating_sub(crate::phasor::facet::constants::RELATION_WINDOW);
        let hi_j = (i + crate::phasor::facet::constants::RELATION_WINDOW).min(t.len() - 1);
        for j in lo_j..=hi_j {
            if j == i {
                continue;
            }
            let (lo, hi) = if j < i { (j, i) } else { (i, j) };
            let k = match self.relations.channel(&t, lo, hi) {
                Some(k) => k,
                None => continue,
            };
            if let Some(o) = self.lexicon.get(&t[j]) {
                if o.evidence_count(k) > 0 && cp.evidence_count(k) > 0 {
                    sum += if j < i { o.on(cp, k) } else { cp.on(o, k) };
                    n += 1;
                }
            }
        }
        match n {
            0 => (0.0, 0),
            _ => (sum / n as f64, n),
        }
    }

    /// How much `word` calls for `goal` somewhere after it: the mean directed
    /// agreement over the offset channels that carry evidence on both sides.
    /// The endgame reading — a move is judged by the mate it moves toward.
    pub fn calls_for(&self, word: &str, goal: &str) -> f64 {
        let (a, g) = match (self.lexicon.get(word), self.lexicon.get(goal)) {
            (Some(a), Some(g)) => (a, g),
            _ => return 0.0,
        };
        let (mut sum, mut n) = (0.0, 0usize);
        for k in 0..crate::relation::OFFSET_CHANNELS {
            if a.evidence_count(k) > 0 && g.evidence_count(k) > 0 {
                sum += a.on(g, k);
                n += 1;
            }
        }
        if n == 0 { 0.0 } else { sum / n as f64 }
    }

    /// Agreement between two words on relation `k` alone.
    pub fn on(&self, word_a: &str, word_b: &str, k: usize) -> f64 {
        match (self.lexicon.get(word_a), self.lexicon.get(word_b)) {
            (Some(a), Some(b)) => a.on(b, k),
            _ => 0.0,
        }
    }

    /// Human-readable name of relation channel `k`.
    pub fn relation_name(&self, k: usize) -> String {
        self.relations.name(k)
    }

    /// Returns the number of words in the lexicon.
    pub fn vocabulary_size(&self) -> usize {
        self.lexicon.len()
    }

    /// Returns true if the facet contains the given word.
    pub fn contains_word(&self, word: &str) -> bool {
        self.lexicon.contains_key(word)
    }

    /// Returns a reference to the phasor for the given word, if it exists.
    pub fn get_phasor(&self, word: &str) -> Option<&SpectralPhasor> {
        self.lexicon.get(word)
    }

    /// True when no transition statistics have been recorded yet.
    pub fn has_ngrams(&self) -> bool {
        !self.bigrams.is_empty()
    }

    /// Gets or initializes a phasor, seeded from the word's *identity*.
    ///
    /// Seeding previously used `word.len() * PHI`, which depends only on
    /// character length: `cat`, `the`, `dog` and `war` all began at exactly
    /// 4.854102 rad, and a 100k vocabulary started from about twenty distinct
    /// positions. [`SpectralPhasor::seeded`] hashes the word instead.
    pub fn get_or_init(&mut self, word: &str) -> &mut SpectralPhasor {
        self.lexicon.entry(word.to_string()).or_insert_with(|| {
            SpectralPhasor::seeded(
                word,
                crate::phasor::facet::constants::AMPLITUDE_INITIAL,
                crate::phasor::facet::constants::BAND_N_INITIAL,
            )
        })
    }

    /// Fills phase channels for any phasor loaded from a pre-multi-channel
    /// model file, preserving each word's learned base phase on channel 0.
    pub fn migrate_channels(&mut self) -> usize {
        let words: Vec<String> = self
            .lexicon
            .iter()
            .filter(|(_, p)| p.channels_unset())
            .map(|(w, _)| w.clone())
            .collect();
        for w in &words {
            if let Some(p) = self.lexicon.get_mut(w) {
                p.ensure_channels(w);
            }
        }
        words.len()
    }

    /// Rebuilds the flat sampling pool if it has drifted from the lexicon.
    pub fn rebuild_sample_pool(&mut self) {
        let target = self.lexicon.len();
        if !self.sample_pool.is_empty() && self.sample_pool.len() >= target {
            return;
        }
        // Sorted, not HashMap-iteration order.
        //
        // `lexicon` is a HashMap, and Rust seeds HashMap hashing randomly per
        // process. Building the pool by iterating it made the negative-sample
        // sequence differ on every run of the same binary on the same data, so
        // training was not reproducible and neither was anything measured on
        // top of it. Two runs of the identical composition experiment returned
        // analogy MRR 0.1066 and 0.0521 — the same ranking, magnitudes a factor
        // of two apart. An experiment whose effect size moves that much between
        // runs cannot support a claim about an effect of that size.
        //
        // Sorting costs one O(V log V) pass per rebuild and makes every
        // downstream measurement checkable.
        let mut words: Vec<(&String, &SpectralPhasor)> = self.lexicon.iter().collect();
        words.sort_unstable_by(|a, b| a.0.cmp(b.0));

        let mut pool = Vec::with_capacity(target * 2);
        for (word, phasor) in words {
            let reps = ((phasor.count as f64).sqrt().round() as usize).clamp(1, 8);
            for _ in 0..reps {
                pool.push(word.clone());
            }
        }
        self.sample_pool = pool;
    }
}
