//! Print ids. The page stays at home.

use std::collections::HashMap;
use khgraphdb::{ask_query, keep_view, Graph, Val};

fn named(name: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("name".to_string(), name.to_string());
    m
}

fn main() {
    let mut g = Graph::named("notes");
    let _ = g.add_vertex(named("Ada"), Some("Doc")).unwrap();
    let _ = g.add_vertex(named("Alan"), Some("Doc")).unwrap();
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    keep_view(&mut g, "Hit", 0).unwrap();
    let local = ask_query(&g, "MATCH (h:Hit) RETURN h");
    println!("local {}", local.rows.len());
    for row in local.rows.iter() {
        match row.first().and_then(|c| c.as_ref()) {
            Some(Val::Id(id)) => println!("id {}", id),
            _ => {}
        }
    }
    g.mark_view("Report", "MATCH (h:Hit) RETURN h");
    let n = keep_view(&mut g, "Report", 1).unwrap();
    println!("report {}", n);
    let cost = ask_query(&g, "EXPLAIN MATCH (h:Hit) RETURN h");
    println!("{}", cost.message);
}
