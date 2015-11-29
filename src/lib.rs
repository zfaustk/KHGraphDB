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
pub mod io;
pub mod algo;

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
    fn snapshot() {
        use std::io::Cursor;
        let mut g = Graph::new();
        let ada = g.add_vertex(attrs("Ada"), Some("Person")).unwrap();
        g.add_type_to_vertex(&ada, "Author").unwrap();
        let bob = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
        g.add_edge(&ada, &bob, Some("KNOWS")).unwrap();
        let mut buf = Vec::new();
        super::io::write_graph(&g, &mut buf).unwrap();
        let mut cur = Cursor::new(buf);
        let h = super::io::read_graph(&mut cur).unwrap();
        assert_eq!(h.vertex_count(), 2);
        assert!(h.vertex_by_name("Ada").is_some());
        let ada2 = h.vertex_by_name("Ada").unwrap().khid().to_string();
        assert!(h.has_type(&ada2, "Author"));
        assert_eq!(h.edges_of_type("KNOWS").len(), 1);
    }

    #[test]
    fn bfs_and_dijkstra() {
        let mut g = Graph::new();
        let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
        let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
        let c = g.add_vertex(attrs("C"), Some("City")).unwrap();
        g.add_edge(&a, &b, Some("ROAD")).unwrap();
        g.add_edge(&b, &c, Some("ROAD")).unwrap();
        g.add_edge(&a, &c, Some("ROAD")).unwrap();
        let near = super::algo::nearby(&g, &a, 2);
        assert!(near.len() >= 1);
        let p = super::algo::path(&g, &a, &c).unwrap();
        assert_eq!(p[0], a);
        assert_eq!(*p.last().unwrap(), c);
        assert!(!super::algo::has_cycle(&g));
        let s = super::algo::shortest(&g, &a, &c).unwrap();
        assert_eq!(s.last().unwrap(), &c);
    }
}
