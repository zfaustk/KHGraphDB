impl Graph {
    /// An episode is a vertex. It is not a view.
    /// KEEP and NOTE hop IN while one is open.
    pub fn episode(&mut self) -> Result<Khid> {
        self.episode_as("")
    }

    pub fn episode_as(&mut self, name: &str) -> Result<Khid> {
        self.episode_at(name, None)
    }

    /// Stamp the prefix this look sees. Content, not SET.
    pub fn episode_at(&mut self, name: &str, pos: Option<&str>) -> Result<Khid> {
        let _ = self.add_type("Episode")?;
        self.mark_content("Episode", "pos");
        let mut attrs = HashMap::new();
        if !name.is_empty() {
            attrs.insert("name".to_string(), name.to_string());
        }
        let id = self.add_vertex(attrs, Some("Episode"))?;
        if let Some(p) = pos {
            let _ = self.stamp_pos(id, p);
        }
        self.open_episode = Some(id);
        Ok(id)
    }

    pub fn open_episode(&self) -> Option<Khid> {
        self.open_episode
    }

    /// The Pos on the open episode. KEEP inherits it.
    pub fn look_pos(&self) -> Option<String> {
        match self.open_episode {
            Some(e) => self.vertex(e).and_then(|v| v.get("pos").map(|s| s.to_string())),
            None => None,
        }
    }

    pub fn close_episode(&mut self) {
        self.open_episode = None;
    }

    pub fn set_episode(&mut self, id: Khid) -> bool {
        if !self.vhas(id) {
            return false;
        }
        self.open_episode = Some(id);
        true
    }

    /// Reopen by name. The cursor is still not on the log.
    pub fn open_named(&mut self, name: &str) -> bool {
        match self.vertex_by_name(name) {
            Some(v) => self.set_episode(v.khid()),
            None => false,
        }
    }

    /// Member -> Episode. Same Addr twice is one hop.
    pub fn enclose(&mut self, member: Khid, ep: Khid) -> Result<Khid> {
        if !self.vhas(member) || !self.vhas(ep) {
            return Err(Error::new("missing vertex"));
        }
        if let Some(eid) = self.in_edge(member, ep) {
            return Ok(eid);
        }
        self.add_edge(member, ep, Some("IN"))
    }

    pub fn enclose_open(&mut self, member: Khid) -> Result<Option<Khid>> {
        match self.open_episode {
            Some(ep) => self.enclose(member, ep).map(Some),
            None => Ok(None),
        }
    }

    fn in_edge(&self, member: Khid, ep: Khid) -> Option<Khid> {
        let v = match self.vertex(member) {
            Some(v) => v,
            None => return None,
        };
        for eid in v.outgoing().iter() {
            let e = match self.edge(*eid) {
                Some(e) => e,
                None => continue,
            };
            match self.edge_type_name(*eid) {
                Some(ref n) if n == "IN" => {}
                _ => continue,
            }
            if !e.is_far() && e.target() == ep {
                return Some(*eid);
            }
        }
        None
    }

    pub fn episode_of(&self, member: Khid) -> Vec<Khid> {
        let mut out = Vec::new();
        let v = match self.vertex(member) {
            Some(v) => v,
            None => return out,
        };
        for eid in v.outgoing().iter() {
            let e = match self.edge(*eid) {
                Some(e) => e,
                None => continue,
            };
            match self.edge_type_name(*eid) {
                Some(ref n) if n == "IN" => {}
                _ => continue,
            }
            if !e.is_far() {
                out.push(e.target());
            }
        }
        out
    }

    pub fn in_episode(&self, ep: Khid) -> Vec<Khid> {
        let mut out = Vec::new();
        let v = match self.vertex(ep) {
            Some(v) => v,
            None => return out,
        };
        for eid in v.incoming().iter() {
            let e = match self.edge(*eid) {
                Some(e) => e,
                None => continue,
            };
            match self.edge_type_name(*eid) {
                Some(ref n) if n == "IN" => {}
                _ => continue,
            }
            out.push(e.source());
        }
        out
    }

    fn is_bag(&self, id: Khid) -> bool {
        match self.vertex(id) {
            Some(v) => {
                for tid in v.types() {
                    if let Some(t) = self.ty(*tid) {
                        let n = t.name();
                        if n == "Episode" || n == "Community" {
                            return true;
                        }
                    }
                }
                false
            }
            None => false,
        }
    }

    /// IN must land on a bag. Hits stay. Compact
    /// calls this after drop_stale_derived.
    pub fn drop_orphan_in(&mut self) -> usize {
        let mut drop_e = Vec::new();
        for i in 1..self.edges.len() {
            let eid = Khid::from_raw(i as u64);
            let e = match self.edge(eid) {
                Some(e) => e,
                None => continue,
            };
            match self.edge_type_name(eid) {
                Some(ref n) if n == "IN" => {}
                _ => continue,
            }
            if e.is_far() {
                continue;
            }
            let dst = e.target();
            if !self.vhas(dst) || !self.is_bag(dst) {
                drop_e.push(eid);
            }
        }
        let n = drop_e.len();
        for eid in drop_e {
            self.remove_edge(eid);
        }
        n
    }
}
