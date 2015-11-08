use std::collections::{HashMap, HashSet};

/// First-class type. Not a string label. Vertices and edges wear it.
pub struct Type {
    id: String,
    name: String,
    vertices: HashSet<String>,
    edges: HashSet<String>,
    attrs: HashMap<String, String>,
}

impl Type {
    pub fn new(id: String, name: String) -> Type {
        Type {
            id: id,
            name: name,
            vertices: HashSet::new(),
            edges: HashSet::new(),
            attrs: HashMap::new(),
        }
    }

    pub fn khid(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn vertices(&self) -> &HashSet<String> {
        &self.vertices
    }

    pub fn edges(&self) -> &HashSet<String> {
        &self.edges
    }

    pub fn add_vertex(&mut self, vid: &str) -> bool {
        self.vertices.insert(vid.to_string())
    }

    pub fn remove_vertex(&mut self, vid: &str) -> bool {
        self.vertices.remove(vid)
    }

    pub fn add_edge(&mut self, eid: &str) -> bool {
        self.edges.insert(eid.to_string())
    }

    pub fn remove_edge(&mut self, eid: &str) -> bool {
        self.edges.remove(eid)
    }
}
