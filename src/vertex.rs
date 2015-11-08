use std::collections::HashMap;

/// A node. KHID is identity. `types` is every type the vertex wears.
/// The first type is the primary, same as C# `Type`.
pub struct Vertex {
    id: String,
    attrs: HashMap<String, String>,
    types: Vec<String>,
    outgoing: Vec<String>,
    incoming: Vec<String>,
    algo: HashMap<String, i64>,
}

impl Vertex {
    pub fn new(id: String, attrs: HashMap<String, String>) -> Vertex {
        Vertex {
            id: id,
            attrs: attrs,
            types: Vec::new(),
            outgoing: Vec::with_capacity(4),
            incoming: Vec::with_capacity(4),
            algo: HashMap::new(),
        }
    }

    pub fn khid(&self) -> &str {
        &self.id
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).map(|s| &s[..])
    }

    pub fn attrs(&self) -> &HashMap<String, String> {
        &self.attrs
    }

    pub fn types(&self) -> &[String] {
        &self.types
    }

    pub fn primary_type(&self) -> Option<&str> {
        if self.types.is_empty() {
            None
        } else {
            Some(&self.types[0])
        }
    }

    pub fn has_type_name(&self, types: &HashMap<String, String>, name: &str) -> bool {
        for tid in self.types.iter() {
            if let Some(n) = types.get(tid) {
                if n == name {
                    return true;
                }
            }
        }
        false
    }

    pub fn out_degree(&self) -> usize {
        self.outgoing.len()
    }

    pub fn in_degree(&self) -> usize {
        self.incoming.len()
    }

    pub fn outgoing(&self) -> &[String] {
        &self.outgoing
    }

    pub fn incoming(&self) -> &[String] {
        &self.incoming
    }

    pub fn get_algo(&self, key: &str) -> Option<i64> {
        self.algo.get(key).map(|v| *v)
    }

    pub fn set_attr(&mut self, key: &str, value: &str) {
        self.attrs.insert(key.to_string(), value.to_string());
    }

    pub fn remove_attr(&mut self, key: &str) -> Option<String> {
        self.attrs.remove(key)
    }

    pub fn attach_type(&mut self, type_id: &str) -> bool {
        if self.types.iter().any(|t| t == type_id) {
            return false;
        }
        self.types.push(type_id.to_string());
        true
    }

    pub fn detach_type(&mut self, type_id: &str) -> bool {
        let before = self.types.len();
        self.types.retain(|t| t != type_id);
        before != self.types.len()
    }

    pub fn add_out(&mut self, eid: &str) -> bool {
        if self.outgoing.iter().any(|e| e == eid) {
            return false;
        }
        self.outgoing.push(eid.to_string());
        true
    }

    pub fn add_in(&mut self, eid: &str) -> bool {
        if self.incoming.iter().any(|e| e == eid) {
            return false;
        }
        self.incoming.push(eid.to_string());
        true
    }

    pub fn remove_out(&mut self, eid: &str) {
        self.outgoing.retain(|e| e != eid);
    }

    pub fn remove_in(&mut self, eid: &str) {
        self.incoming.retain(|e| e != eid);
    }

    pub fn set_algo(&mut self, key: &str, val: i64) {
        self.algo.insert(key.to_string(), val);
    }

    pub fn clear_algo(&mut self) {
        self.algo.clear();
    }
}
