//! The kit told as rows. Every enumeration in the kit is a [`Named`] type —
//! a kind with its cases — and every struct a kind with its fields; both
//! are told with phiecs's `Row` in the quest's own kinds (`is-a`, `has`,
//! `within`, `has-word`, `says`), so phient's `Tally` and `ScopeTree` read
//! the kit the way they read a story or a crate.
//!
//! For a case `MouthSmileLeft` of kind `Shape` under the path `body/face`:
//! - the thing is `shape:mouth_smile_left`; it is `within` `scope:body`,
//!   `scope:body/face`, `scope:body/face/shape` and its own scope
//!   `scope:body/face/shape/mouth smile left`; the scopes are `within`
//!   each other; the kind's scope says the kind's words;
//! - it `is-a` `shape`, `has-word` and `says` each of its words, and
//!   `says` its pairs, so a cast can be drawn by two words together.
//!
//! Every thing is one scope under its kind, so a longer name draws no more
//! than a shorter one (docs/140): the tree is domain → module → kind →
//! thing, at one depth.

use crate::kit::words::{snake, split};
use crate::quest::Row;
use crate::scope::{nest, scopes, SAYS, SCOPE};

/// A kind with its cases, each named by its identifier.
pub trait Named: Copy + 'static {
    /// The kind's name (`Shape`).
    const KIND: &'static str;
    /// Every case, in declaration order.
    const ALL: &'static [Self];
    /// The case's identifier (`MouthSmileLeft`).
    fn ident(self) -> &'static str;
    /// The thing's name in the rows: `shape:mouth_smile_left`.
    fn name(self) -> String {
        format!("{}:{}", snake(Self::KIND), snake(self.ident()))
    }
    /// The case's words (`mouth smile left`).
    fn words(self) -> Vec<String> {
        split(self.ident())
    }
    /// The case whose name or words are `s`, if any.
    fn by_name(s: &str) -> Option<Self> {
        let s = snake(s);
        Self::ALL.iter().copied().find(|c| snake(c.ident()) == s)
    }
}

/// Declare an enumeration that is [`Named`]: `named! { pub enum Shape { BrowDownLeft, … } }`.
#[macro_export]
macro_rules! named {
    ($(#[$m:meta])* $v:vis enum $name:ident { $($(#[$cm:meta])* $case:ident),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        $v enum $name { $($(#[$cm])* $case),+ }
        impl $crate::kit::tell::Named for $name {
            const KIND: &'static str = stringify!($name);
            const ALL: &'static [$name] = &[$($name::$case),+];
            fn ident(self) -> &'static str { match self { $($name::$case => stringify!($case)),+ } }
        }
    };
}

pub(crate) fn row(a: &str, k: &str, o: &str) -> Row {
    Row::new(a, k, o)
}

/// The rows of one thing named `words`, of `kind`, under `path/<kind words>`.
fn thing(name: &str, kind: &str, words: &[String], path: &str) -> Vec<Row> {
    let kind_words = split(kind);
    let full = format!("{path}/{}/{}", kind_words.join(" "), words.join(" "));
    let path_scopes = scopes(&full, 0);
    let mut out = nest(&full, 0);
    for s in &path_scopes {
        out.push(row(name, "within", s));
    }
    // The kind is a name too: its scope says its own words, so `hand` tells
    // the hand's joints from the body's.
    if let Some(ks) = path_scopes.iter().rev().nth(1) {
        for w in &kind_words {
            out.push(row(ks, "has-word", w));
            out.push(row(ks, SAYS, w));
        }
    }
    for (i, w) in words.iter().enumerate() {
        out.push(row(name, "has-word", w));
        out.push(row(name, SAYS, w));
        if i > 0 {
            out.push(row(name, SAYS, &format!("pair:{} {w}", words[i - 1])));
        }
    }
    out
}

/// Every case of `T` told under `path` (`body/face`).
pub fn cases<T: Named>(path: &str) -> Vec<Row> {
    let kind = snake(T::KIND);
    T::ALL
        .iter()
        .flat_map(|c| {
            let mut r = thing(&c.name(), T::KIND, &c.words(), path);
            r.push(row(&c.name(), "is-a", &kind));
            r
        })
        .collect()
}

/// A struct `kind` with its `fields` told under `path`: the kind `has`
/// each field, and each field is a thing of the kind's scope.
pub fn fields(path: &str, kind: &str, fields: &[&str]) -> Vec<Row> {
    let k = snake(kind);
    fields
        .iter()
        .flat_map(|f| {
            let name = format!("{k}.{}", snake(f));
            let mut r = thing(&name, kind, &split(f), path);
            r.push(row(&k, "has", &name));
            r
        })
        .collect()
}

/// The scopes of a module path say their own words, once: `scope:body`
/// says `body`, `scope:body/pose` says `pose`, so `pose left elbow` tells
/// the pose's elbow from the skeleton's.
pub fn scope_says(path: &str) -> Vec<Row> {
    let mut out = Vec::new();
    for s in scopes(path, 0) {
        let Some(last) = s.rsplit('/').next() else { continue };
        for w in split(last.trim_start_matches(SCOPE)) {
            out.push(row(&s, "has-word", &w));
            out.push(row(&s, SAYS, &w));
        }
    }
    out
}

/// The innermost scope of a thing's path.
pub fn leaf_scope(rows: &[Row], name: &str) -> Option<String> {
    rows.iter().filter(|r| r.about == name && r.kind == "within" && r.other.starts_with(SCOPE)).map(|r| r.other.clone()).max_by_key(|s| s.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    named! { enum Side { Left, RightHand } }

    #[test]
    fn a_case_is_a_scope_of_its_words_under_its_kind() {
        assert_eq!(Side::RightHand.name(), "side:right_hand");
        assert_eq!(Side::by_name("right hand"), Some(Side::RightHand));
        let rows = cases::<Side>("body/test");
        let leaf = leaf_scope(&rows, "side:right_hand").unwrap();
        assert_eq!(leaf, "scope:body/test/side/right hand");
        assert!(rows.iter().any(|r| r.about == leaf && r.kind == "within" && r.other == "scope:body/test/side"));
        assert!(rows.iter().any(|r| r.about == "scope:body/test/side" && r.kind == SAYS && r.other == "side"));
        assert!(rows.iter().any(|r| r.about == "side:right_hand" && r.kind == SAYS && r.other == "pair:right hand"));
        assert!(rows.iter().any(|r| r.about == "side:left" && r.kind == "is-a" && r.other == "side"));
        let f = fields("track/test", "Camera", &["focal_length"]);
        assert!(f.iter().any(|r| r.about == "camera" && r.kind == "has" && r.other == "camera.focal_length"));
        assert_eq!(leaf_scope(&f, "camera.focal_length").unwrap(), "scope:track/test/camera/focal length");
    }
}
