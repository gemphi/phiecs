//! # phiecs — an entity-component-system for language
//!
//! Apple's RealityKit puts entities in a world, hangs components on them and
//! runs systems over both. That is the right shape for meaning, with one
//! substitution: the entity is a **word**, and what it carries is not a
//! position in space but a phase on the relation circle.
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c). The Phical paradigm, X ⇒ Y, drives and composes Phient's
//! Phasor Language Models: Phinet, Phiano, PhiTon. Phical stores no point: a
//! word is 32–64 relations, each a complex phasor on a closed circle with a
//! count; meaning is a side effect of pairs of words moving under a condition.
//! This crate is that sentence as a data structure.
//!
//! | RealityKit / reality-foundation | phiecs |
//! |---|---|
//! | `Entity` | [`Word`](word::Word) |
//! | `EntityGenerator` | [`Lexicon`](word::Lexicon) |
//! | `Component` | [`Aspect`](aspect::Aspect) |
//! | `Storage<T>` (linear scan) | [`Dense<A>`](store::Dense) (id-indexed) |
//! | `World` | [`World`](world::World) |
//! | `EntityBuilder` | [`WordBuilder`](world::WordBuilder) |
//! | `spawn` / `despawn` | `coin` / `forget` |
//! | `Transform` | [`Phasor`](phasor::Phasor) |
//! | `Parent` / `Children` | [`Edge`](relation::Edge) / [`Graph`](relation::Graph) |
//! | `add_child` / `remove_child` | `relate` / `unrelate` |
//! | `Query` / `Query2` | `with` / `with2` / [`Related`](query::Related) |
//! | `System` / `SystemStage` | [`System`](system::System) / [`Stage`](system::Stage) |
//! | `Scene` / `EntityDescriptor` | [`Quest`](quest::Quest) / [`Step`](quest::Step) |
//! | `Resources` (orphaned) | [`Resources`](resources::Resources), world-owned |
//! | `GKStateMachine` | [`Condition`](condition::Condition) |
//! | *(none)* | [`Relation`](relation::Relation) — the kind of an edge |
//!
//! ```
//! use phiecs::prelude::*;
//!
//! let mut world = World::new();
//! let cat = world.coin("cat").bind(Phasor::seeded("cat")).word();
//! let mat = world.coin_bare("mat");
//! let on = world.declare("on").unwrap();
//! world.relate(cat, on, mat);
//! assert_eq!(world.calls_for(cat, on).first().map(|(w, _)| *w), Some(mat));
//! ```

pub mod aspect;
pub mod condition;
pub mod persona;
pub mod phasor;
pub mod quest;
pub mod query;
pub mod relation;
pub mod resources;
pub mod row;
pub mod scope;
pub mod speak;
pub mod kit;
pub mod store;
pub mod system;
pub mod word;
pub mod world;

pub use row::{parse_tsv, Row};

/// Everything a caller usually wants.
pub mod prelude {
    pub use crate::aspect::Aspect;
    pub use crate::condition::{Condition, ConditionSet};
    pub use crate::phasor::Phasor;
    pub use crate::quest::{Oracle, Quest, Scripted, Spawned};
    pub use crate::relation::{Edge, Relation, Register};
    pub use crate::row::{parse_tsv, Row};
    pub use crate::system::{FnSystem, Stage, System};
    pub use crate::word::{Lexicon, Word};
    pub use crate::world::World;
}
