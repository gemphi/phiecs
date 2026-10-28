#[allow(unused_imports)]
use super::*;

#[allow(dead_code)]
impl PhaseFlow {

    /// Records a generation step in the trajectory.
    pub fn record_step(&mut self, step: usize, word: Option<&str>, resonance: f64, novelty: f64) {
        self.trajectory.push(FlowStep {
            step,
            collective_phase: self.collective_phase,
            momentum: self.momentum,
            selected_word: word.map(|w| w.to_string()),
            resonance_score: resonance,
            novelty,
        });
    }

    /// Updates momentum based on phase difference.
    pub fn update_momentum(&mut self, phase_diff: f64) {
        self.momentum = (0.85 * self.momentum + 0.15 * phase_diff.abs().max(0.05)).min(0.5);
    }

    /// Applies Hebbian plasticity — shifts node phases toward the collective.
    pub fn hebbian_update(&self, facet: &mut Facet) {
        for node in &self.nodes {
            if let Some(phasor) = facet.lexicon.get_mut(&node.word) {
                let diff = (self.collective_phase - phasor.phase).sin();
                phasor.phase = (phasor.phase + LEARNING_RATE * diff).rem_euclid(TWO_PI);
            }
        }
    }

    /// Physics-aware Hebbian update using the color-space-time manifold gradient.
    ///
    /// Instead of raw sin(Δφ), this uses [`PhysicsOp::relax_phase`] which
    /// computes the geodesic gradient on the T² manifold — accounting for
    /// the fine-structure sub-band spacing and golden-ratio weighting.
    /// This produces more natural phase convergence than the raw Kuramoto rule.
    pub fn hebbian_update_phical(&self, facet: &mut Facet) {
        for node in &self.nodes {
            if let Some(phasor) = facet.lexicon.get_mut(&node.word) {
                phasor.phase = PhysicsOp::relax_phase(
                    phasor.phase,
                    phasor.band_n,
                    self.collective_phase,
                    0,
                    crate::phasor::facet::constants::LEARNING_RATE,
                );
            }
        }
    }

    /// Applies an anti-phase pulse between two words (self-correction).
    pub fn apply_antiphase(facet: &mut Facet, wrong: &str, correct: &str) {
        if let (Some(w), Some(c)) = (facet.lexicon.get(wrong), facet.lexicon.get(correct)) {
            let repulsion = c.phase + crate::phasor::facet::constants::PHASE_REPULSION;
            let target = (repulsion - w.phase).sin();
            if let Some(w_phasor) = facet.lexicon.get_mut(wrong) {
                w_phasor.phase = (w_phasor.phase + 0.5 * target).rem_euclid(TWO_PI);
            }
        }
    }

    /// Computes the collective phase from all nodes.
    pub(super) fn compute_collective_phase(nodes: &[FlowNode]) -> f64 {
        let sum_x: f64 = nodes.iter().map(|n| n.amplitude * n.phase.cos()).sum();
        let sum_y: f64 = nodes.iter().map(|n| n.amplitude * n.phase.sin()).sum();
        let angle = sum_y.atan2(sum_x);
        match angle < 0.0 {
            true => angle + TWO_PI,
            false => angle,
        }
    }

    /// Computes the Kuramoto order parameter R ∈ [0, 1].
    pub(super) fn compute_order_parameter(nodes: &[FlowNode]) -> f64 {
        match nodes.is_empty() {
            true => 0.0,
            false => {
                let sum_x: f64 = nodes.iter().map(|n| n.phase.cos()).sum();
                let sum_y: f64 = nodes.iter().map(|n| n.phase.sin()).sum();
                (sum_x * sum_x + sum_y * sum_y).sqrt() / nodes.len() as f64
            }
        }
    }

    /// Computes resonance of a candidate word with the current collective phase.
    /// Alignment between the flow's collective phase and a word.
    ///
    /// Compared on channel 0 alone, because that is the only channel a single
    /// collective angle determines. The previous version built a whole
    /// `SpectralPhasor` from that one angle and compared all its harmonics — but
    /// those harmonics came from `SpectralPhasor::new`, which spreads by the
    /// golden angle, while a real word's come from its hash. So 63 of the 64
    /// channels being compared were fabricated on one side and unrelated on the
    /// other, and the result was noise with a sign.
    ///
    /// It read as working only because seeded phases used to cluster by spelling:
    /// with that fixed, `test_resonance_with_known_word` began failing on a word
    /// compared against *itself*, which is what a fabricated comparison looks
    /// like once the accidental structure it rode on is removed.
    pub fn resonance_with(&self, facet: &Facet, word: &str) -> f64 {
        match facet.lexicon.get(word) {
            Some(phasor) => (self.collective_phase - phasor.phase).cos(),
            None => 0.0,
        }
    }

    /// Returns the complex wave representation of the collective phase.
    pub fn collective_wave(&self) -> c64 {
        c64::new(self.collective_phase.cos(), self.collective_phase.sin())
    }

    /// Computes novelty as phase distance from the starting collective phase.
    pub fn novelty(&self) -> f64 {
        match self.trajectory.first() {
            Some(first) => {
                let diff = (self.collective_phase - first.collective_phase).abs();
                let wrapped = diff.min(TWO_PI - diff);
                wrapped / std::f64::consts::PI
            }
            None => 0.0,
        }
    }
}
