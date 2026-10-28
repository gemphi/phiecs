//! (`crate::phasor::layers` — it was phiano's `layers`.)
mod phase;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use phase::*;
#[allow(unused_imports)]

use crate::phasor::facet::Facet;
use crate::phasor::SpectralPhasor;
use std::collections::HashMap;
use std::f64::consts::PI;

pub const PHASE_LAYERS: usize = 4; // 2^2
pub const LAYER_SECTORS: [u16; 4] = [64, 32, 16, 8]; // Halving resolution per layer

/// ClusterNode - represents a centroid node in a higher phase layer.
#[derive(Debug, Clone)]
pub struct ClusterNode {
    #[allow(dead_code)]
    pub id: usize,
    pub phasor: SpectralPhasor,
    pub member_count: usize,
}

/// PhaseLayer - a single coarse/fine phase circle layer in the hierarchy.
#[derive(Debug, Clone)]
pub struct PhaseLayer {
    #[allow(dead_code)]
    pub level: usize,
    pub sector_count: u16,
    pub clusters: HashMap<u16, ClusterNode>,
}

/// HierarchicalPhaseField - 4-layer deep hierarchical phase architecture (Phase 3).
pub struct HierarchicalPhaseField {
    pub layers: Vec<PhaseLayer>,
}
