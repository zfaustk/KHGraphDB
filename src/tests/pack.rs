//! Neighbour names in one string. First try.

use crate::{ask_query, Graph};
use super::common::attrs;

#[test]
fn pack_joins_neighbour_names() {
    let mut g = Graph::new();
    let a = g.add_vertex(attrs("Ada"), Some("Doc")).unwrap();
    let b = g.add_vertex(attrs("Alan"), Some("Doc")).unwrap();
    g.add_edge(a, b, Some("KNOWS")).unwrap();
    let r = ask_query(&g, "PACK Ada");
    assert!(r.ok);
    assert_eq!(r.message, "Alan");
    assert_eq!(r.columns, vec!["pack".to_string()]);
}
