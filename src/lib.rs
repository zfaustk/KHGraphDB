//! KHGraphDB. A directed property graph.
//! Type is a first-class object. KHID is identity.

pub use error::{Error, Result};
pub use graph::Graph;
pub use vertex::Vertex;
pub use edge::Edge;
pub use ty::Type;

pub mod error;
pub mod graph;
pub mod vertex;
pub mod edge;
pub mod ty;
pub mod index;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::Graph;

    fn attrs(name: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("name".to_string(), name.to_string());
        m
    }

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
    fn multi_type() {
        let mut g = Graph::new();
        let ada = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
        assert!(g.add_type_to_vertex(&ada, "Author").unwrap());
        assert!(g.has_type(&ada, "Person"));
        assert!(g.has_type(&ada, "Author"));
        assert_eq!(g.type_by_name("Author").unwrap().vertex_count(), 1);
    }
}
