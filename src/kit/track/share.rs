//! Sharing a space: other participants as anchors, the collaboration data
//! sessions exchange and the state of a shared session, and the coaching
//! that guides a person to move the device until tracking is good.

use super::anchor::{Anchor, Transform};
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

/// Another device in the same space.
#[derive(Debug, Clone, PartialEq)]
pub struct Participant {
    pub anchor: Anchor,
}

impl Participant {
    pub fn new(transform: Transform) -> Self {
        Self { anchor: Anchor::new(transform) }
    }
}

named! {
    /// How urgently collaboration data must be delivered.
    pub enum Priority { Critical, Optional }
}

named! { pub enum Role { Host, Participant } }

named! {
    /// Where a shared session stands.
    pub enum SharedState { NotParticipating, Connecting, Participating, Disconnected, Error }
}

/// Bytes one session sends another so both see one space.
#[derive(Debug, Clone, PartialEq)]
pub struct CollaborationData {
    pub priority: Priority,
    pub bytes: Vec<u8>,
}

/// How a shared session is set up.
#[derive(Debug, Clone, PartialEq)]
pub struct Collaboration {
    pub enabled: bool,
    pub role: Role,
    pub local_network_only: bool,
    pub requires_authentication: bool,
    pub max_participants: usize,
    pub state: SharedState,
    pub participants: Vec<Participant>,
}

impl Default for Collaboration {
    fn default() -> Self {
        Self { enabled: false, role: Role::Host, local_network_only: true, requires_authentication: false, max_participants: 8, state: SharedState::NotParticipating, participants: Vec::new() }
    }
}

impl Collaboration {
    /// Admit a participant, if there is room; the state becomes participating.
    pub fn join(&mut self, p: Participant) -> bool {
        if self.participants.len() >= self.max_participants {
            return false;
        }
        self.participants.push(p);
        self.state = SharedState::Participating;
        true
    }
}

named! {
    /// What coaching is asking the person to achieve.
    pub enum Goal { Tracking, HorizontalPlane, VerticalPlane, AnyPlane, GeoTracking }
}

named! {
    /// Where the coaching overlay is.
    pub enum Coaching { Inactive, RequestingCameraAccess, Coaching, Completed }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Priority>(path);
    r.extend(cases::<Role>(path));
    r.extend(cases::<SharedState>(path));
    r.extend(cases::<Goal>(path));
    r.extend(cases::<Coaching>(path));
    r.extend(fields(path, "Participant", &["anchor"]));
    r.extend(fields(path, "CollaborationData", &["priority", "bytes"]));
    r.extend(fields(path, "Collaboration", &["enabled", "role", "local_network_only", "requires_authentication", "max_participants", "state", "participants"]));
    r
}

#[cfg(test)]
mod tests {
    use super::super::anchor::IDENTITY;
    use super::*;

    #[test]
    fn a_shared_session_fills_up() {
        let mut c = Collaboration { max_participants: 1, ..Default::default() };
        assert!(c.join(Participant::new(IDENTITY)));
        assert!(!c.join(Participant::new(IDENTITY)));
        assert_eq!(c.state, SharedState::Participating);
    }
}
