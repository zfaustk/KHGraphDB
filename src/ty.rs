use std::collections::HashSet;

use super::khid::Khid;

/// First-class type. Not a string label. Vertices and edges wear it.
#[derive(Clone)]
pub struct Type {
    id: Khid,
    name: String,
    vertices: HashSet<String>,
    edges: HashSet<String>,
}

impl Type {
    pub fn new(id: String, name: String) -> Type {
        let kid = Khid::parse(&id).unwrap_or(Khid::nil());
        Type::with_khid(kid, name)
    }

    pub fn with_khid(id: Khid, name: String) -> Type {
        Type {
            id: id,
            name: name,
            vertices: HashSet::new(),
            edges: HashSet::new(),
        }
    }

    pub fn khid(&self) -> Khid {
        self.id
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
