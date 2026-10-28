#[allow(unused_imports)]
use super::*;

impl Storage {
    /// Saves a facet to a binary .chroma file, atomically.
    ///
    /// `rename` is atomic on every mainstream filesystem, so an interrupted save
    /// can no longer leave a truncated model that fails to load and silently
    /// starts the next session from an empty lexicon.
    pub fn save(facet: &Facet, path: &str) -> Result<()> {
        let tmp = format!("{}.tmp", path);

        // Every lexicon word needs an id. Words that only ever appeared in the
        // lexicon (never in an n-gram) have none yet, so they are appended here
        // rather than requiring a mutable facet to save.
        let mut words: Vec<String> = facet.vocab.words().to_vec();
        let mut extra: HashMap<&str, WordId> = HashMap::new();
        for w in facet.lexicon.keys() {
            if facet.vocab.id(w).is_none() {
                extra.insert(w.as_str(), words.len() as WordId);
                words.push(w.clone());
            }
        }
        let id_of = |w: &str| -> WordId {
            facet.vocab.id(w).unwrap_or_else(|| extra[w])
        };

        let body = BodyV6Ref {
            words: &words,
            lexicon: facet.lexicon.iter().map(|(w, p)| (id_of(w), DiskPhasorV6::from(p))).collect(),
            bigrams: facet.bigrams.iter().map(|(k, v)| (*k, v)).collect(),
            trigrams: facet.trigrams.iter().map(|(k, v)| (*k, v)).collect(),
            phase_lags: facet.phase_lags.iter().map(|(k, v)| (*k, *v)).collect(),
            grounded_version: facet.grounded_version,
            relations: facet.relations.entries(),
        };

        {
            let file = File::create(&tmp)?;
            let mut writer = BufWriter::new(file);
            let header = ChromaHeader {
                version: FORMAT_VERSION,
                vocabulary_size: facet.vocabulary_size(),
                fine_structure_alpha: ALPHA,
            };
            bincode::serialize_into(&mut writer, &header)
                .map_err(Error::other)?;
            bincode::serialize_into(&mut writer, &body)
                .map_err(Error::other)?;
        }

        std::fs::rename(&tmp, path)
    }
}
