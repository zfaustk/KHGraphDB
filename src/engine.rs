//! Process facade. Readers share an Arc snapshot.
//! One writer holds the store. Locks are the table
//! for KHID. Isolation is the snapshot, not a
//! version chain. Clone at commit is the picture
//! we dropped in 2022 for a notebook; here it is
//! the price of concurrent ask.

use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

use super::graph::Graph;
use super::khid::Khid;
use super::lock::{Acquire, LockMgr, Mode, TxId};
use super::pos::Pos;
use super::query::{self, QueryResult};
use super::store::Store;

pub struct Engine {
    store: Mutex<Store>,
    snap: RwLock<Arc<Graph>>,
    locks: Mutex<LockMgr>,
}

impl Engine {
    pub fn open(dir: &Path, name: &str, shard: u32) -> io::Result<Engine> {
        let s = Store::open(dir, name, shard)?;
        let g = Arc::new(s.graph().clone());
        Ok(Engine {
            store: Mutex::new(s),
            snap: RwLock::new(g),
            locks: Mutex::new(LockMgr::new()),
        })
    }

    /// Last committed picture. Cheap to clone the Arc.
    pub fn snapshot(&self) -> Arc<Graph> {
        self.snap.read().unwrap().clone()
    }

    pub fn ask(&self, text: &str) -> QueryResult {
        let g = self.snapshot();
        query::ask(&g, text)
    }

    /// One writer. `f` mutates the store. Commit
    /// publishes a new picture. Readers never hold
    /// this mutex.
    pub fn apply<F>(&self, f: F) -> io::Result<Pos>
        where F: FnOnce(&mut Store) -> io::Result<()>
    {
        let mut st = self.store.lock().unwrap();
        f(&mut st)?;
        let pos = st.commit()?;
        let g = Arc::new(st.graph().clone());
        *self.snap.write().unwrap() = g;
        Ok(pos)
    }

    pub fn pos(&self) -> io::Result<Pos> {
        self.store.lock().unwrap().pos()
    }

    pub fn begin_lock(&self) -> TxId {
        self.locks.lock().unwrap().begin()
    }

    pub fn lock(&self, tx: TxId, k: Khid, m: Mode) -> Acquire {
        self.locks.lock().unwrap().acquire(tx, k, m)
    }

    pub fn unlock(&self, tx: TxId) -> Vec<TxId> {
        self.locks.lock().unwrap().release(tx)
    }

    pub fn graph_count(&self) -> usize {
        self.snapshot().vertex_count()
    }
}
