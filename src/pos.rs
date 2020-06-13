//! A replica is this far on the log.
//! Same generation: offset is a prefix.
//! A compact bumps generation; offsets
//! from the old generation are void.

/// Position on a shard log.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Pos {
    generation: u32,
    offset: u64,
}

impl Pos {
    pub fn new(generation: u32, offset: u64) -> Pos {
        Pos {
            generation: generation,
            offset: offset,
        }
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Same epoch and not ahead of `other`.
    pub fn prefix_of(&self, other: Pos) -> bool {
        self.generation == other.generation && self.offset <= other.offset
    }

    /// This replica may serve a read that needs `need`.
    /// Older generation cannot. Same generation needs
    /// an offset at least `need`.
    pub fn honors(self, need: Pos) -> bool {
        if self.generation < need.generation {
            false
        } else if self.generation > need.generation {
            true
        } else {
            self.offset >= need.offset
        }
    }
}
