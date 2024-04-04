//! A hierarchical cut. First try.

use crate::{cluster, nest, Graph};
use super::common::attrs;

#[test]
fn two_pairs_are_two_clusters() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let b = g.add_vertex(attrs("Alan"), Some("Doc")).unwrap();
    let c = g.add_vertex(attrs("Grace"), Some("Doc")).unwrap();
    let d = g.add_vertex(attrs("Donald"), Some("Doc")).unwrap();
    g.add_edge(a, b, Some("KNOWS")).unwrap();
    g.add_edge(c, d, Some("KNOWS")).unwrap();
    let cs = cluster(&mut g).unwrap();
    assert_eq!(cs.len(), 2);
    assert!(g.type_by_name("Community").is_some());
}

#[test]
fn a_lonely_vertex_is_not_a_cluster() {
    let mut g = Graph::new();
    let _ = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    assert!(cluster(&mut g).unwrap().is_empty());
}

#[test]
fn nest_is_a_parent() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let b = g.add_vertex(attrs("Alan"), Some("Doc")).unwrap();
    g.add_edge(a, b, Some("KNOWS")).unwrap();
    let cs = cluster(&mut g).unwrap();
    let p = nest(&mut g, &cs).unwrap();
    assert!(p.is_none() || g.vertex(p.unwrap()).is_some());
}
