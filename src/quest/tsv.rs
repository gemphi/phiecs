#[allow(unused_imports)]
use super::*;

use std::path::Path;

/// `Row` and `parse_tsv` live at the crate root (docs/159); re-exported
/// here so `phiecs::quest::Row` keeps working.
pub use crate::row::{parse_tsv, Row};

impl Quest {
    /// Rebuild a world from a written `relations.tsv`: every row becomes an
    /// edge and a coupling, exactly as it did when the oracle said it. The
    /// steps are not recovered — the table is the graph, the pages are the
    /// transcript — so the result is a quest with a world and no history,
    /// ready to be continued from any word.
    pub fn from_rows(rows: &[Row]) -> Self {
        let seed = rows.first().map(|r| r.about.as_str()).unwrap_or("word");
        let mut quest = Quest::new(seed);
        quest.world.tell(rows);
        for r in rows {
            if !quest.asked.iter().any(|a| a == &r.about) {
                quest.asked.push(r.about.clone());
            }
        }
        quest.frontier.clear();
        quest
    }

    /// [`Quest::from_rows`] on a file.
    pub fn from_tsv(path: &Path) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Ok(Self::from_rows(&parse_tsv(&text)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_synthetic_quest() {
        let mut oracle = Synthetic::new(16);
        let mut quest = Quest::new("w0").with_depth(3).with_budget(64);
        quest.run(&mut oracle);
        let tsv: String = quest.steps.iter().map(Step::to_tsv).collect();
        let rows = parse_tsv(&tsv);
        assert_eq!(rows.len(), quest.world.edges.len());
        let back = Quest::from_rows(&rows);
        assert_eq!(back.world.edges.len(), quest.world.edges.len());
        assert_eq!(back.world.lexicon.len(), quest.world.lexicon.len());
        // Leaves answered nothing and left no rows, so only words with edges
        // count as asked in the rebuilt quest.
        let with_edges = quest.steps.iter().filter(|s| !s.kept.is_empty()).count();
        assert_eq!(back.asked().len(), with_edges);
    }
}
