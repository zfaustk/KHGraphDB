//! A copy of the log. Read-only until promote.

use std::fs;
use crate::{Role, Store};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khl1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn replica_is_read_only() {
    let prim = tmp("p-ro");
    let copy = tmp("r-ro");
    {
        let mut s = Store::open(&prim, "notes", 1).unwrap();
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
    }
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    assert!(r.is_replica());
    assert_eq!(r.role(), Role::Replica);
    assert!(r.graph().vertex_by_name("Ada").is_some());
    assert!(r.graph_mut().is_err());
    assert!(r.commit().is_err());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn catch_up_sees_new_writes() {
    let prim = tmp("p-cu");
    let copy = tmp("r-cu");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    s.commit().unwrap();
    assert!(r.graph().vertex_by_name("Bob").is_none());
    r.catch_up(&prim).unwrap();
    assert!(r.graph().vertex_by_name("Bob").is_some());
    assert!(r.is_replica());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn promote_can_write() {
    let prim = tmp("p-pr");
    let copy = tmp("r-pr");
    {
        let mut s = Store::open(&prim, "notes", 1).unwrap();
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
    }
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    r.promote();
    assert!(!r.is_replica());
    assert_eq!(r.role(), Role::Primary);
    r.graph_mut().unwrap().add_vertex(attrs("Zed"), Some("Doc")).unwrap();
    r.commit().unwrap();
    assert!(r.graph().vertex_by_name("Zed").is_some());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn catch_up_appends_new_bytes() {
    let prim = tmp("p-ap");
    let copy = tmp("r-ap");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    let before = fs::metadata(copy.join("log")).unwrap().len();
    r.catch_up(&prim).unwrap();
    assert_eq!(fs::metadata(copy.join("log")).unwrap().len(), before);
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let prim_len = fs::metadata(prim.join("log")).unwrap().len();
    r.catch_up(&prim).unwrap();
    assert_eq!(fs::metadata(copy.join("log")).unwrap().len(), prim_len);
    assert!(r.graph().vertex_by_name("Bob").is_some());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn catch_up_after_compact() {
    let prim = tmp("p-co");
    let copy = tmp("r-co");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    s.compact().unwrap();
    r.catch_up(&prim).unwrap();
    assert!(r.graph().vertex_by_name("Ada").is_some());
    assert!(r.graph().vertex_by_name("Bob").is_some());
    assert!(r.is_replica());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}
