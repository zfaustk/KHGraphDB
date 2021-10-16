//! Clocks, not a 200-chain. Tiny N so cargo test
//! stays a unit test. The example is the bench.

use std::fs;
use crate::Store;
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khl1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

fn pct(xs: &mut [u64], p: u32) -> u64 {
    xs.sort();
    let i = ((p as u64) * (xs.len() as u64 - 1)) / 100;
    xs[i as usize]
}

#[test]
fn histogram_order() {
    let mut xs = [10u64, 30, 20, 40, 50];
    assert_eq!(pct(&mut xs, 0), 10);
    assert_eq!(pct(&mut xs, 50), 30);
    assert_eq!(pct(&mut xs, 100), 50);
}

#[test]
fn one_put_does_not_rewrite_the_arena() {
    let dir = tmp("p-delta");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    let mut i = 0;
    while i < 80 {
        s.graph_mut().unwrap().add_vertex(attrs(&format!("n{}", i)), Some("Doc")).unwrap();
        i += 1;
    }
    s.commit().unwrap();
    let before = fs::metadata(dir.join("log")).unwrap().len();
    s.graph_mut().unwrap().add_vertex(attrs("last"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let delta = fs::metadata(dir.join("log")).unwrap().len() - before;
    s.compact().unwrap();
    let full = fs::metadata(dir.join("log")).unwrap().len();
    assert!(delta < full);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn begin_does_not_need_a_clone_to_commit() {
    let dir = tmp("p-noclone");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.query("CREATE (a:Doc {name:'Ada'})");
    s.commit().unwrap();
    s.query("CREATE (b:Doc {name:'Bob'})");
    s.commit().unwrap();
    drop(s);
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex_by_name("Ada").is_some());
    assert!(s.graph().vertex_by_name("Bob").is_some());
    let _ = fs::remove_dir_all(&dir);
}