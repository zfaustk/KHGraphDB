use std::collections::HashMap;

use super::error::{Error, Result};
use super::graph::Graph;

/// Several arenas. MATCH still takes one Graph.
pub struct Catalog {
    graphs: HashMap<String, Graph>,
}

impl Catalog {
    pub fn new() -> Catalog {
        Catalog { graphs: HashMap::new() }
    }

    pub fn create(&mut self, name: &str) -> Result<&mut Graph> {
        if name.is_empty() {
            return Err(Error::new("empty graph name"));
        }
        if self.graphs.contains_key(name) {
            return Err(Error::new("graph exists"));
        }
        self.graphs.insert(name.to_string(), Graph::named(name));
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

    /// Take a graph that already has a KHID. Replaces
    /// a graph of the same name.
    pub fn put(&mut self, g: Graph) -> String {
        let name = g.khid().to_string();
        let name = if name.is_empty() {
            "g1".to_string()
        } else {
            name
        };
        self.graphs.insert(name.clone(), g);
        name
    }
}
