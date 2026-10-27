#[allow(unused_imports)]
use super::*;

impl RelationVocab {
    /// Tally what appears between word pairs and gives the commonest their own
    /// channels.
    ///
    /// # Why the gap is not fixed at one word
    ///
    /// The first version took the marker to be the single word between a pair,
    /// and only when there was exactly one. `dog eats donut` worked. `a dog eats
    /// a donut` did not, because the determiner makes the gap three, and English
    /// puts a determiner in that position constantly — so the rule found the
    /// relation in the sentence a grammarian would write and missed it in the
    /// sentence a person would say. A unit test in `flow` caught this by
    /// asserting the marker was `eats` and getting `offset+3`.
    ///
    /// The fix is not a wider fixed gap, which would let `a` name relations. It
    /// is to skip the words that cannot be naming anything and see what is left.
    /// Which words those are is read off the corpus by rank — the same criterion
    /// `wave::STOPWORD_RANK` already uses — and not from a list written here,
    /// because a hand-written list of English function words is a claim about
    /// English that this module has no way to check.
    pub fn learn(docs: &[Vec<String>]) -> Self {
        // Pass one: how often does each word occur at all, and how often does it
        // occur as the single word joining two others?
        let mut total: HashMap<&str, usize> = HashMap::new();
        for toks in docs {
            for t in toks {
                *total.entry(t.as_str()).or_insert(0) += 1;
            }
        }
        // How often each word stands between a pair the marker rule will actually
        // consult — the same gaps `marker` reads, so the ranking is a count of
        // the thing being ranked.
        //
        // An earlier version restricted this to words standing between two
        // *content* words, on the reasoning that a relation joins two things and
        // the things are the rare words. It is a good argument and it is wrong
        // about English, in a way that kills exactly the case it was written for:
        // in `the dog eats the donut` the neighbours of `eats` are `dog` and
        // `the`, because a determiner always stands next to its noun. Under that
        // restriction `eats` can never be counted as linking anything, and the
        // sentence that motivated the whole rule is the sentence it cannot see.
        let mut linking: HashMap<&str, usize> = HashMap::new();
        for toks in docs {
            for i in 0..toks.len() {
                for j in (i + 2)..(i + 1 + MARKER_MAX_GAP).min(toks.len()) {
                    for t in &toks[(i + 1)..j] {
                        *linking.entry(t.as_str()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Ranked by how often the word links, in absolute terms. `bin/markers`
        // prints both this ranking and the obvious alternative, and the two are
        // not close: absolute count gives `the and of to in a was by on for with
        // that as from at`, which is the inventory of English relational markers
        // with the determiners and copulas included. The share of a word's
        // occurrences spent linking — tried here, and wrong — gives `nautical
        // stoppani cu facto botanist`, every one of them a word that occurs a
        // dozen times and happens to occur between two rare words each time. A
        // ratio rewards a small denominator, and the words that name relations
        // are the words with the largest denominators in the language.
        // No minimum count is needed: ranking is by absolute linking count, so a
        // word seen five times ranks below one seen thirty thousand times and
        // falls outside the channels on its own. A threshold here would do
        // nothing on a corpus and would silently empty the vocabulary on a small
        // one, which is how two unit tests failed before it was removed.
        let mut ranked: Vec<(&str, usize)> = linking.into_iter().collect();
        let _ = &total;
        // Sorted by count, ties by word, so the assignment is reproducible.
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

        let room = MARKER_CHANNELS_MAX.min(PHASE_CHANNELS - OFFSET_CHANNELS);
        let mut v = Self::default();
        for (w, _) in ranked.into_iter().take(room) {
            v.inner.declare(w);
        }
        v
    }
}
