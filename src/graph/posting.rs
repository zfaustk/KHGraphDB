impl Graph {
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
        if self.ty_vector(type_name, key) {
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
        self.rec_undo(Undo::IndexGone {
            type_name: type_name.to_string(),
            key: key.to_string(),
        });
        self.rec(Touch::Index {
            type_name: type_name.to_string(),
            key: key.to_string(),
            unique: unique,
        });
        true
    }

    pub fn create_edge_index(&mut self, type_name: &str, key: &str) -> bool {
        if type_name.is_empty() || key.is_empty() {
            return false;
        }
        if self.ty_content(type_name, key) {
            return false;
        }
        if self.ty_vector(type_name, key) {
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

    /// Range on an ordered posting. No index: empty.
    /// Filter still runs; this only cuts the seed.
    pub fn find_range(&self,
                      type_name: &str,
                      key: &str,
                      lo: Option<&Prop>,
                      hi: Option<&Prop>,
                      lo_inc: bool,
                      hi_inc: bool)
                      -> Vec<Khid> {
        let id = SchemaIndex::id(type_name, key);
        match self.indexes.get(&id) {
            Some(idx) => idx.range(lo, hi, lo_inc, hi_inc),
            None => Vec::new(),
        }
    }

    pub fn has_unique(&self, type_name: &str, key: &str) -> bool {
        let id = SchemaIndex::id(type_name, key);
        match self.indexes.get(&id) {
            Some(idx) => idx.unique(),
            None => false,
        }
    }

    pub fn index_len(&self, type_name: &str, key: &str) -> usize {
        let id = SchemaIndex::id(type_name, key);
        match self.indexes.get(&id) {
            Some(idx) => idx.len(),
            None => 0,
        }
    }

    /// True when (Type, key) has a posting list.
    pub fn has_index(&self, type_name: &str, key: &str) -> bool {
        self.indexes.contains_key(&SchemaIndex::id(type_name, key))
    }

    pub(crate) fn ty_content(&self, type_name: &str, key: &str) -> bool {
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
        self.rec(Touch::Content {
            type_name: type_name.to_string(),
            key: key.to_string(),
        });
        true
    }

    pub(crate) fn ty_vector(&self, type_name: &str, key: &str) -> bool {
        match self.type_by_name(type_name) {
            Some(t) => t.is_vector(key),
            None => false,
        }
    }

    /// Mark a property as an embedding. Drops a posting
    /// list. The floats live in vec/, not in the B-tree.
    pub fn mark_vector(&mut self, type_name: &str, key: &str) -> bool {
        if type_name.is_empty() || key.is_empty() {
            return false;
        }
        let tid = match self.add_type(type_name) {
            Ok(id) => id,
            Err(_) => return false,
        };
        match self.tget_mut(tid) {
            Some(t) => {
                t.mark_vector(key);
            }
            None => return false,
        }
        self.indexes.remove(&SchemaIndex::id(type_name, key));
        self.edge_indexes.remove(&SchemaIndex::id(type_name, key));
        self.rec(Touch::VecMark {
            type_name: type_name.to_string(),
            key: key.to_string(),
        });
        true
    }

    pub fn set_vec(&mut self, id: Khid, key: &str, v: &[f32]) -> bool {
        if self.vertex(id).is_none() || key.is_empty() {
            return false;
        }
        let old = self.vectors.get(&(id, key.to_string())).cloned();
        self.rec_undo(Undo::EmbWas {
            id: id,
            key: key.to_string(),
            old: old,
        });
        self.vectors.insert((id, key.to_string()), v.to_vec());
        self.rec(Touch::Emb {
            id: id,
            key: key.to_string(),
        });
        true
    }

    pub fn get_vec(&self, id: Khid, key: &str) -> Option<&[f32]> {
        self.vectors.get(&(id, key.to_string())).map(|v| v.as_slice())
    }

    /// Cosine over the type. The notebook is the set.
    pub fn similar(&self, type_name: &str, key: &str, q: &[f32], k: usize) -> Vec<(Khid, f32)> {
        let mut hits = Vec::new();
        let members = match self.type_by_name(type_name) {
            Some(t) => t.vertices().clone(),
            None => return hits,
        };
        for id in members.iter() {
            if let Some(v) = self.get_vec(*id, key) {
                let s = super::vec::cosine(q, v);
                hits.push((*id, s));
            }
        }
        hits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if hits.len() > k {
            hits.truncate(k);
        }
        hits
    }

    pub fn embeddings(&self) -> Vec<(Khid, String, Vec<f32>)> {
        let mut out = Vec::new();
        for (&(id, ref k), v) in self.vectors.iter() {
            out.push((id, k.clone(), v.clone()));
        }
        out
    }

    fn post_vertex(&mut self, type_name: &str, vid: Khid, key: &str, val: &Prop) {
        if self.ty_content(type_name, key) {
            return;
        }
        if self.ty_vector(type_name, key) {
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

    fn post_restored_edge(&mut self, eid: Khid) {
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
                idx.add(eid, val);
            }
        }
    }

    pub fn set_attr(&mut self, vid: Khid, key: &str, value: &str) -> Result<()> {
        self.set_prop(vid, key, Prop::from_str(value))
    }

    pub fn set_prop(&mut self, vk: Khid, key: &str, value: Prop) -> Result<()> {
        if !self.vhas(vk) {
            return Err(Error::new("missing vertex"));
        }
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
        self.push_vertex_was(vk);
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
        self.rec(Touch::Vertex(vk));
        Ok(())
    }

    pub fn remove_attr(&mut self, vk: Khid, key: &str) -> Result<Option<String>> {
        if !self.vhas(vk) {
            return Err(Error::new("missing vertex"));
        }
        self.push_vertex_was(vk);
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
        let out = match self.at_mut(vk) {
            Some(v) => Ok(v.remove_attr(key).map(|p| p.as_display())),
            None => Err(Error::new("missing vertex")),
        };
        if out.is_ok() {
            self.rec(Touch::Vertex(vk));
        }
        out
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
        if self.eget(kid).is_some() {
            self.remove_edge(kid);
        }
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
        self.post_restored_edge(kid);
        Ok(kid)
    }

    pub fn restore_far_edge(&mut self,
                            kid: Khid,
                            src: Khid,
                            dst: Addr,
                            type_name: Option<String>,
                            attrs: HashMap<String, Prop>)
                            -> Result<Khid> {
        if self.eget(kid).is_some() {
            self.remove_edge(kid);
        }
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
        self.post_restored_edge(kid);
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

    pub fn is_content_key(&self, id: Khid, key: &str) -> bool {
        for tn in self.type_names_of_vertex(id).iter() {
            if self.ty_content(tn, key) {
                return true;
            }
        }
        false
    }

    /// Content attrs of a vertex. These leave the WAL.
    pub fn content_of(&self, id: Khid) -> Vec<(String, Prop)> {
        let v = match self.vertex(id) {
            Some(v) => v,
            None => return Vec::new(),
        };
        let mut out = Vec::new();
        for (k, p) in v.attrs().iter() {
            if self.is_content_key(id, k) {
                out.push((k.clone(), p.clone()));
            }
        }
        out
    }

    pub fn strip_content(&self, id: Khid, attrs: &HashMap<String, Prop>) -> HashMap<String, Prop> {
        let mut m = HashMap::new();
        for (k, p) in attrs.iter() {
            if !self.is_content_key(id, k) {
                m.insert(k.clone(), p.clone());
            }
        }
        m
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
        self.push_edge_was(ek);
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
        self.rec(Touch::Edge(ek));
        true
    }
}

impl Default for Graph {
    fn default() -> Graph {
        Graph::new()
    }
}
