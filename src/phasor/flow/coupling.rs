#[allow(unused_imports)]
use super::*;

impl CouplingKind {
    /// The channel this coupling reads, when it has one.
    ///
    /// Adjacency has no channel: a bigram edge is a claim about position, and
    /// position is not one of the sixty-four relations.
    pub fn channel(&self) -> Option<usize> {
        match self {
            CouplingKind::Relation { channel, .. } => Some(*channel),
            _ => None,
        }
    }
}
