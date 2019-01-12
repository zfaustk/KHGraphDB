use std::collections::HashMap;

use super::error::{Error, Result};
use super::graph::Graph;

/// Several arenas. MATCH still takes one Graph.
/// Each graph is a shard. This process may hold
/// several; they do not talk yet.
pub struct Catalog {
    graphs: HashMap<String, Graph>,
    next_shard: u32,
}

impl Catalog {
    pub fn new() -> Catalog {
        Catalog {
            graphs: HashMap::new(),
            next_shard: 1,
        }
    }

    fn alloc_shard(&mut self) -> u32 {
        let s = self.next_shard;
        self.next_shard += 1;
        s
    }

    pub fn create(&mut self, name: &str) -> Result<&mut Graph> {
        if name.is_empty() {
            return Err(Error::new("empty graph name"));
        }
        if self.graphs.contains_key(name) {
            return Err(Error::new("graph exists"));
        }
        let shard = self.alloc_shard();
        self.graphs.insert(name.to_string(), Graph::on(name, shard));
        match self.graphs.get_mut(name) {
            Some(g) => Ok(g),
            None => Err(Error::new("graph exists")),
        }
    }

    pub fn graph(&self, name: &str) -> Option<&Graph> {
        self.graphs.get(name)
    }

    pub fn graph_mut(&mut self, name: &str) -> Option<&mut Graph> {
        self.graphs.get_mut(name)
    }

    pub fn names(&self) -> Vec<String> {
        self.graphs.keys().map(|s| s.clone()).collect()
    }

    pub fn drop(&mut self, name: &str) -> bool {
        self.graphs.remove(name).is_some()
    }

    /// Take a graph that already has a name. Replaces
    /// a graph of the same name. Keeps its shard, or
    /// assigns one if it has none.
    pub fn put(&mut self, mut g: Graph) -> String {
        let name = g.khid().to_string();
        let name = if name.is_empty() {
            "g1".to_string()
        } else {
            name
        };
        if g.shard() == 0 {
            g.set_shard(self.alloc_shard());
        } else if g.shard() >= self.next_shard {
            self.next_shard = g.shard() + 1;
        }
        self.graphs.insert(name.clone(), g);
        name
    }
}
