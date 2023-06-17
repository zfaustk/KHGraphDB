//! Invariants of a generated social graph.
//! Every post has a creator. KNOWS is not a loop.
//! Forums contain posts. The kernel must keep this
//! after a compact.

use super::gen::Social;

pub fn posts_have_creators(s: &Social) -> bool {
    let mut i = 0;
    while i < s.posts.len() {
        let p = s.posts[i];
        let mut ok = false;
        if let Some(v) = s.g.vertex(p) {
            for e in v.outgoing().iter() {
                if s.g.edge_type_name(*e).as_ref().map(|t| t.as_str()) == Some("HAS_CREATOR") {
                    ok = true;
                }
            }
        }
        if !ok {
            return false;
        }
        i += 1;
    }
    true
}

pub fn knows_not_loop(s: &Social) -> bool {
    for &(id, src, dst, _) in s.g.all_edges().iter() {
        if s.g.edge_type_name(id).as_ref().map(|t| t.as_str()) == Some("KNOWS") {
            if src == dst {
                return false;
            }
        }
    }
    true
}

pub fn forums_have_members(s: &Social) -> bool {
    let mut i = 0;
    while i < s.forums.len() {
        let f = s.forums[i];
        let mut n = 0usize;
        if let Some(v) = s.g.vertex(f) {
            for e in v.outgoing().iter() {
                if s.g.edge_type_name(*e).as_ref().map(|t| t.as_str()) == Some("HAS_MEMBER") {
                    n += 1;
                }
            }
        }
        if n == 0 {
            return false;
        }
        i += 1;
    }
    true
}

pub fn all(s: &Social) -> bool {
    posts_have_creators(s) && knows_not_loop(s) && forums_have_members(s)
}
