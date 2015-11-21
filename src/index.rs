use std::collections::{HashMap, HashSet};

/// Posting list for (Type, key) -> value -> vertex ids.
pub struct SchemaIndex {
    type_name: String,
    key: String,
    unique: bool,
    posting: HashMap<String, HashSet<String>>,
}

impl SchemaIndex {
    pub fn new(type_name: &str, key: &str, unique: bool) -> SchemaIndex {
        SchemaIndex {
            type_name: type_name.to_string(),
            key: key.to_string(),
            unique: unique,
            posting: HashMap::new(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn unique(&self) -> bool {
        self.unique
    }

    pub fn set_unique(&mut self) {
        self.unique = true;
    }

    pub fn add(&mut self, vid: &str, value: &str) {
        if value.is_empty() {
            return;
        }
        self.posting.entry(value.to_string()).or_insert(HashSet::new()).insert(vid.to_string());
    }

    pub fn remove(&mut self, vid: &str, value: &str) {
        let drop_key = match self.posting.get_mut(value) {
            Some(set) => {
                set.remove(vid);
                set.is_empty()
            }
            None => false,
        };
        if drop_key {
            self.posting.remove(value);
        }
    }

    pub fn get(&self, value: &str) -> Vec<String> {
        match self.posting.get(value) {
            Some(set) => set.iter().map(|s| s.clone()).collect(),
            None => Vec::new(),
        }
    }

    pub fn contains_other(&self, value: &str, self_id: &str) -> bool {
        match self.posting.get(value) {
            Some(set) => set.iter().any(|id| id != self_id),
            None => false,
        }
    }

    pub fn id(type_name: &str, key: &str) -> String {
        format!("{}\x1f{}", type_name, key)
    }
}
