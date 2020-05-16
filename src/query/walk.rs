//! Writes and the row operators. MATCH itself compiles
//! in op.rs. CREATE, SET, DELETE, MERGE, WHERE, RETURN
//! still live here: they take a table of rows.
//! A bound id is a Khid. A name or a count is a Prop.

use std::collections::HashMap;

use crate::error::{Error, Result};
use crate::graph::Graph;
use crate::khid::Khid;
use crate::prop::Prop;
use super::{Expr, NodePat, Path, Pattern, QueryResult, RetItem, Val};
use super::scan;
use super::op;

fn name_val(s: &str) -> Option<Val> {
    Some(Val::Prop(Prop::from_str(s)))
}

pub(crate) fn exec_pattern(g: &Graph, pat: &Pattern) -> QueryResult {
    exec_pattern_on(g, pat, &HashMap::new())
}

pub(crate) fn exec_explain(g: &Graph, pat: &Pattern) -> Result<QueryResult> {
    let mut pat = pat.clone();
    if let Some(msg) = scan::resolve_types(g, &mut pat, true) {
        return Ok(QueryResult::fail(&msg));
    }
    scan::name_slots(&mut pat);
    let mut r = QueryResult::ok_msg("EXPLAIN");
    r.columns.push("slot".to_string());
    r.columns.push("name".to_string());
    r.columns.push("khid".to_string());
    for (i, n) in pat.nodes.iter().enumerate() {
        let slot = n.var.clone().unwrap_or(format!("n{}", i));
        let name = n.type_name.clone().unwrap_or(String::new());
        r.rows.push(vec![
            name_val(&slot),
            name_val(&name),
            n.type_id.map(Val::Id),
        ]);
    }
    for (i, rel) in pat.rels.iter().enumerate() {
        let slot = rel.var.clone().unwrap_or(format!("e{}", i));
        let name = rel.type_name.clone().unwrap_or(String::new());
        r.rows.push(vec![
            name_val(&slot),
            name_val(&name),
            rel.type_id.map(Val::Id),
        ]);
    }
    let op = super::op::compile(&pat);
    r.rows.push(vec![
        name_val("plan"),
        name_val(op.kind()),
        name_val(&op.describe()),
    ]);
    for k in op.kinds().iter() {
        r.rows.push(vec![
            name_val("op"),
            name_val(*k),
            None,
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
                let mut seed = HashMap::new();
                let mut i = 0;
                while i < src.columns.len() {
                    if let Some(id) = row.get(i).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                        seed.insert(src.columns[i].clone(), id);
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

fn exec_pattern_on(g: &Graph, pat: &Pattern, seed: &HashMap<String, Khid>) -> QueryResult {
    op::run(g, pat, seed)
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
    if let Some(e) = g.edge(id) {
        if key == "title" {
            if let Some(t) = g.cite_title(id) {
                return Some(Prop::from_str(t));
            }
        }
        return e.get_prop(key).cloned();
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

pub(crate) fn exec_set(g: &mut Graph, src: QueryResult, items: &Vec<(String, String, Prop)>) -> Result<QueryResult> {
    for row in src.rows.iter() {
        for &(ref var, ref key, ref val) in items.iter() {
            let col = match src.columns.iter().position(|c| c == var) {
                Some(i) => i,
                None => return Err(Error::new("SET unknown name")),
            };
            let k = match row.get(col).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                Some(k) => k,
                None => return Err(Error::new("SET needs a node")),
            };
            if g.vertex(k).is_some() {
                g.set_prop(k, key, val.clone())?;
            } else if g.edge(k).is_some() {
                if !g.set_edge_prop(k, key, val.clone()) {
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
            let k = match row.get(col).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                Some(k) => k,
                None => return Err(Error::new("REMOVE needs a node")),
            };
            g.remove_attr(k, key)?;
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
            let k = match row.get(col).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                Some(k) => k,
                None => continue,
            };
            if g.edge(k).is_some() {
                g.remove_edge(k);
                deleted += 1;
                continue;
            }
            if g.vertex(k).is_none() {
                continue;
            }
            if !detach {
                let (out_n, in_n) = match g.vertex(k) {
                    Some(v) => (v.out_degree(), v.in_degree()),
                    None => (0, 0),
                };
                if out_n + in_n > 0 {
                    return Err(Error::new("DELETE edges"));
                }
            }
            g.remove_vertex(k);
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
        return Some(Val::Prop(Prop::from_int(p.hops() as i64)));
    }
    if kind == 4 {
        let mut ids = Vec::new();
        for n in p.nodes().iter() {
            ids.push(Val::Id(*n));
        }
        return Some(Val::List(ids));
    }
    if kind == 5 {
        let mut ids = Vec::new();
        for n in p.edges().iter() {
            ids.push(Val::Id(*n));
        }
        return Some(Val::List(ids));
    }
    None
}

pub(crate) fn project(g: &Graph, src: &QueryResult, cols: &Vec<RetItem>) -> QueryResult {
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
                } else if kind == 6 {
                    let key = match cols[ci].key {
                        Some(ref k) => k.clone(),
                        None => String::new(),
                    };
                    nr.push(lookup_attr(g, &src.columns, row, &cols[ci].name, &key)
                        .map(Val::Prop));
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
            } else if c.kind == 6 {
                let k = match c.key {
                    Some(ref s) => s.clone(),
                    None => String::new(),
                };
                key.push(lookup_attr(g, &src.columns, row, &c.name, &k).map(Val::Prop));
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
            if c.kind == 0 || c.kind == 6 {
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
                nr.push(Some(Val::Prop(Prop::from_int(n as i64))));
            }
        }
        r.rows.push(nr);
    }
    if groups.is_empty() && src.rows.is_empty() {
        let mut nr = Vec::new();
        for c in cols.iter() {
            if c.kind == 0 || c.kind == 6 {
                nr.push(None);
            } else if c.kind == 2 {
                nr.push(Some(Val::List(Vec::new())));
            } else {
                nr.push(Some(Val::Prop(Prop::from_int(0))));
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
                Some(id) => Prop::from_str(&format!("{}", id)),
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
        let mut bound: HashMap<String, Khid> = HashMap::new();
        for (k, v) in seed.iter() {
            bound.insert(k.clone(), *v);
        }
        let mut created = 0;
        let mut node_ids: Vec<Khid> = Vec::new();
        for (i, n) in pat.nodes.iter().enumerate() {
            let key = n.var.clone().unwrap_or(format!("n{}", i));
            let id = if let Some(&existing) = bound.get(&key) {
                existing
            } else {
                let id = create_node(g, n)?;
                bound.insert(key, id);
                created += 1;
                id
            };
            node_ids.push(id);
        }
        let mut i = 0;
        while i < pat.rels.len() {
            let a = node_ids[i];
            let b = node_ids[i + 1];
            let tn = pat.rels[i].type_name.as_ref().map(|s| &s[..]);
            let eid = g.add_edge(a, b, tn)?;
            created += 1;
            if let Some(ref v) = pat.rels[i].var {
                bound.insert(v.clone(), eid);
            }
            i += 1;
        }
        let mut row = Vec::new();
        for c in r.columns.iter() {
            match bound.get(c) {
                Some(&id) => row.push(Some(Val::Id(id))),
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

fn create_binds(prev: Option<&QueryResult>) -> Vec<HashMap<String, Khid>> {
    let mut out = Vec::new();
    match prev {
        None => {
            out.push(HashMap::new());
        }
        Some(p) => {
            if p.rows.is_empty() {
                return out;
            }
            for row in p.rows.iter() {
                let mut m = HashMap::new();
                let mut i = 0;
                while i < p.columns.len() {
                    if let Some(id) = row.get(i).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                        m.insert(p.columns[i].clone(), id);
                    }
                    i += 1;
                }
                out.push(m);
            }
        }
    }
    out
}

fn create_node(g: &mut Graph, n: &NodePat) -> Result<Khid> {
    let mut attrs = HashMap::new();
    for &(ref k, ref v) in n.props.iter() {
        attrs.insert(k.clone(), v.clone());
    }
    g.add_vertex_props(attrs, n.type_name.as_ref().map(|s| &s[..]))
}

pub(crate) fn exec_merge(g: &mut Graph, pat: &Pattern) -> Result<QueryResult> {
    let mut pat = pat.clone();
    scan::resolve_types(g, &mut pat, false);
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
        Some(id) => id,
        None => return Err(Error::new("MERGE nodes")),
    };
    let b = match right.rows.get(0).and_then(|r| r.get(0)).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
        Some(id) => id,
        None => return Err(Error::new("MERGE nodes")),
    };
    let rel = &pat.rels[0];
    let eids: Vec<Khid> = match g.vertex(a) {
        Some(v) => v.outgoing().iter().cloned().collect(),
        None => Vec::new(),
    };
    let mut cols = Vec::new();
    cols.push(pat.nodes[0].var.clone().unwrap_or("a".to_string()));
    if let Some(ref v) = rel.var {
        cols.push(v.clone());
    }
    cols.push(pat.nodes[1].var.clone().unwrap_or("b".to_string()));
    for eid in eids.iter() {
        if let Some(e) = g.edge(*eid) {
            if e.target() == b {
                let ok_t = match rel.type_id {
                    Some(tid) => e.type_id() == Some(tid),
                    None => true,
                };
                if ok_t {
                    let mut r = QueryResult::ok_msg("exists");
                    r.columns = cols;
                    let mut row = vec![Some(Val::Id(a))];
                    if rel.var.is_some() {
                        row.push(Some(Val::Id(*eid)));
                    }
                    row.push(Some(Val::Id(b)));
                    r.rows.push(row);
                    r = exec_set(g, r, &pat.on_match)?;
                    r.message = "exists".to_string();
                    return Ok(r);
                }
            }
        }
    }
    let eid = g.add_edge(a, b, rel.type_name.as_ref().map(|s| &s[..]))?;
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
    let found = scan::seeds(g, n);
    if !found.is_empty() {
        let mut r = QueryResult::ok_msg("exists");
        r.columns.push(n.var.clone().unwrap_or("n".to_string()));
        for id in found.iter() {
            r.rows.push(vec![Some(Val::Id(*id))]);
        }
        return Ok(r);
    }
    let mut attrs = HashMap::new();
    for &(ref k, ref v) in n.props.iter() {
        attrs.insert(k.clone(), v.clone());
    }
    let id = g.add_vertex_props(attrs, n.type_name.as_ref().map(|s| &s[..]))?;
    let mut r = QueryResult::ok_msg("created");
    r.columns.push(n.var.clone().unwrap_or("n".to_string()));
    r.rows.push(vec![Some(Val::Id(id))]);
    Ok(r)
}
