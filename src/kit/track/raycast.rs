//! Asking the world what lies along a line: a query from a point in a
//! direction (or from a point on the screen), what it may hit, and the
//! results — a transform where it hit, the plane if one, the distance. A
//! tracked raycast keeps answering as tracking improves.

use super::anchor::{Point, Point2, Transform, IDENTITY};
use super::camera::Camera;
use super::surface::Plane;
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

named! {
    /// What a raycast may hit.
    pub enum Target { ExistingPlaneGeometry, ExistingPlaneInfinite, EstimatedPlane }
}

named! {
    /// Which planes count.
    pub enum TargetAlignment { Horizontal, Vertical, Any }
}

named! {
    /// What a result hit.
    pub enum Hit { None, ExistingPlaneUsingExtent, ExistingPlaneUsingGeometry, EstimatedPlane, FeaturePoint, EstimatedHorizontalPlane, EstimatedVerticalPlane }
}

/// A line to ask along.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Query {
    pub origin: Point,
    pub direction: Point,
    pub target: Target,
    pub alignment: TargetAlignment,
}

impl Query {
    pub fn new(origin: Point, direction: Point, target: Target, alignment: TargetAlignment) -> Self {
        Self { origin, direction, target, alignment }
    }

    /// The line through a pixel of `camera`, from the camera out.
    pub fn from_screen(camera: &Camera, px: Point2, target: Target, alignment: TargetAlignment) -> Self {
        let d = camera.unproject(px, 1.0);
        let n = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-9);
        Self { origin: camera.transform[3][..3].try_into().unwrap_or([0.0; 3]), direction: [d[0] / n, d[1] / n, d[2] / n], target, alignment }
    }

    /// The point `t` along the line.
    pub fn at(&self, t: f32) -> Point {
        [self.origin[0] + t * self.direction[0], self.origin[1] + t * self.direction[1], self.origin[2] + t * self.direction[2]]
    }

    /// Where the line meets a horizontal plane at height `y`, if ahead.
    pub fn hit_height(&self, y: f32) -> Option<Result> {
        if self.direction[1].abs() < 1e-9 {
            return None;
        }
        let t = (y - self.origin[1]) / self.direction[1];
        if t < 0.0 {
            return None;
        }
        let p = self.at(t);
        let mut transform = IDENTITY;
        transform[3] = [p[0], p[1], p[2], 1.0];
        Some(Result { transform, plane: None, distance: t, hit: Hit::EstimatedHorizontalPlane })
    }
}

/// One answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Result {
    pub transform: Transform,
    pub plane: Option<Plane>,
    pub distance: f32,
    pub hit: Hit,
}

/// A raycast that keeps answering.
#[derive(Debug, Clone, PartialEq)]
pub struct Tracked {
    pub query: Query,
    pub results: Vec<Result>,
    pub active: bool,
}

impl Tracked {
    pub fn new(query: Query) -> Self {
        Self { query, results: Vec::new(), active: true }
    }

    pub fn update(&mut self, query: Query) {
        self.query = query;
    }

    pub fn stop(&mut self) {
        self.active = false;
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Target>(path);
    r.extend(cases::<TargetAlignment>(path));
    r.extend(cases::<Hit>(path));
    r.extend(fields(path, "Query", &["origin", "direction", "target", "alignment"]));
    r.extend(fields(path, "Result", &["transform", "plane", "distance", "hit"]));
    r.extend(fields(path, "Tracked", &["query", "results", "active"]));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_meets_the_floor() {
        let q = Query::new([0.0, 1.5, 0.0], [0.0, -0.6, -0.8], Target::EstimatedPlane, TargetAlignment::Horizontal);
        let r = q.hit_height(0.0).unwrap();
        assert!((r.distance - 2.5).abs() < 1e-5);
        assert!((r.transform[3][2] + 2.0).abs() < 1e-5);
        assert!(q.hit_height(2.0).is_none(), "behind the origin");
        let from_screen = Query::from_screen(&Camera::default(), [960.0, 720.0], Target::EstimatedPlane, TargetAlignment::Any);
        assert!((from_screen.direction[2] + 1.0).abs() < 1e-5, "the image centre looks straight ahead");
    }
}
