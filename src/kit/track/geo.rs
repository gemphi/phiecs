//! Places on the earth: an anchor at a latitude, longitude and altitude,
//! and the state of tracking against the earth's map.

use super::anchor::{Anchor, Transform};
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

/// An anchor fixed to a place on the earth.
#[derive(Debug, Clone, PartialEq)]
pub struct GeoAnchor {
    pub anchor: Anchor,
    pub latitude: f64,
    pub longitude: f64,
    /// Metres above sea level, when known.
    pub altitude: Option<f64>,
}

impl GeoAnchor {
    pub fn new(transform: Transform, latitude: f64, longitude: f64, altitude: Option<f64>) -> Self {
        Self { anchor: Anchor::new(transform), latitude, longitude, altitude }
    }

    /// The great-circle distance to another place, in metres.
    pub fn distance_to(&self, other: &GeoAnchor) -> f64 {
        let r = 6_371_000.0;
        let (p1, p2) = (self.latitude.to_radians(), other.latitude.to_radians());
        let (dp, dl) = ((other.latitude - self.latitude).to_radians(), (other.longitude - self.longitude).to_radians());
        let a = (dp / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
        2.0 * r * a.sqrt().asin()
    }
}

named! {
    /// Where geo-tracking is in finding itself.
    pub enum GeoState { NotAvailable, NeedsLocationPermissions, Initializing, Localizing, Localized }
}

named! {
    /// Why geo-tracking is where it is.
    pub enum GeoReason { None, NotAvailableAtLocation, NeedLocationPermissions, WorldTrackingUnstable, WaitingForLocation, WaitingForAvailabilityCheck, GeoDataNotLoaded, DevicePointedTooLow, VisualLocalizationFailed }
}

named! {
    /// How accurate the earth fix is.
    pub enum GeoAccuracy { Undetermined, Low, Medium, High }
}

/// The state of geo-tracking in a frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoStatus {
    pub state: GeoState,
    pub reason: GeoReason,
    pub accuracy: GeoAccuracy,
}

impl Default for GeoStatus {
    fn default() -> Self {
        Self { state: GeoState::NotAvailable, reason: GeoReason::None, accuracy: GeoAccuracy::Undetermined }
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<GeoState>(path);
    r.extend(cases::<GeoReason>(path));
    r.extend(cases::<GeoAccuracy>(path));
    r.extend(fields(path, "GeoAnchor", &["anchor", "latitude", "longitude", "altitude"]));
    r.extend(fields(path, "GeoStatus", &["state", "reason", "accuracy"]));
    r
}

#[cfg(test)]
mod tests {
    use super::super::anchor::IDENTITY;
    use super::*;

    #[test]
    fn distances_on_the_earth() {
        let a = GeoAnchor::new(IDENTITY, 51.5074, -0.1278, None);
        let b = GeoAnchor::new(IDENTITY, 48.8566, 2.3522, None);
        let d = a.distance_to(&b);
        assert!((d - 343_500.0).abs() < 2_000.0, "{d}");
    }
}
