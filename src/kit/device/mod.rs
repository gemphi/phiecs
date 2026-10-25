//! The device: the kernel's registry of what the hardware is and which
//! drivers serve it, ported from the IOKit kernel headers (`IORegistryEntry`,
//! `IOService`, `IOKitKeys`) in our words. The registry is a tree of
//! conditions found in a kernel: one entry, a different parent under each
//! plane. A real registry dump is read into these names through
//! [`keys::Key::from_reported`] and [`registry::Registry::rows`].
//!
//! | module | what |
//! |---|---|
//! | [`registry`] | `Registry`, `Entry`, `Plane`, `Value`: the device tree in its planes |
//! | [`service`] | `Services`, `State`, `Personality`: attach, register, match, probe, start, stop |
//! | [`keys`] | `Key`: the property keys, with the strings a registry reports |

pub mod keys;
pub mod registry;
pub mod service;

pub use keys::Key;
pub use registry::{Entry, Plane, Registry, Value};
pub use service::{Services, State};

use crate::quest::Row;

/// The root of the device's tree.
pub const PATH: &str = "device";

/// Every module of the device told as rows under `device/<module>`.
pub fn told() -> Vec<Row> {
    let mut r = registry::told(&format!("{PATH}/registry"));
    r.extend(service::told(&format!("{PATH}/service")));
    r.extend(keys::told(&format!("{PATH}/keys")));
    for m in ["registry", "service", "keys"] {
        r.extend(crate::kit::tell::scope_says(&format!("{PATH}/{m}")));
    }
    r
}
