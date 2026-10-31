//! Properties of the ask: the Overflow rule, the check-before-tell order and
//! the refusal of a row (Properties 33, 34, 41). All three run on the
//! workspace's own seeded driver, `phistd::gen` — no outside crate.

use super::*;
use crate::quest::check::{Checker, Standing};
use crate::quest::parse::Claim;
use crate::relation::KIND_NAMES;
use crate::world::World;
use phistd::r#gen::{for_all, Gen, Seeded, CASES};

/// A small set of distinct, cleanly-parsing claims: a base kind, a one-word
/// `other` that is neither the seed nor the kind, and a weight in `(0, 1]`.
/// Distinct `other`s so each is one row, and none a self-edge.
#[derive(Clone, Debug)]
struct Claims(Vec<Claim>);

const SEED: &str = "seed";

impl Gen for Claims {
    fn draw(g: &mut Seeded) -> Self {
        let n = g.below(6) as usize; // 0..=5 claims
        let mut out = Vec::new();
        for i in 0..n {
            let kind = KIND_NAMES[g.below(KIND_NAMES.len() as u64) as usize].to_string();
            // `w{i}` is one word, lower case, never the seed, never a kind.
            let other = format!("w{i}");
            let weight = 0.1 + 0.9 * g.unit(); // in (0.1, 1.0]
            out.push(Claim { kind, other, weight });
        }
        Claims(out)
    }
}

/// Render claims as the pipe lines the parser reads back exactly.
fn answer_of(claims: &[Claim]) -> String {
    claims.iter().map(|c| format!("{} | {} | {}\n", c.kind, c.other, c.weight)).collect()
}

/// An oracle that answers one fixed text and counts how often it was asked.
struct Counting {
    text: String,
    asks: std::rc::Rc<std::cell::Cell<usize>>,
}

impl Oracle for Counting {
    fn ask(&mut self, _prompt: &str) -> Result<String, OracleError> {
        self.asks.set(self.asks.get() + 1);
        Ok(self.text.clone())
    }
}

fn offered() -> Vec<String> {
    KIND_NAMES.iter().map(|k| k.to_string()).collect()
}

/// **Property 33: An Oracle is consulted only on Overflow.**
///
/// With no row held for the ask the Oracle is consulted exactly once; with a
/// row already told for the seed under an offered kind it is not consulted at
/// all, and the world is left as it was.
///
/// **Validates: Requirements 3.9**
#[test]
fn property_33_an_oracle_is_consulted_only_on_overflow() {
    let run = for_all::<Claims>(0x0_33, CASES, |Claims(claims)| {
        let kinds = offered();
        let text = answer_of(claims);

        // Overflow present: nothing held, the Oracle is consulted once.
        let asks = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut o = Counting { text: text.clone(), asks: asks.clone() };
        let mut q = Quest::new(SEED);
        q.ask_about(&mut o, SEED, 0, &template::RELATIONS, &kinds);
        if asks.get() != 1 {
            return false;
        }

        // No overflow: a row is already held for the seed under an offered
        // kind, so no Oracle is consulted and the edge count does not move.
        let asks2 = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut o2 = Counting { text, asks: asks2.clone() };
        let mut q2 = Quest::new(SEED);
        let head = q2.world.coin_bare(SEED);
        let kind = q2.world.declare("is-a").unwrap();
        let other = q2.world.coin_bare("already");
        q2.world.relate_by(head, kind, other, 0.9, 0.08);
        let before = q2.world.edges.len();
        let step = q2.ask_about(&mut o2, SEED, 0, &template::RELATIONS, &kinds);
        asks2.get() == 0 && q2.world.edges.len() == before && step.kept.is_empty()
    });
    assert!(run.is_ok(), "{}", run.unwrap_err());
}

/// A checker that records the World's row count at each call, then tells.
struct Recording {
    seen: std::rc::Rc<std::cell::RefCell<Vec<usize>>>,
}

impl Checker for Recording {
    fn check(&mut self, _about: &str, _claim: &Claim, world: &World) -> Standing {
        self.seen.borrow_mut().push(world.edges.len());
        Standing::Consistent
    }
    fn name(&self) -> String {
        "recording".to_string()
    }
}

/// **Property 34: Every claim is checked before any row is told.**
///
/// A recording checker captures the World's row count at each `check` call.
/// Because the row is told only after `check` returns telling, the count seen
/// at call `i` is exactly `i` — the number of rows already told — so each
/// check strictly precedes its own tell, and the final count equals the kept
/// rows.
///
/// **Validates: Requirements 8.10**
#[test]
fn property_34_every_claim_is_checked_before_any_row_is_told() {
    let run = for_all::<Claims>(0x0_34, CASES, |Claims(claims)| {
        let kinds = offered();
        let seen = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let checker = Recording { seen: seen.clone() };
        let mut o = Counting {
            text: answer_of(claims),
            asks: std::rc::Rc::new(std::cell::Cell::new(0)),
        };
        let mut q = Quest::new(SEED).with_checker(Box::new(checker));
        let step = q.ask_about(&mut o, SEED, 0, &template::RELATIONS, &kinds);
        let seen = seen.borrow();
        // One check per parsed claim; each saw the count before its own row.
        seen.len() == step.kept.len()
            && seen.iter().enumerate().all(|(i, c)| *c == i)
            && q.world.edges.len() == step.kept.len()
    });
    assert!(run.is_ok(), "{}", run.unwrap_err());
}

/// A checker that refuses every claim with a chosen non-telling standing.
struct Refusing(Standing);

impl Checker for Refusing {
    fn check(&mut self, _about: &str, _claim: &Claim, _world: &World) -> Standing {
        self.0.clone()
    }
}

/// **Property 41: A refused claim tells no row.**
///
/// Under a checker that refuses every claim, no row is written: `kept` is
/// empty, every claim lands in `refused` carrying the refusing standing, and
/// the World holds no edge.
///
/// **Validates: Requirements 8.10**
#[test]
fn property_41_a_refused_claim_tells_no_row() {
    let refusals = [
        Standing::Unreadable { reason: "x".into() },
        Standing::Degenerate { test: crate::quest::check::Degeneracy::SelfEdge },
        Standing::Contradicting { rows: Vec::new() },
    ];
    for refusal in refusals {
        let run = for_all::<Claims>(0x0_41, CASES, |Claims(claims)| {
            let kinds = offered();
            let mut o = Counting {
                text: answer_of(claims),
                asks: std::rc::Rc::new(std::cell::Cell::new(0)),
            };
            let mut q = Quest::new(SEED).with_checker(Box::new(Refusing(refusal.clone())));
            let parsed = parse_relations(&answer_of(claims), &kinds)
                .into_iter()
                .filter(|c| !parse::is_self_edge(c, SEED) && kinds.contains(&c.kind))
                .count();
            let step = q.ask_about(&mut o, SEED, 0, &template::RELATIONS, &kinds);
            step.kept.is_empty() && step.refused.len() == parsed && q.world.edges.len() == 0
        });
        assert!(run.is_ok(), "{:?}: {}", refusal, run.unwrap_err());
    }
}
