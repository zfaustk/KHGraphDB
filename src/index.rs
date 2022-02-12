use std::collections::{BTreeMap, HashSet};
use std::ops::Bound::{Excluded, Included, Unbounded};

use super::khid::Khid;
use super::prop::Prop;

/// Ordered posting for (Type, key) → Prop → vertex ids.
/// A B-tree, not a hash: comparison is a range.
/// There is no pager. The arena is the pool.
#[derive(Clone)]
pub struct SchemaIndex {
    type_name: String,
    key: String,
    unique: bool,
    posting: BTreeMap<Prop, HashSet<Khid>>,
}

impl SchemaIndex {
    pub fn new(type_name: &str, key: &str, unique: bool) -> SchemaIndex {
        SchemaIndex {
            type_name: type_name.to_string(),
            key: key.to_string(),
            unique: unique,
            posting: BTreeMap::new(),
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

    /// Inclusive/exclusive range on the ordered keys.
    pub fn range(&self,
                 lo: Option<&Prop>,
                 hi: Option<&Prop>,
                 lo_inc: bool,
                 hi_inc: bool)
                 -> Vec<Khid> {
        let start = match lo {
            Some(p) if lo_inc => Included(p),
            Some(p) => Excluded(p),
            None => Unbounded,
        };
        let end = match hi {
            Some(p) if hi_inc => Included(p),
            Some(p) => Excluded(p),
            None => Unbounded,
        };
        let mut v = Vec::new();
        for (_, set) in self.posting.range((start, end)) {
            v.extend(set.iter().cloned());
        }
        v
    }

    pub fn len(&self) -> usize {
        let mut n = 0;
        for set in self.posting.values() {
            n += set.len();
        }
        n
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
