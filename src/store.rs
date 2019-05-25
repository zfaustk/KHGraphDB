//! A shard on disk. The log is truth. Commit
//! captures the arena, appends, sync_data.
//! Drop without commit keeps the last snapshot.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use super::error::Error;
use super::graph::Graph;
use super::wal::{self, Rec};
use super::khid::Khid;
use super::addr::Addr;
use super::edge::Edge;

/// Durable home of one shard.
pub struct Store {
    dir: PathBuf,
    log: File,
    g: Graph,
    next_tx: u64,
    open_tx: Option<u64>,
    snap: Option<Graph>,
    read_only: bool,
    term: u64,
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
        let (g, next_tx, term) = if len == 0 {
            wal::write_header(shard, &mut log)?;
            log.sync_data()?;
            (Graph::on(name, shard), 1, 0)
        } else {
            log.seek(SeekFrom::Start(0))?;
            let (sh, recs) = wal::read(&mut log)?;
            let mut g = match wal::replay(sh, &recs) {
                Ok(g) => g,
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e.message())),
            };
            g.set_id(name);
            let mut max = 0u64;
            let mut term = 0u64;
            for rec in recs.iter() {
                if rec.tx() > max {
                    max = rec.tx();
                }
                if let wal::Rec::Term { term: t, .. } = rec {
                    if *t > term {
                        term = *t;
                    }
                }
            }
            log.seek(SeekFrom::End(0))?;
            (g, max + 1, term)
        };
        Ok(Store {
            dir: dir.to_path_buf(),
            log: log,
            g: g,
            next_tx: next_tx,
            open_tx: None,
            snap: None,
            read_only: false,
            term: term,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn graph(&self) -> &Graph {
        &self.g
    }

    pub fn graph_mut(&mut self) -> &mut Graph {
        &mut self.g
    }

    pub fn begin(&mut self) {
        if self.open_tx.is_some() {
            return;
        }
        self.snap = Some(self.g.snapshot());
        self.open_tx = Some(self.next_tx);
        self.next_tx += 1;
    }

    /// Capture the arena, append, fsync. The log is truth.
    pub fn commit(&mut self) -> io::Result<()> {
        if self.read_only {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "read-only replica"));
        }
        let tx = match self.open_tx {
            Some(t) => t,
            None => {
                self.begin();
                self.open_tx.unwrap()
            }
        };
        let recs = capture(tx, self.term, &self.g);
        wal::append(&recs, &mut self.log)?;
        self.log.sync_data()?;
        self.write_beat(tx)?;
        self.open_tx = None;
        self.snap = None;
        Ok(())
    }

    pub fn rollback(&mut self) {
        if let Some(s) = self.snap.take() {
            self.g = s;
        }
        self.open_tx = None;
    }

    pub fn in_tx(&self) -> bool {
        self.open_tx.is_some()
    }

    pub fn name(&self) -> &str {
        self.g.khid()
    }

    /// Rewrite the log as one capture. Same truth,
    /// less tail. Caller is not in a tx.
    pub fn compact(&mut self) -> io::Result<()> {
        if self.read_only {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "read-only replica"));
        }
        if self.open_tx.is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "in a transaction"));
        }
        let shard = self.g.shard();
        let tx = self.next_tx;
        self.next_tx += 1;
        let recs = capture(tx, self.term, &self.g);
        let tmp = self.dir.join("log.tmp");
        {
            let mut f = File::create(&tmp)?;
            wal::write(shard, &recs, &mut f)?;
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
        Ok(())
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

    /// A copy of the log. Read-only until promote.
    pub fn tail(dir: &Path, from: &Path, name: &str) -> io::Result<Store> {
        fs::create_dir_all(dir)?;
        fs::copy(from.join("log"), dir.join("log"))?;
        if from.join("beat").exists() {
            let _ = fs::copy(from.join("beat"), dir.join("beat"));
        }
        let mut s = Store::open(dir, name, 0)?;
        s.read_only = true;
        Ok(s)
    }

    /// Copy the primary log again. Only a replica.
    pub fn catch_up(&mut self, from: &Path) -> io::Result<()> {
        if !self.read_only {
            return Err(io::Error::new(io::ErrorKind::Other, "not a replica"));
        }
        if self.open_tx.is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "in a transaction"));
        }
        fs::copy(from.join("log"), self.dir.join("log"))?;
        if from.join("beat").exists() {
            let _ = fs::copy(from.join("beat"), self.dir.join("beat"));
        }
        let name = self.g.khid().to_string();
        let mut s = Store::open(&self.dir, &name, 0)?;
        s.read_only = true;
        *self = s;
        Ok(())
    }

    /// This copy is now home. Split brain is the deal.
    pub fn promote(&mut self) {
        self.read_only = false;
    }

    pub fn term(&self) -> u64 {
        self.term
    }

    /// Bump the term. A week of thinking a copy
    /// needs an election.
    pub fn elect(&mut self) -> io::Result<()> {
        if self.read_only {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "read-only replica"));
        }
        self.term += 1;
        Ok(())
    }
}

fn capture(tx: u64, term: u64, g: &Graph) -> Vec<Rec> {
    let mut recs = Vec::new();
    recs.push(Rec::Begin { tx: tx });
    if term > 0 {
        recs.push(Rec::Term { tx: tx, term: term });
    }
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

/// Open failed as a kernel error.
pub fn open_err(e: io::Error) -> Error {
    Error::new(&e.to_string())
}
