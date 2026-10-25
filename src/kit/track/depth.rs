//! What a frame carries besides the picture: the captured image, the
//! scene's depth with its confidence, the feature points tracking holds
//! on to, and the segmentation of people from the scene.

use super::anchor::Point;
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

named! {
    /// How good the depth is.
    pub enum DepthQuality { Unknown, Low, High }
}

named! {
    /// Whether depth is in true metres or only relative.
    pub enum DepthAccuracy { Relative, Absolute }
}

/// A picture, four bytes per pixel.
#[derive(Debug, Clone, PartialEq)]
pub struct CapturedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl CapturedImage {
    pub fn blank(width: u32, height: u32) -> Self {
        Self { width, height, pixels: vec![0; (width * height * 4) as usize] }
    }
}

/// A map of one value per pixel.
#[derive(Debug, Clone, PartialEq)]
pub struct Map<T> {
    pub width: u32,
    pub height: u32,
    pub data: Vec<T>,
}

impl<T: Copy + Default> Map<T> {
    pub fn filled(width: u32, height: u32, value: T) -> Self {
        Self { width, height, data: vec![value; (width * height) as usize] }
    }

    /// The value at a pixel; the default outside the map.
    pub fn at(&self, x: u32, y: u32) -> T {
        if x >= self.width || y >= self.height {
            return T::default();
        }
        self.data.get((y * self.width + x) as usize).copied().unwrap_or_default()
    }
}

/// Depth in metres per pixel, and how sure each is (0, 1, 2 for low, medium, high).
#[derive(Debug, Clone, PartialEq)]
pub struct SceneDepth {
    pub depth: Map<f32>,
    pub confidence: Map<u8>,
    pub quality: DepthQuality,
    pub accuracy: DepthAccuracy,
}

/// The feature points tracking is following, each with its identifier.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PointCloud {
    pub points: Vec<Point>,
    pub identifiers: Vec<u64>,
}

impl PointCloud {
    pub fn count(&self) -> usize {
        self.points.len()
    }

    pub fn push(&mut self, p: Point, id: u64) {
        self.points.push(p);
        self.identifiers.push(id);
    }
}

/// Which pixels are a person: `0` none, else the person's index.
pub type Segmentation = Map<u8>;

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<DepthQuality>(path);
    r.extend(cases::<DepthAccuracy>(path));
    r.extend(fields(path, "CapturedImage", &["width", "height", "pixels"]));
    r.extend(fields(path, "SceneDepth", &["depth", "confidence", "quality", "accuracy"]));
    r.extend(fields(path, "PointCloud", &["points", "identifiers"]));
    r.extend(fields(path, "Segmentation", &["width", "height", "data"]));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_read_inside_and_default_outside() {
        let mut m: Map<f32> = Map::filled(4, 2, 1.5);
        m.data[5] = 3.0;
        assert_eq!(m.at(1, 1), 3.0);
        assert_eq!(m.at(0, 0), 1.5);
        assert_eq!(m.at(9, 9), 0.0);
        let mut c = PointCloud::default();
        c.push([1.0, 2.0, 3.0], 7);
        assert_eq!(c.count(), 1);
        assert_eq!(CapturedImage::blank(2, 2).pixels.len(), 16);
    }
}
