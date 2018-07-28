//! How a MATCH starts. Seed from a Type, or from an
//! index when a keyed property is present. The walk
//! used to own this; the operators borrow it.

use super::super::graph::Graph;
use super::super::khid::Khid;
use super::super::prop::Prop;
use super::{NodePat, Path, Pattern, QueryResult, RelPat, Val};

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
                bind: &[Option<String>],
                trail: &[String],
                rel_edges: &[Vec<String>],
                r: &mut QueryResult) {
    let mut row = Vec::new();
    if pat.path_var.is_some() {
        row.push(Some(Val::Path(Path::parse_all(trail))));
    }
    for (i, b) in bind.iter().enumerate() {
        match *b {
            Some(ref id) => row.push(Some(Val::Id(id.clone()))),
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
                        ids.push(Val::Id(e.clone()));
                    }
                    row.push(Some(Val::List(ids)));
                } else if edges.len() == 1 {
                    row.push(Some(Val::Id(edges[0].clone())));
                } else {
                    row.push(None);
                }
            }
        }
    }
    r.rows.push(row);
}

pub fn resolve_types(g: &Graph, pat: &mut Pattern, required: bool) -> Option<String> {
    for n in pat.nodes.iter_mut() {
        if let Some(ref tn) = n.type_name {
            match g.type_by_name(tn) {
                Some(t) => n.type_id = Some(t.khid().to_string()),
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
                Some(t) => r.type_id = Some(t.khid().to_string()),
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

pub fn seed_ok(seed: &std::collections::HashMap<String, String>, n: &NodePat, id: &str) -> bool {
    match n.var {
        Some(ref v) => {
            match seed.get(v) {
                Some(s) => s == id,
                None => true,
            }
        }
        None => true,
    }
}

pub fn contains_id(ids: &Vec<String>, id: &str) -> bool {
    for x in ids.iter() {
        if x == id {
            return true;
        }
    }
    false
}

/// Seed ids for a node pattern. Indexed (Type, key) wins.
pub fn seeds(g: &Graph, n: &NodePat) -> Vec<String> {
    if let Some(ref tn) = n.type_name {
        if !n.props.is_empty() {
            let mut picked: Option<(String, Prop)> = None;
            for &(ref k, ref val) in n.props.iter() {
                if g.has_index(tn, k) {
                    picked = Some((k.clone(), val.clone()));
                    break;
                }
            }
            let (k, val) = match picked {
                Some(p) => p,
                None => (n.props[0].0.clone(), n.props[0].1.clone()),
            };
            let found = g.find_prop(tn, &k, &val);
            return found.into_iter().filter(|id| node_ok(g, id, n)).collect();
        }
    }
    let src: Vec<String> = match n.type_id {
        Some(ref tid) => {
            match g.ty(tid) {
                Some(t) => Khid::display_all(&t.vertices().iter().cloned().collect::<Vec<_>>()),
                None => Vec::new(),
            }
        }
        None => g.vertex_ids(),
    };
    src.into_iter().filter(|id| node_ok(g, id, n)).collect()
}

pub fn keyed(n: &NodePat) -> bool {
    n.type_name.is_some() && !n.props.is_empty()
}

pub fn should_flip(pat: &Pattern, seed: &std::collections::HashMap<String, String>) -> bool {
    if pat.shortest {
        return false;
    }
    if pat.rels.len() != 1 || pat.nodes.len() != 2 {
        return false;
    }
    if keyed(&pat.nodes[0]) {
        return false;
    }
    if !keyed(&pat.nodes[1]) {
        return false;
    }
    if let Some(ref v) = pat.nodes[0].var {
        if seed.contains_key(v) {
            return false;
        }
    }
    true
}

pub fn flip_one_hop(pat: &Pattern) -> Pattern {
    let mut p = pat.clone();
    let n0 = p.nodes[0].clone();
    p.nodes[0] = p.nodes[1].clone();
    p.nodes[1] = n0;
    p.rels[0].dir = -p.rels[0].dir;
    p
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

pub fn start_seeds(g: &Graph, pat: &Pattern) -> Vec<String> {
    let n0 = &pat.nodes[0];
    if n0.type_id.is_some() || !n0.props.is_empty() {
        return seeds(g, n0);
    }
    if !pat.rels.is_empty() {
        if let Some(ref tid) = pat.rels[0].type_id {
            return starts_from_type(g, tid, pat.rels[0].dir, n0);
        }
    }
    seeds(g, n0)
}

fn starts_from_type(g: &Graph, tid: &str, dir: i32, n0: &NodePat) -> Vec<String> {
    let eids: Vec<String> = match g.ty(tid) {
        Some(t) => Khid::display_all(&t.edges().iter().cloned().collect::<Vec<_>>()),
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for eid in eids.iter() {
        let e = match g.edge(eid) {
            Some(e) => e,
            None => continue,
        };
        if dir >= 0 {
            let s = format!("{}", e.source());
            if node_ok(g, &s, n0) && !contains_id(&out, &s) {
                out.push(s);
            }
        }
        if dir <= 0 {
            let s = format!("{}", e.target());
            if node_ok(g, &s, n0) && !contains_id(&out, &s) {
                out.push(s);
            }
        }
    }
    out
}

fn wears(g: &Graph, vid: &str, tid: &str) -> bool {
    match g.vertex(vid) {
        Some(v) => {
            match Khid::parse(tid) {
                Some(k) => v.types().iter().any(|t| *t == k),
                None => false,
            }
        }
        None => false,
    }
}

pub fn node_ok(g: &Graph, vid: &str, n: &NodePat) -> bool {
    if let Some(ref tid) = n.type_id {
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

pub fn edges_of(g: &Graph, vid: &str, rel: &RelPat) -> Vec<String> {
    let v = match g.vertex(vid) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let mut ids = Vec::new();
    let src: Vec<String> = if rel.dir > 0 {
        Khid::display_all(v.outgoing())
    } else if rel.dir < 0 {
        Khid::display_all(v.incoming())
    } else {
        let mut both = Khid::display_all(v.outgoing());
        both.extend(Khid::display_all(v.incoming()));
        both
    };
    for eid in src.iter() {
        if let Some(ref tid) = rel.type_id {
            match g.edge(eid).and_then(|e| e.type_id()) {
                Some(et) if Some(et) == Khid::parse(tid) => {}
                _ => continue,
            }
        }
        ids.push(eid.clone());
    }
    ids
}
