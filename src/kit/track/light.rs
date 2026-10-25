//! Light as tracking estimates it: the ambient intensity and colour
//! temperature, the direction and intensity of the main light when a face
//! is tracked, and environment probes that capture the surroundings as a
//! texture for reflections.

use super::anchor::{Anchor, Point, Transform};
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

/// The light in a frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LightEstimate {
    /// In lumens; 1000 is a well-lit room.
    pub ambient_intensity: f32,
    /// In kelvin; 6500 is daylight.
    pub ambient_color_temperature: f32,
}

impl Default for LightEstimate {
    fn default() -> Self {
        Self { ambient_intensity: 1000.0, ambient_color_temperature: 6500.0 }
    }
}

/// The main light's direction and intensity, estimated from a face.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DirectionalLight {
    pub estimate: LightEstimate,
    /// A unit vector toward the light.
    pub direction: Point,
    pub intensity: f32,
    /// Spherical-harmonic coefficients of the light, when given (27 floats: 9 per channel).
    pub harmonics: Option<[f32; 27]>,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self { estimate: LightEstimate::default(), direction: [0.0, -1.0, 0.0], intensity: 0.0, harmonics: None }
    }
}

named! {
    /// How environment textures are produced.
    pub enum Texturing { None, Manual, Automatic }
}

/// A cube-map of the surroundings, one square face at a time.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EnvironmentTexture {
    pub size: u32,
    /// Six faces, each `size × size × 4` bytes, when captured.
    pub faces: Vec<Vec<u8>>,
}

/// A probe: a place whose surroundings are captured, over an extent.
#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    pub anchor: Anchor,
    pub extent: Point,
    pub texture: Option<EnvironmentTexture>,
}

impl Probe {
    pub fn new(transform: Transform, extent: Point) -> Self {
        Self { anchor: Anchor::new(transform), extent, texture: None }
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Texturing>(path);
    r.extend(fields(path, "LightEstimate", &["ambient_intensity", "ambient_color_temperature"]));
    r.extend(fields(path, "DirectionalLight", &["estimate", "direction", "intensity", "harmonics"]));
    r.extend(fields(path, "EnvironmentTexture", &["size", "faces"]));
    r.extend(fields(path, "Probe", &["anchor", "extent", "texture"]));
    r
}
