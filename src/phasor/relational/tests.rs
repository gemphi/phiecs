#[allow(unused_imports)]
use super::*;

use phiton::p64::words::Tokenizer;

fn docs(raw: &[&str]) -> Vec<Vec<String>> {
    raw.iter().map(|s| Tokenizer::tokenize(s)).collect()
}

/// The point of the module: an observation moves the channel of the relation
/// it observed, and no other.
///
/// The invariant is *evidence implies movement, and only that* — not "one
/// channel moves". A single sentence exhibits several relations at once:
/// `alpha of beta` puts `alpha` next to `of` (an offset relation) as well as
/// two away from `beta` (the `of` relation), so two channels legitimately
/// fire. The first version of this test asserted the narrower thing and
/// failed on correct behaviour.
#[test]
fn test_training_moves_only_the_channels_it_has_evidence_for() {
    let d = docs(&["alpha of beta", "alpha of beta"]);
    let v = RelationVocab::learn(&d);
    let k = v.channel(&d[0], 0, 2).expect("a channel for the `of` relation");

    let before = RelationalPhasor::seeded("alpha");
    let f = RelationalFacet::train(&d, &v, 3);
    let after = &f.words["alpha"];

    assert!(after.mag[k] > 0.0, "the `of` relation must leave evidence");
    assert!(
        (after.theta[k] - before.theta[k]).abs() > 1e-9,
        "the relation's own channel must move"
    );

    let mut moved = 0usize;
    for c in 0..PHASE_CHANNELS {
        let shifted = (after.theta[c] - before.theta[c]).abs() > 1e-12;
        match after.mag[c] > 0.0 {
            // Evidence and movement go together, in both directions.
            true => {
                moved += 1;
                assert!(shifted, "channel {c} has evidence but never moved");
            }
            false => assert!(!shifted, "channel {c} moved with no evidence"),
        }
    }
    assert!(
        moved < PHASE_CHANNELS / 4,
        "a three-word sentence exhibits a handful of relations, not {moved} of \
         {PHASE_CHANNELS} — that would be the symmetric update this replaces"
    );
}

/// Two words seen under the same relation agree on that channel, and two seen
/// under different relations do not.
#[test]
fn test_the_channel_holds_its_own_relation() {
    let mut raw: Vec<String> = Vec::new();
    for _ in 0..40 {
        raw.push("king of realm".into());
        raw.push("queen of realm".into());
        raw.push("bread in oven".into());
    }
    let d: Vec<Vec<String>> = raw.iter().map(|s| Tokenizer::tokenize(s)).collect();
    let v = RelationVocab::learn(&d);
    let f = RelationalFacet::train(&d, &v, 6);

    let of = v.channel(&Tokenizer::tokenize("king of realm"), 0, 2).unwrap();
    let in_ = v.channel(&Tokenizer::tokenize("bread in oven"), 0, 2).unwrap();
    assert_ne!(of, in_, "different markers must take different channels");

    // `king` and `queen` are both pulled toward `realm` on the `of` channel,
    // so they converge there.
    assert!(
        f.on("king", "queen", of) > 0.9,
        "co-related words should agree on their shared relation: {}",
        f.on("king", "queen", of)
    );
    // And `king` has no evidence at all on the `in` channel.
    assert_eq!(f.words["king"].mag[in_], 0.0);
    assert_eq!(f.on("king", "bread", in_), 0.0);
}

/// Magnitude must be able to change a comparison — the failure that made the
/// shipped representation's complex numbers inert.
#[test]
fn test_per_channel_magnitude_changes_the_similarity() {
    let mut a = RelationalPhasor::seeded("a");
    let mut b = RelationalPhasor::seeded("b");
    for k in 0..PHASE_CHANNELS {
        a.theta[k] = 0.0;
        b.theta[k] = match k == 3 {
            true => 0.0,
            false => std::f64::consts::PI,
        };
        a.mag[k] = 1.0;
        b.mag[k] = 1.0;
    }
    // Evidence spread evenly: the one agreeing channel is outvoted.
    let spread = a.similarity(&b);

    // The same angles, with the evidence concentrated where they agree.
    for k in 0..PHASE_CHANNELS {
        let w = match k == 3 {
            true => 8.0,
            false => 0.05,
        };
        a.mag[k] = w;
        b.mag[k] = w;
    }
    let concentrated = a.similarity(&b);

    assert!(
        concentrated > spread + 0.5,
        "magnitude must move the score: {spread:.4} -> {concentrated:.4}"
    );
    // A per-word scalar could not do this: it cancels in the normalisation.
    assert!(spread < 0.0 && concentrated > 0.0, "the sign should flip");
}
