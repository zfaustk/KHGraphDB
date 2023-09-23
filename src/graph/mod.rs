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

/// A live write, for the log. Restore does not record.
#[derive(Clone, Debug)]
pub enum Touch {
    Vertex(Khid),
    Edge(Khid),
    DropVertex(Khid),
    DropEdge(Khid),
    Index { type_name: String, key: String, unique: bool },
    Content { type_name: String, key: String },
    VecMark { type_name: String, key: String },
    Emb { id: Khid, key: String },
}

/// Inverse of a live write. Rollback walks this
/// backwards. Restore does not record.
#[derive(Clone, Debug)]
pub enum Undo {
    VertexGone(Khid),
    VertexWas {
        id: Khid,
        types: Vec<String>,
        attrs: HashMap<String, Prop>,
    },
    EdgeGone(Khid),
    EdgeWas {
        id: Khid,
        src: Khid,
        dst: Khid,
        ty: String,
        attrs: HashMap<String, Prop>,
        far: Option<Addr>,
    },
    IndexGone {
        type_name: String,
        key: String,
    },
    EmbGone { id: Khid, key: String },
    EmbWas {
        id: Khid,
        key: String,
        old: Option<Vec<f32>>,
    },
}

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
    vectors: HashMap<(Khid, String), Vec<f32>>,
    recording: bool,
    armed: bool,
    touches: Vec<Touch>,
    undos: Vec<Undo>,
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
            vectors: HashMap::new(),
            recording: true,
            armed: false,
            touches: Vec::new(),
            undos: Vec::new(),
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

    pub(crate) fn quiet(&mut self) {
        self.recording = false;
        self.touches.clear();
    }

    pub(crate) fn live(&mut self) {
        self.recording = true;
        self.armed = false;
        self.touches.clear();
        self.undos.clear();
    }

    pub(crate) fn arm(&mut self) {
        self.armed = true;
        self.touches.clear();
        self.undos.clear();
    }

    pub(crate) fn disarm(&mut self) {
        self.armed = false;
        self.touches.clear();
        self.undos.clear();
    }

    pub(crate) fn apply_undos(&mut self) {
        let rec = self.recording;
        let arm = self.armed;
        self.recording = false;
        self.armed = false;
        let undos = ::std::mem::replace(&mut self.undos, Vec::new());
        for u in undos.into_iter().rev() {
            match u {
                Undo::VertexGone(id) => {
                    self.remove_vertex(id);
                }
                Undo::VertexWas { id, types, attrs } => {
                    let _ = self.restore_vertex(id, attrs, types);
                }
                Undo::EdgeGone(id) => {
                    self.remove_edge(id);
                }
                Undo::EdgeWas { id, src, dst, ty, attrs, far } => {
                    let tno = if ty.is_empty() { None } else { Some(ty) };
                    if let Some(a) = far {
                        let _ = self.restore_far_edge(id, src, a, tno, attrs);
                    } else {
                        let _ = self.restore_edge(id, src, dst, tno, attrs);
                    }
                }
                Undo::IndexGone { type_name, key } => {
                    self.indexes.remove(&SchemaIndex::id(&type_name, &key));
                }
                Undo::EmbGone { id, key } => {
                    self.vectors.remove(&(id, key));
                }
                Undo::EmbWas { id, key, old } => {
                    match old {
                        Some(v) => {
                            self.vectors.insert((id, key), v);
                        }
                        None => {
                            self.vectors.remove(&(id, key));
                        }
                    }
                }
            }
        }
        self.recording = rec;
        self.armed = arm;
        self.touches.clear();
        self.undos.clear();
    }

    pub(crate) fn undos(&self) -> &[Undo] {
        &self.undos
    }

    pub(crate) fn touches(&self) -> &[Touch] {
        &self.touches
    }

    fn rec(&mut self, t: Touch) {
        if self.recording && self.armed {
            self.touches.push(t);
        }
    }

    fn rec_undo(&mut self, u: Undo) {
        if self.recording && self.armed {
            self.undos.push(u);
        }
    }

    fn push_vertex_was(&mut self, id: Khid) {
        if let Some(v) = self.vertex(id) {
            let attrs = v.attrs().clone();
            let types = self.type_names_of_vertex(id);
            self.rec_undo(Undo::VertexWas {
                id: id,
                types: types,
                attrs: attrs,
            });
        }
    }

    fn push_edge_was(&mut self, id: Khid) {
        if let Some(e) = self.edge(id) {
            let ty = self.edge_type_name(id).unwrap_or(String::new());
            self.rec_undo(Undo::EdgeWas {
                id: id,
                src: e.source(),
                dst: e.target(),
                ty: ty,
                attrs: e.attrs().clone(),
                far: e.far(),
            });
        }
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
        self.touches.clear();
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
        self.rec_undo(Undo::VertexGone(kid));
        self.rec(Touch::Vertex(kid));
        Ok(kid)
    }

    /// Install at a KHID the engine already handed out.
    pub fn adopt_vertex(&mut self,
                        id: Khid,
                        attrs: HashMap<String, Prop>,
                        type_name: Option<&str>)
                        -> Result<Khid> {
        if id.is_nil() {
            return Err(Error::new("nil"));
        }
        self.note_khid(id);
        let mut v = Vertex::with_props(id, attrs);
        if let Some(name) = v.get("name") {
            if !self.vertices_by_name.contains_key(name) {
                self.vertices_by_name.insert(name.to_string(), id);
            }
        }
        if let Some(tn) = type_name {
            let tid = self.add_type(tn)?;
            v.attach_type(tid);
            if let Some(t) = self.tget_mut(tid) {
                t.add_vertex(id);
            }
            let keys: Vec<(String, Prop)> = v.attrs()
                .iter()
                .map(|(k, val)| (k.clone(), val.clone()))
                .collect();
            for (k, val) in keys.iter() {
                self.post_vertex(tn, id, k, val);
            }
        }
        self.vput(id, v);
        self.rec_undo(Undo::VertexGone(id));
        self.rec(Touch::Vertex(id));
        Ok(id)
    }

    pub fn serial(&self) -> u64 {
        self.serial
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
        self.rec_undo(Undo::EdgeGone(kid));
        self.rec(Touch::Edge(kid));
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
        self.rec_undo(Undo::EdgeGone(kid));
        self.rec(Touch::Edge(kid));
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
        self.push_edge_was(ek);
        self.rec(Touch::DropEdge(ek));
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
        let drop_keys: Vec<String> = self.vectors.keys()
            .filter(|k| k.0 == vk)
            .map(|k| k.1.clone())
            .collect();
        for key in drop_keys.iter() {
            if let Some(old) = self.vectors.remove(&(vk, key.clone())) {
                self.rec_undo(Undo::EmbWas {
                    id: vk,
                    key: key.clone(),
                    old: Some(old),
                });
            }
        }
        self.push_vertex_was(vk);
        self.rec(Touch::DropVertex(vk));
        self.vtake(vk)
    }

    pub fn add_type_to_vertex(&mut self, vk: Khid, type_name: &str) -> Result<bool> {
        if !self.vhas(vk) {
            return Err(Error::new("missing vertex"));
        }
        if self.has_type(vk, type_name) {
            return Ok(false);
        }
        self.push_vertex_was(vk);
        let tid = self.add_type(type_name)?;
        {
            let v = self.at_mut(vk).unwrap();
            v.attach_type(tid);
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
        self.rec(Touch::Vertex(vk));
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
}

include!("posting.rs");
