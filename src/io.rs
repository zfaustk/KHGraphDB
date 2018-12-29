use std::collections::HashMap;
use std::io::{Read, Write, Result, Error, ErrorKind};

use super::graph::Graph;
use super::khid::Khid;
use super::prop::Prop;

const MAGIC: &'static [u8] = b"KHG2";
const MAGIC3: &'static [u8] = b"KHG3";
const MAGIC4: &'static [u8] = b"KHG4";

fn write_u32<W: Write>(w: &mut W, n: u32) -> Result<()> {
    let b = [n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8];
    w.write_all(&b)
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32> {
    let mut b = [0u8; 4];
    read_exact(r, &mut b)?;
    Ok(b[0] as u32 | ((b[1] as u32) << 8) | ((b[2] as u32) << 16) | ((b[3] as u32) << 24))
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
    let u = n as u64;
    let b = [u as u8, (u >> 8) as u8, (u >> 16) as u8, (u >> 24) as u8,
             (u >> 32) as u8, (u >> 40) as u8, (u >> 48) as u8, (u >> 56) as u8];
    w.write_all(&b)
}

fn read_i64<R: Read>(r: &mut R) -> Result<i64> {
    let mut b = [0u8; 8];
    read_exact(r, &mut b)?;
    let mut u = 0u64;
    let mut i = 0;
    while i < 8 {
        u |= (b[i] as u64) << (8 * i);
        i += 1;
    }
    Ok(u as i64)
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
            let bits = read_i64(r)? as u64;
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

fn read_attrs_tagged<R: Read>(r: &mut R) -> Result<HashMap<String, Prop>> {
    let n = read_u32(r)? as usize;
    let mut m = HashMap::new();
    for _ in 0..n {
        let k = read_str(r)?;
        let v = read_prop(r)?;
        m.insert(k, v);
    }
    Ok(m)
}

fn read_attrs_str<R: Read>(r: &mut R) -> Result<HashMap<String, Prop>> {
    let n = read_u32(r)? as usize;
    let mut m = HashMap::new();
    for _ in 0..n {
        let k = read_str(r)?;
        let v = read_str(r)?;
        m.insert(k, Prop::from_str(&v));
    }
    Ok(m)
}

fn read_khid<R: Read>(r: &mut R) -> Result<Khid> {
    let s = read_str(r)?;
    match Khid::parse(&s) {
        Some(k) => Ok(k),
        None => Err(Error::new(ErrorKind::InvalidData, "bad khid")),
    }
}

/// KHG4 snapshot. Attributes keep their tag.
/// KHG3 and KHG2 still read; their values become Str.
/// The letters on the wire are Display. The arena
/// takes Khid.
pub fn write_graph<W: Write>(g: &Graph, w: &mut W) -> Result<()> {
    w.write_all(MAGIC4)?;
    write_str(w, g.khid())?;

    let types = g.all_types();
    write_u32(w, types.len() as u32)?;
    for &(id, ref name) in types.iter() {
        write_str(w, &format!("{}", id))?;
        write_str(w, name)?;
    }

    let vids = g.vertex_ids();
    write_u32(w, vids.len() as u32)?;
    for vid in vids.iter() {
        write_str(w, &format!("{}", vid))?;
        let names = g.type_names_of_vertex(*vid);
        write_u32(w, names.len() as u32)?;
        for n in names.iter() {
            write_str(w, n)?;
        }
        match g.vertex(*vid) {
            Some(v) => write_attrs(w, v.attrs())?,
            None => write_u32(w, 0)?,
        }
    }

    let edges = g.all_edges();
    write_u32(w, edges.len() as u32)?;
    for &(id, src, dst, _) in edges.iter() {
        write_str(w, &format!("{}", id))?;
        write_str(w, &format!("{}", src))?;
        write_str(w, &format!("{}", dst))?;
        let tn = g.edge_type_name(id).unwrap_or(String::new());
        write_str(w, &tn)?;
        match g.edge(id) {
            Some(e) => write_attrs(w, e.attrs())?,
            None => write_u32(w, 0)?,
        }
    }
    Ok(())
}

pub fn read_graph<R: Read>(r: &mut R) -> Result<Graph> {
    let mut magic = [0u8; 4];
    read_exact(r, &mut magic)?;
    if &magic != MAGIC && &magic != MAGIC3 && &magic != MAGIC4 {
        return Err(Error::new(ErrorKind::InvalidData, "not KHG2"));
    }
    let tagged = &magic == MAGIC4;
    let v3 = &magic == MAGIC3 || tagged;
    let gid = read_str(r)?;
    let mut g = if gid.is_empty() {
        Graph::new()
    } else {
        Graph::named(&gid)
    };

    let n_t = read_u32(r)? as usize;
    for _ in 0..n_t {
        let _id = read_str(r)?;
        let name = read_str(r)?;
        match g.add_type(&name) {
            Ok(_) => {}
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.message())),
        }
    }

    let n_v = read_u32(r)? as usize;
    for _ in 0..n_v {
        let id = read_khid(r)?;
        let n_names = read_u32(r)? as usize;
        let mut names = Vec::new();
        for _ in 0..n_names {
            names.push(read_str(r)?);
        }
        let attrs = if tagged {
            read_attrs_tagged(r)?
        } else {
            read_attrs_str(r)?
        };
        match g.restore_vertex(id, attrs, names) {
            Ok(_) => {}
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.message())),
        }
    }

    let n_e = read_u32(r)? as usize;
    for _ in 0..n_e {
        let id = read_khid(r)?;
        let src = read_khid(r)?;
        let dst = read_khid(r)?;
        let tn = read_str(r)?;
        let tno = if tn.is_empty() { None } else { Some(tn) };
        let attrs = if tagged {
            read_attrs_tagged(r)?
        } else if v3 {
            read_attrs_str(r)?
        } else {
            HashMap::new()
        };
        match g.restore_edge(id, src, dst, tno, attrs) {
            Ok(_) => {}
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.message())),
        }
    }
    Ok(g)
}
