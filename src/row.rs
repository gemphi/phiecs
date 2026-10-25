//! `Row` — the one interchange type of the workspace (docs/159): every source
//! (an oracle, prose, a strand, Rust code, a phikit vocabulary) becomes rows,
//! and every reader counts them. `about \t kind \t other \t weight`, the line
//! of a `relations.tsv`. Lives at the crate root because phient and phikit
//! depend on phiecs for this type before anything else.

/// One row: `about \t kind \t other \t weight`.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub about: String,
    pub kind: String,
    pub other: String,
    pub weight: f64,
}

impl Row {
    /// A row of full weight: the common case, where a source states a
    /// relation rather than measuring one.
    pub fn new(about: &str, kind: &str, other: &str) -> Self {
        Self::weighted(about, kind, other, 1.0)
    }

    /// A row with a given weight: a measure in `0..=1` from an oracle, or a
    /// count from a strand. Only [`parse_tsv`] clamps, as the file format does.
    pub fn weighted(about: &str, kind: &str, other: &str, weight: f64) -> Self {
        Self { about: about.to_string(), kind: kind.to_string(), other: other.to_string(), weight }
    }
}

/// Parse the table a quest writes. Blank lines and a header row (one whose
/// weight does not parse) are skipped; nothing else is forgiven, so a
/// malformed file is noticed rather than half-read.
pub fn parse_tsv(text: &str) -> Vec<Row> {
    text.lines()
        .filter_map(|line| {
            let mut f = line.split('\t').map(str::trim);
            let (about, kind, other, weight) = (f.next()?, f.next()?, f.next()?, f.next()?);
            let weight = weight.parse::<f64>().ok()?;
            if about.is_empty() || kind.is_empty() || other.is_empty() {
                return None;
            }
            Some(Row::weighted(about, kind, other, weight.clamp(0.0, 1.0)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_the_header() {
        let rows = parse_tsv("about\tkind\tother\tweight\ncat\tis-a\tanimal\t0.9\n");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].other, "animal");
    }
}
