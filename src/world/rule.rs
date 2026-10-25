#[allow(unused_imports)]
use super::*;

use crate::relation::Relation;

/// The layer a learned representation space is said to lack: the heads
/// that map it to a constraint ("not the wall"), an objective ("the door
/// open") and a yes-or-no ("is it open?"). Here none is trained. A
/// constraint and a goal are rows told under a condition — `scene avoid
/// wall`, `scene reach open` — read back by name; a validator is whether
/// the world says a row; a projection is how much it says it. The
/// engineer and the model put them in the same words.
pub const AVOID: &str = "avoid";
pub const REACH: &str = "reach";

/// What a condition asks of a plan: the words to reach and the words never
/// to pass through, as told under it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rule {
    pub reach: Vec<Word>,
    pub avoid: Vec<Word>,
}

impl World {
    /// Tell a constraint: under `scope`, never pass through `word`.
    pub fn forbid(&mut self, scope: Word, word: Word) {
        if let Some(k) = self.declare(AVOID) {
            self.relate(scope, k, word);
        }
    }

    /// Tell an objective: under `scope`, `word` is what to reach.
    pub fn want(&mut self, scope: Word, word: Word) {
        if let Some(k) = self.declare(REACH) {
            self.relate(scope, k, word);
        }
    }

    /// The rule a condition carries, read back from its rows: heaviest first.
    pub fn rule(&self, scope: Word) -> Rule {
        let read = |name: &str| self.relations.get(name).map(|k| self.calls_for(scope, k).into_iter().map(|(w, _)| w).collect()).unwrap_or_default();
        Rule { reach: read(REACH), avoid: read(AVOID) }
    }

    /// The validator: does the world say `about kind other`? A yes-or-no
    /// that is read, not trained.
    pub fn holds(&self, about: Word, kind: Relation, other: Word) -> bool {
        self.calls_for(about, kind).iter().any(|(w, _)| *w == other)
    }

    /// The projection: how much the world says `about kind other` — the
    /// edge's weight, 0 when it was never said. The cost function a
    /// controller asks for is this, summed along a plan.
    pub fn degree(&self, about: Word, kind: Relation, other: Word) -> f64 {
        self.calls_for(about, kind).into_iter().find(|(w, _)| *w == other).map_or(0.0, |(_, w)| w)
    }

    /// Whether `path` keeps `rule`: ends on a word to reach (or there is
    /// none asked) and passes through no word to avoid.
    pub fn keeps(&self, path: &[Word], rule: &Rule) -> bool {
        let reached = rule.reach.is_empty() || path.last().is_some_and(|w| rule.reach.contains(w));
        reached && !path.iter().any(|w| rule.avoid.contains(w))
    }

    /// A plan under a condition: from `from`, along `kind`, to the first
    /// of the condition's goals a chain reaches, through none of what it
    /// forbids. The constraint layer, read from the rows.
    pub fn plan_in(&self, scope: Word, from: Word, kind: Relation, depth: usize) -> Option<Vec<Word>> {
        let rule = self.rule(scope);
        rule.reach.iter().find_map(|goal| self.plan_avoiding(from, kind, *goal, depth, &rule.avoid)).filter(|p| self.keeps(p, &rule))
    }

    /// The weight of a plan: the product of its edges' degrees under `kind`.
    pub fn plan_weight(&self, path: &[Word], kind: Relation) -> f64 {
        path.windows(2).map(|p| self.degree(p[0], kind, p[1])).product()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_constraint_is_a_row_and_a_validator_is_a_read() {
        let mut w = World::new();
        let before = w.declare("before").unwrap();
        let [hall, door, wall, open, window] = ["hall", "door", "wall", "open", "window"].map(|s| w.coin_bare(s));
        let scene = w.coin_bare("scope:room");
        // Two ways to open: through the door (shown twice) or over the wall (once).
        for _ in 0..2 {
            w.relate(hall, before, door);
            w.relate(door, before, open);
        }
        w.relate(hall, before, wall);
        w.relate(wall, before, open);
        w.relate(hall, before, window);
        // The engineer speaks in the world's words: reach `open`, never the wall.
        w.want(scene, open);
        w.forbid(scene, wall);
        assert_eq!(w.rule(scene), Rule { reach: vec![open], avoid: vec![wall] });
        assert!(w.holds(door, before, open) && !w.holds(window, before, open));
        assert_eq!(w.degree(hall, before, door), 2.0);
        assert_eq!(w.degree(window, before, open), 0.0);
        let plan = w.plan_in(scene, hall, before, 4).unwrap();
        assert_eq!(plan, vec![hall, door, open]);
        assert_eq!(w.plan_weight(&plan, before), 4.0);
        assert!(w.keeps(&plan, &w.rule(scene)) && !w.keeps(&[hall, wall, open], &w.rule(scene)));
        // Forbid the door too, and the only road left is the forbidden wall: no plan, honestly.
        w.forbid(scene, door);
        assert_eq!(w.plan_in(scene, hall, before, 4), None);
    }
}
