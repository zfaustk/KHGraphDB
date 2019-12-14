//! Store: log is truth. Commit syncs. Reopen replays.

use std::fs;
use crate::{Addr, Graph, Khid, Prop, Store};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khl1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn commit_survives_reopen() {
    let dir = tmp("survive");
    let ada;
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        ada = s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.graph_mut().unwrap().mark_content("Doc", "body");
        s.graph_mut().unwrap().create_index("Doc", "title");
        s.commit().unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert_eq!(s.graph().shard(), 1);
    assert_eq!(s.graph().vertex(ada).unwrap().get("name"), Some("Ada"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rollback_is_not_on_the_log() {
    let dir = tmp("rollback");
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
        s.begin().unwrap();
        s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
        s.rollback();
        assert!(s.graph().vertex_by_name("Bob").is_none());
        assert!(s.graph().vertex_by_name("Ada").is_some());
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex_by_name("Bob").is_none());
    assert!(s.graph().vertex_by_name("Ada").is_some());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn body_and_far_cite_roundtrip() {
    let dir = tmp("body");
    let far = Addr::new(2, Khid::from_raw(9));
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        s.graph_mut().unwrap().mark_content("Doc", "body");
        let mut p = std::collections::HashMap::new();
        p.insert("title".to_string(), Prop::from_str("Ada"));
        p.insert("body".to_string(), Prop::from_str("the page"));
        let id = s.graph_mut().unwrap().add_vertex_props(p, Some("Doc")).unwrap();
        s.graph_mut().unwrap().create_index("Doc", "title");
        s.graph_mut().unwrap().add_far_edge(id, far, Some("CITES")).unwrap();
        s.commit().unwrap();
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    let g: &Graph = s.graph();
    assert!(g.type_by_name("Doc").unwrap().is_content("body"));
    assert_eq!(g.find("Doc", "title", "Ada").len(), 1);
    let id = g.find("Doc", "title", "Ada")[0];
    assert_eq!(g.vertex(id).unwrap().get("body"), Some("the page"));
    let e = g.edge(g.vertex(id).unwrap().outgoing()[0]).unwrap();
    assert_eq!(e.far(), Some(far));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn compact_is_one_capture() {
    let dir = tmp("compact");
    {
        let mut s = Store::open(&dir, "notes", 1).unwrap();
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.commit().unwrap();
        s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
        s.commit().unwrap();
        let before = fs::metadata(dir.join("log")).unwrap().len();
        s.compact().unwrap();
        let after = fs::metadata(dir.join("log")).unwrap().len();
        assert!(after < before);
    }
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex_by_name("Ada").is_some());
    assert!(s.graph().vertex_by_name("Bob").is_some());
    let _ = fs::remove_dir_all(&dir);
}
