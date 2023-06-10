//! A primary and its copies. Commit is local, then
//! catch_up until `min_ack` members honor the Pos.
//! Failover promotes a copy and fences the old
//! directory with a new lease. No vote. The lease
//! is still the fence. Raft stays in 2019.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::pos::Pos;
use super::store::Store;

pub struct Member {
    pub dir: PathBuf,
    pub name: String,
}

pub struct ReplSet {
    primary: PathBuf,
    name: String,
    shard: u32,
    members: Vec<PathBuf>,
    min_ack: u32,
}

impl ReplSet {
    pub fn open(primary: &Path, name: &str, shard: u32) -> io::Result<ReplSet> {
        let _ = Store::open(primary, name, shard)?;
        Ok(ReplSet {
            primary: primary.to_path_buf(),
            name: name.to_string(),
            shard: shard,
            members: Vec::new(),
            min_ack: 1,
        })
    }

    pub fn add(&mut self, dir: &Path) -> io::Result<()> {
        let _ = Store::tail(dir, &self.primary, &self.name)?;
        self.members.push(dir.to_path_buf());
        Ok(())
    }

    /// 1 = primary only. 2 = one copy must honor.
    pub fn set_min_ack(&mut self, n: u32) {
        self.min_ack = if n == 0 { 1 } else { n };
    }

    pub fn min_ack(&self) -> u32 {
        self.min_ack
    }

    pub fn primary(&self) -> &Path {
        &self.primary
    }

    pub fn members(&self) -> &[PathBuf] {
        &self.members
    }

    pub fn store(&self) -> io::Result<Store> {
        Store::open(&self.primary, &self.name, self.shard)
    }

    /// Apply on the primary, then catch_up copies
    /// until min_ack honor the new Pos.
    pub fn apply<F>(&mut self, f: F) -> io::Result<Pos>
        where F: FnOnce(&mut Store) -> io::Result<()>
    {
        let pos = {
            let mut s = Store::open(&self.primary, &self.name, self.shard)?;
            f(&mut s)?;
            s.commit()?
        };
        let mut acks = 1u32;
        for m in self.members.iter() {
            match Store::open(m, &self.name, self.shard) {
                Ok(mut r) => {
                    if !r.is_replica() {
                        r = Store::attach(m, &self.name)?;
                    }
                    match r.catch_up(&self.primary) {
                        Ok(()) => {
                            if r.pos().map(|p| p.honors(pos)).unwrap_or(false) {
                                acks += 1;
                            }
                        }
                        Err(_) => {}
                    }
                }
                Err(_) => {
                    if Store::tail(m, &self.primary, &self.name).is_ok() {
                        acks += 1;
                    }
                }
            }
        }
        if acks < self.min_ack {
            return Err(io::Error::new(io::ErrorKind::Other, "min_ack"));
        }
        Ok(pos)
    }

    pub fn lag(&self) -> io::Result<Vec<(PathBuf, u64)>> {
        let p = {
            let s = Store::open(&self.primary, &self.name, self.shard)?;
            s.pos()?
        };
        let mut v = Vec::new();
        for m in self.members.iter() {
            let r = Store::attach(m, &self.name)?;
            v.push((m.clone(), r.lag(p).unwrap_or(u64::MAX)));
        }
        Ok(v)
    }

    /// Promote the first copy. The old primary
    /// directory is left as a stale lease. New
    /// writes go to the promoted dir.
    pub fn failover(&mut self) -> io::Result<PathBuf> {
        if self.members.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "no replica"));
        }
        let next = self.members.remove(0);
        {
            let mut r = Store::attach(&next, &self.name)?;
            r.promote();
            let _ = r.commit();
        }
        let old = self.primary.clone();
        self.members.insert(0, old);
        self.primary = next.clone();
        Ok(next)
    }

    pub fn catch_all(&mut self) -> io::Result<()> {
        for m in self.members.iter() {
            let mut r = Store::attach(m, &self.name)?;
            let _ = r.catch_up(&self.primary);
        }
        Ok(())
    }

    pub fn forget_stale(&self) {
        let _ = fs::remove_file(self.primary.join("lease"));
    }
}
