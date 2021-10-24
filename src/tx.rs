//! In-memory transaction. Inverse of each touch.
//! Writes hit the live graph. Drop walks undos
//! unless commit disarmed them.

use super::graph::Graph;

/// A write that can still go back. The inverse
/// is on the graph. No second arena. Drop is rollback.
pub struct Tx<'a> {
    live: &'a mut Graph,
    open: bool,
}

impl<'a> Tx<'a> {
    pub fn begin(g: &'a mut Graph) -> Tx<'a> {
        g.arm();
        Tx {
            live: g,
            open: true,
        }
    }

    pub fn graph(&mut self) -> &mut Graph {
        self.live
    }

    pub fn commit(mut self) {
        self.live.disarm();
        self.open = false;
    }

    pub fn rollback(&mut self) {
        self.live.apply_undos();
        self.live.arm();
    }
}

impl<'a> Drop for Tx<'a> {
    fn drop(&mut self) {
        if self.open {
            self.live.apply_undos();
            self.live.disarm();
            self.open = false;
        }
    }
}
