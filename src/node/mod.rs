//! `node` — what an entity is made of in a world. Levels 1 and 2 of phiecs.
//!
//! | part | level | what | a robot, in MuJoCo's terms |
//! |---|---|---|---|
//! | [`component`] | 1 · data | what an entity is, hung on it by type | geom, inertia, joint type |
//! | [`state`] | 1 · data | what changes as the world steps, kept twice | `qpos`, `qvel` |
//! | [`tree`] | 2 · structure | one parent each, no loops | the kinematic body tree |
//! | [`graph`] | 2 · structure | typed, weighted edges, many to many | tendons, contacts, constraints |
//!
//! A language uses the same parts: its phasors and conditions are
//! components, its relations are the graph.

pub mod component;
pub mod graph;
pub mod state;
pub mod tree;
#[cfg(test)]
mod tests;

pub use component::{Component, Dense};
pub use graph::{Edge, Graph, Kind, Kinds};
pub use state::State;
pub use tree::Tree;
