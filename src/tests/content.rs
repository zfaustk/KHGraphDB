//! Content keys stay off the posting list.

use std::collections::HashMap;
use crate::{Graph, Prop};

fn title_body(title: &str, body: &str) -> HashMap<String, Prop> {
    let mut m = HashMap::new();
    m.insert("title".to_string(), Prop::from_str(title));
    m.insert("body".to_string(), Prop::from_str(body));
    m
}

#[test]
fn index_refuses_a_content_key() {
    let mut g = Graph::new();
    assert!(g.mark_content("Doc", "body"));
    assert!(!g.create_index("Doc", "body"));
    assert!(g.create_index("Doc", "title"));
    let _ = g.add_vertex_props(title_body("Ada", "a long page"), Some("Doc")).unwrap();
    assert!(g.has_index("Doc", "title"));
    assert!(!g.has_index("Doc", "body"));
    assert_eq!(g.find("Doc", "title", "Ada").len(), 1);
    assert_eq!(g.find("Doc", "body", "a long page").len(), 1);
}

#[test]
fn mark_content_drops_a_posting() {
    let mut g = Graph::new();
    g.create_index("Doc", "body");
    let _ = g.add_vertex_props(title_body("Ada", "page"), Some("Doc")).unwrap();
    assert!(g.has_index("Doc", "body"));
    assert!(g.mark_content("Doc", "body"));
    assert!(!g.has_index("Doc", "body"));
    assert!(g.type_by_name("Doc").unwrap().is_content("body"));
}

#[test]
fn body_survives_on_the_vertex() {
    let mut g = Graph::new();
    g.mark_content("Doc", "body");
    let id = g.add_vertex_props(title_body("Ada", "the page"), Some("Doc")).unwrap();
    assert_eq!(g.vertex(id).unwrap().get("body"), Some("the page"));
}
