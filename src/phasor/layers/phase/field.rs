#[allow(unused_imports)]
use super::super::*;

impl HierarchicalPhaseField {
    /// Walks the hierarchy from meta-patterns down to concept clusters.
    ///
    /// Returns the sector chosen at each level, coarsest first. The layers were
    /// built, displayed once, and never consulted during retrieval; this is the
    /// descent they exist for.
    pub fn descend(&self, target_phase: f64) -> Vec<(usize, u16)> {
        let mut path = Vec::new();
        for level in (1..self.layers.len()).rev() {
            let layer = &self.layers[level];
            if layer.clusters.is_empty() {
                continue;
            }
            let width = (2.0 * PI) / layer.sector_count as f64;
            let sector = ((target_phase.rem_euclid(2.0 * PI)) / width).floor() as u16
                % layer.sector_count;
            path.push((level, sector));
        }
        path.reverse();
        path
    }

    /// Coarse-to-fine retrieval: narrow by hierarchy, then rank exactly.
    ///
    /// Full ray casting is O(V) per query. Descending the hierarchy first
    /// restricts the exhaustive part to the words inside one layer-1 concept
    /// cluster and its immediate neighbours.
    pub fn candidates(&self, facet: &Facet, target_phase: f64, top_k: usize) -> Vec<(String, f64)> {
        let layer = match self.layers.get(1) {
            Some(l) if !l.clusters.is_empty() => l,
            _ => return Vec::new(),
        };
        let n = layer.sector_count;
        let width = (2.0 * PI) / n as f64;
        let home = ((target_phase.rem_euclid(2.0 * PI)) / width).floor() as u16 % n;

        let wanted: Vec<u16> = vec![home, (home + 1) % n, (home + n - 1) % n];
        let mut hits: Vec<(String, f64)> = facet
            .lexicon
            .iter()
            .filter(|(_, p)| {
                let s = ((p.phase.rem_euclid(2.0 * PI)) / width).floor() as u16 % n;
                wanted.contains(&s)
            })
            .map(|(w, p)| {
                let mut d = (p.phase - target_phase).abs();
                if d > PI {
                    d = 2.0 * PI - d;
                }
                (w.clone(), d)
            })
            .collect();

        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        hits.truncate(top_k);
        hits
    }
}
