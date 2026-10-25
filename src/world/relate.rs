#[allow(unused_imports)]
use super::*;

use crate::phasor::Phasor;
use crate::relation::Relation;

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

    /// What `word` calls for under `kind`, READ FROM THE PHASORS: the words
    /// standing nearest one turn past it on that channel, each scored by its
    /// agreement there times its amplitude — the difference under the
    /// condition, not a count of a pair (docs/187). Silence is excluded: a
    /// word with no evidence on the channel is not a candidate.
    pub fn reads(&self, word: Word, kind: Relation) -> Vec<(Word, f64)> {
        let Some(me) = self.read::<Phasor>(word) else { return Vec::new() };
        if me.spoken(kind) == 0 {
            return Vec::new();
        }
        let turn = if self.relations.is_symmetric(kind) { 0.0 } else { TURN };
        // The slots of this kind the word has spoken on, each with the
        // target one turn past the word there (docs/190).
        let slot: Vec<Option<(Relation, f64)>> = (0..SLOTS)
            .map(|s| {
                if turn == 0.0 {
                    return None;
                }
                let r = self.relations.get(&format!("{}#{s}", self.relations.name(kind)?))?;
                (me.spoken(r) > 0).then(|| (r, me.theta(r) + turn))
            })
            .collect();
        let plain = me.theta(kind) + turn;
        let mut out: Vec<(Word, f64)> = self
            .words()
            .iter()
            .filter(|w| **w != word)
            .filter_map(|w| {
                let p = self.read::<Phasor>(*w)?;
                // Read on the answer's own slot where the word spoke there;
                // the plain channel answers only when no slot does.
                // The DIFFERENCE rules (docs/192): agreement first, and the
                // amplitude only tempers it — ln(1+n) made "the" win every
                // crowded read by frequency alone; the square root of it
                // leaves agreement the larger term.
                if let Some((r, target)) = slot[slot_of(*w) as usize] {
                    if p.evidence(r) > 0 {
                        return Some((*w, (p.theta(r) - target).cos() * p.amplitude(r).sqrt() + 1.0));
                    }
                }
                if slot.iter().all(Option::is_none) && p.evidence(kind) > 0 {
                    return Some((*w, (p.theta(kind) - plain).cos() * p.amplitude(kind).sqrt()));
                }
                None
            })
            .collect();
        out.sort_by(|x, y| y.1.total_cmp(&x.1));
        out
    }

    /// Take the relation back.
    pub fn unrelate(&mut self, from: Word, kind: Relation, to: Word) -> bool {
        self.edges.unrelate(from, kind, to)
    }

    /// What `word` calls for under `kind`, heaviest first — X ⇒ Y.
    pub fn calls_for(&self, word: Word, kind: Relation) -> Vec<(Word, f64)> {
        self.edges.calls_for(word, kind)
    }

    /// Declare a relation kind by name, interning it as a word too, so a kind
    /// can itself be related to (a condition is a word, see `condition`).
    pub fn declare(&mut self, name: &str) -> Option<Relation> {
        self.lexicon.intern(name);
        self.relations.declare(name)
    }

    /// `from ⇒ to under condition`: the relation told in a setting. The
    /// condition is coined as a word like any other.
    pub fn relate_under(&mut self, condition: &str, from: Word, to: Word, weight: f64) {
        let c = self.coin_bare(condition);
        self.under.relate(c, from, to, weight);
    }

    /// What `word` calls for read under `condition`: the condition's own
    /// edges when it has any for the word, else the word's plain edges
    /// under `kind` — the level above answers when the setting is silent.
    pub fn calls_for_under(&self, word: Word, kind: Relation, condition: Word) -> Vec<(Word, f64)> {
        let own = self.under.calls_for(condition, word);
        if own.is_empty() {
            self.calls_for(word, kind)
        } else {
            own
        }
    }
}
