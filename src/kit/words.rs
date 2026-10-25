//! A name is its words. The kit's types and cases are named the Rust way —
//! `MouthSmileLeft`, `LeftElbow`, `IndexFingerKnuckle`, `DeviceTree` — and
//! the words of such a name, split at its case changes and underscores, are
//! the thing's own name as a sequence, coarse to fine: `mouth smile left`.
//! The tree is built from these words and nothing else.

/// CamelCase and snake_case split into lower-case words. Digits stay with
/// the word they follow (`Pose3D` ⇒ `pose3d`); a run of capitals is one
/// word up to its last letter (`CMC`, `MCP`, `DIP`; `IOService` ⇒ `io
/// service`).
pub fn split(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == '_' || c == '-' || c == ' ' {
            flush(&mut out, &mut cur);
            continue;
        }
        let prev = i.checked_sub(1).map(|j| chars[j]);
        let next = chars.get(i + 1).copied();
        let boundary = match (prev, c) {
            (Some(p), c) if c.is_ascii_uppercase() && p.is_ascii_lowercase() => true,
            // After a digit a capital starts a word only when a word follows it: `Pose3D` ⇒ pose3d, `3DPoint` ⇒ 3d, point.
            (Some(p), c) if c.is_ascii_uppercase() && p.is_ascii_digit() && next.is_some_and(|n| n.is_ascii_lowercase()) => true,
            // The last capital of a run starts a new word when a lowercase follows: `MCPJoint` ⇒ mcp, joint.
            (Some(p), c) if c.is_ascii_uppercase() && p.is_ascii_uppercase() && next.is_some_and(|n| n.is_ascii_lowercase()) => true,
            _ => false,
        };
        if boundary {
            flush(&mut out, &mut cur);
        }
        cur.push(c.to_ascii_lowercase());
    }
    flush(&mut out, &mut cur);
    out
}

/// The words joined by `_`: the thing's name as an identifier
/// (`MouthSmileLeft` ⇒ `mouth_smile_left`).
pub fn snake(s: &str) -> String {
    split(s).join("_")
}

/// The words joined by a space: the thing's name as it is said
/// (`MouthSmileLeft` ⇒ `mouth smile left`).
pub fn said(s: &str) -> String {
    split(s).join(" ")
}

fn flush(out: &mut Vec<String>, cur: &mut String) {
    if !cur.is_empty() {
        out.push(std::mem::take(cur));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_split_into_their_words() {
        assert_eq!(split("MouthSmileLeft"), ["mouth", "smile", "left"]);
        assert_eq!(split("ThumbCMC"), ["thumb", "cmc"]);
        assert_eq!(split("IndexMCP"), ["index", "mcp"]);
        assert_eq!(split("index_finger_knuckle"), ["index", "finger", "knuckle"]);
        assert_eq!(split("Pose3D"), ["pose3d"]);
        assert_eq!(split("IOService"), ["io", "service"]);
        assert_eq!(snake("LeftElbow"), "left_elbow");
        assert_eq!(said("DeviceTree"), "device tree");
    }
}
