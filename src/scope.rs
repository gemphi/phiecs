//! Paths as scopes, scopes as rows. These live beside [`Row`](crate::Row)
//! because every teller — a story, a kit, a file tree — writes them before
//! its own rows, and the crates above (phient's relate, the prose tree)
//! read them back; the builders belong where the one intermediate lives.

use crate::quest::Row;

/// The prefix a scope wears as a word.
pub const SCOPE: &str = "scope:";

/// The kind by which a scope says its words.
pub const SAYS: &str = "says";

/// A path's components as scope conditions, outermost first, each the
/// path down to it (so two paragraphs both called `p1` in two chapters are
/// two scopes), the last `drop` components left out (a file's own name
/// when it is the thing asked about): `a/b/c.rs` ⇒ `[scope:a, scope:a/b,
/// scope:a/b/c]`.
pub fn scopes(path: &str, drop: usize) -> Vec<String> {
    let parts: Vec<&str> = path.split(['/', '\\']).filter(|p| !p.is_empty() && *p != ".").collect();
    let n = parts.len().saturating_sub(drop);
    let mut so_far = String::new();
    parts[..n]
        .iter()
        .map(|p| {
            let stem = p.rsplit_once('.').filter(|x| !x.0.is_empty()).map_or(*p, |x| x.0);
            so_far = if so_far.is_empty() { stem.to_string() } else { format!("{so_far}/{stem}") };
            format!("{SCOPE}{so_far}")
        })
        .collect()
}

/// The scopes of `path` told `within` each other, inner within outer: the
/// rows every source writes before its own (a paragraph, a kit's thing, a
/// file), so the tree is one tree however it was told.
pub fn nest(path: &str, drop: usize) -> Vec<Row> {
    scopes(path, drop).windows(2).map(|w| Row::new(&w[1], "within", &w[0])).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scopes_are_the_path() {
        assert_eq!(scopes("data/above/above_i64.rs", 0), ["scope:data", "scope:data/above", "scope:data/above/above_i64"]);
        assert_eq!(scopes("data/above/above_i64.rs", 1), ["scope:data", "scope:data/above"]);
        let n = nest("a/b/c", 0);
        assert_eq!(n, [Row::new("scope:a/b", "within", "scope:a"), Row::new("scope:a/b/c", "within", "scope:a/b")]);
    }
}
