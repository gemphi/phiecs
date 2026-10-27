//! Reading from the phasors: what a word calls for under a kind.

use super::super::World;
use crate::phasor::Phasor;
use crate::relation::Relation;
use crate::word::Word;
use super::mode::{read_mode, weigh, Mode};
use super::{slot_of, SLOTS, TURN};

impl World {
    /// The range channel if it has already been declared (a read never
    /// declares).
    pub(super) fn range_read(&self, kind: Relation) -> Option<Relation> {
        self.relations.get(&format!("{}>", self.relations.name(kind)?))
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
        let typed: Option<std::collections::HashSet<Word>> = match (read_mode(), self.range_read(kind)) {
            (Mode::Typed(turns), Some(range)) if turn != 0.0 => {
                // The type: members within the band of the condition.
                let target = me.theta(kind) + turn;
                Some(self.words().iter().copied().filter(|w| {
                    *w != word && self.read::<Phasor>(*w).is_some_and(|p| {
                        let d = p.theta(range) - target;
                        p.evidence(range) > 0 && d.sin().atan2(d.cos()).abs() <= turns * TURN
                    })
                }).collect())
            }
            _ => None,
        };
        if let (Mode::Domain | Mode::Bound(_) | Mode::Sure, Some(range)) = (read_mode(), self.range_read(kind)) {
            if turn != 0.0 {
                let target = me.theta(kind) + turn;
                let mut out: Vec<(Word, f64)> = self
                    .words()
                    .iter()
                    .filter(|w| **w != word)
                    .filter_map(|w| {
                        let p = self.read::<Phasor>(*w)?;
                        if p.evidence(range) == 0 {
                            return None;
                        }
                        weigh(p.theta(range), target, p.amplitude(range)).map(|x| (*w, x))
                    })
                    .collect();
                out.sort_by(|x, y| y.1.total_cmp(&x.1));
                return out;
            }
        }
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
            .filter(|w| typed.as_ref().is_none_or(|t| t.contains(*w)))
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
                        return weigh(p.theta(r), target, p.amplitude(r)).map(|x| (*w, x + 1.0));
                    }
                }
                if slot.iter().all(Option::is_none) && p.evidence(kind) > 0 {
                    return weigh(p.theta(kind), plain, p.amplitude(kind)).map(|x| (*w, x));
                }
                None
            })
            .collect();
        out.sort_by(|x, y| y.1.total_cmp(&x.1));
        out
    }
}
