//! The subtractions: ordered posting, cheaper seed,
//! lag is a Pos, grouped sync is a session.

use crate::{query, Graph, Meta, Store};
use crate::prop::Prop;
use super::common::attrs;
use std::fs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khk-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn unique_seed_costs_one() {
    let mut g = Graph::new();
    g.create_unique("Person", "name");
    g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
    g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
    g.add_vertex(attrs("Carol"), Some("Person")).unwrap();
    let r = query::ask(&g, "EXPLAIN MATCH (a:Person {name:'Alice'})");
    assert!(r.ok);
    let mut cost = None;
    for row in r.rows.iter() {
        let slot = row[0].as_ref().and_then(|v| v.as_prop()).and_then(|p| p.as_str());
        if slot == Some("cost") {
            cost = row[1].as_ref().and_then(|v| v.as_prop()).and_then(|p| p.as_str()).map(|s| s.to_string());
        }
    }
    assert_eq!(cost.as_ref().map(|s| s.as_str()), Some("1"));
}

#[test]
fn range_where_uses_the_posting() {
    let mut g = Graph::new();
    g.create_index("Person", "age");
    let mut i = 0;
    while i < 8 {
        let a = g.add_vertex(attrs(&format!("n{}", i)), Some("Person")).unwrap();
        g.set_prop(a, "age", Prop::from_int(i as i64 * 10)).unwrap();
        i += 1;
    }
    let r = query::ask(&g, "MATCH (a:Person) WHERE a.age > 30 RETURN a");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 4);
    let r2 = query::ask(&g, "MATCH (a:Person) WHERE a.age > 20 AND a.age < 50 RETURN a");
    assert_eq!(r2.rows.len(), 2);
}

#[test]
fn cheaper_end_flips() {
    let mut g = Graph::new();
    g.create_unique("City", "name");
    g.create_index("Person", "name");
    let c = g.add_vertex(attrs("Paris"), Some("City")).unwrap();
    let mut i = 0;
    while i < 6 {
        let p = g.add_vertex(attrs(&format!("p{}", i)), Some("Person")).unwrap();
        g.add_edge(p, c, Some("LIVES")).unwrap();
        i += 1;
    }
    let r = query::ask(&g, "EXPLAIN MATCH (a:Person)-[:LIVES]->(b:City {name:'Paris'})");
    assert!(r.ok);
    let mut seed = None;
    for row in r.rows.iter() {
        let slot = row[0].as_ref().and_then(|v| v.as_prop()).and_then(|p| p.as_str());
        if slot == Some("cost") {
            seed = row[2].as_ref().and_then(|v| v.as_prop()).and_then(|p| p.as_str()).map(|s| s.to_string());
        }
    }
    assert_eq!(seed.as_ref().map(|s| s.as_str()), Some("b"));
}

#[test]
fn lag_is_subtraction() {
    let prim = tmp("lag-p");
    let copy = tmp("lag-r");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    assert_eq!(r.lag(s.pos().unwrap()).unwrap(), 0);
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    let at = s.commit().unwrap();
    assert!(r.lag(at).unwrap() > 0);
    r.catch_up(&prim).unwrap();
    assert_eq!(r.lag(s.pos().unwrap()).unwrap(), 0);
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn fold_drops_cancelled_pairs() {
    let dir = tmp("fold");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.query("MATCH (a:Doc {name:'Ada'}) SET a.name = 'Bob'");
    s.commit().unwrap();
    s.query("MATCH (a:Doc {name:'Bob'}) SET a.name = 'Cara'");
    s.commit().unwrap();
    let before = fs::metadata(dir.join("meta")).unwrap().len();
    s.fold_meta().unwrap();
    let after = fs::metadata(dir.join("meta")).unwrap().len();
    assert!(after <= before);
    let m = Meta::open(&dir).unwrap();
    assert_eq!(m.find("Doc", "name", "Cara").len(), 1);
    assert_eq!(m.find("Doc", "name", "Ada").len(), 0);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn grouped_sync_is_a_session() {
    let dir = tmp("sync");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.set_durable(false);
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.flush().unwrap();
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert_eq!(s.graph().vertex_count(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn reverse_from_the_far_end() {
    let mut g = Graph::new();
    g.create_unique("City", "name");
    g.create_index("Person", "name");
    let c = g.add_vertex(attrs("Paris"), Some("City")).unwrap();
    let mut i = 0;
    let mut prev = None;
    while i < 5 {
        let p = g.add_vertex(attrs(&format!("p{}", i)), Some("Person")).unwrap();
        g.add_edge(p, c, Some("LIVES")).unwrap();
        if let Some(q) = prev {
            g.add_edge(q, p, Some("KNOWS")).unwrap();
        }
        prev = Some(p);
        i += 1;
    }
    let r = query::ask(&g,
        "EXPLAIN MATCH (a:Person)-[:KNOWS]->(b:Person)-[:LIVES]->(c:City {name:'Paris'})");
    assert!(r.ok);
    let mut seed = None;
    for row in r.rows.iter() {
        let slot = row[0].as_ref().and_then(|v| v.as_prop()).and_then(|p| p.as_str());
        if slot == Some("cost") {
            seed = row[2].as_ref().and_then(|v| v.as_prop()).and_then(|p| p.as_str()).map(|s| s.to_string());
        }
    }
    assert_eq!(seed.as_ref().map(|s| s.as_str()), Some("c"));
}

#[test]
fn picture_hides_the_tail() {
    let dir = tmp("pic");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    let live = s.graph().vertex_count();
    let r = s.ask("MATCH (a:Doc {name:'Bob'}) RETURN a");
    assert_eq!(live, 2);
    assert_eq!(r.rows.len(), 0);
    s.commit().unwrap();
    let r = s.ask("MATCH (a:Doc {name:'Bob'}) RETURN a");
    assert_eq!(r.rows.len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn group_count_then_flush() {
    let dir = tmp("grp");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.set_sync_every(3);
    s.graph_mut().unwrap().add_vertex(attrs("a"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("b"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.flush().unwrap();
    drop(s);
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert_eq!(s.graph().vertex_count(), 2);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn writer_notices_a_fat_log() {
    let dir = tmp("ckpt");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.compact().unwrap();
    let mut i = 0;
    while i < 12 {
        s.graph_mut().unwrap().add_vertex(attrs(&format!("n{}", i)), Some("Doc")).unwrap();
        s.commit().unwrap();
        i += 1;
    }
    let did = s.maybe_compact().unwrap();
    assert!(did.is_some());
    let _ = fs::remove_dir_all(&dir);
}

