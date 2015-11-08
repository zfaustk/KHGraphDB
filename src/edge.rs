use std::collections::HashMap;

pub struct Edge {
    id: String,
    source: String,
    target: String,
    type_id: Option<String>,
    attrs: HashMap<String, String>,
}

impl Edge {
    pub fn new(id: String, source: String, target: String, attrs: HashMap<String, String>) -> Edge {
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
        self.attrs.get(key).map(|s| &s[..])
    }

    pub fn attrs(&self) -> &HashMap<String, String> {
        &self.attrs
    }
}
