//! A primary and its copies, held open. Commit
//! then catch_up on the live handles. min_ack.
//! Failover promotes a copy. No vote.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::pos::Pos;
use super::store::Store;

pub struct ReplSet {
    primary: Store,
    replicas: Vec<Store>,
    min_ack: u32,
    name: String,
}

impl ReplSet {
    pub fn open(primary: &Path, name: &str, shard: u32) -> io::Result<ReplSet> {
        let s = Store::open(primary, name, shard)?;
        Ok(ReplSet {
            primary: s,
            replicas: Vec::new(),
            min_ack: 1,
            name: name.to_string(),
        })
    }

    pub fn add(&mut self, dir: &Path) -> io::Result<()> {
        let r = Store::tail(dir, self.primary.dir(), &self.name)?;
        self.replicas.push(r);
        Ok(())
    }

    pub fn set_min_ack(&mut self, n: u32) {
        self.min_ack = if n == 0 { 1 } else { n };
    }

    pub fn min_ack(&self) -> u32 {
        self.min_ack
    }

    pub fn primary(&self) -> &Path {
        self.primary.dir()
    }

    pub fn members(&self) -> Vec<PathBuf> {
        self.replicas.iter().map(|r| r.dir().to_path_buf()).collect()
    }

    pub fn store(&mut self) -> &mut Store {
        &mut self.primary
    }

    pub fn apply<F>(&mut self, f: F) -> io::Result<Pos>
        where F: FnOnce(&mut Store) -> io::Result<()>
    {
        f(&mut self.primary)?;
        let pos = self.primary.commit()?;
        let from = self.primary.dir().to_path_buf();
        let mut acks = 1u32;
        for r in self.replicas.iter_mut() {
            match r.catch_up(&from) {
                Ok(()) => {
                    if r.pos().map(|p| p.honors(pos)).unwrap_or(false) {
                        acks += 1;
                    }
                }
                Err(_) => {}
            }
        }
        if acks < self.min_ack {
            return Err(io::Error::new(io::ErrorKind::Other, "min_ack"));
        }
        Ok(pos)
    }

    pub fn lag(&self) -> io::Result<Vec<(PathBuf, u64)>> {
        let p = self.primary.pos()?;
        let mut v = Vec::new();
        for r in self.replicas.iter() {
            v.push((r.dir().to_path_buf(), r.lag(p).unwrap_or(u64::MAX)));
        }
        Ok(v)
    }

    pub fn failover(&mut self) -> io::Result<PathBuf> {
        if self.replicas.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "no replica"));
        }
        self.primary.demote();
        let mut next = self.replicas.remove(0);
        next.promote();
        let old = ::std::mem::replace(&mut self.primary, next);
        let dir = self.primary.dir().to_path_buf();
        self.replicas.insert(0, old);
        Ok(dir)
    }

    pub fn catch_all(&mut self) -> io::Result<()> {
        let from = self.primary.dir().to_path_buf();
        for r in self.replicas.iter_mut() {
            let _ = r.catch_up(&from);
        }
        Ok(())
    }

    pub fn forget_stale(&self) {
        let _ = fs::remove_file(self.primary.dir().join("lease"));
    }
}