#[allow(unused_imports)]
use super::*;

impl Wave {

    /// Finds words in a specific sector of the phase circle.
    #[allow(dead_code)]
    pub fn words_in_sector(facet: &Facet, sector: u16, top_k: usize) -> Vec<(String, f64)> {
        let mut hits: Vec<(String, f64)> = facet
            .lexicon
            .par_iter()
            .filter(|(_, p)| Self::sector_of(p.effective_phase()) == sector)
            .map(|(w, p)| (w.clone(), p.amplitude))
            .collect();

        hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
        hits.into_iter().take(top_k).collect()
    }

    /// Occupancy histogram across sectors — the cheapest manifold-health check.
    pub fn sector_histogram(facet: &Facet) -> Vec<usize> {
        let n = Self::sector_count() as usize;
        let mut hist = vec![0usize; n];
        for p in facet.lexicon.values() {
            hist[Self::sector_of(p.effective_phase()) as usize] += 1;
        }
        hist
    }
}

impl SectorIndex {
    /// Builds the index from a facet's current phases.
    pub fn build(facet: &Facet) -> Self {
        let n = Wave::sector_count() as usize;
        let mut buckets = vec![Vec::new(); n];
        for (word, p) in &facet.lexicon {
            buckets[Wave::sector_of(p.effective_phase()) as usize].push(word.clone());
        }
        Self { buckets }
    }

    /// Number of sectors in the index.
    pub fn sectors(&self) -> usize {
        self.buckets.len()
    }

    /// The `k` words nearest a phase, scanning outward from its sector.
    ///
    /// Returns `(word, delta)` ascending, matching [`Wave::ray_cast`], so it is
    /// a drop-in for callers that only need local neighbours.
    pub fn near(&self, facet: &Facet, phase: f64, k: usize) -> Vec<(String, f64)> {
        let n = self.buckets.len();
        if n == 0 || k == 0 {
            return Vec::new();
        }
        let home = Wave::sector_of(phase) as usize;
        let mut hits: Vec<(String, f64)> = Vec::new();

        // Widen the ring until enough candidates are in hand, then rank.
        let mut radius = 0usize;
        while hits.len() < k * 4 && radius <= n / 2 {
            for s in [(home + radius) % n, (home + n - radius % n) % n] {
                for w in &self.buckets[s] {
                    if let Some(p) = facet.lexicon.get(w) {
                        let d = (phase - p.effective_phase()).cos().max(0.0);
                        let fam = p.amplitude / crate::phasor::spectral::AMPLITUDE_MAX;
                        hits.push((w.clone(), 1.0 - d * (0.8 + 0.2 * fam)));
                    }
                }
                if radius == 0 {
                    break;
                }
            }
            radius += 1;
        }

        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        hits.truncate(k);
        hits
    }
}
