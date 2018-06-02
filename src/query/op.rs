//! Pull operators. An enum, not a trait object.
//! rustc 1.18 has no dyn to miss. MATCH will compile
//! to a tree of these; the recursive walk leaves.

/// One operator. Volcano-style: the engine matches on
/// Op and pulls a row. No trait objects.
#[derive(Clone)]
pub enum Op {
    Seed { var: String },
    Expand {
        from: String,
        to: String,
        rel: String,
        dir: i32,
        inner: Box<Op>,
    },
    Filter,
    Project,
    Limit { n: usize },
}

impl Op {
    pub fn kind(&self) -> &'static str {
        match *self {
            Op::Seed { .. } => "Seed",
            Op::Expand { .. } => "Expand",
            Op::Filter => "Filter",
            Op::Project => "Project",
            Op::Limit { .. } => "Limit",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Op;

    #[test]
    fn kinds() {
        assert_eq!(Op::Seed { var: "a".to_string() }.kind(), "Seed");
        let inner = Box::new(Op::Seed { var: "a".to_string() });
        let e = Op::Expand {
            from: "a".to_string(),
            to: "b".to_string(),
            rel: "KNOWS".to_string(),
            dir: 1,
            inner: inner,
        };
        assert_eq!(e.kind(), "Expand");
        assert_eq!(Op::Filter.kind(), "Filter");
        assert_eq!(Op::Project.kind(), "Project");
        assert_eq!(Op::Limit { n: 1 }.kind(), "Limit");
    }

    #[test]
    fn expand_owns_the_seed() {
        let op = Op::Expand {
            from: "a".to_string(),
            to: "b".to_string(),
            rel: "KNOWS".to_string(),
            dir: -1,
            inner: Box::new(Op::Seed { var: "a".to_string() }),
        };
        match op {
            Op::Expand { ref inner, dir, .. } => {
                assert_eq!(inner.kind(), "Seed");
                assert_eq!(dir, -1);
            }
            _ => panic!("expected Expand"),
        }
    }
}
