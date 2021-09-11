//! A shard on disk. The log is truth. Commit
//! appends this tx, sync_data. Empty pending
//! falls back to a capture. compact bumps
//! generation. Drop without commit keeps
//! the last snapshot.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::error::Error;
use super::graph::Graph;
use super::wal::{self, Rec};
use super::khid::Khid;
use super::addr::Addr;
use super::edge::Edge;
use super::pos::Pos;
use super::prop::Prop;

/// Primary writes. Replica tails until promote.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Role {
    Primary,
    Replica,
}

/// Durable home of one shard.
pub struct Store {
    dir: PathBuf,
    log: File,
    g: Graph,
    next_tx: u64,
    open_tx: Option<u64>,
    snap: Option<Graph>,
    read_only: bool,
    generation: u32,
    pending: Vec<Rec>,
    token: u64,
}

impl Store {
    pub fn open(dir: &Path, name: &str, shard: u32) -> io::Result<Store> {
        fs::create_dir_all(dir)?;
        let path = dir.join("log");
        let mut log = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;
        let len = log.metadata()?.len();
        let (g, next_tx, generation) = if len == 0 {
            wal::write_header(shard, 1, &mut log)?;
            log.sync_data()?;
            (Graph::on(name, shard), 1, 1)
        } else {
            log.seek(SeekFrom::Start(0))?;
            let (h, recs) = wal::read_at(&mut log)?;
            let mut g = match wal::replay(h.shard, &recs) {
                Ok(g) => g,
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e.message())),
            };
            g.set_id(name);
            let mut max = 0u64;
            for rec in recs.iter() {
                if rec.tx() > max {
                    max = rec.tx();
                }
            }
            log.seek(SeekFrom::End(0))?;
            let gen = if h.generation == 0 { 1 } else { h.generation };
            (g, max + 1, gen)
        };
        let s = Store {
            dir: dir.to_path_buf(),
            log: log,
            g: g,
            next_tx: next_tx,
            open_tx: None,
            snap: None,
            read_only: false,
            generation: generation,
            pending: Vec::new(),
            token: new_token(),
        };
        Ok(s)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn graph(&self) -> &Graph {
        &self.g
    }

    pub fn arena_mut(&mut self) -> &mut Graph {
        &mut self.g
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn pos(&self) -> io::Result<Pos> {
        Ok(Pos::new(self.generation, self.log.metadata()?.len()))
    }

    /// The arena, if this store may write. A replica
    /// is refused here, not at commit.
    pub fn graph_mut(&mut self) -> io::Result<&mut Graph> {
        if self.read_only {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "read-only replica"));
        }
        if !self.hold_lease() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "no lease"));
        }
        if self.open_tx.is_none() {
            self.begin()?;
        }
        Ok(&mut self.g)
    }

    pub fn begin(&mut self) -> io::Result<()> {
        if self.read_only {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "read-only replica"));
        }
        if !self.hold_lease() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "no lease"));
        }
        if self.open_tx.is_some() {
            return Ok(());
        }
        self.snap = Some(self.g.snapshot());
        self.open_tx = Some(self.next_tx);
        self.next_tx += 1;
        self.pending.clear();
        Ok(())
    }

    fn tx_id(&mut self) -> io::Result<u64> {
        match self.open_tx {
            Some(t) => Ok(t),
            None => {
                self.begin()?;
                Ok(self.open_tx.unwrap())
            }
        }
    }

    /// A put that goes on the log. Prefer this over
    /// graph_mut when the write should tail.
    pub fn put_vertex(&mut self,
                      attrs: HashMap<String, Prop>,
                      ty: Option<&str>)
                      -> io::Result<Khid> {
        let tx = self.tx_id()?;
        let id = match self.g.add_vertex_props(attrs.clone(), ty) {
            Ok(id) => id,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.message())),
        };
        let types = self.g.type_names_of_vertex(id);
        self.pending.push(Rec::Vertex {
            tx: tx,
            id: id,
            types: types,
            attrs: attrs,
        });
        Ok(id)
    }

    pub fn put_far(&mut self,
                   src: Khid,
                   dst: Addr,
                   ty: Option<&str>)
                   -> io::Result<Khid> {
        let tx = self.tx_id()?;
        let id = match self.g.add_far_edge(src, dst, ty) {
            Ok(id) => id,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.message())),
        };
        let tname = match ty {
            Some(s) => s.to_string(),
            None => String::new(),
        };
        self.pending.push(Rec::FarEdge {
            tx: tx,
            id: id,
            src: src,
            dst: dst,
            ty: tname,
            attrs: HashMap::new(),
        });
        Ok(id)
    }

    pub fn put_content(&mut self, type_name: &str, key: &str) -> io::Result<()> {
        let tx = self.tx_id()?;
        self.g.mark_content(type_name, key);
        self.pending.push(Rec::Content {
            tx: tx,
            type_name: type_name.to_string(),
            key: key.to_string(),
        });
        Ok(())
    }

    pub fn put_index(&mut self, type_name: &str, key: &str) -> io::Result<()> {
        let tx = self.tx_id()?;
        self.g.create_index(type_name, key);
        self.pending.push(Rec::Index {
            tx: tx,
            type_name: type_name.to_string(),
            key: key.to_string(),
            unique: false,
        });
        Ok(())
    }

    /// Append this tx. Pending puts go as they are.
    /// graph_mut with empty pending writes the
    /// delta against the snapshot, not the arena.
    pub fn commit(&mut self) -> io::Result<Pos> {
        if self.read_only {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "read-only replica"));
        }
        let tx = self.tx_id()?;
        let recs = if !self.pending.is_empty() {
            let mut recs = Vec::new();
            recs.push(Rec::Begin { tx: tx });
            recs.append(&mut self.pending);
            recs.push(Rec::Commit { tx: tx });
            recs
        } else if let Some(ref snap) = self.snap {
            capture_delta(tx, snap, &self.g)
        } else {
            capture(tx, &self.g)
        };
        wal::append(&recs, &mut self.log)?;
        self.log.sync_data()?;
        let _ = sync_dir(&self.dir);
        self.write_beat(tx)?;
        let _ = self.take_lease();
        self.open_tx = None;
        self.snap = None;
        self.pending.clear();
        let _ = super::meta::Meta::rebuild(&self.dir, &self.g);
        self.pos()
    }

    pub fn rollback(&mut self) {
        self.open_tx = None;
        self.pending.clear();
        self.snap = None;
        let _ = self.replay_self();
    }

    fn replay_self(&mut self) -> io::Result<()> {
        let ro = self.read_only;
        let token = self.token;
        let dir = self.dir.clone();
        let name = self.g.khid().to_string();
        let shard = self.g.shard();
        let mut s = Store::open(&dir, &name, shard)?;
        s.read_only = ro;
        s.token = token;
        *self = s;
        Ok(())
    }

    /// Graph as of `at`. Same generation only.
    pub fn read_at(&self, at: Pos) -> io::Result<Graph> {
        if at.generation() != self.generation {
            return Err(io::Error::new(io::ErrorKind::Other, "old generation"));
        }
        let mut f = File::open(self.dir.join("log"))?;
        let (h, recs) = wal::read_prefix(&mut f, at.offset())?;
        let mut g = match wal::replay(h.shard, &recs) {
            Ok(g) => g,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e.message())),
        };
        g.set_id(self.g.khid());
        Ok(g)
    }

    pub fn in_tx(&self) -> bool {
        self.open_tx.is_some()
    }

    pub fn name(&self) -> &str {
        self.g.khid()
    }

    /// Rewrite the log as one capture. Bumps generation.
    /// Offsets from the old generation are void.
    pub fn compact(&mut self) -> io::Result<Pos> {
        if self.read_only {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "read-only replica"));
        }
        if self.open_tx.is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "in a transaction"));
        }
        let shard = self.g.shard();
        self.generation += 1;
        let tx = self.next_tx;
        self.next_tx += 1;
        let recs = capture(tx, &self.g);
        let tmp = self.dir.join("log.tmp");
        {
            let mut f = File::create(&tmp)?;
            wal::write_at(shard, self.generation, &recs, &mut f)?;
            f.sync_data()?;
        }
        let path = self.dir.join("log");
        fs::rename(&tmp, &path)?;
        let mut log = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)?;
        log.seek(SeekFrom::End(0))?;
        self.log = log;
        let _ = super::meta::Meta::rebuild(&self.dir, &self.g);
        self.pos()
    }

    fn write_beat(&self, tx: u64) -> io::Result<()> {
        let mut f = File::create(self.dir.join("beat"))?;
        write!(f, "{}", tx)?;
        f.sync_data()
    }

    /// Last committed tx on this directory. Missing beat is 0.
    pub fn beat(dir: &Path) -> u64 {
        match fs::read_to_string(dir.join("beat")) {
            Ok(s) => s.trim().parse().unwrap_or(0),
            Err(_) => 0,
        }
    }

    pub fn is_replica(&self) -> bool {
        self.read_only
    }

    pub fn role(&self) -> Role {
        if self.read_only {
            Role::Replica
        } else {
            Role::Primary
        }
    }

    /// A copy of the log. Read-only until promote.
    pub fn tail(dir: &Path, from: &Path, name: &str) -> io::Result<Store> {
        fs::create_dir_all(dir)?;
        fs::copy(from.join("log"), dir.join("log"))?;
        if from.join("beat").exists() {
            let _ = fs::copy(from.join("beat"), dir.join("beat"));
        }
        if from.join("meta").exists() {
            let _ = fs::copy(from.join("meta"), dir.join("meta"));
        }
        let mut s = Store::open(dir, name, 0)?;
        s.read_only = true;
        Ok(s)
    }

    /// Pull to match `from`. Same generation: append
    /// new bytes. New generation: replace the file.
    pub fn catch_up(&mut self, from: &Path) -> io::Result<()> {
        if !self.read_only {
            return Err(io::Error::new(io::ErrorKind::Other, "not a replica"));
        }
        if self.open_tx.is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "in a transaction"));
        }
        let src = from.join("log");
        let src_pos = pos_of(&src)?;
        let dst_pos = self.pos()?;
        if src_pos == dst_pos {
            if from.join("beat").exists() {
                let _ = fs::copy(from.join("beat"), self.dir.join("beat"));
            }
            let _ = super::meta::catch_up(&self.dir, from);
            return Ok(());
        }
        if src_pos.generation() != dst_pos.generation()
            || src_pos.offset() < dst_pos.offset() {
            let tmp = self.dir.join("log.new");
            fs::copy(&src, &tmp)?;
            fs::rename(&tmp, self.dir.join("log"))?;
        } else {
            let mut f = File::open(&src)?;
            f.seek(SeekFrom::Start(dst_pos.offset()))?;
            self.log.seek(SeekFrom::End(0))?;
            io::copy(&mut f, &mut self.log)?;
            self.log.sync_data()?;
        }
        if from.join("beat").exists() {
            let _ = fs::copy(from.join("beat"), self.dir.join("beat"));
        }
        let _ = super::meta::catch_up(&self.dir, from);
        self.reopen_replica()
    }

    /// Replica: catch_up until `need` is honored.
    /// Primary always honors. Fails if still behind.
    pub fn honor(&mut self, from: &Path, need: Pos) -> io::Result<()> {
        if !self.read_only {
            return Ok(());
        }
        if self.pos()?.honors(need) {
            return Ok(());
        }
        self.catch_up(from)?;
        if self.pos()?.honors(need) {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "bookmark not honored"))
        }
    }

    fn reopen_replica(&mut self) -> io::Result<()> {
        let name = self.g.khid().to_string();
        let dir = self.dir.clone();
        let mut s = Store::open(&dir, &name, 0)?;
        s.read_only = true;
        let _ = super::meta::Meta::rebuild(&s.dir, &s.g);
        *self = s;
        Ok(())
    }

    /// Pull from a primary over TCP. Same Pos rules
    /// as catch_up. Does not wait on commit.
    pub fn follow(&mut self, addr: SocketAddr) -> io::Result<Pos> {
        if !self.read_only {
            return Err(io::Error::new(io::ErrorKind::Other, "not a replica"));
        }
        if self.open_tx.is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "in a transaction"));
        }
        let have = self.pos()?;
        let dest = self.dir.join("log");
        let p = super::wire::pull(addr, have, &dest)?;
        self.reopen_replica()?;
        Ok(p)
    }

    /// This copy is now home. Split brain is the deal.
    pub fn promote(&mut self) {
        self.read_only = false;
        let _ = self.take_lease();
    }

    fn take_lease(&self) -> io::Result<()> {
        let until = unix() + 3600;
        let mut f = File::create(self.dir.join("lease"))?;
        write!(f, "{} {}\n", self.token, until)?;
        f.sync_data()
    }

    fn hold_lease(&self) -> bool {
        match read_lease(&self.dir) {
            Some((tok, until)) => {
                if tok == self.token {
                    true
                } else if unix() >= until {
                    let _ = self.take_lease();
                    true
                } else {
                    false
                }
            }
            None => {
                let _ = self.take_lease();
                true
            }
        }
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        if self.read_only {
            return;
        }
        if let Some((tok, _)) = read_lease(&self.dir) {
            if tok == self.token {
                let _ = fs::remove_file(self.dir.join("lease"));
            }
        }
    }
}

fn pos_of(path: &Path) -> io::Result<Pos> {
    let mut f = File::open(path)?;
    let len = f.metadata()?.len();
    let h = wal::head(&mut f)?;
    let gen = if h.generation == 0 { 1 } else { h.generation };
    Ok(Pos::new(gen, len))
}

fn unix() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => 0,
    }
}

fn new_token() -> u64 {
    use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
    static SEQ: AtomicUsize = ATOMIC_USIZE_INIT;
    let n = SEQ.fetch_add(1, Ordering::SeqCst) as u64 + 1;
    unix() ^ (n << 16) ^ ((std::process::id() as u64) << 32)
}

fn sync_dir(dir: &Path) -> io::Result<()> {
    let f = File::open(dir)?;
    f.sync_data()
}

fn read_lease(dir: &Path) -> Option<(u64, u64)> {
    let s = match fs::read_to_string(dir.join("lease")) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let mut it = s.split_whitespace();
    let tok = match it.next().and_then(|x| x.parse().ok()) {
        Some(n) => n,
        None => return None,
    };
    let until = match it.next().and_then(|x| x.parse().ok()) {
        Some(n) => n,
        None => return None,
    };
    Some((tok, until))
}

fn capture(tx: u64, g: &Graph) -> Vec<Rec> {
    let mut recs = Vec::new();
    recs.push(Rec::Begin { tx: tx });
    for &(tid, _) in g.all_types().iter() {
        if let Some(t) = g.ty(tid) {
            let name = t.name().to_string();
            for k in t.content_keys().iter() {
                recs.push(Rec::Content {
                    tx: tx,
                    type_name: name.clone(),
                    key: k.clone(),
                });
            }
        }
    }
    for id in g.vertex_ids().iter() {
        let types = g.type_names_of_vertex(*id);
        let attrs = match g.vertex(*id) {
            Some(v) => v.attrs().clone(),
            None => continue,
        };
        recs.push(Rec::Vertex {
            tx: tx,
            id: *id,
            types: types,
            attrs: attrs,
        });
    }
    for &(id, src, dst, _) in g.all_edges().iter() {
        let e: &Edge = match g.edge(id) {
            Some(e) => e,
            None => continue,
        };
        let ty = g.edge_type_name(id).unwrap_or(String::new());
        let attrs = e.attrs().clone();
        if e.is_far() {
            recs.push(Rec::FarEdge {
                tx: tx,
                id: id,
                src: src,
                dst: e.far().unwrap_or(Addr::here(Khid::nil())),
                ty: ty,
                attrs: attrs,
            });
        } else {
            recs.push(Rec::Edge {
                tx: tx,
                id: id,
                src: src,
                dst: dst,
                ty: ty,
                attrs: attrs,
            });
        }
    }
    for &(ref tn, ref k, u) in g.index_specs().iter() {
        recs.push(Rec::Index {
            tx: tx,
            type_name: tn.clone(),
            key: k.clone(),
            unique: u,
        });
    }
    recs.push(Rec::Commit { tx: tx });
    recs
}

fn vertex_same(a: &Graph, b: &Graph, id: Khid) -> bool {
    match (a.vertex(id), b.vertex(id)) {
        (Some(x), Some(y)) => {
            x.attrs() == y.attrs() && a.type_names_of_vertex(id) == b.type_names_of_vertex(id)
        }
        (None, None) => true,
        _ => false,
    }
}

fn capture_delta(tx: u64, before: &Graph, after: &Graph) -> Vec<Rec> {
    let mut recs = Vec::new();
    recs.push(Rec::Begin { tx: tx });
    for &(tid, _) in after.all_types().iter() {
        if let Some(t) = after.ty(tid) {
            let name = t.name().to_string();
            let old = before.type_by_name(&name);
            for k in t.content_keys().iter() {
                let already = match old {
                    Some(o) => o.is_content(k),
                    None => false,
                };
                if already {
                    continue;
                }
                recs.push(Rec::Content {
                    tx: tx,
                    type_name: name.clone(),
                    key: k.clone(),
                });
            }
        }
    }
    for id in after.vertex_ids().iter() {
        if vertex_same(before, after, *id) {
            continue;
        }
        let types = after.type_names_of_vertex(*id);
        let attrs = match after.vertex(*id) {
            Some(v) => v.attrs().clone(),
            None => continue,
        };
        recs.push(Rec::Vertex {
            tx: tx,
            id: *id,
            types: types,
            attrs: attrs,
        });
    }
    let mut old_edges: HashMap<Khid, ()> = HashMap::new();
    for &(id, _, _, _) in before.all_edges().iter() {
        old_edges.insert(id, ());
    }
    for &(id, src, dst, _) in after.all_edges().iter() {
        if old_edges.contains_key(&id) {
            continue;
        }
        let e: &Edge = match after.edge(id) {
            Some(e) => e,
            None => continue,
        };
        let ty = after.edge_type_name(id).unwrap_or(String::new());
        let attrs = e.attrs().clone();
        if e.is_far() {
            recs.push(Rec::FarEdge {
                tx: tx,
                id: id,
                src: src,
                dst: e.far().unwrap_or(Addr::here(Khid::nil())),
                ty: ty,
                attrs: attrs,
            });
        } else {
            recs.push(Rec::Edge {
                tx: tx,
                id: id,
                src: src,
                dst: dst,
                ty: ty,
                attrs: attrs,
            });
        }
    }
    for &(ref tn, ref k, u) in after.index_specs().iter() {
        if before.has_index(tn, k) {
            continue;
        }
        recs.push(Rec::Index {
            tx: tx,
            type_name: tn.clone(),
            key: k.clone(),
            unique: u,
        });
    }
    recs.push(Rec::Commit { tx: tx });
    recs
}

/// Open failed as a kernel error.
pub fn open_err(e: io::Error) -> Error {
    Error::new(&e.to_string())
}
