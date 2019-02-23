//! Two shards in one process. A cite is an Addr.
//! Hydrate fills a stub. The page stays at home.

use std::collections::HashMap;
use khgraphdb::{Addr, Catalog, Prop};

fn doc(title: &str, body: &str) -> HashMap<String, Prop> {
    let mut m = HashMap::new();
    m.insert("title".to_string(), Prop::from_str(title));
    m.insert("body".to_string(), Prop::from_str(body));
    m
}

fn main() {
    let mut cat = Catalog::new();
    cat.create("notes").unwrap();
    cat.create("other").unwrap();

    let ada = {
        let o = cat.graph_mut("other").unwrap();
        o.mark_content("Doc", "body");
        o.add_vertex_props(doc("Ada", "the page lives on other"), Some("Doc"))
            .unwrap()
    };
    let far = Addr::new(cat.graph("other").unwrap().shard(), ada);

    {
        let n = cat.graph_mut("notes").unwrap();
        n.mark_content("Doc", "body");
        n.create_index("Doc", "title");
        let notes = n.add_vertex_props(doc("Notes", "cites Ada elsewhere"), Some("Doc"))
            .unwrap();
        n.add_far_edge(notes, far, Some("CITES")).unwrap();
    }

    let home = cat.graph("notes").unwrap().shard();
    cat.fill_stub(home, far);
    let notes = cat.graph("notes").unwrap();
    println!("notes stub: {}", notes.stub(far).unwrap().title());
    println!("other body: {}",
             cat.graph("other").unwrap().vertex(ada).unwrap().get("body").unwrap_or(""));
}
