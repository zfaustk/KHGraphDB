use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use super::graph::Graph;
use super::khid::Khid;

/// Scratch lives next to the walk, keyed by KHID.
/// Attributes stay clean. That is AlgorithmObjs, without
/// writing on the vertex.

pub fn nearby(g: &Graph, start: Khid, depth: i32) -> Vec<Khid> {
    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    let mut out = Vec::new();
    seen.insert(start);
    q.push_back((start, 0i32));
    while let Some((u, d)) = q.pop_front() {
        if d > 0 {
            out.push(u);
        }
        if d >= depth {
            continue;
        }
        let eids: Vec<Khid> = match g.vertex(u) {
            Some(v) => v.outgoing().iter().cloned().collect(),
            None => Vec::new(),
        };
        for eid in eids.iter() {
            if let Some(e) = g.edge(*eid) {
                let w = e.target();
                if seen.insert(w) {
                    q.push_back((w, d + 1));
                }
            }
        }
    }
    out
}

pub fn path(g: &Graph, start: Khid, goal: Khid) -> Option<Vec<Khid>> {
    if start == goal {
        return Some(vec![start]);
    }
    let mut pred: HashMap<Khid, Khid> = HashMap::new();
    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    seen.insert(start);
    q.push_back(start);
    let mut found = false;
    while let Some(u) = q.pop_front() {
        let eids: Vec<Khid> = match g.vertex(u) {
            Some(v) => v.outgoing().iter().cloned().collect(),
            None => Vec::new(),
        };
        for eid in eids.iter() {
            if let Some(e) = g.edge(*eid) {
                let w = e.target();
                if seen.insert(w) {
                    pred.insert(w, u);
                    if w == goal {
                        found = true;
                        break;
                    }
                    q.push_back(w);
                }
            }
        }
        if found {
            break;
        }
    }
    if !found {
        return None;
    }
    Some(walk_pred(&pred, start, goal))
}

/// Hop-count BFS along typed edges. `type_id` is the Type's KHID.
/// `dir` is 1 out, -1 in, 0 both.
/// The walk is node, edge, node. Weighted remains `shortest`.
pub fn path_on(g: &Graph,
               start: Khid,
               goal: Khid,
               type_id: Option<Khid>,
               dir: i32,
               min_hops: usize,
               max_hops: usize)
               -> Option<Vec<Khid>> {
    if start == goal && min_hops == 0 {
        return Some(vec![start]);
    }
    let mut pred: HashMap<Khid, (Khid, Khid)> = HashMap::new();
    let mut dist: HashMap<Khid, usize> = HashMap::new();
    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    seen.insert(start);
    dist.insert(start, 0);
    q.push_back(start);
    let mut found = false;
    while let Some(u) = q.pop_front() {
        let hops = match dist.get(&u) {
            Some(&d) => d,
            None => 0,
        };
        if hops >= max_hops {
            continue;
        }
        let nxts = neighbors(g, u, type_id, dir);
        for &(eid, w) in nxts.iter() {
            if w == goal {
                let d = hops + 1;
                if d >= min_hops && d <= max_hops {
                    pred.insert(w, (u, eid));
                    found = true;
                    break;
                }
                continue;
            }
            if seen.contains(&w) {
                continue;
            }
            seen.insert(w);
            pred.insert(w, (u, eid));
            dist.insert(w, hops + 1);
            q.push_back(w);
        }
        if found {
            break;
        }
    }
    if !found {
        return None;
    }
    Some(walk_edges(&pred, start, goal))
}

fn neighbors(g: &Graph, u: Khid, type_id: Option<Khid>, dir: i32) -> Vec<(Khid, Khid)> {
    let v = match g.vertex(u) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let mut eids = Vec::new();
    if dir >= 0 {
        eids.extend(v.outgoing().iter().cloned());
    }
    if dir <= 0 {
        eids.extend(v.incoming().iter().cloned());
    }
    let mut out = Vec::new();
    for eid in eids.iter() {
        let e = match g.edge(*eid) {
            Some(e) => e,
            None => continue,
        };
        if let Some(tid) = type_id {
            if e.type_id() != Some(tid) {
                continue;
            }
        }
        let w = if e.source() == u {
            e.target()
        } else {
            e.source()
        };
        out.push((*eid, w));
    }
    out
}

fn walk_edges(pred: &HashMap<Khid, (Khid, Khid)>, start: Khid, goal: Khid) -> Vec<Khid> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut cur = goal;
    nodes.push(cur);
    while cur != start {
        match pred.get(&cur) {
            Some(pair) => {
                edges.push(pair.1);
                cur = pair.0;
                nodes.push(cur);
            }
            None => break,
        }
    }
    nodes.reverse();
    edges.reverse();
    let mut out = Vec::new();
    let mut i = 0;
    while i < edges.len() {
        out.push(nodes[i]);
        out.push(edges[i]);
        i += 1;
    }
    if !nodes.is_empty() {
        out.push(nodes[nodes.len() - 1]);
    }
    out
}

fn walk_pred(pred: &HashMap<Khid, Khid>, start: Khid, goal: Khid) -> Vec<Khid> {
    let mut chain = Vec::new();
    let mut cur = goal;
    chain.push(cur);
    while cur != start {
        match pred.get(&cur) {
            Some(&p) => {
                cur = p;
                chain.push(cur);
            }
            None => break,
        }
    }
    chain.reverse();
    chain
}

pub fn has_cycle(g: &Graph) -> bool {
    let mut color: HashMap<Khid, i32> = HashMap::new();
    for id in g.vertex_ids() {
        color.insert(id, 0);
    }
    let ids = g.vertex_ids();
    for id in ids.iter() {
        if color.get(id) == Some(&0) {
            if dfs_cycle(g, *id, &mut color) {
                return true;
            }
        }
    }
    false
}

fn dfs_cycle(g: &Graph, u: Khid, color: &mut HashMap<Khid, i32>) -> bool {
    color.insert(u, 1);
    let eids: Vec<Khid> = match g.vertex(u) {
        Some(v) => v.outgoing().iter().cloned().collect(),
        None => Vec::new(),
    };
    for eid in eids.iter() {
        if let Some(e) = g.edge(*eid) {
            let w = e.target();
            match color.get(&w).cloned() {
                Some(1) => return true,
                Some(0) => {
                    if dfs_cycle(g, w, color) {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    color.insert(u, 2);
    false
}

struct State {
    dist: i64,
    id: Khid,
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.dist == other.dist && self.id == other.id
    }
}
impl Eq for State {}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        other.dist.cmp(&self.dist)
    }
}

pub fn shortest(g: &Graph, start: Khid, goal: Khid) -> Option<Vec<Khid>> {
    let mut dist: HashMap<Khid, i64> = HashMap::new();
    let mut pred: HashMap<Khid, Khid> = HashMap::new();
    let mut heap = BinaryHeap::new();
    dist.insert(start, 0);
    heap.push(State {
        dist: 0,
        id: start,
    });
    while let Some(State { dist: d, id: u }) = heap.pop() {
        if u == goal {
            break;
        }
        if let Some(&best) = dist.get(&u) {
            if d > best {
                continue;
            }
        }
        let eids: Vec<Khid> = match g.vertex(u) {
            Some(v) => v.outgoing().iter().cloned().collect(),
            None => Vec::new(),
        };
        for eid in eids.iter() {
            if let Some(e) = g.edge(*eid) {
                let wgt = match e.get("weight") {
                    Some(s) => s.parse::<i64>().unwrap_or(1),
                    None => 1,
                };
                let nxt = e.target();
                let nd = d + wgt;
                let better = match dist.get(&nxt) {
                    Some(&old) => nd < old,
                    None => true,
                };
                if better {
                    dist.insert(nxt, nd);
                    pred.insert(nxt, u);
                    heap.push(State {
                        dist: nd,
                        id: nxt,
                    });
                }
            }
        }
    }
    if dist.get(&goal).is_none() {
        return None;
    }
    Some(walk_pred(&pred, start, goal))
}

/// Undirected components. Parent pointers live in scratch.
pub fn components(g: &Graph) -> Vec<Vec<Khid>> {
    let ids = g.vertex_ids();
    let mut parent: HashMap<Khid, Khid> = HashMap::new();
    for id in ids.iter() {
        parent.insert(*id, *id);
    }
    for id in ids.iter() {
        let eids: Vec<Khid> = match g.vertex(*id) {
            Some(v) => {
                let mut e: Vec<Khid> = v.outgoing().iter().cloned().collect();
                e.extend(v.incoming().iter().cloned());
                e
            }
            None => Vec::new(),
        };
        for eid in eids.iter() {
            if let Some(e) = g.edge(*eid) {
                let a = uf_find(&mut parent, e.source());
                let b = uf_find(&mut parent, e.target());
                if a != b {
                    parent.insert(a, b);
                }
            }
        }
    }
    let mut groups: HashMap<Khid, Vec<Khid>> = HashMap::new();
    for id in ids.iter() {
        let r = uf_find(&mut parent, *id);
        groups.entry(r).or_insert(Vec::new()).push(*id);
    }
    let mut out: Vec<Vec<Khid>> = Vec::new();
    for (_, v) in groups {
        out.push(v);
    }
    out
}

fn uf_find(parent: &mut HashMap<Khid, Khid>, x: Khid) -> Khid {
    let p = match parent.get(&x) {
        Some(&s) => s,
        None => x,
    };
    if p != x {
        let r = uf_find(parent, p);
        parent.insert(x, r);
        r
    } else {
        p
    }
}
