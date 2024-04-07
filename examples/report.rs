//! A community is a Type. A report is a view.

use std::collections::HashMap;
use khgraphdb::{keep_view, Graph};

fn named(name: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("name".to_string(), name.to_string());
    m
}

fn main() {
    let mut g = Graph::named("lab");
    let lab = g.community_as("lab").unwrap();
    let org = g.community_as("org").unwrap();
    g.enclose(lab, org).unwrap();
    let ada = g.add_vertex(named("Ada"), Some("Doc")).unwrap();
    let alan = g.add_vertex(named("Alan"), Some("Doc")).unwrap();
    g.enclose(ada, lab).unwrap();
    g.enclose(alan, lab).unwrap();
    g.mark_view("Report", "MATCH (d:Doc)-[:IN]->(:Community {name:'lab'}) RETURN d");
    let n = keep_view(&mut g, "Report", 0).unwrap();
    println!("community {}", lab);
    println!("nested {}", g.episode_of(lab).len());
    println!("report {}", n);
}
