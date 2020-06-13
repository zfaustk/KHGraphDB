use std::collections::{HashMap, HashSet};

use super::khid::Khid;
use super::prop::Prop;

/// Posting list for (Type, key) -> Prop -> vertex ids.
#[derive(Clone)]
pub struct SchemaIndex {
    type_name: String,
    key: String,
    unique: bool,
    posting: HashMap<Prop, HashSet<Khid>>,
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

    pub fn add(&mut self, vid: Khid, value: &Prop) {
        if let Prop::Str(ref s) = *value {
            if s.is_empty() {
                return;
            }
        }
        self.posting.entry(value.clone()).or_insert(HashSet::new()).insert(vid);
    }

    pub fn remove(&mut self, vid: Khid, value: &Prop) {
        let drop_key = match self.posting.get_mut(value) {
            Some(set) => {
                set.remove(&vid);
                set.is_empty()
            }
            None => false,
        };
        if drop_key {
            self.posting.remove(value);
        }
    }

    pub fn get(&self, value: &Prop) -> Vec<Khid> {
        match self.posting.get(value) {
            Some(set) => set.iter().cloned().collect(),
            None => Vec::new(),
        }
    }

    /// All postings. Meta rebuilds from this.
    pub fn entries(&self) -> Vec<(Prop, Vec<Khid>)> {
        let mut v = Vec::new();
        for (p, set) in self.posting.iter() {
            v.push((p.clone(), set.iter().cloned().collect()));
        }
        v
    }

    /// True when some other KHID already posts this value.
    /// Pass nil on insert: any posting is a duplicate.
    pub fn contains_other(&self, value: &Prop, self_id: Khid) -> bool {
        match self.posting.get(value) {
            Some(set) => set.iter().any(|id| *id != self_id),
            None => false,
        }
    }

    pub fn id(type_name: &str, key: &str) -> String {
        format!("{}\x1f{}", type_name, key)
    }
}
