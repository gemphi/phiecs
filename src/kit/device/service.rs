//! A service: an entry that does something — a driver. It is attached to
//! a provider, registered, matched against the catalogue's personalities
//! by their matching keys, probed (the best score wins), started, and
//! stopped. The states are bits an entry carries; a client is a service
//! whose provider is this one.

use super::keys::Key;
use super::registry::{Entry, Plane, Registry, Value};
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

named! {
    /// The states of a service, each a bit.
    pub enum State { Inactive, Registered, Matched, FirstPublish, FirstMatch }
}

impl State {
    pub fn bit(self) -> u32 {
        1 << (self as u32)
    }
}

/// A personality: a dictionary a driver is loaded by when it matches.
#[derive(Debug, Clone, PartialEq)]
pub struct Personality {
    pub class: String,
    pub provider_class: String,
    pub name_match: Vec<String>,
    pub probe_score: i64,
}

impl Personality {
    /// Whether the personality matches `provider`: its class is the
    /// provider's, and if names are given one is the provider's name.
    pub fn matches(&self, provider: &Entry) -> bool {
        self.provider_class == provider.class && (self.name_match.is_empty() || self.name_match.iter().any(|n| *n == provider.name))
    }

    pub fn entry(&self) -> Entry {
        Entry::new(self.class.clone(), self.class.clone())
            .with(Key::Class, Value::Text(self.class.clone()))
            .with(Key::ProviderClass, Value::Text(self.provider_class.clone()))
            .with(Key::ProbeScore, Value::Number(self.probe_score))
    }
}

/// The states of every service, and what started under what.
#[derive(Debug, Clone, Default)]
pub struct Services {
    pub states: Vec<(u64, u32)>,
    pub catalogue: Vec<Personality>,
}

impl Services {
    pub fn state(&self, id: u64) -> u32 {
        self.states.iter().find(|(i, _)| *i == id).map_or(0, |(_, s)| *s)
    }

    pub fn is(&self, id: u64, state: State) -> bool {
        self.state(id) & state.bit() != 0
    }

    fn set(&mut self, id: u64, state: State) {
        match self.states.iter_mut().find(|(i, _)| *i == id) {
            Some((_, s)) => *s |= state.bit(),
            None => self.states.push((id, state.bit())),
        }
    }

    /// Publish `id`: it is registered, and matching begins — every
    /// personality that matches is probed, the best score starts as a
    /// client in the service plane. Returns what started.
    pub fn register(&mut self, registry: &mut Registry, id: u64) -> Vec<u64> {
        self.set(id, State::Registered);
        self.set(id, State::FirstPublish);
        let Some(provider) = registry.entry(id).cloned() else { return Vec::new() };
        let mut matched: Vec<&Personality> = self.catalogue.iter().filter(|p| p.matches(&provider)).collect();
        matched.sort_by(|a, b| b.probe_score.cmp(&a.probe_score));
        let mut started = Vec::new();
        if let Some(best) = matched.first() {
            let client = registry.add(best.entry());
            registry.attach(client, Plane::Service, id);
            started.push(client);
            self.set(id, State::Matched);
            self.set(id, State::FirstMatch);
        }
        started
    }

    /// Stop `id`: inactive, and its clients in the service plane with it.
    pub fn terminate(&mut self, registry: &Registry, id: u64) {
        self.set(id, State::Inactive);
        for c in registry.children(id, Plane::Service) {
            self.terminate(registry, c);
        }
    }

    pub fn clients(&self, registry: &Registry, id: u64) -> Vec<u64> {
        registry.children(id, Plane::Service)
    }

    pub fn provider(&self, registry: &Registry, id: u64) -> Option<u64> {
        registry.parents(id, Plane::Service).first().copied()
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<State>(path);
    r.extend(fields(path, "Personality", &["class", "provider_class", "name_match", "probe_score"]));
    r.extend(fields(path, "Services", &["states", "catalogue"]));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_best_personality_starts_as_a_client() {
        let mut reg = Registry::default();
        let cam = reg.add(Entry::new("camera", "CameraDevice"));
        let mut s = Services::default();
        s.catalogue.push(Personality { class: "CameraDriver".into(), provider_class: "CameraDevice".into(), name_match: vec![], probe_score: 1000 });
        s.catalogue.push(Personality { class: "GenericCameraDriver".into(), provider_class: "CameraDevice".into(), name_match: vec![], probe_score: 100 });
        s.catalogue.push(Personality { class: "DiskDriver".into(), provider_class: "BlockStorageDevice".into(), name_match: vec![], probe_score: 5000 });
        let started = s.register(&mut reg, cam);
        assert_eq!(started.len(), 1);
        assert_eq!(reg.entry(started[0]).unwrap().class, "CameraDriver");
        assert!(s.is(cam, State::Matched) && s.is(cam, State::Registered) && !s.is(cam, State::Inactive));
        assert_eq!(s.provider(&reg, started[0]), Some(cam));
        s.terminate(&reg, cam);
        assert!(s.is(started[0], State::Inactive));
    }
}
