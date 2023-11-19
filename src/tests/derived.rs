//! A view is a recipe on Type. Members come later.

use crate::ty::{Type, hash_view};
use crate::Khid;

#[test]
fn hash_is_stable() {
    assert_eq!(hash_view("MATCH (a) RETURN a"), hash_view("MATCH (a) RETURN a"));
    assert!(hash_view("MATCH (a) RETURN a") != hash_view("MATCH (b) RETURN b"));
}

#[test]
fn recipe_lives_on_the_type() {
    let mut t = Type::new(Khid::from_raw(1), "Hit".to_string());
    assert!(!t.is_view());
    assert!(t.mark_view("MATCH (a:Doc) RETURN a"));
    assert!(t.is_view());
    assert_eq!(t.view(), Some("MATCH (a:Doc) RETURN a"));
    assert_eq!(t.view_hash(), hash_view("MATCH (a:Doc) RETURN a"));
    assert!(t.mark_view("MATCH (a:Doc) RETURN a.title"));
    assert!(t.view_hash() != hash_view("MATCH (a:Doc) RETURN a"));
}
