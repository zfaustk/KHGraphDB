impl Store {
    /// Open an existing copy as read-only. Does not
    /// copy. catch_up still pulls.
    pub fn attach(dir: &Path, name: &str) -> io::Result<Store> {
        let mut s = Store::open(dir, name, 0)?;
        s.read_only = true;
        Ok(s)
    }

    /// A copy of the log. Read-only until promote.
    pub fn tail(dir: &Path, from: &Path, name: &str) -> io::Result<Store> {
        fs::create_dir_all(dir)?;
        fs::copy(from.join("log"), dir.join("log"))?;
        if from.join("beat").exists() {
            let _ = fs::copy(from.join("beat"), dir.join("beat"));
        }
        if from.join("meta").exists() {
            let _ = fs::copy(from.join("meta"), dir.join("meta"));
        }
        let _ = super::blob::copy_all(from, dir);
        let _ = super::vec::copy_all(from, dir);
        let mut s = Store::open(dir, name, 0)?;
        s.read_only = true;
        Ok(s)
    }

    /// Pull to match `from`. Same generation: append
    /// new bytes. New generation: replace the file.
    pub fn catch_up(&mut self, from: &Path) -> io::Result<()> {
        if !self.read_only {
            return Err(io::Error::new(io::ErrorKind::Other, "not a replica"));
        }
        if self.open_tx.is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "in a transaction"));
        }
        let src = from.join("log");
        let src_pos = pos_of(&src)?;
        let dst_pos = self.pos()?;
        if src_pos == dst_pos {
            if from.join("beat").exists() {
                let _ = fs::copy(from.join("beat"), self.dir.join("beat"));
            }
            let _ = super::meta::catch_up(&self.dir, from);
            let _ = super::blob::copy_all(from, &self.dir);
            let _ = super::vec::copy_all(from, &self.dir);
            return Ok(());
        }
        if src_pos.generation() != dst_pos.generation()
            || src_pos.offset() < dst_pos.offset() {
            let tmp = self.dir.join("log.new");
            fs::copy(&src, &tmp)?;
            fs::rename(&tmp, self.dir.join("log"))?;
        } else {
            let mut f = File::open(&src)?;
            f.seek(SeekFrom::Start(dst_pos.offset()))?;
            self.log.seek(SeekFrom::End(0))?;
            io::copy(&mut f, &mut self.log)?;
            self.log.sync_data()?;
        }
        if from.join("beat").exists() {
            let _ = fs::copy(from.join("beat"), self.dir.join("beat"));
        }
        let _ = super::meta::catch_up(&self.dir, from);
        let _ = super::blob::copy_all(from, &self.dir);
        let _ = super::vec::copy_all(from, &self.dir);
        self.reopen_replica()
    }

    /// Replica: catch_up until `need` is honored.
    /// Primary always honors. Fails if still behind.
    pub fn honor(&mut self, from: &Path, need: Pos) -> io::Result<()> {
        if !self.read_only {
            return Ok(());
        }
        if self.pos()?.honors(need) {
            return Ok(());
        }
        self.catch_up(from)?;
        if self.pos()?.honors(need) {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "bookmark not honored"))
        }
    }

    fn reopen_replica(&mut self) -> io::Result<()> {
        let name = self.g.khid().to_string();
        let dir = self.dir.clone();
        let mut s = Store::open(&dir, &name, 0)?;
        s.read_only = true;
        let _ = super::meta::Meta::rebuild(&s.dir, &s.g);
        *self = s;
        Ok(())
    }

    /// Pull from a primary over TCP. Same Pos rules
    /// as catch_up. Does not wait on commit.
    pub fn follow(&mut self, addr: SocketAddr) -> io::Result<Pos> {
        if !self.read_only {
            return Err(io::Error::new(io::ErrorKind::Other, "not a replica"));
        }
        if self.open_tx.is_some() {
            return Err(io::Error::new(io::ErrorKind::Other, "in a transaction"));
        }
        let have = self.pos()?;
        let dest = self.dir.join("log");
        let p = super::wire::pull(addr, have, &dest)?;
        self.reopen_replica()?;
        Ok(p)
    }

    /// This copy is now home. Split brain is the deal.
    pub fn promote(&mut self) {
        self.read_only = false;
        let _ = self.take_lease();
    }

    fn take_lease(&self) -> io::Result<()> {
        let until = unix() + 3600;
        let mut f = File::create(self.dir.join("lease"))?;
        write!(f, "{} {}\n", self.token, until)?;
        f.sync_data()
    }

    fn hold_lease(&self) -> bool {
        match read_lease(&self.dir) {
            Some((tok, until)) => {
                if tok == self.token {
                    true
                } else if unix() >= until {
                    let _ = self.take_lease();
                    true
                } else {
                    false
                }
            }
            None => {
                let _ = self.take_lease();
                true
            }
        }
    }
}

