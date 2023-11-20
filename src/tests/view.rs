//! Extra cases for MARK VIEW.

use crate::ty::{Type, hash_view};
use crate::Khid;

#[test]
fn empty_recipe_is_refused() {
    let mut t = Type::new(Khid::from_raw(1), "Hit".to_string());
    assert!(!t.mark_view(""));
}

#[test]
fn empty_hash_is_the_offset() {
    assert_eq!(hash_view(""), 0xcbf29ce484222325);
}

use crate::{Graph, wal::{self, Rec}};

#[test]
fn graph_refuses_a_blank_view() {
    let mut g = Graph::new();
    assert!(!g.mark_view("Hit", ""));
    assert!(!g.mark_view("", "MATCH (a) RETURN a"));
}

#[test]
fn uncommitted_view_does_not_replay() {
    let recs = vec![
        Rec::Begin { tx: 1 },
        Rec::View {
            tx: 1,
            type_name: "Hit".to_string(),
            query: "MATCH (a) RETURN a".to_string(),
        },
        Rec::Begin { tx: 2 },
        Rec::View {
            tx: 2,
            type_name: "Hit".to_string(),
            query: "MATCH (b) RETURN b".to_string(),
        },
        Rec::Commit { tx: 2 },
    ];
    let g = wal::replay(1, &recs).unwrap();
    assert_eq!(g.view_of("Hit"), Some("MATCH (b) RETURN b"));
}
