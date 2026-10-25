//! A session: tracking run under a configuration, frame after frame. It
//! holds the anchors, takes each new frame, and tells what happened —
//! anchors added, changed or gone, a frame, a failure, an interruption —
//! as events a persona reads in order.

use super::anchor::{next_id, Transform, IDENTITY};
use super::config::Configuration;
use super::frame::{Frame, Tracked};
use super::map::{Mapping, WorldMap};
use super::share::CollaborationData;
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;
use std::collections::VecDeque;

named! {
    /// Why a session failed.
    pub enum SessionError { CameraAccessDenied, UnsupportedConfiguration, SensorFailed, WorldTrackingFailed }
}

named! {
    /// What a session tells.
    pub enum Happening { FrameUpdated, AnchorsAdded, AnchorsUpdated, AnchorsRemoved, Failed, Interrupted, InterruptionEnded, CollaborationData }
}

/// One thing that happened, with what it concerned.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Frame(Frame),
    Added(Vec<Tracked>),
    Updated(Vec<Tracked>),
    Removed(Vec<u64>),
    Failed(SessionError),
    Interrupted,
    InterruptionEnded,
    Collaboration(CollaborationData),
}

impl Event {
    pub fn happening(&self) -> Happening {
        match self {
            Event::Frame(_) => Happening::FrameUpdated,
            Event::Added(_) => Happening::AnchorsAdded,
            Event::Updated(_) => Happening::AnchorsUpdated,
            Event::Removed(_) => Happening::AnchorsRemoved,
            Event::Failed(_) => Happening::Failed,
            Event::Interrupted => Happening::Interrupted,
            Event::InterruptionEnded => Happening::InterruptionEnded,
            Event::Collaboration(_) => Happening::CollaborationData,
        }
    }
}

/// How to start again.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RunOptions {
    pub reset_tracking: bool,
    pub remove_anchors: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Session {
    pub id: u64,
    pub configuration: Option<Configuration>,
    pub running: bool,
    pub frame: Option<Frame>,
    pub anchors: Vec<Tracked>,
    pub origin: Transform,
    pub events: VecDeque<Event>,
}

impl Session {
    pub fn new() -> Self {
        Self { id: next_id(), origin: IDENTITY, ..Default::default() }
    }

    pub fn run(&mut self, configuration: Configuration) {
        self.run_with(configuration, RunOptions::default());
    }

    pub fn run_with(&mut self, configuration: Configuration, options: RunOptions) {
        if options.remove_anchors && !self.anchors.is_empty() {
            let ids = self.anchors.drain(..).map(|a| a.id()).collect();
            self.events.push_back(Event::Removed(ids));
        }
        if options.reset_tracking {
            self.frame = None;
        }
        self.configuration = Some(configuration);
        self.running = true;
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    pub fn add(&mut self, anchor: Tracked) {
        self.events.push_back(Event::Added(vec![anchor.clone()]));
        self.anchors.push(anchor);
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let n = self.anchors.len();
        self.anchors.retain(|a| a.id() != id);
        let gone = self.anchors.len() < n;
        if gone {
            self.events.push_back(Event::Removed(vec![id]));
        }
        gone
    }

    /// Take a frame: anchors it carries that are new are added, known ones
    /// updated, and the frame itself is told.
    pub fn update(&mut self, frame: Frame) {
        if !self.running {
            return;
        }
        let (mut added, mut updated) = (Vec::new(), Vec::new());
        for t in &frame.anchors {
            match self.anchors.iter_mut().find(|a| a.id() == t.id()) {
                Some(a) => {
                    *a = t.clone();
                    updated.push(t.clone());
                }
                None => {
                    self.anchors.push(t.clone());
                    added.push(t.clone());
                }
            }
        }
        if !added.is_empty() {
            self.events.push_back(Event::Added(added));
        }
        if !updated.is_empty() {
            self.events.push_back(Event::Updated(updated));
        }
        self.events.push_back(Event::Frame(frame.clone()));
        self.frame = Some(frame);
    }

    pub fn fail(&mut self, error: SessionError) {
        self.running = false;
        self.events.push_back(Event::Failed(error));
    }

    pub fn interrupt(&mut self, ended: bool) {
        self.events.push_back(if ended { Event::InterruptionEnded } else { Event::Interrupted });
    }

    pub fn mapping(&self) -> Mapping {
        self.frame.as_ref().map_or(Mapping::NotAvailable, |f| f.mapping)
    }

    /// The space as mapped so far: every anchor's plain anchor.
    pub fn world_map(&self) -> WorldMap {
        let mut map = WorldMap::default();
        for a in &self.anchors {
            map.add(a.anchor().clone());
        }
        map
    }

    /// Take the events in order.
    pub fn drain(&mut self) -> Vec<Event> {
        self.events.drain(..).collect()
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<SessionError>(path);
    r.extend(cases::<Happening>(path));
    r.extend(fields(path, "RunOptions", &["reset_tracking", "remove_anchors"]));
    r.extend(fields(path, "Session", &["id", "configuration", "running", "frame", "anchors", "origin", "events"]));
    r
}

#[cfg(test)]
mod tests {
    use super::super::anchor::Anchor;
    use super::*;

    #[test]
    fn a_session_tells_what_happened_in_order() {
        let mut s = Session::new();
        s.run(Configuration::default());
        let a = Tracked::Plain(Anchor::default());
        let mut f = Frame::default();
        f.anchors.push(a.clone());
        s.update(f.clone());
        s.update(f);
        assert!(s.remove(a.id()));
        s.fail(SessionError::SensorFailed);
        let got: Vec<Happening> = s.drain().iter().map(Event::happening).collect();
        assert_eq!(got, [Happening::AnchorsAdded, Happening::FrameUpdated, Happening::AnchorsUpdated, Happening::FrameUpdated, Happening::AnchorsRemoved, Happening::Failed]);
        assert!(!s.running);
    }
}
