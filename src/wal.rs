//! The log on a shard. KHL1. Begin / Commit wrap
//! puts. Uncommitted records do not replay.
//! KHID is a raw u64 here. The letters are Display.

use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{Cursor, Read, Seek, SeekFrom, Write, Result, Error, ErrorKind};

use super::graph::Graph;
use super::khid::Khid;
use super::prop::Prop;
use super::addr::Addr;

const MAGIC: &'static [u8] = b"KHL1";
const MAGIC2: &'static [u8] = b"KHL2";
const MAGIC3: &'static [u8] = b"KHL3";

const TAG_BEGIN: u8 = 1;
const TAG_COMMIT: u8 = 2;
const TAG_VERTEX: u8 = 3;
const TAG_EDGE: u8 = 4;
const TAG_FAR: u8 = 5;
const TAG_INDEX: u8 = 6;
const TAG_CONTENT: u8 = 7;
const TAG_DROP_V: u8 = 8;
const TAG_DROP_E: u8 = 9;

/// One record. A tx is Begin, puts, Commit.
#[derive(Clone, Debug, PartialEq)]
pub enum Rec {
    Begin {
        tx: u64,
    },
    Commit {
        tx: u64,
    },
    Vertex {
        tx: u64,
        id: Khid,
        types: Vec<String>,
        attrs: HashMap<String, Prop>,
    },
    Edge {
        tx: u64,
        id: Khid,
        src: Khid,
        dst: Khid,
        ty: String,
        attrs: HashMap<String, Prop>,
    },
    FarEdge {
        tx: u64,
        id: Khid,
        src: Khid,
        dst: Addr,
        ty: String,
        attrs: HashMap<String, Prop>,
    },
    Index {
        tx: u64,
        type_name: String,
        key: String,
        unique: bool,
    },
    Content {
        tx: u64,
        type_name: String,
        key: String,
    },
    DropVertex {
        tx: u64,
        id: Khid,
    },
    DropEdge {
        tx: u64,
        id: Khid,
    },
}

impl Rec {
    pub fn tx(&self) -> u64 {
        match *self {
            Rec::Begin { tx } |
            Rec::Commit { tx } |
            Rec::Vertex { tx, .. } |
            Rec::Edge { tx, .. } |
            Rec::FarEdge { tx, .. } |
            Rec::Index { tx, .. } |
            Rec::Content { tx, .. } |
            Rec::DropVertex { tx, .. } |
            Rec::DropEdge { tx, .. } => tx,
        }
    }
}

fn write_u32<W: Write>(w: &mut W, n: u32) -> Result<()> {
    let b = [n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8];
    w.write_all(&b)
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32> {
    let mut b = [0u8; 4];
    read_exact(r, &mut b)?;
    Ok(b[0] as u32 | ((b[1] as u32) << 8) | ((b[2] as u32) << 16) | ((b[3] as u32) << 24))
}

fn write_u64<W: Write>(w: &mut W, n: u64) -> Result<()> {
    let b = [n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8,
             (n >> 32) as u8, (n >> 40) as u8, (n >> 48) as u8, (n >> 56) as u8];
    w.write_all(&b)
}

fn read_u64<R: Read>(r: &mut R) -> Result<u64> {
    let mut b = [0u8; 8];
    read_exact(r, &mut b)?;
    let mut u = 0u64;
    let mut i = 0;
    while i < 8 {
        u |= (b[i] as u64) << (8 * i);
        i += 1;
    }
    Ok(u)
}

fn read_exact<R: Read>(r: &mut R, buf: &mut [u8]) -> Result<()> {
    let mut got = 0;
    while got < buf.len() {
        match r.read(&mut buf[got..]) {
            Ok(0) => return Err(Error::new(ErrorKind::UnexpectedEof, "eof")),
            Ok(n) => got += n,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn write_str<W: Write>(w: &mut W, s: &str) -> Result<()> {
    let b = s.as_bytes();
    write_u32(w, b.len() as u32)?;
    w.write_all(b)
}

fn read_str<R: Read>(r: &mut R) -> Result<String> {
    let n = read_u32(r)? as usize;
    let mut b = vec![0u8; n];
    read_exact(r, &mut b)?;
    match String::from_utf8(b) {
        Ok(s) => Ok(s),
        Err(_) => Err(Error::new(ErrorKind::InvalidData, "utf8")),
    }
}

fn write_i64<W: Write>(w: &mut W, n: i64) -> Result<()> {
    write_u64(w, n as u64)
}

fn read_i64<R: Read>(r: &mut R) -> Result<i64> {
    Ok(read_u64(r)? as i64)
}

fn write_prop<W: Write>(w: &mut W, p: &Prop) -> Result<()> {
    w.write_all(&[p.tag()])?;
    match *p {
        Prop::Bool(true) => w.write_all(&[1]),
        Prop::Bool(false) => w.write_all(&[0]),
        Prop::Int(n) => write_i64(w, n),
        Prop::Float(n) => write_i64(w, n.to_bits() as i64),
        Prop::Str(ref s) => write_str(w, s),
    }
}

fn read_prop<R: Read>(r: &mut R) -> Result<Prop> {
    let mut tag = [0u8; 1];
    read_exact(r, &mut tag)?;
    match tag[0] {
        0 => {
            let mut b = [0u8; 1];
            read_exact(r, &mut b)?;
            Ok(Prop::from_bool(b[0] != 0))
        }
        1 => Ok(Prop::from_int(read_i64(r)?)),
        2 => {
            let bits = read_u64(r)?;
            Ok(Prop::from_float(f64::from_bits(bits)))
        }
        3 => Ok(Prop::from_str(&read_str(r)?)),
        _ => Err(Error::new(ErrorKind::InvalidData, "prop tag")),
    }
}

fn write_attrs<W: Write>(w: &mut W, attrs: &HashMap<String, Prop>) -> Result<()> {
    write_u32(w, attrs.len() as u32)?;
    for (k, v) in attrs.iter() {
        write_str(w, k)?;
        write_prop(w, v)?;
    }
    Ok(())
}

fn read_attrs<R: Read>(r: &mut R) -> Result<HashMap<String, Prop>> {
    let n = read_u32(r)? as usize;
    let mut m = HashMap::new();
    let mut i = 0;
    while i < n {
        let k = read_str(r)?;
        let v = read_prop(r)?;
        m.insert(k, v);
        i += 1;
    }
    Ok(m)
}

fn write_khid<W: Write>(w: &mut W, k: Khid) -> Result<()> {
    write_u64(w, k.raw())
}

fn read_khid<R: Read>(r: &mut R) -> Result<Khid> {
    Ok(Khid::from_raw(read_u64(r)?))
}

fn crc32(data: &[u8]) -> u32 {
    let mut c = 0xffffffffu32;
    for &b in data.iter() {
        c ^= b as u32;
        let mut i = 0;
        while i < 8 {
            if c & 1 != 0 {
                c = (c >> 1) ^ 0xedb88320;
            } else {
                c >>= 1;
            }
            i += 1;
        }
    }
    !c
}

fn write_framed<W: Write>(w: &mut W, rec: &Rec) -> Result<()> {
    let mut buf = Vec::new();
    write_rec(&mut buf, rec)?;
    write_u32(w, buf.len() as u32)?;
    write_u32(w, crc32(&buf))?;
    w.write_all(&buf)
}

fn read_framed<R: Read>(r: &mut R) -> Result<Option<Rec>> {
    let len = match read_u32(r) {
        Ok(n) => n as usize,
        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };
    let want = match read_u32(r) {
        Ok(n) => n,
        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    };
    let mut buf = vec![0u8; len];
    match read_exact(r, &mut buf) {
        Ok(()) => {}
        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    if crc32(&buf) != want {
        return Ok(None);
    }
    let rec = read_rec(&mut Cursor::new(buf))?;
    Ok(Some(rec))
}

fn write_rec<W: Write>(w: &mut W, rec: &Rec) -> Result<()> {
    match *rec {
        Rec::Begin { tx } => {
            w.write_all(&[TAG_BEGIN])?;
            write_u64(w, tx)
        }
        Rec::Commit { tx } => {
            w.write_all(&[TAG_COMMIT])?;
            write_u64(w, tx)
        }
        Rec::Vertex { tx, id, ref types, ref attrs } => {
            w.write_all(&[TAG_VERTEX])?;
            write_u64(w, tx)?;
            write_khid(w, id)?;
            write_u32(w, types.len() as u32)?;
            for t in types.iter() {
                write_str(w, t)?;
            }
            write_attrs(w, attrs)
        }
        Rec::Edge { tx, id, src, dst, ref ty, ref attrs } => {
            w.write_all(&[TAG_EDGE])?;
            write_u64(w, tx)?;
            write_khid(w, id)?;
            write_khid(w, src)?;
            write_khid(w, dst)?;
            write_str(w, ty)?;
            write_attrs(w, attrs)
        }
        Rec::FarEdge { tx, id, src, dst, ref ty, ref attrs } => {
            w.write_all(&[TAG_FAR])?;
            write_u64(w, tx)?;
            write_khid(w, id)?;
            write_khid(w, src)?;
            write_u32(w, dst.shard())?;
            write_khid(w, dst.khid())?;
            write_str(w, ty)?;
            write_attrs(w, attrs)
        }
        Rec::Index { tx, ref type_name, ref key, unique } => {
            w.write_all(&[TAG_INDEX])?;
            write_u64(w, tx)?;
            write_str(w, type_name)?;
            write_str(w, key)?;
            w.write_all(&[if unique { 1 } else { 0 }])
        }
        Rec::Content { tx, ref type_name, ref key } => {
            w.write_all(&[TAG_CONTENT])?;
            write_u64(w, tx)?;
            write_str(w, type_name)?;
            write_str(w, key)
        }
        Rec::DropVertex { tx, id } => {
            w.write_all(&[TAG_DROP_V])?;
            write_u64(w, tx)?;
            write_khid(w, id)
        }
        Rec::DropEdge { tx, id } => {
            w.write_all(&[TAG_DROP_E])?;
            write_u64(w, tx)?;
            write_khid(w, id)
        }
    }
}

fn read_rec<R: Read>(r: &mut R) -> Result<Rec> {
    let mut tag = [0u8; 1];
    read_exact(r, &mut tag)?;
    let tx = read_u64(r)?;
    match tag[0] {
        TAG_BEGIN => Ok(Rec::Begin { tx: tx }),
        TAG_COMMIT => Ok(Rec::Commit { tx: tx }),
        TAG_VERTEX => {
            let id = read_khid(r)?;
            let n = read_u32(r)? as usize;
            let mut types = Vec::new();
            let mut i = 0;
            while i < n {
                types.push(read_str(r)?);
                i += 1;
            }
            let attrs = read_attrs(r)?;
            Ok(Rec::Vertex {
                tx: tx,
                id: id,
                types: types,
                attrs: attrs,
            })
        }
        TAG_EDGE => {
            let id = read_khid(r)?;
            let src = read_khid(r)?;
            let dst = read_khid(r)?;
            let ty = read_str(r)?;
            let attrs = read_attrs(r)?;
            Ok(Rec::Edge {
                tx: tx,
                id: id,
                src: src,
                dst: dst,
                ty: ty,
                attrs: attrs,
            })
        }
        TAG_FAR => {
            let id = read_khid(r)?;
            let src = read_khid(r)?;
            let shard = read_u32(r)?;
            let khid = read_khid(r)?;
            let ty = read_str(r)?;
            let attrs = read_attrs(r)?;
            Ok(Rec::FarEdge {
                tx: tx,
                id: id,
                src: src,
                dst: Addr::new(shard, khid),
                ty: ty,
                attrs: attrs,
            })
        }
        TAG_INDEX => {
            let type_name = read_str(r)?;
            let key = read_str(r)?;
            let mut u = [0u8; 1];
            read_exact(r, &mut u)?;
            Ok(Rec::Index {
                tx: tx,
                type_name: type_name,
                key: key,
                unique: u[0] != 0,
            })
        }
        TAG_CONTENT => {
            let type_name = read_str(r)?;
            let key = read_str(r)?;
            Ok(Rec::Content {
                tx: tx,
                type_name: type_name,
                key: key,
            })
        }
        TAG_DROP_V => Ok(Rec::DropVertex {
            tx: tx,
            id: read_khid(r)?,
        }),
        TAG_DROP_E => Ok(Rec::DropEdge {
            tx: tx,
            id: read_khid(r)?,
        }),
        _ => Err(Error::new(ErrorKind::InvalidData, "rec tag")),
    }
}

/// Write a shard header and records. Caller fsyncs.
pub fn write<W: Write>(shard: u32, recs: &[Rec], w: &mut W) -> Result<()> {
    write_at(shard, 1, recs, w)
}

pub fn write_at<W: Write>(shard: u32, gen: u32, recs: &[Rec], w: &mut W) -> Result<()> {
    write_header(shard, gen, w)?;
    append(recs, w)
}

pub fn write_header<W: Write>(shard: u32, gen: u32, w: &mut W) -> Result<()> {
    w.write_all(MAGIC3)?;
    write_u32(w, shard)?;
    write_u32(w, gen)
}

/// Magic, shard, generation. Does not read records.
pub fn head<R: Read>(r: &mut R) -> Result<Head> {
    let mut magic = [0u8; 4];
    read_exact(r, &mut magic)?;
    let shard = read_u32(r)?;
    let generation = if &magic == MAGIC2 || &magic == MAGIC3 {
        read_u32(r)?
    } else if &magic == MAGIC {
        0
    } else {
        return Err(Error::new(ErrorKind::InvalidData, "not KHL1"));
    };
    Ok(Head { shard: shard, generation: generation })
}

pub fn append<W: Write>(recs: &[Rec], w: &mut W) -> Result<()> {
    for rec in recs.iter() {
        write_framed(w, rec)?;
    }
    Ok(())
}

/// Read header and records to EOF.
/// KHL3 frames with CRC. A bad frame is a torn tail
/// and is dropped. KHL2 / KHL1 have no CRC.
pub fn read<R: Read>(r: &mut R) -> Result<(u32, Vec<Rec>)> {
    let (h, recs) = read_at(r)?;
    Ok((h.shard, recs))
}

/// Header of a log. generation 0 is a KHL1 file.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Head {
    pub shard: u32,
    pub generation: u32,
}

pub fn read_at<R: Read>(r: &mut R) -> Result<(Head, Vec<Rec>)> {
    let mut magic = [0u8; 4];
    read_exact(r, &mut magic)?;
    let shard = read_u32(r)?;
    let generation = if &magic == MAGIC2 || &magic == MAGIC3 {
        read_u32(r)?
    } else if &magic == MAGIC {
        0
    } else {
        return Err(Error::new(ErrorKind::InvalidData, "not KHL1"));
    };
    let framed = &magic == MAGIC3;
    let mut recs = Vec::new();
    loop {
        if framed {
            match read_framed(r) {
                Ok(Some(rec)) => recs.push(rec),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        } else {
            match read_rec(r) {
                Ok(rec) => recs.push(rec),
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
        }
    }
    Ok((Head { shard: shard, generation: generation }, recs))
}

/// Like `read_at`, plus the byte offset of the last
/// good record. A torn frame is not part of `end`.
pub fn read_valid<R: Read + Seek>(r: &mut R) -> Result<(Head, Vec<Rec>, u64)> {
    let mut magic = [0u8; 4];
    read_exact(r, &mut magic)?;
    let shard = read_u32(r)?;
    let generation = if &magic == MAGIC2 || &magic == MAGIC3 {
        read_u32(r)?
    } else if &magic == MAGIC {
        0
    } else {
        return Err(Error::new(ErrorKind::InvalidData, "not KHL1"));
    };
    let framed = &magic == MAGIC3;
    let mut recs = Vec::new();
    let mut end = r.seek(SeekFrom::Current(0))?;
    loop {
        let pos = r.seek(SeekFrom::Current(0))?;
        if framed {
            match read_framed(r) {
                Ok(Some(rec)) => {
                    recs.push(rec);
                    end = r.seek(SeekFrom::Current(0))?;
                }
                Ok(None) => {
                    r.seek(SeekFrom::Start(pos))?;
                    break;
                }
                Err(e) => return Err(e),
            }
        } else {
            match read_rec(r) {
                Ok(rec) => {
                    recs.push(rec);
                    end = r.seek(SeekFrom::Current(0))?;
                }
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                    r.seek(SeekFrom::Start(pos))?;
                    break;
                }
                Err(e) => return Err(e),
            }
        }
    }
    Ok((Head { shard: shard, generation: generation }, recs, end))
}

/// Records whose bytes lie at or before `end`.
pub fn read_prefix<R: Read + Seek>(r: &mut R, end: u64) -> Result<(Head, Vec<Rec>)> {
    let mut magic = [0u8; 4];
    read_exact(r, &mut magic)?;
    let shard = read_u32(r)?;
    let generation = if &magic == MAGIC2 || &magic == MAGIC3 {
        read_u32(r)?
    } else if &magic == MAGIC {
        0
    } else {
        return Err(Error::new(ErrorKind::InvalidData, "not KHL1"));
    };
    let framed = &magic == MAGIC3;
    let mut recs = Vec::new();
    loop {
        let pos = r.seek(SeekFrom::Current(0))?;
        if pos >= end {
            break;
        }
        if framed {
            match read_framed(r) {
                Ok(Some(rec)) => recs.push(rec),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        } else {
            match read_rec(r) {
                Ok(rec) => recs.push(rec),
                Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
        }
    }
    Ok((Head { shard: shard, generation: generation }, recs))
}

/// Replay committed records. Begin without Commit is dropped.
pub fn replay(shard: u32, recs: &[Rec]) -> super::error::Result<Graph> {
    let mut committed = HashSet::new();
    for rec in recs.iter() {
        if let Rec::Commit { tx } = *rec {
            committed.insert(tx);
        }
    }
    let mut g = Graph::on("g1", shard);
    g.quiet();
    for rec in recs.iter() {
        if !committed.contains(&rec.tx()) {
            continue;
        }
        match *rec {
            Rec::Begin { .. } | Rec::Commit { .. } => {}
            Rec::Vertex { id, ref types, ref attrs, .. } => {
                g.restore_vertex(id, attrs.clone(), types.clone())?;
            }
            Rec::Edge { id, src, dst, ref ty, ref attrs, .. } => {
                let tno = if ty.is_empty() {
                    None
                } else {
                    Some(ty.clone())
                };
                g.restore_edge(id, src, dst, tno, attrs.clone())?;
            }
            Rec::FarEdge { id, src, dst, ref ty, ref attrs, .. } => {
                let tno = if ty.is_empty() {
                    None
                } else {
                    Some(ty.clone())
                };
                g.restore_far_edge(id, src, dst, tno, attrs.clone())?;
            }
            Rec::Content { ref type_name, ref key, .. } => {
                g.mark_content(type_name, key);
            }
            Rec::Index { ref type_name, ref key, unique, .. } => {
                if unique {
                    let _ = g.create_unique(type_name, key);
                } else {
                    let _ = g.create_index(type_name, key);
                }
            }
            Rec::DropVertex { id, .. } => {
                g.remove_vertex(id);
            }
            Rec::DropEdge { id, .. } => {
                g.remove_edge(id);
            }
        }
    }
    g.live();
    Ok(g)
}

/// Read a log and replay it.
pub fn recover<R: Read>(r: &mut R) -> Result<Graph> {
    let (h, recs) = read_at(r)?;
    match replay(h.shard, &recs) {
        Ok(g) => Ok(g),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e.message())),
    }
}
