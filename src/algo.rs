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
        if let Some(v) = g.vertex(&u) {
            for eid in v.outgoing() {
                if let Some(e) = g.edge(eid) {
                    let w = e.target().to_string();
                    if seen.insert(w.clone()) {
                        q.push_back((w, d + 1));
                    }
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
        if let Some(v) = g.vertex(&u) {
            for eid in v.outgoing() {
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
    if let Some(v) = g.vertex(u) {
        for eid in v.outgoing() {
            if let Some(e) = g.edge(eid) {
                let w = e.target();
                match color.get(w).cloned() {
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
        if let Some(v) = g.vertex(&u) {
            for eid in v.outgoing() {
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
    }
    if dist.get(goal).is_none() {
        return None;
    }
    Some(walk_pred(&pred, start, goal))
}
