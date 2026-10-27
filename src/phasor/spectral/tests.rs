#[allow(unused_imports)]
use super::*;

/// The seed must not encode spelling.
///
/// `cow`, `cod` and `cot` hash to `f604f819…`, `f604eb19…`, `f604fb19…`:
/// FNV-1a has not avalanched after three bytes, so its high bits still carry
/// the prefix. Reading `h >> 11` from that gave those three words the same
/// phase on 63 of 64 channels and a resonance of 1.0000, and every word in
/// the lexicon started life positioned by how it was spelled.
#[test]
fn test_seeds_do_not_cluster_by_spelling_prefix() {
    let cow = SpectralPhasor::seeded("cow", 1.0, 1);
    for w in ["cod", "cot", "cob", "coo", "com"] {
        let r = cow.resonance(&SpectralPhasor::seeded(w, 1.0, 1));
        assert!(
            r.abs() < 0.35,
            "cow/{w} seeded resonance {r:.4}: the seed is encoding spelling"
        );
    }

    // Longer shared prefixes are the harder case — more rounds of the hash
    // agree before the words diverge.
    let a = SpectralPhasor::seeded("consideration", 1.0, 1);
    for w in ["considerations", "considerate", "considering", "considered"] {
        let r = a.resonance(&SpectralPhasor::seeded(w, 1.0, 1));
        assert!(r.abs() < 0.35, "consideration/{w} seeded resonance {r:.4}");
    }
}

/// Seeding must spread, not merely differ.
///
/// Pairwise resonance over a sample of words should look like the mean of 64
/// independent cosines: centred on zero with a small spread. A distribution
/// shifted away from zero means the seeds share structure.
#[test]
fn test_seed_resonance_is_centred_on_zero() {
    let words: Vec<String> = (0..300).map(|i| format!("word{i}")).collect();
    let ps: Vec<SpectralPhasor> =
        words.iter().map(|w| SpectralPhasor::seeded(w, 1.0, 1)).collect();

    let mut rs = Vec::new();
    for i in 0..ps.len() {
        for j in (i + 1)..ps.len() {
            rs.push(ps[i].resonance(&ps[j]));
        }
    }
    let mean = rs.iter().sum::<f64>() / rs.len() as f64;
    let worst = rs.iter().cloned().fold(0.0f64, |a, b| a.max(b.abs()));
    assert!(mean.abs() < 0.02, "mean pairwise seed resonance {mean:.4}");
    assert!(worst < 0.6, "some pair of seeds is nearly identical: {worst:.4}");
}

#[test]
fn test_band_resonance_over_the_whole_range_equals_plain_resonance() {
    let a = SpectralPhasor::seeded("alpha", 1.0, 1);
    let b = SpectralPhasor::seeded("beta", 1.0, 1);
    let full = a.resonance(&b);
    let band = a.resonance_band(&b, 0, PHASE_CHANNELS);
    assert!((full - band).abs() < 1e-12, "{full} vs {band}");
}

#[test]
fn test_bands_partition_the_channels() {
    let a = SpectralPhasor::seeded("gamma", 1.0, 1);
    let b = SpectralPhasor::seeded("delta", 1.0, 1);
    // Eight bands of eight average back to the whole.
    let mean: f64 = (0..8)
        .map(|i| a.resonance_band(&b, i * 8, (i + 1) * 8))
        .sum::<f64>()
        / 8.0;
    assert!((mean - a.resonance(&b)).abs() < 1e-12);
    // An empty band is not silently a perfect match.
    assert_eq!(a.resonance_band(&b, 5, 5), 0.0);
}

#[test]
fn test_theta_byte_is_the_stored_quantisation_of_theta() {
    let p = SpectralPhasor::seeded("epsilon", 1.0, 1);
    for k in 0..PHASE_CHANNELS {
        let from_byte = p.theta_byte(k) as f64 * TWO_PI / CHANNEL_QUANTA;
        let d = (from_byte - p.theta(k)).abs();
        assert!(d < 1e-9 || (d - TWO_PI).abs() < 1e-9, "channel {k}: {from_byte} vs {}", p.theta(k));
    }
}

/// The lookup table must be the real cosine, or every resonance in the
/// engine is quietly wrong. Generated tables rot; this recomputes.
#[test]
fn test_cos_table_matches_cos() {
    for (i, &cos) in COS_BYTE.iter().enumerate() {
        let expected = (i as f64 * TWO_PI / CHANNEL_QUANTA).cos();
        assert!(
            (cos - expected).abs() < 1e-15,
            "COS_BYTE[{}] = {} but cos = {}",
            i,
            cos,
            expected
        );
    }
}

/// The fast path must agree with the definition it replaced.
#[test]
fn test_resonance_matches_the_transcendental_form() {
    let words = ["cat", "dog", "grandmother", "oscillate", "phi", "a"];
    for a in &words {
        for b in &words {
            let (pa, pb) = (
                SpectralPhasor::seeded(a, 1.0, 1),
                SpectralPhasor::seeded(b, 1.0, 1),
            );
            let reference = {
                let mut sum = 0.0;
                for k in 0..PHASE_CHANNELS {
                    sum += (pa.theta(k) - pb.theta(k)).cos();
                }
                (sum / PHASE_CHANNELS as f64).clamp(-1.0, 1.0)
            };
            assert!(
                (pa.resonance(&pb) - reference).abs() < 1e-12,
                "{}/{}: table {} vs cos {}",
                a,
                b,
                pa.resonance(&pb),
                reference
            );
        }
    }
}
pub(super) use phiton::units::{ALPHA, TWO_PI};
pub(super) use std::f64::consts::PI as PI_CONST;

#[test]
fn test_phase_wrapping() {
    let p = SpectralPhasor::new(3.0 * PI_CONST, 1.0, 0);
    assert!((p.phase - PI_CONST).abs() < 0.03);
}

#[test]
fn test_phase_zero() {
    let p = SpectralPhasor::new(0.0, 1.0, 0);
    assert!(p.phase.abs() < 0.03);
}

#[test]
fn test_phase_2pi_wraps_to_zero() {
    let p = SpectralPhasor::new(TWO_PI, 1.0, 0);
    assert!(p.phase.abs() < 0.03);
}

#[test]
fn test_to_complex_amplitude() {
    let p = SpectralPhasor::new(0.0, 2.5, 0);
    assert!((p.to_complex().norm() - 2.5).abs() < 1e-10);
}

#[test]
fn test_to_complex_band_shift() {
    let p0 = SpectralPhasor::new(0.0, 1.0, 0);
    let p1 = SpectralPhasor::new(0.0, 1.0, 1);
    assert!((p1.to_complex().arg() - p0.to_complex().arg() - ALPHA).abs() < 1e-10);
}

/// Seeding must depend on word identity, not word length.
#[test]
fn test_seed_distinguishes_same_length_words() {
    let a = SpectralPhasor::seeded("cat", 1.0, 1);
    let b = SpectralPhasor::seeded("dog", 1.0, 1);
    let c = SpectralPhasor::seeded("war", 1.0, 1);
    assert!((a.phase - b.phase).abs() > 0.05);
    assert!((b.phase - c.phase).abs() > 0.05);
    assert!((a.phase - c.phase).abs() > 0.05);
}

#[cfg(test)]
mod seed;

#[cfg(test)]
mod width;
