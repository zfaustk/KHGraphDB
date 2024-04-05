//! A community is a Type. Members hop IN.

use crate::{keep_view, Graph};
use super::common::attrs;

#[test]
fn community_is_not_a_view() {
    let mut g = Graph::new();
    let c = g.community().unwrap();
    assert!(!g.is_view("Community"));
    assert!(!g.mark_view("Community", "MATCH (a) RETURN a"));
    assert!(g.vertex(c).is_some());
}

#[test]
fn member_hops_in() {
    let mut g = Graph::new();
    let c = g.community_as("lab").unwrap();
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    g.enclose(a, c).unwrap();
    assert_eq!(g.episode_of(a), vec![c]);
    assert_eq!(g.in_episode(c), vec![a]);
}

#[test]
fn nest_is_in() {
    let mut g = Graph::new();
    let inner = g.community_as("lab").unwrap();
    let outer = g.community_as("org").unwrap();
    g.enclose(inner, outer).unwrap();
    assert_eq!(g.episode_of(inner), vec![outer]);
}

#[test]
fn compact_keeps_in_to_a_community() {
    let mut g = Graph::new();
    let c = g.community().unwrap();
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    g.enclose(a, c).unwrap();
    assert_eq!(g.drop_orphan_in(), 0);
    assert_eq!(g.in_episode(c), vec![a]);
}

#[test]
fn a_report_is_a_view_over_in() {
    let mut g = Graph::new();
    let c = g.community_as("lab").unwrap();
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    g.enclose(a, c).unwrap();
    g.mark_view("Report", "MATCH (d:Doc)-[:IN]->(:Community) RETURN d");
    assert_eq!(keep_view(&mut g, "Report", 0).unwrap(), 1);
}
