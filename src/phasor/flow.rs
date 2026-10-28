//! (`phiecs::phasor::flow` — it was phiano's `flow`.)
mod coupling;
mod propagate;
mod record;
#[cfg(test)]
mod tests;
#[allow(unused_imports)]
use coupling::*;
#[allow(unused_imports)]
use propagate::*;
#[allow(unused_imports)]
use record::*;

use crate::phasor::facet::constants::{LEARNING_RATE, SYNTACTIC_MOMENTUM_DEFAULT};
use phiton::units::TWO_PI;
use crate::phasor::facet::Facet;
use phiton::physics::PhysicsOp;
use crate::phasor::SpectralPhasor;
use crate::relation::RelationVocab;
use crate::phasor::relational::{RelationalFacet, WINDOW};
use phiton::p64::words::Tokenizer;
use phistd::c64;
use serde::Serialize;

/// What kind of coupling holds two flow nodes together.
///
/// This enum used to carry two more variants, `Semantic` and `AntiPhase`, which
/// nothing constructed — held alive by an `#[allow(dead_code)]`. Deleting them
/// was right and it was not enough, because the idea they gestured at is real
/// and was missing: when two words are associated, the association *has a kind*,
/// and the kind is nameable. In `a dog eats a donut` the coupling between `dog`
/// and `donut` is `eats`. The graph did not have that edge in any form. It built
/// `dog→eats`, `eats→a`, `a→donut` and stopped, so the one relation in the
/// sentence a reader would name was the one relation the graph could not.
///
/// [`CouplingKind::Relation`] is that edge, and it carries the thing an enum
/// variant could never carry: the **dimension** the coupling acts in.
/// [`crate::relation::RelationVocab`] assigns every observable relation its
/// own channel out of the sixty-four, so `eats` is not just a label on the edge
/// — it names which of the word's sixty-four complex components the coupling
/// reads. Two words coupled by `eats` are compared on the `eats` channel and no
/// other. That is why the variant is a struct and not a bare tag, and why it is
/// discovered from the corpus rather than declared here: the kinds are whatever
/// the text supplies, up to the channel count.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CouplingKind {
    /// Adjacency, weighted by bigram probability.
    Bigram,
    /// Adjacency, carrying the learned phase lag.
    SyntaxLag,
    /// A named relation between two words that need not be adjacent, and the
    /// channel it occupies. `marker` is the word standing between them, or the
    /// offset's name when the relation is bare distance.
    Relation { marker: String, channel: usize },
}

/// A node in the dynamic phase computation graph.
#[derive(Debug, Clone, Serialize)]
pub struct FlowNode {
    pub word: String,
    pub phase: f64,
    pub amplitude: f64,
    pub band_n: u32,
    pub activation: f64,
    pub novelty: f64,
}

/// An edge in the dynamic phase computation graph.
#[derive(Debug, Clone, Serialize)]
pub struct FlowEdge {
    pub from: usize,
    pub to: usize,
    pub coupling: CouplingKind,
    pub lag: f64,
    pub weight: f64,
}

/// A snapshot of the phase flow at one generation step.
#[derive(Debug, Clone, Serialize)]
pub struct FlowStep {
    pub step: usize,
    pub collective_phase: f64,
    pub momentum: f64,
    pub selected_word: Option<String>,
    pub resonance_score: f64,
    pub novelty: f64,
}

/// PhaseFlow — a dynamic phase computation graph built per input.
///
/// Like PyTorch's dynamic graph, the topology emerges from execution.
/// Parameters persist in Facet; the PhaseFlow is ephemeral per forward pass.
#[derive(Debug, Clone, Serialize)]
pub struct PhaseFlow {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub trajectory: Vec<FlowStep>,
    pub collective_phase: f64,
    pub momentum: f64,
    pub order_parameter: f64,
}
