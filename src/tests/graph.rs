//! graph tests.

use std::collections::HashMap;
use super::super::Graph;
use super::common::{attrs, social};

#[test]
fn add_and_lookup() {
    let mut g = Graph::new();
    let alice = g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
    let bob = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
    let _e = g.add_edge(&alice, &bob, Some("KNOWS")).unwrap();
    assert_eq!(g.vertex_count(), 2);
    assert_eq!(g.edge_count(), 1);
    assert!(g.vertex_by_name("Alice").is_some());
    assert!(g.has_type(&alice, "Person"));
    assert_eq!(g.edges_of_type("KNOWS").len(), 1);
    assert!(g.remove_vertex(&bob));
    assert_eq!(g.edge_count(), 0);
    assert_eq!(g.vertex(&alice).unwrap().out_degree(), 0);
}

#[test]
fn add_edge_with_attrs() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
    let mut w = HashMap::new();
    w.insert("weight".to_string(), "5".to_string());
    let e = g.add_edge_with(&a, &b, Some("ROAD"), w).unwrap();
    assert_eq!(g.edge(&e).unwrap().get("weight"), Some("5"));
}

#[test]
fn catalog_put() {
    let mut cat = super::super::Catalog::new();
    let mut g = Graph::named("social");
    g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
    let name = cat.put(g);
    assert_eq!(name, "social");
    assert!(cat.graph("social").unwrap().vertex_by_name("Alice").is_some());
}

#[test]
fn catalog_two_graphs() {
    let mut cat = super::super::Catalog::new();
    {
        let social = cat.create("social").unwrap();
        social.add_vertex(attrs("Alice"), Some("Person")).unwrap();
    }
    {
        let other = cat.create("other").unwrap();
        other.add_vertex(attrs("Bob"), Some("Person")).unwrap();
    }
    assert!(cat.create("social").is_err());
    assert!(cat.graph("social").unwrap().vertex_by_name("Alice").is_some());
    assert!(cat.graph("other").unwrap().vertex_by_name("Alice").is_none());
    let mut names = cat.names();
    names.sort();
    assert_eq!(names, vec!["other".to_string(), "social".to_string()]);
}

#[test]
fn clear_graph() {
    let mut g = social();
    g.clear();
    assert_eq!(g.vertex_count(), 0);
    assert_eq!(g.edge_count(), 0);
    assert_eq!(g.khid(), "g1");
}

#[test]
fn clone_graph() {
    let g = social();
    let mut h = g.clone();
    h.add_vertex(attrs("Dan"), Some("Person")).unwrap();
    assert_eq!(g.vertex_count(), 3);
    assert_eq!(h.vertex_count(), 4);
    assert_eq!(h.khid(), g.khid());
}

#[test]
fn edge_index() {
    let mut g = social();
    let eids = g.edges_of_type("KNOWS");
    g.set_edge_attr(&eids[0], "since", "2010");
    g.create_edge_index("KNOWS", "since");
    assert_eq!(g.find_edge("KNOWS", "since", "2010").len(), 1);
}

#[test]
fn multi_type() {
    let mut g = Graph::new();
    let ada = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    assert!(g.add_type_to_vertex(&ada, "Author").unwrap());
    assert!(g.has_type(&ada, "Person"));
    assert!(g.has_type(&ada, "Author"));
    assert_eq!(g.type_by_name("Author").unwrap().vertex_count(), 1);
}

#[test]
fn named_graph() {
    let g = Graph::named("social");
    assert_eq!(g.khid(), "social");
}

#[test]
fn snapshot() {
    use std::io::Cursor;
    let mut g = Graph::new();
    let ada = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    g.add_type_to_vertex(&ada, "Author").unwrap();
    let bob = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
    g.add_edge(&ada, &bob, Some("KNOWS")).unwrap();
    let mut buf = Vec::new();
    super::super::io::write_graph(&g, &mut buf).unwrap();
    let mut cur = Cursor::new(buf);
    let mut h = super::super::io::read_graph(&mut cur).unwrap();
    assert_eq!(h.vertex_count(), 2);
    assert!(h.vertex_by_name("Ada").is_some());
    let ada2 = h.vertex_by_name("Ada").unwrap().khid().to_string();
    assert!(h.has_type(&ada2, "Author"));
    assert_eq!(h.edges_of_type("KNOWS").len(), 1);
    assert_eq!(h.khid(), "g1");
    let before = h.vertex_count();
    h.add_vertex(attrs("Zoe"), Some("Person")).unwrap();
    assert_eq!(h.vertex_count(), before + 1);
}

#[test]
fn snapshot_edge_attrs() {
    use std::io::Cursor;
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
    let mut w = HashMap::new();
    w.insert("weight".to_string(), "7".to_string());
    g.add_edge_with(&a, &b, Some("ROAD"), w).unwrap();
    let mut buf = Vec::new();
    super::super::io::write_graph(&g, &mut buf).unwrap();
    let mut cur = Cursor::new(buf);
    let h = super::super::io::read_graph(&mut cur).unwrap();
    let eids = h.edges_of_type("ROAD");
    assert_eq!(eids.len(), 1);
    assert_eq!(h.edge(&eids[0]).unwrap().get("weight"), Some("7"));
}

#[test]
fn subgraph_alice() {
    let g = social();
    let alice = g.vertex_by_name("Alice").unwrap().khid().to_string();
    let bob = g.vertex_by_name("Bob").unwrap().khid().to_string();
    let h = g.subgraph(&[alice.clone(), bob.clone()]);
    assert_eq!(h.vertex_count(), 2);
    assert_eq!(h.edge_count(), 1);
    assert!(h.vertex_by_name("Carol").is_none());
    assert_eq!(g.vertex_count(), 3);
}

#[test]
fn unique_name() {
    let mut g = Graph::new();
    g.add_type("Person").unwrap();
    assert!(g.create_unique("Person", "name"));
    let a = g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
    assert!(g.add_vertex(attrs("Alice"), Some("Person")).is_err());
    assert_eq!(g.vertex_count(), 1);
    let c = g.add_vertex(attrs("Carol"), Some("Person")).unwrap();
    assert!(g.set_attr(&c, "name", "Alice").is_err());
    assert_eq!(g.vertex(&c).unwrap().get("name"), Some("Carol"));
    assert_eq!(g.find("Person", "name", "Alice"), vec![a.clone()]);
}

#[test]
fn vertex_stores_int() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    g.vertex_mut(&a).unwrap().set_prop("born", super::super::Prop::from_int(1815));
    let v = g.vertex(&a).unwrap();
    assert!(v.get("born").is_none());
    assert_eq!(v.get_prop("born").and_then(|p| p.as_int()), Some(1815));
    assert_eq!(v.get("name"), Some("Ada"));
}

#[test]
fn index_int_not_str() {
    let mut g = Graph::new();
    g.add_type("Person").unwrap();
    g.create_index("Person", "born");
    let a = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    g.set_prop(&a, "born", super::super::Prop::from_int(1815)).unwrap();
    assert_eq!(g.find_prop("Person", "born", &super::super::Prop::from_int(1815)).len(), 1);
    assert_eq!(g.find("Person", "born", "1815").len(), 0);
}

#[test]
fn snapshot_keeps_int() {
    use std::io::Cursor;
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    g.set_prop(&a, "born", super::super::Prop::from_int(1815)).unwrap();
    let mut buf = Vec::new();
    super::super::io::write_graph(&g, &mut buf).unwrap();
    assert_eq!(&buf[0..4], b"KHG4");
    let mut cur = Cursor::new(buf);
    let h = super::super::io::read_graph(&mut cur).unwrap();
    let ada = h.vertex_by_name("Ada").unwrap();
    assert_eq!(ada.get_prop("born").and_then(|p| p.as_int()), Some(1815));
    assert!(ada.get("born").is_none());
}

