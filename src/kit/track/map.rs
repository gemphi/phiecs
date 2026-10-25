//! The world map: what tracking has learned of a space — its anchors and
//! feature points, with a centre and an extent — saved and loaded so a
//! session can pick the space up again, and how far mapping has got.

use super::anchor::{next_id, Anchor, Point};
use super::depth::PointCloud;
use crate::named;
use crate::kit::tell::{cases, fields};
use crate::quest::Row;

named! {
    /// How far the current space has been mapped.
    pub enum Mapping { NotAvailable, Limited, Extending, Mapped }
}

named! {
    /// Why a saved map could not be used.
    pub enum MapError { InvalidData, IncompatibleVersion, SerializationFailed }
}

/// The map of a space.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldMap {
    pub id: u64,
    pub anchors: Vec<Anchor>,
    pub points: PointCloud,
    pub center: Point,
    pub extent: Point,
}

impl Default for WorldMap {
    fn default() -> Self {
        Self { id: next_id(), anchors: Vec::new(), points: PointCloud::default(), center: [0.0; 3], extent: [0.0; 3] }
    }
}

const MAGIC: &[u8; 4] = b"PKWM";

impl WorldMap {
    pub fn add(&mut self, anchor: Anchor) {
        self.anchors.push(anchor);
    }

    /// The map as bytes: a magic, the id, then each anchor's id and position.
    pub fn serialize(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend(self.id.to_le_bytes());
        out.extend((self.anchors.len() as u32).to_le_bytes());
        for a in &self.anchors {
            out.extend(a.id.to_le_bytes());
            for v in a.position() {
                out.extend(v.to_le_bytes());
            }
        }
        out
    }

    /// A map back from its bytes; the anchors come back at their positions, nothing else.
    pub fn deserialize(data: &[u8]) -> Result<Self, MapError> {
        if data.len() < 16 || &data[..4] != MAGIC {
            return Err(MapError::InvalidData);
        }
        let id = u64::from_le_bytes(data[4..12].try_into().map_err(|_| MapError::InvalidData)?);
        let n = u32::from_le_bytes(data[12..16].try_into().map_err(|_| MapError::InvalidData)?) as usize;
        let mut map = WorldMap { id, ..Default::default() };
        let mut at = 16;
        for _ in 0..n {
            let rec = data.get(at..at + 20).ok_or(MapError::InvalidData)?;
            let aid = u64::from_le_bytes(rec[..8].try_into().map_err(|_| MapError::InvalidData)?);
            let f = |i: usize| f32::from_le_bytes(rec[8 + 4 * i..12 + 4 * i].try_into().unwrap_or([0; 4]));
            let mut a = Anchor::default();
            a.id = aid;
            a.transform[3] = [f(0), f(1), f(2), 1.0];
            map.anchors.push(a);
            at += 20;
        }
        Ok(map)
    }
}

/// The module told as rows under `path`.
pub fn told(path: &str) -> Vec<Row> {
    let mut r = cases::<Mapping>(path);
    r.extend(cases::<MapError>(path));
    r.extend(fields(path, "WorldMap", &["id", "anchors", "points", "center", "extent"]));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_survives_its_bytes() {
        let mut m = WorldMap::default();
        let mut a = Anchor::default();
        a.transform[3] = [1.0, 2.0, 3.0, 1.0];
        m.add(a.clone());
        let back = WorldMap::deserialize(&m.serialize()).unwrap();
        assert_eq!(back.id, m.id);
        assert_eq!(back.anchors[0].id, a.id);
        assert_eq!(back.anchors[0].position(), [1.0, 2.0, 3.0]);
        assert_eq!(WorldMap::deserialize(b"nope"), Err(MapError::InvalidData));
    }
}
