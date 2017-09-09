use std::collections::HashMap;

use super::error::{Error, Result};
use super::vertex::Vertex;
use super::edge::Edge;
use super::ty::Type;
use super::index::SchemaIndex;

/// Directed property graph. Lookups are HashMaps. KHID is identity.
#[derive(Clone)]
pub struct Graph {
    id: String,
    serial: u64,
    vertices: HashMap<String, Vertex>,
    edges: HashMap<String, Edge>,
    types: HashMap<String, Type>,
    types_by_name: HashMap<String, String>,
    vertices_by_name: HashMap<String, String>,
    indexes: HashMap<String, SchemaIndex>,
    edge_indexes: HashMap<String, SchemaIndex>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph::named("g1")
    }

    pub fn named(id: &str) -> Graph {
        Graph {
            id: id.to_string(),
            serial: 0,
            vertices: HashMap::new(),
            edges: HashMap::new(),
            types: HashMap::new(),
            types_by_name: HashMap::new(),
            vertices_by_name: HashMap::new(),
            indexes: HashMap::new(),
            edge_indexes: HashMap::new(),
        }
    }

    pub fn khid(&self) -> &str {
        &self.id
    }

    pub fn clear(&mut self) {
        self.serial = 0;
        self.vertices.clear();
        self.edges.clear();
        self.types.clear();
        self.types_by_name.clear();
        self.vertices_by_name.clear();
        self.indexes.clear();
        self.edge_indexes.clear();
    }

    pub fn subgraph(&self, vids: &[String]) -> Graph {
        let mut g = self.clone();
        let ids = g.vertex_ids();
        for id in ids.iter() {
            let mut keep = false;
            for v in vids.iter() {
                if v == id {
                    keep = true;
                    break;
                }
            }
            if !keep {
                g.remove_vertex(id);
            }
        }
        g
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn type_count(&self) -> usize {
        self.types.len()
    }

    fn next_id(&mut self) -> String {
        self.serial += 1;
        format!("k{:x}", self.serial)
    }

    fn note_id(&mut self, id: &str) {
        if id.len() > 1 && id.as_bytes()[0] == b'k' {
            match u64::from_str_radix(&id[1..], 16) {
                Ok(n) => {
                    if n > self.serial {
                        self.serial = n;
                    }
                }
                Err(_) => {}
            }
        }
    }

    pub fn vertex(&self, khid: &str) -> Option<&Vertex> {
        self.vertices.get(khid)
    }

    pub fn vertex_mut(&mut self, khid: &str) -> Option<&mut Vertex> {
        self.vertices.get_mut(khid)
    }

    pub fn edge(&self, khid: &str) -> Option<&Edge> {
        self.edges.get(khid)
    }

    pub fn ty(&self, khid: &str) -> Option<&Type> {
        self.types.get(khid)
    }

    pub fn type_by_name(&self, name: &str) -> Option<&Type> {
        match self.types_by_name.get(name) {
            Some(id) => self.types.get(id),
            None => None,
        }
    }

    pub fn vertex_by_name(&self, name: &str) -> Option<&Vertex> {
        match self.vertices_by_name.get(name) {
            Some(id) => self.vertices.get(id),
            None => None,
        }
    }

    pub fn add_type(&mut self, name: &str) -> Result<String> {
        if name.is_empty() {
            return Err(Error::new("empty type name"));
        }
        if let Some(id) = self.types_by_name.get(name) {
            return Ok(id.clone());
        }
        let id = self.next_id();
        let t = Type::new(id.clone(), name.to_string());
        self.types_by_name.insert(name.to_string(), id.clone());
        self.types.insert(id.clone(), t);
        Ok(id)
    }

    pub fn add_vertex(&mut self,
                      attrs: HashMap<String, String>,
                      type_name: Option<&str>)
                      -> Result<String> {
        if let Some(tn) = type_name {
            for (k, val) in attrs.iter() {
                let iid = SchemaIndex::id(tn, k);
                if let Some(idx) = self.indexes.get(&iid) {
                    if idx.unique() && idx.contains_other(val, "") {
                        return Err(Error::new("unique constraint"));
                    }
                }
            }
        }
        let id = self.next_id();
        let mut v = Vertex::new(id.clone(), attrs);
        if let Some(name) = v.get("name") {
            if !self.vertices_by_name.contains_key(name) {
                self.vertices_by_name.insert(name.to_string(), id.clone());
            }
        }
        if let Some(tn) = type_name {
            let tid = self.add_type(tn)?;
            v.attach_type(&tid);
            match self.types.get_mut(&tid) {
                Some(t) => {
                    t.add_vertex(&id);
                }
                None => {}
            }
            let keys: Vec<(String, String)> = v.attrs()
                .iter()
                .map(|(k, val)| (k.clone(), val.clone()))
                .collect();
            for (k, val) in keys.iter() {
                let iid = SchemaIndex::id(tn, k);
                if let Some(idx) = self.indexes.get_mut(&iid) {
                    idx.add(&id, val);
                }
            }
        }
        self.vertices.insert(id.clone(), v);
        Ok(id)
    }

    pub fn add_edge(&mut self,
                    src: &str,
                    dst: &str,
                    type_name: Option<&str>)
                    -> Result<String> {
        if !self.vertices.contains_key(src) || !self.vertices.contains_key(dst) {
            return Err(Error::new("missing vertex"));
        }
        let id = self.next_id();
        let mut e = Edge::new(id.clone(), src.to_string(), dst.to_string(), HashMap::new());
        if let Some(tn) = type_name {
            let tid = self.add_type(tn)?;
            e.set_type(&tid);
            if let Some(t) = self.types.get_mut(&tid) {
                t.add_edge(&id);
            }
        }
        {
            let srcv = self.vertices.get_mut(src).unwrap();
            srcv.add_out(&id);
        }
        {
            let dstv = self.vertices.get_mut(dst).unwrap();
            dstv.add_in(&id);
        }
        self.edges.insert(id.clone(), e);
        Ok(id)
    }

    pub fn remove_edge(&mut self, eid: &str) -> bool {
        let (src, dst, tid) = match self.edges.get(eid) {
            Some(e) => (e.source().to_string(), e.target().to_string(), e.type_id().map(|s| s.to_string())),
            None => return false,
        };
        if let Some(v) = self.vertices.get_mut(&src) {
            v.remove_out(eid);
        }
        if let Some(v) = self.vertices.get_mut(&dst) {
            v.remove_in(eid);
        }
        if let Some(t) = tid {
            if let Some(ty) = self.types.get_mut(&t) {
                ty.remove_edge(eid);
            }
        }
        self.edges.remove(eid).is_some()
    }

    pub fn remove_vertex(&mut self, vid: &str) -> bool {
        let (outs, ins, tps, name) = match self.vertices.get(vid) {
            Some(v) => {
                let o: Vec<String> = v.outgoing().iter().map(|s| s.clone()).collect();
                let i: Vec<String> = v.incoming().iter().map(|s| s.clone()).collect();
                let t: Vec<String> = v.types().iter().map(|s| s.clone()).collect();
                let n = v.get("name").map(|s| s.to_string());
                (o, i, t, n)
            }
            None => return false,
        };
        for e in outs.iter() {
            self.remove_edge(e);
        }
        for e in ins.iter() {
            self.remove_edge(e);
        }
        for t in tps.iter() {
            if let Some(ty) = self.types.get_mut(t) {
                ty.remove_vertex(vid);
            }
        }
        if let Some(n) = name {
            if let Some(owned) = self.vertices_by_name.get(&n).cloned() {
                if owned == vid {
                    self.vertices_by_name.remove(&n);
                }
            }
        }
        self.vertices.remove(vid).is_some()
    }

    pub fn add_type_to_vertex(&mut self, vid: &str, type_name: &str) -> Result<bool> {
        let tid = self.add_type(type_name)?;
        {
            let v = match self.vertices.get_mut(vid) {
                Some(v) => v,
                None => return Err(Error::new("missing vertex")),
            };
            if !v.attach_type(&tid) {
                return Ok(false);
            }
        }
        if let Some(t) = self.types.get_mut(&tid) {
            t.add_vertex(vid);
        }
        Ok(true)
    }

    pub fn has_type(&self, vid: &str, type_name: &str) -> bool {
        let v = match self.vertices.get(vid) {
            Some(v) => v,
            None => return false,
        };
        for tid in v.types().iter() {
            if let Some(t) = self.types.get(tid) {
                if t.name() == type_name {
                    return true;
                }
            }
        }
        false
    }

    pub fn vertices_of_type(&self, type_name: &str) -> Vec<String> {
        match self.type_by_name(type_name) {
            Some(t) => t.vertices().iter().map(|s| s.clone()).collect(),
            None => Vec::new(),
        }
    }

    pub fn edges_of_type(&self, type_name: &str) -> Vec<String> {
        match self.type_by_name(type_name) {
            Some(t) => t.edges().iter().map(|s| s.clone()).collect(),
            None => Vec::new(),
        }
    }

    pub fn vertex_ids(&self) -> Vec<String> {
        self.vertices.keys().map(|k| k.clone()).collect()
    }

    pub fn type_name_of(&self, tid: &str) -> Option<&str> {
        self.types.get(tid).map(|t| t.name())
    }

    pub fn create_index(&mut self, type_name: &str, key: &str) -> bool {
        self.create_index_inner(type_name, key, false)
    }

    pub fn create_unique(&mut self, type_name: &str, key: &str) -> bool {
        self.create_index_inner(type_name, key, true)
    }

    fn create_index_inner(&mut self, type_name: &str, key: &str, unique: bool) -> bool {
        if type_name.is_empty() || key.is_empty() {
            return false;
        }
        let id = SchemaIndex::id(type_name, key);
        if let Some(idx) = self.indexes.get_mut(&id) {
            if unique {
                idx.set_unique();
            }
            return true;
        }
        let mut idx = SchemaIndex::new(type_name, key, unique);
        let vids = self.vertices_of_type(type_name);
        for vid in vids.iter() {
            let val = match self.vertices.get(vid) {
                Some(v) => v.get(key).unwrap_or("").to_string(),
                None => String::new(),
            };
            if unique && idx.contains_other(&val, vid) {
                return false;
            }
            idx.add(vid, &val);
        }
        self.indexes.insert(id, idx);
        true
    }

    pub fn create_edge_index(&mut self, type_name: &str, key: &str) -> bool {
        if type_name.is_empty() || key.is_empty() {
            return false;
        }
        let id = SchemaIndex::id(type_name, key);
        if self.edge_indexes.contains_key(&id) {
            return true;
        }
        let mut idx = SchemaIndex::new(type_name, key, false);
        let eids = self.edges_of_type(type_name);
        for eid in eids.iter() {
            let val = match self.edges.get(eid) {
                Some(e) => e.get(key).unwrap_or("").to_string(),
                None => String::new(),
            };
            idx.add(eid, &val);
        }
        self.edge_indexes.insert(id, idx);
        true
    }

    pub fn find_edge(&self, type_name: &str, key: &str, value: &str) -> Vec<String> {
        let id = SchemaIndex::id(type_name, key);
        if let Some(idx) = self.edge_indexes.get(&id) {
            return idx.get(value);
        }
        let mut hits = Vec::new();
        for eid in self.edges_of_type(type_name).iter() {
            if let Some(e) = self.edges.get(eid) {
                if e.get(key) == Some(value) {
                    hits.push(eid.clone());
                }
            }
        }
        hits
    }

    pub fn find(&self, type_name: &str, key: &str, value: &str) -> Vec<String> {
        let id = SchemaIndex::id(type_name, key);
        if let Some(idx) = self.indexes.get(&id) {
            return idx.get(value);
        }
        let mut hits = Vec::new();
        for vid in self.vertices_of_type(type_name).iter() {
            if let Some(v) = self.vertices.get(vid) {
                if v.get(key) == Some(value) {
                    hits.push(vid.clone());
                }
            }
        }
        hits
    }

    pub fn set_attr(&mut self, vid: &str, key: &str, value: &str) -> Result<()> {
        let types: Vec<String> = match self.vertices.get(vid) {
            Some(v) => v.types().iter().map(|s| s.clone()).collect(),
            None => return Err(Error::new("missing vertex")),
        };
        let old = match self.vertices.get(vid) {
            Some(v) => v.get(key).unwrap_or("").to_string(),
            None => String::new(),
        };
        for tid in types.iter() {
            let tname = match self.types.get(tid) {
                Some(t) => t.name().to_string(),
                None => continue,
            };
            let iid = SchemaIndex::id(&tname, key);
            if let Some(idx) = self.indexes.get(&iid) {
                if idx.unique() && idx.contains_other(value, vid) {
                    return Err(Error::new("unique constraint"));
                }
            }
        }
        if let Some(v) = self.vertices.get_mut(vid) {
            v.set_attr(key, value);
        }
        if key == "name" {
            if !old.is_empty() {
                if let Some(owned) = self.vertices_by_name.get(&old).cloned() {
                    if owned == vid {
                        self.vertices_by_name.remove(&old);
                    }
                }
            }
            if !self.vertices_by_name.contains_key(value) {
                self.vertices_by_name.insert(value.to_string(), vid.to_string());
            }
        }
        for tid in types.iter() {
            let tname = match self.types.get(tid) {
                Some(t) => t.name().to_string(),
                None => continue,
            };
            let iid = SchemaIndex::id(&tname, key);
            if let Some(idx) = self.indexes.get_mut(&iid) {
                idx.remove(vid, &old);
                idx.add(vid, value);
            }
        }
        Ok(())
    }

    pub fn remove_attr(&mut self, vid: &str, key: &str) -> Result<Option<String>> {
        let types: Vec<String> = match self.vertices.get(vid) {
            Some(v) => v.types().iter().map(|s| s.clone()).collect(),
            None => return Err(Error::new("missing vertex")),
        };
        let old = match self.vertices.get(vid) {
            Some(v) => v.get(key).map(|s| s.to_string()),
            None => None,
        };
        let old_s = old.clone().unwrap_or(String::new());
        if key == "name" && !old_s.is_empty() {
            if let Some(owned) = self.vertices_by_name.get(&old_s).cloned() {
                if owned == vid {
                    self.vertices_by_name.remove(&old_s);
                }
            }
        }
        for tid in types.iter() {
            let tname = match self.types.get(tid) {
                Some(t) => t.name().to_string(),
                None => continue,
            };
            let iid = SchemaIndex::id(&tname, key);
            if let Some(idx) = self.indexes.get_mut(&iid) {
                idx.remove(vid, &old_s);
            }
        }
        match self.vertices.get_mut(vid) {
            Some(v) => Ok(v.remove_attr(key)),
            None => Err(Error::new("missing vertex")),
        }
    }

    pub fn all_types(&self) -> Vec<(String, String)> {
        self.types.values().map(|t| (t.khid().to_string(), t.name().to_string())).collect()
    }

    pub fn all_edges(&self) -> Vec<(String, String, String, Option<String>)> {
        self.edges
            .values()
            .map(|e| {
                (e.khid().to_string(),
                 e.source().to_string(),
                 e.target().to_string(),
                 e.type_id().map(|s| s.to_string()))
            })
            .collect()
    }

    pub fn restore_vertex(&mut self,
                          id: String,
                          attrs: HashMap<String, String>,
                          type_names: Vec<String>)
                          -> Result<String> {
        self.note_id(&id);
        let mut v = Vertex::new(id.clone(), attrs);
        if let Some(name) = v.get("name") {
            if !self.vertices_by_name.contains_key(name) {
                self.vertices_by_name.insert(name.to_string(), id.clone());
            }
        }
        let names = type_names;
        for (i, tn) in names.iter().enumerate() {
            let tid = self.add_type(tn)?;
            v.attach_type(&tid);
            if let Some(t) = self.types.get_mut(&tid) {
                t.add_vertex(&id);
            }
            let _ = i;
        }
        self.vertices.insert(id.clone(), v);
        Ok(id)
    }

    pub fn restore_edge(&mut self,
                        id: String,
                        src: String,
                        dst: String,
                        type_name: Option<String>)
                        -> Result<String> {
        if !self.vertices.contains_key(&src) || !self.vertices.contains_key(&dst) {
            return Err(Error::new("missing vertex"));
        }
        self.note_id(&id);
        let mut e = Edge::new(id.clone(), src.clone(), dst.clone(), HashMap::new());
        if let Some(ref tn) = type_name {
            if !tn.is_empty() {
                let tid = self.add_type(tn)?;
                e.set_type(&tid);
                if let Some(t) = self.types.get_mut(&tid) {
                    t.add_edge(&id);
                }
            }
        }
        {
            let srcv = self.vertices.get_mut(&src).unwrap();
            srcv.add_out(&id);
        }
        {
            let dstv = self.vertices.get_mut(&dst).unwrap();
            dstv.add_in(&id);
        }
        self.edges.insert(id.clone(), e);
        Ok(id)
    }

    pub fn type_names_of_vertex(&self, vid: &str) -> Vec<String> {
        match self.vertices.get(vid) {
            Some(v) => {
                let mut names = Vec::new();
                for tid in v.types().iter() {
                    if let Some(t) = self.types.get(tid) {
                        names.push(t.name().to_string());
                    }
                }
                names
            }
            None => Vec::new(),
        }
    }

    pub fn edge_type_name(&self, eid: &str) -> Option<String> {
        match self.edges.get(eid) {
            Some(e) => e.type_id().and_then(|tid| self.types.get(tid).map(|t| t.name().to_string())),
            None => None,
        }
    }


    pub fn set_edge_attr(&mut self, eid: &str, key: &str, value: &str) -> bool {
        let tid = match self.edges.get(eid) {
            Some(e) => e.type_id().map(|s| s.to_string()),
            None => return false,
        };
        let old = match self.edges.get(eid) {
            Some(e) => e.get(key).unwrap_or("").to_string(),
            None => String::new(),
        };
        match self.edges.get_mut(eid) {
            Some(e) => {
                e.set_attr(key, value);
            }
            None => return false,
        }
        if let Some(tn) = tid.and_then(|t| self.type_name_of(&t).map(|s| s.to_string())) {
            let iid = SchemaIndex::id(&tn, key);
            if let Some(idx) = self.edge_indexes.get_mut(&iid) {
                idx.remove(eid, &old);
                idx.add(eid, value);
            }
        }
        true
    }
}

impl Default for Graph {
    fn default() -> Graph {
        Graph::new()
    }
}
