//! Register told under a condition: `X ⇒ Y under c`, the paradigm's whole
//! sentence. A word's plain edges say what it relates to; these say what it
//! relates to *in a setting* — `bank` under `money` calls for `deposit`, under
//! `river` for `shore`. The condition is an ordinary word, so it can itself be
//! related to and be a condition for further conditions.
//!
//! A reading under a condition takes the condition's own edges when it has
//! any for the word, and otherwise the word's plain edges: the level above
//! answers when the scope has nothing of its own to say (D4's nesting in its
//! simplest form).

use crate::word::Word;
use std::collections::HashMap;

/// `(condition, word)` ⇒ what the word relates to under it, with weights.
#[derive(Default, Clone, Debug)]
pub struct Under {
    by: HashMap<(Word, Word), Vec<(Word, f64)>>,
}

impl Under {
    /// Add `from ⇒ to under condition`, or add `weight` to it if it is there.
    pub fn relate(&mut self, condition: Word, from: Word, to: Word, weight: f64) {
        let list = self.by.entry((condition, from)).or_default();
        match list.iter_mut().find(|(w, _)| *w == to) {
            Some((_, x)) => *x += weight,
            None => list.push((to, weight)),
        }
    }

    /// What `from` relates to under `condition`, heaviest first.
    pub fn calls_for(&self, condition: Word, from: Word) -> Vec<(Word, f64)> {
        let mut v = self.by.get(&(condition, from)).cloned().unwrap_or_default();
        v.sort_by(|a, b| b.1.total_cmp(&a.1));
        v
    }

    /// The conditions `from` was told under.
    pub fn conditions_of(&self, from: Word) -> Vec<Word> {
        let mut v: Vec<Word> = self.by.keys().filter(|(_, f)| *f == from).map(|(c, _)| *c).collect();
        v.sort_by_key(|w| w.id());
        v
    }

    pub fn len(&self) -> usize {
        self.by.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.by.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_word_calls_for_different_things_under_different_conditions() {
        let w = |i| Word::from_id(i);
        let (bank, money, river, deposit, shore) = (w(1), w(2), w(3), w(4), w(5));
        let mut u = Under::default();
        u.relate(money, bank, deposit, 0.9);
        u.relate(river, bank, shore, 0.8);
        u.relate(river, bank, shore, 0.1);
        assert_eq!(u.calls_for(money, bank), vec![(deposit, 0.9)]);
        assert_eq!(u.calls_for(river, bank), vec![(shore, 0.9)]);
        assert_eq!(u.conditions_of(bank), vec![money, river]);
        assert_eq!(u.len(), 2);
    }
}
