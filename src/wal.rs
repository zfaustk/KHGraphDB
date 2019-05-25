//! The log on a shard. KHL1. Begin / Commit wrap
//! puts. Uncommitted records do not replay.
//! KHID is a raw u64 here. The letters are Display.

use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{Read, Write, Result, Error, ErrorKind};

use super::graph::Graph;
use super::khid::Khid;
use super::prop::Prop;
use super::addr::Addr;

const MAGIC: &'static [u8] = b"KHL1";

const TAG_BEGIN: u8 = 1;
const TAG_COMMIT: u8 = 2;
const TAG_VERTEX: u8 = 3;
const TAG_EDGE: u8 = 4;
const TAG_FAR: u8 = 5;
const TAG_INDEX: u8 = 6;
const TAG_CONTENT: u8 = 7;
const TAG_TERM: u8 = 8;

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
    Term {
        tx: u64,
        term: u64,
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
            Rec::Term { tx, .. } => tx,
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
        Rec::Term { tx, term } => {
            w.write_all(&[TAG_TERM])?;
            write_u64(w, tx)?;
            write_u64(w, term)
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
        TAG_TERM => {
            let term = read_u64(r)?;
            Ok(Rec::Term { tx: tx, term: term })
        }
        _ => Err(Error::new(ErrorKind::InvalidData, "rec tag")),
    }
}

/// Write a shard header and records. Caller fsyncs.
pub fn write<W: Write>(shard: u32, recs: &[Rec], w: &mut W) -> Result<()> {
    write_header(shard, w)?;
    append(recs, w)
}

pub fn write_header<W: Write>(shard: u32, w: &mut W) -> Result<()> {
    w.write_all(MAGIC)?;
    write_u32(w, shard)
}

pub fn append<W: Write>(recs: &[Rec], w: &mut W) -> Result<()> {
    for rec in recs.iter() {
        write_rec(w, rec)?;
    }
    Ok(())
}

/// Read header and records to EOF.
pub fn read<R: Read>(r: &mut R) -> Result<(u32, Vec<Rec>)> {
    let mut magic = [0u8; 4];
    read_exact(r, &mut magic)?;
    if &magic != MAGIC {
        return Err(Error::new(ErrorKind::InvalidData, "not KHL1"));
    }
    let shard = read_u32(r)?;
    let mut recs = Vec::new();
    loop {
        match read_rec(r) {
            Ok(rec) => recs.push(rec),
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }
    Ok((shard, recs))
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
    for rec in recs.iter() {
        if !committed.contains(&rec.tx()) {
            continue;
        }
        match *rec {
            Rec::Begin { .. } | Rec::Commit { .. } | Rec::Term { .. } => {}
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
        }
    }
    Ok(g)
}

/// Read a log and replay it.
pub fn recover<R: Read>(r: &mut R) -> Result<Graph> {
    let (shard, recs) = read(r)?;
    match replay(shard, &recs) {
        Ok(g) => Ok(g),
        Err(e) => Err(Error::new(ErrorKind::InvalidData, e.message())),
    }
}
