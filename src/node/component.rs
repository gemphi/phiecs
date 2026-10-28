//! `component` — data hung on an entity: phiecs's [`Aspect`](crate::aspect::Aspect)
//! under its ECS name, and [`Dense`], the id-indexed store each kind of
//! component lives in (O(1) per read and write, because the entity's id is
//! its slot).

pub use crate::aspect::Aspect as Component;
pub use crate::store::Dense;
