//! The body: what a device can see of a person, named the way the devices
//! name it and told as a tree of words. The face is fifty-two shapes in
//! seven regions; the body is the joints pose detection finds (two- and
//! three-dimensional) and the skeleton body tracking follows; the hand is
//! the twenty-one joints of hand-pose detection and the twenty-seven of
//! the tracked hand skeleton. Each frame of any of them is a sentence —
//! the shapes or joints it is sure of, in their words — so a persona
//! learns its body as it learns a story.
//!
//! | module | what |
//! |---|---|
//! | [`shape`] | `Shape` (52), `Region`, `Side`, `Coefficients` |
//! | [`face`] | `Face`: a frame of a tracked face; `rows` for a sequence of frames as a paragraph |
//! | [`pose`] | `Joint` (23), `Group`, `Located`, `Pose`: a detected pose |
//! | [`skeleton`] | `SkeletonJoint` (67), `Bone`, `Skeleton`, `Body`: a tracked body |
//! | [`hand`] | `Finger`, `Chirality`, `HandJoint` (21), `HandSkeletonJoint` (27), `Hand`, `Skeleton` |

pub mod face;
pub mod hand;
pub mod pose;
pub mod shape;
pub mod skeleton;

pub use face::Face;
pub use shape::{Region, Shape, Side};

use crate::kit::tell::{cases, row, scope_says, Named};
use crate::quest::Row;

/// The root of the body's tree.
pub const PATH: &str = "body";

/// Every module of the body told as rows under `body/<module>`.
pub fn told() -> Vec<Row> {
    let face = format!("{PATH}/face");
    let mut r = cases::<Shape>(&face);
    r.extend(cases::<Region>(&face));
    r.extend(cases::<Side>(&face));
    // A shape is in its region, as its first word says.
    for s in Shape::ALL {
        r.push(row(&s.name(), "in-region", &s.region().name()));
    }
    r.extend(face::told(&face));
    r.extend(pose::told(&format!("{PATH}/pose")));
    r.extend(skeleton::told(&format!("{PATH}/skeleton")));
    r.extend(hand::told(&format!("{PATH}/hand")));
    for m in ["face", "pose", "skeleton", "hand"] {
        r.extend(scope_says(&format!("{PATH}/{m}")));
    }
    r
}
