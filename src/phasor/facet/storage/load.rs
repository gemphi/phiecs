#[allow(unused_imports)]
use super::*;

impl Storage {

    /// Loads a facet, migrating older formats forward.
    pub fn load(path: &str) -> Result<Facet> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let header: ChromaHeader = bincode::deserialize_from(&mut reader)
            .map_err(Error::other)?;

        if header.version > FORMAT_VERSION {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "{} is format v{}, this build reads v{}",
                    path, header.version, FORMAT_VERSION
                ),
            ));
        }
        if (header.fine_structure_alpha - ALPHA).abs() > 1e-12 {
            eprintln!(
                "  [warn] {} was trained with alpha={:.9}, this build uses {:.9}",
                path, header.fine_structure_alpha, ALPHA
            );
        }

        match header.version {
            6 => {
                let b: BodyV6 = bincode::deserialize_from(&mut reader)
                    .map_err(Error::other)?;
                let vocab = Vocab::from_words(b.words);
                let lexicon = b
                    .lexicon
                    .into_iter()
                    .filter_map(|(id, p)| vocab.word(id).map(|w| (w.to_string(), p.into_phasor())))
                    .collect();
                Ok(Facet {
                    lexicon,
                    vocab,
                    bigrams: b.bigrams.into_iter().collect(),
                    trigrams: b.trigrams.into_iter().collect(),
                    phase_lags: b.phase_lags.into_iter().collect(),
                    grounded_version: b.grounded_version,
                    relations: RelationVocab::from_entries(b.relations),
                    sample_pool: Vec::new(),
                    relation_pool: Vec::new(),
                })
            }
            5 => {
                let b: BodyV5 = bincode::deserialize_from(&mut reader)
                    .map_err(Error::other)?;
                let vocab = Vocab::from_words(b.words);
                let lexicon = b
                    .lexicon
                    .into_iter()
                    .filter_map(|(id, p)| vocab.word(id).map(|w| (w.to_string(), p.into_phasor())))
                    .collect();
                Ok(Facet {
                    lexicon,
                    vocab,
                    bigrams: b.bigrams.into_iter().collect(),
                    trigrams: b.trigrams.into_iter().collect(),
                    phase_lags: b.phase_lags.into_iter().collect(),
                    grounded_version: b.grounded_version,
                    relations: RelationVocab::from_entries(b.relations),
                    sample_pool: Vec::new(),
                    relation_pool: Vec::new(),
                })
            }
            4 => {
                let b: BodyV4 = bincode::deserialize_from(&mut reader)
                    .map_err(Error::other)?;
                let vocab = Vocab::from_words(b.words);
                let lexicon = b
                    .lexicon
                    .into_iter()
                    .filter_map(|(id, p)| vocab.word(id).map(|w| (w.to_string(), p.into_phasor())))
                    .collect();
                Ok(Facet {
                    lexicon,
                    vocab,
                    bigrams: b.bigrams.into_iter().collect(),
                    trigrams: b.trigrams.into_iter().collect(),
                    phase_lags: b.phase_lags.into_iter().collect(),
                    grounded_version: b.grounded_version,
                    relations: RelationVocab::from_entries(b.relations),
                    sample_pool: Vec::new(),
                    relation_pool: Vec::new(),
                })
            }
            3 => {
                let b: BodyV3 = bincode::deserialize_from(&mut reader)
                    .map_err(Error::other)?;
                eprintln!("  [migrate] v3 model has no relation evidence; channels will fill on the next training pass");
                let vocab = Vocab::from_words(b.words);
                let lexicon = b
                    .lexicon
                    .into_iter()
                    .filter_map(|(id, p)| vocab.word(id).map(|w| (w.to_string(), p.into_phasor())))
                    .collect();
                Ok(Facet {
                    lexicon,
                    vocab,
                    bigrams: b.bigrams.into_iter().collect(),
                    trigrams: b.trigrams.into_iter().collect(),
                    phase_lags: b.phase_lags.into_iter().collect(),
                    grounded_version: b.grounded_version,
                    relations: RelationVocab::default(),
                    sample_pool: Vec::new(),
                    relation_pool: Vec::new(),
                })
            }
            2 => {
                let b: BodyV2 = bincode::deserialize_from(&mut reader)
                    .map_err(Error::other)?;
                eprintln!("  [migrate] converting v2 string-keyed n-grams to interned ids");
                Ok(Self::from_string_keyed(
                    b.lexicon,
                    b.bigrams,
                    b.trigrams,
                    b.phase_lags,
                    b.grounded_version,
                ))
            }
            _ => {
                let b = Self::read_v1_lexicon(&mut reader, header.vocabulary_size)?;
                Ok(Self::from_string_keyed(
                    b.lexicon,
                    b.bigrams,
                    b.trigrams,
                    b.phase_lags,
                    b.grounded_version,
                ))
            }
        }
    }

    /// Reads a v1 body, whose phasor layout depends on when it was written.
    ///
    /// The header version stayed at 1 while `SpectralPhasor` grew `count` and
    /// then `packed`, and while the body grew n-gram tables behind the lexicon.
    /// bincode is positional, so `#[serde(default)]` never helped:
    /// `data/manifold.chroma` (216,097 words, written with the 20-byte
    /// `{phase, amplitude, band_n}` layout and 194,788 bigram contexts behind
    /// it) failed to load with a 47 GB allocation because the reader took the
    /// next word's length prefix as part of a phasor. Each candidate layout is
    /// tried in turn and accepted only if the lexicon parses to exactly the
    /// header's word count; whatever tables follow are read while they last.
    pub(super) fn read_v1_lexicon<R: std::io::Read>(
        reader: &mut R,
        vocabulary_size: usize,
    ) -> Result<V1Body> {
        let mut body = Vec::new();
        reader.read_to_end(&mut body)?;

        // Fixed-record widths after the string: (phase, amplitude, band_n)
        // = 20; + count = 24; + packed [u64; 8] = 88.
        for &record in &[20usize, 24, 88] {
            if let Some(v1) = V1Reader::parse(&body, record, vocabulary_size) {
                eprintln!(
                    "  [migrate] v1 lexicon with {}-byte phasors, {} bigram contexts, {} trigram contexts",
                    record,
                    v1.bigrams.len(),
                    v1.trigrams.len()
                );
                return Ok(v1);
            }
        }
        Err(Error::new(ErrorKind::InvalidData, "unrecognised v1 phasor layout"))
    }
}
