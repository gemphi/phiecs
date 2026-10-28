//! (`crate::phasor::attention` — it was phiano's `attention`.)
//!
//! Phase-space multi-head self-attention mechanism.
//!
//! Words attend to each other based on their phase relationships in the
//! spectral phasor space. Unlike standard transformer attention (which uses
//! learned $Q, K, V$ projection weight matrices), this mechanism uses the
//! natural circular phase geometry of the facet as the attention basis.
//!
//! Each "head" corresponds to a distinct phase sector (chromatic band),
//! allowing the model to attend to multiple semantic aspects simultaneously.
//! All operations are encapsulated in [`Attention`], following the Diem
//! convention that all public symbols belong to named types.
//!
//! # Architecture
//!
//! ```text
//! Context Tokens [t_1, t_2, ..., t_N]
//!   │
//!   ▼ (Phase Alignment per Head h ∈ {0..7})
//! Query Phase: θ_q(h) = 0.7 * (2πh/H) + 0.3 * θ_context
//!   │
//!   ├─▶ Head 0 (Red Sector)    ──▶ Complex Sum_0
//!   ├─▶ Head 1 (Orange Sector) ──▶ Complex Sum_1
//!   ├─▶ ...
//!   └─▶ Head 7 (Violet Sector) ──▶ Complex Sum_7
//!   │
//!   ▼ (Average Heads)
//! Combined Phasor Output: c64
//! ```

use crate::phasor::facet::Facet;
use crate::phasor::SpectralPhasor;
use phistd::c64;
use std::f64::consts::PI;

/// Number of attention heads (corresponding to 8 primary phase sectors).
pub const NUM_HEADS: usize = 8;

/// Multi-head phase self-attention engine.
pub struct Attention;

/// Output package from multi-head self-attention evaluation.
#[derive(Debug, Clone)]
pub struct AttentionOutput {
    /// Weighted combination of token phasors (one complex sum per head).
    pub head_outputs: Vec<c64>,
    /// Attention weights matrix `[head][token_index]` representing attention density.
    pub weights: Vec<Vec<f64>>,
    /// Combined attention phasor across all heads.
    pub combined: c64,
}

impl Attention {
    /// Computes multi-head self-attention over a token sequence.
    ///
    /// For each head $h \in \{0 \dots H-1\}$:
    /// - **Query**: Target center phase for head $h$, blended with context phase
    /// - **Key**: Each token's lexical phase $\theta_i$
    /// - **Score**: Phase alignment $\cos(\theta_i - \theta_q) \cdot (1 + 0.1 \cdot A_i)$
    /// - **Value**: The token's phasor weighted by normalized softmax weights
    pub fn self_attend(
        facet: &Facet,
        tokens: &[String],
        context_phase: f64,
    ) -> AttentionOutput {
        let phasors: Vec<Option<&SpectralPhasor>> = tokens.iter()
            .map(|t| facet.lexicon.get(t))
            .collect();

        let mut head_outputs = Vec::with_capacity(NUM_HEADS);
        let mut all_weights = Vec::with_capacity(NUM_HEADS);

        for head in 0..NUM_HEADS {
            // Each head focuses on a different phase sector
            let head_center = (head as f64 / NUM_HEADS as f64) * 2.0 * PI;
            // Blend with context phase for relevance
            let query_phase = 0.7 * head_center + 0.3 * context_phase;

            // Compute attention scores (phase alignment)
            let mut scores: Vec<f64> = Vec::with_capacity(tokens.len());
            for phasor in &phasors {
                let score = match phasor {
                    Some(p) => {
                        let phase_diff = (p.phase - query_phase).cos();
                        // Scale by amplitude (more confident words attend more)
                        phase_diff * (1.0 + p.amplitude * 0.1)
                    }
                    None => 0.0,
                };
                scores.push(score);
            }

            // Softmax normalization with temperature
            let temperature = 0.5;
            let max_score = scores.iter().cloned().fold(0.0f64, f64::max);
            let exp_scores: Vec<f64> = scores.iter()
                .map(|s| ((s - max_score) / temperature).exp())
                .collect();
            let total: f64 = exp_scores.iter().sum();
            let weights: Vec<f64> = if total > 0.0 {
                exp_scores.iter().map(|e| e / total).collect()
            } else {
                vec![0.0; tokens.len()]
            };

            // Weighted sum of phasor values
            let mut sum_x = 0.0f64;
            let mut sum_y = 0.0f64;
            for (i, phasor) in phasors.iter().enumerate() {
                if let Some(p) = phasor {
                    let v = p.to_complex();
                    sum_x += weights[i] * v.re;
                    sum_y += weights[i] * v.im;
                }
            }

            head_outputs.push(c64::new(sum_x, sum_y));
            all_weights.push(weights);
        }

        // Combine heads by averaging
        let combined_x: f64 = head_outputs.iter().map(|h| h.re).sum::<f64>() / NUM_HEADS as f64;
        let combined_y: f64 = head_outputs.iter().map(|h| h.im).sum::<f64>() / NUM_HEADS as f64;
        let combined = c64::new(combined_x, combined_y);

        AttentionOutput {
            head_outputs,
            weights: all_weights,
            combined,
        }
    }

    /// Returns the top-k attended words from a token list for a given head.
    pub fn top_attended(
        tokens: &[String],
        weights: &[f64],
        k: usize,
    ) -> Vec<(String, f64)> {
        let mut indexed: Vec<(usize, f64)> = weights.iter()
            .enumerate()
            .map(|(i, &w)| (i, w))
            .collect();
        indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        indexed.iter()
            .take(k)
            .filter_map(|(i, w)| tokens.get(*i).map(|t| (t.clone(), *w)))
            .collect()
    }

    /// Attention-guided word selection: combines n-gram transition frequencies
    /// with phase-space alignment to rank next-word candidates.
    pub fn next_words(
        facet: &Facet,
        context_tokens: &[String],
        candidates: &[(String, u32)],
        context_phase: f64,
        top_k: usize,
    ) -> Vec<(String, f64)> {
        if candidates.is_empty() {
            return Vec::new();
        }

        // Run self-attention on context tokens
        let attn = Self::self_attend(facet, context_tokens, context_phase);

        // Score each candidate by normalized log n-gram and phase attention alignment
        let mut scored: Vec<(String, f64)> = candidates.iter()
            .map(|(word, count)| {
                let log_ngram = (*count as f64 + 1.0).ln();

                // Attention score: how well does this word's phase align with attention output?
                let attn_score = facet.lexicon.get(word).map(|p| {
                    let word_v = p.to_complex();
                    let diff = (word_v - attn.combined).norm();
                    1.0 / (1.0 + diff)
                }).unwrap_or(0.0);

                // Balanced blend: 40% log n-gram frequency + 60% semantic phase alignment
                let combined = 0.4 * log_ngram + 0.6 * attn_score * 10.0;
                (word.clone(), combined)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        scored
    }
}
