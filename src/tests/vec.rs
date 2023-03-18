//! Embedding is a posting. Cosine is a scan.

use std::fs;
use crate::{query, Graph, Store};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khv-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn cosine_orders_the_type() {
    let mut g = Graph::new();
    g.mark_vector("Doc", "emb");
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let b = g.add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    let c = g.add_vertex(attrs("Carol"), Some("Doc")).unwrap();
    g.set_vec(a, "emb", &[1.0, 0.0, 0.0]);
    g.set_vec(b, "emb", &[0.9, 0.1, 0.0]);
    g.set_vec(c, "emb", &[0.0, 1.0, 0.0]);
    let hits = g.similar("Doc", "emb", &[1.0, 0.0, 0.0], 2);
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].0, a);
    assert_eq!(hits[1].0, b);
}

#[test]
fn index_refuses_a_vector_key() {
    let mut g = Graph::new();
    assert!(g.mark_vector("Doc", "emb"));
    assert!(!g.create_index("Doc", "emb"));
}

#[test]
fn similar_query_and_explain() {
    let mut g = Graph::new();
    g.mark_vector("Doc", "emb");
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    g.set_vec(a, "emb", &[1.0, 0.0]);
    let r = query::ask(&g, "SIMILAR (x:Doc) ON emb TO [1.0, 0.0] LIMIT 1 RETURN x");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    let e = query::ask(&g, "EXPLAIN SIMILAR (x:Doc) ON emb TO [1.0, 0.0]");
    assert!(e.ok);
    let mut cost = None;
    for row in e.rows.iter() {
        let slot = row[0].as_ref().and_then(|v| v.as_prop()).and_then(|p| p.as_str());
        if slot == Some("cost") {
            cost = row[1].as_ref().and_then(|v| v.as_prop()).and_then(|p| p.as_str()).map(|s| s.to_string());
        }
    }
    assert_eq!(cost.as_ref().map(|s| s.as_str()), Some("1"));
}

#[test]
fn floats_are_not_in_the_log() {
    let dir = tmp("vlog");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().mark_vector("Doc", "emb");
    let id = s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.graph_mut().unwrap().set_vec(id, "emb", &[0.125, 0.25, 0.5]);
    s.commit().unwrap();
    let log = fs::read(dir.join("log")).unwrap();
    let needle = 0.125f32.to_le_bytes();
    let in_log = log.windows(4).any(|w| w == needle);
    assert!(!in_log);
    drop(s);
    let s = Store::open(&dir, "notes", 1).unwrap();
    let got = s.graph().get_vec(id, "emb").unwrap();
    assert_eq!(got, &[0.125, 0.25, 0.5]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pin_sees_the_old_vector() {
    let dir = tmp("vpin");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().mark_vector("Doc", "emb");
    let id = s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.graph_mut().unwrap().set_vec(id, "emb", &[1.0, 0.0]);
    let a = s.commit().unwrap();
    s.graph_mut().unwrap().set_vec(id, "emb", &[0.0, 1.0]);
    s.commit().unwrap();
    let old = s.read_at(a).unwrap();
    assert_eq!(old.get_vec(id, "emb").unwrap(), &[1.0, 0.0]);
    assert_eq!(s.graph().get_vec(id, "emb").unwrap(), &[0.0, 1.0]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn replica_has_the_vector() {
    let prim = tmp("vp");
    let copy = tmp("vr");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().mark_vector("Doc", "emb");
    let id = s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.graph_mut().unwrap().set_vec(id, "emb", &[0.0, 1.0, 0.0]);
    s.commit().unwrap();
    let r = Store::tail(&copy, &prim, "notes").unwrap();
    assert_eq!(r.graph().get_vec(id, "emb").unwrap(), &[0.0, 1.0, 0.0]);
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn rollback_drops_a_new_vec() {
    let mut g = Graph::new();
    g.arm();
    g.mark_vector("Doc", "emb");
    let id = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    g.set_vec(id, "emb", &[1.0]);
    g.apply_undos();
    assert!(g.get_vec(id, "emb").is_none());
}
