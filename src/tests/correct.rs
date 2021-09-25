//! Combinations the kernel already claimed to do.

use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use crate::Store;
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khl1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn remove_vertex_clears_the_index() {
    let mut g = crate::Graph::new();
    g.create_index("Doc", "name");
    let id = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert_eq!(g.find("Doc", "name", "Ada"), vec![id]);
    assert!(g.remove_vertex(id));
    assert!(g.find("Doc", "name", "Ada").is_empty());
    assert!(g.vertex(id).is_none());
}

#[test]
fn deleted_vertex_does_not_return_on_reopen() {
    let dir = tmp("drop-v");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    let id = s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.graph_mut().unwrap().remove_vertex(id);
    s.commit().unwrap();
    drop(s);
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex_by_name("Ada").is_none());
    assert!(s.graph().find("Doc", "name", "Ada").is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn torn_tail_is_truncated_then_writes_survive() {
    let dir = tmp("trunc");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    drop(s);
    {
        let mut f = OpenOptions::new().write(true).open(dir.join("log")).unwrap();
        f.seek(SeekFrom::End(0)).unwrap();
        f.write_all(&[9, 9, 9, 9, 9, 9, 9, 9]).unwrap();
    }
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex_by_name("Ada").is_some());
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    s.commit().unwrap();
    drop(s);
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex_by_name("Ada").is_some());
    assert!(s.graph().vertex_by_name("Bob").is_some());
    let _ = fs::remove_dir_all(&dir);
}