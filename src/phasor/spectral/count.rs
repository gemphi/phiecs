#[allow(unused_imports)]
use super::*;

impl SpectralPhasor {

    /// Sets the raw count for channel `k`, saturating at 16 bits.
    #[inline]
    pub fn set_evidence_count(&mut self, k: usize, count: u32) {
        let k = k % PHASE_CHANNELS;
        let c = (count as u64).min(EVIDENCE_MAX);
        let shift = 32 * (k % 2);
        let w = k / 2;
        self.evid[w] = (self.evid[w] & !(0xFFFF_FFFFu64 << shift)) | (c << shift);
    }

    /// Records `n` more observations of this word under relation `k`.
    #[inline]
    pub fn deposit(&mut self, k: usize, n: u32) {
        let c = self.evidence_count(k).saturating_add(n);
        self.set_evidence_count(k, c);
    }

    /// Evidence for relation `k` on a compressed scale: `ln(1 + count)`.
    ///
    /// A count would let one very frequent relation decide every comparison a
    /// word takes part in; the log keeps a relation seen ten thousand times
    /// from outvoting one seen a hundred times by more than about 2:1.
    #[inline]
    pub fn evidence(&self, k: usize) -> f64 {
        (1.0 + self.evidence_count(k) as f64).ln()
    }

    /// Total compressed evidence across all channels. Zero means the word has
    /// never been observed under any relation, and every weighted comparison
    /// falls back to the plain mean-cosine [`SpectralPhasor::resonance`].
    pub fn evidence_total(&self) -> f64 {
        (0..PHASE_CHANNELS).map(|k| self.evidence(k)).sum()
    }

    /// True if any channel carries evidence.
    #[inline]
    pub fn has_evidence(&self) -> bool {
        self.evid.iter().any(|w| *w != 0)
    }

    /// Clears all evidence, leaving the phases untouched.
    pub fn clear_evidence(&mut self) {
        self.evid = [0; EVIDENCE_WORDS];
    }

    /// The channel this word has the most evidence for, with that evidence.
    pub fn dominant_relation(&self) -> (usize, f64) {
        let mut best = (0usize, 0.0f64);
        for k in 0..PHASE_CHANNELS {
            let e = self.evidence(k);
            if e > best.1 {
                best = (k, e);
            }
        }
        best
    }

    /// Evidence-weighted similarity: a normalised complex inner product where
    /// the magnitude on channel `k` is each word's evidence for relation `k`.
    ///
    /// A channel where either word has no evidence contributes nothing, so two
    /// words are compared only on the relations they both participate in. That
    /// is the behaviour the plain `resonance` cannot have: averaging 64 cosines
    /// gives a channel the same vote whether the word was ever seen there or not.
    ///
    /// Falls back to [`SpectralPhasor::resonance`] when either side has no
    /// evidence at all, so a lexicon that never deposited any compares exactly
    /// as it used to. `weights` rescales each channel's vote — pass the facet's
    /// channel balance so that a relation every pair feeds does not outvote one
    /// that only a few do; pass `None` for the raw product.
    pub fn similarity_with(&self, other: &Self, weights: Option<&[f64]>) -> f64 {
        if !self.has_evidence() || !other.has_evidence() {
            return self.resonance(other);
        }
        let (mut dot, mut na, mut nb) = (0.0f64, 0.0f64, 0.0f64);
        for k in 0..PHASE_CHANNELS {
            let w = weights.map_or(1.0, |w| w[k]);
            let a = self.evidence(k) * w;
            let b = other.evidence(k) * w;
            if a <= 0.0 || b <= 0.0 {
                continue;
            }
            // Two words are alike when they present the same sides: as the
            // relating word (source) and as the related-to word (target).
            let src = COS_BYTE[self.theta_byte(k).wrapping_sub(other.theta_byte(k)) as usize];
            let tgt = COS_BYTE[self.target_byte(k).wrapping_sub(other.target_byte(k)) as usize];
            dot += a * b * 0.5 * (src + tgt);
            na += a * a;
            nb += b * b;
        }
        match na > 1e-12 && nb > 1e-12 {
            true => (dot / (na.sqrt() * nb.sqrt())).clamp(-1.0, 1.0),
            false => self.resonance(other),
        }
    }

    /// [`SpectralPhasor::similarity_with`] with unit channel weights.
    #[inline]
    pub fn similarity(&self, other: &Self) -> f64 {
        self.similarity_with(other, None)
    }

    /// Agreement on relation `k` alone: does `self`, as the relating word,
    /// point at `other`, as the word related to?
    ///
    /// Returns `cos(θ_src^a[k] − θ_tgt^b[k])`, and zero when either side has
    /// no evidence on `k`: a channel neither word was ever observed under has
    /// nothing to say about the pair. Directed: `a.on(b, k)` is "a then b".
    pub fn on(&self, other: &Self, k: usize) -> f64 {
        let w = (self.evidence(k) * other.evidence(k)).sqrt();
        match w > 1e-12 {
            true => {
                let da = self.theta_byte(k);
                let db = other.target_byte(k);
                COS_BYTE[da.wrapping_sub(db) as usize]
            }
            false => 0.0,
        }
    }

    /// The channel on which this pair has the most shared evidence — the
    /// relation the corpus most often placed these two words under.
    ///
    /// Raw evidence is dominated by adjacency: every pair of neighbours feeds
    /// an offset channel, so without `weights` the answer is nearly always
    /// `offset+1`. Pass phiano's `Facet::channel_balance`
    /// to ask which relation this pair uses *more than the vocabulary does*.
    pub fn shared_relation_with(&self, other: &Self, weights: Option<&[f64]>) -> Option<usize> {
        let mut best: Option<(usize, f64)> = None;
        for k in 0..PHASE_CHANNELS {
            let w = weights.map_or(1.0, |w| w[k]);
            let v = self.evidence(k) * other.evidence(k) * w * w;
            if v > 0.0 && best.is_none_or(|(_, b)| v > b) {
                best = Some((k, v));
            }
        }
        best.map(|(k, _)| k)
    }

    /// [`SpectralPhasor::shared_relation_with`] on raw evidence.
    pub fn shared_relation(&self, other: &Self) -> Option<usize> {
        self.shared_relation_with(other, None)
    }

    /// Number of independent phase channels.
    #[inline]
    pub const fn channels() -> usize {
        PHASE_CHANNELS
    }

    /// Returns the phase angle of channel `k`, in radians [0, 2*pi).
    #[inline]
    pub fn theta(&self, k: usize) -> f64 {
        let k = k % PHASE_CHANNELS;
        let byte = (self.packed[k / 8] >> (8 * (k % 8))) & 0xFF;
        byte as f64 * TWO_PI / CHANNEL_QUANTA
    }

    /// Sets the phase angle of channel `k`, wrapping into [0, 2*pi).
    #[inline]
    pub fn set_theta(&mut self, k: usize, theta: f64) {
        let k = k % PHASE_CHANNELS;
        let q = ((theta.rem_euclid(TWO_PI) / TWO_PI) * CHANNEL_QUANTA)
            .round()
            .rem_euclid(CHANNEL_QUANTA) as u64;
        let shift = 8 * (k % 8);
        let w = k / 8;
        self.packed[w] = (self.packed[w] & !(0xFFu64 << shift)) | (q << shift);
        if k == 0 {
            self.phase = self.theta(0);
        }
    }
}
