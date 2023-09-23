//! Optimistic SI. The writeset is private.
//! Commit is first-committer-wins on KHID.

use std::collections::{HashMap, HashSet};
use std::io;
use std::thread;

use super::engine::Engine;
use super::error::{Error, Result};
use super::graph::Graph;
use super::khid::Khid;
use super::lock::{Acquire, Mode, TxId};
use super::pos::Pos;
use super::prop::Prop;
use super::query::{self, QueryResult};
use super::store::Store;

struct VWrite {
    created: bool,
    gone: bool,
    ty: Option<String>,
    attrs: HashMap<String, Prop>,
}

struct EWrite {
    src: Khid,
    dst: Khid,
    ty: String,
    gone: bool,
    id: Option<Khid>,
}

pub struct WriteSet {
    verts: HashMap<Khid, VWrite>,
    edges: Vec<EWrite>,
    keys: HashSet<Khid>,
}

impl WriteSet {
    fn new() -> WriteSet {
        WriteSet {
            verts: HashMap::new(),
            edges: Vec::new(),
            keys: HashSet::new(),
        }
    }

    fn touch(&mut self, k: Khid) {
        self.keys.insert(k);
    }

    fn empty(&self) -> bool {
        self.verts.is_empty() && self.edges.is_empty()
    }

    fn apply_to(&self, g: &mut Graph) -> Result<()> {
        for (&id, w) in self.verts.iter() {
            if w.gone {
                g.remove_vertex(id);
                continue;
            }
            if w.created {
                let _ = g.adopt_vertex(id, w.attrs.clone(), w.ty.as_ref().map(|s| s.as_str()))?;
            } else {
                for (k, v) in w.attrs.iter() {
                    g.set_prop(id, k, v.clone())?;
                }
            }
        }
        for e in self.edges.iter() {
            if e.gone {
                if let Some(id) = e.id {
                    g.remove_edge(id);
                }
            } else {
                g.add_edge(e.src, e.dst, Some(&e.ty))?;
            }
        }
        Ok(())
    }
}

/// A transaction against an Engine. Drop aborts.
pub struct Session<'a> {
    eng: &'a Engine,
    id: TxId,
    start: u64,
    snap: std::sync::Arc<Graph>,
    ws: WriteSet,
    reads: HashSet<Khid>,
    open: bool,
}

impl<'a> Session<'a> {
    pub(crate) fn new(eng: &'a Engine) -> Session<'a> {
        Session {
            eng: eng,
            id: eng.begin_lock(),
            start: eng.ts(),
            snap: eng.snapshot(),
            ws: WriteSet::new(),
            reads: HashSet::new(),
            open: true,
        }
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn id(&self) -> TxId {
        self.id
    }

    fn read(&mut self, k: Khid) {
        self.reads.insert(k);
    }

    /// Own writes first, then the pin.
    pub fn vertex_name(&mut self, k: Khid) -> Option<String> {
        self.read(k);
        if let Some(w) = self.ws.verts.get(&k) {
            if w.gone {
                return None;
            }
            return w.attrs.get("name").and_then(|p| p.as_str()).map(|s| s.to_string());
        }
        self.snap.vertex(k).and_then(|v| v.get("name")).map(|s| s.to_string())
    }

    pub fn find_name(&mut self, ty: &str, name: &str) -> Option<Khid> {
        let hits = self.snap.find(ty, "name", name);
        let id = hits.first().cloned();
        if let Some(k) = id {
            self.read(k);
        }
        if let Some(k) = id {
            if let Some(w) = self.ws.verts.get(&k) {
                if w.gone {
                    return None;
                }
            }
        }
        for (&k, w) in self.ws.verts.iter() {
            if w.gone || w.ty.as_ref().map(|s| s.as_str()) != Some(ty) {
                continue;
            }
            if w.attrs.get("name").and_then(|p| p.as_str()) == Some(name) {
                return Some(k);
            }
        }
        id
    }

    pub fn create(&mut self, name: &str, ty: &str) -> Khid {
        let id = self.eng.alloc();
        let mut attrs = HashMap::new();
        attrs.insert("name".to_string(), Prop::from_str(name));
        self.ws.touch(id);
        self.ws.verts.insert(id, VWrite {
            created: true,
            gone: false,
            ty: Some(ty.to_string()),
            attrs: attrs,
        });
        id
    }

    pub fn set(&mut self, id: Khid, key: &str, val: Prop) -> Result<()> {
        self.ws.touch(id);
        if let Some(w) = self.ws.verts.get_mut(&id) {
            if w.gone {
                return Err(Error::new("missing vertex"));
            }
            w.attrs.insert(key.to_string(), val);
            return Ok(());
        }
        let v = match self.snap.vertex(id) {
            Some(v) => v,
            None => return Err(Error::new("missing vertex")),
        };
        let mut attrs = v.attrs().clone();
        let ty = v.primary_type().and_then(|t| self.snap.type_name_of(t).map(|s| s.to_string()));
        self.read(id);
        attrs.insert(key.to_string(), val);
        self.ws.verts.insert(id, VWrite {
            created: false,
            gone: false,
            ty: ty,
            attrs: attrs,
        });
        Ok(())
    }

    pub fn delete(&mut self, id: Khid) {
        self.read(id);
        self.ws.touch(id);
        self.ws.verts.insert(id, VWrite {
            created: false,
            gone: true,
            ty: None,
            attrs: HashMap::new(),
        });
    }

    pub fn link(&mut self, src: Khid, dst: Khid, ty: &str) {
        self.ws.touch(src);
        self.ws.touch(dst);
        self.ws.edges.push(EWrite {
            src: src,
            dst: dst,
            ty: ty.to_string(),
            gone: false,
            id: None,
        });
    }

    /// Own writes on a copy of the pin.
    pub fn ask(&mut self, text: &str) -> QueryResult {
        if self.ws.empty() {
            return query::ask(&self.snap, text);
        }
        let mut g = (*self.snap).clone();
        match self.ws.apply_to(&mut g) {
            Ok(()) => query::ask(&g, text),
            Err(e) => QueryResult::error(e.message()),
        }
    }

    pub fn commit(mut self) -> io::Result<Pos> {
        self.open = false;
        if self.ws.empty() {
            self.eng.unlock(self.id);
            return self.eng.pos();
        }
        let mut keys: Vec<Khid> = self.ws.keys.iter().cloned().collect();
        keys.sort();
        for k in keys.iter() {
            let mut spins = 0u32;
            loop {
                match self.eng.lock(self.id, *k, Mode::X) {
                    Acquire::Ok => break,
                    Acquire::Deadlock => {
                        self.eng.unlock(self.id);
                        return Err(io::Error::new(io::ErrorKind::Other, "conflict"));
                    }
                    Acquire::Wait => {
                        spins += 1;
                        if spins > 128 {
                            self.eng.unlock(self.id);
                            return Err(io::Error::new(io::ErrorKind::Other, "conflict"));
                        }
                        thread::yield_now();
                    }
                }
            }
        }
        let pos = {
            let mut st = self.eng.store_lock();
            if self.eng.conflicts(self.start, &self.ws.keys) {
                drop(st);
                self.eng.unlock(self.id);
                return Err(io::Error::new(io::ErrorKind::Other, "conflict"));
            }
            apply_store(&mut st, &self.ws).map_err(|e| {
                io::Error::new(io::ErrorKind::Other, e.message())
            })?;
            let pos = st.commit()?;
            let g = st.graph().clone();
            self.eng.publish(g, self.start, &self.ws.keys);
            pos
        };
        self.eng.unlock(self.id);
        Ok(pos)
    }

    pub fn abort(mut self) {
        self.open = false;
        self.eng.unlock(self.id);
    }
}

impl<'a> Drop for Session<'a> {
    fn drop(&mut self) {
        if self.open {
            self.eng.unlock(self.id);
            self.open = false;
        }
    }
}

fn apply_store(st: &mut Store, ws: &WriteSet) -> Result<()> {
    let g = match st.graph_mut() {
        Ok(g) => g,
        Err(e) => return Err(Error::new(&e.to_string())),
    };
    ws.apply_to(g)
}
