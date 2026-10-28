#[allow(unused_imports)]
use super::*;

impl Storage {

    /// Rebuilds an interned facet from the old string-keyed tables.
    pub(super) fn from_string_keyed(
        lexicon: HashMap<String, LegacyPhasor>,
        bigrams: HashMap<String, HashMap<String, u32>>,
        trigrams: HashMap<String, HashMap<String, u32>>,
        phase_lags: HashMap<String, HashMap<String, f64>>,
        grounded_version: u32,
    ) -> Facet {
        let mut facet = Facet::new();
        facet.lexicon = lexicon.into_iter().map(|(w, p)| (w, p.into_phasor())).collect();
        facet.grounded_version = grounded_version;

        for (a, followers) in bigrams {
            let ai = facet.vocab.intern(&a);
            let mut list: Vec<(WordId, u32)> = followers
                .into_iter()
                .map(|(b, c)| (facet.vocab.intern(&b), c))
                .collect();
            list.sort_unstable_by_key(|(k, _)| *k);
            facet.bigrams.insert(ai, list);
        }

        for (key, followers) in trigrams {
            // v2 trigram keys were the two context words joined by a space.
            let mut parts = key.splitn(2, ' ');
            let (a, b) = match (parts.next(), parts.next()) {
                (Some(a), Some(b)) => (a.to_string(), b.to_string()),
                _ => continue,
            };
            let k = (facet.vocab.intern(&a), facet.vocab.intern(&b));
            let mut list: Vec<(WordId, u32)> = followers
                .into_iter()
                .map(|(c, n)| (facet.vocab.intern(&c), n))
                .collect();
            list.sort_unstable_by_key(|(k, _)| *k);
            facet.trigrams.insert(k, list);
        }

        for (a, targets) in phase_lags {
            let ai = facet.vocab.intern(&a);
            for (b, v) in targets {
                let bi = facet.vocab.intern(&b);
                facet.phase_lags.insert((ai, bi), v as f32);
            }
        }

        facet
    }
}
