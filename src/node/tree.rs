//! `tree` — one parent, many children: the hierarchy a graph does not give.
//!
//! A [`Graph`](super::Graph) relates anything to anything. A tree is the
//! stricter shape a body or a scene needs: every entity has at most one
//! parent and none is its own ancestor, so "where does this part hang"
//! (its parents up to the root) and "what moves with it" (everything under
//! it) each have one answer. RealityKit keeps this as `Parent` / `Children`.

use crate::entity::Entity;

/// Parent and children per entity, indexed by the entity's id.
#[derive(Clone, Debug, Default)]
pub struct Tree {
    parent: Vec<Option<Entity>>,
    children: Vec<Vec<Entity>>,
}

impl Tree {
    pub fn new() -> Self {
        Self::default()
    }

    fn grow(&mut self, e: Entity) {
        let i = e.id() as usize;
        if i >= self.parent.len() {
            self.parent.resize(i + 1, None);
            self.children.resize_with(i + 1, Vec::new);
        }
    }

    /// Hangs `child` under `parent`, taking it from any parent it had; its
    /// own children come with it. Refused (false, nothing changed) when
    /// `parent` is `child` or stands under it, which would close a loop.
    pub fn attach(&mut self, child: Entity, parent: Entity) -> bool {
        match child == parent || self.is_ancestor(child, parent) {
            true => false,
            false => {
                self.detach(child);
                self.grow(child);
                self.grow(parent);
                self.parent[child.id() as usize] = Some(parent);
                self.children[parent.id() as usize].push(child);
                true
            }
        }
    }

    /// Takes `e` from its parent, returning the parent it had; its own
    /// children stay with it.
    pub fn detach(&mut self, e: Entity) -> Option<Entity> {
        let p = self.parent.get_mut(e.id() as usize)?.take()?;
        self.children[p.id() as usize].retain(|c| *c != e);
        Some(p)
    }

    pub fn parent(&self, e: Entity) -> Option<Entity> {
        self.parent.get(e.id() as usize).copied().flatten()
    }

    /// `e`'s children, in the order they were attached.
    pub fn children(&self, e: Entity) -> &[Entity] {
        self.children.get(e.id() as usize).map_or(&[], Vec::as_slice)
    }

    /// `e`'s parents, nearest first, up to its root.
    pub fn ancestors(&self, e: Entity) -> impl Iterator<Item = Entity> + '_ {
        std::iter::successors(self.parent(e), move |p| self.parent(*p))
    }

    /// Whether `a` stands above `b`.
    pub fn is_ancestor(&self, a: Entity, b: Entity) -> bool {
        self.ancestors(b).any(|p| p == a)
    }

    /// Everything under `e`, depth first in attach order; `e` itself is not
    /// included.
    pub fn descendants(&self, e: Entity) -> Vec<Entity> {
        let mut out = Vec::new();
        let mut stack: Vec<Entity> = self.children(e).iter().rev().copied().collect();
        while let Some(n) = stack.pop() {
            out.push(n);
            stack.extend(self.children(n).iter().rev().copied());
        }
        out
    }

    /// The top of each tree: entities with children and no parent.
    pub fn roots(&self) -> Vec<Entity> {
        (0..self.children.len())
            .filter(|&i| self.parent[i].is_none() && !self.children[i].is_empty())
            .map(|i| Entity::from_id(i as u32))
            .collect()
    }
}
