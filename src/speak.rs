//! A paragraph spoken as rows. Moved down from phient's prose (which
//! re-exports these, so the old address still answers): the builders that
//! turn words into [`Row`](crate::Row)s belong beside the one intermediate,
//! so the kit's face can speak frames and a story can speak sentences with
//! the same mouth.
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c), grows from it.

use crate::quest::Row;
use crate::scope::{nest, scopes, SAYS};

/// The end of a paragraph, as a word.
pub const END: &str = "⟨end⟩";

/// A typographic apostrophe (U+2019, U+02BC) is an apostrophe only BETWEEN
/// two alphanumerics; standing alone it is a closing quote, and punctuation.
/// Typeset prose carries U+2019 rather than the straight one, and reading it
/// as punctuation shatters every contraction into fragments -- t, s, ve --
/// which then crowd the vocabulary as if they were words. Measured on the
/// English guide: 8,199 of them, and `t` was the 6th most frequent token of
/// the whole corpus (docs/198).
fn held(before: Option<char>, c: char, after: Option<char>) -> bool {
    matches!(c, '\u{2019}' | '\u{02BC}')
        && before.is_some_and(char::is_alphanumeric)
        && after.is_some_and(char::is_alphanumeric)
}

/// Lower-case words and punctuation marks, each a token.
pub fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &raw) in chars.iter().enumerate() {
        let before = if i > 0 { chars.get(i - 1).copied() } else { None };
        let c = if held(before, raw, chars.get(i + 1).copied()) { '\'' } else { raw };
        if c.is_alphanumeric() || c == '\'' {
            word.extend(c.to_lowercase());
        } else {
            if !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
            if !c.is_whitespace() {
                out.push(c.to_string());
            }
        }
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

fn row(about: &str, kind: &str, other: &str) -> Row {
    Row::new(about, kind, other)
}

/// One paragraph: its sentences' tokens in order, each occurrence a row
/// `w@<path>#i` told `has-word w`, `within` every scope on `path`, `pair`
/// the word before it with itself (`pair:u w`, D8: a word's condition is
/// set by the words before it), `in-word` and `in-pair` the same under
/// each scope below the root (`scope:text/story|w`), what it `says`
/// (itself), and `next` the token after it (`END` after the last).
pub fn paragraph(path: &str, sentences: &[String]) -> Vec<Row> {
    let scopes = scopes(path, 0);
    let toks: Vec<String> = sentences.iter().flat_map(|s| tokens(s)).collect();
    let mut rows = nest(path, 0);
    for (i, t) in toks.iter().enumerate() {
        let occ = format!("{t}@{path}#{i}");
        rows.push(row(&occ, "has-word", t));
        for s in &scopes {
            rows.push(row(&occ, "within", s));
        }
        if i > 0 {
            rows.push(row(&occ, "pair", &format!("pair:{} {t}", toks[i - 1])));
        }
        // The word, and the pair, under each scope of the path: what this
        // word goes on to *in this story*, before what it goes on to anywhere.
        for s in scopes.iter().skip(1) {
            rows.push(row(&occ, "in-word", &format!("{s}|{t}")));
            if i > 0 {
                rows.push(row(&occ, "in-pair", &format!("{s}|pair:{} {t}", toks[i - 1])));
            }
        }
        rows.push(row(&occ, SAYS, t));
        if i > 0 {
            // A scope says its pairs too, so a cast can be drawn by two words
            // together (`november is` is a key; `november` alone is a month).
            rows.push(row(&occ, SAYS, &format!("pair:{} {t}", toks[i - 1])));
        }
        rows.push(row(&occ, "next", toks.get(i + 1).map_or(END, String::as_str)));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_paragraph_is_rows_under_its_path() {
        let rows = paragraph("story/ch1/p1", &["He remained.".into(), "She came.".into()]);
        assert!(rows.iter().any(|r| r.about == "he@story/ch1/p1#0" && r.kind == "next" && r.other == "remained"));
        assert!(rows.iter().any(|r| r.about == ".@story/ch1/p1#2" && r.kind == "next" && r.other == "she"));
        assert!(rows.iter().any(|r| r.about == "came@story/ch1/p1#4" && r.kind == "within" && r.other == "scope:story/ch1/p1"));
        assert!(rows.iter().any(|r| r.about == "scope:story/ch1/p1" && r.kind == "within" && r.other == "scope:story/ch1"));
        assert_eq!(tokens("She suddenly had an idea: their new"), ["she", "suddenly", "had", "an", "idea", ":", "their", "new"]);
    }

    #[test]
    fn a_typeset_contraction_is_one_word_and_a_closing_quote_is_not() {
        // U+2019 between letters holds the word together...
        assert_eq!(tokens("don\u{2019}t it\u{2019}s I\u{2019}ve"), ["don't", "it's", "i've"]);
        // ...but the same character closing a quotation is punctuation.
        assert_eq!(tokens("\u{2018}go home\u{2019} now"), ["\u{2018}", "go", "home", "\u{2019}", "now"]);
        assert_eq!(tokens("the dogs\u{2019} bowls"), ["the", "dogs", "\u{2019}", "bowls"]);
    }
}
