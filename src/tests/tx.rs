//! Transaction tests. Drop rolls back. commit keeps the write.

use super::super::{query, Graph, Tx};
use super::common::{attrs, social};

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
    // still open: another write, then Drop rolls back to Ada
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
}
