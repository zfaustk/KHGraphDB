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
    pub fn derive_from(&mut self, hit: Khid, src: Addr) -> Result<Khid> {
        if let Some(h) = self.hash_for_hit(hit) {
            let _ = self.set_attr(hit, "view", &format!("{:x}", h));
        }
        self.add_far_edge(hit, src, Some("DERIVED_FROM"))
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
}
