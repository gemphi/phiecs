//! The face's fifty-two shapes. A tracked face is reported every frame as a
//! coefficient in `[0, 1]` per shape — how far the brow is down, the jaw
//! open, the left corner of the mouth up. The names are the ones face
//! tracking reports (ported from the ARKit blend-shape locations, in our
//! words); the region a shape belongs to is its first word, read, not
//! declared.

use crate::named;
use crate::kit::tell::Named;

named! {
    /// A shape of the face. Its words are its name: `MouthSmileLeft` is
    /// `mouth smile left`, of the region `mouth`.
    pub enum Shape {
        BrowDownLeft, BrowDownRight, BrowInnerUp, BrowOuterUpLeft, BrowOuterUpRight,
        CheekPuff, CheekSquintLeft, CheekSquintRight,
        EyeBlinkLeft, EyeBlinkRight, EyeLookDownLeft, EyeLookDownRight, EyeLookInLeft, EyeLookInRight,
        EyeLookOutLeft, EyeLookOutRight, EyeLookUpLeft, EyeLookUpRight, EyeSquintLeft, EyeSquintRight,
        EyeWideLeft, EyeWideRight,
        JawForward, JawLeft, JawOpen, JawRight,
        MouthClose, MouthDimpleLeft, MouthDimpleRight, MouthFrownLeft, MouthFrownRight, MouthFunnel, MouthLeft,
        MouthLowerDownLeft, MouthLowerDownRight, MouthPressLeft, MouthPressRight, MouthPucker, MouthRight,
        MouthRollLower, MouthRollUpper, MouthShrugLower, MouthShrugUpper, MouthSmileLeft, MouthSmileRight,
        MouthStretchLeft, MouthStretchRight, MouthUpperUpLeft, MouthUpperUpRight,
        NoseSneerLeft, NoseSneerRight,
        TongueOut,
    }
}

named! {
    /// A region of the face: the first word of a shape's name.
    pub enum Region { Brow, Cheek, Eye, Jaw, Mouth, Nose, Tongue }
}

named! {
    /// Which side a shape or a joint is on, when its name says.
    pub enum Side { Left, Right }
}

impl Shape {
    /// The shape's region — its first word.
    pub fn region(self) -> Region {
        Region::by_name(&self.words()[0]).expect("a shape's first word is a region")
    }

    /// The side the shape's name ends with, if it names one.
    pub fn side(self) -> Option<Side> {
        self.words().last().and_then(|w| Side::by_name(w))
    }

    /// The shape's index in a frame's coefficients, its place in [`Named::ALL`].
    pub fn index(self) -> usize {
        self as usize
    }
}

/// The coefficients of one frame, one per shape, in [`Shape::ALL`] order.
pub type Coefficients = [f32; Shape::ALL.len()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifty_two_shapes_in_seven_regions() {
        assert_eq!(Shape::ALL.len(), 52);
        assert_eq!(Shape::MouthSmileLeft.region(), Region::Mouth);
        assert_eq!(Shape::MouthSmileLeft.side(), Some(Side::Left));
        assert_eq!(Shape::JawOpen.side(), None);
        assert_eq!(Shape::TongueOut.index(), 51);
        for r in Region::ALL {
            assert!(Shape::ALL.iter().any(|s| s.region() == *r), "{r:?} has a shape");
        }
    }
}
