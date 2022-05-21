//! How a MATCH starts. Seed from a Type, or from a
//! (Type, key) index. The operators borrow this.
//! Every id here is a Khid. Display is the shell's job.

use std::collections::HashMap;

use crate::graph::Graph;
use crate::khid::Khid;
use crate::prop::Prop;
use super::{Expr, NodePat, Path, Pattern, QueryResult, RelPat, Val};

/// Fill missing node names so a flip cannot rename n0 to n1.
pub fn name_slots(pat: &mut Pattern) {
    let mut i = 0;
    while i < pat.nodes.len() {
        if pat.nodes[i].var.is_none() {
            pat.nodes[i].var = Some(format!("n{}", i));
        }
        i += 1;
    }
}

pub fn columns_of(pat: &Pattern) -> Vec<String> {
    let mut cols = Vec::new();
    if let Some(ref p) = pat.path_var {
        cols.push(p.clone());
    }
    for (i, n) in pat.nodes.iter().enumerate() {
        cols.push(n.var.clone().unwrap_or(format!("n{}", i)));
        if i < pat.rels.len() {
            if let Some(ref v) = pat.rels[i].var {
                cols.push(v.clone());
            }
        }
    }
    cols
}

pub fn emit_row(pat: &Pattern,
                bind: &[Option<Khid>],
                trail: &[Khid],
                rel_edges: &[Vec<Khid>],
                r: &mut QueryResult) {
    let mut row = Vec::new();
    if pat.path_var.is_some() {
        row.push(Some(Val::Path(Path::new(trail.to_vec()))));
    }
    for (i, b) in bind.iter().enumerate() {
        match *b {
            Some(id) => row.push(Some(Val::Id(id))),
            None => row.push(None),
        }
        if i < pat.rels.len() {
            if pat.rels[i].var.is_some() {
                let edges = if i < rel_edges.len() {
                    rel_edges[i].clone()
                } else {
                    Vec::new()
                };
                if pat.rels[i].star {
                    let mut ids = Vec::new();
                    for e in edges.iter() {
                        ids.push(Val::Id(*e));
                    }
                    row.push(Some(Val::List(ids)));
                } else if edges.len() == 1 {
                    row.push(Some(Val::Id(edges[0])));
                } else {
                    row.push(None);
                }
            }
        }
    }
    r.rows.push(row);
}

/// Bind the Type object. The pattern holds its KHID,
/// not the name string, after this.
pub fn resolve_types(g: &Graph, pat: &mut Pattern, required: bool) -> Option<String> {
    for n in pat.nodes.iter_mut() {
        if let Some(ref tn) = n.type_name {
            match g.type_by_name(tn) {
                Some(t) => n.type_id = Some(t.khid()),
                None => {
                    if required {
                        return Some(format!("unknown type {}", tn));
                    }
                }
            }
        }
    }
    for r in pat.rels.iter_mut() {
        if let Some(ref tn) = r.type_name {
            match g.type_by_name(tn) {
                Some(t) => r.type_id = Some(t.khid()),
                None => {
                    if required {
                        return Some(format!("unknown type {}", tn));
                    }
                }
            }
        }
    }
    None
}

pub fn seed_ok(seed: &HashMap<String, Khid>, n: &NodePat, id: Khid) -> bool {
    match n.var {
        Some(ref v) => {
            match seed.get(v) {
                Some(&s) => s == id,
                None => true,
            }
        }
        None => true,
    }
}

pub fn contains_id(ids: &[Khid], id: Khid) -> bool {
    for x in ids.iter() {
        if *x == id {
            return true;
        }
    }
    false
}

/// Seed ids for a node pattern. Indexed (Type, key) wins.
/// A comparison on an ordered posting is a range, not a scan.
pub fn seeds(g: &Graph, n: &NodePat, pred: Option<&Expr>) -> Vec<Khid> {
    if let Some(ref tn) = n.type_name {
        if !n.props.is_empty() {
            let mut picked: Option<(String, Prop)> = None;
            let mut unique = false;
            for &(ref k, ref val) in n.props.iter() {
                if g.has_unique(tn, k) {
                    picked = Some((k.clone(), val.clone()));
                    unique = true;
                    break;
                }
                if g.has_index(tn, k) && picked.is_none() {
                    picked = Some((k.clone(), val.clone()));
                }
            }
            let _ = unique;
            let (k, val) = match picked {
                Some(p) => p,
                None => (n.props[0].0.clone(), n.props[0].1.clone()),
            };
            let found = g.find_prop(tn, &k, &val);
            return found.into_iter().filter(|id| node_ok(g, *id, n)).collect();
        }
        if let Some((k, lo, hi, li, ri)) = bounds_for(n, pred) {
            if g.has_index(tn, &k) {
                let found = g.find_range(tn, &k, lo.as_ref(), hi.as_ref(), li, ri);
                return found.into_iter().filter(|id| node_ok(g, *id, n)).collect();
            }
        }
    }
    let src: Vec<Khid> = match n.type_id {
        Some(tid) => {
            match g.ty(tid) {
                Some(t) => t.vertices().iter().cloned().collect(),
                None => Vec::new(),
            }
        }
        None => g.vertex_ids(),
    };
    src.into_iter().filter(|id| node_ok(g, *id, n)).collect()
}

/// Selinger without the catalog: unique is 1, a posting
/// is its length, a type is its members. No bushy trees.
pub fn card(g: &Graph, n: &NodePat, pred: Option<&Expr>) -> usize {
    if let Some(ref tn) = n.type_name {
        for &(ref k, ref val) in n.props.iter() {
            if g.has_unique(tn, k) {
                return 1;
            }
            if g.has_index(tn, k) {
                let n = g.find_prop(tn, k, val).len();
                if n > 0 {
                    return n;
                }
            }
        }
        if let Some((k, lo, hi, li, ri)) = bounds_for(n, pred) {
            if g.has_index(tn, &k) {
                let n = g.find_range(tn, &k, lo.as_ref(), hi.as_ref(), li, ri).len();
                if n > 0 {
                    return n;
                }
            }
        }
        let n = g.vertices_of_type(tn).len();
        if n > 0 {
            return n;
        }
    }
    let n = g.vertex_count();
    if n == 0 { 1 } else { n }
}

fn bounds_for(n: &NodePat, pred: Option<&Expr>) -> Option<(String, Option<Prop>, Option<Prop>, bool, bool)> {
    let pred = match pred {
        Some(p) => p,
        None => return None,
    };
    let var = match n.var {
        Some(ref v) => v.as_str(),
        None => return None,
    };
    walk_bounds(var, pred)
}

fn walk_bounds(var: &str, e: &Expr) -> Option<(String, Option<Prop>, Option<Prop>, bool, bool)> {
    match *e {
        Expr::Cmp(ref v, ref k, op, ref val) if v == var => {
            match op {
                -1 => Some((k.clone(), None, Some(val.clone()), true, false)),
                -2 => Some((k.clone(), None, Some(val.clone()), true, true)),
                1 => Some((k.clone(), Some(val.clone()), None, false, true)),
                2 => Some((k.clone(), Some(val.clone()), None, true, true)),
                _ => None,
            }
        }
        Expr::Eq(ref v, ref k, ref val) if v == var => {
            Some((k.clone(), Some(val.clone()), Some(val.clone()), true, true))
        }
        Expr::And(ref a, ref b) => {
            match (walk_bounds(var, a), walk_bounds(var, b)) {
                (Some(l), Some(r)) => merge_bounds(l, r),
                (Some(x), None) | (None, Some(x)) => Some(x),
                (None, None) => None,
            }
        }
        _ => None,
    }
}

fn merge_bounds(a: (String, Option<Prop>, Option<Prop>, bool, bool),
                b: (String, Option<Prop>, Option<Prop>, bool, bool))
                -> Option<(String, Option<Prop>, Option<Prop>, bool, bool)> {
    if a.0 != b.0 {
        return Some(a);
    }
    let lo = match (a.1, b.1) {
        (Some(x), Some(y)) => Some(if x > y { x } else { y }),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    };
    let hi = match (a.2, b.2) {
        (Some(x), Some(y)) => Some(if x < y { x } else { y }),
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
    };
    Some((a.0, lo, hi, a.3 && b.3, a.4 && b.4))
}

pub fn keyed(n: &NodePat) -> bool {
    n.type_name.is_some() && !n.props.is_empty()
}

/// Reverse the walk when the cheaper end is nearer
/// the last node. Left-deep. Do not start in the middle.
pub fn should_reverse(g: &Graph, pat: &Pattern, seed: &HashMap<String, Khid>) -> bool {
    if pat.shortest {
        return false;
    }
    if pat.nodes.len() < 2 {
        return false;
    }
    if let Some(ref v) = pat.nodes[0].var {
        if seed.contains_key(v) {
            return false;
        }
    }
    let mut best = 0usize;
    let mut best_c = card(g, &pat.nodes[0], pat.pred.as_ref());
    let mut i = 1;
    while i < pat.nodes.len() {
        let c = card(g, &pat.nodes[i], pat.pred.as_ref());
        if c < best_c {
            best = i;
            best_c = c;
        }
        i += 1;
    }
    best * 2 >= pat.nodes.len()
}

pub fn reverse_walk(pat: &Pattern) -> Pattern {
    let mut p = pat.clone();
    p.nodes.reverse();
    p.rels.reverse();
    let mut i = 0;
    while i < p.rels.len() {
        p.rels[i].dir = -p.rels[i].dir;
        i += 1;
    }
    p
}

/// Seed card, then times degree per hop. Rows, not pages.
pub fn plan_cost(g: &Graph, pat: &Pattern) -> (usize, usize) {
    let seed = card(g, &pat.nodes[0], pat.pred.as_ref());
    let mut tot = seed;
    let mut i = 0;
    while i < pat.rels.len() {
        tot = tot.saturating_mul(rel_degree(g, &pat.rels[i]));
        i += 1;
    }
    (seed, tot)
}

fn rel_degree(g: &Graph, rel: &RelPat) -> usize {
    let e = match rel.type_name {
        Some(ref tn) if !tn.is_empty() => g.edges_of_type(tn).len(),
        _ => g.edge_count(),
    };
    let v = g.vertex_count();
    if v == 0 {
        return 1;
    }
    let d = (e * 2) / v;
    if d == 0 { 1 } else { d }
}

pub fn unflip_result(mut r: QueryResult, orig_cols: &[String]) -> QueryResult {
    let src_cols = r.columns.clone();
    let mut new_rows = Vec::new();
    for row in r.rows.iter() {
        let mut nr = Vec::new();
        for c in orig_cols.iter() {
            match src_cols.iter().position(|x| x == c) {
                Some(j) => {
                    let cell = row.get(j).cloned().unwrap_or(None);
                    let cell = match cell {
                        Some(Val::Path(p)) => {
                            let mut ids = p.ids().to_vec();
                            ids.reverse();
                            Some(Val::Path(Path::new(ids)))
                        }
                        other => other,
                    };
                    nr.push(cell);
                }
                None => nr.push(None),
            }
        }
        new_rows.push(nr);
    }
    r.columns = orig_cols.to_vec();
    r.rows = new_rows;
    r
}

pub fn start_seeds(g: &Graph, pat: &Pattern) -> Vec<Khid> {
    let n0 = &pat.nodes[0];
    if n0.type_id.is_some() || !n0.props.is_empty() || pat.pred.is_some() {
        return seeds(g, n0, pat.pred.as_ref());
    }
    if !pat.rels.is_empty() {
        if let Some(tid) = pat.rels[0].type_id {
            return starts_from_type(g, tid, pat.rels[0].dir, n0);
        }
    }
    seeds(g, n0, pat.pred.as_ref())
}

fn starts_from_type(g: &Graph, tid: Khid, dir: i32, n0: &NodePat) -> Vec<Khid> {
    let eids: Vec<Khid> = match g.ty(tid) {
        Some(t) => t.edges().iter().cloned().collect(),
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for eid in eids.iter() {
        let e = match g.edge(*eid) {
            Some(e) => e,
            None => continue,
        };
        if dir >= 0 {
            let s = e.source();
            if node_ok(g, s, n0) && !contains_id(&out, s) {
                out.push(s);
            }
        }
        if dir <= 0 {
            let s = e.target();
            if node_ok(g, s, n0) && !contains_id(&out, s) {
                out.push(s);
            }
        }
    }
    out
}

fn wears(g: &Graph, vid: Khid, tid: Khid) -> bool {
    match g.vertex(vid) {
        Some(v) => v.types().iter().any(|t| *t == tid),
        None => false,
    }
}

pub fn node_ok(g: &Graph, vid: Khid, n: &NodePat) -> bool {
    if let Some(tid) = n.type_id {
        if !wears(g, vid, tid) {
            return false;
        }
    }
    for &(ref k, ref val) in n.props.iter() {
        match g.vertex(vid).and_then(|v| v.get_prop(k)) {
            Some(got) if got == val => {}
            _ => return false,
        }
    }
    true
}

pub fn edges_of(g: &Graph, vid: Khid, rel: &RelPat) -> Vec<Khid> {
    let v = match g.vertex(vid) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let mut ids = Vec::new();
    let src: Vec<Khid> = if rel.dir > 0 {
        v.outgoing().iter().cloned().collect()
    } else if rel.dir < 0 {
        v.incoming().iter().cloned().collect()
    } else {
        let mut both: Vec<Khid> = v.outgoing().iter().cloned().collect();
        both.extend(v.incoming().iter().cloned());
        both
    };
    for eid in src.iter() {
        if let Some(tid) = rel.type_id {
            match g.edge(*eid).and_then(|e| e.type_id()) {
                Some(et) if et == tid => {}
                _ => continue,
            }
        }
        ids.push(*eid);
    }
    ids
}
