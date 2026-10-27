#[allow(unused_imports)]
use super::*;

/// Retrieval must not offer words the model never moved.
///
/// A word seen once sits at its hash seed, which is a function of its
/// spelling — the reason `synonym cow` returned `cod, cot, coz`.
#[test]
fn test_retrieval_excludes_the_untrained_tail() {
    let mut f = Facet::new();
    // A well-trained pool, plus one rare word placed right on top of the
    // target so it would win every ranking without a floor.
    for i in 0..200 {
        let p = f.get_or_init(&format!("trained{i}"));
        p.count = 50;
    }
    let target = f.get_or_init("target");
    target.count = 50;
    let thetas: Vec<f64> =
        (0..phiton::units::PHASE_CHANNELS).map(|k| f.lexicon["target"].theta(k)).collect();

    let rare = f.get_or_init("rare_twin");
    rare.count = 1;
    for (k, t) in thetas.iter().enumerate() {
        rare.set_theta(k, *t);
    }
    rare.sync_phase();

    let hits = Wave::ray_cast_word(&f, "target", 5);
    assert!(
        !hits.iter().any(|(w, _)| w == "rare_twin"),
        "a word seen once should not be offered: {hits:?}"
    );

    // And the floor backs off rather than returning nothing when almost
    // everything is rare.
    let mut sparse = Facet::new();
    for i in 0..6 {
        let p = sparse.get_or_init(&format!("w{i}"));
        p.count = 1;
    }
    assert!(
        !Wave::ray_cast_word(&sparse, "w0", 3).is_empty(),
        "the floor must back off on a facet with nothing above it"
    );
}

fn facet_with(words: &[&str]) -> Facet {
    let mut f = Facet::new();
    for w in words {
        f.get_or_init(w);
    }
    f
}

#[test]
fn test_plain_sentence_is_order_invariant() {
    let f = facet_with(&["dog", "bites", "man"]);
    let a = Wave::text(&f, "dog bites man");
    let b = Wave::text(&f, "man bites dog");
    assert!((a - b).norm() < 1e-12, "the bag representation is order-free by design");
}

#[test]
fn test_bound_sentence_is_order_sensitive() {
    let f = facet_with(&["dog", "bites", "man"]);
    let a = Wave::text_bound(&f, "dog bites man");
    let b = Wave::text_bound(&f, "man bites dog");
    assert!((a - b).norm() > 0.1, "positional binding must distinguish word order");
}

#[test]
fn test_negation_order_distinguished() {
    let f = facet_with(&["not", "safe"]);
    let a = Wave::text_bound(&f, "not safe");
    let b = Wave::text_bound(&f, "safe not");
    assert!((a - b).norm() > 0.1);
}

#[test]
fn test_ray_cast_prefers_phase_over_amplitude() {
    let mut f = Facet::new();
    // exact phase twin at low familiarity vs a distant word at high familiarity
    let mut twin = crate::phasor::SpectralPhasor::seeded("query", 1.0, 1);
    twin.amplitude = 1.0;
    f.lexicon.insert("twin".into(), twin);

    let mut far = crate::phasor::SpectralPhasor::seeded("query", 1.0, 1);
    for k in 0..PHASE_CHANNELS {
        far.set_theta(k, far.theta(k) + std::f64::consts::PI);
    }
    far.amplitude = crate::phasor::spectral::AMPLITUDE_MAX;
    f.lexicon.insert("far".into(), far);

    f.lexicon.insert("query".into(), crate::phasor::SpectralPhasor::seeded("query", 1.0, 1));

    let hits = Wave::ray_cast_word(&f, "query", 2);
    assert_eq!(hits[0].0, "twin", "an exact phase match must beat a familiar distant word");
}

#[test]
fn test_sector_index_matches_a_full_scan_locally() {
    let mut f = Facet::new();
    for w in ["alpha","beta","gamma","delta","epsilon","zeta","eta","theta","iota","kappa"] {
        f.get_or_init(w);
    }
    let probe = f.lexicon["alpha"].effective_phase();
    let idx = SectorIndex::build(&f);
    let fast = idx.near(&f, probe, 3);
    let exact = Wave::ray_cast(&f, phistd::c64::from_polar(1.0, probe), 3);
    assert!(!fast.is_empty());
    assert_eq!(fast[0].0, exact[0].0, "the index must agree with the scan on the nearest word");
}

#[test]
fn test_ray_cast_delta_is_ascending() {
    let f = facet_with(&["alpha", "beta", "gamma", "delta", "epsilon"]);
    let hits = Wave::ray_cast_word(&f, "alpha", 4);
    for w in hits.windows(2) {
        assert!(w[0].1 <= w[1].1, "callers rely on ascending delta");
    }
}
