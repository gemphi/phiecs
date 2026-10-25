//! Surfaces tracking finds: planes, with their alignment, classification,
//! extent and geometry, and known images it recognises and follows.

use super::anchor::{Anchor, Point, Transform};
use super::mesh::Mesh;
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

named! {
    /// How a plane lies.
    pub enum Alignment { Horizontal, Vertical, Slanted, Unknown }
}

named! {
    /// What a plane is, when tracking can say.
    pub enum PlaneClassification { Wall, Floor, Ceiling, Table, Seat, Window, Door, None, Unknown }
}

named! {
    /// Whether classification of a plane is available, and whether it is decided.
    pub enum PlaneClassificationStatus { NotAvailable, Undetermined, Unknown, Known }
}

/// A detected plane.
#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    pub anchor: Anchor,
    pub alignment: Alignment,
    pub classification: PlaneClassification,
    pub status: PlaneClassificationStatus,
    /// The centre of the plane in the anchor's space.
    pub center: Point,
    /// Its extent along the anchor's x and z.
    pub extent: Point,
    pub geometry: Mesh,
}

impl Plane {
    pub fn new(transform: Transform, alignment: Alignment) -> Self {
        Self { anchor: Anchor::new(transform), alignment, classification: PlaneClassification::Unknown, status: PlaneClassificationStatus::NotAvailable, center: [0.0; 3], extent: [0.0; 3], geometry: Mesh::default() }
    }

    /// The plane's area from its extent.
    pub fn area(&self) -> f32 {
        self.extent[0] * self.extent[2]
    }
}

/// A recognised image being followed.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub anchor: Anchor,
    /// The name the reference image was given.
    pub reference: String,
    /// The physical size seen over the size the reference was declared with.
    pub scale: f32,
}

impl Image {
    pub fn new(transform: Transform, reference: impl Into<String>) -> Self {
        Self { anchor: Anchor::new(transform), reference: reference.into(), scale: 1.0 }
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Alignment>(path);
    r.extend(cases::<PlaneClassification>(path));
    r.extend(cases::<PlaneClassificationStatus>(path));
    r.extend(fields(path, "Plane", &["anchor", "alignment", "classification", "status", "center", "extent", "geometry"]));
    r.extend(fields(path, "Image", &["anchor", "reference", "scale"]));
    r
}

#[cfg(test)]
mod tests {
    use super::super::anchor::IDENTITY;
    use super::*;

    #[test]
    fn a_plane_has_an_area_and_an_image_a_reference() {
        let mut p = Plane::new(IDENTITY, Alignment::Horizontal);
        p.extent = [2.0, 0.0, 1.5];
        assert_eq!(p.area(), 3.0);
        assert_eq!(Image::new(IDENTITY, "poster").reference, "poster");
        assert!(told("track/surface").iter().any(|r| r.about == "plane_classification:floor"));
    }
}
