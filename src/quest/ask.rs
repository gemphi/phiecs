#[allow(unused_imports)]
use super::*;

impl Quest {
    /// Ask about one word: fill the template, put it to the oracle, parse what
    /// comes back, write it into the world, and open the words it mentioned.
    ///
    /// A failed answer is a step too — it is recorded, the word counts as
    /// asked, and the quest goes on. An oracle that cannot answer is data
    /// about the oracle, not a reason to stop. In one-kind-at-a-time mode
    /// ([`Quest::one_kind_at_a_time`]) the next question is a (word, kind).
    pub fn ask_one(&mut self, oracle: &mut dyn Oracle) -> Option<Step> {
        if self.per_kind.is_some() {
            return self.ask_one_kind(oracle);
        }
        let (about, depth) = self.next_word()?;
        // The prompt offers the kinds the quest began with, never ones a model
        // invented on the way (`has-live`, from `has live young`): offered
        // back, an invented kind is used again and becomes a habit.
        let offered = self.offered.clone();
        let step = self.ask_about(oracle, &about, depth, &template::RELATIONS, &offered);
        for w in &step.opened {
            self.push(w, depth + 1);
        }
        self.steps.push(step.clone());
        if self.conditioned {
            let under = self.ask_conditions(oracle, &about, depth);
            for w in &under.opened {
                self.push(w, depth + 1);
            }
            self.steps.push(under);
        }
        Some(step)
    }

    /// One question about `about` offering `kinds`; claims under any other
    /// kind are dropped. Writes the kept relations into the world and returns
    /// the step, with the words it opened.
    ///
    /// An Oracle is consulted only on Overflow (R3.9): the rows already held
    /// in the World that answer this ask are counted first, and when that
    /// count is above zero no Oracle is asked at all and no row is told — the
    /// step records what was held and leaves the world untouched. For a
    /// consulted ask the held-row count is zero by construction, which the
    /// step records.
    pub(crate) fn ask_about(&mut self, oracle: &mut dyn Oracle, about: &str, depth: u32, t: &Template, kinds: &[String]) -> Step {
        let names: Vec<&str> = kinds.iter().map(String::as_str).collect();
        let prompt = t.fill(about, &names);
        // Overflow: the rows held inside that answer this ask. Above zero, the
        // Oracle is not consulted and the step is empty.
        if self.held_rows(about, kinds) > 0 {
            let answer = Answer::Nothing("held inside; no overflow".to_string());
            return Step { about: about.to_string(), depth, template: t.name, prompt, answer, kept: Vec::new(), refused: Vec::new(), opened: Vec::new() };
        }
        let (answer, claims) = match oracle.ask(&prompt) {
            Ok(text) => {
                let c = parse_relations(&text, kinds);
                (Answer::Said(text), c)
            }
            Err(e) => (Answer::Nothing(e.to_string()), Vec::new()),
        };
        let head = self.world.coin_bare(about);
        let mut kept = Vec::new();
        let mut refused = Vec::new();
        for c in claims.into_iter().filter(|c| !parse::is_self_edge(c, about) && kinds.contains(&c.kind)) {
            // Past the channel bound a claim is dropped rather than silently
            // rebucketed.
            let Some(kind) = self.world.declare(&c.kind) else { continue };
            let other = self.world.coin_bare(&c.other);
            // The Checker classifies the claim immediately before the row
            // would be told; a standing that does not tell refuses the row
            // (R8.10) and nothing is written for it.
            let standing = self.checker.check(about, &c, &self.world);
            if !standing.tells() {
                refused.push((c.kind, c.other, c.weight, standing));
                continue;
            }
            self.world.relate_by(head, kind, other, c.weight, crate::world::COUPLE_RATE * c.weight);
            kept.push((c.kind, c.other, c.weight));
        }
        let opened = step::opened(&kept, self.asked());
        Step { about: about.to_string(), depth, template: t.name, prompt, answer, kept, refused, opened }
    }

    /// How many rows the World already holds that answer this ask: edges out
    /// of `about` under any offered kind. A word the World has not coined has
    /// none, so this is zero before the first ask about it.
    fn held_rows(&self, about: &str, kinds: &[String]) -> usize {
        let Some(head) = self.world.lexicon.get(about) else { return 0 };
        self.world
            .edges
            .out_of(head)
            .iter()
            .filter(|e| kinds.iter().any(|k| self.world.relations.get(k) == Some(e.kind)))
            .count()
    }
}

#[cfg(test)]
mod tests;
