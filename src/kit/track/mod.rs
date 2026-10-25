//! Tracking: what a device knows of the space it is in and of the things
//! in it, frame by frame. Ported from the Rust ARKit binding
//! (`gemphi/phi/crates_extracted/crates/rust-ios-ar/src/arkit`), in our
//! words: an anchor is a place tracking holds on to; a frame is everything
//! known at one instant; a session is tracking run under a configuration,
//! telling what happened as events. Every enumeration and struct here is
//! told as rows too, so the whole interface is one tree of words a cast
//! can be drawn into.
//!
//! | module | what |
//! |---|---|
//! | [`anchor`] | `Anchor`, `Transform`, `Point`, `Tracking`, `Reason` |
//! | [`camera`] | `Camera` with projection, `VideoFormat`, `Orientation`, focus, exposure, white balance |
//! | [`frame`] | `Frame`, `Tracked` (every kind of anchor) |
//! | [`session`] | `Session`, `Configuration` run, `Event`, `Happening`, `RunOptions`, `SessionError` |
//! | [`config`] | `Configuration`, `Mode`, `PlaneDetection`, `WorldAlignment`, `Semantics` |
//! | [`surface`] | `Plane` with `Alignment` and classification, `Image` |
//! | [`mesh`] | `Mesh`, `MeshAnchor`, `Classification`, `Reconstruction`, `Scene` |
//! | [`depth`] | `CapturedImage`, `Map`, `SceneDepth`, `PointCloud`, `Segmentation` |
//! | [`light`] | `LightEstimate`, `DirectionalLight`, `Probe`, `Texturing` |
//! | [`raycast`] | `Query`, `Result`, `Tracked` raycast, `Target`, `Hit` |
//! | [`geo`] | `GeoAnchor`, `GeoState`, `GeoReason`, `GeoAccuracy` |
//! | [`map`] | `WorldMap` saved and loaded, `Mapping` |
//! | [`share`] | `Participant`, `Collaboration`, `Role`, coaching `Goal` and state |
//! | [`room`] | `Room`, its surfaces, openings, furniture and materials |
//! | [`capture`] | `Capture`: a room scan with settings, guidance and export |

pub mod anchor;
pub mod camera;
pub mod capture;
pub mod config;
pub mod depth;
pub mod frame;
pub mod geo;
pub mod light;
pub mod map;
pub mod mesh;
pub mod raycast;
pub mod room;
pub mod session;
pub mod share;
pub mod surface;

pub use anchor::{Anchor, Point, Transform, IDENTITY};
pub use config::{Configuration, Mode};
pub use frame::{Frame, Tracked};
pub use session::{Event, Session};

use crate::quest::Row;

/// The root of tracking's tree.
pub const PATH: &str = "track";

/// Every module of tracking told as rows under `track/<module>`.
pub fn told() -> Vec<Row> {
    let at = |m: &str| format!("{PATH}/{m}");
    let mut r = anchor::told(&at("anchor"));
    r.extend(camera::told(&at("camera")));
    r.extend(frame::told(&at("frame")));
    r.extend(session::told(&at("session")));
    r.extend(config::told(&at("config")));
    r.extend(surface::told(&at("surface")));
    r.extend(mesh::told(&at("mesh")));
    r.extend(depth::told(&at("depth")));
    r.extend(light::told(&at("light")));
    r.extend(raycast::told(&at("raycast")));
    r.extend(geo::told(&at("geo")));
    r.extend(map::told(&at("map")));
    r.extend(share::told(&at("share")));
    r.extend(room::told(&at("room")));
    r.extend(capture::told(&at("capture")));
    for m in ["anchor", "camera", "frame", "session", "config", "surface", "mesh", "depth", "light", "raycast", "geo", "map", "share", "room", "capture"] {
        r.extend(crate::kit::tell::scope_says(&at(m)));
    }
    r
}
