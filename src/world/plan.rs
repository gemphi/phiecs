#[allow(unused_imports)]
use super::*;

use crate::relation::Relation;

impl World {
    /// A plan: the heaviest chain of `kind` edges from `from` to `to`, at
    /// most `depth` steps long — the sequence of words the world was shown
    /// leading from one to the other. `None` when no chain was ever shown.
    /// This is what a world model is for (optimal control's question: which
    /// sequence of actions reaches the outcome), answered from the edges
    /// alone: X ⇒ Y under `kind`, followed.
    pub fn plan(&self, from: Word, kind: Relation, to: Word, depth: usize) -> Option<Vec<Word>> {
        self.plan_avoiding(from, kind, to, depth, &[])
    }

    /// [`World::plan`] under a constraint: no step through a word in
    /// `avoid`. A constraint in this space is not a head to be trained — it
    /// is a word, stated in the words the world is written in ("not the
    /// wall"), and the plan is the heaviest chain that never says it.
    pub fn plan_avoiding(&self, from: Word, kind: Relation, to: Word, depth: usize, avoid: &[Word]) -> Option<Vec<Word>> {
        // Best-first by the product of edge weights along the path.
        let mut frontier: Vec<(f64, Vec<Word>)> = vec![(1.0, vec![from])];
        let mut best: Option<(f64, Vec<Word>)> = None;
        while let Some((score, path)) = frontier.pop() {
            let last = *path.last()?;
            if last == to && path.len() > 1 {
                if best.as_ref().is_none_or(|(s, _)| score > *s) {
                    best = Some((score, path));
                }
                continue;
            }
            if path.len() > depth {
                continue;
            }
            for (next, w) in self.calls_for(last, kind) {
                if (path.contains(&next) && next != to) || avoid.contains(&next) {
                    continue;
                }
                let mut p = path.clone();
                p.push(next);
                frontier.push((score * w, p));
            }
            frontier.sort_by(|a, b| a.0.total_cmp(&b.0));
        }
        best.map(|(_, p)| p)
    }

    /// Everything reachable from `from` along `kind` within `depth` steps,
    /// with the weight of the heaviest chain to each, heaviest first.
    pub fn reach(&self, from: Word, kind: Relation, depth: usize) -> Vec<(Word, f64)> {
        let mut seen: Vec<(Word, f64)> = Vec::new();
        let mut layer: Vec<(Word, f64)> = vec![(from, 1.0)];
        for _ in 0..depth {
            let mut next: Vec<(Word, f64)> = Vec::new();
            for (w, s) in &layer {
                for (n, weight) in self.calls_for(*w, kind) {
                    if n == from {
                        continue;
                    }
                    let score = s * weight;
                    match seen.iter_mut().find(|(x, _)| *x == n) {
                        Some(e) => e.1 = e.1.max(score),
                        None => {
                            seen.push((n, score));
                            next.push((n, score));
                        }
                    }
                }
            }
            layer = next;
        }
        seen.sort_by(|a, b| b.1.total_cmp(&a.1));
        seen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plan_is_the_heaviest_chain_shown() {
        let mut w = World::new();
        let before = w.declare("before").unwrap();
        let [a, b, c, d, e] = ["a", "b", "c", "d", "e"].map(|s| w.coin_bare(s));
        // a → b → d twice; a → c → d once; d → e; nothing reaches a from e.
        for _ in 0..2 {
            w.relate(a, before, b);
            w.relate(b, before, d);
        }
        w.relate(a, before, c);
        w.relate(c, before, d);
        w.relate(d, before, e);
        assert_eq!(w.plan(a, before, e, 4), Some(vec![a, b, d, e]));
        // The constraint is a word: told not to pass through b, the plan takes the lighter road.
        assert_eq!(w.plan_avoiding(a, before, e, 4, &[b]), Some(vec![a, c, d, e]));
        assert_eq!(w.plan_avoiding(a, before, e, 4, &[d]), None, "every road to e runs through d");
        assert_eq!(w.plan(a, before, e, 2), None, "too short");
        assert_eq!(w.plan(e, before, a, 4), None, "never shown");
        let r = w.reach(a, before, 3);
        assert_eq!(r[0].0, d, "{r:?}");
        assert_eq!(r.len(), 4);
    }
}
