use std::collections::HashMap;

use super::error::{Error, Result};
use super::graph::Graph;
use super::addr::Addr;
use super::stub::Stub;

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

    pub fn by_shard(&self, shard: u32) -> Option<&Graph> {
        for g in self.graphs.values() {
            if g.shard() == shard {
                return Some(g);
            }
        }
        None
    }

    pub fn by_shard_mut(&mut self, shard: u32) -> Option<&mut Graph> {
        for g in self.graphs.values_mut() {
            if g.shard() == shard {
                return Some(g);
            }
        }
        None
    }

    /// Title at home. The page stays there.
    pub fn hydrate(&self, addr: Addr) -> Option<Stub> {
        let g = match self.by_shard(addr.shard()) {
            Some(g) => g,
            None => return None,
        };
        let v = match g.vertex(addr.khid()) {
            Some(v) => v,
            None => return None,
        };
        let title = match v.get("title").or(v.get("name")) {
            Some(s) => s,
            None => "",
        };
        Some(Stub::new(title, 1))
    }

    /// Copy a far title onto `home`. One round, this process.
    pub fn fill_stub(&mut self, home: u32, addr: Addr) -> bool {
        let title = match self.hydrate(addr) {
            Some(s) => s.title().to_string(),
            None => return false,
        };
        match self.by_shard_mut(home) {
            Some(g) => {
                g.put_stub(addr, &title, 1);
                true
            }
            None => false,
        }
    }

    /// One round. Titles at home, stubs on `home`.
    pub fn fill_round(&mut self, home: u32, addrs: &[Addr]) -> usize {
        let mut got: Vec<(Addr, Stub)> = Vec::new();
        for a in addrs.iter() {
            if let Some(s) = self.hydrate(*a) {
                got.push((*a, s));
            }
        }
        let n = got.len();
        match self.by_shard_mut(home) {
            Some(g) => {
                for &(a, ref s) in got.iter() {
                    g.put_stub(a, s.title(), s.ver());
                }
            }
            None => return 0,
        }
        n
    }
}
