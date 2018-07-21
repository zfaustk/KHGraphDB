use std::collections::HashMap;

use super::error::{Error, Result};
use super::vertex::Vertex;
use super::edge::Edge;
use super::ty::Type;
use super::index::SchemaIndex;
use super::prop::Prop;
use super::khid::Khid;

/// Directed property graph. Vertices live in a slot Vec.
/// The index is the KHID. KHID is identity.
#[derive(Clone)]
pub struct Graph {
    id: String,
    serial: u64,
    vertices: Vec<Option<Vertex>>,
    edges: Vec<Option<Edge>>,
    types: Vec<Option<Type>>,
    types_by_name: HashMap<String, Khid>,
    vertices_by_name: HashMap<String, Khid>,
    indexes: HashMap<String, SchemaIndex>,
    edge_indexes: HashMap<String, SchemaIndex>,
}

fn disp(k: Khid) -> String {
    format!("{}", k)
}

impl Graph {
    pub fn new() -> Graph {
        Graph::named("g1")
    }

    pub fn named(id: &str) -> Graph {
        Graph {
            id: id.to_string(),
            serial: 0,
            vertices: {
                let mut v = Vec::new();
                v.push(None);
                v
            },
            edges: {
                let mut v = Vec::new();
                v.push(None);
                v
            },
            types: {
                let mut v = Vec::new();
                v.push(None);
                v
            },
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
        self.vertices.push(None);
        self.edges.clear();
        self.edges.push(None);
        self.types.clear();
        self.types.push(None);
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
        let mut n = 0;
        let mut i = 1;
        while i < self.vertices.len() {
            if self.vertices[i].is_some() {
                n += 1;
            }
            i += 1;
        }
        n
    }

    pub fn edge_count(&self) -> usize {
        let mut n = 0;
        let mut i = 1;
        while i < self.edges.len() {
            if self.edges[i].is_some() {
                n += 1;
            }
            i += 1;
        }
        n
    }

    pub fn type_count(&self) -> usize {
        let mut n = 0;
        let mut i = 1;
        while i < self.types.len() {
            if self.types[i].is_some() {
                n += 1;
            }
            i += 1;
        }
        n
    }

    fn next_khid(&mut self) -> Khid {
        self.serial += 1;
        Khid::from_raw(self.serial)
    }

    fn note_id(&mut self, id: &str) {
        if let Some(k) = Khid::parse(id) {
            if k.raw() > self.serial {
                self.serial = k.raw();
            }
        }
    }

    fn at(&self, k: Khid) -> Option<&Vertex> {
        self.vertices.get(k.raw() as usize).and_then(|s| s.as_ref())
    }

    fn at_mut(&mut self, k: Khid) -> Option<&mut Vertex> {
        self.vertices.get_mut(k.raw() as usize).and_then(|s| s.as_mut())
    }

    fn vput(&mut self, k: Khid, v: Vertex) {
        let i = k.raw() as usize;
        while self.vertices.len() <= i {
            self.vertices.push(None);
        }
        self.vertices[i] = Some(v);
    }

    fn vtake(&mut self, k: Khid) -> bool {
        let i = k.raw() as usize;
        if i >= self.vertices.len() {
            return false;
        }
        self.vertices[i].take().is_some()
    }

    fn vhas(&self, k: Khid) -> bool {
        self.at(k).is_some()
    }

    fn eget(&self, k: Khid) -> Option<&Edge> {
        self.edges.get(k.raw() as usize).and_then(|s| s.as_ref())
    }

    fn eget_mut(&mut self, k: Khid) -> Option<&mut Edge> {
        self.edges.get_mut(k.raw() as usize).and_then(|s| s.as_mut())
    }

    fn eput(&mut self, k: Khid, e: Edge) {
        let i = k.raw() as usize;
        while self.edges.len() <= i {
            self.edges.push(None);
        }
        self.edges[i] = Some(e);
    }

    fn etake(&mut self, k: Khid) -> bool {
        let i = k.raw() as usize;
        if i >= self.edges.len() {
            return false;
        }
        self.edges[i].take().is_some()
    }

    fn tget(&self, k: Khid) -> Option<&Type> {
        self.types.get(k.raw() as usize).and_then(|s| s.as_ref())
    }

    fn tget_mut(&mut self, k: Khid) -> Option<&mut Type> {
        self.types.get_mut(k.raw() as usize).and_then(|s| s.as_mut())
    }

    fn tput(&mut self, k: Khid, t: Type) {
        let i = k.raw() as usize;
        while self.types.len() <= i {
            self.types.push(None);
        }
        self.types[i] = Some(t);
    }

    fn parse(&self, khid: &str) -> Option<Khid> {
        Khid::parse(khid)
    }

    pub fn vertex(&self, khid: &str) -> Option<&Vertex> {
        match self.parse(khid) {
            Some(k) => self.at(k),
            None => None,
        }
    }

    pub fn vertex_mut(&mut self, khid: &str) -> Option<&mut Vertex> {
        match Khid::parse(khid) {
            Some(k) => self.at_mut(k),
            None => None,
        }
    }

    pub fn edge(&self, khid: &str) -> Option<&Edge> {
        match Khid::parse(khid) {
            Some(k) => self.eget(k),
            None => None,
        }
    }

    pub fn ty(&self, khid: &str) -> Option<&Type> {
        match Khid::parse(khid) {
            Some(k) => self.tget(k),
            None => None,
        }
    }

    pub fn type_by_name(&self, name: &str) -> Option<&Type> {
        match self.types_by_name.get(name) {
            Some(id) => self.tget(*id),
            None => None,
        }
    }

    pub fn vertex_by_name(&self, name: &str) -> Option<&Vertex> {
        match self.vertices_by_name.get(name) {
            Some(id) => self.at(*id),
            None => None,
        }
    }

    pub fn add_type(&mut self, name: &str) -> Result<String> {
        if name.is_empty() {
            return Err(Error::new("empty type name"));
        }
        if let Some(id) = self.types_by_name.get(name) {
            return Ok(disp(*id));
        }
        let id = self.next_khid();
        let t = Type::with_khid(id, name.to_string());
        self.types_by_name.insert(name.to_string(), id);
        self.tput(id, t);
        Ok(disp(id))
    }

    pub fn add_vertex(&mut self,
                      attrs: HashMap<String, String>,
                      type_name: Option<&str>)
                      -> Result<String> {
        let mut p = HashMap::new();
        for (k, v) in attrs.into_iter() {
            p.insert(k, Prop::from_str(&v));
        }
        self.add_vertex_props(p, type_name)
    }

    pub fn add_vertex_props(&mut self,
                            attrs: HashMap<String, Prop>,
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
        let kid = self.next_khid();
        let id = disp(kid);
        let mut v = Vertex::with_props(kid, attrs);
        if let Some(name) = v.get("name") {
            if !self.vertices_by_name.contains_key(name) {
                self.vertices_by_name.insert(name.to_string(), kid);
            }
        }
        if let Some(tn) = type_name {
            let tid = self.add_type(tn)?;
            v.attach_type(&tid);
            match Khid::parse(&tid).and_then(|k| self.tget_mut(k)) {
                Some(t) => {
                    t.add_vertex(kid);
                }
                None => {}
            }
            let keys: Vec<(String, Prop)> = v.attrs()
                .iter()
                .map(|(k, val)| (k.clone(), val.clone()))
                .collect();
            for (k, val) in keys.iter() {
                let iid = SchemaIndex::id(tn, k);
                if let Some(idx) = self.indexes.get_mut(&iid) {
                    idx.add_khid(kid, val);
                }
            }
        }
        self.vput(kid, v);
        Ok(id)
    }

    pub fn add_edge(&mut self,
                    src: &str,
                    dst: &str,
                    type_name: Option<&str>)
                    -> Result<String> {
        self.add_edge_with(src, dst, type_name, HashMap::new())
    }

    pub fn add_edge_with(&mut self,
                         src: &str,
                         dst: &str,
                         type_name: Option<&str>,
                         attrs: HashMap<String, String>)
                         -> Result<String> {
        let sk = match Khid::parse(src) {
            Some(k) => k,
            None => return Err(Error::new("missing vertex")),
        };
        let dk = match Khid::parse(dst) {
            Some(k) => k,
            None => return Err(Error::new("missing vertex")),
        };
        if !self.vhas(sk) || !self.vhas(dk) {
            return Err(Error::new("missing vertex"));
        }
        let kid = self.next_khid();
        let id = disp(kid);
        let mut props = HashMap::new();
        for (k, v) in attrs.into_iter() {
            props.insert(k, Prop::from_str(&v));
        }
        let mut e = Edge::with_props(kid, sk, dk, props);
        if let Some(tn) = type_name {
            let tid = self.add_type(tn)?;
            e.set_type(&tid);
            if let Some(t) = Khid::parse(&tid).and_then(|k| self.tget_mut(k)) {
                t.add_edge(kid);
            }
            let keys: Vec<(String, Prop)> = e.attrs()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (k, val) in keys.iter() {
                let iid = SchemaIndex::id(tn, k);
                if let Some(idx) = self.edge_indexes.get_mut(&iid) {
                    idx.add_khid(kid, val);
                }
            }
        }
        {
            let srcv = self.at_mut(sk).unwrap();
            srcv.add_out(kid);
        }
        {
            let dstv = self.at_mut(dk).unwrap();
            dstv.add_in(kid);
        }
        self.eput(kid, e);
        Ok(id)
    }

    pub fn remove_edge(&mut self, eid: &str) -> bool {
        let ek = match Khid::parse(eid) {
            Some(k) => k,
            None => return false,
        };
        let (src, dst, tid) = match self.eget(ek) {
            Some(e) => (e.source(), e.target(), e.type_id().map(|s| s.to_string())),
            None => return false,
        };
        if let Some(v) = self.at_mut(src) {
            v.remove_out(ek);
        }
        if let Some(v) = self.at_mut(dst) {
            v.remove_in(ek);
        }
        if let Some(t) = tid {
            if let Some(tk) = Khid::parse(&t) {
                if let Some(ty) = self.tget_mut(tk) {
                    ty.remove_edge(ek);
                }
            }
        }
        self.etake(ek)
    }

    pub fn remove_vertex(&mut self, vid: &str) -> bool {
        let vk = match Khid::parse(vid) {
            Some(k) => k,
            None => return false,
        };
        let (outs, ins, tps, name) = match self.at(vk) {
            Some(v) => {
                let o: Vec<String> = Khid::display_all(v.outgoing());
                let i: Vec<String> = Khid::display_all(v.incoming());
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
            if let Some(tk) = Khid::parse(t) {
                if let Some(ty) = self.tget_mut(tk) {
                    ty.remove_vertex(vk);
                }
            }
        }
        if let Some(n) = name {
            if let Some(owned) = self.vertices_by_name.get(&n).cloned() {
                if owned == vk {
                    self.vertices_by_name.remove(&n);
                }
            }
        }
        self.vtake(vk)
    }

    pub fn add_type_to_vertex(&mut self, vid: &str, type_name: &str) -> Result<bool> {
        let tid = self.add_type(type_name)?;
        let vk = match Khid::parse(vid) {
            Some(k) => k,
            None => return Err(Error::new("missing vertex")),
        };
        {
            let v = match self.at_mut(vk) {
                Some(v) => v,
                None => return Err(Error::new("missing vertex")),
            };
            if !v.attach_type(&tid) {
                return Ok(false);
            }
        }
        if let Some(tk) = Khid::parse(&tid) {
            if let Some(t) = self.tget_mut(tk) {
                t.add_vertex(vk);
            }
        }
        Ok(true)
    }

    pub fn has_type(&self, vid: &str, type_name: &str) -> bool {
        let v = match self.vertex(vid) {
            Some(v) => v,
            None => return false,
        };
        for tid in v.types().iter() {
            if let Some(t) = self.ty(tid) {
                if t.name() == type_name {
                    return true;
                }
            }
        }
        false
    }

    pub fn vertices_of_type(&self, type_name: &str) -> Vec<String> {
        match self.type_by_name(type_name) {
            Some(t) => Khid::display_all(&t.vertices().iter().cloned().collect::<Vec<_>>()),
            None => Vec::new(),
        }
    }

    pub fn edges_of_type(&self, type_name: &str) -> Vec<String> {
        match self.type_by_name(type_name) {
            Some(t) => Khid::display_all(&t.edges().iter().cloned().collect::<Vec<_>>()),
            None => Vec::new(),
        }
    }

    pub fn vertex_ids(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut i = 1;
        while i < self.vertices.len() {
            if self.vertices[i].is_some() {
                out.push(disp(Khid::from_raw(i as u64)));
            }
            i += 1;
        }
        out
    }

    pub fn type_name_of(&self, tid: &str) -> Option<&str> {
        self.ty(tid).map(|t| t.name())
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
            let val = match self.vertex(vid).and_then(|v| v.get_prop(key)).cloned() {
                Some(p) => p,
                None => continue,
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
            if let Some(p) = self.edge(eid).and_then(|e| e.get_prop(key)).cloned() {
                idx.add(eid, &p);
            }
        }
        self.edge_indexes.insert(id, idx);
        true
    }

    pub fn find_edge(&self, type_name: &str, key: &str, value: &str) -> Vec<String> {
        self.find_edge_prop(type_name, key, &Prop::from_str(value))
    }

    pub fn find_edge_prop(&self, type_name: &str, key: &str, value: &Prop) -> Vec<String> {
        let id = SchemaIndex::id(type_name, key);
        if let Some(idx) = self.edge_indexes.get(&id) {
            return idx.get(value);
        }
        let mut hits = Vec::new();
        for eid in self.edges_of_type(type_name).iter() {
            if let Some(e) = self.edge(eid) {
                if e.get_prop(key) == Some(value) {
                    hits.push(eid.clone());
                }
            }
        }
        hits
    }

    pub fn find(&self, type_name: &str, key: &str, value: &str) -> Vec<String> {
        self.find_prop(type_name, key, &Prop::from_str(value))
    }

    pub fn find_prop(&self, type_name: &str, key: &str, value: &Prop) -> Vec<String> {
        let id = SchemaIndex::id(type_name, key);
        if let Some(idx) = self.indexes.get(&id) {
            return idx.get(value);
        }
        let mut hits = Vec::new();
        for vid in self.vertices_of_type(type_name).iter() {
            if let Some(v) = self.vertex(vid) {
                if v.get_prop(key) == Some(value) {
                    hits.push(vid.clone());
                }
            }
        }
        hits
    }

    /// True when (Type, key) has a posting list.
    pub fn has_index(&self, type_name: &str, key: &str) -> bool {
        self.indexes.contains_key(&SchemaIndex::id(type_name, key))
    }

    pub fn set_attr(&mut self, vid: &str, key: &str, value: &str) -> Result<()> {
        self.set_prop(vid, key, Prop::from_str(value))
    }

    pub fn set_prop(&mut self, vid: &str, key: &str, value: Prop) -> Result<()> {
        let vk = match Khid::parse(vid) {
            Some(k) => k,
            None => return Err(Error::new("missing vertex")),
        };
        let types: Vec<String> = match self.at(vk) {
            Some(v) => v.types().iter().map(|s| s.clone()).collect(),
            None => return Err(Error::new("missing vertex")),
        };
        let old_prop = self.at(vk).and_then(|v| v.get_prop(key)).cloned();
        let old_name = self.at(vk).and_then(|v| v.get("name")).unwrap_or("").to_string();
        for tid in types.iter() {
            let tname = match self.ty(tid) {
                Some(t) => t.name().to_string(),
                None => continue,
            };
            let iid = SchemaIndex::id(&tname, key);
            if let Some(idx) = self.indexes.get(&iid) {
                if idx.unique() && idx.contains_other(&value, vid) {
                    return Err(Error::new("unique constraint"));
                }
            }
        }
        if let Some(v) = self.at_mut(vk) {
            v.set_prop(key, value.clone());
        }
        if key == "name" {
            if !old_name.is_empty() {
                if let Some(owned) = self.vertices_by_name.get(&old_name).cloned() {
                    if owned == vk {
                        self.vertices_by_name.remove(&old_name);
                    }
                }
            }
            if let Prop::Str(ref s) = value {
                if !self.vertices_by_name.contains_key(s) {
                    self.vertices_by_name.insert(s.clone(), vk);
                }
            }
        }
        for tid in types.iter() {
            let tname = match self.ty(tid) {
                Some(t) => t.name().to_string(),
                None => continue,
            };
            let iid = SchemaIndex::id(&tname, key);
            if let Some(idx) = self.indexes.get_mut(&iid) {
                if let Some(ref o) = old_prop {
                    idx.remove(vid, o);
                }
                idx.add(vid, &value);
            }
        }
        Ok(())
    }

    pub fn remove_attr(&mut self, vid: &str, key: &str) -> Result<Option<String>> {
        let vk = match Khid::parse(vid) {
            Some(k) => k,
            None => return Err(Error::new("missing vertex")),
        };
        let types: Vec<String> = match self.at(vk) {
            Some(v) => v.types().iter().map(|s| s.clone()).collect(),
            None => return Err(Error::new("missing vertex")),
        };
        let old_prop = self.at(vk).and_then(|v| v.get_prop(key)).cloned();
        let old_s = match old_prop {
            Some(Prop::Str(ref s)) => s.clone(),
            _ => String::new(),
        };
        if key == "name" && !old_s.is_empty() {
            if let Some(owned) = self.vertices_by_name.get(&old_s).cloned() {
                if owned == vk {
                    self.vertices_by_name.remove(&old_s);
                }
            }
        }
        for tid in types.iter() {
            let tname = match self.ty(tid) {
                Some(t) => t.name().to_string(),
                None => continue,
            };
            let iid = SchemaIndex::id(&tname, key);
            if let Some(idx) = self.indexes.get_mut(&iid) {
                if let Some(ref o) = old_prop {
                    idx.remove(vid, o);
                }
            }
        }
        match self.at_mut(vk) {
            Some(v) => Ok(v.remove_attr(key).map(|p| p.as_display())),
            None => Err(Error::new("missing vertex")),
        }
    }

    pub fn all_types(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let mut i = 1;
        while i < self.types.len() {
            if let Some(ref t) = self.types[i] {
                out.push((disp(t.khid()), t.name().to_string()));
            }
            i += 1;
        }
        out
    }

    pub fn all_edges(&self) -> Vec<(String, String, String, Option<String>)> {
        let mut out = Vec::new();
        let mut i = 1;
        while i < self.edges.len() {
            if let Some(ref e) = self.edges[i] {
                out.push((disp(e.khid()),
                          disp(e.source()),
                          disp(e.target()),
                          e.type_id().map(|s| s.to_string())));
            }
            i += 1;
        }
        out
    }

    pub fn restore_vertex(&mut self,
                          id: String,
                          attrs: HashMap<String, Prop>,
                          type_names: Vec<String>)
                          -> Result<String> {
        self.note_id(&id);
        let kid = match Khid::parse(&id) {
            Some(k) => k,
            None => return Err(Error::new("bad khid")),
        };
        let mut v = Vertex::with_props(kid, attrs);
        if let Some(name) = v.get("name") {
            if !self.vertices_by_name.contains_key(name) {
                self.vertices_by_name.insert(name.to_string(), kid);
            }
        }
        let names = type_names;
        for (i, tn) in names.iter().enumerate() {
            let tid = self.add_type(tn)?;
            v.attach_type(&tid);
            if let Some(tk) = Khid::parse(&tid) {
                if let Some(t) = self.tget_mut(tk) {
                    t.add_vertex(kid);
                }
            }
            let _ = i;
        }
        self.vput(kid, v);
        Ok(id)
    }

    pub fn restore_edge(&mut self,
                        id: String,
                        src: String,
                        dst: String,
                        type_name: Option<String>,
                        attrs: HashMap<String, Prop>)
                        -> Result<String> {
        let sk = match Khid::parse(&src) {
            Some(k) => k,
            None => return Err(Error::new("missing vertex")),
        };
        let dk = match Khid::parse(&dst) {
            Some(k) => k,
            None => return Err(Error::new("missing vertex")),
        };
        if !self.vhas(sk) || !self.vhas(dk) {
            return Err(Error::new("missing vertex"));
        }
        self.note_id(&id);
        let kid = match Khid::parse(&id) {
            Some(k) => k,
            None => return Err(Error::new("bad khid")),
        };
        let mut e = Edge::with_props(kid, sk, dk, attrs);
        if let Some(ref tn) = type_name {
            if !tn.is_empty() {
                let tid = self.add_type(tn)?;
                e.set_type(&tid);
                if let Some(tk) = Khid::parse(&tid) {
                    if let Some(t) = self.tget_mut(tk) {
                        t.add_edge(kid);
                    }
                }
            }
        }
        {
            let srcv = self.at_mut(sk).unwrap();
            srcv.add_out(kid);
        }
        {
            let dstv = self.at_mut(dk).unwrap();
            dstv.add_in(kid);
        }
        self.eput(kid, e);
        Ok(id)
    }

    pub fn type_names_of_vertex(&self, vid: &str) -> Vec<String> {
        match self.vertex(vid) {
            Some(v) => {
                let mut names = Vec::new();
                for tid in v.types().iter() {
                    if let Some(t) = self.ty(tid) {
                        names.push(t.name().to_string());
                    }
                }
                names
            }
            None => Vec::new(),
        }
    }

    pub fn edge_type_name(&self, eid: &str) -> Option<String> {
        match self.edge(eid) {
            Some(e) => e.type_id().and_then(|tid| self.ty(tid).map(|t| t.name().to_string())),
            None => None,
        }
    }

    pub fn set_edge_attr(&mut self, eid: &str, key: &str, value: &str) -> bool {
        self.set_edge_prop(eid, key, Prop::from_str(value))
    }

    pub fn set_edge_prop(&mut self, eid: &str, key: &str, value: Prop) -> bool {
        let ek = match Khid::parse(eid) {
            Some(k) => k,
            None => return false,
        };
        let tid = match self.eget(ek) {
            Some(e) => e.type_id().map(|s| s.to_string()),
            None => return false,
        };
        let old = self.eget(ek).and_then(|e| e.get_prop(key)).cloned();
        match self.eget_mut(ek) {
            Some(e) => {
                e.set_prop(key, value.clone());
            }
            None => return false,
        }
        if let Some(tn) = tid.and_then(|t| self.type_name_of(&t).map(|s| s.to_string())) {
            let iid = SchemaIndex::id(&tn, key);
            if let Some(idx) = self.edge_indexes.get_mut(&iid) {
                if let Some(ref o) = old {
                    idx.remove(eid, o);
                }
                idx.add(eid, &value);
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
