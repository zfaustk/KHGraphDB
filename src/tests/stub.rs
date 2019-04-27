//! A stub is a far title. Not the page.

use crate::{Addr, Catalog, Graph, Khid, Stub};
use super::common::attrs;

#[test]
fn stub_is_title_and_ver() {
    let s = Stub::new("Ada", 3);
    assert_eq!(s.title(), "Ada");
    assert_eq!(s.ver(), 3);
}

#[test]
fn graph_holds_a_stub() {
    let mut g = Graph::new();
    let a = Addr::new(2, Khid::from_raw(9));
    g.put_stub(a, "Ada", 1);
    assert_eq!(g.stub(a).unwrap().title(), "Ada");
    assert!(g.drop_stub(a));
    assert!(g.stub(a).is_none());
}

#[test]
fn rebuild_index_skips_content() {
    let mut g = Graph::new();
    g.mark_content("Doc", "body");
    g.create_index("Doc", "title");
    let mut p = std::collections::HashMap::new();
    p.insert("title".to_string(), crate::Prop::from_str("Ada"));
    p.insert("body".to_string(), crate::Prop::from_str("page"));
    let _ = g.add_vertex_props(p, Some("Doc")).unwrap();
    g.rebuild_index();
    assert!(g.has_index("Doc", "title"));
    assert!(!g.has_index("Doc", "body"));
    assert_eq!(g.find("Doc", "title", "Ada").len(), 1);
}

#[test]
fn catalog_hydrates_a_stub() {
    let mut cat = Catalog::new();
    cat.create("notes").unwrap();
    cat.create("other").unwrap();
    let other_shard = cat.graph("other").unwrap().shard();
    let home = cat.graph("notes").unwrap().shard();
    let id = {
        let o = cat.graph_mut("other").unwrap();
        o.add_vertex(attrs("Ada"), Some("Doc")).unwrap()
    };
    let addr = Addr::new(other_shard, id);
    assert!(cat.fill_stub(home, addr));
    assert_eq!(cat.graph("notes").unwrap().stub(addr).unwrap().title(), "Ada");
}

#[test]
fn cite_title_is_the_stub() {
    let mut g = Graph::new();
    let ada = g.add_vertex(attrs("Notes"), Some("Doc")).unwrap();
    let far = Addr::new(2, Khid::from_raw(9));
    let e = g.add_far_edge(ada, far, Some("CITES")).unwrap();
    assert_eq!(g.cite(e), Some(far));
    assert!(g.cite_title(e).is_none());
    g.put_stub(far, "Ada", 1);
    assert_eq!(g.cite_title(e), Some("Ada"));
}
