//! The index posts Khid, not a letter string as identity.

use crate::index::SchemaIndex;
use crate::khid::Khid;
use crate::prop::Prop;

#[test]
fn posting_is_khid() {
    let mut idx = SchemaIndex::new("Person", "name", false);
    idx.add(Khid::from_raw(1), &Prop::from_str("Ada"));
    idx.add(Khid::from_raw(1), &Prop::from_str("Ada"));
    let hits = idx.get(&Prop::from_str("Ada"));
    assert_eq!(hits, vec![Khid::from_raw(1)]);
    idx.remove(Khid::from_raw(1), &Prop::from_str("Ada"));
    assert!(idx.get(&Prop::from_str("Ada")).is_empty());
}

#[test]
fn unique_compares_khid() {
    let mut idx = SchemaIndex::new("Person", "name", true);
    idx.add(Khid::from_raw(1), &Prop::from_str("Ada"));
    assert!(idx.contains_other(&Prop::from_str("Ada"), Khid::from_raw(2)));
    assert!(!idx.contains_other(&Prop::from_str("Ada"), Khid::from_raw(1)));
    assert!(!idx.contains_other(&Prop::from_str("Bob"), Khid::from_raw(1)));
}

#[test]
fn empty_str_is_not_posted() {
    let mut idx = SchemaIndex::new("Person", "city", false);
    idx.add(Khid::from_raw(1), &Prop::from_str(""));
    assert!(idx.get(&Prop::from_str("")).is_empty());
    idx.add(Khid::from_raw(1), &Prop::from_int(0));
    assert_eq!(idx.get(&Prop::from_int(0)).len(), 1);
}

#[test]
fn get_is_khid() {
    let mut idx = SchemaIndex::new("Person", "name", false);
    idx.add(Khid::from_raw(26), &Prop::from_str("Ada"));
    assert_eq!(idx.get(&Prop::from_str("Ada")), vec![Khid::from_raw(26)]);
}

#[test]
fn range_is_ordered() {
    let mut idx = SchemaIndex::new("Doc", "n", false);
    idx.add(Khid::from_raw(1), &Prop::from_int(10));
    idx.add(Khid::from_raw(2), &Prop::from_int(20));
    idx.add(Khid::from_raw(3), &Prop::from_int(30));
    let mid = idx.range(Some(&Prop::from_int(10)), Some(&Prop::from_int(30)), false, false);
    assert_eq!(mid.len(), 1);
    assert_eq!(mid[0], Khid::from_raw(2));
    let ge = idx.range(Some(&Prop::from_int(20)), None, true, true);
    assert_eq!(ge.len(), 2);
}
