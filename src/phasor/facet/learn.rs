#[allow(unused_imports)]
use super::*;

impl Facet {
    /// Creates an empty facet with no words.
    pub fn new() -> Self {
        Self {
            lexicon: HashMap::new(),
            vocab: Vocab::default(),
            bigrams: HashMap::new(),
            trigrams: HashMap::new(),
            phase_lags: HashMap::new(),
            grounded_version: 0,
            relations: RelationVocab::default(),
            sample_pool: Vec::new(),
            relation_pool: Vec::new(),
        }
    }

    /// Learns the relation inventory — which between-word markers get channels
    /// — from a tokenised corpus. Idempotent on a facet that already has one.
    pub fn learn_relations(&mut self, docs: &[Vec<String>]) -> usize {
        if self.relations.is_empty() {
            self.relations = RelationVocab::learn(docs);
        }
        self.relations.markers()
    }

    /// Per-channel weight that equalises how loudly each relation votes.
    ///
    /// Traffic is wildly unequal: every adjacent pair feeds an offset channel,
    /// while a marker channel only sees pairs joined by that one word. Left raw,
    /// the offset channels carry orders of magnitude more evidence and decide
    /// every comparison. Dividing by each channel's mean evidence makes a
    /// channel's vote proportional to how much *this word* uses it relative to
    /// how much anyone does, rather than to how common the relation is.
    ///
    /// Channels no word has evidence for weigh zero. `O(V · 64)`; compute once
    /// per evaluation, not per pair.
    pub fn channel_balance(&self) -> [f64; PHASE_CHANNELS] {
        let mut mean = [0.0f64; PHASE_CHANNELS];
        let mut n = 0usize;
        for p in self.lexicon.values() {
            if !p.has_evidence() {
                continue;
            }
            n += 1;
            for (k, m) in mean.iter_mut().enumerate() {
                *m += p.evidence(k);
            }
        }
        let n = n.max(1) as f64;
        let mut w = [0.0f64; PHASE_CHANNELS];
        for k in 0..PHASE_CHANNELS {
            let m = mean[k] / n;
            w[k] = if m > 1e-9 { 1.0 / m } else { 0.0 };
        }
        w
    }

    /// Circular dispersion of channel `k` over the words with evidence on it.
    ///
    /// 1.0 is uniform around the circle, 0.0 is every word at one angle. A
    /// relation channel every pair feeds is the one most likely to collapse,
    /// after which it agrees about everything and votes loudly for nothing.
    pub fn channel_dispersion(&self, k: usize) -> f64 {
        let (mut x, mut y, mut n) = (0.0f64, 0.0f64, 0usize);
        for p in self.lexicon.values() {
            if p.evidence_count(k) > 0 {
                let t = p.theta(k);
                x += t.cos();
                y += t.sin();
                n += 1;
            }
        }
        match n {
            0 => 1.0,
            _ => 1.0 - x.hypot(y) / n as f64,
        }
    }

    /// Number of words carrying any relation evidence.
    pub fn evidenced_words(&self) -> usize {
        self.lexicon.values().filter(|p| p.has_evidence()).count()
    }

    /// Evidence-weighted similarity between two words, balanced per channel.
    ///
    /// Prefer [`SpectralPhasor::similarity_with`] with a cached
    /// [`Facet::channel_balance`] when comparing many pairs.
    pub fn similarity(&self, word_a: &str, word_b: &str) -> f64 {
        match (self.lexicon.get(word_a), self.lexicon.get(word_b)) {
            (Some(a), Some(b)) => {
                let w = self.channel_balance();
                a.similarity_with(b, Some(&w))
            }
            _ => 0.0,
        }
    }

    /// The relational score of a candidate next word given its context: the
    /// engine's own directed reading, pair by pair.
    ///
    /// Each context word `c_j` is a wheel; the condition between `c_j` and the
    /// candidate's position (offset, or the marker word between them) selects
    /// the channel `k_j` — the gear — and the candidate's target wheel on that
    /// channel is read against `c_j`'s source wheel: `on(c_j, cand, k_j)`.
    /// Nothing is averaged into a context vector first; every pair keeps its
    /// own channel, which is what the two-sided lexicon is for. Returns the
    /// mean agreement over the pairs that carry evidence and how many did.
    /// A pair without evidence is silent, not neutral: the score is exactly
    /// what the corpus has shown, and nothing else is in play.
    pub fn relational_score(&self, ctx: &[String], cand: &str) -> (f64, usize) {
        let cp = match self.lexicon.get(cand) {
            Some(p) => p,
            None => return (0.0, 0),
        };
        let mut toks: Vec<String> = ctx.to_vec();
        toks.push(cand.to_string());
        let i = toks.len() - 1;
        let (mut sum, mut n) = (0.0f64, 0usize);
        for j in 0..i {
            let k = match self.relations.channel(&toks, j, i) {
                Some(k) => k,
                None => continue,
            };
            if let Some(a) = self.lexicon.get(&toks[j]) {
                if a.evidence_count(k) > 0 && cp.evidence_count(k) > 0 {
                    sum += a.on(cp, k);
                    n += 1;
                }
            }
        }
        match n {
            0 => (0.0, 0),
            _ => (sum / n as f64, n),
        }
    }
}
