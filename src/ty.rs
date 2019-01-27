use std::collections::HashSet;

use super::khid::Khid;

/// First-class type. Not a string label. Vertices and edges wear it.
#[derive(Clone)]
pub struct Type {
    id: Khid,
    name: String,
    vertices: HashSet<Khid>,
    edges: HashSet<Khid>,
    content: HashSet<String>,
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
            content: HashSet::new(),
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

    /// A content key is payload. The index refuses it.
    pub fn mark_content(&mut self, key: &str) -> bool {
        if key.is_empty() {
            return false;
        }
        self.content.insert(key.to_string())
    }

    pub fn is_content(&self, key: &str) -> bool {
        self.content.contains(key)
    }

    pub fn content_keys(&self) -> &HashSet<String> {
        &self.content
    }
}
