//! Khid is a u64. The letters are only Display.

use super::super::Khid;

#[test]
fn display_is_k_hex() {
    assert_eq!(format!("{}", Khid::from_raw(0)), "k0");
    assert_eq!(format!("{}", Khid::from_raw(1)), "k1");
    assert_eq!(format!("{}", Khid::from_raw(10)), "ka");
    assert_eq!(format!("{}", Khid::from_raw(0xff)), "kff");
    assert_eq!(format!("{}", Khid::from_raw(26)), "k1a");
}

#[test]
fn parse_roundtrip() {
    let k = Khid::from_raw(0x1a);
    assert_eq!(Khid::parse(&format!("{}", k)), Some(k));
    assert_eq!(Khid::parse("k1a"), Some(k));
    assert_eq!(Khid::parse("K1A"), Some(k));
    assert_eq!(Khid::parse("k0"), Some(Khid::nil()));
}

#[test]
fn parse_rejects() {
    assert!(Khid::parse("").is_none());
    assert!(Khid::parse("k").is_none());
    assert!(Khid::parse("1a").is_none());
    assert!(Khid::parse("kg").is_none());
    assert!(Khid::parse("g1").is_none());
    assert!(Khid::parse("k 1").is_none());
}

#[test]
fn nil_is_zero() {
    assert!(Khid::nil().is_nil());
    assert_eq!(Khid::nil().raw(), 0);
    assert!(!Khid::from_raw(1).is_nil());
}

#[test]
fn copy_and_order() {
    let a = Khid::from_raw(1);
    let b = a;
    assert_eq!(a, b);
    assert!(Khid::from_raw(1) < Khid::from_raw(2));
    assert!(Khid::nil() < Khid::from_raw(1));
}

#[test]
fn from_str_err() {
    let e = "nope".parse::<Khid>().unwrap_err();
    assert_eq!(e.message(), "bad khid");
    assert_eq!("k2".parse::<Khid>().unwrap().raw(), 2);
}

#[test]
fn hash_by_raw() {
    use std::collections::HashSet;
    let mut s = HashSet::new();
    s.insert(Khid::from_raw(1));
    s.insert(Khid::from_raw(1));
    s.insert(Khid::from_raw(2));
    assert_eq!(s.len(), 2);
}
