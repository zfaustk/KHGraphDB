//! KHG4 round-trip. Tags survive. KHG3 still reads.

use std::io::Cursor;
use crate::{io as khio, Graph, Prop};
use super::common::attrs;

#[test]
fn khg4_keeps_int() {
    let mut g = Graph::new();
    let ada = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    g.set_prop(ada, "born", Prop::from_int(1815)).unwrap();
    let mut buf = Vec::new();
    khio::write_graph(&g, &mut buf).unwrap();
    assert_eq!(&buf[0..4], b"KHG4");
    let h = khio::read_graph(&mut Cursor::new(buf)).unwrap();
    let ada2 = h.vertex_by_name("Ada").unwrap();
    assert_eq!(ada2.get_prop("born").and_then(|p| p.as_int()), Some(1815));
    assert!(ada2.get("born").is_none());
}

#[test]
fn khg4_keeps_bool() {
    let mut g = Graph::new();
    let ada = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
    g.set_prop(ada, "alive", Prop::from_bool(true)).unwrap();
    let mut buf = Vec::new();
    khio::write_graph(&g, &mut buf).unwrap();
    let h = khio::read_graph(&mut Cursor::new(buf)).unwrap();
    let ada2 = h.vertex_by_name("Ada").unwrap();
    assert_eq!(ada2.get_prop("alive").and_then(|p| p.as_bool()), Some(true));
}

#[test]
fn empty_graph_roundtrip() {
    let g = Graph::named("empty");
    let mut buf = Vec::new();
    khio::write_graph(&g, &mut buf).unwrap();
    let h = khio::read_graph(&mut Cursor::new(buf)).unwrap();
    assert_eq!(h.khid(), "empty");
    assert_eq!(h.vertex_count(), 0);
    assert_eq!(h.edge_count(), 0);
}

#[test]
fn edge_ends_survive() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
    let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
    let e = g.add_edge(a, b, Some("ROAD")).unwrap();
    let mut buf = Vec::new();
    khio::write_graph(&g, &mut buf).unwrap();
    let h = khio::read_graph(&mut Cursor::new(buf)).unwrap();
    let e2 = h.edge(e).unwrap();
    assert_eq!(e2.source(), a);
    assert_eq!(e2.target(), b);
    assert_eq!(h.edge_type_name(e).as_ref().map(|s| s.as_str()), Some("ROAD"));
}
