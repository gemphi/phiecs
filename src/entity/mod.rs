//! `entity` — the id everything in a world hangs on. Level 0 of phiecs.
//!
//! An [`Entity`] is an opaque id issued by a [`Names`] table. A word is the
//! entity named by its spelling; a body, a joint, a geom or a device is named
//! the same way, so one world can hold a language and a robot side by side.
//!
//! These are the ECS names of [`Word`](crate::word::Word) and
//! [`Lexicon`](crate::word::Lexicon), the same types: every word-keyed store,
//! graph, tree and system already takes entities, and code that says `Word`
//! keeps working.

pub use crate::word::{Lexicon as Names, Word as Entity};
