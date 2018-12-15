//! The index posts Khid, not a letter string as identity.

use crate::index::SchemaIndex;
use crate::khid::Khid;
use crate::prop::Prop;

#[test]
fn posting_is_khid() {
    let mut idx = SchemaIndex::new("Person", "name", false);
    idx.add("k1", &Prop::from_str("Ada"));
    idx.add("k1", &Prop::from_str("Ada"));
    let hits = idx.get_khid(&Prop::from_str("Ada"));
    assert_eq!(hits, vec![Khid::from_raw(1)]);
    idx.remove("k1", &Prop::from_str("Ada"));
    assert!(idx.get_khid(&Prop::from_str("Ada")).is_empty());
}

#[test]
fn unique_compares_khid() {
    let mut idx = SchemaIndex::new("Person", "name", true);
    idx.add("k1", &Prop::from_str("Ada"));
    assert!(idx.contains_other(&Prop::from_str("Ada"), "k2"));
    assert!(!idx.contains_other(&Prop::from_str("Ada"), "k1"));
    assert!(!idx.contains_other(&Prop::from_str("Bob"), "k1"));
}

#[test]
fn empty_str_is_not_posted() {
    let mut idx = SchemaIndex::new("Person", "city", false);
    idx.add("k1", &Prop::from_str(""));
    assert!(idx.get_khid(&Prop::from_str("")).is_empty());
    idx.add("k1", &Prop::from_int(0));
    assert_eq!(idx.get_khid(&Prop::from_int(0)).len(), 1);
}

#[test]
fn get_print_form() {
    let mut idx = SchemaIndex::new("Person", "name", false);
    idx.add_khid(Khid::from_raw(26), &Prop::from_str("Ada"));
    assert_eq!(idx.get(&Prop::from_str("Ada")), vec!["k1a".to_string()]);
}
