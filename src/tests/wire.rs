//! Pull and hydrate over a socket. Commit does not wait.

use std::collections::HashMap;
use std::fs;
use std::thread;
use crate::{wire, Addr, Prop, Store};

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khl1-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

fn doc(name: &str) -> HashMap<String, Prop> {
    let mut m = HashMap::new();
    m.insert("name".to_string(), Prop::from_str(name));
    m
}

#[test]
fn pull_appends_over_tcp() {
    let prim = tmp("p-tcp");
    let copy = tmp("r-tcp");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.put_vertex(doc("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    s.put_vertex(doc("Bob"), Some("Doc")).unwrap();
    let bookmark = s.commit().unwrap();
    let listener = wire::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = prim.clone();
    let g = s.graph().clone();
    thread::spawn(move || {
        let (st, _) = listener.accept().unwrap();
        let _ = wire::handle(&dir, &g, st);
    });
    r.follow(addr).unwrap();
    assert!(r.graph().vertex_by_name("Bob").is_some());
    assert!(r.pos().unwrap().prefix_of(bookmark) || r.pos().unwrap() == bookmark);
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}

#[test]
fn hydrate_is_one_round() {
    let prim = tmp("p-hy");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    let id = s.put_vertex(doc("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let far = Addr::new(s.graph().shard(), id);
    let missing = Addr::new(s.graph().shard(), crate::Khid::from_raw(99));
    let listener = wire::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = prim.clone();
    let g = s.graph().clone();
    thread::spawn(move || {
        let (st, _) = listener.accept().unwrap();
        let _ = wire::handle(&dir, &g, st);
    });
    let stubs = wire::get_stubs(addr, &[far, missing]).unwrap();
    assert_eq!(stubs.len(), 2);
    assert_eq!(stubs[0].as_ref().unwrap().title(), "Ada");
    assert!(stubs[1].is_none());
    let _ = fs::remove_dir_all(&prim);
}

#[test]
fn compact_over_tcp_replaces() {
    let prim = tmp("p-cmp");
    let copy = tmp("r-cmp");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.put_vertex(doc("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    s.put_vertex(doc("Bob"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let mut r = Store::tail(&copy, &prim, "notes").unwrap();
    s.compact().unwrap();
    let listener = wire::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = prim.clone();
    let g = s.graph().clone();
    thread::spawn(move || {
        let (st, _) = listener.accept().unwrap();
        let _ = wire::handle(&dir, &g, st);
    });
    r.follow(addr).unwrap();
    assert_eq!(r.generation(), s.generation());
    assert!(r.graph().vertex_by_name("Ada").is_some());
    assert!(r.graph().vertex_by_name("Bob").is_some());
    let _ = fs::remove_dir_all(&prim);
    let _ = fs::remove_dir_all(&copy);
}