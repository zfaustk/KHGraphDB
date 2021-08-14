//! Known homes. FIND fans out once, then the
//! client talks to those addresses. Not a cluster
//! manager. Not a majority.

use std::io;
use std::net::SocketAddr;

use super::addr::Addr;
use super::wire;

/// shard → socket. Locate asks every home.
/// Empty homes: no Addr.
pub struct Route {
    homes: Vec<(u32, SocketAddr)>,
}

impl Route {
    pub fn new() -> Route {
        Route { homes: Vec::new() }
    }

    pub fn add(&mut self, shard: u32, addr: SocketAddr) {
        let mut i = 0;
        while i < self.homes.len() {
            if self.homes[i].0 == shard {
                self.homes[i].1 = addr;
                return;
            }
            i += 1;
        }
        self.homes.push((shard, addr));
    }

    pub fn get(&self, shard: u32) -> Option<SocketAddr> {
        for &(s, a) in self.homes.iter() {
            if s == shard {
                return Some(a);
            }
        }
        None
    }

    /// One round: each home answers its posting.
    pub fn locate(&self, type_name: &str, key: &str, value: &str)
                  -> io::Result<Vec<Addr>> {
        let mut out = Vec::new();
        for &(_, addr) in self.homes.iter() {
            let mut found = wire::find(addr, type_name, key, value)?;
            out.append(&mut found);
        }
        Ok(out)
    }

    pub fn len(&self) -> usize {
        self.homes.len()
    }
}