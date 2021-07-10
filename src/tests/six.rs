//! 6.0: prefix, lease, CRC tail, MATCH via meta.

use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use crate::{Catalog, Store, query};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khl1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn read_at_does_not_see_later_commit() {
    let dir = tmp("six-pos");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let bm = s.commit().unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let old = s.read_at(bm).unwrap();
    assert!(old.vertex_by_name("Ada").is_some());
    assert!(old.vertex_by_name("Bob").is_none());
    assert!(s.graph().vertex_by_name("Bob").is_some());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rollback_replays_the_log() {
    let dir = tmp("six-rb");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.begin().unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    s.rollback();
    assert!(s.graph().vertex_by_name("Ada").is_some());
    assert!(s.graph().vertex_by_name("Bob").is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn torn_tail_is_dropped() {
    let dir = tmp("six-crc");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    {
        let mut f = OpenOptions::new().write(true).open(dir.join("log")).unwrap();
        f.seek(SeekFrom::End(0)).unwrap();
        f.write_all(&[1, 2, 3, 4, 5]).unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex_by_name("Ada").is_some());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn second_writer_has_no_lease() {
    let dir = tmp("six-lease");
    let mut a = Store::open(&dir, "notes", 1).unwrap();
    a.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    a.commit().unwrap();
    let mut b = Store::open(&dir, "notes", 1).unwrap();
    assert!(b.graph_mut().is_err());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn match_runs_at_the_home_meta_names() {
    let mut cat = Catalog::new();
    cat.create("notes").unwrap();
    cat.create("other").unwrap();
    {
        let o = cat.graph_mut("other").unwrap();
        o.create_index("Doc", "name");
        o.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    }
    let r = query::run_located(&mut cat, "notes",
        "MATCH (a:Doc {name:'Ada'}) RETURN a");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
}