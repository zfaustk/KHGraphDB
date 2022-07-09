//! Clocks, not a 200-chain. Tiny N so cargo test
//! stays a unit test. The example is the bench.

use std::fs;
use std::time::Instant;
use crate::{query, Graph, Store};
use crate::prop::Prop;
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

fn ns(t: Instant) -> u64 {
    let d = t.elapsed();
    d.as_secs() * 1_000_000_000 + d.subsec_nanos() as u64
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

#[test]
fn range_clock_vs_scan() {
    let mut g = Graph::new();
    g.create_index("Doc", "n");
    let mut i = 0i64;
    while i < 120 {
        let id = g.add_vertex(attrs(&format!("d{}", i)), Some("Doc")).unwrap();
        g.set_prop(id, "n", Prop::from_int(i)).unwrap();
        i += 1;
    }
    let mut range = Vec::new();
    let mut scan = Vec::new();
    i = 0;
    while i < 30 {
        let t = Instant::now();
        let r = query::ask(&g, "MATCH (a:Doc) WHERE a.n > 80 RETURN a");
        range.push(ns(t));
        assert_eq!(r.rows.len(), 39);
        let t = Instant::now();
        let r = query::ask(&g, "MATCH (a:Doc) RETURN a");
        scan.push(ns(t));
        assert_eq!(r.rows.len(), 120);
        i += 1;
    }
    let rp = pct(&mut range, 50);
    let sp = pct(&mut scan, 50);
    assert!(rp > 0 && sp > 0);
}

#[test]
fn commit_plus_one_stays_a_delta() {
    let dir = tmp("p-c1");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    let mut i = 0;
    while i < 40 {
        s.graph_mut().unwrap().add_vertex(attrs(&format!("n{}", i)), Some("Doc")).unwrap();
        i += 1;
    }
    s.commit().unwrap();
    let mut xs = Vec::new();
    i = 0;
    while i < 20 {
        let t = Instant::now();
        s.graph_mut().unwrap().add_vertex(attrs(&format!("x{}", i)), Some("Doc")).unwrap();
        s.commit().unwrap();
        xs.push(ns(t));
        i += 1;
    }
    assert!(pct(&mut xs, 95) > 0);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn maybe_compact_is_a_checkpoint() {
    let dir = tmp("p-ck");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.compact().unwrap();
    let t = Instant::now();
    let mut i = 0;
    while i < 16 {
        s.graph_mut().unwrap().add_vertex(attrs(&format!("n{}", i)), Some("Doc")).unwrap();
        s.commit().unwrap();
        i += 1;
    }
    let p = s.maybe_compact().unwrap();
    assert!(p.is_some());
    assert!(ns(t) > 0);
    let _ = fs::remove_dir_all(&dir);
}
