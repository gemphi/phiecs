#[allow(unused_imports)]
use super::*;
use phiton::p64::words::Tokenizer;

fn docs(raw: &[&str]) -> Vec<Vec<String>> {
    raw.iter().map(|s| Tokenizer::tokenize(s)).collect()
}
#[test]
fn test_relation_vocab_is_bounded_and_reproducible() {
    let d = docs(&[
        "a of b", "c of d", "e in f", "g in h", "i on j", "k at l", "m of n",
    ]);
    let v1 = RelationVocab::learn(&d);
    let v2 = RelationVocab::learn(&d);
    assert!(v1.markers() + OFFSET_CHANNELS <= PHASE_CHANNELS);
    for k in 0..PHASE_CHANNELS {
        assert_eq!(v1.name(k), v2.name(k), "channel {k} assignment is not stable");
    }
    // Offsets own the low channels.
    assert_eq!(v1.name(0), "offset+1");
}
