//! The tracked body: a skeleton of joints each with a transform, a parent
//! and a confidence, under a body anchor with the body's estimated scale.
//! The joint names are the body skeleton's as the Rust ARKit binding
//! defines them (`rust-ios-ar`'s `define_2d_body_joints` and
//! `define_3d_body_joints`, in our words): thirty-two for the planar
//! skeleton, thirty-five more for the three-dimensional one, which adds
//! the hands' joints.

use crate::named;
use crate::kit::tell::{cases, fields, Named};
use crate::kit::track::anchor::{Anchor, Transform, IDENTITY};
use crate::quest::Row;

named! {
    /// A joint of the tracked skeleton.
    pub enum SkeletonJoint {
        Root, TorsoCenter, Neck1, Head,
        LeftShoulder1, LeftElbow, LeftWrist, LeftHand,
        RightShoulder1, RightElbow, RightWrist, RightHand,
        LeftHip, LeftKnee, LeftAnkle, LeftFoot,
        RightHip, RightKnee, RightAnkle, RightFoot,
        Spine1, Spine2, Spine3, Spine4, Spine5, Spine6, Spine7,
        LeftHandThumbTip, LeftHandIndexTip, LeftHandMiddleTip, LeftHandRingTip, LeftHandLittleTip,
        LeftThumbCmc, LeftThumbMcp, LeftThumbIp,
        LeftIndexMcp, LeftIndexPip, LeftIndexDip,
        LeftMiddleMcp, LeftMiddlePip, LeftMiddleDip,
        LeftRingMcp, LeftRingPip, LeftRingDip,
        LeftLittleMcp, LeftLittlePip, LeftLittleDip,
        RightThumbCmc, RightThumbMcp, RightThumbIp,
        RightIndexMcp, RightIndexPip, RightIndexDip, RightIndexTip,
        RightMiddleMcp, RightMiddlePip, RightMiddleDip, RightMiddleTip,
        RightRingMcp, RightRingPip, RightRingDip, RightRingTip,
        RightLittleMcp, RightLittlePip, RightLittleDip, RightLittleTip,
        RightThumbTip,
    }
}

impl SkeletonJoint {
    /// The joints of the planar skeleton, in order.
    pub fn planar() -> &'static [SkeletonJoint] {
        &SkeletonJoint::ALL[..32]
    }

    /// Whether the three-dimensional skeleton alone reports this joint.
    pub fn is_spatial_only(self) -> bool {
        (self as usize) >= 32
    }
}

/// One joint of a skeleton as tracked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bone {
    pub joint: SkeletonJoint,
    /// Relative to the parent joint.
    pub local: Transform,
    /// Relative to the body anchor.
    pub model: Transform,
    pub parent: Option<SkeletonJoint>,
    pub confidence: f32,
}

/// The skeleton: its bones in the order tracking reports them.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
}

impl Skeleton {
    /// A skeleton of `joints` at rest, each at the identity with no parent.
    pub fn at_rest(joints: &[SkeletonJoint]) -> Self {
        Self { bones: joints.iter().map(|j| Bone { joint: *j, local: IDENTITY, model: IDENTITY, parent: None, confidence: 0.0 }).collect() }
    }

    pub fn joint_count(&self) -> usize {
        self.bones.len()
    }

    pub fn find(&self, joint: SkeletonJoint) -> Option<&Bone> {
        self.bones.iter().find(|b| b.joint == joint)
    }

    pub fn find_mut(&mut self, joint: SkeletonJoint) -> Option<&mut Bone> {
        self.bones.iter_mut().find(|b| b.joint == joint)
    }

    /// The joints from `joint` up to the root, by the parents reported.
    pub fn chain(&self, joint: SkeletonJoint) -> Vec<SkeletonJoint> {
        let mut out = vec![joint];
        let mut cur = joint;
        while let Some(p) = self.find(cur).and_then(|b| b.parent) {
            if out.contains(&p) {
                break;
            }
            out.push(p);
            cur = p;
        }
        out
    }

    /// The joints tracked with confidence at or above `tau`, surest first.
    pub fn said(&self, tau: f32) -> Vec<SkeletonJoint> {
        let mut v: Vec<&Bone> = self.bones.iter().filter(|b| b.confidence >= tau).collect();
        v.sort_by(|a, b| b.confidence.total_cmp(&a.confidence).then(a.joint.cmp(&b.joint)));
        v.into_iter().map(|b| b.joint).collect()
    }
}

/// A tracked body.
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub anchor: Anchor,
    pub skeleton: Skeleton,
    /// The body's size relative to the reference skeleton.
    pub scale: f32,
}

impl Default for Body {
    fn default() -> Self {
        Self { anchor: Anchor::default(), skeleton: Skeleton::at_rest(SkeletonJoint::planar()), scale: 1.0 }
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<SkeletonJoint>(path);
    r.extend(fields(path, "Bone", &["joint", "local", "model", "parent", "confidence"]));
    r.extend(fields(path, "Skeleton", &["bones"]));
    r.extend(fields(path, "Body", &["anchor", "skeleton", "scale"]));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skeleton_chains_to_its_root() {
        assert_eq!(SkeletonJoint::ALL.len(), 67);
        assert_eq!(SkeletonJoint::planar().len(), 32);
        assert!(SkeletonJoint::LeftThumbCmc.is_spatial_only() && !SkeletonJoint::LeftHand.is_spatial_only());
        let mut s = Skeleton::at_rest(&[SkeletonJoint::Root, SkeletonJoint::Spine1, SkeletonJoint::LeftShoulder1, SkeletonJoint::LeftElbow]);
        s.find_mut(SkeletonJoint::Spine1).unwrap().parent = Some(SkeletonJoint::Root);
        s.find_mut(SkeletonJoint::LeftShoulder1).unwrap().parent = Some(SkeletonJoint::Spine1);
        s.find_mut(SkeletonJoint::LeftElbow).unwrap().parent = Some(SkeletonJoint::LeftShoulder1);
        s.find_mut(SkeletonJoint::LeftElbow).unwrap().confidence = 0.8;
        assert_eq!(s.chain(SkeletonJoint::LeftElbow), [SkeletonJoint::LeftElbow, SkeletonJoint::LeftShoulder1, SkeletonJoint::Spine1, SkeletonJoint::Root]);
        assert_eq!(s.said(0.5), [SkeletonJoint::LeftElbow]);
        assert_eq!(Body::default().skeleton.joint_count(), 32);
    }
}
