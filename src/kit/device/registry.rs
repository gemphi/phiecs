//! The registry: the device tree. Every piece of hardware and every driver
//! attached to it is an entry with a name, a class and a property table,
//! and the entries hang from each other in planes — the service plane (who
//! provides for whom), the device tree (how the hardware is wired), power,
//! audio, FireWire, USB. A plane is a condition, and the same entry has a
//! different parent under each: X ⇒ Y under c, which is the paradigm's
//! shape, found in a kernel.

use super::keys::Key;
use crate::named;
use crate::kit::tell::{cases, fields, row, Named};
use crate::kit::words::snake;
use crate::quest::Row;
use std::collections::HashMap;

named! {
    /// A plane of the registry.
    pub enum Plane { Service, Power, DeviceTree, Audio, FireWire, Usb }
}

/// A value in a property table.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(i64),
    Flag(bool),
    Bytes(Vec<u8>),
    List(Vec<Value>),
}

/// An entry of the registry.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: u64,
    pub name: String,
    /// The class of the object, as `Key::Class` reports it.
    pub class: String,
    pub properties: Vec<(String, Value)>,
}

impl Entry {
    pub fn new(name: impl Into<String>, class: impl Into<String>) -> Self {
        Self { id: crate::kit::track::anchor::next_id(), name: name.into(), class: class.into(), properties: Vec::new() }
    }

    pub fn with(mut self, key: Key, value: Value) -> Self {
        self.properties.push((key.reported(), value));
        self
    }

    pub fn property(&self, key: Key) -> Option<&Value> {
        let k = key.reported();
        self.properties.iter().find(|(n, _)| *n == k).map(|(_, v)| v)
    }
}

/// The registry: entries and, per plane, who hangs from whom.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    pub entries: Vec<Entry>,
    /// `(parent, plane, child)`.
    pub edges: Vec<(u64, Plane, u64)>,
}

impl Registry {
    pub fn add(&mut self, entry: Entry) -> u64 {
        let id = entry.id;
        self.entries.push(entry);
        id
    }

    /// Attach `child` to `parent` in `plane`.
    pub fn attach(&mut self, child: u64, plane: Plane, parent: u64) {
        if !self.edges.contains(&(parent, plane, child)) {
            self.edges.push((parent, plane, child));
        }
    }

    pub fn entry(&self, id: u64) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == id)
    }

    pub fn parents(&self, id: u64, plane: Plane) -> Vec<u64> {
        self.edges.iter().filter(|(_, p, c)| *p == plane && *c == id).map(|(a, _, _)| *a).collect()
    }

    pub fn children(&self, id: u64, plane: Plane) -> Vec<u64> {
        self.edges.iter().filter(|(a, p, _)| *p == plane && *a == id).map(|(_, _, c)| *c).collect()
    }

    /// The path of names from the plane's root down to `id`, as the registry prints it.
    pub fn path(&self, id: u64, plane: Plane) -> String {
        let mut names = Vec::new();
        let mut cur = Some(id);
        while let Some(c) = cur {
            let Some(e) = self.entry(c) else { break };
            names.push(e.name.clone());
            cur = self.parents(c, plane).first().copied();
            if names.len() > 64 {
                break;
            }
        }
        names.reverse();
        format!("{}:/{}", plane.ident(), names.join("/"))
    }

    /// The roots of a plane: entries with no parent there but a child.
    pub fn roots(&self, plane: Plane) -> Vec<u64> {
        let mut v: Vec<u64> = self.entries.iter().map(|e| e.id).filter(|id| self.parents(*id, plane).is_empty() && !self.children(*id, plane).is_empty()).collect();
        v.dedup();
        v
    }

    /// The entries whose class is `class`.
    pub fn of_class<'a>(&'a self, class: &'a str) -> impl Iterator<Item = &'a Entry> + 'a {
        self.entries.iter().filter(move |e| e.class == class)
    }

    /// The registry as rows: each entry a thing `within` its device-tree
    /// path and `is-a` its class, `has` each property key, and
    /// `provides`/`child-of` in the other planes; the planes as cases.
    pub fn rows(&self, path: &str) -> Vec<Row> {
        let mut r = Vec::new();
        let mut seen: HashMap<u64, String> = HashMap::new();
        for e in &self.entries {
            let name = format!("entry:{}", snake(&e.name));
            seen.insert(e.id, name.clone());
            let full = format!("{path}/{}/{}", Plane::DeviceTree.words().join(" "), self.path(e.id, Plane::DeviceTree).trim_start_matches("DeviceTree:/"));
            let scopes = crate::scope::scopes(&full, 0);
            for w in scopes.windows(2) {
                r.push(row(&w[1], "within", &w[0]));
            }
            for s in &scopes {
                r.push(row(&name, "within", s));
            }
            r.push(row(&name, "is-a", &snake(&e.class)));
            for w in crate::kit::words::split(&e.name) {
                r.push(row(&name, "has-word", &w));
                r.push(row(&name, crate::scope::SAYS, &w));
            }
            for (k, _) in &e.properties {
                if let Some(key) = Key::from_reported(k) {
                    r.push(row(&name, "has", &key.name()));
                }
            }
        }
        for (a, plane, c) in &self.edges {
            if let (Some(pa), Some(ch)) = (seen.get(a), seen.get(c)) {
                r.push(row(ch, &format!("child-of:{}", snake(plane.ident())), pa));
            }
        }
        r
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Plane>(path);
    r.extend(fields(path, "Entry", &["id", "name", "class", "properties"]));
    r.extend(fields(path, "Registry", &["entries", "edges"]));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_hang_differently_in_different_planes() {
        let mut reg = Registry::default();
        let root = reg.add(Entry::new("device-tree", "IOPlatformExpertDevice"));
        let usb = reg.add(Entry::new("usb-drd0", "AppleT8103USBXHCI"));
        let cam = reg.add(Entry::new("camera", "AppleH13CamIn").with(Key::ProviderClass, Value::Text("IOService".into())));
        reg.attach(usb, Plane::DeviceTree, root);
        reg.attach(cam, Plane::DeviceTree, root);
        reg.attach(cam, Plane::Service, usb);
        assert_eq!(reg.path(cam, Plane::DeviceTree), "DeviceTree:/device-tree/camera");
        assert_eq!(reg.path(cam, Plane::Service), "Service:/usb-drd0/camera");
        assert_eq!(reg.roots(Plane::DeviceTree), [root]);
        assert_eq!(reg.children(root, Plane::DeviceTree).len(), 2);
        assert!(reg.entry(cam).unwrap().property(Key::ProviderClass).is_some());
        let rows = reg.rows("device/registry");
        assert!(rows.iter().any(|r| r.about == "entry:camera" && r.kind == "has" && r.other == "key:provider_class"));
        assert!(rows.iter().any(|r| r.about == "entry:camera" && r.kind == "child-of:service" && r.other == "entry:usb_drd0"));
        assert!(rows.iter().any(|r| r.about == "entry:camera" && r.kind == "within" && r.other == "scope:device/registry/device tree/device-tree/camera"));
    }
}
