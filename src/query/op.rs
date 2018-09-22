//! Pull operators. An enum, not a trait object.
//! rustc 1.18 has no dyn to miss. MATCH compiles
//! to a tree of these; hop DFS for `*` stays a
//! stack of depth at most 16.

use super::super::graph::Graph;
use super::super::prop::Prop;
use super::scan;
use super::{Expr, Pattern, QueryResult};

/// One operator. Volcano-style: the engine matches on
/// Op and pulls rows. No trait objects.
#[derive(Clone)]
pub enum Op {
    Seed { var: String, node: usize },
    Expand {
        from: String,
        to: String,
        rel: String,
        dir: i32,
        rel_i: usize,
        inner: Box<Op>,
    },
    Filter { pred: Expr, inner: Box<Op> },
    Project { inner: Box<Op> },
    Limit { n: usize, inner: Box<Op> },
    Optional { inner: Box<Op> },
    Shortest,
}

impl Op {
    pub fn kind(&self) -> &'static str {
        match *self {
            Op::Seed { .. } => "Seed",
            Op::Expand { .. } => "Expand",
            Op::Filter { .. } => "Filter",
            Op::Project { .. } => "Project",
            Op::Limit { .. } => "Limit",
            Op::Optional { .. } => "Optional",
            Op::Shortest => "Shortest",
        }
    }

    /// Flatten the tree for EXPLAIN. Inner operator first.
    pub fn kinds(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        self.push_kinds(&mut v);
        v
    }

    /// One line per operator. Seed and Expand name their slots.
    pub fn describe(&self) -> String {
        match *self {
            Op::Seed { ref var, .. } => format!("Seed {}", var),
            Op::Expand { ref from, ref to, ref rel, dir, .. } => {
                let arrow = if dir < 0 {
                    "<-"
                } else if dir == 0 {
                    "-"
                } else {
                    "->"
                };
                format!("Expand {} -[{}]{} {}", from, rel, arrow, to)
            }
            Op::Filter { .. } => "Filter".to_string(),
            Op::Project { .. } => "Project".to_string(),
            Op::Limit { n, .. } => format!("Limit {}", n),
            Op::Optional { ref inner } => format!("Optional ({})", inner.kind()),
            Op::Shortest => "Shortest".to_string(),
        }
    }

    fn push_kinds(&self, v: &mut Vec<&'static str>) {
        match *self {
            Op::Expand { ref inner, .. } |
            Op::Optional { ref inner } |
            Op::Filter { ref inner, .. } |
            Op::Project { ref inner } |
            Op::Limit { ref inner, .. } => {
                inner.push_kinds(v);
            }
            _ => {}
        }
        v.push(self.kind());
    }
}

/// Compile a resolved pattern. Flip is the caller's job.
pub fn compile(pat: &Pattern) -> Op {
    if pat.shortest {
        return Op::Shortest;
    }
    let var0 = pat.nodes[0].var.clone().unwrap_or("n0".to_string());
    let mut op = Op::Seed {
        var: var0,
        node: 0,
    };
    let mut i = 0;
    while i < pat.rels.len() {
        let from = match op {
            Op::Seed { ref var, .. } => var.clone(),
            Op::Expand { ref to, .. } => to.clone(),
            _ => format!("n{}", i),
        };
        let to = pat.nodes[i + 1].var.clone().unwrap_or(format!("n{}", i + 1));
        let reln = pat.rels[i].type_name.clone().unwrap_or(String::new());
        op = Op::Expand {
            from: from,
            to: to,
            rel: reln,
            dir: pat.rels[i].dir,
            rel_i: i,
            inner: Box::new(op),
        };
        i += 1;
    }
    if pat.optional {
        op = Op::Optional { inner: Box::new(op) };
    }
    if let Some(ref pred) = pat.pred {
        op = Op::Filter {
            pred: pred.clone(),
            inner: Box::new(op),
        };
    }
    if pat.project {
        op = Op::Project { inner: Box::new(op) };
    }
    if let Some(n) = pat.limit {
        op = Op::Limit {
            n: n,
            inner: Box::new(op),
        };
    }
    op
}

#[derive(Clone)]
struct Row {
    bind: Vec<Option<String>>,
    trail: Vec<String>,
    rel_edges: Vec<Vec<String>>,
    seen_v: Vec<String>,
    seen_e: Vec<String>,
}

/// Run MATCH. Same rows as the old recursive walk.
pub fn run(g: &Graph,
           pat: &Pattern,
           seed: &std::collections::HashMap<String, String>)
           -> QueryResult {
    let mut pat = pat.clone();
    if let Some(msg) = scan::resolve_types(g, &mut pat, true) {
        return QueryResult::fail(&msg);
    }
    scan::name_slots(&mut pat);
    let orig_cols = scan::columns_of(&pat);
    let flipped = scan::should_flip(&pat, seed);
    let walk_pat = if flipped {
        scan::flip_one_hop(&pat)
    } else {
        pat.clone()
    };
    let op = compile(&walk_pat);
    let rows = exec_op(g, &walk_pat, &op, seed);
    let mut r = QueryResult::ok_msg("MATCH");
    r.columns = scan::columns_of(&walk_pat);
    for row in rows.iter() {
        scan::emit_row(&walk_pat,
                       &row.bind,
                       &row.trail,
                       &row.rel_edges,
                       &mut r);
    }
    if flipped {
        r = scan::unflip_result(r, &orig_cols);
    }
    if walk_pat.optional && r.rows.is_empty() {
        // compile already wrapped Optional; exec_op should have
        // produced the empty row. Keep the old belt.
        if r.columns.is_empty() {
            r.columns.push("n".to_string());
        }
        let mut row = Vec::new();
        let mut i = 0;
        while i < r.columns.len() {
            row.push(None);
            i += 1;
        }
        r.rows.push(row);
        r.message = "optional empty".to_string();
        return r;
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn exec_op(g: &Graph,
           pat: &Pattern,
           op: &Op,
           seed: &std::collections::HashMap<String, String>)
           -> Vec<Row> {
    match *op {
        Op::Seed { node, .. } => exec_seed(g, pat, node, seed),
        Op::Expand { rel_i, ref inner, .. } => {
            let inner_rows = exec_op(g, pat, inner, seed);
            let mut out = Vec::new();
            for row in inner_rows.iter() {
                expand_from(g, pat, rel_i, row, seed, &mut out);
            }
            out
        }
        Op::Optional { ref inner } => {
            let rows = exec_op(g, pat, inner, seed);
            if rows.is_empty() {
                vec![empty_row(pat)]
            } else {
                rows
            }
        }
        Op::Shortest => exec_shortest_rows(g, pat, seed),
        Op::Filter { ref pred, ref inner } => {
            let rows = exec_op(g, pat, inner, seed);
            let mut out = Vec::new();
            for row in rows.iter() {
                if row_pred(g, pat, row, pred) {
                    out.push(row.clone());
                }
            }
            out
        }
        Op::Project { ref inner } => exec_op(g, pat, inner, seed),
        Op::Limit { n, ref inner } => {
            let mut rows = exec_op(g, pat, inner, seed);
            if rows.len() > n {
                rows.truncate(n);
            }
            rows
        }
    }
}

fn empty_row(pat: &Pattern) -> Row {
    Row {
        bind: vec![None; pat.nodes.len()],
        trail: Vec::new(),
        rel_edges: vec![Vec::new(); pat.rels.len()],
        seen_v: Vec::new(),
        seen_e: Vec::new(),
    }
}

fn exec_seed(g: &Graph,
             pat: &Pattern,
             node: usize,
             seed: &std::collections::HashMap<String, String>)
             -> Vec<Row> {
    let n = &pat.nodes[node];
    let found = if node == 0 {
        scan::start_seeds(g, pat)
    } else {
        scan::seeds(g, n)
    };
    let mut rows = Vec::new();
    for id in found.iter() {
        if !scan::seed_ok(seed, n, id) {
            continue;
        }
        let mut bind = vec![None; pat.nodes.len()];
        bind[node] = Some(id.clone());
        let mut rel_edges = Vec::new();
        let mut k = 0;
        while k < pat.rels.len() {
            rel_edges.push(Vec::new());
            k += 1;
        }
        rows.push(Row {
            bind: bind,
            trail: vec![id.clone()],
            rel_edges: rel_edges,
            seen_v: vec![id.clone()],
            seen_e: Vec::new(),
        });
    }
    rows
}

fn expand_from(g: &Graph,
               pat: &Pattern,
               rel_i: usize,
               row: &Row,
               seed: &std::collections::HashMap<String, String>,
               out: &mut Vec<Row>) {
    let from = match row.bind[rel_i] {
        Some(ref id) => id.clone(),
        None => return,
    };
    expand_rel(g, pat, rel_i, &from, 0, row, seed, out);
}

fn expand_rel(g: &Graph,
              pat: &Pattern,
              rel_i: usize,
              u: &str,
              hops: usize,
              row: &Row,
              seed: &std::collections::HashMap<String, String>,
              out: &mut Vec<Row>) {
    let rel = &pat.rels[rel_i];
    let next = &pat.nodes[rel_i + 1];
    if hops >= rel.min && hops <= rel.max && scan::node_ok(g, u, next) &&
       scan::seed_ok(seed, next, u) {
        let mut emit = row.clone();
        emit.bind[rel_i + 1] = Some(u.to_string());
        out.push(emit);
    }
    if hops >= rel.max {
        return;
    }
    let eids = scan::edges_of(g, u, rel);
    for eid in eids.iter() {
        if scan::contains_id(&row.seen_e, eid) {
            continue;
        }
        let e = match g.edge(eid) {
            Some(e) => e,
            None => continue,
        };
        let v = if rel.dir < 0 {
            format!("{}", e.source())
        } else if rel.dir == 0 {
            if format!("{}", e.source()) == u {
                format!("{}", e.target())
            } else {
                format!("{}", e.source())
            }
        } else {
            format!("{}", e.target())
        };
        if scan::contains_id(&row.seen_v, &v) {
            continue;
        }
        let mut nxt = row.clone();
        nxt.seen_e.push(eid.clone());
        nxt.seen_v.push(v.clone());
        nxt.trail.push(eid.clone());
        nxt.trail.push(v.clone());
        nxt.rel_edges[rel_i].push(eid.clone());
        expand_rel(g, pat, rel_i, &v, hops + 1, &nxt, seed, out);
    }
}

fn exec_shortest_rows(g: &Graph,
                      pat: &Pattern,
                      seed: &std::collections::HashMap<String, String>)
                      -> Vec<Row> {
    if pat.rels.len() != 1 {
        return Vec::new();
    }
    let starts = scan::start_seeds(g, pat);
    let ends = scan::seeds(g, &pat.nodes[1]);
    let rel = &pat.rels[0];
    let tid = match rel.type_id {
        Some(ref s) => Some(&s[..]),
        None => None,
    };
    let mut rows = Vec::new();
    for s in starts.iter() {
        if !scan::seed_ok(seed, &pat.nodes[0], s) {
            continue;
        }
        for t in ends.iter() {
            if !scan::seed_ok(seed, &pat.nodes[1], t) {
                continue;
            }
            match super::super::algo::path_on(g, s, t, tid, rel.dir, rel.min, rel.max) {
                Some(path) => {
                    let mut bind = vec![None; pat.nodes.len()];
                    bind[0] = Some(s.clone());
                    bind[1] = Some(t.clone());
                    let mut rel_edges = vec![Vec::new(); pat.rels.len()];
                    let mut es = Vec::new();
                    for k in super::Path::parse_all(&path).edges().iter() {
                        es.push(format!("{}", k));
                    }
                    rel_edges[0] = es;
                    rows.push(Row {
                        bind: bind,
                        trail: path,
                        rel_edges: rel_edges,
                        seen_v: Vec::new(),
                        seen_e: Vec::new(),
                    });
                }
                None => {}
            }
        }
    }
    rows
}

fn row_pred(g: &Graph, pat: &Pattern, row: &Row, pred: &Expr) -> bool {
    match *pred {
        Expr::Eq(ref var, ref key, ref val) => {
            match lookup_row(g, pat, row, var, key) {
                Some(ref got) if got == val => true,
                _ => false,
            }
        }
        Expr::Cmp(ref var, ref key, op, ref val) => {
            match lookup_row(g, pat, row, var, key) {
                Some(ref got) => cmp_prop(got, val, op),
                None => false,
            }
        }
        Expr::In(ref var, ref key, ref vals) => {
            match lookup_row(g, pat, row, var, key) {
                Some(ref got) => {
                    let mut hit = false;
                    for v in vals.iter() {
                        if v == got {
                            hit = true;
                            break;
                        }
                    }
                    hit
                }
                None => false,
            }
        }
        Expr::And(ref a, ref b) => row_pred(g, pat, row, a) && row_pred(g, pat, row, b),
        Expr::Or(ref a, ref b) => row_pred(g, pat, row, a) || row_pred(g, pat, row, b),
        Expr::Not(ref a) => !row_pred(g, pat, row, a),
    }
}

fn lookup_row(g: &Graph, pat: &Pattern, row: &Row, var: &str, key: &str) -> Option<Prop> {
    let mut i = 0;
    while i < pat.nodes.len() {
        let hit = match pat.nodes[i].var {
            Some(ref v) if v == var => true,
            _ => false,
        };
        if hit {
            if let Some(ref id) = row.bind[i] {
                if let Some(p) = g.vertex(id).and_then(|v| v.get_prop(key)).cloned() {
                    return Some(p);
                }
                return g.edge(id).and_then(|e| e.get_prop(key)).cloned();
            }
            return None;
        }
        i += 1;
    }
    let mut r = 0;
    while r < pat.rels.len() {
        let hit = match pat.rels[r].var {
            Some(ref v) if v == var => true,
            _ => false,
        };
        if hit {
            if r < row.rel_edges.len() && row.rel_edges[r].len() == 1 {
                let id = &row.rel_edges[r][0];
                if let Some(p) = g.edge(id).and_then(|e| e.get_prop(key)).cloned() {
                    return Some(p);
                }
                return g.vertex(id).and_then(|v| v.get_prop(key)).cloned();
            }
            return None;
        }
        r += 1;
    }
    None
}

fn cmp_prop(got: &Prop, val: &Prop, op: i32) -> bool {
    if got.tag() != val.tag() {
        return op == 3;
    }
    match op {
        -2 => got <= val,
        -1 => got < val,
        1 => got > val,
        2 => got >= val,
        3 => got != val,
        _ => got == val,
    }
}

#[cfg(test)]
mod tests {
    use super::Op;
    use super::Expr;
    use super::super::super::prop::Prop;

    fn dummy_filter() -> Op {
        Op::Filter {
            pred: Expr::Eq("a".to_string(), "name".to_string(), Prop::from_str("x")),
            inner: Box::new(Op::Seed {
                var: "a".to_string(),
                node: 0,
            }),
        }
    }

    #[test]
    fn kinds() {
        assert_eq!(Op::Seed {
                       var: "a".to_string(),
                       node: 0,
                   }
                   .kind(),
                   "Seed");
        let inner = Box::new(Op::Seed {
            var: "a".to_string(),
            node: 0,
        });
        let e = Op::Expand {
            from: "a".to_string(),
            to: "b".to_string(),
            rel: "KNOWS".to_string(),
            dir: 1,
            rel_i: 0,
            inner: inner,
        };
        assert_eq!(e.kind(), "Expand");
        assert_eq!(dummy_filter().kind(), "Filter");
        assert_eq!(Op::Project {
                       inner: Box::new(Op::Seed {
                           var: "a".to_string(),
                           node: 0,
                       }),
                   }
                   .kind(),
                   "Project");
        assert_eq!(Op::Limit {
                       n: 1,
                       inner: Box::new(Op::Seed {
                           var: "a".to_string(),
                           node: 0,
                       }),
                   }
                   .kind(),
                   "Limit");
        assert_eq!(Op::Shortest.kind(), "Shortest");
    }

    #[test]
    fn expand_owns_the_seed() {
        let op = Op::Expand {
            from: "a".to_string(),
            to: "b".to_string(),
            rel: "KNOWS".to_string(),
            dir: -1,
            rel_i: 0,
            inner: Box::new(Op::Seed {
                var: "a".to_string(),
                node: 0,
            }),
        };
        match op {
            Op::Expand { ref inner, dir, .. } => {
                assert_eq!(inner.kind(), "Seed");
                assert_eq!(dir, -1);
            }
            _ => panic!("expected Expand"),
        }
    }

    #[test]
    fn kinds_flatten_inner_first() {
        let op = Op::Optional {
            inner: Box::new(Op::Expand {
                from: "a".to_string(),
                to: "b".to_string(),
                rel: "KNOWS".to_string(),
                dir: 1,
                rel_i: 0,
                inner: Box::new(Op::Seed {
                    var: "a".to_string(),
                    node: 0,
                }),
            }),
        };
        assert_eq!(op.kinds(), vec!["Seed", "Expand", "Optional"]);
    }
}
