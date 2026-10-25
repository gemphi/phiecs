//! The hand. Two namings exist for it and both are kept, since both are
//! what devices report: the twenty-one joints of hand-pose detection
//! (`ThumbCMC`, `IndexMCP`, `IndexPIP`, `IndexDIP`, `IndexTip`, … — the
//! anatomical names, ported from Vision), and the twenty-seven joints of
//! the hand skeleton a headset tracks (`IndexFingerKnuckle`,
//! `IndexFingerIntermediateBase`, … with the forearm; ported from the
//! visionOS hand skeleton). A joint's finger is its first word.

use crate::named;
use crate::kit::tell::{cases, fields, Named};
use crate::kit::track::anchor::{Point, Transform};
use crate::quest::Row;

named! {
    /// The fingers.
    pub enum Finger { Thumb, Index, Middle, Ring, Little }
}

named! {
    /// Which hand.
    pub enum Chirality { Left, Right }
}

named! {
    /// A joint of the hand as hand-pose detection names it: the wrist and
    /// four joints per finger, root to tip.
    pub enum HandJoint {
        Wrist,
        ThumbCMC, ThumbMP, ThumbIP, ThumbTip,
        IndexMCP, IndexPIP, IndexDIP, IndexTip,
        MiddleMCP, MiddlePIP, MiddleDIP, MiddleTip,
        RingMCP, RingPIP, RingDIP, RingTip,
        LittleMCP, LittlePIP, LittleDIP, LittleTip,
    }
}

named! {
    /// A joint of the tracked hand skeleton: five per finger (the thumb
    /// has no metacarpal), the wrist, and the forearm at the wrist and the arm.
    pub enum HandSkeletonJoint {
        Wrist,
        ThumbKnuckle, ThumbIntermediateBase, ThumbIntermediateTip, ThumbTip,
        IndexFingerMetacarpal, IndexFingerKnuckle, IndexFingerIntermediateBase, IndexFingerIntermediateTip, IndexFingerTip,
        MiddleFingerMetacarpal, MiddleFingerKnuckle, MiddleFingerIntermediateBase, MiddleFingerIntermediateTip, MiddleFingerTip,
        RingFingerMetacarpal, RingFingerKnuckle, RingFingerIntermediateBase, RingFingerIntermediateTip, RingFingerTip,
        LittleFingerMetacarpal, LittleFingerKnuckle, LittleFingerIntermediateBase, LittleFingerIntermediateTip, LittleFingerTip,
        ForearmWrist, ForearmArm,
    }
}

/// The finger a joint's first word names, if it names one.
fn finger_of(words: &[String]) -> Option<Finger> {
    words.first().and_then(|w| Finger::by_name(w))
}

impl HandJoint {
    pub fn finger(self) -> Option<Finger> {
        finger_of(&self.words())
    }

    /// The joints of a finger, root to tip.
    pub fn of(finger: Finger) -> Vec<HandJoint> {
        HandJoint::ALL.iter().copied().filter(|j| j.finger() == Some(finger)).collect()
    }

    pub fn is_tip(self) -> bool {
        self.words().last().is_some_and(|w| w == "tip")
    }
}

impl HandSkeletonJoint {
    pub fn finger(self) -> Option<Finger> {
        finger_of(&self.words())
    }

    pub fn of(finger: Finger) -> Vec<HandSkeletonJoint> {
        HandSkeletonJoint::ALL.iter().copied().filter(|j| j.finger() == Some(finger)).collect()
    }

    /// The joint this one hangs from, root to tip along its finger; the
    /// finger's first joint and the forearm hang from the wrist.
    pub fn parent(self) -> Option<HandSkeletonJoint> {
        match self.finger() {
            Some(f) => {
                let chain = HandSkeletonJoint::of(f);
                let i = chain.iter().position(|j| *j == self)?;
                Some(if i == 0 { HandSkeletonJoint::Wrist } else { chain[i - 1] })
            }
            None => match self {
                HandSkeletonJoint::Wrist => None,
                HandSkeletonJoint::ForearmArm => Some(HandSkeletonJoint::ForearmWrist),
                _ => Some(HandSkeletonJoint::Wrist),
            },
        }
    }
}

/// A hand as hand-pose detection found it: which hand, and each joint's
/// place and confidence.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Hand {
    pub chirality: Option<Chirality>,
    pub joints: Vec<(HandJoint, Point, f32)>,
}

impl Hand {
    /// The joints found with confidence at or above `tau`, surest first.
    pub fn said(&self, tau: f32) -> Vec<HandJoint> {
        let mut v: Vec<&(HandJoint, Point, f32)> = self.joints.iter().filter(|(_, _, c)| *c >= tau).collect();
        v.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));
        v.into_iter().map(|(j, _, _)| *j).collect()
    }

    /// The hand as a sentence: its hand, then the joints it is sure of, in their words.
    pub fn sentence(&self, tau: f32) -> String {
        let mut said: Vec<String> = self.chirality.map(|c| format!("{} hand", c.words().join(" "))).into_iter().collect();
        said.extend(self.said(tau).iter().map(|j| j.words().join(" ")));
        if said.is_empty() { "⟨rest⟩".to_string() } else { said.join(", ") }
    }
}

/// A tracked hand skeleton: a transform per joint, and whether each is tracked.
#[derive(Debug, Clone, PartialEq)]
pub struct Skeleton {
    pub chirality: Chirality,
    pub joints: Vec<(HandSkeletonJoint, Transform, bool)>,
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Finger>(path);
    r.extend(cases::<Chirality>(path));
    r.extend(cases::<HandJoint>(path));
    r.extend(cases::<HandSkeletonJoint>(path));
    r.extend(fields(path, "Hand", &["chirality", "joints"]));
    r.extend(fields(path, "Skeleton", &["chirality", "joints"]));
    for j in HandSkeletonJoint::ALL {
        if let Some(p) = j.parent() {
            r.push(crate::kit::tell::row(&j.name(), "hangs-from", &p.name()));
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingers_are_read_from_the_names() {
        assert_eq!(HandJoint::ALL.len(), 21);
        assert_eq!(HandSkeletonJoint::ALL.len(), 27);
        assert_eq!(HandJoint::of(Finger::Thumb), [HandJoint::ThumbCMC, HandJoint::ThumbMP, HandJoint::ThumbIP, HandJoint::ThumbTip]);
        assert!(HandJoint::IndexTip.is_tip() && !HandJoint::IndexDIP.is_tip());
        assert_eq!(HandSkeletonJoint::IndexFingerTip.parent(), Some(HandSkeletonJoint::IndexFingerIntermediateTip));
        assert_eq!(HandSkeletonJoint::ThumbKnuckle.parent(), Some(HandSkeletonJoint::Wrist));
        assert_eq!(HandSkeletonJoint::ForearmArm.parent(), Some(HandSkeletonJoint::ForearmWrist));
        assert_eq!(HandSkeletonJoint::Wrist.parent(), None);
        let hand = Hand { chirality: Some(Chirality::Left), joints: vec![(HandJoint::ThumbTip, [0.0; 3], 0.9), (HandJoint::Wrist, [0.0; 3], 0.2)] };
        assert_eq!(hand.sentence(0.5), "left hand, thumb tip");
    }
}
