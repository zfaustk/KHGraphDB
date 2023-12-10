//! Extra cases for MARK VIEW.

use crate::ty::{Type, hash_view};
use crate::Khid;

#[test]
fn empty_recipe_is_refused() {
    let mut t = Type::new(Khid::from_raw(1), "Hit".to_string());
    assert!(!t.mark_view(""));
}

#[test]
fn empty_hash_is_the_offset() {
    assert_eq!(hash_view(""), 0xcbf29ce484222325);
}

use crate::{Graph, wal::{self, Rec}};

#[test]
fn graph_refuses_a_blank_view() {
    let mut g = Graph::new();
    assert!(!g.mark_view("Hit", ""));
    assert!(!g.mark_view("", "MATCH (a) RETURN a"));
}

#[test]
fn uncommitted_view_does_not_replay() {
    let recs = vec![
        Rec::Begin { tx: 1 },
        Rec::View {
            tx: 1,
            type_name: "Hit".to_string(),
            query: "MATCH (a) RETURN a".to_string(),
        },
        Rec::Begin { tx: 2 },
        Rec::View {
            tx: 2,
            type_name: "Hit".to_string(),
            query: "MATCH (b) RETURN b".to_string(),
        },
        Rec::Commit { tx: 2 },
    ];
    let g = wal::replay(1, &recs).unwrap();
    assert_eq!(g.view_of("Hit"), Some("MATCH (b) RETURN b"));
}

use std::io::Cursor;

#[test]
fn last_view_wins() {
    let recs = vec![
        Rec::Begin { tx: 1 },
        Rec::View {
            tx: 1,
            type_name: "Hit".to_string(),
            query: "MATCH (a) RETURN a".to_string(),
        },
        Rec::Commit { tx: 1 },
        Rec::Begin { tx: 2 },
        Rec::View {
            tx: 2,
            type_name: "Hit".to_string(),
            query: "MATCH (b) RETURN b".to_string(),
        },
        Rec::Commit { tx: 2 },
    ];
    let mut buf = Vec::new();
    wal::write(1, &recs, &mut buf).unwrap();
    let g = wal::recover(&mut Cursor::new(buf)).unwrap();
    assert_eq!(g.view_of("Hit"), Some("MATCH (b) RETURN b"));
}

#[test]
fn two_types_keep_two_recipes() {
    let mut g = Graph::new();
    assert!(g.mark_view("Hit", "MATCH (a) RETURN a"));
    assert!(g.mark_view("Note", "MATCH (b) RETURN b"));
    assert_eq!(g.view_of("Hit"), Some("MATCH (a) RETURN a"));
    assert_eq!(g.view_of("Note"), Some("MATCH (b) RETURN b"));
}

use crate::Store;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("kh-view-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&p);
    p
}

#[test]
fn replica_has_the_recipe() {
    let prim = tmp("p-view");
    let copy = tmp("r-view");
    {
        let mut s = Store::open(&prim, "notes", 1).unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a");
        s.commit().unwrap();
    }
    let r = Store::tail(&copy, &prim, "notes").unwrap();
    assert_eq!(r.graph().view_of("Hit"), Some("MATCH (a:Doc) RETURN a"));
    let _ = std::fs::remove_dir_all(&prim);
    let _ = std::fs::remove_dir_all(&copy);
}

#[test]
fn compact_twice_keeps_the_recipe() {
    let dir = tmp("twice");
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a");
        s.commit().unwrap();
        s.compact().unwrap();
        s.compact().unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert_eq!(s.graph().view_of("Hit"), Some("MATCH (a:Doc) RETURN a"));
    let _ = std::fs::remove_dir_all(&dir);
}

use crate::{ask_query, run_query};

#[test]
fn mark_view_needs_as_and_a_string() {
    let mut g = Graph::new();
    assert!(!run_query(&mut g, "MARK VIEW Hit").ok);
    assert!(!run_query(&mut g, "MARK VIEW Hit AS").ok);
    assert!(!run_query(&mut g, "MARK CONTENT Hit AS 'x'").ok);
    assert!(run_query(&mut g, "MARK VIEW Hit AS 'MATCH (a) RETURN a'").ok);
}

#[test]
fn ask_mark_is_a_write() {
    let g = Graph::new();
    let a = ask_query(&g, "MARK VIEW Hit AS 'MATCH (a) RETURN a'");
    assert!(!a.ok);
}

use crate::Addr;
use super::common::attrs;

#[test]
fn missing_hit_cannot_cite() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let src = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert!(g.derive_from(Khid::from_raw(99), Addr::here(src)).is_err());
}

#[test]
fn two_sources_are_two_cites() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let b = g.add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    let hit = g.add_vertex(attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(a)).unwrap();
    let _ = g.derive_from(hit, Addr::here(b)).unwrap();
    assert_eq!(g.derived(hit).len(), 2);
}

#[test]
fn far_and_local_together() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let hit = g.add_vertex(attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(a)).unwrap();
    let _ = g.derive_from(hit, Addr::new(2, Khid::from_raw(9))).unwrap();
    assert_eq!(g.derived(hit).len(), 2);
}

#[test]
fn stamp_is_lowercase_hex() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let src = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let hit = g.add_vertex(attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(src)).unwrap();
    let s = g.vertex(hit).unwrap().get("view").unwrap();
    assert!(s.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
}

#[test]
fn unstamped_member_is_stale() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let hit = g.add_vertex(attrs("h1"), Some("Hit")).unwrap();
    assert_eq!(g.drop_stale_derived(), 1);
    assert!(g.vertex(hit).is_none());
}

#[test]
fn remaining_cite_stands() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let b = g.add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    let hit = g.add_vertex(attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(a)).unwrap();
    let _ = g.derive_from(hit, Addr::here(b)).unwrap();
    g.remove_vertex(a);
    assert_eq!(g.drop_stale_derived(), 0);
    assert_eq!(g.derived(hit).len(), 1);
    g.remove_vertex(b);
    assert_eq!(g.drop_stale_derived(), 1);
    assert!(g.vertex(hit).is_none());
}

#[test]
fn far_cite_outlives_a_local_delete() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let hit = g.add_vertex(attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(a)).unwrap();
    let _ = g.derive_from(hit, Addr::new(2, Khid::from_raw(9))).unwrap();
    g.remove_vertex(a);
    assert_eq!(g.drop_stale_derived(), 0);
    assert_eq!(g.derived(hit).len(), 1);
}

#[test]
fn same_recipe_again_keeps_the_hit() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let src = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let hit = g.add_vertex(attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(src)).unwrap();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    assert_eq!(g.drop_stale_derived(), 0);
    assert!(g.vertex(hit).is_some());
}

#[test]
fn whitespace_is_a_new_recipe() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let src = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let hit = g.add_vertex(attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(src)).unwrap();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a ");
    assert_eq!(g.drop_stale_derived(), 1);
}

#[test]
fn replica_after_compact_still_has_the_recipe() {
    let prim = tmp("p-cc");
    let copy = tmp("r-cc");
    {
        let mut s = Store::open(&prim, "notes", 1).unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a");
        s.commit().unwrap();
        s.compact().unwrap();
    }
    let r = Store::tail(&copy, &prim, "notes").unwrap();
    assert_eq!(r.graph().view_of("Hit"), Some("MATCH (a:Doc) RETURN a"));
    let _ = std::fs::remove_dir_all(&prim);
    let _ = std::fs::remove_dir_all(&copy);
}
