#[allow(unused_imports)]
use super::*;

#[test]
fn test_phase_flow_build() {
    let mut facet = Facet::new();
    facet.get_or_init("hello");
    facet.get_or_init("world");

    let flow = PhaseFlow::build(&facet, "hello world");

    assert_eq!(flow.nodes.len(), 2);
    assert!(!flow.edges.is_empty());
    assert!(flow.collective_phase >= 0.0 && flow.collective_phase < TWO_PI);
    assert!(flow.order_parameter > 0.0);
}

#[test]
fn test_phase_flow_propagate() {
    let mut facet = Facet::new();
    facet.get_or_init("alpha");
    facet.get_or_init("beta");
    facet.get_or_init("gamma");

    let mut flow = PhaseFlow::build(&facet, "alpha beta gamma");
    let initial_phase = flow.collective_phase;
    flow.propagate(5);

    assert!(flow.trajectory.is_empty());
    assert!(flow.order_parameter > 0.0);
    let _ = initial_phase;
}

#[test]
fn test_antiphase_correction() {
    let mut facet = Facet::new();
    facet.get_or_init("wrong");
    facet.get_or_init("correct");

    let before = facet.lexicon["wrong"].phase;
    PhaseFlow::apply_antiphase(&mut facet, "wrong", "correct");
    let after = facet.lexicon["wrong"].phase;

    assert!(before != after, "anti-phase pulse should shift the phase");
}

#[test]
fn test_novelty_empty_trajectory() {
    let facet = Facet::new();
    let flow = PhaseFlow::build(&facet, "test");
    assert_eq!(flow.novelty(), 0.0);
}

#[test]
fn test_resonance_with_known_word() {
    let mut facet = Facet::new();
    facet.get_or_init("hello");

    let flow = PhaseFlow::build(&facet, "hello");
    let res = flow.resonance_with(&facet, "hello");
    assert!(
        res > 0.99,
        "a flow built from one word sits on that word's phase, so its \
         self-resonance is 1: got {res:.4}"
    );
}

/// The edge the graph could not previously draw.
///
/// `a dog eats a donut` has one relation in it that a reader would name, and
/// it holds between two words that are not adjacent. `build` produces only
/// adjacency, so `dog` and `donut` are three hops apart and connected by
/// nothing that mentions `eats`. This asserts the relation edge exists, that
/// it runs between the right two nodes, and that its channel is the one the
/// vocabulary gave `eats` — not merely that *some* extra edge appeared.
#[test]
fn test_a_relation_edge_names_the_word_between_its_endpoints() {
    use crate::relation::RelationVocab;
    use crate::phasor::relational::RelationalFacet;

    // A corpus, not four sentences. The rule that picks the marker compares
    // how often each candidate links, so it needs a text where the
    // determiner actually behaves like one — common everywhere, not merely
    // present. On a four-sentence fixture `a` occurs twice and `eats` three
    // times, so `a` is the rarer word and correctly wins a rule that is
    // looking for the rarer word. The fixture was wrong, not the rule.
    let mut sentences: Vec<String> = Vec::new();
    let eaters = ["dog", "cat", "child", "bird", "fox", "otter"];
    let foods = ["donut", "fish", "apple", "seed", "berry", "clam"];
    for (a, f) in eaters.iter().zip(foods.iter()) {
        sentences.push(format!("a {a} eats a {f}"));
    }
    // The determiner earns its frequency the way it does in English: by
    // appearing in every other sentence, most of which are about nothing in
    // particular.
    for a in &eaters {
        for verb in ["sleeps", "waits", "runs", "watches", "rests"] {
            sentences.push(format!("a {a} {verb} near a river"));
            sentences.push(format!("the {a} {verb} by the shore"));
        }
    }
    for f in &foods {
        for adj in ["fresh", "small", "ripe", "plain"] {
            sentences.push(format!("the {f} was {adj} today"));
            sentences.push(format!("a {f} sits on a table"));
        }
    }
    let sentences: Vec<&str> = sentences.iter().map(|s| s.as_str()).collect();
    let docs: Vec<Vec<String>> = sentences
        .iter()
        .map(|s| Tokenizer::tokenize(s))
        .collect();
    let vocab = RelationVocab::learn(&docs);
    let rel = RelationalFacet::train(&docs, &vocab, 4);
    let facet = Facet::new();

    let flow = PhaseFlow::build_relational(&facet, &rel, &vocab, "a dog eats a donut");
    let plain = PhaseFlow::build(&facet, "a dog eats a donut");

    assert!(
        plain.coupling_kinds().is_empty(),
        "adjacency-only build must produce no named relation at all — that \
         absence is what this edge exists to fix"
    );

    let dog = flow.nodes.iter().position(|n| n.word == "dog").unwrap();
    let donut = flow.nodes.iter().position(|n| n.word == "donut").unwrap();
    let eats_channel = vocab.channel(&Tokenizer::tokenize("a dog eats a donut"), 1, 3);

    let edge = flow
        .edges
        .iter()
        .find(|e| e.from == dog && e.to == donut)
        .expect("dog and donut must be coupled directly, not only through the words between them");

    match &edge.coupling {
        CouplingKind::Relation { marker, channel } => {
            assert_eq!(
                marker, "eats",
                "the coupling must be named by the word standing between the pair"
            );
            assert_eq!(
                Some(*channel),
                eats_channel,
                "the edge must read the channel the vocabulary assigned to `eats`, \
                 or the kind is a label and not a dimension"
            );
        }
        other => panic!("expected a named relation, got {other:?}"),
    }
}
