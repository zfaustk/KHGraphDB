impl Drop for Store {
    fn drop(&mut self) {
        if self.read_only {
            return;
        }
        if let Some((tok, _)) = read_lease(&self.dir) {
            if tok == self.token {
                let _ = fs::remove_file(self.dir.join("lease"));
            }
        }
    }
}

fn pos_of(path: &Path) -> io::Result<Pos> {
    let mut f = File::open(path)?;
    let len = f.metadata()?.len();
    let h = wal::head(&mut f)?;
    let gen = if h.generation == 0 { 1 } else { h.generation };
    Ok(Pos::new(gen, len))
}

fn unix() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => 0,
    }
}

fn new_token() -> u64 {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let n = SEQ.fetch_add(1, Ordering::SeqCst) as u64 + 1;
    unix() ^ (n << 16) ^ ((std::process::id() as u64) << 32)
}

fn sync_dir(dir: &Path) -> io::Result<()> {
    let f = File::open(dir)?;
    f.sync_data()
}

fn read_lease(dir: &Path) -> Option<(u64, u64)> {
    let s = match fs::read_to_string(dir.join("lease")) {
        Ok(s) => s,
        Err(_) => return None,
    };
    let mut it = s.split_whitespace();
    let tok = match it.next().and_then(|x| x.parse().ok()) {
        Some(n) => n,
        None => return None,
    };
    let until = match it.next().and_then(|x| x.parse().ok()) {
        Some(n) => n,
        None => return None,
    };
    Some((tok, until))
}

fn vertex_rec(tx: u64, g: &Graph, id: Khid, blob_of: &HashMap<(Khid, String), u64>) -> Rec {
    let types = g.type_names_of_vertex(id);
    let attrs = match g.vertex(id) {
        Some(v) => g.strip_content(id, v.attrs()),
        None => HashMap::new(),
    };
    let mut blobs = Vec::new();
    for (k, _) in g.content_of(id).iter() {
        if let Some(&s) = blob_of.get(&(id, k.clone())) {
            blobs.push((k.clone(), s));
        }
    }
    Rec::Vertex {
        tx: tx,
        id: id,
        types: types,
        attrs: attrs,
        blobs: blobs,
    }
}

fn recs_from_touches(tx: u64,
                     g: &Graph,
                     blob_of: &HashMap<(Khid, String), u64>,
                     vec_of: &HashMap<(Khid, String), u64>) -> Vec<Rec> {
    let mut recs = Vec::new();
    recs.push(Rec::Begin { tx: tx });
    for t in g.touches().iter() {
        match *t {
            Touch::Vertex(id) => {
                recs.push(vertex_rec(tx, g, id, blob_of));
            }
            Touch::Edge(id) => {
                let e = match g.edge(id) {
                    Some(e) => e,
                    None => continue,
                };
                let ty = g.edge_type_name(id).unwrap_or(String::new());
                let attrs = e.attrs().clone();
                if e.is_far() {
                    recs.push(Rec::FarEdge {
                        tx: tx,
                        id: id,
                        src: e.source(),
                        dst: e.far().unwrap_or(Addr::here(Khid::nil())),
                        ty: ty,
                        attrs: attrs,
                    });
                } else {
                    recs.push(Rec::Edge {
                        tx: tx,
                        id: id,
                        src: e.source(),
                        dst: e.target(),
                        ty: ty,
                        attrs: attrs,
                    });
                }
            }
            Touch::DropVertex(id) => {
                recs.push(Rec::DropVertex { tx: tx, id: id });
            }
            Touch::DropEdge(id) => {
                recs.push(Rec::DropEdge { tx: tx, id: id });
            }
            Touch::Index { ref type_name, ref key, unique } => {
                recs.push(Rec::Index {
                    tx: tx,
                    type_name: type_name.clone(),
                    key: key.clone(),
                    unique: unique,
                });
            }
            Touch::Content { ref type_name, ref key } => {
                recs.push(Rec::Content {
                    tx: tx,
                    type_name: type_name.clone(),
                    key: key.clone(),
                });
            }
            Touch::VecMark { ref type_name, ref key } => {
                recs.push(Rec::VecMark {
                    tx: tx,
                    type_name: type_name.clone(),
                    key: key.clone(),
                });
            }
            Touch::Emb { id, ref key } => {
                if let Some(&serial) = vec_of.get(&(id, key.clone())) {
                    recs.push(Rec::Emb {
                        tx: tx,
                        id: id,
                        key: key.clone(),
                        serial: serial,
                    });
                }
            }
        }
    }
    recs.push(Rec::Commit { tx: tx });
    recs
}

fn capture(tx: u64,
           g: &Graph,
           blob_of: &HashMap<(Khid, String), u64>,
           vec_of: &HashMap<(Khid, String), u64>) -> Vec<Rec> {
    let mut recs = Vec::new();
    recs.push(Rec::Begin { tx: tx });
    for &(tid, _) in g.all_types().iter() {
        if let Some(t) = g.ty(tid) {
            let name = t.name().to_string();
            for k in t.content_keys().iter() {
                recs.push(Rec::Content {
                    tx: tx,
                    type_name: name.clone(),
                    key: k.clone(),
                });
            }
            for k in t.vector_keys().iter() {
                recs.push(Rec::VecMark {
                    tx: tx,
                    type_name: name.clone(),
                    key: k.clone(),
                });
            }
        }
    }
    for id in g.vertex_ids().iter() {
        recs.push(vertex_rec(tx, g, *id, blob_of));
    }
    for &(id, src, dst, _) in g.all_edges().iter() {
        let e: &Edge = match g.edge(id) {
            Some(e) => e,
            None => continue,
        };
        let ty = g.edge_type_name(id).unwrap_or(String::new());
        let attrs = e.attrs().clone();
        if e.is_far() {
            recs.push(Rec::FarEdge {
                tx: tx,
                id: id,
                src: src,
                dst: e.far().unwrap_or(Addr::here(Khid::nil())),
                ty: ty,
                attrs: attrs,
            });
        } else {
            recs.push(Rec::Edge {
                tx: tx,
                id: id,
                src: src,
                dst: dst,
                ty: ty,
                attrs: attrs,
            });
        }
    }
    for &(ref tn, ref k, u) in g.index_specs().iter() {
        recs.push(Rec::Index {
            tx: tx,
            type_name: tn.clone(),
            key: k.clone(),
            unique: u,
        });
    }
    for (id, k, _) in g.embeddings().iter() {
        if let Some(&serial) = vec_of.get(&(*id, k.clone())) {
            recs.push(Rec::Emb {
                tx: tx,
                id: *id,
                key: k.clone(),
                serial: serial,
            });
        }
    }
    recs.push(Rec::Commit { tx: tx });
    recs
}

/// Open failed as a kernel error.
pub fn open_err(e: io::Error) -> Error {
    Error::new(&e.to_string())
}
