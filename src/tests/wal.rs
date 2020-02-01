//! KHL1 log. Uncommitted records do not replay.

use std::collections::HashMap;
use std::io::Cursor;
use crate::wal::{self, Rec};
use crate::{Khid, Prop};

fn name(s: &str) -> HashMap<String, Prop> {
    let mut m = HashMap::new();
    m.insert("name".to_string(), Prop::from_str(s));
    m
}

#[test]
fn roundtrip_records() {
    let recs = vec![
        Rec::Begin { tx: 1 },
        Rec::Vertex {
            tx: 1,
            id: Khid::from_raw(1),
            types: vec!["Person".to_string()],
            attrs: name("Ada"),
        },
        Rec::Commit { tx: 1 },
    ];
    let mut buf = Vec::new();
    wal::write(3, &recs, &mut buf).unwrap();
    assert_eq!(&buf[0..4], b"KHL2");
    let (shard, got) = wal::read(&mut Cursor::new(buf)).unwrap();
    assert_eq!(shard, 3);
    assert_eq!(got, recs);
}

#[test]
fn recover_puts_the_vertex() {
    let recs = vec![
        Rec::Begin { tx: 1 },
        Rec::Vertex {
            tx: 1,
            id: Khid::from_raw(1),
            types: vec!["Person".to_string()],
            attrs: name("Ada"),
        },
        Rec::Commit { tx: 1 },
    ];
    let mut buf = Vec::new();
    wal::write(1, &recs, &mut buf).unwrap();
    let g = wal::recover(&mut Cursor::new(buf)).unwrap();
    assert_eq!(g.shard(), 1);
    let v = g.vertex(Khid::from_raw(1)).unwrap();
    assert_eq!(v.get("name"), Some("Ada"));
    assert!(g.has_type(Khid::from_raw(1), "Person"));
}

#[test]
fn uncommitted_does_not_replay() {
    let recs = vec![
        Rec::Begin { tx: 1 },
        Rec::Vertex {
            tx: 1,
            id: Khid::from_raw(1),
            types: vec!["Person".to_string()],
            attrs: name("Ada"),
        },
        Rec::Begin { tx: 2 },
        Rec::Vertex {
            tx: 2,
            id: Khid::from_raw(2),
            types: vec!["Person".to_string()],
            attrs: name("Bob"),
        },
        Rec::Commit { tx: 2 },
    ];
    let g = wal::replay(1, &recs).unwrap();
    assert!(g.vertex(Khid::from_raw(1)).is_none());
    assert_eq!(g.vertex(Khid::from_raw(2)).unwrap().get("name"), Some("Bob"));
}

#[test]
fn body_is_on_the_log() {
    let mut attrs = HashMap::new();
    attrs.insert("title".to_string(), Prop::from_str("Ada"));
    attrs.insert("body".to_string(), Prop::from_str("a page that is not a posting"));
    let recs = vec![
        Rec::Begin { tx: 1 },
        Rec::Vertex {
            tx: 1,
            id: Khid::from_raw(1),
            types: vec!["Doc".to_string()],
            attrs: attrs,
        },
        Rec::Commit { tx: 1 },
    ];
    let g = wal::replay(1, &recs).unwrap();
    assert_eq!(g.vertex(Khid::from_raw(1)).unwrap().get("body"),
               Some("a page that is not a posting"));
}

#[test]
fn edge_after_ends() {
    let recs = vec![
        Rec::Begin { tx: 1 },
        Rec::Vertex {
            tx: 1,
            id: Khid::from_raw(1),
            types: vec!["Person".to_string()],
            attrs: name("Ada"),
        },
        Rec::Vertex {
            tx: 1,
            id: Khid::from_raw(2),
            types: vec!["Person".to_string()],
            attrs: name("Bob"),
        },
        Rec::Edge {
            tx: 1,
            id: Khid::from_raw(3),
            src: Khid::from_raw(1),
            dst: Khid::from_raw(2),
            ty: "KNOWS".to_string(),
            attrs: HashMap::new(),
        },
        Rec::Commit { tx: 1 },
    ];
    let g = wal::replay(1, &recs).unwrap();
    let e = g.edge(Khid::from_raw(3)).unwrap();
    assert_eq!(e.source(), Khid::from_raw(1));
    assert_eq!(e.target(), Khid::from_raw(2));
    assert_eq!(g.addr(e.source()).shard(), 1);
}

#[test]
fn far_edge_replays() {
    let recs = vec![
        Rec::Begin { tx: 1 },
        Rec::Vertex {
            tx: 1,
            id: Khid::from_raw(1),
            types: vec!["Doc".to_string()],
            attrs: name("Ada"),
        },
        Rec::FarEdge {
            tx: 1,
            id: Khid::from_raw(2),
            src: Khid::from_raw(1),
            dst: crate::Addr::new(2, Khid::from_raw(9)),
            ty: "CITES".to_string(),
            attrs: HashMap::new(),
        },
        Rec::Commit { tx: 1 },
    ];
    let g = wal::replay(1, &recs).unwrap();
    let e = g.edge(Khid::from_raw(2)).unwrap();
    assert_eq!(e.far(), Some(crate::Addr::new(2, Khid::from_raw(9))));
}

#[test]
fn bad_magic() {
    let err = wal::read(&mut Cursor::new(b"KHG4xxxx")).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}
