use std::collections::HashMap;
use std::io::{Read, Write, Result, Error, ErrorKind};

use super::graph::Graph;

const MAGIC: &'static [u8] = b"KHG2";

fn write_u32<W: Write>(w: &mut W, n: u32) -> Result<()> {
    let b = [n as u8, (n >> 8) as u8, (n >> 16) as u8, (n >> 24) as u8];
    w.write_all(&b)
}

fn read_u32<R: Read>(r: &mut R) -> Result<u32> {
    let mut b = [0u8; 4];
    try!(read_exact(r, &mut b));
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
    try!(write_u32(w, b.len() as u32));
    w.write_all(b)
}

fn read_str<R: Read>(r: &mut R) -> Result<String> {
    let n = try!(read_u32(r)) as usize;
    let mut b = vec![0u8; n];
    try!(read_exact(r, &mut b));
    match String::from_utf8(b) {
        Ok(s) => Ok(s),
        Err(_) => Err(Error::new(ErrorKind::InvalidData, "utf8")),
    }
}

fn write_attrs<W: Write>(w: &mut W, attrs: &HashMap<String, String>) -> Result<()> {
    try!(write_u32(w, attrs.len() as u32));
    for (k, v) in attrs.iter() {
        try!(write_str(w, k));
        try!(write_str(w, v));
    }
    Ok(())
}

fn read_attrs<R: Read>(r: &mut R) -> Result<HashMap<String, String>> {
    let n = try!(read_u32(r)) as usize;
    let mut m = HashMap::new();
    for _ in 0..n {
        let k = try!(read_str(r));
        let v = try!(read_str(r));
        m.insert(k, v);
    }
    Ok(m)
}

/// KHG2 snapshot. View state stays out.
pub fn write_graph<W: Write>(g: &Graph, w: &mut W) -> Result<()> {
    try!(w.write_all(MAGIC));
    try!(write_str(w, g.khid()));

    let types = g.all_types();
    try!(write_u32(w, types.len() as u32));
    for &(ref id, ref name) in types.iter() {
        try!(write_str(w, id));
        try!(write_str(w, name));
    }

    let vids = g.vertex_ids();
    try!(write_u32(w, vids.len() as u32));
    for vid in vids.iter() {
        try!(write_str(w, vid));
        let names = g.type_names_of_vertex(vid);
        try!(write_u32(w, names.len() as u32));
        for n in names.iter() {
            try!(write_str(w, n));
        }
        match g.vertex(vid) {
            Some(v) => try!(write_attrs(w, v.attrs())),
            None => try!(write_u32(w, 0)),
        }
    }

    let edges = g.all_edges();
    try!(write_u32(w, edges.len() as u32));
    for &(ref id, ref src, ref dst, _) in edges.iter() {
        try!(write_str(w, id));
        try!(write_str(w, src));
        try!(write_str(w, dst));
        let tn = g.edge_type_name(id).unwrap_or(String::new());
        try!(write_str(w, &tn));
    }
    Ok(())
}

pub fn read_graph<R: Read>(r: &mut R) -> Result<Graph> {
    let mut magic = [0u8; 4];
    try!(read_exact(r, &mut magic));
    if &magic != MAGIC {
        return Err(Error::new(ErrorKind::InvalidData, "not KHG2"));
    }
    let _gid = try!(read_str(r));
    let mut g = Graph::new();

    let n_t = try!(read_u32(r)) as usize;
    for _ in 0..n_t {
        let _id = try!(read_str(r));
        let name = try!(read_str(r));
        match g.add_type(&name) {
            Ok(_) => {}
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.message())),
        }
    }

    let n_v = try!(read_u32(r)) as usize;
    for _ in 0..n_v {
        let id = try!(read_str(r));
        let n_names = try!(read_u32(r)) as usize;
        let mut names = Vec::new();
        for _ in 0..n_names {
            names.push(try!(read_str(r)));
        }
        let attrs = try!(read_attrs(r));
        match g.restore_vertex(id, attrs, names) {
            Ok(_) => {}
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.message())),
        }
    }

    let n_e = try!(read_u32(r)) as usize;
    for _ in 0..n_e {
        let id = try!(read_str(r));
        let src = try!(read_str(r));
        let dst = try!(read_str(r));
        let tn = try!(read_str(r));
        let tno = if tn.is_empty() { None } else { Some(tn) };
        match g.restore_edge(id, src, dst, tno) {
            Ok(_) => {}
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.message())),
        }
    }
    Ok(g)
}
