//! `RelationVocab` on disk: its marker entries, exactly as `.chroma` stores
//! them, so a vocabulary backed by the shared registry reads and writes the
//! same bytes the `HashMap` form did.

#[allow(unused_imports)]
use super::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for RelationVocab {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.entries().serialize(s)
    }
}

impl<'de> Deserialize<'de> for RelationVocab {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Vec::<(String, usize)>::deserialize(d).map(Self::from_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_round_trip_through_disk_and_registry() {
        let v = RelationVocab::from_entries(vec![("of".into(), 8), ("in".into(), 9), ("as".into(), 11)]);
        assert_eq!(v.markers(), 4, "the gap at 10 is held by a placeholder");
        assert_eq!(v.channel_of("as"), Some(11));
        assert_eq!(v.channel_of("offset+3"), None, "an offset is not a marker");
        assert_eq!(v.name(2), "offset+3");
        assert_eq!(v.name(10), "ch10");
        assert_eq!(v.entries(), vec![("of".into(), 8), ("in".into(), 9), ("as".into(), 11)]);
        let json = serde_json::to_string(&v).unwrap();
        let back: RelationVocab = serde_json::from_str(&json).unwrap();
        assert_eq!(back.entries(), v.entries());
        assert_eq!(v.unclaimed(), (12..PHASE_CHANNELS).collect::<Vec<_>>());
        assert!(RelationVocab::default().is_empty());
        assert_eq!(RelationVocab::default().unclaimed().len(), PHASE_CHANNELS - OFFSET_CHANNELS);
    }
}
