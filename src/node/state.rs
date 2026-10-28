//! `state` — the values that change as a world steps, kept twice.
//!
//! A component is what an entity is; its state is where it is now. A step
//! reads every entity's state as the last step left it and writes the next
//! one, so the order systems run in cannot leak a half-written step into a
//! read (MuJoCo keeps the same split between `mjModel` and `mjData`).
//! [`State::commit`] makes the written step the read one; an entity not
//! written in a step keeps its state.

use crate::entity::Entity;

/// One kind of state for every entity that has it, indexed by the entity's id.
#[derive(Clone, Debug)]
pub struct State<T> {
    /// As the last committed step left it: what [`State::get`] reads.
    now: Vec<Option<T>>,
    /// The step being written: `now` plus this step's writes.
    next: Vec<Option<T>>,
    step: u64,
}

impl<T> Default for State<T> {
    fn default() -> Self {
        Self { now: Vec::new(), next: Vec::new(), step: 0 }
    }
}

impl<T: Clone> State<T> {
    pub fn new() -> Self {
        Self::default()
    }

    fn slot(v: &mut Vec<Option<T>>, e: Entity) -> &mut Option<T> {
        let i = e.id() as usize;
        if i >= v.len() {
            v.resize_with(i + 1, || None);
        }
        &mut v[i]
    }

    /// Starts `e` at `value`, readable at once (before any step).
    pub fn init(&mut self, e: Entity, value: T) {
        *Self::slot(&mut self.now, e) = Some(value.clone());
        *Self::slot(&mut self.next, e) = Some(value);
    }

    /// `e`'s state as the last committed step left it.
    pub fn get(&self, e: Entity) -> Option<&T> {
        self.now.get(e.id() as usize)?.as_ref()
    }

    /// Writes `e`'s state for the step being built; reads still see the
    /// last committed one until [`State::commit`].
    pub fn set(&mut self, e: Entity, value: T) {
        *Self::slot(&mut self.next, e) = Some(value);
    }

    /// `e`'s state in the step being built: this step's write, else the
    /// committed state.
    pub fn pending(&self, e: Entity) -> Option<&T> {
        self.next.get(e.id() as usize)?.as_ref()
    }

    /// Makes the written step the read one, and starts the next from it.
    pub fn commit(&mut self) {
        self.now.clone_from(&self.next);
        self.step += 1;
    }

    /// How many steps have been committed.
    pub fn step(&self) -> u64 {
        self.step
    }
}
