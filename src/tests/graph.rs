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
fn edge_endpoints_are_khid() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
    let e = g.add_edge(&a, &b, Some("ROAD")).unwrap();
    let src = g.edge(&e).unwrap().source();
    let dst = g.edge(&e).unwrap().target();
    assert_eq!(src, super::super::Khid::parse(&a).unwrap());
    assert_eq!(dst, super::super::Khid::parse(&b).unwrap());
    assert_eq!(format!("{}", src), a);
}

#[test]
fn edge_type_is_khid() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
    let e = g.add_edge(&a, &b, Some("ROAD")).unwrap();
    let tid = g.edge(&e).unwrap().type_id().unwrap();
    assert_eq!(g.ty(&format!("{}", tid)).unwrap().name(), "ROAD");
}

#[test]
fn vertex_types_are_khid() {
    let mut g = Graph::new();
    let ada = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    let types = g.vertex(&ada).unwrap().types();
    assert_eq!(types.len(), 1);
    assert_eq!(g.ty(&format!("{}", types[0])).unwrap().name(), "Person");
}

#[test]
fn lookup_by_khid() {
    let mut g = Graph::new();
    let ada = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    let k = super::super::Khid::parse(&ada).unwrap();
    assert!(g.vertex_k(k).is_some());
    assert_eq!(g.vertex_k(k).unwrap().get("name"), Some("Ada"));
    let tid = g.vertex_k(k).unwrap().types()[0];
    assert_eq!(g.ty_k(tid).unwrap().name(), "Person");
}

#[test]
fn vertex_ids_are_khid() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    let ids = g.vertex_ids_k();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], super::super::Khid::parse(&a).unwrap());
    assert_eq!(g.vertex_ids(), vec![a]);
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
fn catalog_drop() {
    let mut cat = super::super::Catalog::new();
    cat.create("social").unwrap();
    cat.create("other").unwrap();
    assert!(cat.drop("other"));
    assert!(!cat.drop("other"));
    assert!(cat.graph("social").is_some());
    assert!(cat.graph("other").is_none());
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
fn snapshot_is_a_clone() {
    let g = social();
    let mut h = g.snapshot();
    h.add_vertex(attrs("Eve"), Some("Person")).unwrap();
    assert_eq!(g.vertex_count(), 3);
    assert_eq!(h.vertex_count(), 4);
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
fn unique_backfill_rejects_dup() {
    let mut g = Graph::new();
    g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
    g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
    assert_eq!(g.vertex_count(), 2);
    assert!(!g.create_unique("Person", "name"));
}

#[test]
fn unique_int_keeps_tag() {
    let mut g = Graph::new();
    g.create_unique("Person", "born");
    let a = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    g.set_prop(&a, "born", super::super::Prop::from_int(1815)).unwrap();
    let b = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
    assert!(g.set_prop(&b, "born", super::super::Prop::from_int(1815)).is_err());
    assert!(g.set_prop(&b, "born", super::super::Prop::from_str("1815")).is_ok());
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

#[test]
fn vertex_khid_is_copy() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    let k = g.vertex(&a).unwrap().khid();
    assert_eq!(format!("{}", k), a);
    assert!(!k.is_nil());
    let k2 = k;
    assert_eq!(k, k2);
    assert!(k.raw() > 0);
}

#[test]
fn edge_khid_is_copy() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
    let e = g.add_edge(&a, &b, Some("ROAD")).unwrap();
    let k = g.edge(&e).unwrap().khid();
    assert_eq!(format!("{}", k), e);
    assert!(!k.is_nil());
}

#[test]
fn type_khid_is_copy() {
    let mut g = Graph::new();
    g.add_type("Person").unwrap();
    let t = g.type_by_name("Person").unwrap();
    let k = t.khid();
    assert!(!k.is_nil());
    assert_eq!(format!("{}", k).as_bytes()[0], b'k');
    assert_eq!(t.name(), "Person");
}

#[test]
fn graph_keys_are_khid() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    let k = super::super::Khid::parse(&a).unwrap();
    assert_eq!(g.vertex(&a).unwrap().khid(), k);
    g.remove_vertex(&a);
    let b = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
    let k2 = super::super::Khid::parse(&b).unwrap();
    assert!(k2.raw() > k.raw());
}

#[test]
fn slot_zero_is_not_a_vertex() {
    let mut g = Graph::new();
    assert!(g.vertex("k0").is_none());
    let a = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    let k = super::super::Khid::parse(&a).unwrap();
    assert!(k.raw() >= 1);
    assert!(g.vertex("k0").is_none());
}

#[test]
fn arena_holes_after_delete() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    let b = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
    g.add_edge(&a, &b, Some("KNOWS")).unwrap();
    let before_types = g.type_count();
    assert!(g.remove_vertex(&a));
    assert_eq!(g.vertex_count(), 1);
    assert!(g.vertex(&a).is_none());
    assert!(g.vertex(&b).is_some());
    let c = g.add_vertex(attrs("Carol"), Some("Person")).unwrap();
    let ka = super::super::Khid::parse(&a).unwrap().raw();
    let kc = super::super::Khid::parse(&c).unwrap().raw();
    assert!(kc > ka);
    assert_eq!(g.type_count(), before_types);
}

#[test]
fn arena_clone_keeps_slots() {
    let g = social();
    let h = g.clone();
    assert_eq!(h.vertex_count(), g.vertex_count());
    assert_eq!(h.edge_count(), g.edge_count());
    let alice = g.vertex_by_name("Alice").unwrap().khid();
    assert_eq!(h.vertex(&format!("{}", alice)).unwrap().get("name"), Some("Alice"));
}

