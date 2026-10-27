#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

#[test]
fn test_seed_is_deterministic() {
    let a = SpectralPhasor::seeded("borrow", 1.0, 1);
    let b = SpectralPhasor::seeded("borrow", 1.0, 1);
    assert!((a.phase - b.phase).abs() < 1e-12);
    assert_eq!(a.resonance(&b), 1.0_f64.min(a.resonance(&b)));
    assert!(a.resonance(&b) > 0.999);
}

/// Two unrelated words should have near-zero mean coherence across channels.
#[test]
fn test_resonance_of_unrelated_words_is_low() {
    let a = SpectralPhasor::seeded("photosynthesis", 1.0, 1);
    let b = SpectralPhasor::seeded("mortgage", 1.0, 1);
    assert!(a.resonance(&b).abs() < 0.35, "got {}", a.resonance(&b));
}

#[test]
fn test_channel_roundtrip() {
    let mut p = SpectralPhasor::seeded("rust", 1.0, 1);
    p.set_theta(37, 2.5);
    assert!((p.theta(37) - 2.5).abs() < 0.03);
    p.set_theta(0, 1.25);
    assert!((p.phase - 1.25).abs() < 0.03);
}

#[test]
fn test_bind_unbind_roundtrip() {
    let filler = SpectralPhasor::seeded("dog", 1.0, 1);
    let role = SpectralPhasor::seeded("__SUBJ", 1.0, 1);
    let bound = filler.bind(&role);
    let recovered = bound.unbind(&role);
    assert!(
        recovered.resonance(&filler) > 0.99,
        "resonance {}",
        recovered.resonance(&filler)
    );
    // and the bound form must NOT look like the filler
    assert!(bound.resonance(&filler) < 0.9);
}

#[test]
fn test_observe_is_log_frequency() {
    let mut p = SpectralPhasor::seeded("the", 1.0, 1);
    for _ in 0..40 { p.observe(); }
    let a40 = p.amplitude;
    for _ in 0..100_000 { p.observe(); }
    assert!(p.amplitude > a40, "amplitude must keep rising past 1000 touches");
    assert!(p.amplitude <= AMPLITUDE_MAX);
}

#[test]
fn test_band_n_capped() {
    let low = SpectralPhasor::new(1.0, 1.0, 13);
    let high = SpectralPhasor::new(1.0, 1.0, 100_000);
    assert!((low.effective_phase() - high.effective_phase()).abs() < 1e-12);
}

#[test]
fn test_evidence_packs_and_saturates() {
    let mut p = SpectralPhasor::seeded("evidence", 1.0, 1);
    assert!(!p.has_evidence());
    assert_eq!(p.evidence_total(), 0.0);
    for k in 0..PHASE_CHANNELS {
        p.set_evidence_count(k, (k * 977) as u32);
    }
    for k in 0..PHASE_CHANNELS {
        assert_eq!(p.evidence_count(k), (k * 977) as u32, "channel {k}");
    }
    p.deposit(3, u32::MAX);
    assert_eq!(p.evidence_count(3), u32::MAX, "saturates at 32 bits");
    assert_eq!(p.evidence_count(2), 2 * 977, "neighbours untouched");
    assert!((p.evidence(0) - 0.0).abs() < 1e-12);
    assert!((p.evidence(1) - (1.0 + 977.0f64).ln()).abs() < 1e-12);
    // Phases are untouched by evidence writes.
    let q = SpectralPhasor::seeded("evidence", 1.0, 1);
    assert!((p.resonance(&q) - 1.0).abs() < 1e-12);
}

/// With no evidence anywhere, the weighted comparison is the plain one —
/// every pre-evidence measurement stays reproducible.
#[test]
fn test_similarity_without_evidence_is_resonance() {
    let a = SpectralPhasor::seeded("alpha", 1.0, 1);
    let b = SpectralPhasor::seeded("beta", 1.0, 1);
    assert!((a.similarity(&b) - a.resonance(&b)).abs() < 1e-12);
    let mut c = a;
    c.deposit(5, 3);
    // One side without evidence still falls back.
    assert!((c.similarity(&b) - c.resonance(&b)).abs() < 1e-12);
}

/// The point of the magnitude: two words that agree on the one relation they
/// share should score high even when 63 unrelated channels disagree.
#[test]
fn test_similarity_compares_only_shared_relations() {
    let mut a = SpectralPhasor::seeded("king", 1.0, 1);
    let mut b = SpectralPhasor::seeded("queen", 1.0, 1);
    // Same angle on channel 9, both sides, random everywhere else.
    a.set_theta(9, 1.0);
    b.set_theta(9, 1.0);
    a.set_theta_target(9, 1.0);
    b.set_theta_target(9, 1.0);
    a.deposit(9, 50);
    b.deposit(9, 50);
    assert!(a.resonance(&b).abs() < 0.4, "dense mean is noise: {}", a.resonance(&b));
    assert!(a.similarity(&b) > 0.99, "weighted sees the agreement: {}", a.similarity(&b));
    assert!((a.on(&b, 9) - 1.0).abs() < 1e-9);
    assert_eq!(a.on(&b, 10), 0.0, "no evidence, no vote");
    assert_eq!(a.shared_relation(&b), Some(9));
    // Opposite angles on the shared channel score negative.
    b.set_theta(9, 1.0 + std::f64::consts::PI);
    b.set_theta_target(9, 1.0 + std::f64::consts::PI);
    assert!(a.similarity(&b) < -0.99);
    // `on` is directed: a's source side against b's target side.
    b.set_theta_target(9, 1.0);
    assert!((a.on(&b, 9) - 1.0).abs() < 1e-9);
    assert!((b.on(&a, 9) + 1.0).abs() < 1e-9);
}

/// Bind and unbind carry evidence through: a composed identity keeps the
/// relation profile of what it was composed from.
#[test]
fn test_bind_preserves_evidence() {
    let mut filler = SpectralPhasor::seeded("filler", 1.0, 1);
    filler.deposit(4, 7);
    let role = SpectralPhasor::seeded("role", 1.0, 1);
    let bound = filler.bind(&role);
    assert_eq!(bound.evidence_count(4), 7);
    let back = bound.unbind(&role);
    assert_eq!(back.evidence_count(4), 7);
    assert!(back.resonance(&filler) > 0.99);
}

#[test]
fn test_legacy_phasor_gets_channels() {
    let mut p = SpectralPhasor { phase: 1.75, amplitude: 1.3, band_n: 4, count: 9, packed: [0; PACKED_WORDS], evid: [0; EVIDENCE_WORDS], tgt: [0; PACKED_WORDS] };
    assert!(p.channels_unset());
    p.ensure_channels("ownership");
    assert!(!p.channels_unset());
    assert!((p.phase - 1.75).abs() < 0.03, "learned phase must survive");
}

#[test]
fn test_torus_reads_independent_channels() {
    let a = SpectralPhasor::seeded("alpha", 1.0, 1);
    let t = TorusPhasor::from_spectral(&a);
    // harmonics must not all be a fixed function of base_phase
    let distinct: std::collections::HashSet<u64> =
        t.harmonics.iter().map(|h| (h * 1000.0) as u64).collect();
    assert!(distinct.len() > TORUS_HARMONICS_COUNT / 2);
}
