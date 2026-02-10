use super::arena::ArenaStats;
use super::{FastOctTree, Leafable};
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FastOctTreeStats {
    pub arena_stats: ArenaStats,
}
impl<T: Leafable> FastOctTree<T> {
    pub fn stats(&self) -> FastOctTreeStats {
        FastOctTreeStats {
            arena_stats: self.arena.stats(),
        }
    }
}
