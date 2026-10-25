//! The kit whole: the body, tracking and the device told as one set of
//! rows, coined into a `World` so systems can run over it (the counting
//! and casting over these rows live up in phient's `relate::kit`). Nothing is read from a file: the kit is code, and its rows are what
//! the code says of itself.

use crate::prelude::*;
use crate::quest::Row;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Kit {
    pub rows: Vec<Row>,
}

impl Kit {
    /// Everything the kit says of itself: `body/…`, `track/…`, `device/…`.
    pub fn new() -> Self {
        let mut rows = crate::kit::body::told();
        rows.extend(crate::kit::track::told());
        rows.extend(crate::kit::device::told());
        Self { rows }
    }

    /// The kit with more rows told under it — a registry's, a scene's frames.
    pub fn with(mut self, rows: Vec<Row>) -> Self {
        self.rows.extend(rows);
        self
    }

    /// The things whose kind-row is `is-a kind`: `joint`, `shape`, `key`, `mode`.
    pub fn of_kind(&self, kind: &str) -> Vec<String> {
        let mut v: Vec<String> = self.rows.iter().filter(|r| r.kind == "is-a" && r.other == kind).map(|r| r.about.clone()).collect();
        v.dedup();
        v
    }

    /// Every kind, with how many things are of it, most first.
    pub fn kinds(&self) -> Vec<(String, usize)> {
        let mut m: HashMap<&str, usize> = HashMap::new();
        for r in self.rows.iter().filter(|r| r.kind == "is-a") {
            *m.entry(r.other.as_str()).or_default() += 1;
        }
        let mut v: Vec<(String, usize)> = m.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        v
    }

    /// A thing's words as its scope says them (`left elbow`).
    pub fn words(&self, name: &str) -> Option<String> {
        self.rows.iter().filter(|r| r.about == name && r.kind == "within").map(|r| r.other.clone()).max_by_key(|s| s.len()).and_then(|s| s.rsplit('/').next().map(str::to_string))
    }

    /// The things under a scope, in the order told.
    pub fn under(&self, scope: &str) -> Vec<String> {
        let mut v: Vec<String> = self.rows.iter().filter(|r| r.kind == "within" && r.other == scope && !r.about.starts_with("scope:")).map(|r| r.about.clone()).collect();
        v.dedup();
        v
    }

    /// The rows as a phiecs world: a word for every thing, scope and word,
    /// a relation per kind (phiecs allows sixty-four), every row an edge
    /// that couples the two words' phasors under that kind.
    pub fn world(&self) -> World {
        // The one Row → World loop lives in phiecs (docs/159).
        World::from_rows(&self.rows)
    }
}

