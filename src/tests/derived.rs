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

#[test]
fn drop_hit_when_recipe_changes() {
    use crate::Addr;
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let src = g.add_vertex(super::common::attrs("Ada"), Some("Doc")).unwrap();
    let hit = g.add_vertex(super::common::attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(src)).unwrap();
    let old = g.type_by_name("Hit").unwrap().khid();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a.title");
    assert_eq!(g.drop_stale_derived(), 0);
    assert!(g.vertex(hit).is_some());
    assert!(g.vertex(src).is_some());
    assert_ne!(g.type_by_name("Hit").unwrap().khid(), old);
    assert!(!g.type_by_name("Hit").unwrap().vertices().contains(&hit));
    assert!(g.ty(old).unwrap().vertices().contains(&hit));
}

#[test]
fn a_new_recipe_is_a_new_type() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let old = g.type_by_name("Hit").unwrap().khid();
    let old_hash = g.view_hash_of("Hit").unwrap();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a.title");
    assert_ne!(g.type_by_name("Hit").unwrap().khid(), old);
    assert_eq!(g.view_of("Hit"), Some("MATCH (a:Doc) RETURN a.title"));
    let retired = format!("Hit~{:x}", old_hash);
    assert_eq!(g.type_by_name(&retired).unwrap().khid(), old);
    assert_eq!(g.ty(old).unwrap().view(), Some("MATCH (a:Doc) RETURN a"));
}

#[test]
fn keep_fills_the_live_name() {
    use crate::keep_view;
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let _ = g.add_vertex(super::common::attrs("Ada"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a ");
    assert_eq!(g.type_by_name("Hit").unwrap().vertex_count(), 0);
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    assert_eq!(g.type_by_name("Hit").unwrap().vertex_count(), 1);
}

#[test]
fn compact_drops_a_stale_hit() {
    use std::fs;
    use crate::{Addr, Store};
    let dir = std::env::temp_dir().join(format!("kh-stale-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let src;
    let hit;
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a");
        src = s.graph_mut().unwrap().add_vertex(super::common::attrs("Ada"), Some("Doc")).unwrap();
        hit = s.graph_mut().unwrap().add_vertex(super::common::attrs("h1"), Some("Hit")).unwrap();
        s.graph_mut().unwrap().derive_from(hit, Addr::here(src)).unwrap();
        s.commit().unwrap();
        s.graph_mut().unwrap().remove_vertex(src);
        s.commit().unwrap();
        s.compact().unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex(hit).is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn compact_drops_after_a_new_recipe() {
    use std::fs;
    use crate::{Addr, Store};
    let dir = std::env::temp_dir().join(format!("kh-recipe-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let hit;
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a");
        let src = s.graph_mut().unwrap().add_vertex(super::common::attrs("Ada"), Some("Doc")).unwrap();
        hit = s.graph_mut().unwrap().add_vertex(super::common::attrs("h1"), Some("Hit")).unwrap();
        s.graph_mut().unwrap().derive_from(hit, Addr::here(src)).unwrap();
        s.commit().unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a.title");
        s.commit().unwrap();
        s.compact().unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex(hit).is_some());
    assert_eq!(s.graph().view_of("Hit"), Some("MATCH (a:Doc) RETURN a.title"));
    assert!(!s.graph().type_by_name("Hit").unwrap().vertices().contains(&hit));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn reason_is_content() {
    let mut t = Type::new(Khid::from_raw(1), "Hit".to_string());
    t.mark_view("MATCH (a) RETURN a");
    assert!(t.is_content("reason"));
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    assert!(!g.create_index("Hit", "reason"));
    let hit = g.add_vertex(super::common::attrs("h1"), Some("Hit")).unwrap();
    g.set_attr(hit, "reason", "because").unwrap();
    assert_eq!(g.vertex(hit).unwrap().get("reason"), Some("because"));
    assert!(!g.has_index("Hit", "reason"));
}

#[test]
fn the_stamp_is_not_a_set() {
    use crate::Addr;
    use crate::Prop;
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    assert!(g.type_by_name("Hit").unwrap().is_content("view"));
    assert!(!g.create_index("Hit", "view"));
    let src = g.add_vertex(super::common::attrs("Ada"), Some("Doc")).unwrap();
    let hit = g.add_vertex(super::common::attrs("h1"), Some("Hit")).unwrap();
    let _ = g.derive_from(hit, Addr::here(src)).unwrap();
    assert!(g.set_prop(hit, "view", Prop::from_str("deadbeef")).is_err());
    let want = format!("{:x}", crate::ty::hash_view("MATCH (a:Doc) RETURN a"));
    assert_eq!(g.vertex(hit).unwrap().get("view"), Some(want.as_str()));
}










