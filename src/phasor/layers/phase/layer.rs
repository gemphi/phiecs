#[allow(unused_imports)]
use super::super::*;

impl PhaseLayer {
    pub fn new(level: usize, sector_count: u16) -> Self {
        Self {
            level,
            sector_count,
            clusters: HashMap::new(),
        }
    }
}

impl Default for HierarchicalPhaseField {
    fn default() -> Self {
        Self::new()
    }
}

impl HierarchicalPhaseField {
    pub fn new() -> Self {
        let mut layers = Vec::with_capacity(PHASE_LAYERS);
        for (idx, &num_sectors) in LAYER_SECTORS.iter().enumerate() {
            layers.push(PhaseLayer::new(idx, num_sectors));
        }
        Self { layers }
    }

    /// Computes bottom-up hierarchical phase centroids across all 4 layers.
    ///
    /// Layer 0: Surface Words (Facet Lexicon)
    /// Layer 1: Concept Clusters (32 sectors)
    /// Layer 2: Domain Sectors (16 sectors)
    /// Layer 3: Meta-Patterns (8 sectors)
    pub fn build_hierarchy(&mut self, facet: &Facet) {
        if facet.lexicon.is_empty() {
            return;
        }

        // 1. Build Layer 1 (32 Concept Clusters from Layer 0 words)
        self.build_layer_from_words(facet, 1, LAYER_SECTORS[1]);

        // 2. Build Layer 2 (16 Domain Sectors from Layer 1 clusters)
        self.build_layer_from_prev_layer(1, 2, LAYER_SECTORS[2]);

        // 3. Build Layer 3 (8 Meta-Patterns from Layer 2 clusters)
        self.build_layer_from_prev_layer(2, 3, LAYER_SECTORS[3]);
    }

    pub(in crate::phasor::layers) fn build_layer_from_words(&mut self, facet: &Facet, layer_idx: usize, num_sectors: u16) {
        let mut sector_sums_x: HashMap<u16, f64> = HashMap::new();
        let mut sector_sums_y: HashMap<u16, f64> = HashMap::new();
        let mut counts: HashMap<u16, usize> = HashMap::new();

        let sector_width = (2.0 * PI) / (num_sectors as f64);

        for phasor in facet.lexicon.values() {
            let sector = ((phasor.phase / sector_width).floor() as u16) % num_sectors;
            *sector_sums_x.entry(sector).or_default() += phasor.amplitude * phasor.phase.cos();
            *sector_sums_y.entry(sector).or_default() += phasor.amplitude * phasor.phase.sin();
            *counts.entry(sector).or_default() += 1;
        }

        let mut clusters = HashMap::new();
        for sector in 0..num_sectors {
            let count = *counts.get(&sector).unwrap_or(&0);
            if count > 0 {
                let sx = *sector_sums_x.get(&sector).unwrap_or(&0.0);
                let sy = *sector_sums_y.get(&sector).unwrap_or(&0.0);
                let mut phase = sy.atan2(sx);
                if phase < 0.0 {
                    phase += 2.0 * PI;
                }
                let amp = (sx * sx + sy * sy).sqrt() / (count as f64);
                
                clusters.insert(
                    sector,
                    ClusterNode {
                        id: sector as usize,
                        phasor: SpectralPhasor::new(phase, amp, 1),
                        member_count: count,
                    },
                );
            }
        }

        self.layers[layer_idx].clusters = clusters;
    }

    pub(in crate::phasor::layers) fn build_layer_from_prev_layer(&mut self, prev_idx: usize, curr_idx: usize, num_sectors: u16) {
        let prev_layer = self.layers[prev_idx].clone();
        let mut sector_sums_x: HashMap<u16, f64> = HashMap::new();
        let mut sector_sums_y: HashMap<u16, f64> = HashMap::new();
        let mut counts: HashMap<u16, usize> = HashMap::new();

        let sector_width = (2.0 * PI) / (num_sectors as f64);

        for node in prev_layer.clusters.values() {
            let sector = ((node.phasor.phase / sector_width).floor() as u16) % num_sectors;
            *sector_sums_x.entry(sector).or_default() += node.phasor.amplitude * node.phasor.phase.cos();
            *sector_sums_y.entry(sector).or_default() += node.phasor.amplitude * node.phasor.phase.sin();
            *counts.entry(sector).or_default() += node.member_count;
        }

        let mut clusters = HashMap::new();
        for sector in 0..num_sectors {
            let count = *counts.get(&sector).unwrap_or(&0);
            if count > 0 {
                let sx = *sector_sums_x.get(&sector).unwrap_or(&0.0);
                let sy = *sector_sums_y.get(&sector).unwrap_or(&0.0);
                let mut phase = sy.atan2(sx);
                if phase < 0.0 {
                    phase += 2.0 * PI;
                }
                let amp = (sx * sx + sy * sy).sqrt() / (count as f64);

                clusters.insert(
                    sector,
                    ClusterNode {
                        id: sector as usize,
                        phasor: SpectralPhasor::new(phase, amp, 1),
                        member_count: count,
                    },
                );
            }
        }

        self.layers[curr_idx].clusters = clusters;
    }

    /// Evaluates multi-layer depth resonance for a query phase angle.
    pub fn resonate_depth(&self, target_phase: f64) -> Vec<(usize, u16, f64)> {
        let mut layer_resonances = Vec::new();
        for (level, layer) in self.layers.iter().enumerate().skip(1) {
            let sector_width = (2.0 * PI) / (layer.sector_count as f64);
            let sector = ((target_phase / sector_width).floor() as u16) % layer.sector_count;
            if let Some(node) = layer.clusters.get(&sector) {
                let mut diff = (node.phasor.phase - target_phase).abs();
                if diff > PI {
                    diff = 2.0 * PI - diff;
                }
                layer_resonances.push((level, sector, diff));
            }
        }
        layer_resonances
    }
}
