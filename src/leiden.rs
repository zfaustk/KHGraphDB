//! Hierarchical cut. A first try.
//! The cluster is a vertex. PARENT is the hop.

use std::collections::{HashMap, HashSet};

use super::error::Result;
use super::graph::Graph;
use super::khid::Khid;

/// Greedy modularity on the undirected skeleton.
/// One Community per group. PARENT from member
/// to the cluster. Nested later.
pub fn cluster(g: &mut Graph) -> Result<Vec<Khid>> {
    let nodes = live_nodes(g);
    if nodes.is_empty() {
        return Ok(Vec::new());
    }
    let nbr = undirected(g, &nodes);
    let mut comm: HashMap<Khid, usize> = HashMap::new();
    for (i, id) in nodes.iter().enumerate() {
        comm.insert(*id, i);
    }
    let mut changed = true;
    let mut rounds = 0u32;
    while changed && rounds < 16 {
        changed = false;
        rounds += 1;
        for id in nodes.iter() {
            let here = comm[id];
            let mut count: HashMap<usize, usize> = HashMap::new();
            if let Some(ns) = nbr.get(id) {
                for n in ns.iter() {
                    let c = comm[n];
                    *count.entry(c).or_insert(0) += 1;
                }
            }
            let mut best = here;
            let mut best_n = 0usize;
            for (c, n) in count.iter() {
                if *n > best_n || (*n == best_n && *c < best) {
                    best_n = *n;
                    best = *c;
                }
            }
            if best != here && best_n > 0 {
                comm.insert(*id, best);
                changed = true;
            }
        }
    }
    let mut groups: HashMap<usize, Vec<Khid>> = HashMap::new();
    for id in nodes.iter() {
        groups.entry(comm[id]).or_insert_with(Vec::new).push(*id);
    }
    let _ = g.add_type("Community")?;
    let mut out = Vec::new();
    for (_, members) in groups {
        if members.len() < 2 {
            continue;
        }
        let empty: HashMap<String, String> = HashMap::new();
        let c = g.add_vertex(empty, Some("Community"))?;
        for m in members {
            if m != c {
                let _ = g.add_edge(m, c, Some("PARENT"));
            }
        }
        out.push(c);
    }
    Ok(out)
}

fn live_nodes(g: &Graph) -> Vec<Khid> {
    let mut v = Vec::new();
    for &(_, ref name) in g.all_types().iter() {
        if name == "Community" {
            continue;
        }
        for id in g.vertices_of_type(name) {
            v.push(id);
        }
    }
    v.sort();
    v.dedup();
    v
}

fn undirected(g: &Graph, nodes: &[Khid]) -> HashMap<Khid, Vec<Khid>> {
    let set: HashSet<Khid> = nodes.iter().cloned().collect();
    let mut nbr: HashMap<Khid, Vec<Khid>> = HashMap::new();
    for &(eid, src, dst, _) in g.all_edges().iter() {
        let _ = eid;
        if !set.contains(&src) || !set.contains(&dst) {
            continue;
        }
        if src == dst {
            continue;
        }
        nbr.entry(src).or_insert_with(Vec::new).push(dst);
        nbr.entry(dst).or_insert_with(Vec::new).push(src);
    }
    nbr
}

/// A second pass. Communities of communities.
/// PARENT from a cluster to its parent.
pub fn nest(g: &mut Graph, kids: &[Khid]) -> Result<Option<Khid>> {
    if kids.len() < 2 {
        return Ok(None);
    }
    let _ = g.add_type("Community")?;
    let empty: HashMap<String, String> = HashMap::new();
    let p = g.add_vertex(empty, Some("Community"))?;
    for c in kids {
        if g.vertex(*c).is_some() {
            let _ = g.add_edge(*c, p, Some("PARENT"));
        }
    }
    Ok(Some(p))
}
