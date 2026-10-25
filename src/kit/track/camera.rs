//! The camera: where it is and how it is set — its transform, intrinsics,
//! tracking state, exposure, focus and white balance, the video formats it
//! can run at — and the projection from the world into its image.

use super::anchor::{Point, Point2, Reason, Tracking, Transform, IDENTITY};
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;
use std::time::Duration;

named! {
    /// How the device is held.
    pub enum Orientation { Unknown, Portrait, PortraitUpsideDown, LandscapeLeft, LandscapeRight }
}

named! { pub enum Focus { Fixed, Auto, Locked } }
named! { pub enum Exposure { Auto, Locked } }
named! { pub enum WhiteBalanceMode { Auto, Locked } }

/// A resolution and rate the camera can capture at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VideoFormat {
    pub resolution: (u32, u32),
    pub frames_per_second: u32,
    pub hdr: bool,
    pub high_resolution: bool,
}

impl VideoFormat {
    /// The formats the binding lists; the first is the one world tracking recommends.
    pub const SUPPORTED: [VideoFormat; 3] = [
        VideoFormat { resolution: (1920, 1440), frames_per_second: 60, hdr: false, high_resolution: true },
        VideoFormat { resolution: (1280, 720), frames_per_second: 60, hdr: false, high_resolution: false },
        VideoFormat { resolution: (1920, 1080), frames_per_second: 30, hdr: true, high_resolution: true },
    ];
}

impl Default for VideoFormat {
    fn default() -> Self {
        Self::SUPPORTED[0]
    }
}

/// The gains of the white balance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WhiteBalance {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl Default for WhiteBalance {
    fn default() -> Self {
        Self { red: 1.0, green: 1.0, blue: 1.0 }
    }
}

/// The camera in one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub transform: Transform,
    /// Focal lengths in pixels and the principal point: `fx, fy, cx, cy`.
    pub intrinsics: [f32; 4],
    pub resolution: (u32, u32),
    pub tracking: Tracking,
    pub reason: Reason,
    pub exposure_duration: Duration,
    pub exposure_offset: f32,
    pub iso: f32,
    pub focal_length: f32,
    pub focus: Focus,
    pub exposure: Exposure,
    pub white_balance: WhiteBalance,
    pub white_balance_mode: WhiteBalanceMode,
}

impl Default for Camera {
    fn default() -> Self {
        let f = VideoFormat::default();
        let (w, h) = f.resolution;
        Self {
            transform: IDENTITY,
            intrinsics: [w as f32, w as f32, w as f32 / 2.0, h as f32 / 2.0],
            resolution: f.resolution,
            tracking: Tracking::NotAvailable,
            reason: Reason::None,
            exposure_duration: Duration::from_millis(16),
            exposure_offset: 0.0,
            iso: 100.0,
            focal_length: w as f32,
            focus: Focus::Auto,
            exposure: Exposure::Auto,
            white_balance: WhiteBalance::default(),
            white_balance_mode: WhiteBalanceMode::Auto,
        }
    }
}

impl Camera {
    /// A point in the camera's space onto its image, in pixels; `None` behind the camera.
    pub fn project(&self, p: Point) -> Option<Point2> {
        let [fx, fy, cx, cy] = self.intrinsics;
        // The camera looks down its −z.
        if p[2] >= 0.0 {
            return None;
        }
        Some([cx + fx * p[0] / -p[2], cy - fy * p[1] / -p[2]])
    }

    /// A pixel at `depth` metres back into the camera's space.
    pub fn unproject(&self, px: Point2, depth: f32) -> Point {
        let [fx, fy, cx, cy] = self.intrinsics;
        [(px[0] - cx) * depth / fx, -(px[1] - cy) * depth / fy, -depth]
    }

    /// The horizontal and vertical fields of view, in radians.
    pub fn field_of_view(&self) -> (f32, f32) {
        let (w, h) = self.resolution;
        (2.0 * (w as f32 / (2.0 * self.intrinsics[0])).atan(), 2.0 * (h as f32 / (2.0 * self.intrinsics[1])).atan())
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Orientation>(path);
    r.extend(cases::<Focus>(path));
    r.extend(cases::<Exposure>(path));
    r.extend(cases::<WhiteBalanceMode>(path));
    r.extend(fields(path, "VideoFormat", &["resolution", "frames_per_second", "hdr", "high_resolution"]));
    r.extend(fields(path, "WhiteBalance", &["red", "green", "blue"]));
    r.extend(fields(path, "Camera", &["transform", "intrinsics", "resolution", "tracking", "reason", "exposure_duration", "exposure_offset", "iso", "focal_length", "focus", "exposure", "white_balance", "white_balance_mode"]));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_and_unprojection_agree() {
        let c = Camera::default();
        let p = [0.3, -0.2, -2.0];
        let px = c.project(p).unwrap();
        let back = c.unproject(px, 2.0);
        for i in 0..3 {
            assert!((back[i] - p[i]).abs() < 1e-4, "{back:?} vs {p:?}");
        }
        assert!(c.project([0.0, 0.0, 1.0]).is_none());
        let (fx, _) = c.field_of_view();
        // fx is one image width, so half the width subtends atan(1/2).
        assert!((fx - 2.0 * 0.5f32.atan()).abs() < 1e-5);
    }
}
