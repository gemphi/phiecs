//! How a session is to track: which mode (the world, a body, a face,
//! images, objects, the earth, position only, orientation only) and the
//! options each mode takes — plane detection, environment texturing,
//! scene reconstruction, the frame semantics, world alignment, and how
//! many faces or images to follow.

use super::camera::VideoFormat;
use super::light::Texturing;
use super::mesh::Reconstruction;
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

named! {
    /// What a session tracks.
    pub enum Mode { WorldTracking, BodyTracking, FaceTracking, ImageTracking, ObjectTracking, GeoTracking, PositionalTracking, OrientationTracking }
}

named! {
    /// Which planes to look for.
    pub enum PlaneDetection { None, Horizontal, Vertical, Slanted }
}

named! {
    /// What the world's y axis and heading are fixed to.
    pub enum WorldAlignment { Gravity, GravityAndHeading, Camera }
}

/// What else each frame is to carry.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Semantics {
    pub person_segmentation: bool,
    pub person_segmentation_with_depth: bool,
    pub body_detection: bool,
    pub scene_depth: bool,
    pub smoothed_scene_depth: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Configuration {
    pub mode: Mode,
    pub plane_detection: Vec<PlaneDetection>,
    pub texturing: Texturing,
    pub reconstruction: Reconstruction,
    pub semantics: Semantics,
    pub collaboration: bool,
    /// Track the person's face with the front camera while tracking the world with the back.
    pub user_face_tracking: bool,
    pub alignment: WorldAlignment,
    pub light_estimation: bool,
    pub max_faces: u32,
    pub max_images: u32,
    pub video_format: VideoFormat,
}

impl Configuration {
    /// The defaults for a mode: world tracking finds horizontal planes
    /// and estimates light; face tracking follows one face; image tracking one image.
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            plane_detection: if mode == Mode::WorldTracking { vec![PlaneDetection::Horizontal] } else { Vec::new() },
            texturing: Texturing::None,
            reconstruction: Reconstruction::None,
            semantics: Semantics::default(),
            collaboration: false,
            user_face_tracking: false,
            alignment: WorldAlignment::Gravity,
            light_estimation: true,
            max_faces: u32::from(mode == Mode::FaceTracking),
            max_images: u32::from(mode == Mode::ImageTracking),
            video_format: VideoFormat::default(),
        }
    }

    /// Whether this configuration can see a person's body or face.
    pub fn sees_a_person(&self) -> bool {
        matches!(self.mode, Mode::BodyTracking | Mode::FaceTracking) || self.user_face_tracking || self.semantics.body_detection
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new(Mode::WorldTracking)
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Mode>(path);
    r.extend(cases::<PlaneDetection>(path));
    r.extend(cases::<WorldAlignment>(path));
    r.extend(fields(path, "Semantics", &["person_segmentation", "person_segmentation_with_depth", "body_detection", "scene_depth", "smoothed_scene_depth"]));
    r.extend(fields(path, "Configuration", &["mode", "plane_detection", "texturing", "reconstruction", "semantics", "collaboration", "user_face_tracking", "alignment", "light_estimation", "max_faces", "max_images", "video_format"]));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_have_their_defaults() {
        assert_eq!(Configuration::default().plane_detection, [PlaneDetection::Horizontal]);
        let face = Configuration::new(Mode::FaceTracking);
        assert_eq!(face.max_faces, 1);
        assert!(face.sees_a_person() && !Configuration::default().sees_a_person());
    }
}
