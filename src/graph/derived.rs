impl Graph {
    /// Recipe on the Type. Does not fill members.
    pub fn mark_view(&mut self, type_name: &str, query: &str) -> bool {
        if type_name.is_empty() || query.is_empty() {
            return false;
        }
        let tid = match self.add_type(type_name) {
            Ok(id) => id,
            Err(_) => return false,
        };
        match self.tget_mut(tid) {
            Some(t) => {
                t.mark_view(query);
            }
            None => return false,
        }
        self.rec(Touch::View {
            type_name: type_name.to_string(),
            query: query.to_string(),
        });
        true
    }

    pub fn view_of(&self, type_name: &str) -> Option<&str> {
        self.type_by_name(type_name).and_then(|t| t.view())
    }

    pub fn view_hash_of(&self, type_name: &str) -> Option<u64> {
        match self.type_by_name(type_name) {
            Some(t) if t.is_view() => Some(t.view_hash()),
            _ => None,
        }
    }

    pub fn is_view(&self, type_name: &str) -> bool {
        self.type_by_name(type_name).map(|t| t.is_view()).unwrap_or(false)
    }

    /// Point a hit at a source. Does not copy the page.
    /// Stamps the recipe hash on the hit.
    /// The same Addr twice is one edge.
    pub fn derive_from(&mut self, hit: Khid, src: Addr) -> Result<Khid> {
        if !self.vhas(hit) {
            return Err(Error::new("missing vertex"));
        }
        if let Some(eid) = self.derived_edge(hit, src) {
            return Ok(eid);
        }
        if let Some(h) = self.hash_for_hit(hit) {
            let hex = format!("{:x}", h);
            self.push_vertex_was(hit);
            if let Some(v) = self.at_mut(hit) {
                v.set_attr("view", &hex);
            }
            self.rec(Touch::Vertex(hit));
        }
        self.add_far_edge(hit, src, Some("DERIVED_FROM"))
    }

    fn derived_edge(&self, hit: Khid, src: Addr) -> Option<Khid> {
        let v = match self.vertex(hit) {
            Some(v) => v,
            None => return None,
        };
        for eid in v.outgoing().iter() {
            let e = match self.edge(*eid) {
                Some(e) => e,
                None => continue,
            };
            match self.edge_type_name(*eid) {
                Some(ref n) if n == "DERIVED_FROM" => {}
                _ => continue,
            }
            let a = if e.is_far() {
                match e.far() {
                    Some(a) => a,
                    None => continue,
                }
            } else {
                self.addr(e.target())
            };
            let want = if src.on(self.shard) {
                self.addr(src.khid())
            } else {
                src
            };
            if a == want {
                return Some(*eid);
            }
        }
        None
    }

    fn hash_for_hit(&self, hit: Khid) -> Option<u64> {
        let v = match self.vertex(hit) {
            Some(v) => v,
            None => return None,
        };
        for tid in v.types() {
            if let Some(t) = self.ty(*tid) {
                if t.is_view() {
                    return Some(t.view_hash());
                }
            }
        }
        None
    }

    /// Addresses this hit points at.
    pub fn derived(&self, hit: Khid) -> Vec<Addr> {
        let mut out = Vec::new();
        let v = match self.vertex(hit) {
            Some(v) => v,
            None => return out,
        };
        for eid in v.outgoing().iter() {
            let e = match self.edge(*eid) {
                Some(e) => e,
                None => continue,
            };
            match self.edge_type_name(*eid) {
                Some(ref n) if n == "DERIVED_FROM" => {}
                _ => continue,
            }
            if e.is_far() {
                if let Some(a) = e.far() {
                    out.push(a);
                }
            } else {
                out.push(self.addr(e.target()));
            }
        }
        out
    }

    /// Drop hits whose here-source is gone, or whose
    /// stamped hash is not the Type's recipe. Far
    /// sources are not ours to judge.
    pub fn drop_stale_derived(&mut self) -> usize {
        let mut drop_v = Vec::new();
        let views = self.view_type_ids();
        for tid in views {
            let members: Vec<Khid> = match self.ty(tid) {
                Some(t) => t.vertices().iter().cloned().collect(),
                None => continue,
            };
            for vid in members {
                if self.source_gone(vid) || self.recipe_mismatch(vid) {
                    drop_v.push(vid);
                }
            }
        }
        drop_v.sort();
        drop_v.dedup();
        let n = drop_v.len();
        for id in drop_v {
            self.remove_vertex(id);
        }
        n
    }

    fn view_type_ids(&self) -> Vec<Khid> {
        let mut v = Vec::new();
        for &(tid, _) in self.all_types().iter() {
            if let Some(t) = self.ty(tid) {
                if t.is_view() {
                    v.push(tid);
                }
            }
        }
        v
    }

    fn source_gone(&self, hit: Khid) -> bool {
        let srcs = self.derived(hit);
        if srcs.is_empty() {
            return true;
        }
        for a in srcs {
            if a.on(self.shard) && !self.vhas(a.khid()) {
                return true;
            }
        }
        false
    }

    fn recipe_mismatch(&self, hit: Khid) -> bool {
        let want = match self.hash_for_hit(hit) {
            Some(h) => format!("{:x}", h),
            None => return true,
        };
        match self.vertex(hit).and_then(|v| v.get("view")) {
            Some(s) if s == want => false,
            _ => true,
        }
    }
}
