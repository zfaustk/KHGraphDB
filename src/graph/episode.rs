impl Graph {
    /// An episode is a vertex. It is not a view.
    /// KEEP and NOTE hop IN while one is open.
    pub fn episode(&mut self) -> Result<Khid> {
        let _ = self.add_type("Episode")?;
        self.mark_content("Episode", "pos");
        let id = self.add_vertex(HashMap::new(), Some("Episode"))?;
        self.open_episode = Some(id);
        Ok(id)
    }

    pub fn open_episode(&self) -> Option<Khid> {
        self.open_episode
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
}
