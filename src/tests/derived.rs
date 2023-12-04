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

#[test]
fn cite_is_an_address() {
    use crate::Addr;
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let src = g.add_vertex(super::common::attrs("Ada"), Some("Doc")).unwrap();
    let hit = g.add_vertex(super::common::attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(src)).unwrap();
    let cited = g.derived(hit);
    assert_eq!(cited.len(), 1);
    assert_eq!(cited[0].khid(), src);
    assert_eq!(g.vertex(src).unwrap().get("name"), Some("Ada"));
    assert!(g.vertex(hit).unwrap().get("name") != Some("Ada"));
}

#[test]
fn cite_a_far_addr_does_not_copy() {
    use crate::Addr;
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let hit = g.add_vertex(super::common::attrs("h1"), Some("Hit")).unwrap();
    let far = Addr::new(2, Khid::from_raw(9));
    let _ = g.derive_from(hit, far).unwrap();
    assert_eq!(g.derived(hit), vec![far]);
    assert!(g.vertex(Khid::from_raw(9)).is_none());
}

#[test]
fn hit_stamps_the_recipe_hash() {
    use crate::Addr;
    use crate::ty::hash_view;
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let src = g.add_vertex(super::common::attrs("Ada"), Some("Doc")).unwrap();
    let hit = g.add_vertex(super::common::attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(src)).unwrap();
    let want = format!("{:x}", hash_view("MATCH (a:Doc) RETURN a"));
    assert_eq!(g.vertex(hit).unwrap().get("view"), Some(want.as_str()));
}

#[test]
fn drop_hit_when_source_is_gone() {
    use crate::Addr;
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let src = g.add_vertex(super::common::attrs("Ada"), Some("Doc")).unwrap();
    let hit = g.add_vertex(super::common::attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(src)).unwrap();
    g.remove_vertex(src);
    assert_eq!(g.drop_stale_derived(), 1);
    assert!(g.vertex(hit).is_none());
}

#[test]
fn far_source_is_not_ours() {
    use crate::Addr;
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let hit = g.add_vertex(super::common::attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::new(2, Khid::from_raw(9))).unwrap();
    assert_eq!(g.drop_stale_derived(), 0);
    assert!(g.vertex(hit).is_some());
}





