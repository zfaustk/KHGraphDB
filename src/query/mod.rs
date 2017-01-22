use std::ops::Index;

use super::graph::Graph;

mod parse;
mod walk;

#[derive(Clone)]
struct NodePat {
    var: Option<String>,
    type_name: Option<String>,
    type_id: Option<String>,
    props: Vec<(String, String)>,
}


#[derive(Clone)]
struct RelPat {
    var: Option<String>,
    type_name: Option<String>,
    type_id: Option<String>,
    dir: i32, // 1 out, -1 in, 0 both
    min: usize,
    max: usize,
    star: bool,
}

#[derive(Clone)]
struct Pattern {
    nodes: Vec<NodePat>,
    rels: Vec<RelPat>,
    optional: bool,
    path_var: Option<String>,
    shortest: bool,
}

/// A walk. Interleaved node, edge, node. KHID only.
/// The vertices stay in the arena.
#[derive(Clone)]
pub struct Path {
    ids: Vec<String>,
}

impl Path {
    pub fn new(ids: Vec<String>) -> Path {
        Path { ids: ids }
    }

    pub fn ids(&self) -> &[String] {
        &self.ids
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn hops(&self) -> usize {
        if self.ids.is_empty() {
            0
        } else {
            self.ids.len() / 2
        }
    }

    pub fn nodes(&self) -> Vec<String> {
        let mut v = Vec::new();
        let mut i = 0;
        while i < self.ids.len() {
            v.push(self.ids[i].clone());
            i += 2;
        }
        v
    }

    pub fn edges(&self) -> Vec<String> {
        let mut v = Vec::new();
        let mut i = 1;
        while i < self.ids.len() {
            v.push(self.ids[i].clone());
            i += 2;
        }
        v
    }
}

impl Index<usize> for Path {
    type Output = String;
    fn index(&self, i: usize) -> &String {
        &self.ids[i]
    }
}

/// A bound value. An id is a KHID. A path is
/// node, edge, node, ... The vertices stay put.
/// A list holds other values.
#[derive(Clone)]
pub enum Val {
    Id(String),
    Path(Path),
    List(Vec<Val>),
}

impl Val {
    pub fn as_id(&self) -> Option<&str> {
        match *self {
            Val::Id(ref s) => Some(&s[..]),
            Val::Path(_) | Val::List(_) => None,
        }
    }

    pub fn as_path(&self) -> Option<&Path> {
        match *self {
            Val::Path(ref p) => Some(p),
            Val::Id(_) | Val::List(_) => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Val]> {
        match *self {
            Val::List(ref v) => Some(&v[..]),
            Val::Id(_) | Val::Path(_) => None,
        }
    }
}

pub struct QueryResult {
    pub ok: bool,
    pub message: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<Val>>>,
}

impl QueryResult {
    fn fail(msg: &str) -> QueryResult {
        QueryResult {
            ok: false,
            message: msg.to_string(),
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }
    fn ok_msg(msg: &str) -> QueryResult {
        QueryResult {
            ok: true,
            message: msg.to_string(),
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }
}

pub fn run(g: &mut Graph, text: &str) -> QueryResult {
    match parse::run_inner(g, text) {
        Ok(r) => r,
        Err(e) => QueryResult::fail(e.message()),
    }
}
