//! Shared fixtures.

use std::collections::HashMap;
use super::super::Graph;

pub fn attrs(name: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("name".to_string(), name.to_string());
    m
}

pub fn social() -> Graph {
    let mut g = Graph::new();
    g.create_index("Person", "name");
    let alice = g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
    let bob = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
    let _carol = g.add_vertex(attrs("Carol"), Some("Person")).unwrap();
    g.add_edge(&alice, &bob, Some("KNOWS")).unwrap();
    g.add_edge(&bob, &_carol, Some("KNOWS")).unwrap();
    g
}
