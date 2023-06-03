//! Concurrent ask. One writer. The picture is an Arc.

use std::fs;
use std::sync::Arc;
use std::thread;
use crate::Engine;
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khe-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn ask_sees_commit_not_the_tail() {
    let dir = tmp("pic");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    e.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let r = e.ask("MATCH (a:Doc {name:'Ada'}) RETURN a");
    assert_eq!(r.rows.len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn readers_share_the_picture() {
    let dir = tmp("rd");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    e.apply(|s| {
        let mut i = 0;
        while i < 20 {
            s.graph_mut().unwrap().add_vertex(attrs(&format!("n{}", i)), Some("Doc")).unwrap();
            i += 1;
        }
        Ok(())
    }).unwrap();
    let e = Arc::new(e);
    let mut hs = Vec::new();
    let mut t = 0;
    while t < 4 {
        let ee = e.clone();
        hs.push(thread::spawn(move || {
            let mut i = 0;
            while i < 30 {
                let r = ee.ask("MATCH (a:Doc) RETURN count(a)");
                assert!(r.ok);
                i += 1;
            }
        }));
        t += 1;
    }
    let w = e.clone();
    let hw = thread::spawn(move || {
        w.apply(|s| {
            s.graph_mut().unwrap().add_vertex(attrs("zed"), Some("Doc")).unwrap();
            Ok(())
        }).unwrap();
    });
    for h in hs {
        h.join().unwrap();
    }
    hw.join().unwrap();
    let r = e.ask("MATCH (a:Doc {name:'zed'}) RETURN a");
    assert_eq!(r.rows.len(), 1);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn snapshot_does_not_move_with_a_live_write() {
    let dir = tmp("st");
    let e = Engine::open(&dir, "notes", 1).unwrap();
    e.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let before = e.snapshot();
    e.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    assert_eq!(before.vertex_count(), 1);
    assert_eq!(e.snapshot().vertex_count(), 2);
    let _ = fs::remove_dir_all(&dir);
}
