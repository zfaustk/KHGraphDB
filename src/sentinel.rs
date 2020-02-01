//! One watcher. Missed beats promote a replica.
//! No quorum. Split brain is the deal.

use std::path::{Path, PathBuf};

use super::store::Store;

/// Watches a primary directory. On enough unchanged
/// beats, promotes the replica.
pub struct Sentinel {
    primary: PathBuf,
    last: u64,
    miss: u32,
    max_miss: u32,
}

impl Sentinel {
    pub fn new(primary: &Path, max_miss: u32) -> Sentinel {
        let last = Store::beat(primary);
        Sentinel {
            primary: primary.to_path_buf(),
            last: last,
            miss: 0,
            max_miss: max_miss,
        }
    }

    pub fn miss(&self) -> u32 {
        self.miss
    }

    /// Read the beat file. Same value counts as a miss.
    /// After max_miss, promote `replica` and return true.
    pub fn poll(&mut self, replica: &mut Store) -> bool {
        let b = Store::beat(&self.primary);
        if b == self.last {
            self.miss += 1;
        } else {
            self.miss = 0;
            self.last = b;
        }
        if self.miss >= self.max_miss {
            match replica.catch_up(&self.primary) {
                Ok(()) => {
                    replica.promote();
                    true
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }
}
