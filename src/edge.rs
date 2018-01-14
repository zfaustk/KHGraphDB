use std::collections::HashMap;

use super::prop::Prop;

#[derive(Clone)]
pub struct Edge {
    id: String,
    source: String,
    target: String,
    type_id: Option<String>,
    attrs: HashMap<String, Prop>,
}

impl Edge {
    pub fn new(id: String, source: String, target: String, attrs: HashMap<String, String>) -> Edge {
        let mut p = HashMap::new();
        for (k, v) in attrs.into_iter() {
            p.insert(k, Prop::from_str(&v));
        }
        Edge::with_props(id, source, target, p)
    }

    pub fn with_props(id: String,
                      source: String,
                      target: String,
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

    pub fn khid(&self) -> &str {
        &self.id
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn type_id(&self) -> Option<&str> {
        self.type_id.as_ref().map(|s| &s[..])
    }

    pub fn set_type(&mut self, type_id: &str) {
        self.type_id = Some(type_id.to_string());
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
