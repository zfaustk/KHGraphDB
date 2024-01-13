//! KEEP fills a view. The recipe is a read.
//! Hits cite addresses. Fold 0 is the world.

use std::collections::HashMap;
use khgraphdb::{keep_view, Graph, Prop};

fn named(name: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("name".to_string(), name.to_string());
    m
}

fn main() {
    let mut g = Graph::named("notes");
    g.mark_view("Hit", "MATCH (a:Doc) RETURN a");
    let _ = g.add_vertex(named("Ada"), Some("Doc")).unwrap();
    let _ = g.add_vertex(named("Bob"), Some("Doc")).unwrap();
    let n = keep_view(&mut g, "Hit", 0).unwrap();
    println!("hits {}", n);
    println!("members {}", g.type_by_name("Hit").unwrap().vertex_count());
    let again = keep_view(&mut g, "Hit", 0).unwrap();
    println!("again {}", again);

    g.mark_view("Out", "MATCH (h:Hit) RETURN h");
    println!("fold0 {}", keep_view(&mut g, "Out", 0).unwrap());
    println!("fold1 {}", keep_view(&mut g, "Out", 1).unwrap());

    let ada = g.vertex_by_name("Ada").unwrap().khid();
    let body = Prop::from_str("the page stays at home");
    let _ = g.set_prop(ada, "body", body);
    println!("Ada still a Doc, not a Hit");
}
