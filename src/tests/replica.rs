//! A copy of the log. Read-only until promote.

use std::fs;
use crate::Store;
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
        s.graph_mut().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
    }
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    assert!(r.is_replica());
    assert!(r.graph().vertex_by_name("Ada").is_some());
    assert!(r.commit().is_err());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn catch_up_sees_new_writes() {
    let prim = tmp("p-cu");
    let copy = tmp("r-cu");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    s.graph_mut().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
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
        s.graph_mut().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
    }
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    r.promote();
    assert!(!r.is_replica());
    r.graph_mut().add_vertex(attrs("Zed"), Some("Doc")).unwrap();
    r.commit().unwrap();
    assert!(r.graph().vertex_by_name("Zed").is_some());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}
