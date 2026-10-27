//! The edge record beside the phasors: what was told, read back as edges, and relations under a condition.

use super::super::World;
use crate::relation::Relation;
use crate::word::Word;

impl World {
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
