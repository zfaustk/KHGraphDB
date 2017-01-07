use super::super::error::{Error, Result};
use super::super::graph::Graph;
use super::{NodePat, Path, Pattern, QueryResult, RelPat, Val};

fn columns_of(pat: &Pattern) -> Vec<String> {
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

fn emit_row(pat: &Pattern,
            bind: &[Option<String>],
            trail: &[String],
            rel_edges: &[Vec<String>],
            r: &mut QueryResult) {
    let mut row = Vec::new();
    if pat.path_var.is_some() {
        row.push(Some(Val::Path(Path::new(trail.to_vec()))));
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
                    if edges.len() == 1 {
                        row.push(Some(Val::Id(edges[0].clone())));
                    } else {
                        row.push(None);
                    }
                }
        }
    }
    r.rows.push(row);
}

pub fn exec_pattern(g: &Graph, pat: &Pattern) -> QueryResult {
    let mut pat = pat.clone();
    if let Some(msg) = resolve_types(g, &mut pat, true) {
        return QueryResult::fail(&msg);
    }
    let mut r = if pat.shortest {
        exec_shortest(g, &pat)
    } else if pat.rels.is_empty() {
        exec_nodes(g, &pat)
    } else {
        exec_chain(g, &pat)
    };
    if pat.optional && r.rows.is_empty() {
        let mut row = Vec::new();
        if r.columns.is_empty() {
            r.columns.push("n".to_string());
        }
        for _ in 0..r.columns.len() {
            row.push(None);
        }
        r.rows.push(row);
        r.message = "optional empty".to_string();
    }
    r
}

fn resolve_types(g: &Graph, pat: &mut Pattern, required: bool) -> Option<String> {
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

fn exec_shortest(g: &Graph, pat: &Pattern) -> QueryResult {
    if pat.rels.len() != 1 {
        return QueryResult::fail("shortestPath");
    }
    let starts = start_seeds(g, pat);
    let ends = seeds(g, &pat.nodes[1]);
    let rel = &pat.rels[0];
    let tid = match rel.type_id {
        Some(ref s) => Some(&s[..]),
        None => None,
    };
    let mut r = QueryResult::ok_msg("MATCH");
    r.columns = columns_of(pat);
    for s in starts.iter() {
        for t in ends.iter() {
            match super::super::algo::path_on(g, s, t, tid, rel.dir, rel.min, rel.max) {
                Some(path) => {
                    let bind = vec![Some(s.clone()), Some(t.clone())];
                    let mut rel_edges: Vec<Vec<String>> = vec![Vec::new(); pat.rels.len()];
                    if pat.rels.len() == 1 {
                        rel_edges[0] = Path::new(path.clone()).edges();
                    }
                    emit_row(pat, &bind, &path, &rel_edges, &mut r);
                }
                None => {}
            }
        }
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn exec_nodes(g: &Graph, pat: &Pattern) -> QueryResult {
    let n = &pat.nodes[0];
    let found = seeds(g, n);
    let mut r = QueryResult::ok_msg("MATCH");
    r.columns = columns_of(pat);
    for id in found.iter() {
        let bind = vec![Some(id.clone())];
        let trail = vec![id.clone()];
        let rel_edges: Vec<Vec<String>> = Vec::new();
        emit_row(pat, &bind, &trail, &rel_edges, &mut r);
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn exec_chain(g: &Graph, pat: &Pattern) -> QueryResult {
    let seeds0 = start_seeds(g, pat);
    let mut r = QueryResult::ok_msg("MATCH");
    r.columns = columns_of(pat);
    for s in seeds0.iter() {
        let mut bind = vec![None; pat.nodes.len()];
        bind[0] = Some(s.clone());
        let mut seen_v = vec![s.clone()];
        let mut seen_e: Vec<String> = Vec::new();
        let mut trail = vec![s.clone()];
        let mut rel_edges: Vec<Vec<String>> = Vec::new();
        let mut k = 0;
        while k < pat.rels.len() {
            rel_edges.push(Vec::new());
            k += 1;
        }
        walk_named(g,
                   pat,
                   0,
                   &mut bind,
                   &mut seen_v,
                   &mut seen_e,
                   &mut trail,
                   &mut rel_edges,
                   &mut r);
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn walk_named(g: &Graph,
              pat: &Pattern,
              node_i: usize,
              bind: &mut Vec<Option<String>>,
              seen_v: &mut Vec<String>,
              seen_e: &mut Vec<String>,
              trail: &mut Vec<String>,
              rel_edges: &mut Vec<Vec<String>>,
              r: &mut QueryResult) {
    if node_i == pat.rels.len() {
        emit_row(pat, bind, trail, rel_edges, r);
        return;
    }
    let from = match bind[node_i] {
        Some(ref id) => id.clone(),
        None => return,
    };
    expand_rel(g,
               pat,
               node_i,
               &from,
               0,
               bind,
               seen_v,
               seen_e,
               trail,
               rel_edges,
               r);
}

fn contains_id(ids: &Vec<String>, id: &str) -> bool {
    for x in ids.iter() {
        if x == id {
            return true;
        }
    }
    false
}

fn expand_rel(g: &Graph,
              pat: &Pattern,
              rel_i: usize,
              u: &str,
              hops: usize,
              bind: &mut Vec<Option<String>>,
              seen_v: &mut Vec<String>,
              seen_e: &mut Vec<String>,
              trail: &mut Vec<String>,
              rel_edges: &mut Vec<Vec<String>>,
              r: &mut QueryResult) {
    let rel = &pat.rels[rel_i];
    let next = &pat.nodes[rel_i + 1];
    if hops >= rel.min && hops <= rel.max && node_ok(g, u, next) {
        bind[rel_i + 1] = Some(u.to_string());
        walk_named(g, pat, rel_i + 1, bind, seen_v, seen_e, trail, rel_edges, r);
        bind[rel_i + 1] = None;
    }
    if hops >= rel.max {
        return;
    }
    let eids = edges_of(g, u, rel);
    for eid in eids.iter() {
        if contains_id(seen_e, eid) {
            continue;
        }
        let e = match g.edge(eid) {
            Some(e) => e,
            None => continue,
        };
        let v = if rel.dir < 0 {
            e.source().to_string()
        } else if rel.dir == 0 {
            if e.source() == u {
                e.target().to_string()
            } else {
                e.source().to_string()
            }
        } else {
            e.target().to_string()
        };
        if contains_id(seen_v, &v) {
            continue;
        }
        seen_e.push(eid.clone());
        seen_v.push(v.clone());
        trail.push(eid.clone());
        trail.push(v.clone());
        rel_edges[rel_i].push(eid.clone());
        expand_rel(g,
                   pat,
                   rel_i,
                   &v,
                   hops + 1,
                   bind,
                   seen_v,
                   seen_e,
                   trail,
                   rel_edges,
                   r);
        rel_edges[rel_i].pop();
        trail.pop();
        trail.pop();
        seen_v.pop();
        seen_e.pop();
    }
}

fn seeds(g: &Graph, n: &NodePat) -> Vec<String> {
    if n.type_name.is_some() && !n.props.is_empty() {
        let tn = n.type_name.as_ref().unwrap();
        let &(ref k, ref val) = &n.props[0];
        let found = g.find(tn, k, val);
        return found.into_iter().filter(|id| node_ok(g, id, n)).collect();
    }
    let src: Vec<String> = match n.type_id {
        Some(ref tid) => {
            match g.ty(tid) {
                Some(t) => t.vertices().iter().map(|s| s.clone()).collect(),
                None => Vec::new(),
            }
        }
        None => g.vertex_ids(),
    };
    src.into_iter().filter(|id| node_ok(g, id, n)).collect()
}

fn start_seeds(g: &Graph, pat: &Pattern) -> Vec<String> {
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
        Some(t) => t.edges().iter().map(|s| s.clone()).collect(),
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for eid in eids.iter() {
        let e = match g.edge(eid) {
            Some(e) => e,
            None => continue,
        };
        if dir >= 0 {
            let s = e.source();
            if node_ok(g, s, n0) && !contains_id(&out, s) {
                out.push(s.to_string());
            }
        }
        if dir <= 0 {
            let s = e.target();
            if node_ok(g, s, n0) && !contains_id(&out, s) {
                out.push(s.to_string());
            }
        }
    }
    out
}

fn wears(g: &Graph, vid: &str, tid: &str) -> bool {
    match g.vertex(vid) {
        Some(v) => {
            for t in v.types().iter() {
                if t == tid {
                    return true;
                }
            }
            false
        }
        None => false,
    }
}

fn node_ok(g: &Graph, vid: &str, n: &NodePat) -> bool {
    if let Some(ref tid) = n.type_id {
        if !wears(g, vid, tid) {
            return false;
        }
    }
    for &(ref k, ref val) in n.props.iter() {
        match g.vertex(vid).and_then(|v| v.get(k).map(|s| s.to_string())) {
            Some(ref got) if got == val => {}
            _ => return false,
        }
    }
    true
}

fn edges_of(g: &Graph, vid: &str, rel: &RelPat) -> Vec<String> {
    let v = match g.vertex(vid) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let mut ids = Vec::new();
    let src: Vec<String> = if rel.dir > 0 {
        v.outgoing().iter().map(|s| s.clone()).collect()
    } else if rel.dir < 0 {
        v.incoming().iter().map(|s| s.clone()).collect()
    } else {
        let mut both = v.outgoing().iter().map(|s| s.clone()).collect::<Vec<_>>();
        both.extend(v.incoming().iter().map(|s| s.clone()));
        both
    };
    for eid in src.iter() {
        if let Some(ref tid) = rel.type_id {
            match g.edge(eid).and_then(|e| e.type_id().map(|s| s.to_string())) {
                Some(ref et) if et == tid => {}
                _ => continue,
            }
        }
        ids.push(eid.clone());
    }
    ids
}

pub fn filter_where(g: &Graph, src: QueryResult, preds: &Vec<(String, String, String)>) -> QueryResult {
    let mut r = QueryResult::ok_msg("WHERE");
    r.columns = src.columns.clone();
    for row in src.rows.iter() {
        let mut ok = true;
        for &(ref var, ref key, ref val) in preds.iter() {
            let col = match src.columns.iter().position(|c| c == var) {
                Some(i) => i,
                None => {
                    ok = false;
                    break;
                }
            };
            let vid = match row.get(col).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                Some(id) => id,
                None => {
                    ok = false;
                    break;
                }
            };
            match g.vertex(vid).and_then(|v| v.get(key).map(|s| s.to_string())) {
                Some(ref got) if got == val => {}
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            r.rows.push(row.clone());
        }
    }
    r.message = format!("{} row", r.rows.len());
    r
}

pub fn project(src: &QueryResult, cols: &Vec<String>) -> QueryResult {
    let mut r = QueryResult::ok_msg("RETURN");
    let mut map = Vec::new();
    for c in cols.iter() {
        r.columns.push(c.clone());
        map.push(src.columns.iter().position(|x| x == c));
    }
    for row in src.rows.iter() {
        let mut nr = Vec::new();
        for m in map.iter() {
            match *m {
                Some(i) => nr.push(row.get(i).cloned().unwrap_or(None)),
                None => nr.push(None),
            }
        }
        r.rows.push(nr);
    }
    r.message = format!("{} row", r.rows.len());
    r
}

pub fn exec_merge(g: &mut Graph, pat: &Pattern) -> Result<QueryResult> {
    let mut pat = pat.clone();
    resolve_types(g, &mut pat, false);
    for rel in pat.rels.iter() {
        if rel.min != 1 || rel.max != 1 {
            return Err(Error::new("MERGE length"));
        }
    }
    if pat.rels.is_empty() {
        return merge_node(g, &pat.nodes[0]);
    }
    let left = merge_node(g, &pat.nodes[0])?;
    let right = merge_node(g, &pat.nodes[1])?;
    let a = match left.rows.get(0).and_then(|r| r.get(0)).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
        Some(id) => id.to_string(),
        None => return Err(Error::new("MERGE nodes")),
    };
    let b = match right.rows.get(0).and_then(|r| r.get(0)).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
        Some(id) => id.to_string(),
        None => return Err(Error::new("MERGE nodes")),
    };
    let rel = &pat.rels[0];
    let eids: Vec<String> = match g.vertex(&a) {
        Some(v) => v.outgoing().iter().map(|s| s.clone()).collect(),
        None => Vec::new(),
    };
    for eid in eids.iter() {
        if let Some(e) = g.edge(eid) {
            if e.target() == b {
                let ok_t = match rel.type_id {
                    Some(ref tid) => e.type_id() == Some(&tid[..]),
                    None => true,
                };
                if ok_t {
                    let mut r = QueryResult::ok_msg("exists");
                    r.rows.push(vec![Some(Val::Id(a)), Some(Val::Id(b))]);
                    return Ok(r);
                }
            }
        }
    }
    g.add_edge(&a, &b, rel.type_name.as_ref().map(|s| &s[..]))?;
    let mut r = QueryResult::ok_msg("created");
    r.rows.push(vec![Some(Val::Id(a)), Some(Val::Id(b))]);
    Ok(r)
}

fn merge_node(g: &mut Graph, n: &NodePat) -> Result<QueryResult> {
    let found = seeds(g, n);
    if !found.is_empty() {
        let mut r = QueryResult::ok_msg("exists");
        r.columns.push(n.var.clone().unwrap_or("n".to_string()));
        for id in found.iter() {
            r.rows.push(vec![Some(Val::Id(id.clone()))]);
        }
        return Ok(r);
    }
    let mut attrs = std::collections::HashMap::new();
    for &(ref k, ref v) in n.props.iter() {
        attrs.insert(k.clone(), v.clone());
    }
    let id = g.add_vertex(attrs, n.type_name.as_ref().map(|s| &s[..]))?;
    let mut r = QueryResult::ok_msg("created");
    r.columns.push(n.var.clone().unwrap_or("n".to_string()));
    r.rows.push(vec![Some(Val::Id(id))]);
    Ok(r)
}
