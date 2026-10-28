#[allow(unused_imports)]
use super::*;

/// Header metadata prepended to binary .chroma files.
///
/// Written and read on its own, ahead of the body, so the version can be
/// inspected before deciding how to parse the rest.
#[derive(Serialize, Deserialize, Debug)]
pub struct ChromaHeader {
    pub version: u32,
    pub vocabulary_size: usize,
    pub fine_structure_alpha: f64,
}

/// On-disk phasor.
///
/// `phase` is omitted: it is by construction the angle of channel 0, so storing
/// it duplicates eight bytes per word for a value that is recovered exactly.
/// `amplitude` narrows to f32 — its useful range is [0.3, 2.0] and it is
/// compared, never accumulated, so 24 bits of mantissa is far more than the
/// quantity carries.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub(super) struct DiskPhasor {
    pub(super) packed: [u64; 8],
    pub(super) amplitude: f32,
    pub(super) band_n: u32,
    pub(super) count: u32,
}

/// On-disk phasor, v4: adds the per-channel evidence counts.
///
/// 64 channels × 16 bits = 128 bytes per word on top of the 64 bytes of phase.
/// bincode is not self-describing, so a new field is a new version rather than
/// a `serde(default)`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub(super) struct DiskPhasorV4 {
    pub(super) packed: [u64; 8],
    pub(super) evid: [u64; EVIDENCE_WORDS_V5],
    pub(super) amplitude: f32,
    pub(super) band_n: u32,
    pub(super) count: u32,
}

/// On-disk phasor, v5: v4 plus the target side of every channel.
///
/// 64 more bytes per word. A v4 file reads back with both sides equal, which
/// is exactly the state a v5 phasor starts in.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub(super) struct DiskPhasorV5 {
    pub(super) packed: [u64; 8],
    pub(super) tgt: [u64; 8],
    pub(super) evid: [u64; EVIDENCE_WORDS_V5],
    pub(super) amplitude: f32,
    pub(super) band_n: u32,
    pub(super) count: u32,
}

/// On-disk phasor, v6: v5 with 32-bit evidence counts.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub(super) struct DiskPhasorV6 {
    pub(super) packed: [u64; 8],
    pub(super) tgt: [u64; 8],
    pub(super) evid: [u64; EVIDENCE_WORDS],
    pub(super) amplitude: f32,
    pub(super) band_n: u32,
    pub(super) count: u32,
}

/// One trigram row on disk: a context pair and its continuation counts.
pub(super) type TrigramRow = ((WordId, WordId), Vec<(WordId, u32)>);
/// A [`TrigramRow`] borrowed from the live model when saving.
pub(super) type TrigramRowRef<'a> = ((WordId, WordId), &'a Vec<(WordId, u32)>);

#[derive(Serialize)]
pub(super) struct BodyV6Ref<'a> {
    pub(super) words: &'a [String],
    pub(super) lexicon: Vec<(WordId, DiskPhasorV6)>,
    pub(super) bigrams: Vec<(WordId, &'a Vec<(WordId, u32)>)>,
    pub(super) trigrams: Vec<TrigramRowRef<'a>>,
    pub(super) phase_lags: Vec<((WordId, WordId), f32)>,
    pub(super) grounded_version: u32,
    pub(super) relations: Vec<(String, usize)>,
}

#[derive(Deserialize)]
pub(super) struct BodyV6 {
    pub(super) words: Vec<String>,
    pub(super) lexicon: Vec<(WordId, DiskPhasorV6)>,
    pub(super) bigrams: Vec<(WordId, Vec<(WordId, u32)>)>,
    pub(super) trigrams: Vec<TrigramRow>,
    pub(super) phase_lags: Vec<((WordId, WordId), f32)>,
    pub(super) grounded_version: u32,
    pub(super) relations: Vec<(String, usize)>,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub(super) struct BodyV5Ref<'a> {
    pub(super) words: &'a [String],
    pub(super) lexicon: Vec<(WordId, DiskPhasorV5)>,
    pub(super) bigrams: Vec<(WordId, &'a Vec<(WordId, u32)>)>,
    pub(super) trigrams: Vec<TrigramRowRef<'a>>,
    pub(super) phase_lags: Vec<((WordId, WordId), f32)>,
    pub(super) grounded_version: u32,
    pub(super) relations: Vec<(String, usize)>,
}

#[derive(Deserialize)]
pub(super) struct BodyV5 {
    pub(super) words: Vec<String>,
    pub(super) lexicon: Vec<(WordId, DiskPhasorV5)>,
    pub(super) bigrams: Vec<(WordId, Vec<(WordId, u32)>)>,
    pub(super) trigrams: Vec<TrigramRow>,
    pub(super) phase_lags: Vec<((WordId, WordId), f32)>,
    pub(super) grounded_version: u32,
    pub(super) relations: Vec<(String, usize)>,
}

/// The phasor layout v1 and v2 bodies were written with, before evidence.
///
/// Those bodies serialised `SpectralPhasor` directly; the struct has since
/// grown a field, and bincode reads fields positionally, so the old layout has
/// to be spelled out to read the old files.
#[derive(Deserialize, Debug, Clone, Copy)]
pub(super) struct LegacyPhasor {
    pub(super) phase: f64,
    pub(super) amplitude: f64,
    pub(super) band_n: u32,
    #[serde(default)]
    pub(super) count: u32,
    #[serde(default)]
    pub(super) packed: [u64; 8],
}

// ── v4: evidence + relation inventory ──────────────────────────────────────

#[allow(dead_code)]
#[derive(Serialize)]
pub(super) struct BodyV4Ref<'a> {
    pub(super) words: &'a [String],
    pub(super) lexicon: Vec<(WordId, DiskPhasorV4)>,
    pub(super) bigrams: Vec<(WordId, &'a Vec<(WordId, u32)>)>,
    pub(super) trigrams: Vec<TrigramRowRef<'a>>,
    pub(super) phase_lags: Vec<((WordId, WordId), f32)>,
    pub(super) grounded_version: u32,
    pub(super) relations: Vec<(String, usize)>,
}

#[derive(Deserialize)]
pub(super) struct BodyV4 {
    pub(super) words: Vec<String>,
    pub(super) lexicon: Vec<(WordId, DiskPhasorV4)>,
    pub(super) bigrams: Vec<(WordId, Vec<(WordId, u32)>)>,
    pub(super) trigrams: Vec<TrigramRow>,
    pub(super) phase_lags: Vec<((WordId, WordId), f32)>,
    pub(super) grounded_version: u32,
    pub(super) relations: Vec<(String, usize)>,
}

// ── v3: interned ───────────────────────────────────────────────────────────

/// Read-only: v3 files are migrated forward on load and written back as v4.
#[derive(Deserialize)]
pub(super) struct BodyV3 {
    pub(super) words: Vec<String>,
    pub(super) lexicon: Vec<(WordId, DiskPhasor)>,
    pub(super) bigrams: Vec<(WordId, Vec<(WordId, u32)>)>,
    pub(super) trigrams: Vec<TrigramRow>,
    pub(super) phase_lags: Vec<((WordId, WordId), f32)>,
    pub(super) grounded_version: u32,
}

// ── v2: string-keyed n-grams ───────────────────────────────────────────────

#[derive(Deserialize)]
pub(super) struct BodyV2 {
    pub(super) lexicon: HashMap<String, LegacyPhasor>,
    #[serde(default)]
    pub(super) bigrams: HashMap<String, HashMap<String, u32>>,
    #[serde(default)]
    pub(super) trigrams: HashMap<String, HashMap<String, u32>>,
    #[serde(default)]
    pub(super) phase_lags: HashMap<String, HashMap<String, f64>>,
    #[serde(default)]
    pub(super) grounded_version: u32,
}
