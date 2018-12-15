//! algo tests.

use crate::Graph;
use super::common::attrs;

#[test]
fn bfs_and_dijkstra() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
    let c = g.add_vertex(attrs("C"), Some("City")).unwrap();
    g.add_edge(&a, &b, Some("ROAD")).unwrap();
    g.add_edge(&b, &c, Some("ROAD")).unwrap();
    g.add_edge(&a, &c, Some("ROAD")).unwrap();
    let near = crate::algo::nearby(&g, &a, 2);
    assert!(near.len() >= 1);
    let p = crate::algo::path(&g, &a, &c).unwrap();
    assert_eq!(p[0], a);
    assert_eq!(*p.last().unwrap(), c);
    assert!(!crate::algo::has_cycle(&g));
    let s = crate::algo::shortest(&g, &a, &c).unwrap();
    assert_eq!(s.last().unwrap(), &c);
    let comps = crate::algo::components(&g);
    assert_eq!(comps.len(), 1);
}

#[test]
fn two_components() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("N")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("N")).unwrap();
    g.add_vertex(attrs("C"), Some("N")).unwrap();
    g.add_edge(&a, &b, Some("E")).unwrap();
    let comps = crate::algo::components(&g);
    assert_eq!(comps.len(), 2);
}

#[test]
fn weighted_shortest() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
    let c = g.add_vertex(attrs("C"), Some("City")).unwrap();
    let ab = g.add_edge(&a, &b, Some("ROAD")).unwrap();
    let bc = g.add_edge(&b, &c, Some("ROAD")).unwrap();
    let ac = g.add_edge(&a, &c, Some("ROAD")).unwrap();
    g.set_edge_attr(&ab, "weight", "2");
    g.set_edge_attr(&bc, "weight", "2");
    g.set_edge_attr(&ac, "weight", "5");
    let s = crate::algo::shortest(&g, &a, &c).unwrap();
    assert_eq!(s, vec![a.clone(), b.clone(), c.clone()]);
}

#[test]
fn cycle_is_true() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("N")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("N")).unwrap();
    g.add_edge(&a, &b, Some("E")).unwrap();
    g.add_edge(&b, &a, Some("E")).unwrap();
    assert!(crate::algo::has_cycle(&g));
}

#[test]
fn nearby_depth_one() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("N")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("N")).unwrap();
    let c = g.add_vertex(attrs("C"), Some("N")).unwrap();
    g.add_edge(&a, &b, Some("E")).unwrap();
    g.add_edge(&b, &c, Some("E")).unwrap();
    let near = crate::algo::nearby(&g, &a, 1);
    assert_eq!(near, vec![b.clone()]);
}

#[test]
fn path_missing() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("N")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("N")).unwrap();
    assert!(crate::algo::path(&g, &a, &b).is_none());
    assert_eq!(crate::algo::path(&g, &a, &a).unwrap(), vec![a.clone()]);
}


