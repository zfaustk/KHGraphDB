//! SNB Interactive on a tiny scale. The example
//! is the clock. cargo test stays a unit test.

use crate::snb;
use crate::query;

#[test]
fn generate_counts() {
    let s = snb::generate(40, 1);
    assert_eq!(s.people.len(), 40);
    assert_eq!(s.posts.len(), 80);
    assert!(s.forums.len() >= 1);
    assert!(s.knows >= 40);
    assert_eq!(s.g.vertex_count(), 40 + 80 + s.forums.len() + s.g.vertices_of_type("Tag").len());
}

#[test]
fn short_reads_ok() {
    let s = snb::generate(30, 2);
    for (name, r) in snb::is::run_all(&s) {
        assert!(r.ok, "{}", name);
    }
}

#[test]
fn complex_reads_ok() {
    let s = snb::generate(30, 3);
    for (name, r) in snb::ic::run_all(&s) {
        assert!(r.ok, "{} {}", name, r.message);
    }
}

#[test]
fn updates_roundtrip() {
    let mut s = snb::generate(20, 4);
    let id = snb::iu::iu1(&mut s.g, "Zed0", "Zed", "Shaw", "Austin", 1991).unwrap();
    let f = s.people[0];
    snb::iu::iu2(&mut s.g, f, id).unwrap();
    let p = snb::iu::iu3(&mut s.g, "hello", 2021, id).unwrap();
    snb::iu::iu4(&mut s.g, f, p).unwrap();
    let r = query::ask(&s.g, "MATCH (a:Person {name:'Zed0'})-[:KNOWS]->() RETURN a");
    assert!(r.ok);
    assert_eq!(s.g.find("Person", "name", "Zed0").len(), 1);
}

#[test]
fn is1_has_the_person() {
    let s = snb::generate(16, 5);
    let n = snb::is::person_name(&s, 0);
    let r = snb::is::is1(&s.g, &n);
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn is3_friends_are_people() {
    let s = snb::generate(24, 6);
    let n = snb::is::person_name(&s, 0);
    let r = snb::is::is3(&s.g, &n);
    assert!(r.ok);
}

#[test]
fn ic13_is_a_walk() {
    let s = snb::generate(20, 7);
    let a = snb::is::person_name(&s, 0);
    let b = snb::is::person_name(&s, 1);
    let r = snb::ic::ic13(&s.g, &a, &b);
    assert!(r.ok);
}

#[test]
fn bi_reads_ok() {
    let s = snb::generate(24, 8);
    for (name, r) in snb::bi::run_all(&s) {
        assert!(r.ok, "{} {}", name, r.message);
    }
}

#[test]
fn invariants_hold() {
    let s = snb::generate(36, 9);
    assert!(snb::check::all(&s));
}

#[test]
fn count_people_matches_scale() {
    let s = snb::generate(18, 10);
    let r = snb::bi::bi1_count_people(&s.g);
    assert!(r.ok);
    assert_eq!(s.people.len(), 18);
}

