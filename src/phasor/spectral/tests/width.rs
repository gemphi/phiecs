#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

/// 32-bit lanes: a count past 65,535 keeps counting, and the v5 layout
/// widens losslessly.
#[test]
fn evidence_counts_past_u16() {
    let mut p = SpectralPhasor::seeded("the", 1.0, 1);
    p.set_evidence_count(0, 65_535);
    p.deposit(0, 10);
    assert_eq!(p.evidence_count(0), 65_545);
    p.set_evidence_count(63, 4_000_000_000);
    assert_eq!(p.evidence_count(63), 4_000_000_000);
    assert_eq!(p.evidence_count(1), 0, "neighbouring lane untouched");
    let mut old = [0u64; EVIDENCE_WORDS_V5];
    for k in 0..PHASE_CHANNELS {
        old[k / 4] |= ((k as u64 * 977) & 0xFFFF) << (16 * (k % 4));
    }
    let wide = widen_evidence(old);
    let q = SpectralPhasor::from_packed_with_evidence(p.packed(), wide, 1.0, 1, 1);
    for k in 0..PHASE_CHANNELS {
        assert_eq!(q.evidence_count(k) as u64, (k as u64 * 977) & 0xFFFF);
    }
}
