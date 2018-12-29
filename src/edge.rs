use std::collections::HashMap;

use super::khid::Khid;
use super::prop::Prop;

#[derive(Clone)]
pub struct Edge {
    id: Khid,
    source: Khid,
    target: Khid,
    type_id: Option<Khid>,
    attrs: HashMap<String, Prop>,
}

impl Edge {
    pub fn new(id: Khid, source: Khid, target: Khid, attrs: HashMap<String, String>) -> Edge {
        let mut p = HashMap::new();
        for (k, v) in attrs.into_iter() {
            p.insert(k, Prop::from_str(&v));
        }
        Edge::with_props(id, source, target, p)
    }

    pub fn with_props(id: Khid,
                      source: Khid,
                      target: Khid,
                      attrs: HashMap<String, Prop>)
                      -> Edge {
        Edge {
            id: id,
            source: source,
            target: target,
            type_id: None,
            attrs: attrs,
        }
    }

    pub fn khid(&self) -> Khid {
        self.id
    }

    pub fn source(&self) -> Khid {
        self.source
    }

    pub fn target(&self) -> Khid {
        self.target
    }

    pub fn type_id(&self) -> Option<Khid> {
        self.type_id
    }

    pub fn set_type(&mut self, type_id: Khid) {
        self.type_id = Some(type_id);
    }

    pub fn clear_type(&mut self) {
        self.type_id = None;
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.attrs.get(key).and_then(|p| p.as_str())
    }

    pub fn get_prop(&self, key: &str) -> Option<&Prop> {
        self.attrs.get(key)
    }

    pub fn attrs(&self) -> &HashMap<String, Prop> {
        &self.attrs
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
}
