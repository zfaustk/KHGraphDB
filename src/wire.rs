//! Bytes on the socket are the log. Pull is
//! (generation, offset). Hydrate is one round
//! of Addr → Stub. Commit does not wait.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream, SocketAddr, ToSocketAddrs};
use std::path::Path;

use super::addr::Addr;
use super::graph::Graph;
use super::khid::Khid;
use super::pos::Pos;
use super::stub::Stub;
use super::wal;

const TAG_PULL: u8 = 1;
const TAG_HYDRATE: u8 = 2;
const TAG_FIND: u8 = 3;
const KIND_TAIL: u8 = 0;
const KIND_SNAP: u8 = 1;

fn write_u32<W: Write>(w: &mut W, n: u32) -> io::Result<()> {
    let b = [n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8];
    w.write_all(&b)
}

fn read_u32<R: Read>(r: &mut R) -> io::Result<u32> {
    let mut b = [0u8; 4];
    r.read_exact(&mut b)?;
    Ok(b[0] as u32 | ((b[1] as u32) << 8) | ((b[2] as u32) << 16) | ((b[3] as u32) << 24))
}

fn write_u64<W: Write>(w: &mut W, n: u64) -> io::Result<()> {
    let b = [n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8,
             (n >> 32) as u8, (n >> 40) as u8, (n >> 48) as u8, (n >> 56) as u8];
    w.write_all(&b)
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

/// Bind. The kernel does not pick a port for you.
pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<TcpListener> {
    TcpListener::bind(addr)
}

/// One request on an accepted stream. The graph is
/// a snapshot for hydrate; the dir holds the log.
pub fn handle(dir: &Path, g: &Graph, mut s: TcpStream) -> io::Result<()> {
    let mut tag = [0u8; 1];
    s.read_exact(&mut tag)?;
    match tag[0] {
        TAG_PULL => reply_pull(dir, &mut s),
        TAG_HYDRATE => reply_hydrate(g, &mut s),
        TAG_FIND => reply_find(g, &mut s),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "wire tag")),
    }
}

fn reply_pull(dir: &Path, s: &mut TcpStream) -> io::Result<()> {
    let want_gen = read_u32(s)?;
    let want_off = read_u64(s)?;
    let path = dir.join("log");
    let mut f = File::open(&path)?;
    let h = wal::head(&mut f)?;
    let gen = if h.generation == 0 { 1 } else { h.generation };
    let len = fs::metadata(&path)?.len();
    if gen != want_gen || want_off > len {
        s.write_all(&[KIND_SNAP])?;
        write_u32(s, gen)?;
        write_u64(s, len)?;
        f.seek(SeekFrom::Start(0))?;
        io::copy(&mut f, s)?;
        return Ok(());
    }
    s.write_all(&[KIND_TAIL])?;
    let n = len - want_off;
    write_u32(s, n as u32)?;
    if n > 0 {
        f.seek(SeekFrom::Start(want_off))?;
        io::copy(&mut f, s)?;
    }
    Ok(())
}

fn reply_hydrate(g: &Graph, s: &mut TcpStream) -> io::Result<()> {
    let n = read_u32(s)? as usize;
    let mut addrs = Vec::new();
    let mut i = 0;
    while i < n {
        let shard = read_u32(s)?;
        let raw = read_u64(s)?;
        addrs.push(Addr::new(shard, Khid::from_raw(raw)));
        i += 1;
    }
    write_u32(s, n as u32)?;
    for addr in addrs.iter() {
        match stub_at(g, *addr) {
            Some(st) => {
                s.write_all(&[1])?;
                write_str(s, st.title())?;
                write_u64(s, st.ver())?;
            }
            None => {
                s.write_all(&[0])?;
            }
        }
    }
    Ok(())
}

fn reply_find(g: &Graph, s: &mut TcpStream) -> io::Result<()> {
    let tn = read_str(s)?;
    let key = read_str(s)?;
    let val = read_str(s)?;
    let ids = g.find(&tn, &key, &val);
    write_u32(s, ids.len() as u32)?;
    for id in ids.iter() {
        write_u32(s, g.shard())?;
        write_u64(s, id.raw())?;
    }
    Ok(())
}

fn stub_at(g: &Graph, addr: Addr) -> Option<Stub> {
    if addr.shard() != g.shard() {
        return g.stub(addr).cloned();
    }
    match g.vertex(addr.khid()) {
        Some(v) => {
            let title = match v.get("title").or(v.get("name")) {
                Some(s) => s,
                None => "",
            };
            Some(Stub::new(title, 1))
        }
        None => None,
    }
}

/// Ask a primary for bytes from `have`. Writes the
/// log file at `dest`. Returns the primary's Pos.
pub fn pull(addr: SocketAddr, have: Pos, dest: &Path) -> io::Result<Pos> {
    let mut s = TcpStream::connect(addr)?;
    s.write_all(&[TAG_PULL])?;
    write_u32(&mut s, have.generation())?;
    write_u64(&mut s, have.offset())?;
    let mut kind = [0u8; 1];
    s.read_exact(&mut kind)?;
    match kind[0] {
        KIND_TAIL => {
            let n = read_u32(&mut s)? as u64;
            if n > 0 {
                let mut f = OpenOptions::new().write(true).open(dest)?;
                f.seek(SeekFrom::End(0))?;
                let mut take = s.take(n);
                io::copy(&mut take, &mut f)?;
                f.sync_data()?;
            }
            Ok(Pos::new(have.generation(), have.offset() + n))
        }
        KIND_SNAP => {
            let gen = read_u32(&mut s)?;
            let n = read_u64(&mut s)?;
            let tmp = dest.with_extension("pull");
            {
                let mut f = File::create(&tmp)?;
                let mut take = s.take(n);
                io::copy(&mut take, &mut f)?;
                f.sync_data()?;
            }
            fs::rename(&tmp, dest)?;
            Ok(Pos::new(gen, n))
        }
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "pull kind")),
    }
}

/// One round. Missing Addr is None.
pub fn get_stubs(addr: SocketAddr, addrs: &[Addr]) -> io::Result<Vec<Option<Stub>>> {
    let mut s = TcpStream::connect(addr)?;
    s.write_all(&[TAG_HYDRATE])?;
    write_u32(&mut s, addrs.len() as u32)?;
    for a in addrs.iter() {
        write_u32(&mut s, a.shard())?;
        write_u64(&mut s, a.khid().raw())?;
    }
    let n = read_u32(&mut s)? as usize;
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let mut p = [0u8; 1];
        s.read_exact(&mut p)?;
        if p[0] == 0 {
            out.push(None);
        } else {
            let title = read_str(&mut s)?;
            let ver = read_u64(&mut s)?;
            out.push(Some(Stub::new(&title, ver)));
        }
        i += 1;
    }
    Ok(out)
}

/// Locate on one home. Meta is that home's posting.
pub fn find(addr: SocketAddr, type_name: &str, key: &str, value: &str)
            -> io::Result<Vec<Addr>> {
    let mut s = TcpStream::connect(addr)?;
    s.write_all(&[TAG_FIND])?;
    write_str(&mut s, type_name)?;
    write_str(&mut s, key)?;
    write_str(&mut s, value)?;
    let n = read_u32(&mut s)? as usize;
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let shard = read_u32(&mut s)?;
        let raw = read_u64(&mut s)?;
        out.push(Addr::new(shard, Khid::from_raw(raw)));
        i += 1;
    }
    Ok(out)
}
