//! Primary plus copies. min_ack. Failover.

use std::fs;
use crate::ReplSet;
use super::common::attrs;

fn tmp(name: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("khrs-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&p);
    p
}

#[test]
fn one_copy_honors() {
    let p = tmp("p");
    let c = tmp("c");
    let mut rs = ReplSet::open(&p, "notes", 1).unwrap();
    rs.add(&c).unwrap();
    rs.set_min_ack(2);
    rs.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let r = crate::Store::attach(&c, "notes").unwrap();
    assert!(r.graph().vertex_by_name("Ada").is_some());
    assert!(r.is_replica());
    let _ = fs::remove_dir_all(&p);
    let _ = fs::remove_dir_all(&c);
}

#[test]
fn min_ack_without_a_copy_fails() {
    let p = tmp("p2");
    let mut rs = ReplSet::open(&p, "notes", 1).unwrap();
    rs.set_min_ack(2);
    let e = rs.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        Ok(())
    });
    assert!(e.is_err());
    let _ = fs::remove_dir_all(&p);
}

#[test]
fn two_copies_lag_is_zero_after_apply() {
    let p = tmp("p3");
    let a = tmp("a3");
    let b = tmp("b3");
    let mut rs = ReplSet::open(&p, "notes", 1).unwrap();
    rs.add(&a).unwrap();
    rs.add(&b).unwrap();
    rs.set_min_ack(3);
    rs.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let lags = rs.lag().unwrap();
    assert_eq!(lags.len(), 2);
    for &(_, n) in lags.iter() {
        assert_eq!(n, 0);
    }
    let _ = fs::remove_dir_all(&p);
    let _ = fs::remove_dir_all(&a);
    let _ = fs::remove_dir_all(&b);
}

#[test]
fn failover_promotes_a_copy() {
    let p = tmp("pf");
    let c = tmp("cf");
    let mut rs = ReplSet::open(&p, "notes", 1).unwrap();
    rs.add(&c).unwrap();
    rs.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Ada"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let next = rs.failover().unwrap();
    assert_eq!(next, c);
    rs.apply(|s| {
        s.graph_mut().unwrap().add_vertex(attrs("Bob"), Some("Doc")).unwrap();
        Ok(())
    }).unwrap();
    let s = rs.store().unwrap();
    assert!(s.graph().vertex_by_name("Bob").is_some());
    assert!(s.graph().vertex_by_name("Ada").is_some());
    let _ = fs::remove_dir_all(&p);
    let _ = fs::remove_dir_all(&c);
}
