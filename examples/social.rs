//! Alice knows Bob knows Carol. Then Ada.
//! The C# sample in csharp/Samples/Social, in Rust.

use std::collections::HashMap;
use khgraphdb::{Graph, Khid};
use khgraphdb::query;

fn attrs(name: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("name".to_string(), name.to_string());
    m
}

fn main() {
    let mut g = Graph::named("social");
    g.create_unique("Person", "name");
    let alice = g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
    let bob = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
    let carol = g.add_vertex(attrs("Carol"), Some("Person")).unwrap();
    g.add_edge(alice, bob, Some("KNOWS")).unwrap();
    g.add_edge(bob, carol, Some("KNOWS")).unwrap();

    query::run(&mut g, "MERGE (p:Person {name:'Ada'})");
    query::run(&mut g,
               "MERGE (a:Person {name:'Alice'})-[:KNOWS]->(b:Person {name:'Ada'})");

    let r = query::run(&mut g,
        "MATCH (a:Person {name:'Alice'})-[:KNOWS]->(b) RETURN b");
    println!("{}", r.message);
    for row in r.rows.iter() {
        let id = match row.get(0).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
            Some(s) => s,
            None => continue,
        };
        match Khid::parse(id).and_then(|k| g.vertex(k)).and_then(|v| v.get("name")).map(|s| s.to_string()) {
            Some(n) => println!("  {}", n),
            None => println!("  {}", id),
        }
    }
}
