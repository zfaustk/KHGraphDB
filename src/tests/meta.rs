//! KHM1 is derived. FIND reads it.

use std::fs;
use crate::{Catalog, Meta, Store};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khl1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn commit_writes_meta() {
    let dir = tmp("meta-c");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let m = Meta::open(&dir).unwrap();
    assert_eq!(m.find("Doc", "name", "Ada").len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn drop_meta_rebuilds() {
    let dir = tmp("meta-r");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let _ = fs::remove_file(dir.join("meta"));
    let s = Store::open(&dir, "notes", 1).unwrap();
    Meta::rebuild(&dir, s.graph()).unwrap();
    let m = Meta::open(&dir).unwrap();
    assert_eq!(m.find("Doc", "name", "Ada").len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn compact_keeps_find() {
    let dir = tmp("meta-k");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.compact().unwrap();
    let m = Meta::open(&dir).unwrap();
    assert_eq!(m.find("Doc", "name", "Ada").len(), 1);
    assert_eq!(m.find("Doc", "name", "Bob").len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn catalog_locate_two_shards() {
    let mut cat = Catalog::new();
    cat.create("notes").unwrap();
    cat.create("other").unwrap();
    {
        let n = cat.graph_mut("notes").unwrap();
        n.create_index("Doc", "name");
        n.add_vertex(attrs("Notes"), Some("Doc")).unwrap();
    }
    {
        let o = cat.graph_mut("other").unwrap();
        o.create_index("Doc", "name");
        o.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    }
    let found = cat.locate("Doc", "name", "Ada");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].shard(), cat.graph("other").unwrap().shard());
}

#[test]
fn find_statement() {
    let mut g = crate::Graph::new();
    g.create_index("Doc", "name");
    g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let r = crate::query::run(&mut g, "FIND Doc name 'Ada'");
    assert!(r.ok);
    assert_eq!(r.rows.len(), 1);
}

#[test]
fn replica_finds_after_compact() {
    let prim = tmp("p-mf");
    let copy = tmp("r-mf");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let bm0 = s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let bm = s.compact().unwrap();
    assert!(!r.pos().unwrap().honors(bm));
    r.honor(&prim, bm).unwrap();
    assert!(r.pos().unwrap().honors(bm));
    let m = Meta::open(&copy).unwrap();
    assert_eq!(m.find("Doc", "name", "Ada").len(), 1);
    assert_eq!(m.find("Doc", "name", "Bob").len(), 1);
    assert!(bm0.generation() < bm.generation());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}