//! The body's joints as pose detection names them, in two and in three
//! dimensions (ported from Vision's body-pose joint names, in our words),
//! and a detected pose: each joint found, where, and how surely. A joint's
//! group — face, an arm, a leg, the torso — is read from its words.

use crate::named;
use crate::kit::tell::{cases, fields, Named};
use crate::kit::track::anchor::Point;
use crate::quest::Row;

named! {
    /// A joint of the body. The first nineteen are the two-dimensional
    /// pose's; `Spine`, `CenterShoulder`, `CenterHead` and `TopHead` are
    /// named by the three-dimensional pose, which has no nose, eyes or ears.
    pub enum Joint {
        Nose, LeftEye, RightEye, LeftEar, RightEar, Neck,
        LeftShoulder, RightShoulder, LeftElbow, RightElbow, LeftWrist, RightWrist,
        LeftHip, RightHip, LeftKnee, RightKnee, LeftAnkle, RightAnkle, Root,
        Spine, CenterShoulder, CenterHead, TopHead,
    }
}

named! {
    /// The groups a pose's joints are asked for by.
    pub enum Group { Face, LeftArm, RightArm, LeftLeg, RightLeg, Torso, All }
}

impl Joint {
    /// The joint's group, read from its words: an arm is a shoulder, an
    /// elbow or a wrist on a side; a leg a hip, a knee or an ankle; the face
    /// is the nose, the eyes, the ears and the head; the rest is the torso.
    pub fn group(self) -> Group {
        let w = self.words();
        let (side, part) = (w[0].as_str(), w.last().map(String::as_str).unwrap_or(""));
        match (side, part) {
            ("left", "shoulder" | "elbow" | "wrist") => Group::LeftArm,
            ("right", "shoulder" | "elbow" | "wrist") => Group::RightArm,
            ("left", "hip" | "knee" | "ankle") => Group::LeftLeg,
            ("right", "hip" | "knee" | "ankle") => Group::RightLeg,
            (_, "nose" | "eye" | "ear" | "head") => Group::Face,
            _ => Group::Torso,
        }
    }

    /// The joints of a group, in declaration order.
    pub fn of(group: Group) -> Vec<Joint> {
        Joint::ALL.iter().copied().filter(|j| group == Group::All || j.group() == group).collect()
    }

    /// Whether the two-dimensional pose reports this joint.
    pub fn is_planar(self) -> bool {
        (self as usize) < 19
    }
}

/// A joint as detected: where, and how surely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Located {
    pub joint: Joint,
    /// Normalised image coordinates for a planar pose (z = 0); metres
    /// from the root for a three-dimensional one.
    pub position: Point,
    pub confidence: f32,
}

/// One detected pose: the joints found in one frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pose {
    pub joints: Vec<Located>,
    /// The body's estimated height in metres, when the pose is three-dimensional.
    pub height: Option<f32>,
}

impl Pose {
    pub fn with(joints: &[(Joint, Point, f32)]) -> Self {
        Self { joints: joints.iter().map(|(j, p, c)| Located { joint: *j, position: *p, confidence: *c }).collect(), height: None }
    }

    pub fn find(&self, joint: Joint) -> Option<&Located> {
        self.joints.iter().find(|l| l.joint == joint)
    }

    /// The joints of `group` found with confidence at or above `tau`, surest first.
    pub fn said(&self, group: Group, tau: f32) -> Vec<Joint> {
        let mut v: Vec<&Located> = self.joints.iter().filter(|l| l.confidence >= tau && (group == Group::All || l.joint.group() == group)).collect();
        v.sort_by(|a, b| b.confidence.total_cmp(&a.confidence).then(a.joint.cmp(&b.joint)));
        v.into_iter().map(|l| l.joint).collect()
    }

    /// The pose as a sentence: the joints it is sure of, in their words.
    pub fn sentence(&self, tau: f32) -> String {
        let said: Vec<String> = self.said(Group::All, tau).iter().map(|j| j.words().join(" ")).collect();
        if said.is_empty() { "⟨rest⟩".to_string() } else { said.join(", ") }
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Joint>(path);
    r.extend(cases::<Group>(path));
    r.extend(fields(path, "Located", &["joint", "position", "confidence"]));
    r.extend(fields(path, "Pose", &["joints", "height"]));
    // A joint is `within` its group too, as pose detection groups it.
    for j in Joint::ALL {
        r.push(crate::kit::tell::row(&j.name(), "in-group", &j.group().name()));
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joints_group_by_their_words() {
        assert_eq!(Joint::ALL.len(), 23);
        assert_eq!(Joint::LeftElbow.group(), Group::LeftArm);
        assert_eq!(Joint::RightAnkle.group(), Group::RightLeg);
        assert_eq!(Joint::TopHead.group(), Group::Face);
        assert_eq!(Joint::CenterShoulder.group(), Group::Torso);
        assert_eq!(Joint::of(Group::LeftArm), [Joint::LeftShoulder, Joint::LeftElbow, Joint::LeftWrist]);
        assert_eq!(Joint::of(Group::Face).len(), 7);
        assert!(Joint::Root.is_planar() && !Joint::Spine.is_planar());
        let pose = Pose::with(&[(Joint::LeftWrist, [0.1, 0.2, 0.0], 0.9), (Joint::LeftElbow, [0.1, 0.4, 0.0], 0.5), (Joint::Nose, [0.5, 0.9, 0.0], 0.1)]);
        assert_eq!(pose.said(Group::LeftArm, 0.3), [Joint::LeftWrist, Joint::LeftElbow]);
        assert_eq!(pose.sentence(0.3), "left wrist, left elbow");
    }
}
