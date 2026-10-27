//! `Phasor` — what `Transform` is to RealityKit.
//!
//! A RealityKit entity's state in the world is a position, a rotation and a
//! scale in ℝ³ (`reality-foundation/components.rs:12`). A word's state is its
//! phase on the relation circle: [`MAX_KINDS`](crate::relation::MAX_KINDS)
//! channels, each a complex phasor with an amplitude and the count of what was
//! seen under it. Phical stores no point — the number here is never a
//! position, always an angle and how much evidence stands behind it.
//!
//! [`spectral`] holds the same word packed for a lexicon: 64 channels a
//! byte each, with evidence counts and a target side (phiano's store).

pub mod facet;
pub mod gemgum;
pub mod spectral;
pub mod wave;

pub use spectral::SpectralPhasor;

use crate::aspect::Aspect;
use crate::relation::{MAX_KINDS, Relation};

/// One channel of a word: its angle, how much stands behind it, and how
/// often the word has spoken (stood as source) there.
#[derive(Clone, Copy, Debug, Default)]
pub struct Slot {
    pub theta: f64,
    pub amplitude: f64,
    pub evidence: u32,
    pub spoken: u32,
}

/// One word's angles, per relation channel: the base [`MAX_KINDS`] held
/// densely, every channel past them held only where the word has touched
/// it (docs/188) — a condition is a channel, and a word pays for what it
/// uses.
#[derive(Clone, Debug)]
pub struct Phasor {
    base: Vec<Slot>,
    extra: std::collections::HashMap<usize, Slot>,
    /// The word's seed, so an extension channel opens at a reproducible
    /// angle of its own rather than at zero.
    seed: u64,
}

impl Aspect for Phasor {}

const TWO_PI: f64 = std::f64::consts::TAU;

impl Default for Phasor {
    fn default() -> Self {
        Self::new()
    }
}

impl Phasor {
    /// All channels silent: no angle claimed, no evidence behind any.
    pub fn new() -> Self {
        Self { base: vec![Slot::default(); MAX_KINDS], extra: Default::default(), seed: 0 }
    }

    /// Seeded angles on the base channels, so a word is somewhere before it
    /// is told anything; extension channels open seeded too, lazily.
    pub fn seeded(spelling: &str) -> Self {
        let mut p = Self::new();
        for (k, t) in phistd::seed_angles(spelling, MAX_KINDS).into_iter().enumerate() {
            p.base[k].theta = t;
        }
        p.seed = phistd::mix64(spelling.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3)));
        p
    }

    fn slot(&self, k: usize) -> Option<&Slot> {
        if k < MAX_KINDS { self.base.get(k) } else { self.extra.get(&k) }
    }

    fn slot_mut(&mut self, k: usize) -> &mut Slot {
        if k < MAX_KINDS {
            &mut self.base[k]
        } else {
            let seed = self.seed;
            self.extra.entry(k).or_insert_with(|| Slot {
                theta: (phistd::mix64(seed ^ k as u64) >> 11) as f64 / (1u64 << 53) as f64 * TWO_PI,
                ..Slot::default()
            })
        }
    }

    /// The base channels' angles, in channel order: the word as a function
    /// on the relation circle (phiano's field reads it as `e^{iθ}`).
    pub fn angles(&self) -> Vec<f64> {
        self.base.iter().map(|s| s.theta).collect()
    }

    pub fn theta(&self, kind: Relation) -> f64 {
        self.slot(kind.channel()).map(|s| s.theta).unwrap_or(0.0)
    }

    pub fn amplitude(&self, kind: Relation) -> f64 {
        self.slot(kind.channel()).map(|s| s.amplitude).unwrap_or(0.0)
    }

    pub fn evidence(&self, kind: Relation) -> u32 {
        self.slot(kind.channel()).map(|s| s.evidence).unwrap_or(0)
    }

    /// Times the word stood as source under `kind`.
    pub fn spoken(&self, kind: Relation) -> u32 {
        self.slot(kind.channel()).map(|s| s.spoken).unwrap_or(0)
    }

    /// Mark one more time as source under `kind`.
    pub fn speak(&mut self, kind: Relation) {
        let s = self.slot_mut(kind.channel());
        s.spoken = s.spoken.saturating_add(1);
    }

    /// Move this channel toward `target`, the short way round, and count
    /// the observation. The step is the larger of `rate` and `1/(n+1)`: the
    /// first time a word is seen under a kind it goes all the way (one
    /// hearing is believed), after that its angle is the running mean of
    /// what it was seen with, and the floor `rate` keeps it moving for ever
    /// — on the fly, never frozen (docs/187). This is the whole learning
    /// rule: pairs of words moving under a condition.
    pub fn couple(&mut self, kind: Relation, target: f64, rate: f64) {
        let s = self.slot_mut(kind.channel());
        let step = rate.max(1.0 / (s.evidence as f64 + 1.0));
        let d = (target - s.theta + std::f64::consts::PI).rem_euclid(TWO_PI) - std::f64::consts::PI;
        s.theta = (s.theta + step * d).rem_euclid(TWO_PI);
        s.evidence = s.evidence.saturating_add(1);
        s.amplitude = (s.evidence as f64).ln_1p();
    }

    /// How closely two words agree on `kind`: 1 together, −1 opposed. `None`
    /// when either has no evidence there — silence is not agreement.
    pub fn agreement(&self, other: &Phasor, kind: Relation) -> Option<f64> {
        if self.evidence(kind) == 0 || other.evidence(kind) == 0 {
            return None;
        }
        Some((self.theta(kind) - other.theta(kind)).cos())
    }

    /// Agreement across every channel both words have evidence on, weighted
    /// by how much evidence that is.
    pub fn resonance(&self, other: &Phasor) -> f64 {
        let (mut sum, mut weight) = (0.0, 0.0);
        for (kind, _) in self.kinds() {
            if let Some(a) = self.agreement(other, kind) {
                let w = self.amplitude(kind).min(other.amplitude(kind));
                sum += a * w;
                weight += w;
            }
        }
        if weight == 0.0 { 0.0 } else { sum / weight }
    }

    /// Channels this word has evidence on, heaviest first — its relation set,
    /// base and extension alike.
    pub fn kinds(&self) -> Vec<(Relation, u32)> {
        let mut v: Vec<(Relation, u32)> = self
            .base
            .iter()
            .enumerate()
            .filter(|(_, s)| s.evidence > 0)
            .map(|(k, s)| (Relation(k as u32), s.evidence))
            .chain(self.extra.iter().filter(|(_, s)| s.evidence > 0).map(|(k, s)| (Relation(*k as u32), s.evidence)))
            .collect();
        v.sort_by_key(|a| std::cmp::Reverse(a.1));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coupling_moves_together_and_counts() {
        let kind = Relation(3);
        let mut a = Phasor::seeded("cat");
        let mut b = Phasor::seeded("mat");
        assert_eq!(a.agreement(&b, kind), None, "silence is not agreement");
        for _ in 0..40 {
            let (ta, tb) = (a.theta(kind), b.theta(kind));
            a.couple(kind, tb, 0.2);
            b.couple(kind, ta, 0.2);
        }
        assert!(a.agreement(&b, kind).unwrap() > 0.99);
        assert_eq!(a.evidence(kind), 40);
        assert!(a.resonance(&b) > 0.99);
        assert_eq!(a.kinds(), vec![(kind, 40)]);
    }
}
