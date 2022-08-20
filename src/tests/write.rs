//! Blob-first. The page is not a WAL record.
//! A pin names a serial.

use std::fs;
use crate::{query, Store};
use crate::prop::Prop;
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khw-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

fn page() -> std::collections::HashMap<String, Prop> {
    let mut m = std::collections::HashMap::new();
    m.insert("name".to_string(), Prop::from_str("Ada"));
    m.insert("body".to_string(), Prop::from_str("a notebook page that must not sit in the log"));
    m
}

#[test]
fn body_is_not_in_the_log() {
    let dir = tmp("notlog");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().mark_content("Doc", "body");
    s.graph_mut().unwrap().add_vertex_props(page(), Some("Doc")).unwrap();
    s.commit().unwrap();
    let log = fs::read(dir.join("log")).unwrap();
    let needle = b"a notebook page that must not sit in the log";
    assert!(log.windows(needle.len()).all(|w| w != needle));
    let blobdir = dir.join("blob");
    let mut found = false;
    for e in fs::read_dir(&blobdir).unwrap() {
        let b = fs::read(e.unwrap().path()).unwrap();
        if b.windows(needle.len()).any(|w| w == needle) {
            found = true;
        }
    }
    assert!(found);
    drop(s);
    let s = Store::open(&dir, "notes", 1).unwrap();
    let id = s.graph().vertex_by_name("Ada").unwrap().khid();
    assert_eq!(s.graph().vertex(id).unwrap().get("body"),
               Some("a notebook page that must not sit in the log"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pin_sees_the_old_page() {
    let dir = tmp("pin");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().mark_content("Doc", "body");
    s.graph_mut().unwrap().add_vertex_props(page(), Some("Doc")).unwrap();
    let a = s.commit().unwrap();
    let id = s.graph().vertex_by_name("Ada").unwrap().khid();
    s.graph_mut().unwrap().set_prop(id, "body", Prop::from_str("rewritten")).unwrap();
    s.commit().unwrap();
    let old = s.read_at(a).unwrap();
    assert_eq!(old.vertex(id).unwrap().get("body"),
               Some("a notebook page that must not sit in the log"));
    assert_eq!(s.graph().vertex(id).unwrap().get("body"), Some("rewritten"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn orphan_does_not_replay() {
    let dir = tmp("orph");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().mark_content("Doc", "body");
    s.graph_mut().unwrap().add_vertex_props(page(), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
    s.rollback();
    drop(s);
    let s = Store::open(&dir, "notes", 1).unwrap();
    assert!(s.graph().vertex_by_name("Bob").is_none());
    assert!(s.graph().vertex_by_name("Ada").is_some());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn replica_has_the_page() {
    let prim = tmp("wp");
    let copy = tmp("wr");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().mark_content("Doc", "body");
    s.graph_mut().unwrap().add_vertex_props(page(), Some("Doc")).unwrap();
    s.commit().unwrap();
    let r = Store::tail(&copy, &prim, "notes").unwrap();
    let id = r.graph().vertex_by_name("Ada").unwrap().khid();
    assert_eq!(r.graph().vertex(id).unwrap().get("body"),
               Some("a notebook page that must not sit in the log"));
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn compact_drops_orphans() {
    let dir = tmp("gc");
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().mark_content("Doc", "body");
    s.graph_mut().unwrap().add_vertex_props(page(), Some("Doc")).unwrap();
    s.commit().unwrap();
    let id = s.graph().vertex_by_name("Ada").unwrap().khid();
    s.graph_mut().unwrap().set_prop(id, "body", Prop::from_str("v2")).unwrap();
    s.commit().unwrap();
    s.compact().unwrap();
    let mut n = 0;
    for e in fs::read_dir(dir.join("blob")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().map(|x| x == "tmp").unwrap_or(false) {
            continue;
        }
        n += 1;
    }
    assert_eq!(n, 1);
    let r = query::ask(s.graph(), "MATCH (a:Doc {name:'Ada'}) RETURN a");
    assert!(r.ok);
    let _ = fs::remove_dir_all(&dir);
}
