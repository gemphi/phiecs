//! Telling a world a table of rows — the one Row → World loop (docs/159).
//! `Quest::from_rows` and phikit's `Kit::world` both used to write it.

use super::{World, COUPLE_RATE};
use crate::row::Row;

impl World {
    /// Tell the world every row: declare the kind, coin both words (a fresh
    /// word is seeded by `relate_by` from the one seed), and relate them by
    /// the row's weight, exactly as when the oracle said it.
    pub fn tell(&mut self, rows: &[Row]) {
        for r in rows {
            let Some(kind) = self.declare(&r.kind) else { continue };
            let head = self.coin_bare(&r.about);
            let other = self.coin_bare(&r.other);
            self.relate_by(head, kind, other, r.weight, COUPLE_RATE * r.weight);
        }
    }

    /// A world built from a table alone.
    pub fn from_rows(rows: &[Row]) -> Self {
        let mut world = World::new();
        world.tell(rows);
        world
    }
}
