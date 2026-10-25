//! A frame: everything tracking knows at one instant — the camera, the
//! anchors it holds, the light, the picture, depth and feature points,
//! how far the space is mapped, and who is in the picture. A frame is
//! also a sentence: what it says is what its anchors say.

use super::anchor::Anchor;
use super::camera::Camera;
use super::depth::{CapturedImage, PointCloud, SceneDepth, Segmentation};
use super::geo::GeoAnchor;
use super::light::{DirectionalLight, LightEstimate, Probe};
use super::map::Mapping;
use super::mesh::MeshAnchor;
use super::share::Participant;
use super::surface::{Image, Plane};
use crate::kit::body::face::Face;
use crate::kit::body::skeleton::Body;
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;
use std::time::Duration;

/// Any tracked thing, by what it is.
#[derive(Debug, Clone, PartialEq)]
pub enum Tracked {
    Plain(Anchor),
    Plane(Plane),
    Image(Image),
    Face(Face),
    Body(Body),
    Mesh(MeshAnchor),
    Geo(GeoAnchor),
    Participant(Participant),
    Probe(Probe),
}

named! {
    /// The kinds of tracked thing, for telling.
    pub enum Kind { Plain, Plane, Image, Face, Body, Mesh, Geo, Participant, Probe }
}

impl Tracked {
    pub fn anchor(&self) -> &Anchor {
        match self {
            Tracked::Plain(a) => a,
            Tracked::Plane(p) => &p.anchor,
            Tracked::Image(i) => &i.anchor,
            Tracked::Face(f) => &f.anchor,
            Tracked::Body(b) => &b.anchor,
            Tracked::Mesh(m) => &m.anchor,
            Tracked::Geo(g) => &g.anchor,
            Tracked::Participant(p) => &p.anchor,
            Tracked::Probe(p) => &p.anchor,
        }
    }

    pub fn kind(&self) -> Kind {
        match self {
            Tracked::Plain(_) => Kind::Plain,
            Tracked::Plane(_) => Kind::Plane,
            Tracked::Image(_) => Kind::Image,
            Tracked::Face(_) => Kind::Face,
            Tracked::Body(_) => Kind::Body,
            Tracked::Mesh(_) => Kind::Mesh,
            Tracked::Geo(_) => Kind::Geo,
            Tracked::Participant(_) => Kind::Participant,
            Tracked::Probe(_) => Kind::Probe,
        }
    }

    pub fn id(&self) -> u64 {
        self.anchor().id
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub timestamp: Duration,
    pub camera: Camera,
    pub anchors: Vec<Tracked>,
    pub light: LightEstimate,
    pub directional_light: Option<DirectionalLight>,
    pub image: Option<CapturedImage>,
    pub depth: Option<SceneDepth>,
    pub points: PointCloud,
    pub mapping: Mapping,
    pub segmentation: Option<Segmentation>,
}

impl Default for Frame {
    fn default() -> Self {
        Self { timestamp: Duration::ZERO, camera: Camera::default(), anchors: Vec::new(), light: LightEstimate::default(), directional_light: None, image: None, depth: None, points: PointCloud::default(), mapping: Mapping::NotAvailable, segmentation: None }
    }
}

impl Frame {
    pub fn at(timestamp: Duration) -> Self {
        Self { timestamp, ..Default::default() }
    }

    pub fn faces(&self) -> impl Iterator<Item = &Face> {
        self.anchors.iter().filter_map(|t| if let Tracked::Face(f) = t { Some(f) } else { None })
    }

    pub fn bodies(&self) -> impl Iterator<Item = &Body> {
        self.anchors.iter().filter_map(|t| if let Tracked::Body(b) = t { Some(b) } else { None })
    }

    pub fn planes(&self) -> impl Iterator<Item = &Plane> {
        self.anchors.iter().filter_map(|t| if let Tracked::Plane(p) = t { Some(p) } else { None })
    }

    /// The frame as a sentence: each face's shapes above `tau`, in order; a
    /// frame with no face is a pause.
    pub fn sentence(&self, tau: f32) -> String {
        let said: Vec<String> = self.faces().map(|f| f.sentence(tau)).collect();
        if said.is_empty() { "⟨rest⟩".to_string() } else { said.join("; ") }
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Kind>(path);
    r.extend(fields(path, "Frame", &["timestamp", "camera", "anchors", "light", "directional_light", "image", "depth", "points", "mapping", "segmentation"]));
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::body::shape::Shape;

    #[test]
    fn a_frame_says_what_its_faces_say() {
        let mut f = Frame::at(Duration::from_millis(33));
        assert_eq!(f.sentence(0.3), "⟨rest⟩");
        f.anchors.push(Tracked::Face(Face::with(&[(Shape::JawOpen, 0.8)])));
        f.anchors.push(Tracked::Plain(Anchor::default()));
        assert_eq!(f.sentence(0.3), "jaw open");
        assert_eq!(f.faces().count(), 1);
        assert_eq!(f.anchors[1].kind(), Kind::Plain);
    }
}
