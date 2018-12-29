use std::collections::HashSet;

use super::khid::Khid;

/// First-class type. Not a string label. Vertices and edges wear it.
#[derive(Clone)]
pub struct Type {
    id: Khid,
    name: String,
    vertices: HashSet<Khid>,
    edges: HashSet<Khid>,
}

impl Type {
    pub fn new(id: Khid, name: String) -> Type {
        Type::with_khid(id, name)
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

    pub fn vertices(&self) -> &HashSet<Khid> {
        &self.vertices
    }

    pub fn edges(&self) -> &HashSet<Khid> {
        &self.edges
    }

    pub fn add_vertex(&mut self, vid: Khid) -> bool {
        self.vertices.insert(vid)
    }

    pub fn remove_vertex(&mut self, vid: Khid) -> bool {
        self.vertices.remove(&vid)
    }

    pub fn add_edge(&mut self, eid: Khid) -> bool {
        self.edges.insert(eid)
    }

    pub fn remove_edge(&mut self, eid: Khid) -> bool {
        self.edges.remove(&eid)
    }
}
