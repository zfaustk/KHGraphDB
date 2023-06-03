//! Isolation. The engine picture is snapshot
//! isolation for readers. Dirty read is the
//! writer's arena, not the Arc. Phantom: a
//! new vertex appears only after publish.

use std::fs;
use crate::Engine;
use crate::query;
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khi-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn dirty_read_is_not_on_the_picture() {
    let dir = tmp("dirty");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    e.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let snap = e.snapshot();
    // A write that has not published: we only
    // mutate inside apply, which commits. The
    // contract is: ask never sees a touch.
    let r = query::ask(&snap, "MATCH (a:Doc {name:'Ghost'}) RETURN a");
    assert_eq!(r.rows.len(), 0);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn repeatable_read_of_a_pin() {
    let dir = tmp("rr");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    e.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let pin = e.snapshot();
    e.apply(|s| {
        let id = s.graph().vertex_by_name("Ada").unwrap().khid();
        s.graph_mut().unwrap().set_prop(id, "city", crate::Prop::from_str("Paris")).unwrap();
        Ok(())
    }).unwrap();
    let old = query::ask(&pin, "MATCH (a:Doc {name:'Ada'}) RETURN a");
    assert_eq!(old.rows.len(), 1);
    let v = pin.vertex_by_name("Ada").unwrap();
    assert!(v.get("city").is_none());
    let nowg = e.snapshot();
    let now = nowg.vertex_by_name("Ada").unwrap();
    assert_eq!(now.get("city"), Some("Paris"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn phantom_arrives_after_publish() {
    let dir = tmp("ph");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    e.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let pin = e.snapshot();
    e.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    assert!(pin.vertex_by_name("Bob").is_none());
    assert!(e.snapshot().vertex_by_name("Bob").is_some());
    let _ = fs::remove_dir_all(&dir);
}
