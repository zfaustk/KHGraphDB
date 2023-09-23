//! Process facade. Readers share an Arc snapshot.
//! A session writes a set. Commit is SI, first
//! committer wins on a KHID. The clone is only
//! for a writer who asks. See `docs/occ.md`.

use std::collections::HashSet;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, RwLock};

use super::graph::{Graph, Touch};
use super::khid::Khid;
use super::lock::{Acquire, LockMgr, Mode, TxId};
use super::pos::Pos;
use super::query::{self, QueryResult};
use super::session::Session;
use super::store::Store;

struct Committed {
    ts: u64,
    keys: HashSet<Khid>,
}

pub struct Engine {
    store: Mutex<Store>,
    snap: RwLock<Arc<Graph>>,
    locks: Mutex<LockMgr>,
    ts: Mutex<u64>,
    ids: Mutex<u64>,
    recent: Mutex<Vec<Committed>>,
}

impl Engine {
    pub fn open(dir: &Path, name: &str, shard: u32) -> io::Result<Engine> {
        let s = Store::open(dir, name, shard)?;
        let serial = s.graph().serial();
        let g = Arc::new(s.graph().clone());
        Ok(Engine {
            store: Mutex::new(s),
            snap: RwLock::new(g),
            locks: Mutex::new(LockMgr::new()),
            ts: Mutex::new(0),
            ids: Mutex::new(serial),
            recent: Mutex::new(Vec::new()),
        })
    }

    pub fn snapshot(&self) -> Arc<Graph> {
        self.snap.read().unwrap().clone()
    }

    pub fn ask(&self, text: &str) -> QueryResult {
        let g = self.snapshot();
        query::ask(&g, text)
    }

    pub fn session(&self) -> Session {
        Session::new(self)
    }

    pub fn apply<F>(&self, f: F) -> io::Result<Pos>
        where F: FnOnce(&mut Store) -> io::Result<()>
    {
        let mut st = self.store.lock().unwrap();
        f(&mut st)?;
        let keys = keys_of(st.graph());
        let g = st.graph().clone();
        let pos = st.commit()?;
        self.publish(g, 0, &keys);
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

    pub(crate) fn ts(&self) -> u64 {
        *self.ts.lock().unwrap()
    }

    pub(crate) fn alloc(&self) -> Khid {
        let mut n = self.ids.lock().unwrap();
        *n += 1;
        Khid::from_raw(*n)
    }

    pub(crate) fn store_lock(&self) -> MutexGuard<Store> {
        self.store.lock().unwrap()
    }

    pub(crate) fn conflicts(&self, start: u64, keys: &HashSet<Khid>) -> bool {
        let recent = self.recent.lock().unwrap();
        for c in recent.iter() {
            if c.ts > start {
                for k in keys.iter() {
                    if c.keys.contains(k) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub(crate) fn publish(&self, g: Graph, _start: u64, keys: &HashSet<Khid>) {
        let mut ts = self.ts.lock().unwrap();
        *ts += 1;
        let now = *ts;
        drop(ts);
        {
            let mut recent = self.recent.lock().unwrap();
            recent.push(Committed {
                ts: now,
                keys: keys.clone(),
            });
            if recent.len() > 1024 {
                let n = recent.len() - 1024;
                recent.drain(0..n);
            }
        }
        {
            let ser = g.serial();
            let mut ids = self.ids.lock().unwrap();
            if ser > *ids {
                *ids = ser;
            }
        }
        *self.snap.write().unwrap() = Arc::new(g);
    }
}

fn keys_of(g: &Graph) -> HashSet<Khid> {
    let mut s = HashSet::new();
    for t in g.touches() {
        match *t {
            Touch::Vertex(id) | Touch::DropVertex(id) | Touch::DropEdge(id) | Touch::Edge(id) => {
                s.insert(id);
            }
            Touch::Emb { id, .. } => {
                s.insert(id);
            }
            Touch::Index { .. } | Touch::Content { .. } | Touch::VecMark { .. } => {}
        }
    }
    s
}
