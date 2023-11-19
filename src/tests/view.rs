//! Extra cases for MARK VIEW.

use crate::ty::{Type, hash_view};
use crate::Khid;

#[test]
fn empty_recipe_is_refused() {
    let mut t = Type::new(Khid::from_raw(1), "Hit".to_string());
    assert!(!t.mark_view(""));
}

#[test]
fn empty_hash_is_the_offset() {
    assert_eq!(hash_view(""), 0xcbf29ce484222325);
}
