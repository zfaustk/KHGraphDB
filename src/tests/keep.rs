//! KEEP fills a view. NOTE cites the world.

use crate::ty::hash_view;
use crate::{keep_view, run_query, ask_query, Addr, Graph, Store};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("kh-keep-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&p);
    p
}

#[test]
fn keep_fills_one_hit_per_source() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let b = g.add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 2);
    let members: Vec<_> = g.type_by_name("Hit").unwrap().vertices().iter().cloned().collect();
    assert_eq!(members.len(), 2);
    let mut srcs = Vec::new();
    for h in members {
        let d = g.derived(h);
        assert_eq!(d.len(), 1);
        srcs.push(d[0].khid());
        let want = format!("{:x}", hash_view("MATCH (a:Doc) RETURN a"));
        assert_eq!(g.vertex(h).unwrap().get("view"), Some(want.as_str()));
        assert_eq!(g.view_depth(h), 1);
    }
    srcs.sort();
    let mut expect = vec![a, b];
    expect.sort();
    assert_eq!(srcs, expect);
}

#[test]
fn keep_twice_is_the_same_soup() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 0);
    assert_eq!(g.type_by_name("Hit").unwrap().vertex_count(), 1);
}

#[test]
fn keep_skips_its_own_members() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a) RETURN a");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 0);
}

#[test]
fn keep_fold_zero_skips_other_soup() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    g.mark_view("Out", "MATCH (a) RETURN a");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    assert_eq!(keep_view(&mut g, "Out", 0).unwrap(), 1);
    assert_eq!(g.type_by_name("Out").unwrap().vertex_count(), 1);
}

#[test]
fn keep_fold_one_cites_other_soup() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    g.mark_view("Out", "MATCH (h:Hit) RETURN h");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    assert_eq!(keep_view(&mut g, "Out", 0).unwrap(), 0);
    assert_eq!(keep_view(&mut g, "Out", 1).unwrap(), 1);
    let out: Vec<_> = g.type_by_name("Out").unwrap().vertices().iter().cloned().collect();
    assert_eq!(g.view_depth(out[0]), 2);
}

#[test]
fn recipe_that_writes_is_refused() {
    let mut g = Graph::new();
    g.mark_view("Hit", "CREATE (a:Doc {name:'x'})");
    assert!(keep_view(&mut g, "Hit", 0).is_err());
}

#[test]
fn keep_needs_a_view() {
    let mut g = Graph::new();
    assert!(keep_view(&mut g, "Hit", 0).is_err());
}

#[test]
fn keep_is_a_write() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let r = run_query(&mut g, "KEEP Hit");
    assert!(r.ok);
    assert_eq!(r.created, 1);
    let a = ask_query(&g, "KEEP Hit");
    assert!(!a.ok);
}

#[test]
fn keep_fold_in_the_language() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    g.mark_view("Out", "MATCH (h:Hit) RETURN h");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert!(run_query(&mut g, "KEEP Hit").ok);
    assert_eq!(run_query(&mut g, "KEEP Out").created, 0);
    assert_eq!(run_query(&mut g, "KEEP Out FOLD 1").created, 1);
}

#[test]
fn store_stamps_pos() {
    let dir = tmp("pos");
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a");
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
        assert_eq!(s.keep("Hit", 0).unwrap(), 1);
        s.commit().unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    let hit = *s.graph().type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    let p = s.graph().vertex(hit).unwrap().get("pos").unwrap();
    assert!(p.contains(':'));
    assert_eq!(s.graph().derived(hit).len(), 1);
    assert!(s.graph().type_by_name("Hit").unwrap().is_content("pos"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn note_cites_the_world() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let n = g.note(a).unwrap();
    assert!(!g.is_view("Note"));
    assert_eq!(g.seen(n).len(), 1);
    assert_eq!(g.seen(n)[0].khid(), a);
    assert_eq!(g.view_depth(n), 0);
}

#[test]
fn note_is_a_write() {
    let mut g = Graph::new();
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let r = run_query(&mut g, "NOTE Ada");
    assert!(r.ok);
    assert_eq!(r.created, 1);
    assert!(!ask_query(&g, "NOTE Ada").ok);
}

#[test]
fn keep_does_not_cite_a_note_at_fold_zero() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let _ = g.note(a).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
}

#[test]
fn store_note_stamps_pos() {
    let dir = tmp("note");
    let a;
    let n;
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        a = s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
        n = s.note(a).unwrap();
        s.commit().unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex(n).unwrap().get("pos").is_some());
    assert_eq!(s.graph().seen(n)[0].khid(), a);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn one_row_two_ids_is_one_hit() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc {name:'Ada'}) MATCH (b:Doc {name:'Bob'}) RETURN a, b");
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let b = g.add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    let hit = *g.type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    let mut srcs: Vec<_> = g.derived(hit).into_iter().map(|x| x.khid()).collect();
    srcs.sort();
    let mut want = vec![a, b];
    want.sort();
    assert_eq!(srcs, want);
}

#[test]
fn replica_has_the_soup() {
    let prim = tmp("p-soup");
    let copy = tmp("r-soup");
    {
        let mut s = Store::open(&prim, "notes", 1).unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a");
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
        s.keep("Hit", 0).unwrap();
        s.commit().unwrap();
    }
    let r = Store::tail(&copy, &prim, "notes").unwrap();
    assert_eq!(r.graph().type_by_name("Hit").unwrap().vertex_count(), 1);
    let hit = *r.graph().type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    assert_eq!(r.graph().derived(hit).len(), 1);
    let _ = std::fs::remove_dir_all(&prim);
    let _ = std::fs::remove_dir_all(&copy);
}

#[test]
fn fold_cannot_cite_self() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a) RETURN a");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    assert_eq!(keep_view(&mut g, "Hit", 9).unwrap(), 0);
}

#[test]
fn keep_empty_graph_is_zero() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 0);
}

#[test]
fn keep_skips_a_prop_column() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a.name");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 0);
}

#[test]
fn keep_cites_a_far_addr() {
    let mut g = Graph::new();
    let mut m = attrs("Ada");
    m.insert("cite".to_string(), "s2/k2a".to_string());
    let _ = g.add_vertex(m, Some("Doc")).unwrap();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a.cite");
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    let hit = *g.type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    let srcs = g.derived(hit);
    assert_eq!(srcs.len(), 1);
    assert_eq!(srcs[0].shard(), 2);
}

#[test]
fn keep_far_twice_is_one_hit() {
    let mut g = Graph::new();
    let mut m = attrs("Ada");
    m.insert("cite".to_string(), "s2/k2a".to_string());
    let _ = g.add_vertex(m, Some("Doc")).unwrap();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a.cite");
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 0);
}

