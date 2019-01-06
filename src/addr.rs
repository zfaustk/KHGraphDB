//! An address. KHID is the serial on one shard.
//! Together they name a vertex anywhere.

use std::fmt;
use std::str::FromStr;

use super::error::{Error, Result};
use super::khid::Khid;

/// `(shard, khid)`. Shard 0 means this box.
/// Copy, hash, order.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Addr {
    shard: u32,
    id: Khid,
}

impl Addr {
    pub fn new(shard: u32, id: Khid) -> Addr {
        Addr {
            shard: shard,
            id: id,
        }
    }

    /// An address on this box. Display is just the KHID.
    pub fn here(id: Khid) -> Addr {
        Addr::new(0, id)
    }

    pub fn shard(self) -> u32 {
        self.shard
    }

    pub fn khid(self) -> Khid {
        self.id
    }

    pub fn is_here(self) -> bool {
        self.shard == 0
    }

    /// True if this address is on `shard`, or is
    /// unplaced (0) and so belongs wherever we are.
    pub fn on(self, shard: u32) -> bool {
        self.shard == 0 || self.shard == shard
    }

    /// `k1a` on this box. `s2/k1a` elsewhere.
    pub fn parse(s: &str) -> Option<Addr> {
        if s.len() >= 3 && (s.as_bytes()[0] == b's' || s.as_bytes()[0] == b'S') {
            let slash = match s.find('/') {
                Some(i) => i,
                None => return None,
            };
            if slash < 2 {
                return None;
            }
            let shard = match s[1..slash].parse::<u32>() {
                Ok(n) => n,
                Err(_) => return None,
            };
            match Khid::parse(&s[slash + 1..]) {
                Some(id) => Some(Addr::new(shard, id)),
                None => None,
            }
        } else {
            match Khid::parse(s) {
                Some(id) => Some(Addr::here(id)),
                None => None,
            }
        }
    }
}

impl Default for Addr {
    fn default() -> Addr {
        Addr::here(Khid::nil())
    }
}

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.shard == 0 {
            write!(f, "{}", self.id)
        } else {
            write!(f, "s{}/{}", self.shard, self.id)
        }
    }
}

impl FromStr for Addr {
    type Err = Error;
    fn from_str(s: &str) -> Result<Addr> {
        match Addr::parse(s) {
            Some(a) => Ok(a),
            None => Err(Error::new("bad addr")),
        }
    }
}
