//! Immutable pages. A serial does not overwrite.
//! A pin names a file. Compact drops orphans.

use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

use super::graph::Graph;
use super::khid::Khid;
use super::prop::Prop;
use super::wal::Rec;

pub fn dir(store: &Path) -> std::path::PathBuf {
    store.join("blob")
}

fn path(store: &Path, id: Khid, serial: u64) -> std::path::PathBuf {
    dir(store).join(format!("{:x}-{}", id.raw(), serial))
}

/// Write then fsync the file. Caller dirsyncs.
pub fn put(store: &Path, id: Khid, serial: u64, bytes: &[u8]) -> io::Result<()> {
    let d = dir(store);
    fs::create_dir_all(&d)?;
    let dest = path(store, id, serial);
    let tmp = d.join(format!("{:x}-{}.tmp", id.raw(), serial));
    {
        let mut f = OpenOptions::new().create(true).write(true).truncate(true).open(&tmp)?;
        f.write_all(bytes)?;
        f.sync_data()?;
    }
    fs::rename(&tmp, &dest)?;
    let f = File::open(&dest)?;
    f.sync_data()?;
    Ok(())
}

pub fn get(store: &Path, id: Khid, serial: u64) -> io::Result<Option<Vec<u8>>> {
    let p = path(store, id, serial);
    if !p.exists() {
        return Ok(None);
    }
    let mut f = File::open(&p)?;
    let mut b = Vec::new();
    f.read_to_end(&mut b)?;
    Ok(Some(b))
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
                if let Some(n) = parse_serial(&e.file_name().to_string_lossy()) {
                    if n > max {
                        max = n;
                    }
                }
            }
        }
    }
    max
}

fn parse_serial(name: &str) -> Option<u64> {
    let mut it = name.rsplitn(2, '-');
    let ser = it.next()?;
    if ser.ends_with(".tmp") {
        return None;
    }
    ser.parse().ok()
}

/// Fill content keys from blob serials on committed
/// Vertex records. Old records have the page inline.
pub fn fill(store: &Path, g: &mut Graph, recs: &[Rec]) -> io::Result<()> {
    g.quiet();
    for rec in recs.iter() {
        if let Rec::Vertex { id, ref blobs, .. } = *rec {
            for &(ref k, serial) in blobs.iter() {
                match get(store, id, serial) {
                    Ok(Some(b)) => {
                        let s = match String::from_utf8(b) {
                            Ok(s) => s,
                            Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
                        };
                        let _ = g.set_prop(id, k, Prop::from_str(&s));
                    }
                    Ok(None) => {}
                    Err(e) => {
                        g.live();
                        return Err(e);
                    }
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
    let rd = fs::read_dir(&d)?;
    for e in rd {
        let e = e?;
        let name = e.file_name().to_string_lossy().into_owned();
        if let Some((id, ser)) = parse_id_serial(&name) {
            if !live.contains(&(id, ser)) {
                let _ = fs::remove_file(e.path());
                n += 1;
            }
        }
    }
    Ok(n)
}

fn parse_id_serial(name: &str) -> Option<(u64, u64)> {
    let mut it = name.rsplitn(2, '-');
    let ser = it.next()?.parse().ok()?;
    let id = u64::from_str_radix(it.next()?, 16).ok()?;
    Some((id, ser))
}

/// Copy every blob file. Replica catch_up.
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
        if let Rec::Vertex { id, ref blobs, .. } = *rec {
            for &(_, serial) in blobs.iter() {
                s.insert((id.raw(), serial));
            }
        }
    }
    s
}
