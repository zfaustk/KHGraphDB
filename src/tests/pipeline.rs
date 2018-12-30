//! Pipeline tests. MATCH compiles to Seed / Expand.
//! A checkout of 3.6 cargo tests this file.

use crate::query;
use crate::query::Val;
use super::common::social;

fn name_of(v: &Option<Val>) -> Option<&str> {
    v.as_ref().and_then(|x| x.as_prop()).and_then(|p| p.as_str())
}

#[test]
fn explain_names_the_operators() {
    let mut g = social();
    let r = query::run(&mut g, "EXPLAIN MATCH (a:Person)-[:KNOWS]->(b)");
    assert!(r.ok);
    let mut kinds = Vec::new();
    for row in r.rows.iter() {
        if name_of(&row[0]) == Some("op") {
            kinds.push(name_of(&row[1]).unwrap().to_string());
        }
    }
    assert_eq!(kinds, vec!["Seed".to_string(), "Expand".to_string()]);
    let mut saw_plan = false;
    for row in r.rows.iter() {
        if name_of(&row[0]) == Some("plan") {
            saw_plan = true;
            let d = name_of(&row[2]).unwrap();
            assert!(d.contains("Expand"));
        }
    }
    assert!(saw_plan);
}

#[test]
fn explain_optional() {
    let mut g = social();
    let r = query::run(&mut g, "EXPLAIN OPTIONAL MATCH (a:Person {name:'Ada'})-[:KNOWS]->(b)");
    assert!(r.ok);
    let mut kinds = Vec::new();
    for row in r.rows.iter() {
        if name_of(&row[0]) == Some("op") {
            kinds.push(name_of(&row[1]).unwrap().to_string());
        }
    }
    assert_eq!(kinds,
               vec!["Seed".to_string(), "Expand".to_string(), "Optional".to_string()]);
}

#[test]
fn explain_shortest() {
    let mut g = social();
    let r = query::run(&mut g,
                       "EXPLAIN MATCH p = shortestPath((a:Person)-[:KNOWS*]->(b:Person))");
    assert!(r.ok);
    let mut saw = false;
    for row in r.rows.iter() {
        if name_of(&row[0]) == Some("op") {
            if name_of(&row[1]) == Some("Shortest") {
                saw = true;
            }
        }
    }
    assert!(saw);
}

#[test]
fn seed_only_match() {
    let mut g = social();
    let r = query::run(&mut g, "MATCH (a:Person)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 3);
}

#[test]
fn expand_one_hop() {
    let mut g = social();
    let r = query::run(&mut g, "MATCH (a:Person {name:'Alice'})-[:KNOWS]->(b)");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn explain_filter() {
    let mut g = social();
    let r = query::run(&mut g, "EXPLAIN MATCH (a:Person) WHERE a.name = 'Alice'");
    assert!(r.ok);
    let mut kinds = Vec::new();
    for row in r.rows.iter() {
        if name_of(&row[0]) == Some("op") {
            kinds.push(name_of(&row[1]).unwrap().to_string());
        }
    }
    assert_eq!(kinds,
               vec!["Seed".to_string(), "Filter".to_string()]);
}

#[test]
fn filter_runs_inside_match() {
    let mut g = social();
    let r = query::run(&mut g, "MATCH (a:Person) WHERE a.name = 'Alice'");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn explain_project_limit() {
    let mut g = social();
    let r = query::run(&mut g, "EXPLAIN MATCH (a:Person) RETURN a LIMIT 1");
    assert!(r.ok);
    let mut kinds = Vec::new();
    for row in r.rows.iter() {
        if name_of(&row[0]) == Some("op") {
            kinds.push(name_of(&row[1]).unwrap().to_string());
        }
    }
    assert_eq!(kinds,
               vec!["Seed".to_string(), "Project".to_string(), "Limit".to_string()]);
}
