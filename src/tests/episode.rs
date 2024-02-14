//! An episode is a vertex. KEEP hops IN.

use crate::{keep_view, run_query, ask_query, Graph, Store};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("kh-ep-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&p);
    p
}

#[test]
fn episode_is_not_a_view() {
    let mut g = Graph::new();
    let e = g.episode().unwrap();
    assert!(!g.is_view("Episode"));
    assert_eq!(g.open_episode(), Some(e));
    g.close_episode();
    assert!(g.open_episode().is_none());
}

#[test]
fn enclose_is_one_hop() {
    let mut g = Graph::new();
    let e = g.episode().unwrap();
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let x = g.enclose(a, e).unwrap();
    let y = g.enclose(a, e).unwrap();
    assert_eq!(x, y);
    assert_eq!(g.episode_of(a), vec![e]);
    assert_eq!(g.in_episode(e), vec![a]);
}

#[test]
fn keep_without_episode_has_no_in() {
    let mut g = Graph::new();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    let hit = *g.type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    assert!(g.episode_of(hit).is_empty());
}

#[test]
fn keep_hops_into_the_open_episode() {
    let mut g = Graph::new();
    let e = g.episode().unwrap();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert_eq!(keep_view(&mut g, "Hit", 0).unwrap(), 1);
    let hit = *g.type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    assert_eq!(g.episode_of(hit), vec![e]);
    assert_eq!(g.in_episode(e), vec![hit]);
}

#[test]
fn note_hops_into_the_open_episode() {
    let mut g = Graph::new();
    let e = g.episode().unwrap();
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let n = g.note(a).unwrap();
    assert_eq!(g.episode_of(n), vec![e]);
}

#[test]
fn close_stops_the_hop() {
    let mut g = Graph::new();
    let e = g.episode().unwrap();
    g.close_episode();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    keep_view(&mut g, "Hit", 0).unwrap();
    let hit = *g.type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    assert!(g.episode_of(hit).is_empty());
    assert!(g.in_episode(e).is_empty());
}

#[test]
fn episode_is_a_write() {
    let mut g = Graph::new();
    let r = run_query(&mut g, "EPISODE");
    assert!(r.ok);
    assert_eq!(r.created, 1);
    assert!(!ask_query(&g, "EPISODE").ok);
    assert!(run_query(&mut g, "CLOSE EPISODE").ok);
    assert!(g.open_episode().is_none());
}

#[test]
fn review_is_a_view_over_in() {
    let mut g = Graph::new();
    let e = g.episode().unwrap();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    keep_view(&mut g, "Hit", 0).unwrap();
    g.mark_view("Review", "MATCH (h:Hit)-[:IN]->(:Episode) RETURN h");
    assert_eq!(keep_view(&mut g, "Review", 0).unwrap(), 0);
    assert_eq!(keep_view(&mut g, "Review", 1).unwrap(), 1);
    let rev = *g.type_by_name("Review").unwrap().vertices().iter().next().unwrap();
    let srcs: Vec<_> = g.derived(rev).into_iter().map(|a| a.khid()).collect();
    let hit = *g.type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    assert_eq!(srcs, vec![hit]);
    assert_eq!(g.view_depth(rev), 2);
}

#[test]
fn episode_in_match_is_not_a_write() {
    let mut g = Graph::new();
    let _ = g.episode().unwrap();
    let a = ask_query(&g, "MATCH (e:Episode) RETURN e");
    assert!(a.ok);
    assert_eq!(a.rows.len(), 1);
}

#[test]
fn a_second_episode_is_another_vertex() {
    let mut g = Graph::new();
    let a = g.episode().unwrap();
    let b = g.episode().unwrap();
    assert_ne!(a, b);
    assert_eq!(g.open_episode(), Some(b));
    assert_eq!(g.type_by_name("Episode").unwrap().vertex_count(), 2);
}

#[test]
fn set_episode_moves_the_cursor() {
    let mut g = Graph::new();
    let a = g.episode().unwrap();
    let b = g.episode().unwrap();
    assert!(g.set_episode(a));
    g.mark_view("Hit", "MATCH (x:Doc) RETURN x");
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    keep_view(&mut g, "Hit", 0).unwrap();
    let hit = *g.type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    assert_eq!(g.episode_of(hit), vec![a]);
    assert!(g.in_episode(b).is_empty());
}

#[test]
fn compact_keeps_in() {
    let dir = tmp("ep-cc");
    let e;
    let hit;
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        e = s.episode().unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a");
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
        s.graph_mut().unwrap().set_episode(e);
        s.keep("Hit", 0).unwrap();
        s.commit().unwrap();
        s.compact().unwrap();
        hit = *s.graph().type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert_eq!(s.graph().episode_of(hit), vec![e]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn enclose_missing_is_err() {
    let mut g = Graph::new();
    let e = g.episode().unwrap();
    assert!(g.enclose(crate::Khid::from_raw(99), e).is_err());
}

#[test]
fn store_episode_stamps_pos_and_reopens() {
    let dir = tmp("ep");
    let e;
    let hit;
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        e = s.episode().unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a");
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
        s.graph_mut().unwrap().set_episode(e);
        s.keep("Hit", 0).unwrap();
        s.commit().unwrap();
        hit = *s.graph().type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex(e).unwrap().get("pos").is_some());
    assert_eq!(s.graph().episode_of(hit), vec![e]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn replica_has_the_in_hop() {
    let prim = tmp("p-in");
    let copy = tmp("r-in");
    let e;
    {
        let mut s = Store::open(&prim, "notes", 1).unwrap();
        e = s.episode().unwrap();
        s.graph_mut().unwrap().mark_view("Hit", "MATCH (a:Doc) RETURN a");
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
        s.graph_mut().unwrap().set_episode(e);
        s.keep("Hit", 0).unwrap();
        s.commit().unwrap();
    }
    let r = Store::tail(&copy, &prim, "notes").unwrap();
    let hit = *r.graph().type_by_name("Hit").unwrap().vertices().iter().next().unwrap();
    assert_eq!(r.graph().episode_of(hit), vec![e]);
    let _ = std::fs::remove_dir_all(&prim);
    let _ = std::fs::remove_dir_all(&copy);
}
