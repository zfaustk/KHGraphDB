//! Compact identity. The print form is still `k` then hex.
//! The value is the serial. Slot 0 is not a vertex.

use std::fmt;
use std::str::FromStr;

use super::error::{Error, Result};

/// A KHID. Copy, hash, order. Not a string that happens
/// to look like one.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Khid(u64);

impl Khid {
    pub fn nil() -> Khid {
        Khid(0)
    }

    pub fn from_raw(n: u64) -> Khid {
        Khid(n)
    }

    pub fn raw(self) -> u64 {
        self.0
    }

    pub fn is_nil(self) -> bool {
        self.0 == 0
    }

    /// Parse `k1a` / `K1A`. Anything else is None.
    pub fn parse(s: &str) -> Option<Khid> {
        if s.len() < 2 {
            return None;
        }
        let b = s.as_bytes();
        if b[0] != b'k' && b[0] != b'K' {
            return None;
        }
        match u64::from_str_radix(&s[1..], 16) {
            Ok(n) => Some(Khid(n)),
            Err(_) => None,
        }
    }

    pub fn display_all(ks: &[Khid]) -> Vec<String> {
        let mut v = Vec::new();
        let mut i = 0;
        while i < ks.len() {
            v.push(format!("{}", ks[i]));
            i += 1;
        }
        v
    }
}

impl Default for Khid {
    fn default() -> Khid {
        Khid::nil()
    }
}

impl fmt::Display for Khid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "k{:x}", self.0)
    }
}

impl FromStr for Khid {
    type Err = Error;
    fn from_str(s: &str) -> Result<Khid> {
        match Khid::parse(s) {
            Some(k) => Ok(k),
            None => Err(Error::new("bad khid")),
        }
    }
}
