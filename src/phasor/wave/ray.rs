#[allow(unused_imports)]
use super::*;

impl Wave {

    /// Ray cast from an arbitrary complex wave.
    ///
    /// A query wave carries only one angle, so this compares channel 0 —
    /// angular distance, with familiarity as an explicit, tunable secondary
    /// term rather than an artefact of the algebra.
    pub fn ray_cast(facet: &Facet, wave: c64, top_k: usize) -> Vec<(String, f64)> {
        if wave.norm() < 1e-12 {
            return vec![];
        }
        let q = wave.arg();

        let floor = Self::pool_floor(facet, top_k);
        let ceiling = Self::pool_ceiling(facet);
        let mut hits: Vec<(&String, f64)> = facet
            .lexicon
            .par_iter()
            .filter(|(_, p)| p.count >= floor && p.count <= ceiling)
            .map(|(word, phasor)| {
                let d = (q - phasor.effective_phase()).cos().max(0.0);
                (word, 1.0 - d)
            })
            .collect();

        Self::take_smallest(&mut hits, top_k)
    }

    /// Ray cast from a multi-channel query wave (see [`Wave::sentence_channels`]).
    ///
    /// This is the retrieval the torus representation exists for: the query and
    /// every candidate are compared across all `PHASE_CHANNELS`, so the number
    /// of distinguishable outcomes is not bounded by one circle's resolution.
    pub fn ray_cast_channels(facet: &Facet, query: &[c64], top_k: usize) -> Vec<(String, f64)> {
        let qs: Vec<f64> = query.iter().map(|z| z.arg()).collect();
        if qs.is_empty() {
            return vec![];
        }
        let floor = Self::pool_floor(facet, top_k);

        let ceiling = Self::pool_ceiling(facet);

        let mut hits: Vec<(&String, f64)> = facet
            .lexicon
            .par_iter()
            .filter(|(_, p)| p.count >= floor && p.count <= ceiling)
            .map(|(word, phasor)| {
                let mut sum = 0.0;
                for (k, &qk) in qs.iter().enumerate() {
                    sum += (qk - phasor.theta(k)).cos();
                }
                let sim = 0.5 * (sum / qs.len() as f64 + 1.0);
                (word, 1.0 - sim)
            })
            .collect();

        Self::take_smallest(&mut hits, top_k)
    }

    /// Words this frequent are excluded from retrieval results.
    ///
    /// A query for "a large four-footed animal" used to return `is, a, in, to,
    /// of`. Function words sit near everything because they occur with
    /// everything, and no similarity measure fixes that — being common is what
    /// makes them uninformative about any query.
    ///
    /// The first attempt at a fix scaled each candidate's score by its rarity.
    /// That inverted the problem instead of solving it: the multiplier swung
    /// 20% while the similarities between candidates differ by far less, so
    /// rarity decided the ranking outright and the same query returned
    /// `columella, scalariform, superficies, submultiple`. A score multiplier is
    /// the wrong instrument.
    ///
    /// So frequency restricts the *pool* and leaves the score alone: the top
    /// [`Wave::STOPWORD_RANK`] words by count are not offered as answers, and
    /// everything below them is ranked on similarity, undistorted.
    pub const STOPWORD_RANK: usize = 120;

    /// The count above which a word is treated as a stopword, or `u32::MAX` when
    /// the vocabulary is too small for the cut to mean anything.
    pub(super) fn pool_ceiling(facet: &Facet) -> u32 {
        if facet.lexicon.len() < Self::STOPWORD_RANK * 20 {
            return u32::MAX;
        }
        let mut counts: Vec<u32> = facet.lexicon.values().map(|p| p.count).collect();
        counts.sort_unstable_by(|a, b| b.cmp(a));
        counts[Self::STOPWORD_RANK.min(counts.len() - 1)]
    }

    /// Partial-selects the `top_k` smallest deltas, then sorts just those.
    pub(super) fn take_smallest(hits: &mut Vec<(&String, f64)>, top_k: usize) -> Vec<(String, f64)> {
        if hits.len() > top_k && top_k > 0 {
            hits.select_nth_unstable_by(top_k, |a, b| {
                a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal)
            });
            hits.truncate(top_k);
        }
        hits.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        hits.iter().map(|(w, d)| ((*w).clone(), *d)).collect()
    }

    /// Binds a wave to a role angle (phase addition).
    #[inline]
    pub fn bind(z: c64, role_phase: f64) -> c64 {
        z * c64::from_polar(1.0, role_phase)
    }

    /// Unbinds a wave from a role angle (phase subtraction).
    #[inline]
    pub fn unbind(z: c64, role_phase: f64) -> c64 {
        z * c64::from_polar(1.0, -role_phase)
    }

    /// Builds a bound proposition: subject, verb and object each bound to a
    /// distinct role angle, then superposed.
    ///
    /// `query_role` recovers a filler by unbinding. This is what makes
    /// `dog bites man` and `man bites dog` different objects to the model.
    pub fn proposition(facet: &Facet, subject: &str, verb: &str, object: &str) -> c64 {
        let roles = [
            (subject, 0.0),
            (verb, GOLDEN_ANGLE),
            (object, 2.0 * GOLDEN_ANGLE),
        ];
        roles
            .iter()
            .filter_map(|(w, r)| facet.lexicon.get(*w).map(|p| (p, *r)))
            .map(|(p, r)| c64::from_polar(p.amplitude, p.effective_phase() + r))
            .sum()
    }

    /// Recovers the most likely filler of a role from a bound proposition.
    ///
    /// `slot` is 0 for subject, 1 for verb, 2 for object.
    pub fn query_role(facet: &Facet, z: c64, slot: usize, top_k: usize) -> Vec<(String, f64)> {
        Self::ray_cast(facet, Self::unbind(z, slot as f64 * GOLDEN_ANGLE), top_k)
    }

    /// Returns the configured number of phase sectors.
    pub fn sector_count() -> u16 {
        crate::phasor::facet::SECTOR_RESOLUTION
    }

    /// Maps a phase angle to a sector index.
    pub fn sector_of(phase: f64) -> u16 {
        let n = Self::sector_count();
        let normalized = phase.rem_euclid(TWO_PI);
        let sector_size = TWO_PI / n as f64;
        (normalized / sector_size).floor() as u16 % n
    }

    /// Returns the antipodal (opposite) sector index.
    pub fn opposite_sector(sector: u16) -> u16 {
        let n = Self::sector_count();
        (sector + n / 2) % n
    }

    /// Returns the sector of a word's effective phase.
    pub fn word_sector(facet: &Facet, word: &str) -> Option<u16> {
        facet.lexicon.get(word).map(|p| Self::sector_of(p.effective_phase()))
    }

    /// Returns the sector of a complex wave's phase angle.
    pub fn wave_sector(wave: c64) -> u16 {
        if wave.norm() < 1e-10 {
            return 0;
        }
        Self::sector_of(wave.arg())
    }
}
