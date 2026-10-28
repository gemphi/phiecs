//! `graph` — typed, weighted edges between entities, many to many: the
//! relation graph under its ECS name. An edge's kind is a [`Kind`] issued by
//! a [`Kinds`] register (phiecs's [`Relation`](crate::relation::Relation)
//! and [`Register`](crate::relation::Register)), and the kind is the payload:
//! the condition a pair is related under.

pub use crate::relation::{Edge, Graph, Register as Kinds, Relation as Kind};
