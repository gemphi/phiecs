//! Capturing a room: a scan that runs while the device is walked around,
//! with its settings, progress, the rooms it has finished, and the formats
//! a room can be exported in.

use super::room::Room;
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

named! {
    /// A format a captured room can be written in.
    pub enum Export { Usdz, Usd, FloorPlan, Json }
}

named! {
    /// Why a capture cannot go on.
    pub enum CaptureError { DeviceNotSupported, CameraAccessDenied, ScanningFailed, ExportFailed, RoomTooSmall, InsufficientLighting }
}

/// How a capture runs.
#[derive(Debug, Clone, PartialEq)]
pub struct CaptureSettings {
    pub geometry: bool,
    pub elements: bool,
    pub furniture: bool,
    pub export: Export,
    /// Square metres below which a room is too small to keep.
    pub minimum_area: f32,
}

impl Default for CaptureSettings {
    fn default() -> Self {
        Self { geometry: true, elements: true, furniture: true, export: Export::Usdz, minimum_area: 2.0 }
    }
}

/// What the capture shows the person while scanning.
#[derive(Debug, Clone, PartialEq)]
pub struct Guidance {
    pub show_progress: bool,
    pub show_outline: bool,
    pub show_surface_colors: bool,
    pub text: String,
}

impl Default for Guidance {
    fn default() -> Self {
        Self { show_progress: true, show_outline: true, show_surface_colors: false, text: "Move slowly around the room".into() }
    }
}

/// A capture in progress or done.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Capture {
    pub settings: CaptureSettings,
    pub guidance: Guidance,
    pub capturing: bool,
    pub current: Option<Room>,
    pub rooms: Vec<Room>,
    /// 0 to 1.
    pub progress: f32,
}

impl Capture {
    pub fn new(settings: CaptureSettings) -> Self {
        Self { settings, ..Default::default() }
    }

    pub fn start(&mut self, room: Room) {
        self.current = Some(room);
        self.capturing = true;
        self.progress = 0.0;
    }

    /// Advance the scan; at 1 the room is done.
    pub fn advance(&mut self, by: f32) {
        self.progress = (self.progress + by).min(1.0);
    }

    /// Stop, keeping the room if it is big enough.
    pub fn stop(&mut self) -> Result<&Room, CaptureError> {
        self.capturing = false;
        let room = self.current.take().ok_or(CaptureError::ScanningFailed)?;
        if room.dimensions.area() < self.settings.minimum_area {
            return Err(CaptureError::RoomTooSmall);
        }
        self.rooms.push(room);
        Ok(self.rooms.last().expect("just pushed"))
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Export>(path);
    r.extend(cases::<CaptureError>(path));
    r.extend(fields(path, "CaptureSettings", &["geometry", "elements", "furniture", "export", "minimum_area"]));
    r.extend(fields(path, "Guidance", &["show_progress", "show_outline", "show_surface_colors", "text"]));
    r.extend(fields(path, "Capture", &["settings", "guidance", "capturing", "current", "rooms", "progress"]));
    r
}

#[cfg(test)]
mod tests {
    use super::super::room::{Category, Dimensions};
    use super::*;

    #[test]
    fn a_capture_keeps_rooms_big_enough() {
        let mut c = Capture::default();
        c.start(Room::new("closet", Category::Storage, Dimensions { width: 1.0, length: 1.0, height: 2.0 }));
        c.advance(0.7);
        c.advance(0.7);
        assert_eq!(c.progress, 1.0);
        assert_eq!(c.stop().unwrap_err(), CaptureError::RoomTooSmall);
        c.start(Room::new("study", Category::Office, Dimensions { width: 3.0, length: 4.0, height: 2.5 }));
        assert_eq!(c.stop().unwrap().name, "study");
        assert_eq!(c.rooms.len(), 1);
    }
}
