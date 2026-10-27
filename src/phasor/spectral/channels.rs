#[allow(unused_imports)]
use super::*;

impl SpectralPhasor {
    /// Creates a phasor with the given phase, amplitude, and band level.
    ///
    /// Channels are spread from the base phase by the golden angle, which is the
    /// most uniform deterministic spread available on a circle. Prefer
    /// [`SpectralPhasor::seeded`] for lexicon entries: it gives channels that
    /// carry independent information rather than a fixed function of one number.
    pub fn new(phase: f64, amplitude: f64, band_n: u32) -> Self {
        let mut p = Self {
            phase: phase.rem_euclid(TWO_PI),
            amplitude,
            band_n,
            count: 1,
            packed: [0; PACKED_WORDS],
            evid: [0; EVIDENCE_WORDS],
            tgt: [0; PACKED_WORDS],
        };
        for k in 0..PHASE_CHANNELS {
            p.set_theta(k, phase + (k as f64) * GOLDEN_ANGLE);
        }
        p.set_theta(0, p.phase);
        p.tgt = p.packed;
        p
    }

    /// Creates a phasor seeded from a word's identity.
    ///
    /// Every channel is derived from an independent mix of the word's hash, so
    /// two distinct words occupy distinct positions in every channel. This
    /// replaces seeding by `word.len() * PHI`, under which all words of the same
    /// character length shared one exact starting angle — roughly twenty
    /// distinct positions for an entire vocabulary.
    pub fn seeded(word: &str, amplitude: f64, band_n: u32) -> Self {
        // The one seed (docs/159): base phase and per-channel angles come from
        // phinet, so this word is born at the same angles as its phiecs phasor.
        let base = phistd::seed_phase(word);
        let mut p = Self {
            phase: base.rem_euclid(TWO_PI),
            amplitude,
            band_n,
            count: 1,
            packed: [0; PACKED_WORDS],
            evid: [0; EVIDENCE_WORDS],
            tgt: [0; PACKED_WORDS],
        };
        for (k, angle) in phistd::seed_angles(word, PHASE_CHANNELS).into_iter().enumerate() {
            p.set_theta(k, angle);
        }
        p.set_theta(0, p.phase);
        p.tgt = p.packed;
        p
    }

    /// The raw packed channel words, for compact serialization.
    #[inline]
    pub fn packed(&self) -> [u64; PACKED_WORDS] {
        self.packed
    }

    /// Rebuilds a phasor from its packed channels.
    ///
    /// `phase` is not stored on disk: it is by construction the angle of
    /// channel 0, so it is recovered rather than duplicated.
    pub fn from_packed(packed: [u64; PACKED_WORDS], amplitude: f64, band_n: u32, count: u32) -> Self {
        let mut p = Self { phase: 0.0, amplitude, band_n, count, packed, evid: [0; EVIDENCE_WORDS], tgt: packed };
        p.sync_phase();
        p
    }

    /// Rebuilds a phasor from its packed channels and packed evidence.
    pub fn from_packed_with_evidence(
        packed: [u64; PACKED_WORDS],
        evid: [u64; EVIDENCE_WORDS],
        amplitude: f64,
        band_n: u32,
        count: u32,
    ) -> Self {
        let mut p = Self { phase: 0.0, amplitude, band_n, count, packed, evid, tgt: packed };
        p.sync_phase();
        p
    }

    /// Rebuilds a phasor with both sides and evidence (format v5).
    pub fn from_packed_sides(
        packed: [u64; PACKED_WORDS],
        tgt: [u64; PACKED_WORDS],
        evid: [u64; EVIDENCE_WORDS],
        amplitude: f64,
        band_n: u32,
        count: u32,
    ) -> Self {
        let mut p = Self { phase: 0.0, amplitude, band_n, count, packed, evid, tgt };
        p.sync_phase();
        p
    }

    /// The raw packed target-side words, for compact serialization.
    #[inline]
    pub fn packed_target(&self) -> [u64; PACKED_WORDS] {
        self.tgt
    }

    /// Target-side angle of channel `k`: the side this word shows when it is
    /// the one related to.
    #[inline]
    pub fn theta_target(&self, k: usize) -> f64 {
        self.target_byte(k) as f64 * TWO_PI / CHANNEL_QUANTA
    }

    /// The stored target side, untied (what training moves).
    #[inline]
    pub fn theta_target_raw(&self, k: usize) -> f64 {
        let k = k % PHASE_CHANNELS;
        let byte = (self.tgt[k / 8] >> (8 * (k % 8))) & 0xFF;
        byte as f64 * TWO_PI / CHANNEL_QUANTA
    }

    // `TARGET_TIE_FLOOR` is 0 (the tie is disabled but kept), so the floor
    // comparison is always false by design.
    #[inline]
    #[allow(clippy::absurd_extreme_comparisons)]
    pub(super) fn target_byte(&self, k: usize) -> u8 {
        let k = k % PHASE_CHANNELS;
        if self.evidence_count(k) < TARGET_TIE_FLOOR {
            return self.theta_byte(k);
        }
        ((self.tgt[k / 8] >> (8 * (k % 8))) & 0xFF) as u8
    }

    /// Sets the target-side angle of channel `k`.
    #[inline]
    pub fn set_theta_target(&mut self, k: usize, theta: f64) {
        let k = k % PHASE_CHANNELS;
        let q = ((theta.rem_euclid(TWO_PI) / TWO_PI) * CHANNEL_QUANTA)
            .round()
            .rem_euclid(CHANNEL_QUANTA) as u64;
        let shift = 8 * (k % 8);
        let w = k / 8;
        self.tgt[w] = (self.tgt[w] & !(0xFFu64 << shift)) | (q << shift);
    }

    /// Rotates the target side of channel `k` by `delta` radians.
    #[inline]
    pub fn nudge_target(&mut self, k: usize, delta: f64) {
        let t = self.theta_target_raw(k);
        self.set_theta_target(k, t + delta);
    }

    /// True when the two sides are still identical on every channel — a
    /// phasor that has never been through a directed coupling.
    #[inline]
    pub fn sides_equal(&self) -> bool {
        self.tgt == self.packed
    }

    /// The raw packed evidence words, for compact serialization.
    #[inline]
    pub fn packed_evidence(&self) -> [u64; EVIDENCE_WORDS] {
        self.evid
    }

    // ── evidence ─────────────────────────────────────────────────────────

    /// Raw observation count for relation channel `k`.
    #[inline]
    pub fn evidence_count(&self, k: usize) -> u32 {
        let k = k % PHASE_CHANNELS;
        ((self.evid[k / 2] >> (32 * (k % 2))) & 0xFFFF_FFFF) as u32
    }
}
