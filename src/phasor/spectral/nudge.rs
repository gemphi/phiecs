#[allow(unused_imports)]
use super::*;

impl SpectralPhasor {

    /// Rotates channel `k` by `delta` radians, nearest rounding. See
    /// [`SpectralPhasor::nudge_by`] for the dithered form.
    #[inline]
    pub fn nudge(&mut self, k: usize, delta: f64) {
        let t = self.theta(k);
        self.set_theta(k, t + delta);
    }

    /// True if no channel has ever been written (a phasor loaded from a
    /// pre-multi-channel model file).
    #[inline]
    pub fn channels_unset(&self) -> bool {
        self.packed.iter().all(|w| *w == 0)
    }

    /// Fills the channels for a legacy phasor that has none, using the word's
    /// identity, while preserving its learned base `phase` on channel 0.
    pub fn ensure_channels(&mut self, word: &str) {
        if !self.channels_unset() {
            return;
        }
        let learned_phase = self.phase;
        let seeded = Self::seeded(word, self.amplitude, self.band_n);
        self.packed = seeded.packed;
        self.set_theta(0, learned_phase);
    }

    /// Re-derives `phase` from channel 0. Call after writing channel 0 directly.
    #[inline]
    pub fn sync_phase(&mut self) {
        self.phase = self.theta(0);
    }

    /// Writes `phase` into channel 0. Call after mutating `phase` directly, so
    /// that legacy single-angle updates stay visible to channel-aware code.
    #[inline]
    pub fn sync_channel0(&mut self) {
        let p = self.phase;
        self.set_theta(0, p);
    }

    /// Records one more observation and recomputes amplitude as a log-frequency.
    ///
    /// A = 1 + ln(count) / AMPLITUDE_LOG_SCALE, capped at AMPLITUDE_MAX. Linear
    /// increments saturated after a thousand touches, which is well below the
    /// frequency of the words the model sees most.
    #[inline]
    pub fn observe(&mut self) {
        self.count = self.count.saturating_add(1);
        self.amplitude =
            (1.0 + (self.count as f64).ln() / AMPLITUDE_LOG_SCALE).min(AMPLITUDE_MAX);
    }

    /// Mean phase coherence across all channels: (1/D) * sum_k cos(θ_k^a − θ_k^b).
    ///
    /// Ranges from 1.0 (identical in every channel) to −1.0 (antiphase in every
    /// channel). This is the multi-channel replacement for a single angular
    /// difference, and it is the similarity the torus representation exists for.
    pub fn resonance(&self, other: &Self) -> f64 {
        // Exact, not approximate. Channels are quantised to one byte
        // (`CHANNEL_QUANTA` = 256), so the angular difference between any two
        // channels is one of 256 values and its cosine is a table lookup on the
        // byte difference. Nothing is lost — the quantisation already happened
        // when the phase was stored.
        //
        // The previous version called `cos` 64 times per comparison. Resonance
        // is the innermost operation in every retrieval, every relation probe
        // and every analogy: one pass of the expanded benchmark makes roughly
        // 190 million of these calls, and 64 transcendentals each is what made
        // that pass take minutes per condition.
        let mut sum = 0.0f64;
        for w in 0..PACKED_WORDS {
            let (a, b) = (self.packed[w], other.packed[w]);
            for shift in 0..8 {
                let da = ((a >> (8 * shift)) & 0xFF) as u8;
                let db = ((b >> (8 * shift)) & 0xFF) as u8;
                sum += COS_BYTE[da.wrapping_sub(db) as usize];
            }
        }
        (sum / PHASE_CHANNELS as f64).clamp(-1.0, 1.0)
    }

    /// [`SpectralPhasor::resonance`] over one contiguous band of channels.
    ///
    /// The channel index is not a bag of interchangeable dimensions. The kernel
    /// in phiano's `config::channel_kernel` gives channel `k` a decay of
    /// `CONTEXT_LAMBDA^(1 + 3k/64)` and a frequency of `2*pi*p_k/PRIME_MODULUS`
    /// with `p_k` the k-th prime, so low channels are slow with long memory and
    /// high channels fast with short memory. That is a hierarchy of rates built
    /// into the design and never once checked.
    ///
    /// Bentov, *Stalking the Wild Pendulum*, ch. 3, "Hierarchies of motion":
    /// each level up the ladder of structures moves much more slowly than the
    /// one below — atoms at gigahertz, molecules slower, cells slower again —
    /// "a process of gradually stepping down the high vibratory rates." If
    /// language is built the same way, the levels should separate by rate:
    /// discourse in the slow channels, local word transitions in the fast ones.
    ///
    /// This is what makes that measurable: score any task on a band and see
    /// where each level peaks. Bentov's standing waves themselves, the
    /// integer modes, are `phiton::physics::standing`.
    pub fn resonance_band(&self, other: &Self, lo: usize, hi: usize) -> f64 {
        let (lo, hi) = (lo.min(PHASE_CHANNELS), hi.min(PHASE_CHANNELS));
        if hi <= lo {
            return 0.0;
        }
        let mut sum = 0.0f64;
        for k in lo..hi {
            let da = self.theta_byte(k);
            let db = other.theta_byte(k);
            sum += COS_BYTE[da.wrapping_sub(db) as usize];
        }
        (sum / (hi - lo) as f64).clamp(-1.0, 1.0)
    }

    /// The stored byte for channel `k` — the quantised phase itself.
    #[inline]
    pub fn theta_byte(&self, k: usize) -> u8 {
        let w = k / 8;
        let shift = 8 * (k % 8);
        ((self.packed[w] >> shift) & 0xFF) as u8
    }

    /// Binds a filler to a role by channel-wise phase addition.
    ///
    /// This is circular convolution in the phase domain: binding is addition,
    /// unbinding is subtraction, and both are one operation per channel. It is
    /// what lets a representation carry structure — `dog bites man` distinct
    /// from `man bites dog` — rather than an unordered sum.
    pub fn bind(&self, role: &Self) -> Self {
        let mut out = *self;
        for k in 0..PHASE_CHANNELS {
            out.set_theta(k, self.theta(k) + role.theta(k));
        }
        out.sync_phase();
        out
    }

    /// Inverse of [`SpectralPhasor::bind`]: recovers the filler given the role.
    pub fn unbind(&self, role: &Self) -> Self {
        let mut out = *self;
        for k in 0..PHASE_CHANNELS {
            out.set_theta(k, self.theta(k) - role.theta(k));
        }
        out.sync_phase();
        out
    }

    /// Rotates every channel by `delta` — used for positional binding.
    pub fn rotate_all(&self, delta: f64) -> Self {
        let mut out = *self;
        for k in 0..PHASE_CHANNELS {
            out.set_theta(k, self.theta(k) + delta);
        }
        out.sync_phase();
        out
    }

    /// Converts the phasor into its complex wave representation.
    ///
    /// Computes `Z = A * e^(i*(phi + n*alpha))` where alpha is the
    /// fine-structure constant from config. `band_n` is capped at
    /// `BAND_N_EFFECTIVE_MAX` so the sub-band correction stays inside one sector
    /// instead of accumulating a full rotation.
    pub fn to_complex(&self) -> c64 {
        c64::from_polar(self.amplitude, self.effective_phase())
    }

    /// Complex representation of a single channel.
    #[inline]
    pub fn to_complex_ch(&self, k: usize) -> c64 {
        c64::from_polar(self.amplitude, self.theta(k))
    }
}
