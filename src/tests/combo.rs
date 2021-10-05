//! Combinations: query × index × log × compact × replica.

use std::fs;
use crate::{query, wire, Meta, Store};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khl1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

fn reopen(dir: &std::path::Path) -> Store {
    Store::open(dir, "notes", 1).unwrap()
}

#[test]
fn cypher_delete_does_not_revive() {
    let dir = tmp("c-del");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.query("CREATE (a:Doc {name:'Ada'})");
    s.commit().unwrap();
    let r = s.query("MATCH (n:Doc {name:'Ada'}) DELETE n");
    assert!(r.ok);
    assert_eq!(r.deleted, 1);
    s.commit().unwrap();
    drop(s);
    let s = reopen(&dir);
    assert!(s.graph().vertex_by_name("Ada").is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn detach_delete_clears_edges_and_index() {
    let dir = tmp("c-det");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.query("CREATE (a:Doc {name:'Ada'}), (b:Doc {name:'Bob'}), (a)-[:CITES]->(b)");
    s.commit().unwrap();
    let r = s.query("MATCH (n:Doc {name:'Ada'}) DETACH DELETE n");
    assert!(r.ok);
    s.commit().unwrap();
    drop(s);
    let s = reopen(&dir);
    assert!(s.graph().vertex_by_name("Ada").is_none());
    assert!(s.graph().vertex_by_name("Bob").is_some());
    assert!(s.graph().find("Doc", "name", "Ada").is_empty());
    assert_eq!(s.graph().all_edges().len(), 0);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn set_name_then_reopen_finds_new() {
    let dir = tmp("c-set");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.query("CREATE (a:Doc {name:'Ada'})");
    s.commit().unwrap();
    let r = s.query("MATCH (n:Doc {name:'Ada'}) SET n.name = 'Bob'");
    assert!(r.ok);
    s.commit().unwrap();
    drop(s);
    let s = reopen(&dir);
    assert!(s.graph().find("Doc", "name", "Ada").is_empty());
    assert_eq!(s.graph().find("Doc", "name", "Bob").len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn remove_attr_unposts() {
    let dir = tmp("c-rm");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "title");
    s.query("CREATE (a:Doc {name:'Ada', title:'Note'})");
    s.commit().unwrap();
    let r = s.query("MATCH (n:Doc {name:'Ada'}) REMOVE n.title");
    assert!(r.ok);
    s.commit().unwrap();
    drop(s);
    let s = reopen(&dir);
    assert!(s.graph().find("Doc", "title", "Note").is_empty());
    assert!(s.graph().vertex_by_name("Ada").is_some());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn compact_after_delete_is_gone() {
    let dir = tmp("c-cmp");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.query("CREATE (a:Doc {name:'Ada'}), (b:Doc {name:'Bob'})");
    s.commit().unwrap();
    s.query("MATCH (n:Doc {name:'Ada'}) DELETE n");
    s.commit().unwrap();
    s.compact().unwrap();
    drop(s);
    let s = reopen(&dir);
    assert!(s.graph().vertex_by_name("Ada").is_none());
    assert!(s.graph().vertex_by_name("Bob").is_some());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unique_frees_the_name_after_delete() {
    let dir = tmp("c-uni");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_unique("Doc", "name");
    s.query("CREATE (a:Doc {name:'Ada'})");
    s.commit().unwrap();
    let r = s.query("CREATE (b:Doc {name:'Ada'})");
    assert!(!r.ok);
    s.rollback();
    s.query("MATCH (n:Doc {name:'Ada'}) DELETE n");
    s.commit().unwrap();
    let r = s.query("CREATE (b:Doc {name:'Ada'})");
    assert!(r.ok);
    s.commit().unwrap();
    drop(s);
    let s = reopen(&dir);
    assert_eq!(s.graph().find("Doc", "name", "Ada").len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn edge_set_survives_reopen() {
    let dir = tmp("c-eset");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_edge_index("CITES", "w");
    s.query("CREATE (a:Doc {name:'Ada'}), (b:Doc {name:'Bob'}), (a)-[:CITES]->(b)");
    s.commit().unwrap();
    let eid = s.graph().all_edges()[0].0;
    assert!(s.graph_mut().unwrap().set_edge_prop(eid, "w", crate::Prop::from_str("2")));
    s.commit().unwrap();
    drop(s);
    let s = reopen(&dir);
    let g = s.graph();
    assert_eq!(g.find_edge("CITES", "w", "2").len(), 1);
    assert!(g.find_edge("CITES", "w", "1").is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn delete_edge_only() {
    let dir = tmp("c-de");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.query("CREATE (a:Doc {name:'Ada'}), (b:Doc {name:'Bob'}), (a)-[e:CITES]->(b)");
    s.commit().unwrap();
    let r = s.query("MATCH (a:Doc {name:'Ada'})-[e:CITES]->(b) DELETE e");
    assert!(r.ok);
    s.commit().unwrap();
    drop(s);
    let s = reopen(&dir);
    assert!(s.graph().vertex_by_name("Ada").is_some());
    assert_eq!(s.graph().all_edges().len(), 0);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn create_delete_same_tx() {
    let dir = tmp("c-same");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.query("CREATE (a:Doc {name:'Ada'})");
    s.query("MATCH (n:Doc {name:'Ada'}) DELETE n");
    s.commit().unwrap();
    drop(s);
    let s = reopen(&dir);
    assert!(s.graph().vertex_by_name("Ada").is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rollback_forgets_the_create() {
    let dir = tmp("c-rb");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.query("CREATE (a:Doc {name:'Ada'})");
    s.commit().unwrap();
    s.query("CREATE (b:Doc {name:'Bob'})");
    s.rollback();
    assert!(s.graph().vertex_by_name("Bob").is_none());
    assert!(s.graph().vertex_by_name("Ada").is_some());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn replica_does_not_see_deleted() {
    let prim = tmp("c-rp");
    let copy = tmp("c-rr");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.query("CREATE (a:Doc {name:'Ada'}), (b:Doc {name:'Bob'})");
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    s.query("MATCH (n:Doc {name:'Ada'}) DELETE n");
    let bm = s.commit().unwrap();
    r.catch_up(&prim).unwrap();
    r.honor(&prim, bm).unwrap();
    assert!(r.graph().vertex_by_name("Ada").is_none());
    assert!(r.graph().vertex_by_name("Bob").is_some());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn meta_forget_survives_open() {
    let dir = tmp("c-mf");
    fs::create_dir_all(&dir).unwrap();
    let mut m = Meta::empty(&dir);
    let a = crate::Addr::new(1, crate::Khid::from_raw(7));
    m.remember("Doc", "name", "Ada", a).unwrap();
    m.forget("Doc", "name", "Ada", a).unwrap();
    let m = Meta::open(&dir).unwrap();
    assert!(m.find("Doc", "name", "Ada").is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn match_on_the_socket_sees_committed() {
    use std::thread;
    let dir = tmp("c-ask");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.query("CREATE (a:Doc {name:'Ada'})");
    s.commit().unwrap();
    let listener = wire::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let d = dir.clone();
    let g = s.graph().clone();
    thread::spawn(move || {
        let (st, _) = listener.accept().unwrap();
        let _ = wire::handle(&d, &g, st);
    });
    let r = wire::ask(addr, "MATCH (n:Doc {name:'Ada'}) RETURN n.name").unwrap();
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn snapshot_reader_misses_later_delete() {
    let dir = tmp("c-pos");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.query("CREATE (a:Doc {name:'Ada'})");
    let bm = s.commit().unwrap();
    s.query("MATCH (n:Doc {name:'Ada'}) DELETE n");
    s.commit().unwrap();
    let old = s.read_at(bm).unwrap();
    assert!(old.vertex_by_name("Ada").is_some());
    assert!(s.graph().vertex_by_name("Ada").is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn merge_then_delete_then_merge() {
    let dir = tmp("c-mg");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.query("MERGE (a:Doc {name:'Ada'})");
    s.commit().unwrap();
    s.query("MATCH (n:Doc {name:'Ada'}) DELETE n");
    s.commit().unwrap();
    let r = s.query("MERGE (a:Doc {name:'Ada'})");
    assert!(r.ok);
    s.commit().unwrap();
    drop(s);
    let s = reopen(&dir);
    assert_eq!(s.graph().find("Doc", "name", "Ada").len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn explain_still_runs_on_a_store() {
    let dir = tmp("c-ex");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.query("CREATE (a:Doc {name:'Ada'})");
    let r = s.query("EXPLAIN MATCH (n:Doc {name:'Ada'}) RETURN n");
    assert!(r.ok);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn query_without_lease_fails() {
    let dir = tmp("c-ls");
    let mut a = Store::open(&dir, "notes", 1).unwrap();
    a.query("CREATE (a:Doc {name:'Ada'})");
    a.commit().unwrap();
    let mut b = Store::open(&dir, "notes", 1).unwrap();
    let r = b.query("CREATE (b:Doc {name:'Bob'})");
    assert!(!r.ok);
    drop(a);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn far_edge_drop_with_src() {
    let dir = tmp("c-far");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    let src = s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let dst = crate::Addr::new(2, crate::Khid::from_raw(9));
    s.put_far(src, dst, Some("CITES")).unwrap();
    s.commit().unwrap();
    s.graph_mut().unwrap().remove_vertex(src);
    s.commit().unwrap();
    drop(s);
    let s = reopen(&dir);
    assert!(s.graph().vertex_by_name("Ada").is_none());
    assert_eq!(s.graph().all_edges().len(), 0);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn count_after_delete() {
    let dir = tmp("c-cnt");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.query("CREATE (a:Doc {name:'Ada'}), (b:Doc {name:'Bob'})");
    s.commit().unwrap();
    s.query("MATCH (n:Doc {name:'Ada'}) DELETE n");
    s.commit().unwrap();
    let r = query::run(&mut s.graph().clone(), "MATCH (n:Doc) RETURN count(n)");
    assert!(r.ok);
    let _ = fs::remove_dir_all(&dir);
}