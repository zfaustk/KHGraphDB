use std::collections::HashMap;

use super::error::{Error, Result};
use super::vertex::Vertex;
use super::edge::Edge;
use super::ty::Type;
use super::index::SchemaIndex;
use super::prop::Prop;
use super::khid::Khid;
use super::addr::Addr;
use super::stub::Stub;

/// Directed property graph. Vertices live in a slot Vec.
/// The index is the KHID. KHID is identity on this shard.
/// Lookups take Khid. Names stay strings.
#[derive(Clone)]
pub struct Graph {
    id: String,
    shard: u32,
    serial: u64,
    vertices: Vec<Option<Vertex>>,
    edges: Vec<Option<Edge>>,
    types: Vec<Option<Type>>,
    types_by_name: HashMap<String, Khid>,
    vertices_by_name: HashMap<String, Khid>,
    indexes: HashMap<String, SchemaIndex>,
    edge_indexes: HashMap<String, SchemaIndex>,
    stubs: HashMap<Addr, Stub>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph::named("g1")
    }

    pub fn named(id: &str) -> Graph {
        Graph::on(id, 1)
    }

    /// A graph that already knows its shard.
    pub fn on(id: &str, shard: u32) -> Graph {
        Graph {
            id: id.to_string(),
            shard: shard,
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
            stubs: HashMap::new(),
        }
    }

    /// Catalog name. Not a serial KHID.
    pub fn khid(&self) -> &str {
        &self.id
    }

    /// Home of every vertex in this arena.
    pub fn shard(&self) -> u32 {
        self.shard
    }

    pub fn set_shard(&mut self, shard: u32) {
        self.shard = shard;
    }

    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_string();
    }

    /// Address of a local serial. Far edges store this.
    pub fn addr(&self, id: Khid) -> Addr {
        Addr::new(self.shard, id)
    }

    /// A far title. Not the page.
    pub fn put_stub(&mut self, addr: Addr, title: &str, ver: u64) {
        self.stubs.insert(addr, Stub::new(title, ver));
    }

    pub fn stub(&self, addr: Addr) -> Option<&Stub> {
        self.stubs.get(&addr)
    }

    pub fn drop_stub(&mut self, addr: Addr) -> bool {
        self.stubs.remove(&addr).is_some()
    }

    /// Far end of an edge, if it left this box.
    pub fn cite(&self, eid: Khid) -> Option<Addr> {
        self.edge(eid).and_then(|e| e.far())
    }

    /// Stub title for a far cite. None if not hydrated.
    pub fn cite_title(&self, eid: Khid) -> Option<&str> {
        match self.cite(eid) {
            Some(a) => self.stub(a).map(|s| s.title()),
            None => None,
        }
    }

    /// Far ends on this graph. One round asks these.
    pub fn far_cites(&self) -> Vec<Addr> {
        let mut v = Vec::new();
        for &(id, _, _, _) in self.all_edges().iter() {
            if let Some(a) = self.cite(id) {
                if !v.iter().any(|x| *x == a) {
                    v.push(a);
                }
            }
        }
        v
    }

    /// Fill missing stubs from `get`. One pass.
    pub fn fill_round<F>(&mut self, mut get: F) -> usize
        where F: FnMut(Addr) -> Option<Stub>
    {
        let addrs = self.far_cites();
        let mut n = 0;
        for a in addrs.iter() {
            if self.stub(*a).is_some() {
                continue;
            }
            if let Some(s) = get(*a) {
                let title = s.title().to_string();
                let ver = s.ver();
                self.put_stub(*a, &title, ver);
                n += 1;
            }
        }
        n
    }

    /// Rebuild posting lists from the arena. Content
    /// keys stay off. Derived: drop and run again.
    pub fn rebuild_index(&mut self) {
        let mut specs: Vec<(String, String, bool)> = Vec::new();
        for idx in self.indexes.values() {
            specs.push((idx.type_name().to_string(), idx.key().to_string(), idx.unique()));
        }
        self.indexes.clear();
        for &(ref tn, ref k, u) in specs.iter() {
            self.create_index_inner(tn, k, u);
        }
        let mut edge_specs: Vec<(String, String)> = Vec::new();
        for idx in self.edge_indexes.values() {
            edge_specs.push((idx.type_name().to_string(), idx.key().to_string()));
        }
        self.edge_indexes.clear();
        for &(ref tn, ref k) in edge_specs.iter() {
            self.create_edge_index(tn, k);
        }
    }

    pub fn index_specs(&self) -> Vec<(String, String, bool)> {
        let mut v = Vec::new();
        for idx in self.indexes.values() {
            v.push((idx.type_name().to_string(), idx.key().to_string(), idx.unique()));
        }
        v
    }

    /// Postings as addresses. Meta is derived from this.
    pub fn index_addrs(&self) -> Vec<(String, String, String, Addr)> {
        let mut v = Vec::new();
        for idx in self.indexes.values() {
            let tn = idx.type_name().to_string();
            let key = idx.key().to_string();
            for (p, ids) in idx.entries().iter() {
                let val = match p.as_str() {
                    Some(s) => s.to_string(),
                    None => continue,
                };
                for id in ids.iter() {
                    v.push((tn.clone(), key.clone(), val.clone(), Addr::new(self.shard, *id)));
                }
            }
        }
        v
    }

    /// A copy of the arena. Writes on the copy do not
    /// touch the original. Transactions start here.
    pub fn snapshot(&self) -> Graph {
        self.clone()
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
        self.stubs.clear();
    }

    pub fn subgraph(&self, vids: &[Khid]) -> Graph {
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
                g.remove_vertex(*id);
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

    fn note_khid(&mut self, k: Khid) {
        if k.raw() > self.serial {
            self.serial = k.raw();
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

    pub fn vertex(&self, k: Khid) -> Option<&Vertex> {
        self.at(k)
    }

    pub fn vertex_mut(&mut self, k: Khid) -> Option<&mut Vertex> {
        self.at_mut(k)
    }

    pub fn edge(&self, k: Khid) -> Option<&Edge> {
        self.eget(k)
    }

    pub fn ty(&self, k: Khid) -> Option<&Type> {
        self.tget(k)
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

    pub fn add_type(&mut self, name: &str) -> Result<Khid> {
        if name.is_empty() {
            return Err(Error::new("empty type name"));
        }
        if let Some(id) = self.types_by_name.get(name) {
            return Ok(*id);
        }
        let id = self.next_khid();
        let t = Type::with_khid(id, name.to_string());
        self.types_by_name.insert(name.to_string(), id);
        self.tput(id, t);
        Ok(id)
    }

    pub fn add_vertex(&mut self,
                      attrs: HashMap<String, String>,
                      type_name: Option<&str>)
                      -> Result<Khid> {
        let mut p = HashMap::new();
        for (k, v) in attrs.into_iter() {
            p.insert(k, Prop::from_str(&v));
        }
        self.add_vertex_props(p, type_name)
    }

    pub fn add_vertex_props(&mut self,
                            attrs: HashMap<String, Prop>,
                            type_name: Option<&str>)
                            -> Result<Khid> {
        if let Some(tn) = type_name {
            for (k, val) in attrs.iter() {
                let iid = SchemaIndex::id(tn, k);
                if let Some(idx) = self.indexes.get(&iid) {
                    if idx.unique() && idx.contains_other(val, Khid::nil()) {
                        return Err(Error::new("unique constraint"));
                    }
                }
            }
        }
        let kid = self.next_khid();
        let mut v = Vertex::with_props(kid, attrs);
        if let Some(name) = v.get("name") {
            if !self.vertices_by_name.contains_key(name) {
                self.vertices_by_name.insert(name.to_string(), kid);
            }
        }
        if let Some(tn) = type_name {
            let tid = self.add_type(tn)?;
            v.attach_type(tid);
            if let Some(t) = self.tget_mut(tid) {
                t.add_vertex(kid);
            }
            let keys: Vec<(String, Prop)> = v.attrs()
                .iter()
                .map(|(k, val)| (k.clone(), val.clone()))
                .collect();
            for (k, val) in keys.iter() {
                self.post_vertex(tn, kid, k, val);
            }
        }
        self.vput(kid, v);
        Ok(kid)
    }

    pub fn add_edge(&mut self,
                    src: Khid,
                    dst: Khid,
                    type_name: Option<&str>)
                    -> Result<Khid> {
        self.add_edge_with(src, dst, type_name, HashMap::new())
    }

    pub fn add_edge_with(&mut self,
                         src: Khid,
                         dst: Khid,
                         type_name: Option<&str>,
                         attrs: HashMap<String, String>)
                         -> Result<Khid> {
        if !self.vhas(src) || !self.vhas(dst) {
            return Err(Error::new("missing vertex"));
        }
        let kid = self.next_khid();
        let mut props = HashMap::new();
        for (k, v) in attrs.into_iter() {
            props.insert(k, Prop::from_str(&v));
        }
        let mut e = Edge::with_props(kid, src, dst, props);
        if let Some(tn) = type_name {
            let tid = self.add_type(tn)?;
            e.set_type(tid);
            if let Some(t) = self.tget_mut(tid) {
                t.add_edge(kid);
            }
            let keys: Vec<(String, Prop)> = e.attrs()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            for (k, val) in keys.iter() {
                let iid = SchemaIndex::id(tn, k);
                if let Some(idx) = self.edge_indexes.get_mut(&iid) {
                    idx.add(kid, val);
                }
            }
        }
        {
            let srcv = self.at_mut(src).unwrap();
            srcv.add_out(kid);
        }
        {
            let dstv = self.at_mut(dst).unwrap();
            dstv.add_in(kid);
        }
        self.eput(kid, e);
        Ok(kid)
    }

    /// Cite an address that may not live here.
    /// Same-shard Addr becomes a local edge.
    pub fn add_far_edge(&mut self,
                        src: Khid,
                        dst: Addr,
                        type_name: Option<&str>)
                        -> Result<Khid> {
        if dst.on(self.shard) && self.vhas(dst.khid()) {
            return self.add_edge(src, dst.khid(), type_name);
        }
        if !self.vhas(src) {
            return Err(Error::new("missing vertex"));
        }
        let kid = self.next_khid();
        let mut e = Edge::with_props(kid, src, Khid::nil(), HashMap::new());
        e.set_far(dst);
        if let Some(tn) = type_name {
            let tid = self.add_type(tn)?;
            e.set_type(tid);
            if let Some(t) = self.tget_mut(tid) {
                t.add_edge(kid);
            }
        }
        {
            let srcv = self.at_mut(src).unwrap();
            srcv.add_out(kid);
        }
        self.eput(kid, e);
        Ok(kid)
    }

    pub fn remove_edge(&mut self, ek: Khid) -> bool {
        let (src, dst, tid) = match self.eget(ek) {
            Some(e) => (e.source(), e.target(), e.type_id()),
            None => return false,
        };
        if let Some(v) = self.at_mut(src) {
            v.remove_out(ek);
        }
        if let Some(v) = self.at_mut(dst) {
            v.remove_in(ek);
        }
        if let Some(tk) = tid {
            if let Some(ty) = self.tget_mut(tk) {
                ty.remove_edge(ek);
            }
        }
        self.unpost_edge(ek);
        self.etake(ek)
    }

    pub fn remove_vertex(&mut self, vk: Khid) -> bool {
        let (outs, ins, tps, name) = match self.at(vk) {
            Some(v) => {
                let o: Vec<Khid> = v.outgoing().iter().cloned().collect();
                let i: Vec<Khid> = v.incoming().iter().cloned().collect();
                let t: Vec<Khid> = v.types().iter().cloned().collect();
                let n = v.get("name").map(|s| s.to_string());
                (o, i, t, n)
            }
            None => return false,
        };
        for e in outs.iter() {
            self.remove_edge(*e);
        }
        for e in ins.iter() {
            self.remove_edge(*e);
        }
        for t in tps.iter() {
            if let Some(ty) = self.tget_mut(*t) {
                ty.remove_vertex(vk);
            }
        }
        if let Some(n) = name {
            if let Some(owned) = self.vertices_by_name.get(&n).cloned() {
                if owned == vk {
                    self.vertices_by_name.remove(&n);
                }
            }
        }
        self.unpost_vertex(vk);
        self.vtake(vk)
    }

    pub fn add_type_to_vertex(&mut self, vk: Khid, type_name: &str) -> Result<bool> {
        let tid = self.add_type(type_name)?;
        {
            let v = match self.at_mut(vk) {
                Some(v) => v,
                None => return Err(Error::new("missing vertex")),
            };
            if !v.attach_type(tid) {
                return Ok(false);
            }
        }
        if let Some(t) = self.tget_mut(tid) {
            t.add_vertex(vk);
        }
        let attrs = match self.vertex(vk) {
            Some(v) => v.attrs().clone(),
            None => return Ok(true),
        };
        for (k, val) in attrs.iter() {
            self.post_vertex(type_name, vk, k, val);
        }
        Ok(true)
    }

    pub fn has_type(&self, vk: Khid, type_name: &str) -> bool {
        let v = match self.vertex(vk) {
            Some(v) => v,
            None => return false,
        };
        for tid in v.types().iter() {
            if let Some(t) = self.tget(*tid) {
                if t.name() == type_name {
                    return true;
                }
            }
        }
        false
    }

    pub fn vertices_of_type(&self, type_name: &str) -> Vec<Khid> {
        match self.type_by_name(type_name) {
            Some(t) => t.vertices().iter().cloned().collect(),
            None => Vec::new(),
        }
    }

    pub fn edges_of_type(&self, type_name: &str) -> Vec<Khid> {
        match self.type_by_name(type_name) {
            Some(t) => t.edges().iter().cloned().collect(),
            None => Vec::new(),
        }
    }

    pub fn vertex_ids(&self) -> Vec<Khid> {
        let mut out = Vec::new();
        let mut i = 1;
        while i < self.vertices.len() {
            if self.vertices[i].is_some() {
                out.push(Khid::from_raw(i as u64));
            }
            i += 1;
        }
        out
    }

    pub fn type_name_of(&self, tid: Khid) -> Option<&str> {
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
        if self.ty_content(type_name, key) {
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
            let val = match self.vertex(*vid).and_then(|v| v.get_prop(key)).cloned() {
                Some(p) => p,
                None => continue,
            };
            if unique && idx.contains_other(&val, *vid) {
                return false;
            }
            idx.add(*vid, &val);
        }
        self.indexes.insert(id, idx);
        true
    }

    pub fn create_edge_index(&mut self, type_name: &str, key: &str) -> bool {
        if type_name.is_empty() || key.is_empty() {
            return false;
        }
        if self.ty_content(type_name, key) {
            return false;
        }
        let id = SchemaIndex::id(type_name, key);
        if self.edge_indexes.contains_key(&id) {
            return true;
        }
        let mut idx = SchemaIndex::new(type_name, key, false);
        let eids = self.edges_of_type(type_name);
        for eid in eids.iter() {
            if let Some(p) = self.edge(*eid).and_then(|e| e.get_prop(key)).cloned() {
                idx.add(*eid, &p);
            }
        }
        self.edge_indexes.insert(id, idx);
        true
    }

    pub fn find_edge(&self, type_name: &str, key: &str, value: &str) -> Vec<Khid> {
        self.find_edge_prop(type_name, key, &Prop::from_str(value))
    }

    pub fn find_edge_prop(&self, type_name: &str, key: &str, value: &Prop) -> Vec<Khid> {
        let id = SchemaIndex::id(type_name, key);
        if let Some(idx) = self.edge_indexes.get(&id) {
            return idx.get(value);
        }
        let mut hits = Vec::new();
        for eid in self.edges_of_type(type_name).iter() {
            if let Some(e) = self.edge(*eid) {
                if e.get_prop(key) == Some(value) {
                    hits.push(*eid);
                }
            }
        }
        hits
    }

    pub fn find(&self, type_name: &str, key: &str, value: &str) -> Vec<Khid> {
        self.find_prop(type_name, key, &Prop::from_str(value))
    }

    pub fn find_prop(&self, type_name: &str, key: &str, value: &Prop) -> Vec<Khid> {
        let id = SchemaIndex::id(type_name, key);
        if let Some(idx) = self.indexes.get(&id) {
            return idx.get(value);
        }
        let mut hits = Vec::new();
        for vid in self.vertices_of_type(type_name).iter() {
            if let Some(v) = self.vertex(*vid) {
                if v.get_prop(key) == Some(value) {
                    hits.push(*vid);
                }
            }
        }
        hits
    }

    /// True when (Type, key) has a posting list.
    pub fn has_index(&self, type_name: &str, key: &str) -> bool {
        self.indexes.contains_key(&SchemaIndex::id(type_name, key))
    }

    fn ty_content(&self, type_name: &str, key: &str) -> bool {
        match self.type_by_name(type_name) {
            Some(t) => t.is_content(key),
            None => false,
        }
    }

    /// Mark a property as payload. Drops a posting list
    /// if one already sat on that key.
    pub fn mark_content(&mut self, type_name: &str, key: &str) -> bool {
        if type_name.is_empty() || key.is_empty() {
            return false;
        }
        let tid = match self.add_type(type_name) {
            Ok(id) => id,
            Err(_) => return false,
        };
        match self.tget_mut(tid) {
            Some(t) => {
                t.mark_content(key);
            }
            None => return false,
        }
        self.indexes.remove(&SchemaIndex::id(type_name, key));
        self.edge_indexes.remove(&SchemaIndex::id(type_name, key));
        true
    }

    fn post_vertex(&mut self, type_name: &str, vid: Khid, key: &str, val: &Prop) {
        if self.ty_content(type_name, key) {
            return;
        }
        let iid = SchemaIndex::id(type_name, key);
        if let Some(idx) = self.indexes.get_mut(&iid) {
            idx.add(vid, val);
        }
    }

    fn post_restored(&mut self, vid: Khid) {
        let names = self.type_names_of_vertex(vid);
        let attrs = match self.vertex(vid) {
            Some(v) => v.attrs().clone(),
            None => return,
        };
        for tn in names.iter() {
            for (k, val) in attrs.iter() {
                self.post_vertex(tn, vid, k, val);
            }
        }
    }

    fn unpost_vertex(&mut self, vid: Khid) {
        let names = self.type_names_of_vertex(vid);
        let attrs = match self.vertex(vid) {
            Some(v) => v.attrs().clone(),
            None => return,
        };
        for tn in names.iter() {
            for (k, val) in attrs.iter() {
                let iid = SchemaIndex::id(tn, k);
                if let Some(idx) = self.indexes.get_mut(&iid) {
                    idx.remove(vid, val);
                }
            }
        }
    }

    fn unpost_edge(&mut self, eid: Khid) {
        let (tn, attrs) = match self.edge(eid) {
            Some(e) => {
                let tn = match e.type_id().and_then(|t| self.type_name_of(t).map(|s| s.to_string())) {
                    Some(s) => s,
                    None => return,
                };
                (tn, e.attrs().clone())
            }
            None => return,
        };
        for (k, val) in attrs.iter() {
            let iid = SchemaIndex::id(&tn, k);
            if let Some(idx) = self.edge_indexes.get_mut(&iid) {
                idx.remove(eid, val);
            }
        }
    }

    pub fn set_attr(&mut self, vid: Khid, key: &str, value: &str) -> Result<()> {
        self.set_prop(vid, key, Prop::from_str(value))
    }

    pub fn set_prop(&mut self, vk: Khid, key: &str, value: Prop) -> Result<()> {
        let types: Vec<Khid> = match self.at(vk) {
            Some(v) => v.types().iter().cloned().collect(),
            None => return Err(Error::new("missing vertex")),
        };
        let old_prop = self.at(vk).and_then(|v| v.get_prop(key)).cloned();
        let old_name = self.at(vk).and_then(|v| v.get("name")).unwrap_or("").to_string();
        for tid in types.iter() {
            let tname = match self.tget(*tid) {
                Some(t) => t.name().to_string(),
                None => continue,
            };
            let iid = SchemaIndex::id(&tname, key);
            if let Some(idx) = self.indexes.get(&iid) {
                if idx.unique() && idx.contains_other(&value, vk) {
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
            let tname = match self.tget(*tid) {
                Some(t) => t.name().to_string(),
                None => continue,
            };
            let iid = SchemaIndex::id(&tname, key);
            if self.ty_content(&tname, key) {
                continue;
            }
            if let Some(idx) = self.indexes.get_mut(&iid) {
                if let Some(ref o) = old_prop {
                    idx.remove(vk, o);
                }
                idx.add(vk, &value);
            }
        }
        Ok(())
    }

    pub fn remove_attr(&mut self, vk: Khid, key: &str) -> Result<Option<String>> {
        let types: Vec<Khid> = match self.at(vk) {
            Some(v) => v.types().iter().cloned().collect(),
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
            let tname = match self.tget(*tid) {
                Some(t) => t.name().to_string(),
                None => continue,
            };
            let iid = SchemaIndex::id(&tname, key);
            if let Some(idx) = self.indexes.get_mut(&iid) {
                if let Some(ref o) = old_prop {
                    idx.remove(vk, o);
                }
            }
        }
        match self.at_mut(vk) {
            Some(v) => Ok(v.remove_attr(key).map(|p| p.as_display())),
            None => Err(Error::new("missing vertex")),
        }
    }

    pub fn all_types(&self) -> Vec<(Khid, String)> {
        let mut out = Vec::new();
        let mut i = 1;
        while i < self.types.len() {
            if let Some(ref t) = self.types[i] {
                out.push((t.khid(), t.name().to_string()));
            }
            i += 1;
        }
        out
    }

    pub fn all_edges(&self) -> Vec<(Khid, Khid, Khid, Option<Khid>)> {
        let mut out = Vec::new();
        let mut i = 1;
        while i < self.edges.len() {
            if let Some(ref e) = self.edges[i] {
                out.push((e.khid(), e.source(), e.target(), e.type_id()));
            }
            i += 1;
        }
        out
    }

    pub fn restore_vertex(&mut self,
                          kid: Khid,
                          attrs: HashMap<String, Prop>,
                          type_names: Vec<String>)
                          -> Result<Khid> {
        if self.vhas(kid) {
            self.unpost_vertex(kid);
            let old_types: Vec<Khid> = match self.at(kid) {
                Some(v) => v.types().iter().cloned().collect(),
                None => Vec::new(),
            };
            if let Some(n) = self.at(kid).and_then(|v| v.get("name")).map(|s| s.to_string()) {
                if let Some(owned) = self.vertices_by_name.get(&n).cloned() {
                    if owned == kid {
                        self.vertices_by_name.remove(&n);
                    }
                }
            }
            for tid in old_types.iter() {
                if let Some(t) = self.tget_mut(*tid) {
                    t.remove_vertex(kid);
                }
            }
        }
        self.note_khid(kid);
        let mut v = Vertex::with_props(kid, attrs);
        if let Some(name) = v.get("name") {
            if !self.vertices_by_name.contains_key(name) {
                self.vertices_by_name.insert(name.to_string(), kid);
            }
        }
        for tn in type_names.iter() {
            let tid = self.add_type(tn)?;
            v.attach_type(tid);
            if let Some(t) = self.tget_mut(tid) {
                t.add_vertex(kid);
            }
        }
        self.vput(kid, v);
        self.post_restored(kid);
        Ok(kid)
    }

    pub fn restore_edge(&mut self,
                        kid: Khid,
                        src: Khid,
                        dst: Khid,
                        type_name: Option<String>,
                        attrs: HashMap<String, Prop>)
                        -> Result<Khid> {
        if !self.vhas(src) || !self.vhas(dst) {
            return Err(Error::new("missing vertex"));
        }
        self.note_khid(kid);
        let mut e = Edge::with_props(kid, src, dst, attrs);
        if let Some(ref tn) = type_name {
            if !tn.is_empty() {
                let tid = self.add_type(tn)?;
                e.set_type(tid);
                if let Some(t) = self.tget_mut(tid) {
                    t.add_edge(kid);
                }
            }
        }
        {
            let srcv = self.at_mut(src).unwrap();
            srcv.add_out(kid);
        }
        {
            let dstv = self.at_mut(dst).unwrap();
            dstv.add_in(kid);
        }
        self.eput(kid, e);
        Ok(kid)
    }

    pub fn restore_far_edge(&mut self,
                            kid: Khid,
                            src: Khid,
                            dst: Addr,
                            type_name: Option<String>,
                            attrs: HashMap<String, Prop>)
                            -> Result<Khid> {
        if dst.on(self.shard) && self.vhas(dst.khid()) {
            return self.restore_edge(kid, src, dst.khid(), type_name, attrs);
        }
        if !self.vhas(src) {
            return Err(Error::new("missing vertex"));
        }
        self.note_khid(kid);
        let mut e = Edge::with_props(kid, src, Khid::nil(), attrs);
        e.set_far(dst);
        if let Some(ref tn) = type_name {
            if !tn.is_empty() {
                let tid = self.add_type(tn)?;
                e.set_type(tid);
                if let Some(t) = self.tget_mut(tid) {
                    t.add_edge(kid);
                }
            }
        }
        {
            let srcv = self.at_mut(src).unwrap();
            srcv.add_out(kid);
        }
        self.eput(kid, e);
        Ok(kid)
    }

    pub fn type_names_of_vertex(&self, vid: Khid) -> Vec<String> {
        match self.vertex(vid) {
            Some(v) => {
                let mut names = Vec::new();
                for tid in v.types().iter() {
                    if let Some(t) = self.tget(*tid) {
                        names.push(t.name().to_string());
                    }
                }
                names
            }
            None => Vec::new(),
        }
    }

    pub fn edge_type_name(&self, eid: Khid) -> Option<String> {
        match self.edge(eid) {
            Some(e) => e.type_id().and_then(|tid| self.ty(tid).map(|t| t.name().to_string())),
            None => None,
        }
    }

    pub fn set_edge_attr(&mut self, eid: Khid, key: &str, value: &str) -> bool {
        self.set_edge_prop(eid, key, Prop::from_str(value))
    }

    pub fn set_edge_prop(&mut self, ek: Khid, key: &str, value: Prop) -> bool {
        let tid = match self.eget(ek) {
            Some(e) => e.type_id(),
            None => return false,
        };
        let old = self.eget(ek).and_then(|e| e.get_prop(key)).cloned();
        match self.eget_mut(ek) {
            Some(e) => {
                e.set_prop(key, value.clone());
            }
            None => return false,
        }
        if let Some(tn) = tid.and_then(|t| self.type_name_of(t).map(|s| s.to_string())) {
            let iid = SchemaIndex::id(&tn, key);
            if let Some(idx) = self.edge_indexes.get_mut(&iid) {
                if let Some(ref o) = old {
                    idx.remove(ek, o);
                }
                idx.add(ek, &value);
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
