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
fn find_over_tcp_is_one_round() {
    let prim = tmp("p-find");
    let mut s = Store::open(&prim, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.put_vertex(doc("Ada"), Some("Doc")).unwrap();
    s.commit().unwrap();
    let listener = wire::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = prim.clone();
    let g = s.graph().clone();
    thread::spawn(move || {
        let (st, _) = listener.accept().unwrap();
        let _ = wire::handle(&dir, &g, st);
    });
    let found = wire::find(addr, "Doc", "name", "Ada").unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].shard(), s.graph().shard());
    let _ = fs::remove_dir_all(&prim);
}

#[test]
fn route_fans_out_once() {
    let a = tmp("p-ra");
    let b = tmp("p-rb");
    let mut sa = Store::open(&a, "notes", 1).unwrap();
    sa.graph_mut().unwrap().create_index("Doc", "name");
    sa.put_vertex(doc("Ada"), Some("Doc")).unwrap();
    sa.commit().unwrap();
    let mut sb = Store::open(&b, "other", 2).unwrap();
    sb.graph_mut().unwrap().create_index("Doc", "name");
    sb.put_vertex(doc("Bob"), Some("Doc")).unwrap();
    sb.commit().unwrap();
    let la = wire::bind("127.0.0.1:0").unwrap();
    let lb = wire::bind("127.0.0.1:0").unwrap();
    let aa = la.local_addr().unwrap();
    let ab = lb.local_addr().unwrap();
    let da = a.clone();
    let db = b.clone();
    let ga = sa.graph().clone();
    let gb = sb.graph().clone();
    thread::spawn(move || {
        let mut i = 0;
        while i < 4 {
            let (st, _) = la.accept().unwrap();
            let _ = wire::handle(&da, &ga, st);
            i += 1;
        }
    });
    thread::spawn(move || {
        let mut i = 0;
        while i < 4 {
            let (st, _) = lb.accept().unwrap();
            let _ = wire::handle(&db, &gb, st);
            i += 1;
        }
    });
    let mut rt = crate::Route::new();
    rt.add(sa.graph().shard(), aa);
    rt.add(sb.graph().shard(), ab);
    let ada = rt.locate("Doc", "name", "Ada").unwrap();
    let bob = rt.locate("Doc", "name", "Bob").unwrap();
    assert_eq!(ada.len(), 1);
    assert_eq!(bob.len(), 1);
    assert_eq!(ada[0].shard(), sa.graph().shard());
    assert_eq!(bob[0].shard(), sb.graph().shard());
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
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