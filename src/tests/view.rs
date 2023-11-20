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
