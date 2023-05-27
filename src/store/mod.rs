//! A shard on disk. The log is truth. Commit
//! writes the delta against the snapshot.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::error::Error;
use super::graph::{Graph, Touch};
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
    read_only: bool,
    generation: u32,
    token: u64,
    durable: bool,
    dirty: bool,
    sync_every: u32,
    since_sync: u32,
    compact_at: u64,
    next_blob: u64,
    blob_of: HashMap<(Khid, String), u64>,
    next_vec: u64,
    vec_of: HashMap<(Khid, String), u64>,
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
        let (g, next_tx, generation, recs) = if len == 0 {
            wal::write_header(shard, 1, &mut log)?;
            log.sync_data()?;
            (Graph::on(name, shard), 1, 1, Vec::new())
        } else {
            log.seek(SeekFrom::Start(0))?;
            let (h, recs, end) = wal::read_valid(&mut log)?;
            let mut g = match wal::replay(h.shard, &recs) {
                Ok(g) => g,
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e.message())),
            };
            g.set_id(name);
            super::blob::fill(dir, &mut g, &recs)?;
            super::vec::fill(dir, &mut g, &recs)?;
            let mut max = 0u64;
            for rec in recs.iter() {
                if rec.tx() > max {
                    max = rec.tx();
                }
            }
            if end < len {
                log.set_len(end)?;
            }
            log.seek(SeekFrom::End(0))?;
            let gen = if h.generation == 0 { 1 } else { h.generation };
            (g, max + 1, gen, recs)
        };
        let mut blob_of: HashMap<(Khid, String), u64> = HashMap::new();
        let mut vec_of: HashMap<(Khid, String), u64> = HashMap::new();
        for rec in recs.iter() {
            if let Rec::Vertex { id, ref blobs, .. } = *rec {
                for &(ref k, s) in blobs.iter() {
                    blob_of.insert((id, k.clone()), s);
                }
            }
            if let Rec::Emb { id, ref key, serial, .. } = *rec {
                vec_of.insert((id, key.clone()), serial);
            }
        }
        let mut s = Store {
            dir: dir.to_path_buf(),
            log: log,
            g: g,
            next_tx: next_tx,
            open_tx: None,
            read_only: false,
            generation: generation,
            token: new_token(),
            durable: true,
            dirty: false,
            sync_every: 1,
            since_sync: 0,
            compact_at: 0,
            next_blob: super::blob::max_serial(dir) + 1,
            blob_of: blob_of,
            next_vec: super::vec::max_serial(dir) + 1,
            vec_of: vec_of,
        };
        s.compact_at = s.log.metadata()?.len();
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
        self.g.arm();
        self.open_tx = Some(self.next_tx);
        self.next_tx += 1;
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
        let _ = self.tx_id()?;
        let id = match self.g.add_vertex_props(attrs.clone(), ty) {
            Ok(id) => id,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.message())),
        };
        Ok(id)
    }

    pub fn put_far(&mut self,
                   src: Khid,
                   dst: Addr,
                   ty: Option<&str>)
                   -> io::Result<Khid> {
        let _ = self.tx_id()?;
        let id = match self.g.add_far_edge(src, dst, ty) {
            Ok(id) => id,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.message())),
        };
        Ok(id)
    }

    pub fn put_content(&mut self, type_name: &str, key: &str) -> io::Result<()> {
        let _tx = self.tx_id()?;
        self.g.mark_content(type_name, key);
        Ok(())
    }

    pub fn put_index(&mut self, type_name: &str, key: &str) -> io::Result<()> {
        let _tx = self.tx_id()?;
        self.g.create_index(type_name, key);
        Ok(())
    }

    fn spill(&mut self) -> io::Result<()> {
        let mut ids = Vec::new();
        for t in self.g.touches().iter() {
            if let Touch::Vertex(id) = *t {
                ids.push(id);
            }
        }
        self.spill_ids(&ids)
    }

    fn spill_all(&mut self) -> io::Result<()> {
        let ids = self.g.vertex_ids();
        self.spill_ids(&ids)
    }

    fn spill_ids(&mut self, ids: &[Khid]) -> io::Result<()> {
        for id in ids.iter() {
            for (k, p) in self.g.content_of(*id).iter() {
                let bytes = match p.as_str() {
                    Some(s) => s.as_bytes().to_vec(),
                    None => p.as_display().into_bytes(),
                };
                let reuse = match self.blob_of.get(&(*id, k.clone())) {
                    Some(&ser) => match super::blob::get(&self.dir, *id, ser) {
                        Ok(Some(old)) => old == bytes,
                        _ => false,
                    },
                    None => false,
                };
                if reuse {
                    continue;
                }
                let ser = self.next_blob;
                self.next_blob += 1;
                super::blob::put(&self.dir, *id, ser, &bytes)?;
                self.blob_of.insert((*id, k.clone()), ser);
            }
        }
        super::blob::sync_dir(&self.dir)
    }

    fn spill_vecs(&mut self) -> io::Result<()> {
        let mut pairs = Vec::new();
        for t in self.g.touches().iter() {
            if let Touch::Emb { id, ref key } = *t {
                pairs.push((id, key.clone()));
            }
        }
        self.spill_vec_pairs(&pairs)
    }

    fn spill_vecs_all(&mut self) -> io::Result<()> {
        let pairs: Vec<(Khid, String)> = self.g.embeddings().into_iter()
            .map(|(id, k, _)| (id, k))
            .collect();
        self.spill_vec_pairs(&pairs)
    }

    fn spill_vec_pairs(&mut self, pairs: &[(Khid, String)]) -> io::Result<()> {
        for &(id, ref key) in pairs.iter() {
            let v = match self.g.get_vec(id, key) {
                Some(v) => v.to_vec(),
                None => continue,
            };
            let reuse = match self.vec_of.get(&(id, key.clone())) {
                Some(&ser) => match super::vec::get(&self.dir, id, ser) {
                    Ok(Some(old)) => old == v,
                    _ => false,
                },
                None => false,
            };
            if reuse {
                continue;
            }
            let ser = self.next_vec;
            self.next_vec += 1;
            super::vec::put(&self.dir, id, ser, &v)?;
            self.vec_of.insert((id, key.clone()), ser);
        }
        super::vec::sync_dir(&self.dir)
    }

    /// The log is the delta against the snapshot.
    pub fn commit(&mut self) -> io::Result<Pos> {
        if self.read_only {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "read-only replica"));
        }
        let tx = self.tx_id()?;
        self.spill()?;
        self.spill_vecs()?;
        let recs = recs_from_touches(tx, &self.g, &self.blob_of, &self.vec_of);
        wal::append(&recs, &mut self.log)?;
        self.since_sync = self.since_sync.saturating_add(1);
        if self.durable && self.since_sync >= self.sync_every {
            self.log.sync_data()?;
            let _ = sync_dir(&self.dir);
            self.dirty = false;
            self.since_sync = 0;
        } else if !self.durable || self.sync_every > 1 {
            self.dirty = true;
        }
        self.write_beat(tx)?;
        let _ = self.take_lease();
        self.open_tx = None;
        let _ = super::meta::Meta::tail(&self.dir, &self.g);
        self.g.disarm();
        self.pos()
    }

    /// Grouped sync. The default commit already hits
    /// the platter. A session that set durable false
    /// calls this.
    pub fn flush(&mut self) -> io::Result<Pos> {
        if self.dirty {
            self.log.sync_data()?;
            let _ = sync_dir(&self.dir);
            self.dirty = false;
            self.since_sync = 0;
        }
        self.pos()
    }

    pub fn set_durable(&mut self, yes: bool) {
        self.durable = yes;
    }

    pub fn durable(&self) -> bool {
        self.durable
    }

    pub fn set_sync_every(&mut self, n: u32) {
        self.sync_every = if n == 0 { 1 } else { n };
    }

    pub fn fold_meta(&self) -> io::Result<()> {
        let m = super::meta::Meta::open(&self.dir)?;
        m.fold()
    }

    /// How far this copy lags `at`. Same generation only.
    pub fn lag(&self, at: Pos) -> io::Result<u64> {
        let here = self.pos()?;
        if here.generation() != at.generation() {
            return Err(io::Error::new(io::ErrorKind::Other, "old generation"));
        }
        if at.offset() >= here.offset() {
            Ok(at.offset() - here.offset())
        } else {
            Ok(0)
        }
    }

    /// Cypher against the live arena. A read does not
    /// take the lease. Commit is separate.
    pub fn query(&mut self, text: &str) -> super::query::QueryResult {
        if !super::query::writes(text) {
            return super::query::ask(&self.g, text);
        }
        match self.graph_mut() {
            Ok(g) => super::query::run(g, text),
            Err(e) => super::query::QueryResult::error(&e.to_string()),
        }
    }

    pub fn ask(&self, text: &str) -> super::query::QueryResult {
        super::query::ask(&self.g, text)
    }

    pub fn rollback(&mut self) {
        self.open_tx = None;
        self.g.apply_undos();
        self.g.disarm();
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
        super::blob::fill(&self.dir, &mut g, &recs)?;
        super::vec::fill(&self.dir, &mut g, &recs)?;
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
        self.spill_all()?;
        self.spill_vecs_all()?;
        let recs = capture(tx, &self.g, &self.blob_of, &self.vec_of);
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
        let _ = super::blob::gc(&self.dir, &super::blob::live_from(&recs));
        let _ = super::vec::gc(&self.dir, &super::vec::live_from(&recs));
        self.compact_at = self.log.metadata()?.len();
        self.pos()
    }

    /// Compact when the log is a multiple of the
    /// last capture. The writer notices. No thread.
    pub fn maybe_compact(&mut self) -> io::Result<Option<Pos>> {
        let n = self.log.metadata()?.len();
        if self.compact_at > 0 && n > self.compact_at.saturating_mul(4) {
            return self.compact().map(Some);
        }
        Ok(None)
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

}

include!("repl.rs");
include!("spill.rs");
