//! The kit — the body, the space and the device, in our words
//!
//! In Phient's world, a word (w) is a seed: what it relates to, under what
//! conditions (c), grows from it. The Phical paradigm, X ⇒ Y, drives and
//! composes Phient's Phasor Language Models: Phinet, Phiano, PhiTon. Phical
//! stores no point: a word is 32–64 relations, each a complex phasor on a
//! closed circle; meaning is a side effect of pairs of words moving under a
//! condition, and they move by the error of what was expected — toward what
//! came, away from what did not.
//!
//! The kit is the world a persona has a body in. It lived as its own
//! crate (phikit) until the name was given to the reality-kit repo; its
//! home is here, beside the world that holds it. It is a port — of the Rust
//! ARKit binding (`gemphi/phi/crates_extracted/crates/rust-ios-ar`), of the
//! joint and shape vocabularies of Apple's Vision and ARKit headers, and of
//! the kernel registry of IOKit (`gemphi/phi/apple-source/ruskit`) — into
//! this code space, named in our words: no vendor prefix survives, a
//! `Shape` is a `Shape`, a `Joint` a `Joint`, a `Plane` of the registry a
//! `Plane`. What a device reports (a blend-shape coefficient, a joint's
//! confidence, a registry key's string) is read into these names at the
//! edge and nowhere else.
//!
//! Two rules make the port a world rather than a binding, and both are the
//! paradigm's:
//!
//! - **a name is its words.** Every type and case is named the Rust way,
//!   and its words — `MouthSmileLeft` ⇒ `mouth smile left`,
//!   `IndexFingerKnuckle` ⇒ `index finger knuckle` — are the thing's own
//!   name as a sequence, coarse to fine. Every enumeration and struct is
//!   told as rows ([`tell`]): a scope that says its words, `within` its
//!   kind, within its module, within its domain, so the whole interface is
//!   one tree of conditions a cast can be drawn into, at one depth.
//! - **a frame is a sentence.** What a tracked face, pose or hand is sure
//!   of in one frame is what it says, in its words; a sequence of frames
//!   is a paragraph; so a persona learns its body the way it learns a
//!   story, and a `System` ([`watch`]) learns it into a phiecs world as
//!   pairs of words moving under `near` and `before`.
//!
//! | domain | what |
//! |---|---|
//! | [`body`] | the face (52 shapes), the pose (23 joints), the skeleton (67), the hand (21 and 27) |
//! | [`track`] | anchors, frames, sessions, cameras, planes, meshes, depth, light, raycasts, the earth, maps, sharing, rooms |
//! | [`device`] | the registry in its planes, services and their matching, the property keys |
//! | [`whole`] | all of it as rows and a phiecs world (casting lives in phient's `relate::kit`) |
//! | [`watch`] | the system that reads frames into the world |
//! | [`tell`], [`words`] | how a type becomes rows; how a name becomes words |
//!

pub mod body;
pub mod device;
pub mod eye;
pub mod whole;
pub mod tell;
pub mod track;
pub mod watch;
pub mod words;

pub use whole::Kit;
pub use tell::Named;
pub use watch::Watch;
