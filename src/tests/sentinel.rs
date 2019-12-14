//! One watcher. Missed beats promote a replica.

use std::fs;
use crate::{Sentinel, Store};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khl1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn missed_beats_promote() {
    let prim = tmp("p-sen");
    let copy = tmp("r-sen");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    let mut w = Sentinel::new(&prim, 2);
    assert!(!w.poll(&mut r));
    assert!(r.is_replica());
    assert!(w.poll(&mut r));
    assert!(!r.is_replica());
    r.graph_mut().unwrap().add_vertex(attrs("Zed"), Some("Doc")).unwrap();
    r.commit().unwrap();
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn a_beat_resets_miss() {
    let prim = tmp("p-beat");
    let copy = tmp("r-beat");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    let mut w = Sentinel::new(&prim, 2);
    assert!(!w.poll(&mut r));
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    s.commit().unwrap();
    assert!(!w.poll(&mut r));
    assert!(r.is_replica());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}
