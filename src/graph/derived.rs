impl Graph {
    /// Recipe on the Type. Does not fill members.
    /// A new recipe retires the old Type. The name
    /// moves. The old Type keeps its KHID and its soup.
    pub fn mark_view(&mut self, type_name: &str, query: &str) -> bool {
        if type_name.is_empty() || query.is_empty() {
            return false;
        }
        if self.is_view(type_name) {
            if self.view_of(type_name) == Some(query) {
                self.rec(Touch::View {
                    type_name: type_name.to_string(),
                    query: query.to_string(),
                });
                return true;
            }
            if !self.retire_view(type_name) {
                return false;
            }
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

    fn retire_view(&mut self, name: &str) -> bool {
        let tid = match self.types_by_name.get(name) {
            Some(id) => *id,
            None => return false,
        };
        let hash = match self.ty(tid) {
            Some(t) if t.is_view() => t.view_hash(),
            _ => return false,
        };
        let mut retired = format!("{}~{:x}", name, hash);
        let mut n = 1u32;
        while self.types_by_name.contains_key(&retired) {
            retired = format!("{}~{:x}~{}", name, hash, n);
            n += 1;
        }
        self.types_by_name.remove(name);
        if let Some(t) = self.tget_mut(tid) {
            t.set_name(retired.clone());
        }
        self.types_by_name.insert(retired, tid);
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

    pub fn is_view_member(&self, id: Khid) -> bool {
        let v = match self.vertex(id) {
            Some(v) => v,
            None => return false,
        };
        for tid in v.types() {
            if let Some(t) = self.ty(*tid) {
                if t.is_view() {
                    return true;
                }
            }
        }
        false
    }

    pub fn wears(&self, id: Khid, tid: Khid) -> bool {
        match self.vertex(id) {
            Some(v) => v.types().iter().any(|t| *t == tid),
            None => false,
        }
    }

    /// World is 0. A hit is 1 plus the deepest
    /// here-source. Far is not a step.
    pub fn view_depth(&self, id: Khid) -> u32 {
        let mut seen = HashSet::new();
        self.depth_of(id, &mut seen)
    }

    fn depth_of(&self, id: Khid, seen: &mut HashSet<Khid>) -> u32 {
        if !seen.insert(id) {
            return 0;
        }
        if !self.is_view_member(id) {
            return 0;
        }
        let mut d = 0u32;
        for a in self.derived(id) {
            if a.on(self.shard) && self.vhas(a.khid()) {
                let dd = self.depth_of(a.khid(), seen);
                if dd > d {
                    d = dd;
                }
            }
        }
        d + 1
    }

    fn norm_addr(&self, a: Addr) -> Addr {
        if a.on(self.shard) && self.vhas(a.khid()) {
            self.addr(a.khid())
        } else {
            a
        }
    }

    /// A member of this Type that already cites
    /// exactly these addresses.
    pub fn hit_citing(&self, tid: Khid, srcs: &[Addr]) -> Option<Khid> {
        let members: Vec<Khid> = match self.ty(tid) {
            Some(t) => t.vertices().iter().cloned().collect(),
            None => return None,
        };
        let mut want: Vec<Addr> = srcs.iter().map(|a| self.norm_addr(*a)).collect();
        want.sort();
        want.dedup();
        for id in members {
            let mut have: Vec<Addr> = self.derived(id)
                .into_iter()
                .map(|a| self.norm_addr(a))
                .collect();
            have.sort();
            have.dedup();
            if have == want {
                return Some(id);
            }
        }
        None
    }

    pub fn stamp_pos(&mut self, id: Khid, pos: &str) -> bool {
        if pos.is_empty() || !self.vhas(id) {
            return false;
        }
        match self.set_attr(id, "pos", pos) {
            Ok(()) => true,
            Err(_) => false,
        }
    }

    /// A note cites the world. It is not a view.
    /// Stamp pos before the hop so a later vertex rec
    /// does not wipe the edge on replay.
    pub fn note(&mut self, src: Khid) -> Result<Khid> {
        self.note_at(src, None)
    }

    pub fn note_at(&mut self, src: Khid, pos: Option<&str>) -> Result<Khid> {
        if !self.vhas(src) {
            return Err(Error::new("missing vertex"));
        }
        let _ = self.add_type("Note")?;
        self.mark_content("Note", "pos");
        let id = self.add_vertex(HashMap::new(), Some("Note"))?;
        if let Some(p) = pos {
            let _ = self.stamp_pos(id, p);
        }
        self.add_edge(id, src, Some("SEEN"))?;
        let _ = self.enclose_open(id);
        Ok(id)
    }

    pub fn seen(&self, note: Khid) -> Vec<Addr> {
        let mut out = Vec::new();
        let v = match self.vertex(note) {
            Some(v) => v,
            None => return out,
        };
        for eid in v.outgoing().iter() {
            let e = match self.edge(*eid) {
                Some(e) => e,
                None => continue,
            };
            match self.edge_type_name(*eid) {
                Some(ref n) if n == "SEEN" => {}
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
}
