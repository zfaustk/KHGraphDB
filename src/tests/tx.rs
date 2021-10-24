//! Transaction tests. Drop rolls back. commit keeps the write.
//! Inverse of a touch, not a second arena.

use crate::{query, Graph, Store, Tx};
use super::common::{attrs, social};
use std::fs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khtx-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn drop_rolls_back() {
    let mut g = social();
    {
        let mut tx = Tx::begin(&mut g);
        query::run(tx.graph(), "CREATE (n:Person {name:'Zed'})");
        assert_eq!(tx.graph().vertex_count(), 4);
    }
    assert_eq!(g.vertex_count(), 3);
    assert!(g.vertex_by_name("Zed").is_none());
}

#[test]
fn commit_keeps_the_write() {
    let mut g = social();
    {
        let mut tx = Tx::begin(&mut g);
        query::run(tx.graph(), "CREATE (n:Person {name:'Zed'})");
        tx.commit();
    }
    assert_eq!(g.vertex_count(), 4);
    assert!(g.vertex_by_name("Zed").is_some());
}

#[test]
fn rollback_midway() {
    let mut g = Graph::new();
    g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    let mut tx = Tx::begin(&mut g);
    query::run(tx.graph(), "CREATE (n:Person {name:'Bob'})");
    assert_eq!(tx.graph().vertex_count(), 2);
    tx.rollback();
    assert_eq!(tx.graph().vertex_count(), 1);
    query::run(tx.graph(), "CREATE (n:Person {name:'Cara'})");
    drop(tx);
    assert_eq!(g.vertex_count(), 1);
    assert!(g.vertex_by_name("Ada").is_some());
}

#[test]
fn delete_rolls_back() {
    let mut g = social();
    {
        let mut tx = Tx::begin(&mut g);
        query::run(tx.graph(), "MATCH (a:Person {name:'Alice'}) DETACH DELETE a");
        assert_eq!(tx.graph().vertex_count(), 2);
    }
    assert_eq!(g.vertex_count(), 3);
    assert!(g.vertex_by_name("Alice").is_some());
    assert!(g.vertex_by_name("Bob").is_some());
}

#[test]
fn set_rolls_back() {
    let mut g = social();
    {
        let mut tx = Tx::begin(&mut g);
        query::run(tx.graph(), "MATCH (a:Person {name:'Alice'}) SET a.city = 'Paris'");
        let r = query::ask(tx.graph(), "MATCH (a:Person {name:'Alice'}) RETURN a.city");
        assert!(r.ok);
    }
    let r = query::ask(&g, "MATCH (a:Person {name:'Alice'}) RETURN a.city");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn ask_is_a_read() {
    let g = social();
    let r = query::ask(&g, "MATCH (a:Person) RETURN count(a)");
    assert!(r.ok);
    let w = query::ask(&g, "CREATE (n:Person {name:'Zed'})");
    assert!(!w.ok);
}

#[test]
fn store_rollback_does_not_replay() {
    let dir = tmp("rb");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Person")).unwrap();
    s.commit().unwrap();
    s.query("CREATE (n:Person {name:'Bob'})");
    assert_eq!(s.graph().vertex_count(), 2);
    s.rollback();
    assert_eq!(s.graph().vertex_count(), 1);
    assert!(s.graph().vertex_by_name("Ada").is_some());
    assert!(s.graph().vertex_by_name("Bob").is_none());
}

#[test]
fn undo_on_a_wide_graph() {
    let mut g = Graph::new();
    let mut i = 0;
    while i < 400 {
        g.add_vertex(attrs(&format!("n{}", i)), Some("Doc")).unwrap();
        i += 1;
    }
    let n = g.vertex_count();
    {
        let mut tx = Tx::begin(&mut g);
        query::run(tx.graph(), "MATCH (a:Doc {name:'n0'}) SET a.tag = 'x'");
        assert_eq!(tx.graph().vertex_count(), n);
    }
    assert_eq!(g.vertex_count(), n);
    let r = query::ask(&g, "MATCH (a:Doc {name:'n0'}) RETURN a.tag");
    assert!(r.ok);
}
