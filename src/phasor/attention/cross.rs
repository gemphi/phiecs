//! Phase-space cross-attention mechanism between query and key sequences.
//!
//! Implements spectral phase alignment between two token sequences (e.g. prompt
//! prompt tokens as queries and generated tokens as keys). All operations are
//! encapsulated in [`CrossAttention`], following the Diem convention that all
//! public symbols belong to named types.
//!
//! # Architecture
//!
//! ```text
//! Prompt Tokens (Queries)      Generated Tokens (Keys)
//!        │                               │
//!        ▼                               ▼
//!   SpectralPhasor                  SpectralPhasor
//!        │                               │
//!        └───────────────┬───────────────┘
//!                        ▼
//!           CrossAttention::weights()
//!           cos(θ_gen - θ_prompt) * (1 + 0.1 * amp)
//!                        │
//!                        ▼ (Softmax / Temperature = 0.5)
//!           Attention Matrix [prompt_i][gen_j]
//! ```

use crate::phasor::facet::Facet;
use crate::phasor::SpectralPhasor;

/// Cross-attention calculator operating over spectral phase coordinates.
pub struct CrossAttention;

impl CrossAttention {
    /// Computes normalized attention weights `[prompt_token][generated_token]`.
    ///
    /// The attention score between prompt token `p` and generated token `g`
    /// is defined by phase alignment: `cos(θ_g - θ_p) * (1.0 + 0.1 * amp_g)`.
    ///
    /// # Invariants
    ///
    /// - Each row of the output sums to `1.0` (or `0.0` if total energy is zero).
    /// - Output dimension is `[valid_prompt_tokens][generated_tokens.len()]`.
    pub fn weights(
        facet: &Facet,
        prompt_tokens: &[String],
        generated_tokens: &[String],
    ) -> Vec<Vec<f64>> {
        let prompt_phasors: Vec<Option<&SpectralPhasor>> = prompt_tokens.iter()
            .map(|t| facet.lexicon.get(t))
            .collect();
        let gen_phasors: Vec<Option<&SpectralPhasor>> = generated_tokens.iter()
            .map(|t| facet.lexicon.get(t))
            .collect();

        let mut result = Vec::with_capacity(prompt_tokens.len());

        for (_i, p_phasor) in prompt_phasors.iter().enumerate() {
            let mut row = Vec::with_capacity(generated_tokens.len());
            let p_phase = match p_phasor {
                Some(p) => p.phase,
                None => continue,
            };

            let mut scores = Vec::with_capacity(generated_tokens.len());
            for g_phasor in &gen_phasors {
                let score = match g_phasor {
                    Some(g) => (g.phase - p_phase).cos() * (1.0 + g.amplitude * 0.1),
                    None => 0.0,
                };
                scores.push(score);
            }

            let max_s = scores.iter().cloned().fold(0.0f64, f64::max);
            let exp_s: Vec<f64> = scores.iter().map(|s| ((s - max_s) / 0.5).exp()).collect();
            let total: f64 = exp_s.iter().sum();
            if total > 0.0 {
                row = exp_s.iter().map(|e| e / total).collect();
            } else {
                row = vec![0.0; generated_tokens.len()];
            }

            result.push(row);
        }

        result
    }
}
