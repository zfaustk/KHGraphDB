//! An episode is a vertex. KEEP hops IN.
//! A review is a view over that hop.

use std::collections::HashMap;
use khgraphdb::{keep_view, Graph};

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
    let n = keep_view(&mut g, "Hit", 0).unwrap();
    println!("episode {}", e);
    println!("kept {}", n);
    println!("in {}", g.in_episode(e).len());

    g.close_episode();
    g.mark_view("Review", "MATCH (h:Hit)-[:IN]->(:Episode) RETURN h");
    println!("review fold0 {}", keep_view(&mut g, "Review", 0).unwrap());
    println!("review fold1 {}", keep_view(&mut g, "Review", 1).unwrap());

    let _ = g.add_vertex(named("Bob"), Some("Doc")).unwrap();
    let extra = keep_view(&mut g, "Hit", 0).unwrap();
    println!("after close {}", extra);
    println!("still in {}", g.in_episode(e).len());
}
