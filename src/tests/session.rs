//! OCC SI. First committer wins. Write skew lives.

use std::fs;
use std::sync::Arc;
use std::thread;
use crate::{Engine, Prop};
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("kho-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

fn seed_ada(e: &Engine) -> crate::Khid {
    e.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    e.snapshot().vertex_by_name("Ada").unwrap().khid()
}

#[test]
fn own_write_is_visible() {
    let dir = tmp("own");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    let id = seed_ada(&e);
    let mut s = e.session();
    s.set(id, "city", Prop::from_str("Paris")).unwrap();
    let r = s.ask("MATCH (a:Doc {name:'Ada'}) RETURN a");
    assert!(r.ok);
    s.commit().unwrap();
    let g = e.snapshot();
    let v = g.vertex_by_name("Ada").unwrap();
    assert_eq!(v.get("city"), Some("Paris"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn lost_update_aborts_the_second() {
    let dir = tmp("lu");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    let id = seed_ada(&e);
    let mut a = e.session();
    let mut b = e.session();
    a.set(id, "n", Prop::from_int(1)).unwrap();
    b.set(id, "n", Prop::from_int(2)).unwrap();
    a.commit().unwrap();
    assert!(b.commit().is_err());
    let g = e.snapshot();
    let v = g.vertex(id).unwrap();
    assert_eq!(v.get_prop("n"), Some(&Prop::from_int(1)));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn disjoint_writes_both_commit() {
    let dir = tmp("dj");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    e.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let ada = e.snapshot().vertex_by_name("Ada").unwrap().khid();
    let bob = e.snapshot().vertex_by_name("Bob").unwrap().khid();
    let mut a = e.session();
    let mut b = e.session();
    a.set(ada, "city", Prop::from_str("Paris")).unwrap();
    b.set(bob, "city", Prop::from_str("London")).unwrap();
    a.commit().unwrap();
    b.commit().unwrap();
    let g = e.snapshot();
    assert_eq!(g.vertex(ada).unwrap().get("city"), Some("Paris"));
    assert_eq!(g.vertex(bob).unwrap().get("city"), Some("London"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn write_skew_is_si() {
    let dir = tmp("ws");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    e.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let ada = e.snapshot().vertex_by_name("Ada").unwrap().khid();
    let bob = e.snapshot().vertex_by_name("Bob").unwrap().khid();
    let mut a = e.session();
    let mut b = e.session();
    let _ = a.vertex_name(ada);
    a.set(bob, "flag", Prop::from_int(1)).unwrap();
    let _ = b.vertex_name(bob);
    b.set(ada, "flag", Prop::from_int(1)).unwrap();
    a.commit().unwrap();
    b.commit().unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn abort_does_not_publish() {
    let dir = tmp("ab");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    let mut s = e.session();
    s.create("Ghost", "Doc");
    s.abort();
    assert!(e.snapshot().vertex_by_name("Ghost").is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn drop_aborts() {
    let dir = tmp("dr");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    {
        let mut s = e.session();
        s.create("Ghost", "Doc");
    }
    assert!(e.snapshot().vertex_by_name("Ghost").is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn create_then_find() {
    let dir = tmp("cr");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    let mut s = e.session();
    let id = s.create("Zed", "Doc");
    s.commit().unwrap();
    let g = e.snapshot();
    assert_eq!(g.vertex(id).unwrap().get("name"), Some("Zed"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn concurrent_lost_update() {
    let dir = tmp("thr");
    let e = Arc::new(Engine::open(&dir, "notes", 1).unwrap());
    let id = seed_ada(&e);
    let a = e.clone();
    let b = e.clone();
    let ha = thread::spawn(move || {
        let mut s = a.session();
        s.set(id, "n", Prop::from_int(1)).unwrap();
        s.commit()
    });
    let hb = thread::spawn(move || {
        let mut s = b.session();
        s.set(id, "n", Prop::from_int(2)).unwrap();
        s.commit()
    });
    let ra = ha.join().unwrap();
    let rb = hb.join().unwrap();
    let wins = ra.is_ok() as u8 + rb.is_ok() as u8;
    assert_eq!(wins, 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn apply_conflicts_a_session() {
    let dir = tmp("ap");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    let id = seed_ada(&e);
    let mut s = e.session();
    s.set(id, "n", Prop::from_int(1)).unwrap();
    e.apply(|st| {
        st.graph_mut().unwrap().set_prop(id, "n", Prop::from_int(9)).unwrap();
        Ok(())
    }).unwrap();
    assert!(s.commit().is_err());
    let g = e.snapshot();
    assert_eq!(g.vertex(id).unwrap().get_prop("n"), Some(&Prop::from_int(9)));
    let _ = fs::remove_dir_all(&dir);
}
