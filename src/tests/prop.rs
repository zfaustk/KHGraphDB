//! Prop is a value. 1 is not "1".

use std::collections::HashSet;
use super::super::prop::Prop;

#[test]
fn int_is_not_str() {
    assert!(Prop::from_int(1) != Prop::from_str("1"));
    assert_eq!(Prop::from_int(1).as_display(), "1");
    assert_eq!(Prop::from_str("1").as_display(), "1");
}

#[test]
fn same_kind_eq() {
    assert_eq!(Prop::from_int(7), Prop::from_int(7));
    assert_eq!(Prop::from_bool(true), Prop::from_bool(true));
    assert_eq!(Prop::from_str("Ada"), Prop::from_str("Ada"));
    assert!(Prop::from_int(7) != Prop::from_int(8));
    assert!(Prop::from_bool(true) != Prop::from_bool(false));
}

#[test]
fn float_bits() {
    let a = Prop::from_float(1.0);
    let b = Prop::from_float(1.0);
    assert_eq!(a, b);
    assert!(Prop::from_float(1.0) != Prop::from_float(1.1));
    assert!(Prop::from_float(1.0) != Prop::from_int(1));
}

#[test]
fn hash_respects_kind() {
    let mut s = HashSet::new();
    s.insert(Prop::from_int(1));
    s.insert(Prop::from_str("1"));
    assert_eq!(s.len(), 2);
    assert!(s.contains(&Prop::from_int(1)));
    assert!(!s.contains(&Prop::from_bool(true)));
}

#[test]
fn order_by_tag_then_value() {
    assert!(Prop::from_bool(true) < Prop::from_int(0));
    assert!(Prop::from_int(2) < Prop::from_int(9));
    assert!(Prop::from_str("Ada") < Prop::from_str("Bob"));
}

#[test]
fn as_int_does_not_parse_str() {
    assert_eq!(Prop::from_int(3).as_int(), Some(3));
    assert!(Prop::from_str("3").as_int().is_none());
    assert!(Prop::from_bool(true).as_int().is_none());
}

#[test]
fn as_str_only_str() {
    assert_eq!(Prop::from_str("Ada").as_str(), Some("Ada"));
    assert!(Prop::from_int(1).as_str().is_none());
}

#[test]
fn float_from_int_for_math() {
    assert_eq!(Prop::from_int(2).as_float(), Some(2.0));
    assert!(Prop::from_str("2").as_float().is_none());
}
