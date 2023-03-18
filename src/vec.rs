//! Embedding posting. Droppable. Cosine is a scan.
//! Serials do not overwrite. Compact drops orphans.

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use super::graph::Graph;
use super::khid::Khid;
use super::wal::Rec;

pub fn dir(store: &Path) -> std::path::PathBuf {
    store.join("vec")
}

fn path(store: &Path, id: Khid, serial: u64) -> std::path::PathBuf {
    dir(store).join(format!("{:x}-{}", id.raw(), serial))
}

pub fn put(store: &Path, id: Khid, serial: u64, v: &[f32]) -> io::Result<()> {
    let d = dir(store);
    fs::create_dir_all(&d)?;
    let dest = path(store, id, serial);
    let tmp = d.join(format!("{:x}-{}.tmp", id.raw(), serial));
    {
        let mut f = OpenOptions::new().create(true).write(true).truncate(true).open(&tmp)?;
        let n = v.len() as u32;
        f.write_all(&n.to_le_bytes())?;
        for x in v.iter() {
            f.write_all(&x.to_le_bytes())?;
        }
        f.sync_data()?;
    }
    fs::rename(&tmp, &dest)?;
    let f = File::open(&dest)?;
    f.sync_data()?;
    Ok(())
}

pub fn get(store: &Path, id: Khid, serial: u64) -> io::Result<Option<Vec<f32>>> {
    let p = path(store, id, serial);
    if !p.exists() {
        return Ok(None);
    }
    let mut f = File::open(&p)?;
    let mut hdr = [0u8; 4];
    f.read_exact(&mut hdr)?;
    let n = u32::from_le_bytes(hdr) as usize;
    let mut out = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let mut b = [0u8; 4];
        f.read_exact(&mut b)?;
        out.push(f32::from_le_bytes(b));
        i += 1;
    }
    Ok(Some(out))
}

pub fn sync_dir(store: &Path) -> io::Result<()> {
    let d = dir(store);
    if !d.exists() {
        return Ok(());
    }
    let f = File::open(&d)?;
    f.sync_data()
}

pub fn max_serial(store: &Path) -> u64 {
    let d = dir(store);
    if !d.exists() {
        return 0;
    }
    let mut max = 0u64;
    if let Ok(rd) = fs::read_dir(&d) {
        for e in rd {
            if let Ok(e) = e {
                let name = e.file_name().to_string_lossy().into_owned();
                if let Some((_, n)) = parse_id_serial(&name) {
                    if n > max {
                        max = n;
                    }
                }
            }
        }
    }
    max
}

fn parse_id_serial(name: &str) -> Option<(u64, u64)> {
    if name.ends_with(".tmp") {
        return None;
    }
    let mut it = name.rsplitn(2, '-');
    let ser = it.next()?.parse().ok()?;
    let id = u64::from_str_radix(it.next()?, 16).ok()?;
    Some((id, ser))
}

pub fn fill(store: &Path, g: &mut Graph, recs: &[Rec]) -> io::Result<()> {
    g.quiet();
    for rec in recs.iter() {
        if let Rec::Emb { id, ref key, serial, .. } = *rec {
            match get(store, id, serial) {
                Ok(Some(v)) => {
                    g.set_vec(id, key, &v);
                }
                Ok(None) => {}
                Err(e) => {
                    g.live();
                    return Err(e);
                }
            }
        }
    }
    g.live();
    Ok(())
}

pub fn gc(store: &Path, live: &HashSet<(u64, u64)>) -> io::Result<usize> {
    let d = dir(store);
    if !d.exists() {
        return Ok(0);
    }
    let mut n = 0usize;
    for e in fs::read_dir(&d)? {
        let e = e?;
        let name = e.file_name().to_string_lossy().into_owned();
        if let Some(pair) = parse_id_serial(&name) {
            if !live.contains(&pair) {
                let _ = fs::remove_file(e.path());
                n += 1;
            }
        }
    }
    Ok(n)
}

pub fn copy_all(from: &Path, to: &Path) -> io::Result<()> {
    let src = dir(from);
    if !src.exists() {
        return Ok(());
    }
    let dst = dir(to);
    fs::create_dir_all(&dst)?;
    for e in fs::read_dir(&src)? {
        let e = e?;
        let name = e.file_name();
        if name.to_string_lossy().ends_with(".tmp") {
            continue;
        }
        fs::copy(e.path(), dst.join(name))?;
    }
    sync_dir(to)
}

pub fn live_from(recs: &[Rec]) -> HashSet<(u64, u64)> {
    let mut s = HashSet::new();
    for rec in recs.iter() {
        if let Rec::Emb { id, serial, .. } = *rec {
            s.insert((id.raw(), serial));
        }
    }
    s
}

pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    let mut i = 0;
    while i < a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
        i += 1;
    }
    let d = na.sqrt() * nb.sqrt();
    if d == 0.0 {
        0.0
    } else {
        dot / d
    }
}
