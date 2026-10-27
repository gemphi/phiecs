#[allow(unused_imports)]
use super::*;

impl Wave {
    /// Returns the zero wave (no amplitude, no phase).
    pub fn zero() -> c64 {
        c64::new(0.0, 0.0)
    }

    /// Sums an iterator of complex waves into a single superposition wave.
    pub fn from_sum(iter: impl Iterator<Item = c64>) -> c64 {
        iter.sum()
    }

    /// Unordered superposition of a list of known words.
    ///
    /// This is a bag: `Z(a b) == Z(b a)`. That is the correct representation for
    /// measuring *agreement* among words — which is what coherence asks — but
    /// not for representing *what was said*. Use [`Wave::sentence_bound`] for
    /// anything where order carries meaning.
    pub fn sentence(facet: &Facet, words: &[String]) -> c64 {
        words
            .iter()
            .filter_map(|w| facet.lexicon.get(w))
            .map(|p| p.to_complex())
            .sum()
    }

    /// Order-sensitive superposition: each word is bound to its position.
    ///
    /// Position `i` rotates the word by `i * GOLDEN_ANGLE` before summing. This
    /// is circular-convolution binding in the phase domain, and the golden angle
    /// is used because its irrationality means no two positions ever collide.
    ///
    /// `Z("dog bites man") != Z("man bites dog")`, which a plain sum cannot
    /// express — and without which negation scope, argument roles and causal
    /// direction are all unrepresentable.
    pub fn sentence_bound(facet: &Facet, words: &[String]) -> c64 {
        words
            .iter()
            .enumerate()
            .filter_map(|(i, w)| facet.lexicon.get(w).map(|p| (i, p)))
            .map(|(i, p)| c64::from_polar(p.amplitude, p.effective_phase() + i as f64 * GOLDEN_ANGLE))
            .sum()
    }

    /// Per-channel superposition across the full phase torus.
    ///
    /// Returns `PHASE_CHANNELS` complex numbers — the multi-channel counterpart
    /// of [`Wave::sentence`]. Set `bound` to make it order-sensitive.
    pub fn sentence_channels(facet: &Facet, words: &[String], bound: bool) -> Vec<c64> {
        let mut out = vec![Self::zero(); PHASE_CHANNELS];
        for (i, w) in words.iter().enumerate() {
            if let Some(p) = facet.lexicon.get(w) {
                let roll = if bound { i as f64 * GOLDEN_ANGLE } else { 0.0 };
                for (k, z) in out.iter_mut().enumerate() {
                    *z += c64::from_polar(p.amplitude, p.theta(k) + roll);
                }
            }
        }
        out
    }

    /// Order-sensitive superposition that **binds tightly-bound pairs
    /// multiplicatively** instead of adding them.
    ///
    /// A sum is linear, so the representation of `hot dog` is fully determined
    /// by `hot` and `dog` independently and idioms, compounds and metaphors have
    /// no home in it. Multiplying two phasors adds their phases, producing a
    /// point determined by both words *jointly* and distinct from either.
    ///
    /// Pairs are selected by pointwise mutual information from the bigram table
    /// already collected, so nothing new has to be annotated.
    pub fn sentence_compound(facet: &Facet, words: &[String], pmi_threshold: f64) -> c64 {
        let mut out = Self::zero();
        let mut i = 0usize;
        let mut slot = 0usize;

        while i < words.len() {
            let bound = i + 1 < words.len()
                && facet.pmi(&words[i], &words[i + 1]) >= pmi_threshold;

            let roll = slot as f64 * GOLDEN_ANGLE;
            match bound {
                true => {
                    // z_a · z_b — amplitudes multiply, phases add.
                    if let (Some(a), Some(b)) =
                        (facet.lexicon.get(&words[i]), facet.lexicon.get(&words[i + 1]))
                    {
                        out += c64::from_polar(
                            a.amplitude * b.amplitude,
                            a.effective_phase() + b.effective_phase() + roll,
                        );
                    }
                    i += 2;
                }
                false => {
                    if let Some(p) = facet.lexicon.get(&words[i]) {
                        out += c64::from_polar(p.amplitude, p.effective_phase() + roll);
                    }
                    i += 1;
                }
            }
            slot += 1;
        }
        out
    }

    /// Computes the wave for a raw text string (unordered).
    pub fn text(facet: &Facet, text: &str) -> c64 {
        let tokens = Tokenizer::tokenize(text);
        Self::sentence(facet, &tokens)
    }

    /// Computes the order-sensitive wave for a raw text string.
    pub fn text_bound(facet: &Facet, text: &str) -> c64 {
        let tokens = Tokenizer::tokenize(text);
        Self::sentence_bound(facet, &tokens)
    }

    /// Ray cast: finds words that resonate with a target word.
    ///
    /// Ranks by mean phase coherence across all channels, blended with
    /// familiarity. The previous metric, `α·|Z_q − Z_w|²`, expands to
    /// `A_q² + A_w² − 2A_qA_w cos Δφ`, so a word with an amplitude close to the
    /// query's could outrank an exact phase match; and the leading α, being a
    /// positive constant on every candidate, could not affect the ranking at all.
    ///
    /// Returns `(word, delta)` sorted ascending, where delta = 1 − score, so
    /// smaller is still better for every existing caller.
    /// Minimum training count for a word to be offered as a retrieval result.
    ///
    /// A word seen once or twice still sits essentially at its hash seed, and a
    /// hash seed is a function of spelling. That is why `synonym cow` on a
    /// 77,000-word dictionary manifold returned `cod, cot, coz, cos` — not a
    /// semantic failure but a pool full of words the model had never moved.
    /// Every relational benchmark in this project applies a frequency floor for
    /// the same reason; retrieval had never had one.
    pub const RETRIEVAL_COUNT_FLOOR: u32 = 5;

    /// The floor to actually use, backed off when it would empty the pool.
    ///
    /// A freshly bootstrapped or small facet can have almost nothing above the
    /// floor, and returning nothing is worse than returning weak matches. The
    /// floor applies only while it leaves a pool several times the requested
    /// result count.
    pub(super) fn pool_floor(facet: &Facet, top_k: usize) -> u32 {
        let above = facet
            .lexicon
            .values()
            .filter(|p| p.count >= Self::RETRIEVAL_COUNT_FLOOR)
            .count();
        match above >= top_k.max(1) * 10 {
            true => Self::RETRIEVAL_COUNT_FLOOR,
            false => 0,
        }
    }

    pub fn ray_cast_word(facet: &Facet, target_word: &str, top_k: usize) -> Vec<(String, f64)> {
        let target = match facet.lexicon.get(target_word) {
            Some(p) => *p,
            None => return vec![],
        };

        let floor = Self::pool_floor(facet, top_k);
        let ceiling = Self::pool_ceiling(facet);
        let mut hits: Vec<(&String, f64)> = facet
            .lexicon
            .par_iter()
            .filter(|(word, p)| *word != target_word && p.count >= floor && p.count <= ceiling)
            .map(|(word, phasor)| {
                let sim = 0.5 * (target.resonance(phasor) + 1.0);
                (word, 1.0 - sim)
            })
            .collect();

        Self::take_smallest(&mut hits, top_k)
    }
}
