//! KHGraphDB. A directed property graph.
//! Type is a first-class object. KHID is identity.

pub use error::{Error, Result};
pub use graph::Graph;
pub use vertex::Vertex;
pub use edge::Edge;
pub use ty::Type;
pub use query::{run as run_query, QueryResult, Val, Path};

pub mod error;
pub mod graph;
pub mod vertex;
pub mod edge;
pub mod ty;
pub mod index;
pub mod io;
pub mod algo;
pub mod query;

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

    #[test]
    fn weighted_shortest() {
        let mut g = Graph::new();
        let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
        let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
        let c = g.add_vertex(attrs("C"), Some("City")).unwrap();
        let ab = g.add_edge(&a, &b, Some("ROAD")).unwrap();
        let bc = g.add_edge(&b, &c, Some("ROAD")).unwrap();
        let ac = g.add_edge(&a, &c, Some("ROAD")).unwrap();
        g.set_edge_attr(&ab, "weight", "2");
        g.set_edge_attr(&bc, "weight", "2");
        g.set_edge_attr(&ac, "weight", "5");
        let s = super::algo::shortest(&g, &a, &c).unwrap();
        assert_eq!(s, vec![a.clone(), b.clone(), c.clone()]);
    }

    fn social() -> Graph {
        let mut g = Graph::new();
        g.create_index("Person", "name");
        let alice = g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
        let bob = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
        let _carol = g.add_vertex(attrs("Carol"), Some("Person")).unwrap();
        g.add_edge(&alice, &bob, Some("KNOWS")).unwrap();
        g.add_edge(&bob, &_carol, Some("KNOWS")).unwrap();
        g
    }

    #[test]
    fn match_one_hop() {
        let mut g = social();
        let r = super::query::run(&mut g, "MATCH (a:Person)-[:KNOWS]->(b:Person)");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 2);
    }

    #[test]
    fn match_where() {
        let mut g = social();
        let r = super::query::run(&mut g,
                                 "MATCH (a:Person)-[:KNOWS]->(b) WHERE a.name = 'Alice'");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 1);
    }

    #[test]
    fn match_unknown_type() {
        let mut g = social();
        let r = super::query::run(&mut g, "MATCH (n:Nope)");
        assert!(!r.ok);
    }

    #[test]
    fn match_unknown_rel() {
        let mut g = social();
        let r = super::query::run(&mut g, "MATCH (a)-[:Nope]->(b)");
        assert!(!r.ok);
    }

    #[test]
    fn remove_property() {
        let mut g = social();
        super::query::run(&mut g, "MATCH (a:Person {name:'Alice'}) SET a.city = 'London'");
        let r = super::query::run(&mut g,
            "MATCH (a:Person {name:'Alice'}) REMOVE a.city");
        assert!(r.ok);
        let alice = g.vertex_by_name("Alice").unwrap();
        assert!(alice.get("city").is_none());
    }

    #[test]
    fn delete_free_node() {
        let mut g = Graph::new();
        super::query::run(&mut g, "CREATE (n:Person {name:'Solo'})");
        let r = super::query::run(&mut g, "MATCH (n:Person {name:'Solo'}) DELETE n");
        assert!(r.ok);
        assert_eq!(g.vertex_count(), 0);
    }

    #[test]
    fn delete_refuses_edges() {
        let mut g = social();
        let r = super::query::run(&mut g, "MATCH (n:Person {name:'Alice'}) DELETE n");
        assert!(!r.ok);
        assert_eq!(g.vertex_count(), 3);
    }

    #[test]
    fn set_property() {
        let mut g = social();
        let r = super::query::run(&mut g,
            "MATCH (a:Person {name:'Alice'}) SET a.city = 'London'");
        assert!(r.ok);
        let alice = g.vertex_by_name("Alice").unwrap();
        assert_eq!(alice.get("city"), Some("London"));
    }

    #[test]
    fn create_node() {
        let mut g = Graph::new();
        let r = super::query::run(&mut g, "CREATE (n:Person {name:'Ada'})");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 1);
        assert_eq!(g.vertex_count(), 1);
        assert!(g.vertex_by_name("Ada").is_some());
        assert!(g.has_type(g.vertex_by_name("Ada").unwrap().khid(), "Person"));
    }

    #[test]
    fn create_edge() {
        let mut g = Graph::new();
        let r = super::query::run(&mut g,
            "CREATE (a:Person {name:'Ada'})-[:KNOWS]->(b:Person {name:'Bob'})");
        assert!(r.ok);
        assert_eq!(g.vertex_count(), 2);
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.edges_of_type("KNOWS").len(), 1);
    }

    #[test]
    fn merge_ada() {
        let mut g = social();
        let r = super::query::run(&mut g, "MERGE (p:Person {name:'Ada'})");
        assert!(r.ok);
        assert_eq!(r.message, "created");
        let r2 = super::query::run(&mut g, "MERGE (p:Person {name:'Ada'})");
        assert_eq!(r2.message, "exists");
        assert_eq!(g.vertex_count(), 4);
    }

    #[test]
    fn optional_nobody() {
        let mut g = social();
        let r = super::query::run(&mut g,
                                 "OPTIONAL MATCH (a:Person {name:'Nobody'})-[:KNOWS]->(b)");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 1);
    }

    #[test]
    fn match_star_hops() {
        let mut g = social();
        let r = super::query::run(&mut g,
            "MATCH (a:Person {name:'Alice'})-[:KNOWS*1..2]->(b)");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 2);
    }

    #[test]
    fn match_star_exact() {
        let mut g = social();
        let r = super::query::run(&mut g,
            "MATCH (a:Person {name:'Alice'})-[:KNOWS*2]->(b)");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 1);
    }

    #[test]
    fn match_star_zero() {
        let mut g = social();
        let r = super::query::run(&mut g,
            "MATCH (a:Person {name:'Alice'})-[:KNOWS*0..1]->(b:Person)");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 2);
    }

    #[test]
    fn match_star_cycle_lid() {
        let mut g = social();
        let alice = g.vertex_by_name("Alice").unwrap().khid().to_string();
        let carol = g.vertex_by_name("Carol").unwrap().khid().to_string();
        g.add_edge(&carol, &alice, Some("KNOWS")).unwrap();
        let r = super::query::run(&mut g,
            "MATCH (a:Person {name:'Alice'})-[:KNOWS*1..8]->(b)");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 2);
    }

    #[test]
    fn val_list() {
        let v = super::Val::List(vec![
            super::Val::Id("k1".to_string()),
            super::Val::Id("k2".to_string()),
        ]);
        assert_eq!(v.as_list().unwrap().len(), 2);
        assert!(v.as_id().is_none());
        assert!(v.as_path().is_none());
    }

    #[test]
    fn match_rel_bind() {
        let mut g = social();
        let r = super::query::run(&mut g,
            "MATCH (a:Person {name:'Alice'})-[e:KNOWS]->(b)");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 1);
        assert_eq!(r.columns, vec!["a".to_string(), "e".to_string(), "b".to_string()]);
        let e = r.rows[0][1].as_ref().and_then(|v| v.as_id()).unwrap();
        let alice = g.vertex_by_name("Alice").unwrap().khid().to_string();
        assert_eq!(g.edge(e).unwrap().source(), alice);
    }

    #[test]
    fn match_rel_star_list() {
        let mut g = social();
        let r = super::query::run(&mut g,
            "MATCH (a:Person {name:'Alice'})-[e:KNOWS*1..2]->(b)");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 2);
        let mut lens = Vec::new();
        for row in r.rows.iter() {
            let list = row[1].as_ref().and_then(|v| v.as_list()).unwrap();
            lens.push(list.len());
        }
        lens.sort();
        assert_eq!(lens, vec![1, 2]);
    }

    #[test]
    fn match_path_bind() {
        let mut g = social();
        let r = super::query::run(&mut g,
            "MATCH p = (a:Person {name:'Alice'})-[:KNOWS*1..2]->(b)");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 2);
        assert_eq!(&r.columns[0], "p");
        let mut lens = Vec::new();
        let mut hops = Vec::new();
        for row in r.rows.iter() {
            let p = row[0].as_ref().and_then(|v| v.as_path()).unwrap();
            lens.push(p.len());
            hops.push(p.hops());
        }
        lens.sort();
        hops.sort();
        assert_eq!(lens, vec![3, 5]);
        assert_eq!(hops, vec![1, 2]);
    }

    #[test]
    fn match_shortest_hops() {
        let mut g = Graph::new();
        let a = g.add_vertex(attrs("A"), Some("City")).unwrap();
        let b = g.add_vertex(attrs("B"), Some("City")).unwrap();
        let c = g.add_vertex(attrs("C"), Some("City")).unwrap();
        let ab = g.add_edge(&a, &b, Some("ROAD")).unwrap();
        let bc = g.add_edge(&b, &c, Some("ROAD")).unwrap();
        let ac = g.add_edge(&a, &c, Some("ROAD")).unwrap();
        g.set_edge_attr(&ab, "weight", "2");
        g.set_edge_attr(&bc, "weight", "2");
        g.set_edge_attr(&ac, "weight", "5");
        let r = super::query::run(&mut g,
            "MATCH p = shortestPath((x:City {name:'A'})-[:ROAD*]->(y:City {name:'C'}))");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 1);
        let p = r.rows[0][0].as_ref().and_then(|v| v.as_path()).unwrap();
        assert_eq!(p.len(), 3);
        assert_eq!(p.hops(), 1);
        assert_eq!(p[0], a);
        assert_eq!(p[1], ac);
        assert_eq!(p[2], c);
        assert_eq!(p.nodes(), vec![a.clone(), c.clone()]);
        assert_eq!(p.edges(), vec![ac.clone()]);
    }

    #[test]
    fn match_shortest_none() {
        let mut g = social();
        let r = super::query::run(&mut g,
            "MATCH p = shortestPath((a:Person {name:'Carol'})-[:KNOWS*]->(b:Person {name:'Alice'}))");
        assert!(r.ok);
        assert_eq!(r.rows.len(), 0);
    }
}
