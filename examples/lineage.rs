//! The hops are the why. MATCH, not a verb.

use std::collections::HashMap;
use khgraphdb::{ask_query, keep_view, Graph};

fn named(name: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("name".to_string(), name.to_string());
    m
}

fn main() {
    let mut g = Graph::named("notes");
    let e = g.episode().unwrap();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let _ = g.add_vertex(named("Ada"), Some("Doc")).unwrap();
    keep_view(&mut g, "Hit", 0).unwrap();
    let from = ask_query(&g, "MATCH (h:Hit)-[:DERIVED_FROM]->(d) RETURN d");
    let inn = ask_query(&g, "MATCH (h:Hit)-[:IN]->(e:Episode) RETURN e");
    println!("episode {}", e);
    println!("derived {}", from.rows.len());
    println!("in {}", inn.rows.len());
}
