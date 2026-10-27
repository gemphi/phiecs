#[allow(unused_imports)]
use super::*;

use crate::phasor::Phasor;
use crate::relation::Relation;
use mode::{read_mode, Mode};

mod edges;
mod mode;
mod read;

/// How far a word moves toward what it was seen with, per observation.
pub const COUPLE_RATE: f64 = 0.08;

/// The turn a relation is: `from ⇒ to` under a kind puts `to` one TURN
/// past `from` on that kind's channel, so a relation is a rotation — an
/// operator applied, not a pair remembered (docs/187). An eighth of the
/// circle: eight steps of order resolve before the turn comes round.
pub const TURN: f64 = std::f64::consts::TAU / 8.0;

/// ANSWER SLOTS (docs/190, docs/90 §2): one angle per word per channel is
/// the mean of every answer the word ever had there. So a kind owns this
/// many extension channels, and each answer lives on the slot its own id
/// picks; a word is read on a slot only where it has spoken there. Two
/// answers to one word no longer average unless they share a slot.
pub const SLOTS: u64 = 16;

/// The slot an answer lives on: a stable scatter of its word id.
fn slot_of(word: Word) -> u64 {
    phistd::mix64(word.0 as u64 ^ 0x9E37_79B9_7F4A_7C15) % SLOTS
}
impl World {
    /// `from` stands to `to` under `kind`: record the edge, count the kind,
    /// and move the two words' phasors toward each other on that channel.
    /// This is `add_child` in Apple's model (`world.rs:207`) and the whole of
    /// learning in this one.
    pub fn relate(&mut self, from: Word, kind: Relation, to: Word) {
        self.relate_by(from, kind, to, 1.0, COUPLE_RATE);
    }

    /// [`World::relate`] with the edge weight and coupling rate given.
    pub fn relate_by(&mut self, from: Word, kind: Relation, to: Word, weight: f64, rate: f64) {
        self.edges.relate(from, kind, to, weight);
        self.relations.observe(kind);
        self.register::<Phasor>();
        for w in [from, to] {
            if self.read::<Phasor>(w).is_none() {
                let seed = Phasor::seeded(self.spelling(w));
                self.bind(w, seed);
            }
        }
        let turn = if self.relations.is_symmetric(kind) { 0.0 } else { TURN };
        if matches!(read_mode(), Mode::Typed(_)) && turn != 0.0 {
            // The type layer's channels, beside the old ones (which the
            // code below still couples): two reads, one tell.
            if let Some(range) = self.range(kind) {
                if let Some(b) = self.read::<Phasor>(to).map(|p| p.theta(range)) {
                    if let Some(p) = self.read_mut::<Phasor>(from) {
                        p.couple(kind, b - turn, rate);
                    }
                    if let Some(a) = self.read::<Phasor>(from).map(|p| p.theta(kind)) {
                        if let Some(p) = self.read_mut::<Phasor>(to) {
                            p.couple(range, a + turn, rate);
                        }
                    }
                }
            }
        }
        if matches!(read_mode(), Mode::Domain | Mode::Bound(_) | Mode::Sure) && turn != 0.0 {
            // The two roles, each its own channel: `from` is the thing
            // REACHING (the kind's own channel, its domain), `to` the thing
            // REACHED (the range channel). One angle cannot be both.
            let Some(range) = self.range(kind) else { return };
            let Some(b) = self.read::<Phasor>(to).map(|p| p.theta(range)) else { return };
            if let Some(p) = self.read_mut::<Phasor>(from) {
                p.couple(kind, b - turn, rate);
                p.speak(kind);
            }
            let Some(a) = self.read::<Phasor>(from).map(|p| p.theta(kind)) else { return };
            if let Some(p) = self.read_mut::<Phasor>(to) {
                p.couple(range, a + turn, rate);
            }
            return;
        }
        let Some(b) = self.read::<Phasor>(to).map(|p| p.theta(kind)) else { return };
        // Directed, in sequence: `from` moves toward one turn before `to`,
        // then `to` moves toward one turn past where `from` now stands — so
        // after a single tell the pair agrees exactly. The difference
        // between them under the kind is what is learned; the edge is only
        // its record.
        if let Some(p) = self.read_mut::<Phasor>(from) {
            p.couple(kind, b - turn, rate);
            p.speak(kind);
        }
        let Some(a) = self.read::<Phasor>(from).map(|p| p.theta(kind)) else { return };
        if let Some(p) = self.read_mut::<Phasor>(to) {
            p.couple(kind, a + turn, rate);
        }
        // And on the answer's SLOT of this kind (docs/190), the same move:
        // `from` speaks on the slot, `to` stands one turn past it there.
        if turn != 0.0 {
            let Some(name) = self.relations.name(kind).map(|n| format!("{n}#{}", slot_of(to))) else { return };
            let Some(slot) = self.relations.declare(&name) else { return };
            let Some(b) = self.read::<Phasor>(to).map(|p| p.theta(slot)) else { return };
            if let Some(p) = self.read_mut::<Phasor>(from) {
                p.couple(slot, b - turn, rate);
                p.speak(slot);
            }
            let Some(a) = self.read::<Phasor>(from).map(|p| p.theta(slot)) else { return };
            if let Some(p) = self.read_mut::<Phasor>(to) {
                p.couple(slot, a + turn, rate);
            }
        }
    }
    /// The RANGE channel of a kind: where a word sits as the thing reached.
    /// Declared beside the kind, the way an answer slot is (docs/190).
    fn range(&mut self, kind: Relation) -> Option<Relation> {
        let name = format!("{}>", self.relations.name(kind)?);
        self.relations.declare(&name)
    }
}
