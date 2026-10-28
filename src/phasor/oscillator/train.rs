//! OscillatorTrainer - oscillator-based training inspired by Phi-4.

use super::OscillatorField;
use crate::phasor::facet::Facet;
use phiton::p64::words::Tokenizer;
use std::f64::consts::PI;

/// Result of an oscillator training run.
#[derive(Debug, Clone)]
pub struct OscillatorTrainResult {
    pub epochs: usize,
    pub coherence_before: f64,
    pub coherence_after: f64,
    pub sync_before: f64,
    pub sync_after: f64,
    pub converged: bool,
}

impl std::fmt::Display for OscillatorTrainResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "  ── oscillator training result ──")?;
        writeln!(f, "  epochs: {}", self.epochs)?;
        writeln!(f, "  coherence: {:.4} → {:.4}", self.coherence_before, self.coherence_after)?;
        writeln!(f, "  sync:       {:.4} → {:.4}", self.sync_before, self.sync_after)?;
        if self.converged {
            writeln!(f, "  status: converged")?;
        } else {
            writeln!(f, "  status: did not converge")?;
        }
        Ok(())
    }
}

/// OscillatorTrainer - trains the facet using oscillator synchronization.
///
/// Inspired by Phi-4's finetuning approach:
/// - Multi-epoch training with convergence detection
/// - Warmup: gradually increase learning rate for first N steps
/// - Weight decay: amplitude regularization to prevent over-fitting
/// - Eval-guided: track coherence/sync before and after
pub struct OscillatorTrainer {
    pub base_lr: f64,
    pub warmup_steps: usize,
    pub weight_decay: f64,
}

impl OscillatorTrainer {
    pub fn new() -> Self {
        Self {
            base_lr: crate::phasor::facet::LEARNING_RATE,
            warmup_steps: phiton::physics::oscillator::OSCILLATOR_WARMUP_STEPS,
            weight_decay: phiton::physics::oscillator::OSCILLATOR_WEIGHT_DECAY,
        }
    }

    /// Returns the learning rate for the current step (with warmup).
    fn lr_at_step(&self, step: usize) -> f64 {
        if step < self.warmup_steps {
            self.base_lr * (step as f64 / self.warmup_steps as f64)
        } else {
            self.base_lr
        }
    }

    /// Trains for multiple epochs using oscillator synchronization.
    ///
    /// Each epoch:
    /// 1. Build oscillator field from facet
    /// 2. Compute pairwise synchronization targets
    /// 3. Shift phases toward better synchronization
    /// 4. Apply weight decay to amplitudes
    /// 5. Check convergence
    pub fn train(
        &self,
        facet: &mut Facet,
        text: &str,
        max_epochs: usize,
    ) -> OscillatorTrainResult {
        let tokens = Tokenizer::tokenize(text);
        let field_before = OscillatorField::from_facet(facet);
        let coh_before = field_before.sentence_coherence(&tokens);
        let sync_before = field_before.sentence_sync(&tokens);

        let mut prev_coherence = coh_before;
        let mut converged = false;
        let mut epochs_done = 0;

        for epoch in 0..max_epochs {
            let lr = self.lr_at_step(epoch);
            self.train_epoch(facet, &tokens, lr);
            epochs_done = epoch + 1;

            let field = OscillatorField::from_facet(facet);
            let coherence = field.sentence_coherence(&tokens);

            let improvement = (coherence - prev_coherence).abs();
            if improvement < phiton::physics::oscillator::OSCILLATOR_CONVERGENCE_DELTA {
                converged = true;
                break;
            }
            prev_coherence = coherence;
        }

        let field_after = OscillatorField::from_facet(facet);
        let coh_after = field_after.sentence_coherence(&tokens);
        let sync_after = field_after.sentence_sync(&tokens);

        OscillatorTrainResult {
            epochs: epochs_done,
            coherence_before: coh_before,
            coherence_after: coh_after,
            sync_before,
            sync_after,
            converged,
        }
    }

    /// One epoch of oscillator-based training.
    ///
    /// For each pair of words in the sentence, compute their synchronization
    /// and shift their phases toward better alignment. Apply weight decay
    /// to prevent amplitude explosion.
    fn train_epoch(&self, facet: &mut Facet, tokens: &[String], lr: f64) {
        let n = tokens.len();
        if n < 2 { return; }

        // Collect current phases and amplitudes
        let mut phases: Vec<f64> = Vec::with_capacity(n);
        let mut amplitudes: Vec<f64> = Vec::with_capacity(n);
        for token in tokens {
            if let Some(p) = facet.get_phasor(token) {
                phases.push(p.phase);
                amplitudes.push(p.amplitude);
            } else {
                phases.push(0.0);
                amplitudes.push(crate::phasor::facet::AMPLITUDE_INITIAL);
            }
        }

        // Compute pairwise phase adjustments
        let mut adjustments = vec![0.0f64; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let diff = phases[j] - phases[i];
                let coupling = diff.sin() * lr;
                adjustments[i] += coupling;
                adjustments[j] -= coupling;
            }
        }

        // Apply adjustments + weight decay
        for (i, token) in tokens.iter().enumerate() {
            if let Some(phasor) = facet.lexicon.get_mut(token) {
                phasor.phase = (phasor.phase + adjustments[i]).rem_euclid(2.0 * PI);
                phasor.amplitude = (phasor.amplitude * (1.0 - self.weight_decay))
                    .max(crate::phasor::facet::AMPLITUDE_INITIAL);
            }
        }
    }
}

impl Default for OscillatorTrainer {
    fn default() -> Self {
        Self::new()
    }
}
