//! In-memory transaction. Clone the arena. Writes hit
//! the live graph. Drop puts the clone back unless
//! commit dropped the snapshot.

use super::graph::Graph;

/// A write that can still go back. The snapshot is a
/// Graph clone. No Rc. Drop is rollback.
pub struct Tx<'a> {
    live: &'a mut Graph,
    snap: Option<Graph>,
}

impl<'a> Tx<'a> {
    pub fn begin(g: &'a mut Graph) -> Tx<'a> {
        Tx {
            snap: Some(g.snapshot()),
            live: g,
        }
    }

    pub fn graph(&mut self) -> &mut Graph {
        self.live
    }

    pub fn commit(mut self) {
        self.snap = None;
    }

    pub fn rollback(&mut self) {
        if let Some(ref s) = self.snap {
            *self.live = s.clone();
        }
    }
}

impl<'a> Drop for Tx<'a> {
    fn drop(&mut self) {
        if let Some(s) = self.snap.take() {
            *self.live = s;
        }
    }
}
