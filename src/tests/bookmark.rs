//! A bookmark is a Pos. Replica honors or fails.

use std::fs;
use crate::{Pos, Store};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khl1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn replica_honors_after_catch_up() {
    let prim = tmp("p-bm");
    let copy = tmp("r-bm");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    let bm = s.commit().unwrap();
    assert!(!r.pos().unwrap().honors(bm));
    r.honor(&prim, bm).unwrap();
    assert!(r.pos().unwrap().honors(bm));
    assert!(r.graph().vertex_by_name("Bob").is_some());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn primary_always_honors() {
    let dir = tmp("p-ok");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    let bm = s.commit().unwrap();
    s.honor(&dir, bm).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn honor_fails_if_primary_gone() {
    let prim = tmp("p-gone");
    let copy = tmp("r-gone");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    let bm = s.commit().unwrap();
    let _ = fs::remove_dir_all(&prim);
    assert!(r.honor(&prim, bm).is_err());
    assert!(r.is_replica());
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn older_generation_does_not_honor() {
    let a = Pos::new(1, 100);
    let b = Pos::new(2, 10);
    assert!(!a.honors(b));
    assert!(b.honors(a));
    assert!(Pos::new(1, 100).honors(Pos::new(1, 50)));
    assert!(!Pos::new(1, 50).honors(Pos::new(1, 100)));
}