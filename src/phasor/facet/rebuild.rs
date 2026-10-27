#[allow(unused_imports)]
use super::*;

impl Facet {

    /// Rebuilds the relation negative pool: `count^0.75` copies per word,
    /// uncapped, scaled so the pool is about eight entries per word.
    pub fn rebuild_relation_pool(&mut self) {
        if !self.relation_pool.is_empty() && self.relation_pool.len() >= self.lexicon.len() {
            return;
        }
        let mut words: Vec<(&String, &SpectralPhasor)> = self.lexicon.iter().collect();
        words.sort_unstable_by(|a, b| a.0.cmp(b.0));
        let total: f64 = words.iter().map(|(_, p)| (p.count.max(1) as f64).powf(0.75)).sum();
        let scale = (total / (words.len() as f64 * 8.0)).max(1.0);
        let mut pool = Vec::new();
        for (word, phasor) in words {
            let reps = ((phasor.count.max(1) as f64).powf(0.75) / scale).round().max(1.0) as usize;
            for _ in 0..reps {
                pool.push(word.clone());
            }
        }
        self.relation_pool = pool;
    }

    /// Draws a relation negative (see [`Self::rebuild_relation_pool`]).
    #[inline]
    pub fn sample_relation_negative(&self, r: u64) -> Option<&String> {
        match self.relation_pool.is_empty() {
            true => self.sample_negative(r),
            false => self.relation_pool.get((r % self.relation_pool.len() as u64) as usize),
        }
    }

    /// Draws a frequency-biased negative sample.
    #[inline]
    pub fn sample_negative(&self, r: u64) -> Option<&String> {
        if self.sample_pool.is_empty() {
            return None;
        }
        self.sample_pool.get((r % self.sample_pool.len() as u64) as usize)
    }

    /// A negative sample that is not definitionally related to `target`.
    ///
    /// Uniform negative sampling will sometimes draw a word from the target's
    /// own definition, and the contrastive update then pushes apart exactly the
    /// pair the dictionary says belongs together — the training signal working
    /// against itself. Dict2vec (Tissier et al., EMNLP 2017) calls filtering
    /// these out *controlled negative sampling* and measures it discarding
    /// around 2% of drawn negatives; the count is small, and every one of them
    /// was an update in the wrong direction.
    ///
    /// Falls back to the unfiltered draw after `tries` attempts, so a word whose
    /// definition covers much of the pool cannot stall training. `related`
    /// says whether two words are joined by a definition (phiano passes its
    /// definition graph's `is_related`).
    pub fn sample_negative_controlled(
        &self,
        r: u64,
        target: &str,
        related: impl Fn(&str, &str) -> bool,
        tries: u32,
    ) -> Option<&String> {
        let mut x = r | 1;
        for _ in 0..tries {
            let cand = self.sample_negative(x)?;
            if !related(target, cand) {
                return Some(cand);
            }
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
        }
        self.sample_negative(r)
    }

    /// Mean phase coherence between two words across all channels.
    pub fn resonance(&self, word_a: &str, word_b: &str) -> f64 {
        match (self.lexicon.get(word_a), self.lexicon.get(word_b)) {
            (Some(a), Some(b)) => a.resonance(b),
            _ => 0.0,
        }
    }

    /// Circular dispersion of the lexicon's channel-0 phases.
    ///
    /// 1.0 means phases are spread uniformly around the circle; 0.0 means every
    /// word sits at the same angle. Dispersion falling while `coherence` rises
    /// is the signature of collapse rather than learning.
    pub fn phase_dispersion(&self) -> f64 {
        self.dispersion_above(0)
    }

    /// [`Facet::phase_dispersion`] restricted to words seen at least `floor` times.
    ///
    /// The global figure is dominated by the tail. A vocabulary of 30,000 words
    /// whose 500 most frequent members have collapsed onto one angle still reads
    /// ~0.98, because the 29,500 rare words retain their initialisation and
    /// average away. Every task the model is scored on — next-word ranking,
    /// relational probes, sentence completion — draws its candidates from the
    /// frequent band, so that is the band whose dispersion has to be watched.
    pub fn dispersion_above(&self, floor: u32) -> f64 {
        let (n, sx, sy) = self
            .lexicon
            .values()
            .filter(|p| p.count >= floor)
            .fold((0usize, 0.0f64, 0.0f64), |(n, x, y), p| {
                (n + 1, x + p.phase.cos(), y + p.phase.sin())
            });
        if n == 0 {
            return 1.0;
        }
        1.0 - (sx.hypot(sy) / n as f64)
    }

    /// Dispersion of the `k` most frequent words.
    ///
    /// A rank cut rather than a count cut, for when the absolute frequencies are
    /// corpus-dependent but the shape of the band is not.
    pub fn dispersion_top(&self, k: usize) -> f64 {
        if k == 0 {
            return 1.0;
        }
        let mut counts: Vec<u32> = self.lexicon.values().map(|p| p.count).collect();
        if counts.len() <= k {
            return self.phase_dispersion();
        }
        // Select the k-th largest count; ties admit a few extra words, which is
        // the conservative direction (a wider band can only look more dispersed).
        counts.sort_unstable_by(|a, b| b.cmp(a));
        self.dispersion_above(counts[k - 1])
    }

    /// Gini coefficient of sector occupancy.
    pub fn sector_gini(&self) -> f64 {
        let n_sectors = crate::phasor::facet::constants::SECTOR_RESOLUTION as usize;
        let width = phiton::units::TWO_PI / n_sectors as f64;
        let mut hist = vec![0u64; n_sectors];
        for p in self.lexicon.values() {
            let s = (p.phase / width).floor() as usize % n_sectors;
            hist[s] += 1;
        }
        hist.sort_unstable();
        let total: u64 = hist.iter().sum();
        if total == 0 {
            return 0.0;
        }
        let n = hist.len() as f64;
        let mut cum = 0.0;
        for (i, &c) in hist.iter().enumerate() {
            cum += (2.0 * (i as f64 + 1.0) - n - 1.0) * c as f64;
        }
        (cum / (n * total as f64)).clamp(0.0, 1.0)
    }

    /// Computes the average amplitude across all phasors in the lexicon.
    pub fn average_amplitude(&self) -> f64 {
        if self.lexicon.is_empty() {
            return 0.0;
        }
        let total: f64 = self.lexicon.values().map(|p| p.amplitude).sum();
        total / self.lexicon.len() as f64
    }

    /// Returns the most common band level (n) across all phasors.
    pub fn dominant_band(&self) -> u32 {
        let mut counts: HashMap<u32, u32> = HashMap::new();
        for phasor in self.lexicon.values() {
            *counts.entry(phasor.band_n).or_insert(0) += 1;
        }
        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(band, _)| band)
            .unwrap_or(1)
    }
}
