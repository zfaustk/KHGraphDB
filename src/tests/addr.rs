//! An address is a shard and a Khid.

use crate::{Addr, Khid};

#[test]
fn here_prints_as_khid() {
    let a = Addr::here(Khid::from_raw(0x1a));
    assert_eq!(format!("{}", a), "k1a");
    assert!(a.is_here());
    assert_eq!(a.khid(), Khid::from_raw(0x1a));
}

#[test]
fn far_prints_shard() {
    let a = Addr::new(2, Khid::from_raw(0x1a));
    assert_eq!(format!("{}", a), "s2/k1a");
    assert!(!a.is_here());
    assert!(a.on(2));
    assert!(!a.on(1));
}

#[test]
fn here_is_on_any_shard() {
    let a = Addr::here(Khid::from_raw(1));
    assert!(a.on(1));
    assert!(a.on(9));
}

#[test]
fn parse_roundtrip() {
    let here = Addr::here(Khid::from_raw(0x1a));
    assert_eq!(Addr::parse("k1a"), Some(here));
    assert_eq!(Addr::parse("K1A"), Some(here));
    let far = Addr::new(2, Khid::from_raw(0x1a));
    assert_eq!(Addr::parse("s2/k1a"), Some(far));
    assert_eq!(Addr::parse("S2/K1A"), Some(far));
    assert_eq!(Addr::parse(&format!("{}", far)), Some(far));
}

#[test]
fn parse_rejects() {
    assert!(Addr::parse("").is_none());
    assert!(Addr::parse("s2").is_none());
    assert!(Addr::parse("s/k1").is_none());
    assert!(Addr::parse("s2/").is_none());
    assert!(Addr::parse("s2/1a").is_none());
    assert!(Addr::parse("x2/k1").is_none());
}

#[test]
fn from_str_err() {
    let e = "nope".parse::<Addr>().unwrap_err();
    assert_eq!(e.message(), "bad addr");
    assert_eq!("s3/k2".parse::<Addr>().unwrap().shard(), 3);
}

#[test]
fn order_shard_then_khid() {
    let a = Addr::new(1, Khid::from_raw(9));
    let b = Addr::new(2, Khid::from_raw(1));
    assert!(a < b);
    assert!(Addr::here(Khid::from_raw(1)) < a);
}
