#[allow(unused_imports)]
use super::*;

#[allow(dead_code)]
impl PhaseFlow {
    /// Builds a PhaseFlow from a prompt and facet.
    /// Creates nodes for each token, edges for bigrams and semantic neighbors.
    pub fn build(facet: &Facet, prompt: &str) -> Self {
        let tokens = Tokenizer::tokenize(prompt);
        let mut nodes = Vec::with_capacity(tokens.len());
        let mut edges = Vec::new();
        let mut word_indices: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for token in &tokens {
            let idx = match word_indices.get(token) {
                Some(&i) => i,
                None => {
                    let i = nodes.len();
                    word_indices.insert(token.clone(), i);
                    let phasor = facet.lexicon.get(token).copied().unwrap_or_else(|| {
                        let seed = (token.len() as f64 * phiton::units::PHI).rem_euclid(TWO_PI);
                        SpectralPhasor::new(seed, crate::phasor::facet::constants::AMPLITUDE_INITIAL, crate::phasor::facet::constants::BAND_N_INITIAL)
                    });
                    nodes.push(FlowNode {
                        word: token.clone(),
                        phase: phasor.phase,
                        amplitude: phasor.amplitude,
                        band_n: phasor.band_n,
                        activation: phasor.amplitude,
                        novelty: 0.0,
                    });
                    i
                }
            };
            if nodes.len() > 1 && idx != nodes.len() - 1 {
                let prev_idx = nodes.len() - 2;
                let lag = facet.phase_lag(&nodes[prev_idx].word, token);
                edges.push(FlowEdge {
                    from: prev_idx,
                    to: idx,
                    coupling: CouplingKind::SyntaxLag,
                    lag,
                    weight: 1.0,
                });
            }
        }

        for i in 0..nodes.len().saturating_sub(1) {
            let lag = facet.phase_lag(&nodes[i].word, &nodes[i + 1].word);
            let weight = facet.bigram_probability(&nodes[i].word, &nodes[i + 1].word);
            edges.push(FlowEdge {
                from: i,
                to: i + 1,
                coupling: CouplingKind::Bigram,
                lag,
                weight: weight.max(0.1),
            });
        }

        let collective_phase = Self::compute_collective_phase(&nodes);
        let order_parameter = Self::compute_order_parameter(&nodes);

        Self {
            nodes,
            edges,
            trajectory: Vec::new(),
            collective_phase,
            momentum: SYNTACTIC_MOMENTUM_DEFAULT,
            order_parameter,
        }
    }

    /// Builds the graph with named relation edges as well as adjacency.
    ///
    /// [`Self::build`] connects each word to the next one and stops, so the only
    /// couplings it can represent are positional. This adds the edge a reader
    /// would draw: for every pair of words inside the relation window that
    /// [`RelationVocab`] can name, an edge carrying that name and the channel it
    /// owns. `a dog eats a donut` gains `dog —eats→ donut`.
    ///
    /// The lag on a relation edge is read from the channel of its own kind —
    /// `theta_to[k] - theta_from[k]` — not from the word's single global phase.
    /// This is where the coupling kind stops being a label: `eats` and `of`
    /// between the same two words would pull them toward different angles,
    /// because they are angles in different dimensions. The weight is the
    /// geometric mean of the evidence both words carry on that channel, so a
    /// relation asserted once between two words the corpus barely places there
    /// pulls proportionately less than one it has seen a thousand times.
    pub fn build_relational(
        facet: &Facet,
        rel: &RelationalFacet,
        vocab: &RelationVocab,
        prompt: &str,
    ) -> Self {
        let mut flow = Self::build(facet, prompt);
        let toks = Tokenizer::tokenize(prompt);

        // Node indices are deduplicated by word, so a repeated word is one node.
        let index_of = |w: &str| flow.nodes.iter().position(|n| n.word == w);

        for i in 0..toks.len() {
            for j in (i + 1)..(i + 1 + WINDOW).min(toks.len()) {
                let k = match vocab.channel(&toks, i, j) {
                    Some(k) => k,
                    None => continue,
                };
                // Adjacency already has two edges; a relation edge on top of them
                // would be a third vote for the same fact.
                if j == i + 1 {
                    continue;
                }
                let (from, to) = match (index_of(&toks[i]), index_of(&toks[j])) {
                    (Some(a), Some(b)) if a != b => (a, b),
                    _ => continue,
                };
                let (pa, pb) = match (rel.words.get(&toks[i]), rel.words.get(&toks[j])) {
                    (Some(a), Some(b)) => (a, b),
                    _ => continue,
                };
                let weight = (pa.mag[k] * pb.mag[k]).sqrt();
                if weight <= 0.0 {
                    continue;
                }
                flow.edges.push(FlowEdge {
                    from,
                    to,
                    coupling: CouplingKind::Relation {
                        marker: vocab.name(k),
                        channel: k,
                    },
                    lag: (pb.theta[k] - pa.theta[k]).rem_euclid(TWO_PI),
                    weight,
                });
            }
        }
        flow
    }

    /// The distinct relation kinds this graph actually used.
    ///
    /// Reported rather than assumed: the architecture allows sixty-four, and how
    /// many a given sentence or corpus supplies is a fact about the text.
    pub fn coupling_kinds(&self) -> Vec<(String, usize)> {
        let mut seen: Vec<(String, usize)> = Vec::new();
        for e in &self.edges {
            if let CouplingKind::Relation { marker, channel } = &e.coupling {
                if !seen.iter().any(|(_, c)| c == channel) {
                    seen.push((marker.clone(), *channel));
                }
            }
        }
        seen.sort_by_key(|(_, c)| *c);
        seen
    }

    /// Propagates phase waves through the graph for `steps` iterations.
    /// Each step: update activations via Kuramoto coupling at each edge.
    pub fn propagate(&mut self, steps: usize) {
        for _ in 0..steps {
            let mut new_activations = self.nodes.iter().map(|n| n.activation).collect::<Vec<_>>();
            for edge in &self.edges {
                let from_phase = self.nodes[edge.from].phase;
                let to_phase = self.nodes[edge.to].phase;
                let coupling_strength = edge.weight * (to_phase - from_phase + edge.lag).sin();
                new_activations[edge.to] += 0.1 * coupling_strength * self.nodes[edge.from].amplitude;
            }
            for (i, act) in new_activations.into_iter().enumerate() {
                self.nodes[i].activation = act.max(0.0);
            }
            self.collective_phase = Self::compute_collective_phase(&self.nodes);
            self.order_parameter = Self::compute_order_parameter(&self.nodes);
        }
    }
}
