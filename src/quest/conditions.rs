//! The quest's second question about a word, when it asks one (docs/154):
//! under which conditions the word relates to different things. The
//! [`CONDITIONS`] template asks for `condition | other word | weight`; each
//! line becomes `about ⇒ other under condition` in the world
//! ([`World::relate_under`](crate::world::World::relate_under)), and the
//! other words join the frontier like any the oracle mentions.

#[allow(unused_imports)]
use super::*;

impl Quest {
    /// Ask a word's conditions as well as its relations ([`Quest::ask_one`]).
    pub fn with_conditions(mut self) -> Self {
        self.conditioned = true;
        self
    }

    /// One [`CONDITIONS`] question about `about`. A line's first column is
    /// its condition, any word (nothing is offered, so nothing is dropped
    /// for its kind); self-edges and phrases are dropped as for relations.
    pub(crate) fn ask_conditions(&mut self, oracle: &mut dyn Oracle, about: &str, depth: u32) -> Step {
        let prompt = CONDITIONS.fill(about, &[]);
        let (answer, claims) = match oracle.ask(&prompt) {
            Ok(text) => {
                let c = parse_relations(&text, &[]);
                (Answer::Said(text), c)
            }
            Err(e) => (Answer::Nothing(e.to_string()), Vec::new()),
        };
        let head = self.world.coin_bare(about);
        let mut kept = Vec::new();
        for c in claims.into_iter().filter(|c| !parse::is_self_edge(c, about) && c.kind != about) {
            let other = self.world.coin_bare(&c.other);
            self.world.relate_under(&c.kind, head, other, c.weight);
            kept.push((c.kind, c.other, c.weight));
        }
        let opened = step::opened(&kept, self.asked());
        Step { about: about.to_string(), depth, template: CONDITIONS.name, prompt, answer, kept, opened }
    }

    /// Read back a written `conditions.tsv` (`about \t condition \t other \t
    /// weight`) into the world, as [`Quest::from_rows`] reads relations.
    pub fn add_conditions(&mut self, rows: &[Row]) {
        for r in rows {
            let (head, other) = (self.world.coin_bare(&r.about), self.world.coin_bare(&r.other));
            self.world.relate_under(&r.kind, head, other, r.weight);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_word_asked_its_conditions_reads_differently_under_each() {
        let mut oracle = Scripted::new(&[
            ("Name at most 4 conditions", "money | deposit | 0.9\nriver | shore | 0.8\nbank | bank | 0.5"),
            ("Word: bank", "is-a | place | 0.7"),
        ]);
        let mut q = Quest::new("bank").with_depth(0).with_budget(4).with_conditions();
        q.run(&mut oracle);
        assert_eq!(q.steps.iter().map(|s| s.template).collect::<Vec<_>>(), vec!["relations", "conditions"]);
        let w = |s: &str| q.world.lexicon.get(s).unwrap();
        let is_a = q.world.relations.get("is-a").unwrap();
        assert_eq!(q.world.calls_for_under(w("bank"), is_a, w("money"))[0].0, w("deposit"));
        assert_eq!(q.world.calls_for_under(w("bank"), is_a, w("river"))[0].0, w("shore"));
        assert_eq!(q.world.calls_for_under(w("bank"), is_a, w("idle"))[0].0, w("place"), "a silent setting falls back to the plain edge");
        assert_eq!(q.world.under.len(), 2, "the self-edge is dropped");

        let dir = std::env::temp_dir().join("phiecs_conditions_quest");
        q.write(&dir, "scripted").unwrap();
        let rows = parse_tsv(&std::fs::read_to_string(dir.join("conditions.tsv")).unwrap());
        assert_eq!(rows.len(), 2);
        let mut back = Quest::from_rows(&parse_tsv(&std::fs::read_to_string(dir.join("relations.tsv")).unwrap()));
        back.add_conditions(&rows);
        assert_eq!(back.world.under.len(), 2, "conditions.tsv reads back");
        assert!(!std::fs::read_to_string(dir.join("relations.tsv")).unwrap().contains("money"), "conditions stay out of relations.tsv");
    }
}
