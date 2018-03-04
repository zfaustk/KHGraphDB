use std::collections::HashMap;

use super::khid::Khid;
use super::prop::Prop;

/// A node. KHID is identity. `types` is every type the vertex wears.
/// The first type is the primary, same as C# `Type`.
/// Paint for a walk does not live here.
/// Attributes are Prop. C# stored object.
#[derive(Clone)]
pub struct Vertex {
    id: Khid,
    attrs: HashMap<String, Prop>,
    types: Vec<String>,
    outgoing: Vec<String>,
    incoming: Vec<String>,
}

impl Vertex {
    pub fn new(id: String, attrs: HashMap<String, String>) -> Vertex {
        let mut p = HashMap::new();
        for (k, v) in attrs.into_iter() {
            p.insert(k, Prop::from_str(&v));
        }
        let kid = Khid::parse(&id).unwrap_or(Khid::nil());
        Vertex::with_props(kid, p)
    }

    pub fn with_props(id: Khid, attrs: HashMap<String, Prop>) -> Vertex {
        Vertex {
            id: id,
            attrs: attrs,
            types: Vec::new(),
            outgoing: Vec::with_capacity(4),
            incoming: Vec::with_capacity(4),
        }
    }

    pub fn khid(&self) -> Khid {
        self.id
    }

    /// String view. Only Str properties. Int 1 is not here.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).and_then(|p| p.as_str())
    }

    pub fn get_prop(&self, key: &str) -> Option<&Prop> {
        self.attrs.get(key)
    }

    pub fn attrs(&self) -> &HashMap<String, Prop> {
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

    pub fn set_attr(&mut self, key: &str, value: &str) {
        self.attrs.insert(key.to_string(), Prop::from_str(value));
    }

    pub fn set_prop(&mut self, key: &str, value: Prop) {
        self.attrs.insert(key.to_string(), value);
    }

    pub fn remove_attr(&mut self, key: &str) -> Option<Prop> {
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
}
