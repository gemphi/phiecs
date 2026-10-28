#[allow(unused_imports)]
use super::*;

impl RelationalFacet {
    /// Trains by coupling each observed pair **on its relation's channel only**.
    ///
    /// This is the whole point. The shipped trainer moves all 64 channels on
    /// every observation, which is why no channel can come to mean anything. Here
    /// an observation is evidence about one relation, so it moves one channel and
    /// deposits its evidence there.
    pub fn train(docs: &[Vec<String>], vocab: &RelationVocab, epochs: usize) -> Self {
        let mut facet = Self::default();
        for toks in docs {
            for w in toks {
                facet.counts.entry(w.clone()).and_modify(|c| *c += 1).or_insert(1);
                facet
                    .words
                    .entry(w.clone())
                    .or_insert_with(|| RelationalPhasor::seeded(w));
            }
        }

        for _ in 0..epochs.max(1) {
            for toks in docs {
                for i in 0..toks.len() {
                    for j in (i + 1)..(i + 1 + WINDOW).min(toks.len()) {
                        if toks[i] == toks[j] {
                            continue;
                        }
                        let k = match vocab.channel(toks, i, j) {
                            Some(k) => k,
                            None => continue,
                        };
                        // Kuramoto on one channel: each word moves a little
                        // toward the other, and both gain evidence for this
                        // relation.
                        let (ta, tb) = (
                            facet.words[&toks[i]].theta[k],
                            facet.words[&toks[j]].theta[k],
                        );
                        let d = (tb - ta).sin();
                        if let Some(p) = facet.words.get_mut(&toks[i]) {
                            p.theta[k] = (ta + RATE * d).rem_euclid(TWO_PI);
                            p.mag[k] += 1.0;
                        }
                        if let Some(p) = facet.words.get_mut(&toks[j]) {
                            p.theta[k] = (tb - RATE * d).rem_euclid(TWO_PI);
                            p.mag[k] += 1.0;
                        }
                    }
                }
            }
        }

        // Evidence is a count; compress it so one very frequent relation does not
        // dominate every comparison a word takes part in.
        for p in facet.words.values_mut() {
            for m in p.mag.iter_mut() {
                *m = (1.0 + *m).ln();
            }
        }

        facet.balance();
        facet.silence_collapsed();
        facet
    }

    /// Equalises how loudly each channel votes.
    ///
    /// Traffic is wildly unequal: every adjacent pair feeds an offset channel,
    /// while a marker channel only sees pairs joined by that one word. Left raw,
    /// the offset channels carry two orders of magnitude more evidence and decide
    /// every comparison — the first run of `bin/relational` had all ten relation
    /// families peaking on offset channels and not one on a marker.
    ///
    /// Dividing each channel by its own mean evidence makes a channel's vote
    /// proportional to how much *this word* uses it relative to how much anyone
    /// does, rather than to how common the relation is.
    pub(super) fn balance(&mut self) {
        let n = self.words.len().max(1) as f64;
        let mut mean = vec![0.0f64; PHASE_CHANNELS];
        for p in self.words.values() {
            for (m, v) in mean.iter_mut().zip(&p.mag) {
                *m += v;
            }
        }
        for m in mean.iter_mut() {
            *m = (*m / n).max(1e-9);
        }
        for p in self.words.values_mut() {
            for (v, m) in p.mag.iter_mut().zip(&mean) {
                *v /= m;
            }
        }
    }

    /// Mutes channels whose phases have collapsed onto one angle.
    ///
    /// Kuramoto coupling is attraction-dominated, and restricting it to a channel
    /// does not change that — it concentrates the failure. A channel every pair
    /// feeds pulls its whole vocabulary to a point, after which every word agrees
    /// with every other there and the channel is pure noise with a loud voice.
    /// That is what put the first run *below chance*: the collapsed offset
    /// channels agreed about everything, so the only discrimination left came
    /// from sparse marker channels holding near-random angles.
    ///
    /// The project already has this guard for the manifold as a whole
    /// (`core::grounding::DISPERSION_FLOOR`); this applies it per channel,
    /// which is where the new representation can fail.
    pub(super) fn silence_collapsed(&mut self) {
        // Dispersion over a handful of points is not a measurement, and muting a
        // channel on that basis destroys information rather than protecting it.
        // The same back-off `Wave::pool_floor` applies to retrieval.
        if self.words.len() < GUARD_MIN_VOCAB {
            self.live = PHASE_CHANNELS;
            return;
        }
        let n = self.words.len().max(1) as f64;
        let mut keep = [true; PHASE_CHANNELS];
        for (k, kept) in keep.iter_mut().enumerate() {
            let (mut x, mut y) = (0.0f64, 0.0f64);
            for p in self.words.values() {
                if p.mag[k] > 0.0 {
                    x += p.theta[k].cos();
                    y += p.theta[k].sin();
                }
            }
            let dispersion = 1.0 - x.hypot(y) / n;
            *kept = dispersion >= CHANNEL_DISPERSION_FLOOR;
        }
        self.live = keep.iter().filter(|k| **k).count();
        for p in self.words.values_mut() {
            for (v, &kept) in p.mag.iter_mut().zip(&keep) {
                if !kept {
                    *v = 0.0;
                }
            }
        }
    }

    pub fn similarity(&self, a: &str, b: &str) -> f64 {
        match (self.words.get(a), self.words.get(b)) {
            (Some(x), Some(y)) => x.similarity(y),
            _ => 0.0,
        }
    }

    pub fn on(&self, a: &str, b: &str, k: usize) -> f64 {
        match (self.words.get(a), self.words.get(b)) {
            (Some(x), Some(y)) => x.on(y, k),
            _ => 0.0,
        }
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}
