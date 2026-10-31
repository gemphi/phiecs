//! How a claim stands before it is believed.
//!
//! A claim out of an oracle is text, not knowledge. Before a row is told,
//! something classifies it: unreadable, degenerate, contradicting,
//! unsupported, consistent — in that order, stopping at the first test that
//! holds, so the classes are mutually exclusive and jointly exhaustive and a
//! claim that qualifies for two takes the earlier one (R8.1).
//!
//! The Law tests live in phiclm's judge and in the detector above it, which
//! both read phiecs. The quest cannot call down to them without the graph of
//! crates closing into a cycle, so phiecs declares the classes as plain data
//! and the one method that produces them, and the detector arrives through
//! [`Checker`] from the caller. phiecs gains no dependency for it: the judge
//! and the Laws are reached through the trait, not through an edge.

use super::parse::{self, Claim};
use crate::world::World;

/// How a claim stands before it is believed. phiecs declares the four
/// classes of R8.1 as plain data and holds no test for the two it cannot
/// perform without phiclm.
#[derive(Clone, Debug, PartialEq)]
pub enum Standing {
    /// Not a row-shaped triple, or a weight outside 0..=1 (R8.12).
    Unreadable { reason: String },
    /// `about == other`, or a field empty after trimming (R8.5).
    Degenerate { test: Degeneracy },
    /// A declared Law of this kind is broken; every participating held
    /// triple is named (R8.2).
    Contradicting { rows: Vec<(String, String, String)> },
    /// Nothing supports it and nothing it breaks (R8.3).
    Unsupported,
    Consistent,
}

/// Which of R8.5's two tests held, so the record says which (R8.10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Degeneracy {
    SelfEdge,
    EmptyField,
}

impl Standing {
    /// Whether a row may be told. Contradicting and Degenerate are a hard
    /// no (R8.10); Unreadable never reaches a class test at all.
    pub fn tells(&self) -> bool {
        matches!(self, Standing::Unsupported | Standing::Consistent)
    }
}

/// Anything that classifies a claim before `World::tell` sees it. One
/// method, a claim in and a classification out.
pub trait Checker {
    fn check(&mut self, about: &str, claim: &Claim, world: &World) -> Standing;

    /// What the transcript records as the source of these classifications,
    /// so every number produced from the transcript is attributed to it.
    fn name(&self) -> String {
        "checker".to_string()
    }
}

/// The default: the two tests phiecs can perform alone — R8.5's self-edge
/// and empty-field test (phiecs already has `is_self_edge`) and R8.12's
/// readability test — returning `Unsupported` for everything else.
///
/// `Bare` is not permissive by omission. It cannot pass a degenerate or
/// unreadable claim, and it names itself `"bare"` so the run record shows
/// that no Law test ran. The Harness judges every Oracle-fed number from a
/// bare-checker run as `Unjudged` rather than as a pass, so the gap is
/// declared instead of defaulted away.
pub struct Bare;

impl Checker for Bare {
    fn check(&mut self, about: &str, claim: &Claim, _world: &World) -> Standing {
        if let Some(reason) = unreadable(about, claim) {
            return Standing::Unreadable { reason };
        }
        if let Some(test) = degeneracy(about, claim) {
            return Standing::Degenerate { test };
        }
        // No Law of the kind is read here, so nothing can be found to
        // support the claim or to be broken by it (R8.3).
        Standing::Unsupported
    }

    fn name(&self) -> String {
        "bare".to_string()
    }
}

/// Why a claim is not a readable row, if it is not one: a weight off the
/// 0..=1 scale, or a field that cannot be read back as one column of
/// `kind | other | weight` because it holds the separator or a line break
/// (R8.12). This runs before any class test, so an unreadable claim is
/// never also called degenerate.
fn unreadable(about: &str, claim: &Claim) -> Option<String> {
    if claim.weight.is_nan() {
        return Some("weight is not a number".to_string());
    }
    if !(0.0..=1.0).contains(&claim.weight) {
        return Some(format!("weight {} outside 0..=1", claim.weight));
    }
    let fields = [("about", about), ("kind", claim.kind.as_str()), ("other", claim.other.as_str())];
    // Each field is read trimmed, so a field that is nothing but whitespace
    // is left to the empty-field test below rather than called unreadable.
    fields
        .iter()
        .map(|(field, text)| (field, text.trim()))
        .find(|(_, text)| !text.is_empty() && (text.contains('|') || text.chars().any(char::is_control)))
        .map(|(field, text)| format!("{field} is not one row-shaped field: {text:?}"))
}

/// Which degeneracy test holds, if one does. The empty-field test is read
/// first: an empty about beside an empty other is both tests at once, and
/// the emptiness is the more useful half to record.
///
/// The self-edge test is [`parse::is_self_edge`], the one the quest already
/// filters its claims by, given the fields trimmed so that a trailing space
/// cannot hide `cat | cat` from it.
fn degeneracy(about: &str, claim: &Claim) -> Option<Degeneracy> {
    let (about, kind, other) = (about.trim(), claim.kind.trim(), claim.other.trim());
    if about.is_empty() || kind.is_empty() || other.is_empty() {
        return Some(Degeneracy::EmptyField);
    }
    let trimmed = Claim { kind: kind.to_string(), other: other.to_string(), weight: claim.weight };
    parse::is_self_edge(&trimmed, about).then_some(Degeneracy::SelfEdge)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claim(kind: &str, other: &str, weight: f64) -> Claim {
        Claim { kind: kind.to_string(), other: other.to_string(), weight }
    }

    fn bare(about: &str, c: &Claim) -> Standing {
        Bare.check(about, c, &World::new())
    }

    #[test]
    fn only_unsupported_and_consistent_tell() {
        assert!(Standing::Unsupported.tells());
        assert!(Standing::Consistent.tells());
        assert!(!Standing::Unreadable { reason: "weight".into() }.tells());
        assert!(!Standing::Degenerate { test: Degeneracy::SelfEdge }.tells());
        assert!(!Standing::Degenerate { test: Degeneracy::EmptyField }.tells());
        assert!(!Standing::Contradicting { rows: Vec::new() }.tells());
    }

    #[test]
    fn a_claim_with_no_law_to_read_is_unsupported() {
        assert_eq!(bare("cat", &claim("is-a", "mammal", 0.9)), Standing::Unsupported);
    }

    #[test]
    fn a_word_is_not_its_own_relation() {
        let self_edges = [claim("is-a", "cat", 0.9), claim("mammal", "mammal", 1.0), claim("is-a", " cat ", 0.9)];
        for c in self_edges {
            assert_eq!(bare("cat", &c), Standing::Degenerate { test: Degeneracy::SelfEdge }, "{c:?}");
        }
    }

    #[test]
    fn an_empty_field_is_degenerate() {
        let empty = Standing::Degenerate { test: Degeneracy::EmptyField };
        assert_eq!(bare("", &claim("is-a", "mammal", 0.9)), empty);
        assert_eq!(bare("   ", &claim("is-a", "mammal", 0.9)), empty);
        assert_eq!(bare("cat", &claim("", "mammal", 0.9)), empty);
        assert_eq!(bare("cat", &claim("is-a", " \t ", 0.9)), empty);
    }

    #[test]
    fn emptiness_is_recorded_over_the_self_edge_it_also_is() {
        let both = claim("is-a", "", 0.9);
        assert_eq!(bare("", &both), Standing::Degenerate { test: Degeneracy::EmptyField });
    }

    #[test]
    fn a_weight_off_the_scale_is_unreadable() {
        for w in [1.5, -0.1, f64::INFINITY, f64::NAN] {
            let standing = bare("cat", &claim("is-a", "mammal", w));
            assert!(matches!(standing, Standing::Unreadable { .. }), "{w} gave {standing:?}");
        }
        assert_eq!(bare("cat", &claim("is-a", "mammal", 0.0)), Standing::Unsupported);
        assert_eq!(bare("cat", &claim("is-a", "mammal", 1.0)), Standing::Unsupported);
    }

    #[test]
    fn a_field_holding_the_separator_is_not_one_row() {
        let standing = bare("cat", &claim("is-a", "mammal | animal", 0.9));
        let Standing::Unreadable { reason } = standing else { panic!("{standing:?}") };
        assert!(reason.starts_with("other"), "{reason}");
        assert!(matches!(bare("cat\nquest", &claim("is-a", "mammal", 0.9)), Standing::Unreadable { .. }));
    }

    #[test]
    fn readability_is_read_before_any_class_test() {
        let standing = bare("cat", &claim("is-a", "cat", 2.0));
        assert!(matches!(standing, Standing::Unreadable { .. }), "{standing:?}");
    }

    #[test]
    fn a_held_row_still_leaves_the_claim_unsupported() {
        let mut world = World::new();
        let head = world.coin_bare("cat");
        let kind = world.declare("is-a").expect("is-a is a base kind");
        let other = world.coin_bare("mammal");
        world.relate_by(head, kind, other, 0.9, crate::world::COUPLE_RATE * 0.9);
        // A bare checker reads no Law, so a second, different other for a
        // functional kind is not contradicting to it — it is unsupported.
        let standing = Bare.check("cat", &claim("is-a", "fish", 0.9), &world);
        assert_eq!(standing, Standing::Unsupported);
    }

    #[test]
    fn the_record_says_which_checker_classified() {
        struct Nameless;
        impl Checker for Nameless {
            fn check(&mut self, _about: &str, _claim: &Claim, _world: &World) -> Standing {
                Standing::Consistent
            }
        }
        assert_eq!(Bare.name(), "bare");
        assert_eq!(Nameless.name(), "checker");
        let mut held: Box<dyn Checker> = Box::new(Bare);
        assert_eq!(held.check("cat", &claim("is-a", "mammal", 0.9), &World::new()), Standing::Unsupported);
        assert_eq!(held.name(), "bare");
    }
}
