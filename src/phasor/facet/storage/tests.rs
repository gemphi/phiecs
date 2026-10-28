#[allow(unused_imports)]
use super::*;


#[test]
fn test_words_outside_the_ngram_tables_survive() {
    // A word added to the lexicon but never seen in a bigram has no id yet.
    let mut facet = Facet::new();
    facet.get_or_init("orphan");
    let path = std::env::temp_dir().join("phiano_storage_orphan.chroma");
    let path = path.to_str().unwrap();
    Storage::save(&facet, path).unwrap();
    let back = Storage::load(path).unwrap();
    assert!(back.contains_word("orphan"));
}
