use super::super::error::{Error, Result};
use super::super::graph::Graph;
use super::super::khid::Khid;
use super::super::prop::Prop;
use super::{Expr, NodePat, Path, Pattern, QueryResult, RelPat, RetItem, Val};

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

pub(crate) fn exec_pattern(g: &Graph, pat: &Pattern) -> QueryResult {
    exec_pattern_on(g, pat, &std::collections::HashMap::new())
}

pub(crate) fn exec_explain(g: &Graph, pat: &Pattern) -> Result<QueryResult> {
    let mut pat = pat.clone();
    if let Some(msg) = resolve_types(g, &mut pat, true) {
        return Ok(QueryResult::fail(&msg));
    }
    let mut r = QueryResult::ok_msg("EXPLAIN");
    r.columns.push("slot".to_string());
    r.columns.push("name".to_string());
    r.columns.push("khid".to_string());
    for (i, n) in pat.nodes.iter().enumerate() {
        let slot = n.var.clone().unwrap_or(format!("n{}", i));
        let name = n.type_name.clone().unwrap_or(String::new());
        let khid = n.type_id.clone().unwrap_or(String::new());
        r.rows.push(vec![
            Some(Val::Id(slot)),
            Some(Val::Id(name)),
            Some(Val::Id(khid)),
        ]);
    }
    for (i, rel) in pat.rels.iter().enumerate() {
        let slot = rel.var.clone().unwrap_or(format!("e{}", i));
        let name = rel.type_name.clone().unwrap_or(String::new());
        let khid = rel.type_id.clone().unwrap_or(String::new());
        r.rows.push(vec![
            Some(Val::Id(slot)),
            Some(Val::Id(name)),
            Some(Val::Id(khid)),
        ]);
    }
    Ok(r)
}

pub(crate) fn exec_match(g: &Graph, pat: &Pattern, prev: Option<QueryResult>) -> QueryResult {
    match prev {
        None => exec_pattern(g, pat),
        Some(src) => {
            if src.rows.is_empty() {
                return src;
            }
            let mut out = QueryResult::ok_msg("MATCH");
            let mut new_cols: Vec<String> = Vec::new();
            let mut first = true;
            for row in src.rows.iter() {
                let mut seed = std::collections::HashMap::new();
                let mut i = 0;
                while i < src.columns.len() {
                    if let Some(id) = row.get(i).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                        seed.insert(src.columns[i].clone(), id.to_string());
                    }
                    i += 1;
                }
                let r2 = exec_pattern_on(g, pat, &seed);
                if first {
                    out.columns = src.columns.clone();
                    for c in r2.columns.iter() {
                        if !contains_str(&out.columns, c) {
                            new_cols.push(c.clone());
                            out.columns.push(c.clone());
                        }
                    }
                    first = false;
                }
                if r2.rows.is_empty() {
                    continue;
                }
                for row2 in r2.rows.iter() {
                    let mut nr = row.clone();
                    for c in new_cols.iter() {
                        match r2.columns.iter().position(|x| x == c) {
                            Some(j) => nr.push(row2.get(j).cloned().unwrap_or(None)),
                            None => nr.push(None),
                        }
                    }
                    out.rows.push(nr);
                }
            }
            out.message = format!("{} row", out.rows.len());
            out
        }
    }
}

fn contains_str(cols: &Vec<String>, s: &str) -> bool {
    for c in cols.iter() {
        if c == s {
            return true;
        }
    }
    false
}

fn exec_pattern_on(g: &Graph, pat: &Pattern, seed: &std::collections::HashMap<String, String>) -> QueryResult {
    let mut pat = pat.clone();
    if let Some(msg) = resolve_types(g, &mut pat, true) {
        return QueryResult::fail(&msg);
    }
    let mut r = if pat.shortest {
        exec_shortest(g, &pat, seed)
    } else if pat.rels.is_empty() {
        exec_nodes(g, &pat, seed)
    } else {
        exec_chain(g, &pat, seed)
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

fn seed_ok(seed: &std::collections::HashMap<String, String>, n: &NodePat, id: &str) -> bool {
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

fn exec_shortest(g: &Graph, pat: &Pattern, seed: &std::collections::HashMap<String, String>) -> QueryResult {
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
        if !seed_ok(seed, &pat.nodes[0], s) {
            continue;
        }
        for t in ends.iter() {
            if !seed_ok(seed, &pat.nodes[1], t) {
                continue;
            }
            match super::super::algo::path_on(g, s, t, tid, rel.dir, rel.min, rel.max) {
                Some(path) => {
                    let bind = vec![Some(s.clone()), Some(t.clone())];
                    let mut rel_edges: Vec<Vec<String>> = vec![Vec::new(); pat.rels.len()];
                    if pat.rels.len() == 1 {
                        let mut es = Vec::new();
                        for k in Path::parse_all(&path).edges().iter() {
                            es.push(format!("{}", k));
                        }
                        rel_edges[0] = es;
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

fn exec_nodes(g: &Graph, pat: &Pattern, seed: &std::collections::HashMap<String, String>) -> QueryResult {
    let n = &pat.nodes[0];
    let found = seeds(g, n);
    let mut r = QueryResult::ok_msg("MATCH");
    r.columns = columns_of(pat);
    for id in found.iter() {
        if !seed_ok(seed, n, id) {
            continue;
        }
        let bind = vec![Some(id.clone())];
        let trail = vec![id.clone()];
        let rel_edges: Vec<Vec<String>> = Vec::new();
        emit_row(pat, &bind, &trail, &rel_edges, &mut r);
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn exec_chain(g: &Graph, pat: &Pattern, seed: &std::collections::HashMap<String, String>) -> QueryResult {
    let orig_cols = columns_of(pat);
    let flipped = should_flip(pat, seed);
    let walk_pat = if flipped {
        flip_one_hop(pat)
    } else {
        pat.clone()
    };
    let seeds0 = start_seeds(g, &walk_pat);
    let mut r = QueryResult::ok_msg("MATCH");
    r.columns = columns_of(&walk_pat);
    for s in seeds0.iter() {
        if !seed_ok(seed, &walk_pat.nodes[0], s) {
            continue;
        }
        let mut bind = vec![None; walk_pat.nodes.len()];
        bind[0] = Some(s.clone());
        let mut seen_v = vec![s.clone()];
        let mut seen_e: Vec<String> = Vec::new();
        let mut trail = vec![s.clone()];
        let mut rel_edges: Vec<Vec<String>> = Vec::new();
        let mut k = 0;
        while k < walk_pat.rels.len() {
            rel_edges.push(Vec::new());
            k += 1;
        }
        walk_named(g,
                   &walk_pat,
                   0,
                   &mut bind,
                   &mut seen_v,
                   &mut seen_e,
                   &mut trail,
                   &mut rel_edges,
                   seed,
                   &mut r);
    }
    if flipped {
        r = unflip_result(r, &orig_cols);
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
              seed: &std::collections::HashMap<String, String>,
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
               seed,
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
              seed: &std::collections::HashMap<String, String>,
              r: &mut QueryResult) {
    let rel = &pat.rels[rel_i];
    let next = &pat.nodes[rel_i + 1];
    if hops >= rel.min && hops <= rel.max && node_ok(g, u, next) && seed_ok(seed, next, u) {
        bind[rel_i + 1] = Some(u.to_string());
        walk_named(g, pat, rel_i + 1, bind, seen_v, seen_e, trail, rel_edges, seed, r);
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
                   seed,
                   r);
        rel_edges[rel_i].pop();
        trail.pop();
        trail.pop();
        seen_v.pop();
        seen_e.pop();
    }
}

fn seeds(g: &Graph, n: &NodePat) -> Vec<String> {
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
                Some(t) => t.vertices().iter().map(|s| s.clone()).collect(),
                None => Vec::new(),
            }
        }
        None => g.vertex_ids(),
    };
    src.into_iter().filter(|id| node_ok(g, id, n)).collect()
}

fn keyed(n: &NodePat) -> bool {
    n.type_name.is_some() && !n.props.is_empty()
}

fn should_flip(pat: &Pattern, seed: &std::collections::HashMap<String, String>) -> bool {
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

fn flip_one_hop(pat: &Pattern) -> Pattern {
    let mut p = pat.clone();
    let n0 = p.nodes[0].clone();
    p.nodes[0] = p.nodes[1].clone();
    p.nodes[1] = n0;
    p.rels[0].dir = -p.rels[0].dir;
    p
}

fn unflip_result(mut r: QueryResult, orig_cols: &[String]) -> QueryResult {
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
        match g.vertex(vid).and_then(|v| v.get_prop(k)) {
            Some(got) if got == val => {}
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
            match g.edge(eid).and_then(|e| e.type_id().map(|s| s.to_string())) {
                Some(ref et) if et == tid => {}
                _ => continue,
            }
        }
        ids.push(eid.clone());
    }
    ids
}

pub(crate) fn filter_where(g: &Graph, src: QueryResult, pred: &Expr) -> QueryResult {
    let mut r = QueryResult::ok_msg("WHERE");
    r.columns = src.columns.clone();
    for row in src.rows.iter() {
        if eval_expr(g, &src.columns, row, pred) {
            r.rows.push(row.clone());
        }
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn eval_expr(g: &Graph, cols: &Vec<String>, row: &Vec<Option<Val>>, e: &Expr) -> bool {
    match *e {
        Expr::Eq(ref var, ref key, ref val) => {
            match lookup_attr(g, cols, row, var, key) {
                Some(ref got) if got == val => true,
                _ => false,
            }
        }
        Expr::Cmp(ref var, ref key, op, ref val) => {
            match lookup_attr(g, cols, row, var, key) {
                Some(ref got) => cmp_prop(got, val, op),
                None => false,
            }
        }
        Expr::In(ref var, ref key, ref vals) => {
            match lookup_attr(g, cols, row, var, key) {
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
        Expr::And(ref a, ref b) => eval_expr(g, cols, row, a) && eval_expr(g, cols, row, b),
        Expr::Or(ref a, ref b) => eval_expr(g, cols, row, a) || eval_expr(g, cols, row, b),
        Expr::Not(ref a) => !eval_expr(g, cols, row, a),
    }
}

fn lookup_attr(g: &Graph, cols: &Vec<String>, row: &Vec<Option<Val>>, var: &str, key: &str) -> Option<Prop> {
    let col = match cols.iter().position(|c| c == var) {
        Some(i) => i,
        None => return None,
    };
    let id = match row.get(col).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
        Some(id) => id,
        None => return None,
    };
    if let Some(p) = g.vertex(id).and_then(|v| v.get_prop(key).cloned()) {
        return Some(p);
    }
    g.edge(id).and_then(|e| e.get_prop(key).cloned())
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

pub(crate) fn exec_set(g: &mut Graph, src: QueryResult, items: &Vec<(String, String, Prop)>) -> Result<QueryResult> {
    for row in src.rows.iter() {
        for &(ref var, ref key, ref val) in items.iter() {
            let col = match src.columns.iter().position(|c| c == var) {
                Some(i) => i,
                None => return Err(Error::new("SET unknown name")),
            };
            let id = match row.get(col).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                Some(id) => id.to_string(),
                None => return Err(Error::new("SET needs a node")),
            };
            if g.vertex(&id).is_some() {
                g.set_prop(&id, key, val.clone())?;
            } else if g.edge(&id).is_some() {
                if !g.set_edge_prop(&id, key, val.clone()) {
                    return Err(Error::new("SET missing"));
                }
            } else {
                return Err(Error::new("SET missing"));
            }
        }
    }
    let mut r = src;
    r.message = "set".to_string();
    Ok(r)
}

pub(crate) fn exec_remove(g: &mut Graph, src: QueryResult, items: &Vec<(String, String)>) -> Result<QueryResult> {
    for row in src.rows.iter() {
        for &(ref var, ref key) in items.iter() {
            let col = match src.columns.iter().position(|c| c == var) {
                Some(i) => i,
                None => return Err(Error::new("REMOVE unknown name")),
            };
            let id = match row.get(col).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                Some(id) => id.to_string(),
                None => return Err(Error::new("REMOVE needs a node")),
            };
            g.remove_attr(&id, key)?;
        }
    }
    let mut r = src;
    r.message = "removed".to_string();
    Ok(r)
}

pub(crate) fn exec_delete(g: &mut Graph,
                   src: QueryResult,
                   names: &Vec<String>,
                   detach: bool)
                   -> Result<QueryResult> {
    let mut deleted = 0;
    for row in src.rows.iter() {
        for name in names.iter() {
            let col = match src.columns.iter().position(|c| c == name) {
                Some(i) => i,
                None => return Err(Error::new("DELETE unknown name")),
            };
            let id = match row.get(col).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                Some(id) => id.to_string(),
                None => continue,
            };
            if g.edge(&id).is_some() {
                g.remove_edge(&id);
                deleted += 1;
                continue;
            }
            if g.vertex(&id).is_none() {
                continue;
            }
            if !detach {
                let (out_n, in_n) = match g.vertex(&id) {
                    Some(v) => (v.out_degree(), v.in_degree()),
                    None => (0, 0),
                };
                if out_n + in_n > 0 {
                    return Err(Error::new("DELETE edges"));
                }
            }
            g.remove_vertex(&id);
            deleted += 1;
        }
    }
    let mut r = QueryResult::ok_msg("deleted");
    r.columns = src.columns.clone();
    r.deleted = deleted;
    Ok(r)
}

fn path_fn(path: Option<&Path>, kind: i32) -> Option<Val> {
    let p = match path {
        Some(p) => p,
        None => return None,
    };
    if kind == 3 {
        return Some(Val::Id(format!("{}", p.hops())));
    }
    if kind == 4 {
        let mut ids = Vec::new();
        for n in p.nodes().iter() {
            ids.push(Val::Id(format!("{}", n)));
        }
        return Some(Val::List(ids));
    }
    if kind == 5 {
        let mut ids = Vec::new();
        for n in p.edges().iter() {
            ids.push(Val::Id(format!("{}", n)));
        }
        return Some(Val::List(ids));
    }
    None
}

pub(crate) fn project(src: &QueryResult, cols: &Vec<RetItem>) -> QueryResult {
    let mut agg = false;
    for c in cols.iter() {
        if c.kind == 1 || c.kind == 2 {
            agg = true;
            break;
        }
    }
    if !agg {
        let mut r = QueryResult::ok_msg("RETURN");
        let mut map = Vec::new();
        for c in cols.iter() {
            r.columns.push(c.alias.clone());
            map.push(src.columns.iter().position(|x| x == &c.name));
        }
        for row in src.rows.iter() {
            let mut nr = Vec::new();
            for m in map.iter().enumerate() {
                let (ci, pos) = m;
                let kind = cols[ci].kind;
                if kind == 0 {
                    match *pos {
                        Some(i) => nr.push(row.get(i).cloned().unwrap_or(None)),
                        None => nr.push(None),
                    }
                } else {
                    let path = match *pos {
                        Some(i) => row.get(i).and_then(|x| x.as_ref()).and_then(|v| v.as_path()),
                        None => None,
                    };
                    nr.push(path_fn(path, kind));
                }
            }
            r.rows.push(nr);
        }
        r.message = format!("{} row", r.rows.len());
        return r;
    }
    // group by non-agg columns
    let mut groups: Vec<(Vec<Option<Val>>, usize, Vec<Vec<Val>>)> = Vec::new();
    let mut n_collect = 0;
    for c in cols.iter() {
        if c.kind == 2 {
            n_collect += 1;
        }
    }
    for row in src.rows.iter() {
        let mut key = Vec::new();
        for c in cols.iter() {
            if c.kind == 0 {
                match src.columns.iter().position(|x| x == &c.name) {
                    Some(i) => key.push(row.get(i).cloned().unwrap_or(None)),
                    None => key.push(None),
                }
            }
        }
        let mut found = None;
        let mut gi = 0;
        while gi < groups.len() {
            if groups[gi].0 == key {
                found = Some(gi);
                break;
            }
            gi += 1;
        }
        let gi = match found {
            Some(i) => {
                groups[i].1 += 1;
                i
            }
            None => {
                let mut bags = Vec::new();
                let mut b = 0;
                while b < n_collect {
                    bags.push(Vec::new());
                    b += 1;
                }
                groups.push((key, 1, bags));
                groups.len() - 1
            }
        };
        let mut bi = 0;
        for c in cols.iter() {
            if c.kind == 2 {
                if let Some(i) = src.columns.iter().position(|x| x == &c.name) {
                    if let Some(Some(ref v)) = row.get(i).cloned() {
                        groups[gi].2[bi].push(v.clone());
                    }
                }
                bi += 1;
            }
        }
    }
    let mut r = QueryResult::ok_msg("RETURN");
    for c in cols.iter() {
        r.columns.push(c.alias.clone());
    }
    for &(ref key, n, ref bags) in groups.iter() {
        let mut nr = Vec::new();
        let mut ki = 0;
        let mut bi = 0;
        for c in cols.iter() {
            if c.kind == 0 {
                nr.push(key.get(ki).cloned().unwrap_or(None));
                ki += 1;
            } else if c.kind == 2 {
                let list = if bi < bags.len() {
                    bags[bi].clone()
                } else {
                    Vec::new()
                };
                nr.push(Some(Val::List(list)));
                bi += 1;
            } else {
                nr.push(Some(Val::Id(format!("{}", n))));
            }
        }
        r.rows.push(nr);
    }
    if groups.is_empty() && src.rows.is_empty() {
        let mut nr = Vec::new();
        for c in cols.iter() {
            if c.kind == 0 {
                nr.push(None);
            } else if c.kind == 2 {
                nr.push(Some(Val::List(Vec::new())));
            } else {
                nr.push(Some(Val::Id("0".to_string())));
            }
        }
        r.rows.push(nr);
    }
    r.message = format!("{} row", r.rows.len());
    r
}

pub(crate) fn order_by(g: &Graph, mut src: QueryResult, keys: &Vec<(String, Option<String>, bool)>) -> QueryResult {
    let cols = src.columns.clone();
    src.rows.sort_by(|a, b| {
        for &(ref var, ref key, desc) in keys.iter() {
            let ca = order_cell(g, &cols, a, var, key.as_ref().map(|s| &s[..]));
            let cb = order_cell(g, &cols, b, var, key.as_ref().map(|s| &s[..]));
            let ord = ca.cmp(&cb);
            if ord != std::cmp::Ordering::Equal {
                if desc {
                    return ord.reverse();
                }
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    });
    src.message = format!("{} row", src.rows.len());
    src
}

pub(crate) fn distinct_rows(mut src: QueryResult) -> QueryResult {
    let mut out: Vec<Vec<Option<Val>>> = Vec::new();
    for row in src.rows.iter() {
        let mut seen = false;
        for o in out.iter() {
            if o == row {
                seen = true;
                break;
            }
        }
        if !seen {
            out.push(row.clone());
        }
    }
    src.rows = out;
    src.message = format!("{} row", src.rows.len());
    src
}

pub(crate) fn exec_unwind(prev: Option<QueryResult>,
                   col: Option<String>,
                   lits: Vec<Val>,
                   alias: String)
                   -> QueryResult {
    let mut r = QueryResult::ok_msg("UNWIND");
    match prev {
        None => {
            r.columns.push(alias);
            if col.is_some() {
                return QueryResult::fail("UNWIND without MATCH");
            }
            for v in lits.iter() {
                r.rows.push(vec![Some(v.clone())]);
            }
        }
        Some(src) => {
            r.columns = src.columns.clone();
            r.columns.push(alias.clone());
            for row in src.rows.iter() {
                let items: Vec<Val> = if let Some(ref c) = col {
                    match src.columns.iter().position(|x| x == c) {
                        Some(i) => {
                            match row.get(i).and_then(|x| x.as_ref()) {
                                Some(&Val::List(ref xs)) => xs.clone(),
                                Some(v) => vec![v.clone()],
                                None => Vec::new(),
                            }
                        }
                        None => Vec::new(),
                    }
                } else {
                    lits.clone()
                };
                for v in items.iter() {
                    let mut nr = row.clone();
                    nr.push(Some(v.clone()));
                    r.rows.push(nr);
                }
            }
        }
    }
    r.message = format!("{} row", r.rows.len());
    r
}

pub(crate) fn skip_rows(mut src: QueryResult, n: usize) -> QueryResult {
    if n >= src.rows.len() {
        src.rows.clear();
    } else {
        src.rows = src.rows.split_off(n);
    }
    src.message = format!("{} row", src.rows.len());
    src
}

pub(crate) fn limit_rows(mut src: QueryResult, n: usize) -> QueryResult {
    if src.rows.len() > n {
        src.rows.truncate(n);
    }
    src.message = format!("{} row", src.rows.len());
    src
}

fn order_cell(g: &Graph,
              cols: &Vec<String>,
              row: &Vec<Option<Val>>,
              var: &str,
              key: Option<&str>) -> Prop {
    match key {
        Some(k) => lookup_attr(g, cols, row, var, k).unwrap_or(Prop::from_str("")),
        None => {
            let col = match cols.iter().position(|c| c == var) {
                Some(i) => i,
                None => return Prop::from_str(""),
            };
            match row.get(col).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                Some(id) => Prop::from_str(id),
                None => Prop::from_str(""),
            }
        }
    }
}

pub(crate) fn exec_create(g: &mut Graph, pat: &Pattern, prev: Option<&QueryResult>) -> Result<QueryResult> {
    let binds = create_binds(prev);
    let mut r = QueryResult::ok_msg("created");
    r.columns = match prev {
        Some(p) => {
            let mut cols = p.columns.clone();
            for c in create_columns(pat).iter() {
                if !contains_str(&cols, c) {
                    cols.push(c.clone());
                }
            }
            cols
        }
        None => create_columns(pat),
    };
    for seed in binds.iter() {
        let mut bound: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for (k, v) in seed.iter() {
            bound.insert(k.clone(), v.clone());
        }
        let mut created = 0;
        let mut node_ids: Vec<String> = Vec::new();
        for (i, n) in pat.nodes.iter().enumerate() {
            let key = n.var.clone().unwrap_or(format!("n{}", i));
            let id = if let Some(existing) = bound.get(&key).cloned() {
                existing
            } else {
                let id = create_node(g, n)?;
                bound.insert(key, id.clone());
                created += 1;
                id
            };
            node_ids.push(id);
        }
        let mut i = 0;
        while i < pat.rels.len() {
            let a = &node_ids[i];
            let b = &node_ids[i + 1];
            let tn = pat.rels[i].type_name.as_ref().map(|s| &s[..]);
            let eid = g.add_edge(a, b, tn)?;
            created += 1;
            if let Some(ref v) = pat.rels[i].var {
                bound.insert(v.clone(), eid.clone());
            }
            i += 1;
        }
        let mut row = Vec::new();
        for c in r.columns.iter() {
            match bound.get(c) {
                Some(id) => row.push(Some(Val::Id(id.clone()))),
                None => row.push(None),
            }
        }
        r.rows.push(row);
        r.created += created;
    }
    r.message = format!("{} created", r.rows.len());
    Ok(r)
}

fn create_columns(pat: &Pattern) -> Vec<String> {
    let mut cols = Vec::new();
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

fn create_binds(prev: Option<&QueryResult>) -> Vec<std::collections::HashMap<String, String>> {
    let mut out = Vec::new();
    match prev {
        None => {
            out.push(std::collections::HashMap::new());
        }
        Some(p) => {
            if p.rows.is_empty() {
                return out;
            }
            for row in p.rows.iter() {
                let mut m = std::collections::HashMap::new();
                let mut i = 0;
                while i < p.columns.len() {
                    if let Some(id) = row.get(i).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                        m.insert(p.columns[i].clone(), id.to_string());
                    }
                    i += 1;
                }
                out.push(m);
            }
        }
    }
    out
}

fn create_node(g: &mut Graph, n: &NodePat) -> Result<String> {
    let mut attrs = std::collections::HashMap::new();
    for &(ref k, ref v) in n.props.iter() {
        attrs.insert(k.clone(), v.clone());
    }
    g.add_vertex_props(attrs, n.type_name.as_ref().map(|s| &s[..]))
}

pub(crate) fn exec_merge(g: &mut Graph, pat: &Pattern) -> Result<QueryResult> {
    let mut pat = pat.clone();
    resolve_types(g, &mut pat, false);
    for rel in pat.rels.iter() {
        if rel.min != 1 || rel.max != 1 {
            return Err(Error::new("MERGE length"));
        }
    }
    if pat.rels.is_empty() {
        let mut r = merge_node(g, &pat.nodes[0])?;
        let created = r.message == "created";
        if created {
            r = exec_set(g, r, &pat.on_create)?;
            r.message = "created".to_string();
        } else {
            r = exec_set(g, r, &pat.on_match)?;
            r.message = "exists".to_string();
        }
        return Ok(r);
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
        Some(v) => Khid::display_all(v.outgoing()),
        None => Vec::new(),
    };
    let mut cols = Vec::new();
    cols.push(pat.nodes[0].var.clone().unwrap_or("a".to_string()));
    if let Some(ref v) = rel.var {
        cols.push(v.clone());
    }
    cols.push(pat.nodes[1].var.clone().unwrap_or("b".to_string()));
    for eid in eids.iter() {
        if let Some(e) = g.edge(eid) {
            if e.target() == b {
                let ok_t = match rel.type_id {
                    Some(ref tid) => e.type_id() == Some(&tid[..]),
                    None => true,
                };
                if ok_t {
                    let mut r = QueryResult::ok_msg("exists");
                    r.columns = cols;
                    let mut row = vec![Some(Val::Id(a.clone()))];
                    if rel.var.is_some() {
                        row.push(Some(Val::Id(eid.clone())));
                    }
                    row.push(Some(Val::Id(b.clone())));
                    r.rows.push(row);
                    r = exec_set(g, r, &pat.on_match)?;
                    r.message = "exists".to_string();
                    return Ok(r);
                }
            }
        }
    }
    let eid = g.add_edge(&a, &b, rel.type_name.as_ref().map(|s| &s[..]))?;
    let mut r = QueryResult::ok_msg("created");
    r.columns = cols;
    let mut row = vec![Some(Val::Id(a))];
    if rel.var.is_some() {
        row.push(Some(Val::Id(eid)));
    }
    row.push(Some(Val::Id(b)));
    r.rows.push(row);
    r = exec_set(g, r, &pat.on_create)?;
    r.message = "created".to_string();
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
    let id = g.add_vertex_props(attrs, n.type_name.as_ref().map(|s| &s[..]))?;
    let mut r = QueryResult::ok_msg("created");
    r.columns.push(n.var.clone().unwrap_or("n".to_string()));
    r.rows.push(vec![Some(Val::Id(id))]);
    Ok(r)
}
