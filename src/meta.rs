//! Thin posting log. KHM1. Derived: drop it,
//! replay the WAL, it comes back. FIND reads
//! this, not the page.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use super::addr::Addr;
use super::graph::Graph;
use super::khid::Khid;

const MAGIC: &'static [u8] = b"KHM1";
const TAG_PUT: u8 = 1;

/// (type, key, value) → addresses.
pub struct Meta {
    dir: PathBuf,
    map: HashMap<(String, String, String), Vec<Addr>>,
}

impl Meta {
    pub fn empty(dir: &Path) -> Meta {
        Meta {
            dir: dir.to_path_buf(),
            map: HashMap::new(),
        }
    }

    pub fn open(dir: &Path) -> io::Result<Meta> {
        let path = dir.join("meta");
        if !path.exists() {
            return Ok(Meta::empty(dir));
        }
        let mut f = File::open(&path)?;
        let mut magic = [0u8; 4];
        match f.read_exact(&mut magic) {
            Ok(()) => {}
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                return Ok(Meta::empty(dir));
            }
            Err(e) => return Err(e),
        }
        if &magic != MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "not KHM1"));
        }
        let mut map: HashMap<(String, String, String), Vec<Addr>> = HashMap::new();
        loop {
            let mut tag = [0u8; 1];
            match f.read_exact(&mut tag) {
                Ok(()) => {}
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            if tag[0] != TAG_PUT {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "meta tag"));
            }
            let tn = read_str(&mut f)?;
            let key = read_str(&mut f)?;
            let val = read_str(&mut f)?;
            let shard = read_u32(&mut f)?;
            let raw = read_u64(&mut f)?;
            let addr = Addr::new(shard, Khid::from_raw(raw));
            map.entry((tn, key, val)).or_insert(Vec::new()).push(addr);
        }
        Ok(Meta {
            dir: dir.to_path_buf(),
            map: map,
        })
    }

    /// Rewrite from the arena. Same truth as the log.
    pub fn rebuild(dir: &Path, g: &Graph) -> io::Result<Meta> {
        let mut m = Meta::empty(dir);
        for &(ref tn, ref k, ref val, addr) in g.index_addrs().iter() {
            m.map
                .entry((tn.clone(), k.clone(), val.clone()))
                .or_insert(Vec::new())
                .push(addr);
        }
        m.sync()?;
        Ok(m)
    }

    pub fn find(&self, type_name: &str, key: &str, value: &str) -> Vec<Addr> {
        match self.map.get(&(type_name.to_string(), key.to_string(), value.to_string())) {
            Some(v) => v.clone(),
            None => Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        let mut n = 0;
        for v in self.map.values() {
            n += v.len();
        }
        n
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn sync(&self) -> io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        let tmp = self.dir.join("meta.tmp");
        {
            let mut f = File::create(&tmp)?;
            f.write_all(MAGIC)?;
            for (k, addrs) in self.map.iter() {
                let (tn, key, val) = k;
                for a in addrs.iter() {
                    f.write_all(&[TAG_PUT])?;
                    write_str(&mut f, tn)?;
                    write_str(&mut f, key)?;
                    write_str(&mut f, val)?;
                    write_u32(&mut f, a.shard())?;
                    write_u64(&mut f, a.khid().raw())?;
                }
            }
            f.sync_data()?;
        }
        fs::rename(&tmp, self.dir.join("meta"))
    }
}

fn write_u32<W: Write>(w: &mut W, n: u32) -> io::Result<()> {
    w.write_all(&[n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8])
}

fn write_u64<W: Write>(w: &mut W, n: u64) -> io::Result<()> {
    w.write_all(&[n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8,
                  (n >> 32) as u8, (n >> 40) as u8, (n >> 48) as u8, (n >> 56) as u8])
}

fn read_u32<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(b[0] as u32 | ((b[1] as u32) << 8) | ((b[2] as u32) << 16) | ((b[3] as u32) << 24))
}

fn read_u64<R: Read>(r: &mut R) -> io::Result<u64> {
    let mut b = [0u8; 8];
    r.read_exact(&mut b)?;
    Ok(b[0] as u64 | ((b[1] as u64) << 8) | ((b[2] as u64) << 16) | ((b[3] as u64) << 24)
       | ((b[4] as u64) << 32) | ((b[5] as u64) << 40) | ((b[6] as u64) << 48) | ((b[7] as u64) << 56))
}

fn write_str<W: Write>(w: &mut W, s: &str) -> io::Result<()> {
    write_u32(w, s.len() as u32)?;
    w.write_all(s.as_bytes())
}

fn read_str<R: Read>(r: &mut R) -> io::Result<String> {
    let n = read_u32(r)? as usize;
    let mut b = vec![0u8; n];
    r.read_exact(&mut b)?;
    String::from_utf8(b).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "utf8"))
}

/// Copy primary meta onto a replica directory.
pub fn catch_up(dir: &Path, from: &Path) -> io::Result<()> {
    let src = from.join("meta");
    if !src.exists() {
        return Ok(());
    }
    fs::copy(&src, dir.join("meta"))?;
    Ok(())
}

/// Open a meta file without the WAL.
pub fn load(dir: &Path) -> io::Result<Meta> {
    Meta::open(dir)
}
