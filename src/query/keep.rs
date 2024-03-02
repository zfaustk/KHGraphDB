//! Fill a view. The recipe is a read. Hits cite
//! addresses. Fold is the allowed depth of soup.

use std::collections::HashMap;

use crate::addr::Addr;
use crate::error::{Error, Result};
use crate::graph::Graph;
use crate::khid::Khid;
use super::{ask, writes, Val};

/// Run the Type's query. Each row becomes one hit
/// that cites the ids in that row. A vertex that
/// wears this Type is skipped. Fold 0 cites only
/// the world. Fold n may cite a hit of depth n.
pub fn keep(g: &mut Graph, type_name: &str, fold: u32) -> Result<usize> {
    keep_at(g, type_name, fold, None)
}

pub fn keep_at(g: &mut Graph,
               type_name: &str,
               fold: u32,
               pos: Option<&str>)
               -> Result<usize> {
    let q = match g.view_of(type_name) {
        Some(s) => s.to_string(),
        None => return Err(Error::new("not a view")),
    };
    if writes(&q) {
        return Err(Error::new("recipe writes"));
    }
    let r = ask(g, &q);
    if !r.ok {
        return Err(Error::new(&r.message));
    }
    let tid = match g.type_by_name(type_name) {
        Some(t) => t.khid(),
        None => return Err(Error::new("not a view")),
    };
    let stamp = match pos {
        Some(p) if !p.is_empty() => Some(p.to_string()),
        _ => g.look_pos(),
    };
    let mut n = 0usize;
    for row in r.rows.iter() {
        let mut srcs: Vec<Addr> = Vec::new();
        for cell in row.iter() {
            push_ids(g, cell.as_ref(), fold, tid, &mut srcs);
        }
        srcs.sort();
        srcs.dedup();
        if srcs.is_empty() {
            continue;
        }
        if g.hit_citing(tid, &srcs).is_some() {
            continue;
        }
        let hit = g.add_vertex(HashMap::new(), Some(type_name))?;
        if let Some(ref p) = stamp {
            let _ = g.stamp_pos(hit, p);
        }
        for a in srcs.iter() {
            g.derive_from(hit, *a)?;
        }
        let _ = g.enclose_open(hit);
        n += 1;
    }
    Ok(n)
}

fn push_ids(g: &Graph,
            cell: Option<&Val>,
            fold: u32,
            tid: Khid,
            out: &mut Vec<Addr>) {
    let v = match cell {
        Some(v) => v,
        None => return,
    };
    match *v {
        Val::Id(id) => push_id(g, id, fold, tid, out),
        Val::Path(ref p) => {
            for id in p.nodes() {
                push_id(g, id, fold, tid, out);
            }
        }
        Val::List(ref xs) => {
            for x in xs.iter() {
                push_ids(g, Some(x), fold, tid, out);
            }
        }
        Val::Prop(_) => {}
    }
}

fn push_id(g: &Graph, id: Khid, fold: u32, tid: Khid, out: &mut Vec<Addr>) {
    if g.vertex(id).is_none() {
        return;
    }
    if g.wears(id, tid) {
        return;
    }
    if g.view_depth(id) > fold {
        return;
    }
    out.push(g.addr(id));
}
