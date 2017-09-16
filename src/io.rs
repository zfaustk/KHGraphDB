use std::collections::HashMap;
use std::io::{Read, Write, Result, Error, ErrorKind};

use super::graph::Graph;

const MAGIC: &'static [u8] = b"KHG2";
const MAGIC3: &'static [u8] = b"KHG3";

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

fn write_attrs<W: Write>(w: &mut W, attrs: &HashMap<String, String>) -> Result<()> {
    write_u32(w, attrs.len() as u32)?;
    for (k, v) in attrs.iter() {
        write_str(w, k)?;
        write_str(w, v)?;
    }
    Ok(())
}

fn read_attrs<R: Read>(r: &mut R) -> Result<HashMap<String, String>> {
    let n = read_u32(r)? as usize;
    let mut m = HashMap::new();
    for _ in 0..n {
        let k = read_str(r)?;
        let v = read_str(r)?;
        m.insert(k, v);
    }
    Ok(m)
}

/// KHG3 snapshot. Edge attributes travel with the hop.
/// KHG2 still reads.
pub fn write_graph<W: Write>(g: &Graph, w: &mut W) -> Result<()> {
    w.write_all(MAGIC3)?;
    write_str(w, g.khid())?;

    let types = g.all_types();
    write_u32(w, types.len() as u32)?;
    for &(ref id, ref name) in types.iter() {
        write_str(w, id)?;
        write_str(w, name)?;
    }

    let vids = g.vertex_ids();
    write_u32(w, vids.len() as u32)?;
    for vid in vids.iter() {
        write_str(w, vid)?;
        let names = g.type_names_of_vertex(vid);
        write_u32(w, names.len() as u32)?;
        for n in names.iter() {
            write_str(w, n)?;
        }
        match g.vertex(vid) {
            Some(v) => write_attrs(w, v.attrs())?,
            None => write_u32(w, 0)?,
        }
    }

    let edges = g.all_edges();
    write_u32(w, edges.len() as u32)?;
    for &(ref id, ref src, ref dst, _) in edges.iter() {
        write_str(w, id)?;
        write_str(w, src)?;
        write_str(w, dst)?;
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
    if &magic != MAGIC && &magic != MAGIC3 {
        return Err(Error::new(ErrorKind::InvalidData, "not KHG2"));
    }
    let v3 = &magic == MAGIC3;
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
        let id = read_str(r)?;
        let n_names = read_u32(r)? as usize;
        let mut names = Vec::new();
        for _ in 0..n_names {
            names.push(read_str(r)?);
        }
        let attrs = read_attrs(r)?;
        match g.restore_vertex(id, attrs, names) {
            Ok(_) => {}
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.message())),
        }
    }

    let n_e = read_u32(r)? as usize;
    for _ in 0..n_e {
        let id = read_str(r)?;
        let src = read_str(r)?;
        let dst = read_str(r)?;
        let tn = read_str(r)?;
        let tno = if tn.is_empty() { None } else { Some(tn) };
        let attrs = if v3 {
            read_attrs(r)?
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
