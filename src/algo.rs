use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use super::graph::Graph;

/// Scratch lives next to the walk, keyed by KHID.
/// Attributes stay clean. That is AlgorithmObjs, without
/// writing on the vertex.

pub fn nearby(g: &Graph, start: &str, depth: i32) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    let mut out = Vec::new();
    seen.insert(start.to_string());
    q.push_back((start.to_string(), 0i32));
    while let Some((u, d)) = q.pop_front() {
        if d > 0 {
            out.push(u.clone());
        }
        if d >= depth {
            continue;
        }
        let eids: Vec<String> = match g.vertex(&u) {
            Some(v) => v.outgoing().iter().map(|s| s.clone()).collect(),
            None => Vec::new(),
        };
        for eid in eids.iter() {
            if let Some(e) = g.edge(eid) {
                let w = e.target().to_string();
                if seen.insert(w.clone()) {
                    q.push_back((w, d + 1));
                }
            }
        }
    }
    out
}

pub fn path(g: &Graph, start: &str, goal: &str) -> Option<Vec<String>> {
    if start == goal {
        return Some(vec![start.to_string()]);
    }
    let mut pred: HashMap<String, String> = HashMap::new();
    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    seen.insert(start.to_string());
    q.push_back(start.to_string());
    let mut found = false;
    while let Some(u) = q.pop_front() {
        let eids: Vec<String> = match g.vertex(&u) {
            Some(v) => v.outgoing().iter().map(|s| s.clone()).collect(),
            None => Vec::new(),
        };
        for eid in eids.iter() {
            if let Some(e) = g.edge(eid) {
                let w = e.target().to_string();
                if seen.insert(w.clone()) {
                    pred.insert(w.clone(), u.clone());
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
               start: &str,
               goal: &str,
               type_id: Option<&str>,
               dir: i32,
               min_hops: usize,
               max_hops: usize)
               -> Option<Vec<String>> {
    if start == goal && min_hops == 0 {
        return Some(vec![start.to_string()]);
    }
    let mut pred: HashMap<String, (String, String)> = HashMap::new();
    let mut dist: HashMap<String, usize> = HashMap::new();
    let mut seen = HashSet::new();
    let mut q = VecDeque::new();
    seen.insert(start.to_string());
    dist.insert(start.to_string(), 0);
    q.push_back(start.to_string());
    let mut found = false;
    while let Some(u) = q.pop_front() {
        let hops = match dist.get(&u) {
            Some(&d) => d,
            None => 0,
        };
        if hops >= max_hops {
            continue;
        }
        let nxts = neighbors(g, &u, type_id, dir);
        for &(ref eid, ref w) in nxts.iter() {
            if w == goal {
                let d = hops + 1;
                if d >= min_hops && d <= max_hops {
                    pred.insert(w.clone(), (u.clone(), eid.clone()));
                    found = true;
                    break;
                }
                continue;
            }
            if seen.contains(w) {
                continue;
            }
            seen.insert(w.clone());
            pred.insert(w.clone(), (u.clone(), eid.clone()));
            dist.insert(w.clone(), hops + 1);
            q.push_back(w.clone());
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

fn neighbors(g: &Graph, u: &str, type_id: Option<&str>, dir: i32) -> Vec<(String, String)> {
    let v = match g.vertex(u) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let mut eids = Vec::new();
    if dir >= 0 {
        for e in v.outgoing().iter() {
            eids.push(e.clone());
        }
    }
    if dir <= 0 {
        for e in v.incoming().iter() {
            eids.push(e.clone());
        }
    }
    let mut out = Vec::new();
    for eid in eids.iter() {
        let e = match g.edge(eid) {
            Some(e) => e,
            None => continue,
        };
        if let Some(tid) = type_id {
            if e.type_id() != Some(tid) {
                continue;
            }
        }
        let w = if e.source() == u {
            e.target().to_string()
        } else {
            e.source().to_string()
        };
        out.push((eid.clone(), w));
    }
    out
}

fn walk_edges(pred: &HashMap<String, (String, String)>, start: &str, goal: &str) -> Vec<String> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut cur = goal.to_string();
    nodes.push(cur.clone());
    while cur != start {
        match pred.get(&cur) {
            Some(pair) => {
                edges.push(pair.1.clone());
                cur = pair.0.clone();
                nodes.push(cur.clone());
            }
            None => break,
        }
    }
    nodes.reverse();
    edges.reverse();
    let mut out = Vec::new();
    let mut i = 0;
    while i < edges.len() {
        out.push(nodes[i].clone());
        out.push(edges[i].clone());
        i += 1;
    }
    if !nodes.is_empty() {
        out.push(nodes[nodes.len() - 1].clone());
    }
    out
}

fn walk_pred(pred: &HashMap<String, String>, start: &str, goal: &str) -> Vec<String> {
    let mut chain = Vec::new();
    let mut cur = goal.to_string();
    chain.push(cur.clone());
    while cur != start {
        match pred.get(&cur) {
            Some(p) => {
                cur = p.clone();
                chain.push(cur.clone());
            }
            None => break,
        }
    }
    chain.reverse();
    chain
}

pub fn has_cycle(g: &Graph) -> bool {
    let mut color: HashMap<String, i32> = HashMap::new();
    for id in g.vertex_ids() {
        color.insert(id, 0);
    }
    let ids = g.vertex_ids();
    for id in ids.iter() {
        if color.get(id) == Some(&0) {
            if dfs_cycle(g, id, &mut color) {
                return true;
            }
        }
    }
    false
}

fn dfs_cycle(g: &Graph, u: &str, color: &mut HashMap<String, i32>) -> bool {
    color.insert(u.to_string(), 1);
    let eids: Vec<String> = match g.vertex(u) {
        Some(v) => v.outgoing().iter().map(|s| s.clone()).collect(),
        None => Vec::new(),
    };
    for eid in eids.iter() {
        if let Some(e) = g.edge(eid) {
            let w = e.target().to_string();
            match color.get(&w).cloned() {
                Some(1) => return true,
                Some(0) => {
                    if dfs_cycle(g, &w, color) {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    color.insert(u.to_string(), 2);
    false
}

struct State {
    dist: i64,
    id: String,
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

pub fn shortest(g: &Graph, start: &str, goal: &str) -> Option<Vec<String>> {
    let mut dist: HashMap<String, i64> = HashMap::new();
    let mut pred: HashMap<String, String> = HashMap::new();
    let mut heap = BinaryHeap::new();
    dist.insert(start.to_string(), 0);
    heap.push(State {
        dist: 0,
        id: start.to_string(),
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
        let eids: Vec<String> = match g.vertex(&u) {
            Some(v) => v.outgoing().iter().map(|s| s.clone()).collect(),
            None => Vec::new(),
        };
        for eid in eids.iter() {
            if let Some(e) = g.edge(eid) {
                let wgt = match e.get("weight") {
                    Some(s) => s.parse::<i64>().unwrap_or(1),
                    None => 1,
                };
                let nxt = e.target().to_string();
                let nd = d + wgt;
                let better = match dist.get(&nxt) {
                    Some(&old) => nd < old,
                    None => true,
                };
                if better {
                    dist.insert(nxt.clone(), nd);
                    pred.insert(nxt.clone(), u.clone());
                    heap.push(State {
                        dist: nd,
                        id: nxt,
                    });
                }
            }
        }
    }
    if dist.get(goal).is_none() {
        return None;
    }
    Some(walk_pred(&pred, start, goal))
}

/// Undirected components. Parent pointers live in scratch.
pub fn components(g: &Graph) -> Vec<Vec<String>> {
    let ids = g.vertex_ids();
    let mut parent: HashMap<String, String> = HashMap::new();
    for id in ids.iter() {
        parent.insert(id.clone(), id.clone());
    }
    for id in ids.iter() {
        let eids: Vec<String> = match g.vertex(id) {
            Some(v) => {
                let mut e = v.outgoing().iter().map(|s| s.clone()).collect::<Vec<_>>();
                e.extend(v.incoming().iter().map(|s| s.clone()));
                e
            }
            None => Vec::new(),
        };
        for eid in eids.iter() {
            if let Some(e) = g.edge(eid) {
                let a = uf_find(&mut parent, e.source());
                let b = uf_find(&mut parent, e.target());
                if a != b {
                    parent.insert(a, b);
                }
            }
        }
    }
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for id in ids.iter() {
        let r = uf_find(&mut parent, id);
        groups.entry(r).or_insert(Vec::new()).push(id.clone());
    }
    let mut out: Vec<Vec<String>> = Vec::new();
    for (_, v) in groups {
        out.push(v);
    }
    out
}

fn uf_find(parent: &mut HashMap<String, String>, x: &str) -> String {
    let p = match parent.get(x) {
        Some(s) => s.clone(),
        None => x.to_string(),
    };
    if p != x {
        let r = uf_find(parent, &p);
        parent.insert(x.to_string(), r.clone());
        r
    } else {
        p
    }
}
