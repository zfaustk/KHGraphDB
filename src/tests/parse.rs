//! Parse errors. The token is named.

use crate::query;
use crate::Graph;

fn err(q: &str) -> String {
    let mut g = Graph::new();
    let r = query::run(&mut g, q);
    assert!(!r.ok, "expected error for {}", q);
    r.message
}

#[test]
fn bad_char() {
    assert!(err("@").contains("bad char"));
}

#[test]
fn return_needs_match() {
    assert!(err("RETURN a").contains("RETURN without MATCH"));
}

#[test]
fn set_needs_match() {
    assert!(err("SET a.x = 1").contains("SET without MATCH"));
}

#[test]
fn where_needs_match() {
    assert!(err("WHERE a.x = 1").contains("WHERE without MATCH"));
}

#[test]
fn unknown_type() {
    assert!(err("MATCH (a:Ghost)").contains("unknown type"));
}

#[test]
fn explain_needs_match() {
    assert!(err("EXPLAIN CREATE (a)").contains("EXPLAIN expected MATCH"));
}

#[test]
fn merge_star() {
    let mut g = Graph::new();
    query::run(&mut g, "CREATE (a:Person {name:'Ada'})-[:KNOWS]->(b:Person {name:'Bob'})");
    let r = query::run(&mut g, "MERGE (a:Person {name:'Ada'})-[:KNOWS*]->(b)");
    assert!(!r.ok);
    assert!(r.message.contains("MERGE length"));
}

#[test]
fn unknown_param() {
    assert!(err("MATCH (a {name:$n})").contains("unknown param $n"));
}
