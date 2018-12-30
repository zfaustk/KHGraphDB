use std::ops::Index;

use super::graph::Graph;
use super::khid::Khid;
use super::prop::Prop;

mod parse;
mod walk;
mod op;
mod scan;

#[derive(Clone)]
struct NodePat {
    var: Option<String>,
    type_name: Option<String>,
    type_id: Option<Khid>,
    props: Vec<(String, Prop)>,
}


#[derive(Clone)]
struct RelPat {
    var: Option<String>,
    type_name: Option<String>,
    type_id: Option<Khid>,
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
    on_create: Vec<(String, String, Prop)>,
    on_match: Vec<(String, String, Prop)>,
    pred: Option<Expr>,
    project: bool,
    limit: Option<usize>,
}

#[derive(Clone)]
struct RetItem {
    kind: i32, // 0 col, 1 count, 2 collect, 3 length, 4 nodes, 5 rels, 6 prop
    name: String,
    alias: String,
    key: Option<String>,
}

#[derive(Clone)]
enum Expr {
    Eq(String, String, Prop),
    Cmp(String, String, i32, Prop),
    In(String, String, Vec<Prop>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
}

/// A walk. Interleaved node, edge, node. KHID only.
/// The vertices stay in the arena.
#[derive(Clone, PartialEq)]
pub struct Path {
    ids: Vec<Khid>,
}

impl Path {
    pub fn new(ids: Vec<Khid>) -> Path {
        Path { ids: ids }
    }

    /// The walk still speaks in print form. Parse each cell.
    pub fn parse_all(ids: &[String]) -> Path {
        let mut v = Vec::new();
        for s in ids.iter() {
            match Khid::parse(s) {
                Some(k) => v.push(k),
                None => {}
            }
        }
        Path { ids: v }
    }

    pub fn ids(&self) -> &[Khid] {
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

    pub fn nodes(&self) -> Vec<Khid> {
        let mut v = Vec::new();
        let mut i = 0;
        while i < self.ids.len() {
            v.push(self.ids[i]);
            i += 2;
        }
        v
    }

    pub fn edges(&self) -> Vec<Khid> {
        let mut v = Vec::new();
        let mut i = 1;
        while i < self.ids.len() {
            v.push(self.ids[i]);
            i += 2;
        }
        v
    }
}

impl Index<usize> for Path {
    type Output = Khid;
    fn index(&self, i: usize) -> &Khid {
        &self.ids[i]
    }
}

/// A bound value. An id is a KHID. A path is
/// node, edge, node, ... The vertices stay put.
/// A list holds other values. A name or a count
/// is a Prop, not an id.
#[derive(Clone, PartialEq)]
pub enum Val {
    Id(Khid),
    Path(Path),
    List(Vec<Val>),
    Prop(Prop),
}

impl Val {
    pub fn as_id(&self) -> Option<Khid> {
        match *self {
            Val::Id(k) => Some(k),
            Val::Path(_) | Val::List(_) | Val::Prop(_) => None,
        }
    }

    pub fn as_path(&self) -> Option<&Path> {
        match *self {
            Val::Path(ref p) => Some(p),
            Val::Id(_) | Val::List(_) | Val::Prop(_) => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Val]> {
        match *self {
            Val::List(ref v) => Some(&v[..]),
            Val::Id(_) | Val::Path(_) | Val::Prop(_) => None,
        }
    }

    pub fn as_prop(&self) -> Option<&Prop> {
        match *self {
            Val::Prop(ref p) => Some(p),
            Val::Id(_) | Val::Path(_) | Val::List(_) => None,
        }
    }
}

pub struct QueryResult {
    pub ok: bool,
    pub message: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Option<Val>>>,
    pub created: usize,
    pub deleted: usize,
}

impl QueryResult {
    fn fail(msg: &str) -> QueryResult {
        QueryResult {
            ok: false,
            message: msg.to_string(),
            columns: Vec::new(),
            rows: Vec::new(),
            created: 0,
            deleted: 0,
        }
    }
    fn ok_msg(msg: &str) -> QueryResult {
        QueryResult {
            ok: true,
            message: msg.to_string(),
            columns: Vec::new(),
            rows: Vec::new(),
            created: 0,
            deleted: 0,
        }
    }
}

pub fn run(g: &mut Graph, text: &str) -> QueryResult {
    match parse::run_inner(g, text) {
        Ok(r) => r,
        Err(e) => QueryResult::fail(e.message()),
    }
}

/// MATCH with $name bound to a Prop. The tag is kept.
pub fn run_with(g: &mut Graph, text: &str, params: std::collections::HashMap<String, Prop>) -> QueryResult {
    match parse::run_inner_params(g, text, &params) {
        Ok(r) => r,
        Err(e) => QueryResult::fail(e.message()),
    }
}
