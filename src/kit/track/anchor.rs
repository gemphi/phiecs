//! An anchor: a position and orientation in the world that tracking holds
//! on to over time, and the state tracking is in. Every tracked thing —
//! a plane, an image, a face, a body, a mesh, a place on the earth, another
//! participant — is an anchor with more said about it.

use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;
use std::sync::atomic::{AtomicU64, Ordering};

/// A 4×4 transform, column-major, from the anchor's space to the world's.
pub type Transform = [[f32; 4]; 4];
/// A point or direction in space.
pub type Point = [f32; 3];
/// A point on an image or a screen.
pub type Point2 = [f32; 2];

pub const IDENTITY: Transform = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

/// The translation of a transform: where it puts the origin.
pub fn position(t: &Transform) -> Point {
    [t[3][0], t[3][1], t[3][2]]
}

static NEXT: AtomicU64 = AtomicU64::new(1);

/// A fresh identifier; anchors, maps and rooms are told apart by one.
pub fn next_id() -> u64 {
    NEXT.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Anchor {
    pub id: u64,
    pub name: Option<String>,
    pub transform: Transform,
    /// The session that owns the anchor, when one does.
    pub session: Option<u64>,
    /// Whether tracking currently sees it.
    pub tracked: bool,
}

impl Anchor {
    pub fn new(transform: Transform) -> Self {
        Self { id: next_id(), name: None, transform, session: None, tracked: false }
    }

    pub fn named(name: impl Into<String>, transform: Transform) -> Self {
        Self { name: Some(name.into()), ..Self::new(transform) }
    }

    pub fn position(&self) -> Point {
        position(&self.transform)
    }
}

impl Default for Anchor {
    fn default() -> Self {
        Self::new(IDENTITY)
    }
}

named! {
    /// How well the world is being tracked.
    pub enum Tracking { NotAvailable, Limited, Normal }
}

named! {
    /// Why tracking is limited, when it is.
    pub enum Reason { None, Initializing, ExcessiveMotion, InsufficientFeatures, Relocalizing, BadHardwareState }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = fields(path, "Anchor", &["id", "name", "transform", "session", "tracked"]);
    r.extend(cases::<Tracking>(path));
    r.extend(cases::<Reason>(path));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_are_told_apart_and_placed() {
        let a = Anchor::named("table", IDENTITY);
        let b = Anchor::new([[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [1.0, 2.0, 3.0, 1.0]]);
        assert_ne!(a.id, b.id);
        assert_eq!(b.position(), [1.0, 2.0, 3.0]);
        assert!(told("track/anchor").iter().any(|r| r.about == "tracking:limited" && r.kind == "is-a"));
    }
}
