//! A notebook. Title is identity. Body is content.
//! A cite that leaves the box is an Addr.

use std::collections::HashMap;
use khgraphdb::{Addr, Graph, Khid, Prop};

fn doc(title: &str, body: &str) -> HashMap<String, Prop> {
    let mut m = HashMap::new();
    m.insert("title".to_string(), Prop::from_str(title));
    m.insert("body".to_string(), Prop::from_str(body));
    m
}

fn main() {
    let mut g = Graph::named("notes");
    g.mark_content("Doc", "body");
    g.create_index("Doc", "title");
    g.create_unique("Doc", "title");

    let ada = g.add_vertex_props(doc("Ada",
                                    "Notes on the engine. The page stays here."),
                                Some("Doc"))
        .unwrap();
    let notes = g.add_vertex_props(doc("Notes",
                                      "A notebook cites Ada. Body is not an index key."),
                                  Some("Doc"))
        .unwrap();
    g.add_edge(notes, ada, Some("CITES")).unwrap();

    let far = Addr::new(2, Khid::from_raw(9));
    g.add_far_edge(notes, far, Some("CITES")).unwrap();

    println!("shard {}", g.shard());
    println!("index title: {}", g.has_index("Doc", "title"));
    println!("index body: {}", g.has_index("Doc", "body"));
    for id in g.find("Doc", "title", "Ada").iter() {
        let v = g.vertex(*id).unwrap();
        println!("{}  {}", v.get("title").unwrap_or(""), v.get("body").unwrap_or(""));
    }
    for eid in g.vertex(notes).unwrap().outgoing().iter() {
        let e = g.edge(*eid).unwrap();
        if e.is_far() {
            println!("cite {}", e.far().unwrap());
        } else {
            println!("cite {}", e.target());
        }
    }
}
