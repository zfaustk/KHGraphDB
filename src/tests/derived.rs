//! A view is a recipe on Type. Members come later.

use crate::ty::{Type, hash_view};
use crate::{Graph, Khid};

#[test]
fn hash_is_stable() {
    assert_eq!(hash_view("MATCH (a) RETURN a"), hash_view("MATCH (a) RETURN a"));
    assert!(hash_view("MATCH (a) RETURN a") != hash_view("MATCH (b) RETURN b"));
}

#[test]
fn recipe_lives_on_the_type() {
    let mut t = Type::new(Khid::from_raw(1), "Hit".to_string());
    assert!(!t.is_view());
    assert!(t.mark_view("MATCH (a:Doc) RETURN a"));
    assert!(t.is_view());
    assert_eq!(t.view(), Some("MATCH (a:Doc) RETURN a"));
    assert_eq!(t.view_hash(), hash_view("MATCH (a:Doc) RETURN a"));
    assert!(t.mark_view("MATCH (a:Doc) RETURN a.title"));
    assert!(t.view_hash() != hash_view("MATCH (a:Doc) RETURN a"));
}

#[test]
fn graph_marks_the_recipe() {
    let mut g = Graph::new();
    assert!(g.mark_view("Hit", "MATCH (a:Doc) RETURN a"));
    assert!(g.is_view("Hit"));
    assert_eq!(g.view_of("Hit"), Some("MATCH (a:Doc) RETURN a"));
    assert_eq!(g.view_hash_of("Hit"), Some(hash_view("MATCH (a:Doc) RETURN a")));
    assert!(g.type_by_name("Hit").is_some());
}

#[test]
fn recipe_survives_reopen_and_compact() {
    use std::fs;
    use crate::Store;
    let dir = std::env::temp_dir().join(format!("kh-view-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        assert!(s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a"));
        s.commit().unwrap();
    }
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        assert_eq!(s.graph().view_of("Hit"), Some("MATCH (a:Doc) RETURN a"));
        s.compact().unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert_eq!(s.graph().view_of("Hit"), Some("MATCH (a:Doc) RETURN a"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn mark_view_is_a_write() {
    use crate::{ask_query, run_query};
    let mut g = Graph::new();
    let r = run_query(&mut g, "MARK VIEW Hit AS 'MATCH (a:Doc) RETURN a'");
    assert!(r.ok);
    assert_eq!(g.view_of("Hit"), Some("MATCH (a:Doc) RETURN a"));
    let a = ask_query(&g, "MARK VIEW Hit AS 'MATCH (a:Doc) RETURN a'");
    assert!(!a.ok);
}


