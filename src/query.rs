use super::error::{Error, Result};
use super::graph::Graph;

#[derive(Clone, Copy, PartialEq)]
enum TokenKind {
    Eof,
    Ident,
    String,
    Number,
    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Dot,
    Eq,
    Dash,
    Arrow,
    LArrow,
    Star,
}

struct Token {
    kind: TokenKind,
    text: String,
}

struct Lexer {
    s: Vec<char>,
    i: usize,
}

impl Lexer {
    fn new(text: &str) -> Lexer {
        Lexer {
            s: text.chars().collect(),
            i: 0,
        }
    }

    fn skip(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_whitespace() {
            self.i += 1;
        }
    }

    fn next(&mut self) -> Result<Token> {
        self.skip();
        if self.i >= self.s.len() {
            return Ok(Token {
                kind: TokenKind::Eof,
                text: String::new(),
            });
        }
        let c = self.s[self.i];
        if c == '(' {
            self.i += 1;
            return Ok(tok(TokenKind::LParen, "("));
        }
        if c == ')' {
            self.i += 1;
            return Ok(tok(TokenKind::RParen, ")"));
        }
        if c == '[' {
            self.i += 1;
            return Ok(tok(TokenKind::LBrack, "["));
        }
        if c == ']' {
            self.i += 1;
            return Ok(tok(TokenKind::RBrack, "]"));
        }
        if c == '{' {
            self.i += 1;
            return Ok(tok(TokenKind::LBrace, "{"));
        }
        if c == '}' {
            self.i += 1;
            return Ok(tok(TokenKind::RBrace, "}"));
        }
        if c == ':' {
            self.i += 1;
            return Ok(tok(TokenKind::Colon, ":"));
        }
        if c == ',' {
            self.i += 1;
            return Ok(tok(TokenKind::Comma, ","));
        }
        if c == '.' {
            self.i += 1;
            return Ok(tok(TokenKind::Dot, "."));
        }
        if c == '=' {
            self.i += 1;
            return Ok(tok(TokenKind::Eq, "="));
        }
        if c == '<' && self.i + 1 < self.s.len() && self.s[self.i + 1] == '-' {
            self.i += 2;
            return Ok(tok(TokenKind::LArrow, "<-"));
        }
        if c == '-' && self.i + 1 < self.s.len() && self.s[self.i + 1] == '>' {
            self.i += 2;
            return Ok(tok(TokenKind::Arrow, "->"));
        }
        if c == '*' {
            self.i += 1;
            return Ok(tok(TokenKind::Star, "*"));
        }
        if c == '-' {
            self.i += 1;
            return Ok(tok(TokenKind::Dash, "-"));
        }
        if c == '"' || c == '\'' {
            return self.read_string(c);
        }
        if c.is_digit(10) {
            return self.read_number();
        }
        if c.is_alphabetic() || c == '_' {
            return self.read_ident();
        }
        Err(Error::new("bad char"))
    }

    fn read_ident(&mut self) -> Result<Token> {
        let start = self.i;
        self.i += 1;
        while self.i < self.s.len() {
            let c = self.s[self.i];
            if c.is_alphanumeric() || c == '_' {
                self.i += 1;
            } else {
                break;
            }
        }
        let t: String = self.s[start..self.i].iter().cloned().collect();
        Ok(tok(TokenKind::Ident, &t))
    }

    fn read_number(&mut self) -> Result<Token> {
        let start = self.i;
        while self.i < self.s.len() && self.s[self.i].is_digit(10) {
            self.i += 1;
        }
        let t: String = self.s[start..self.i].iter().cloned().collect();
        Ok(tok(TokenKind::Number, &t))
    }

    fn read_string(&mut self, q: char) -> Result<Token> {
        self.i += 1;
        let mut out = String::new();
        while self.i < self.s.len() {
            let c = self.s[self.i];
            self.i += 1;
            if c == q {
                return Ok(tok(TokenKind::String, &out));
            }
            if c == '\\' && self.i < self.s.len() {
                out.push(self.s[self.i]);
                self.i += 1;
                continue;
            }
            out.push(c);
        }
        Err(Error::new("unterminated string"))
    }
}

fn tok(kind: TokenKind, text: &str) -> Token {
    Token {
        kind: kind,
        text: text.to_string(),
    }
}

fn parse_usize(s: &str) -> Result<usize> {
    match s.parse::<usize>() {
        Ok(n) => Ok(n),
        Err(_) => Err(Error::new("bad length")),
    }
}

#[derive(Clone)]
struct NodePat {
    var: Option<String>,
    type_name: Option<String>,
    type_id: Option<String>,
    props: Vec<(String, String)>,
}

const STAR_CAP: usize = 16;

#[derive(Clone)]
struct RelPat {
    type_name: Option<String>,
    type_id: Option<String>,
    dir: i32, // 1 out, -1 in, 0 both
    min: usize,
    max: usize,
}

#[derive(Clone)]
struct Pattern {
    nodes: Vec<NodePat>,
    rels: Vec<RelPat>,
    optional: bool,
    path_var: Option<String>,
    shortest: bool,
}

/// A bound value. An id is a KHID. A path is
/// node, edge, node, ... The vertices stay put.
#[derive(Clone)]
pub enum Val {
    Id(String),
    Path(Vec<String>),
}

impl Val {
    pub fn as_id(&self) -> Option<&str> {
        match *self {
            Val::Id(ref s) => Some(&s[..]),
            Val::Path(_) => None,
        }
    }

    pub fn as_path(&self) -> Option<&[String]> {
        match *self {
            Val::Path(ref p) => Some(&p[..]),
            Val::Id(_) => None,
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
    match run_inner(g, text) {
        Ok(r) => r,
        Err(e) => QueryResult::fail(e.message()),
    }
}

fn run_inner(g: &mut Graph, text: &str) -> Result<QueryResult> {
    let mut lx = Lexer::new(text);
    let mut toks = Vec::new();
    loop {
        let t = lx.next()?;
        let eof = t.kind == TokenKind::Eof;
        toks.push(t);
        if eof {
            break;
        }
    }
    let mut p = Parser {
        toks: toks,
        i: 0,
    };
    p.exec(g)
}

struct Parser {
    toks: Vec<Token>,
    i: usize,
}

impl Parser {
    fn kind(&self) -> TokenKind {
        self.toks[self.i].kind
    }
    fn text(&self) -> String {
        self.toks[self.i].text.clone()
    }
    fn next(&mut self) {
        if self.i + 1 < self.toks.len() {
            self.i += 1;
        }
    }
    fn ident_is(&self, w: &str) -> bool {
        self.kind() == TokenKind::Ident && self.toks[self.i].text.to_lowercase() == w.to_lowercase()
    }
    fn expect(&mut self, k: TokenKind) -> Result<()> {
        if self.kind() != k {
            return Err(Error::new("unexpected token"));
        }
        self.next();
        Ok(())
    }
    fn expect_ident(&mut self) -> Result<String> {
        if self.kind() != TokenKind::Ident {
            return Err(Error::new("expected identifier"));
        }
        let s = self.text();
        self.next();
        Ok(s)
    }

    fn parse_path_eq(&mut self) -> Result<Option<String>> {
        if self.kind() == TokenKind::Ident && self.i + 1 < self.toks.len() {
            if self.toks[self.i + 1].kind == TokenKind::Eq {
                let name = self.text();
                self.next();
                self.next();
                return Ok(Some(name));
            }
        }
        Ok(None)
    }

    fn exec(&mut self, g: &mut Graph) -> Result<QueryResult> {
        let mut last: Option<QueryResult> = None;
        while self.kind() != TokenKind::Eof {
            if self.ident_is("OPTIONAL") {
                self.next();
                if !self.ident_is("MATCH") {
                    return Err(Error::new("expected MATCH"));
                }
                self.next();
                let mut pat = self.parse_match()?;
                pat.optional = true;
                last = Some(exec_pattern(g, &pat));
            } else if self.ident_is("MATCH") {
                self.next();
                let pat = self.parse_match()?;
                last = Some(exec_pattern(g, &pat));
            } else if self.ident_is("WHERE") {
                self.next();
                let preds = self.parse_where()?;
                match last {
                    Some(src) => last = Some(filter_where(g, src, &preds)),
                    None => return Err(Error::new("WHERE without MATCH")),
                }
            } else if self.ident_is("RETURN") {
                self.next();
                let cols = self.parse_return()?;
                match last {
                    Some(src) => {
                        last = Some(project(&src, &cols));
                        break;
                    }
                    None => return Err(Error::new("RETURN without MATCH")),
                }
            } else if self.ident_is("MERGE") {
                self.next();
                let pat = self.parse_pattern()?;
                last = Some(exec_merge(g, &pat)?);
            } else {
                break;
            }
        }
        match last {
            Some(r) => Ok(r),
            None => Err(Error::new("expected MATCH")),
        }
    }

    fn parse_match(&mut self) -> Result<Pattern> {
        let path_var = self.parse_path_eq()?;
        let shortest = self.ident_is("shortestpath");
        if shortest {
            self.next();
            self.expect(TokenKind::LParen)?;
        }
        let mut pat = self.parse_pattern()?;
        if shortest {
            self.expect(TokenKind::RParen)?;
            pat.shortest = true;
        }
        pat.path_var = path_var;
        Ok(pat)
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        let mut pat = Pattern {
            nodes: Vec::new(),
            rels: Vec::new(),
            optional: false,
            path_var: None,
            shortest: false,
        };
        pat.nodes.push(self.parse_node()?);
        loop {
            match self.kind() {
                TokenKind::Dash | TokenKind::LArrow | TokenKind::Arrow => {
                    pat.rels.push(self.parse_rel()?);
                    pat.nodes.push(self.parse_node()?);
                }
                _ => break,
            }
        }
        Ok(pat)
    }

    fn parse_node(&mut self) -> Result<NodePat> {
        self.expect(TokenKind::LParen)?;
        let mut n = NodePat {
            var: None,
            type_name: None,
            type_id: None,
            props: Vec::new(),
        };
        if self.kind() == TokenKind::Ident {
            n.var = Some(self.text());
            self.next();
        }
        if self.kind() == TokenKind::Colon {
            self.next();
            n.type_name = Some(self.expect_ident()?);
        }
        if self.kind() == TokenKind::LBrace {
            n.props = self.parse_props()?;
        }
        self.expect(TokenKind::RParen)?;
        Ok(n)
    }

    fn parse_rel(&mut self) -> Result<RelPat> {
        let mut r = RelPat {
            type_name: None,
            type_id: None,
            dir: 1,
            min: 1,
            max: 1,
        };
        if self.kind() == TokenKind::LArrow {
            r.dir = -1;
            self.next();
        } else if self.kind() == TokenKind::Dash {
            self.next();
        } else if self.kind() == TokenKind::Arrow {
            r.dir = 1;
            self.next();
            return Ok(r);
        }
        if self.kind() == TokenKind::LBrack {
            self.next();
            if self.kind() == TokenKind::Ident {
                self.next();
            }
            if self.kind() == TokenKind::Colon {
                self.next();
                r.type_name = Some(self.expect_ident()?);
            }
            if self.kind() == TokenKind::Star {
                self.next();
                self.parse_star(&mut r)?;
            }
            self.expect(TokenKind::RBrack)?;
        }
        if self.kind() == TokenKind::Arrow {
            r.dir = 1;
            self.next();
        } else if self.kind() == TokenKind::Dash {
            if r.dir != -1 {
                r.dir = 0;
            }
            self.next();
        }
        Ok(r)
    }

    fn parse_star(&mut self, r: &mut RelPat) -> Result<()> {
        r.min = 1;
        r.max = STAR_CAP;
        if self.kind() == TokenKind::Number {
            let n = parse_usize(&self.text())?;
            self.next();
            if self.kind() == TokenKind::Dot {
                self.next();
                self.expect(TokenKind::Dot)?;
                r.min = n;
                if self.kind() == TokenKind::Number {
                    r.max = parse_usize(&self.text())?;
                    self.next();
                }
            } else {
                r.min = n;
                r.max = n;
            }
        } else if self.kind() == TokenKind::Dot {
            self.next();
            self.expect(TokenKind::Dot)?;
            if self.kind() == TokenKind::Number {
                r.max = parse_usize(&self.text())?;
                self.next();
            }
        }
        if r.min > r.max {
            return Err(Error::new("bad length"));
        }
        Ok(())
    }

    fn parse_props(&mut self) -> Result<Vec<(String, String)>> {
        self.expect(TokenKind::LBrace)?;
        let mut d = Vec::new();
        while self.kind() != TokenKind::RBrace && self.kind() != TokenKind::Eof {
            let key = self.expect_ident()?;
            self.expect(TokenKind::Colon)?;
            match self.kind() {
                TokenKind::String | TokenKind::Number | TokenKind::Ident => {
                    d.push((key, self.text()));
                    self.next();
                }
                _ => return Err(Error::new("bad property value")),
            }
            if self.kind() == TokenKind::Comma {
                self.next();
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(d)
    }

    fn parse_where(&mut self) -> Result<Vec<(String, String, String)>> {
        let mut list = Vec::new();
        list.push(self.parse_pred()?);
        while self.ident_is("AND") {
            self.next();
            list.push(self.parse_pred()?);
        }
        Ok(list)
    }

    fn parse_pred(&mut self) -> Result<(String, String, String)> {
        let var = self.expect_ident()?;
        self.expect(TokenKind::Dot)?;
        let key = self.expect_ident()?;
        self.expect(TokenKind::Eq)?;
        match self.kind() {
            TokenKind::String | TokenKind::Number | TokenKind::Ident => {
                let val = self.text();
                self.next();
                Ok((var, key, val))
            }
            _ => Err(Error::new("bad WHERE value")),
        }
    }

    fn parse_return(&mut self) -> Result<Vec<String>> {
        let mut cols = Vec::new();
        cols.push(self.expect_ident()?);
        while self.kind() == TokenKind::Comma {
            self.next();
            cols.push(self.expect_ident()?);
        }
        Ok(cols)
    }
}

fn columns_of(pat: &Pattern) -> Vec<String> {
    let mut cols = Vec::new();
    if let Some(ref p) = pat.path_var {
        cols.push(p.clone());
    }
    for (i, n) in pat.nodes.iter().enumerate() {
        cols.push(n.var.clone().unwrap_or(format!("n{}", i)));
    }
    cols
}

fn emit_row(pat: &Pattern, bind: &[Option<String>], trail: &[String], r: &mut QueryResult) {
    let mut row = Vec::new();
    if pat.path_var.is_some() {
        row.push(Some(Val::Path(trail.to_vec())));
    }
    for b in bind.iter() {
        match *b {
            Some(ref id) => row.push(Some(Val::Id(id.clone()))),
            None => row.push(None),
        }
    }
    r.rows.push(row);
}

fn exec_pattern(g: &Graph, pat: &Pattern) -> QueryResult {
    let mut pat = pat.clone();
    if let Some(msg) = resolve_types(g, &mut pat, true) {
        return QueryResult::fail(&msg);
    }
    let mut r = if pat.shortest {
        exec_shortest(g, &pat)
    } else if pat.rels.is_empty() {
        exec_nodes(g, &pat)
    } else {
        exec_chain(g, &pat)
    };
    if pat.optional && r.rows.is_empty() {
        let mut row = Vec::new();
        if r.columns.is_empty() {
            r.columns.push("n".to_string());
        }
        for _ in 0..r.columns.len() {
            row.push(None);
        }
        r.rows.push(row);
        r.message = "optional empty".to_string();
    }
    r
}

fn resolve_types(g: &Graph, pat: &mut Pattern, required: bool) -> Option<String> {
    for n in pat.nodes.iter_mut() {
        if let Some(ref tn) = n.type_name {
            match g.type_by_name(tn) {
                Some(t) => n.type_id = Some(t.khid().to_string()),
                None => {
                    if required {
                        return Some(format!("unknown type {}", tn));
                    }
                }
            }
        }
    }
    for r in pat.rels.iter_mut() {
        if let Some(ref tn) = r.type_name {
            match g.type_by_name(tn) {
                Some(t) => r.type_id = Some(t.khid().to_string()),
                None => {
                    if required {
                        return Some(format!("unknown type {}", tn));
                    }
                }
            }
        }
    }
    None
}

fn exec_shortest(g: &Graph, pat: &Pattern) -> QueryResult {
    if pat.rels.len() != 1 {
        return QueryResult::fail("shortestPath");
    }
    let starts = start_seeds(g, pat);
    let ends = seeds(g, &pat.nodes[1]);
    let rel = &pat.rels[0];
    let tid = match rel.type_id {
        Some(ref s) => Some(&s[..]),
        None => None,
    };
    let mut r = QueryResult::ok_msg("MATCH");
    r.columns = columns_of(pat);
    for s in starts.iter() {
        for t in ends.iter() {
            match super::algo::path_on(g, s, t, tid, rel.dir, rel.min, rel.max) {
                Some(path) => {
                    let bind = vec![Some(s.clone()), Some(t.clone())];
                    emit_row(pat, &bind, &path, &mut r);
                }
                None => {}
            }
        }
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn exec_nodes(g: &Graph, pat: &Pattern) -> QueryResult {
    let n = &pat.nodes[0];
    let found = seeds(g, n);
    let mut r = QueryResult::ok_msg("MATCH");
    r.columns = columns_of(pat);
    for id in found.iter() {
        let bind = vec![Some(id.clone())];
        let trail = vec![id.clone()];
        emit_row(pat, &bind, &trail, &mut r);
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn exec_chain(g: &Graph, pat: &Pattern) -> QueryResult {
    let seeds0 = start_seeds(g, pat);
    let mut r = QueryResult::ok_msg("MATCH");
    r.columns = columns_of(pat);
    for s in seeds0.iter() {
        let mut bind = vec![None; pat.nodes.len()];
        bind[0] = Some(s.clone());
        let mut seen_v = vec![s.clone()];
        let mut seen_e: Vec<String> = Vec::new();
        let mut trail = vec![s.clone()];
        walk_named(g,
                   pat,
                   0,
                   &mut bind,
                   &mut seen_v,
                   &mut seen_e,
                   &mut trail,
                   &mut r);
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn walk_named(g: &Graph,
              pat: &Pattern,
              node_i: usize,
              bind: &mut Vec<Option<String>>,
              seen_v: &mut Vec<String>,
              seen_e: &mut Vec<String>,
              trail: &mut Vec<String>,
              r: &mut QueryResult) {
    if node_i == pat.rels.len() {
        emit_row(pat, bind, trail, r);
        return;
    }
    let from = match bind[node_i] {
        Some(ref id) => id.clone(),
        None => return,
    };
    expand_rel(g,
               pat,
               node_i,
               &from,
               0,
               bind,
               seen_v,
               seen_e,
               trail,
               r);
}

fn contains_id(ids: &Vec<String>, id: &str) -> bool {
    for x in ids.iter() {
        if x == id {
            return true;
        }
    }
    false
}

fn expand_rel(g: &Graph,
              pat: &Pattern,
              rel_i: usize,
              u: &str,
              hops: usize,
              bind: &mut Vec<Option<String>>,
              seen_v: &mut Vec<String>,
              seen_e: &mut Vec<String>,
              trail: &mut Vec<String>,
              r: &mut QueryResult) {
    let rel = &pat.rels[rel_i];
    let next = &pat.nodes[rel_i + 1];
    if hops >= rel.min && hops <= rel.max && node_ok(g, u, next) {
        bind[rel_i + 1] = Some(u.to_string());
        walk_named(g, pat, rel_i + 1, bind, seen_v, seen_e, trail, r);
        bind[rel_i + 1] = None;
    }
    if hops >= rel.max {
        return;
    }
    let eids = edges_of(g, u, rel);
    for eid in eids.iter() {
        if contains_id(seen_e, eid) {
            continue;
        }
        let e = match g.edge(eid) {
            Some(e) => e,
            None => continue,
        };
        let v = if rel.dir < 0 {
            e.source().to_string()
        } else if rel.dir == 0 {
            if e.source() == u {
                e.target().to_string()
            } else {
                e.source().to_string()
            }
        } else {
            e.target().to_string()
        };
        if contains_id(seen_v, &v) {
            continue;
        }
        seen_e.push(eid.clone());
        seen_v.push(v.clone());
        trail.push(eid.clone());
        trail.push(v.clone());
        expand_rel(g,
                   pat,
                   rel_i,
                   &v,
                   hops + 1,
                   bind,
                   seen_v,
                   seen_e,
                   trail,
                   r);
        trail.pop();
        trail.pop();
        seen_v.pop();
        seen_e.pop();
    }
}

fn seeds(g: &Graph, n: &NodePat) -> Vec<String> {
    if n.type_name.is_some() && !n.props.is_empty() {
        let tn = n.type_name.as_ref().unwrap();
        let &(ref k, ref val) = &n.props[0];
        let found = g.find(tn, k, val);
        return found.into_iter().filter(|id| node_ok(g, id, n)).collect();
    }
    let src: Vec<String> = match n.type_id {
        Some(ref tid) => {
            match g.ty(tid) {
                Some(t) => t.vertices().iter().map(|s| s.clone()).collect(),
                None => Vec::new(),
            }
        }
        None => g.vertex_ids(),
    };
    src.into_iter().filter(|id| node_ok(g, id, n)).collect()
}

fn start_seeds(g: &Graph, pat: &Pattern) -> Vec<String> {
    let n0 = &pat.nodes[0];
    if n0.type_id.is_some() || !n0.props.is_empty() {
        return seeds(g, n0);
    }
    if !pat.rels.is_empty() {
        if let Some(ref tid) = pat.rels[0].type_id {
            return starts_from_type(g, tid, pat.rels[0].dir, n0);
        }
    }
    seeds(g, n0)
}

fn starts_from_type(g: &Graph, tid: &str, dir: i32, n0: &NodePat) -> Vec<String> {
    let eids: Vec<String> = match g.ty(tid) {
        Some(t) => t.edges().iter().map(|s| s.clone()).collect(),
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    for eid in eids.iter() {
        let e = match g.edge(eid) {
            Some(e) => e,
            None => continue,
        };
        if dir >= 0 {
            let s = e.source();
            if node_ok(g, s, n0) && !contains_id(&out, s) {
                out.push(s.to_string());
            }
        }
        if dir <= 0 {
            let s = e.target();
            if node_ok(g, s, n0) && !contains_id(&out, s) {
                out.push(s.to_string());
            }
        }
    }
    out
}

fn wears(g: &Graph, vid: &str, tid: &str) -> bool {
    match g.vertex(vid) {
        Some(v) => {
            for t in v.types().iter() {
                if t == tid {
                    return true;
                }
            }
            false
        }
        None => false,
    }
}

fn node_ok(g: &Graph, vid: &str, n: &NodePat) -> bool {
    if let Some(ref tid) = n.type_id {
        if !wears(g, vid, tid) {
            return false;
        }
    }
    for &(ref k, ref val) in n.props.iter() {
        match g.vertex(vid).and_then(|v| v.get(k).map(|s| s.to_string())) {
            Some(ref got) if got == val => {}
            _ => return false,
        }
    }
    true
}

fn edges_of(g: &Graph, vid: &str, rel: &RelPat) -> Vec<String> {
    let v = match g.vertex(vid) {
        Some(v) => v,
        None => return Vec::new(),
    };
    let mut ids = Vec::new();
    let src: Vec<String> = if rel.dir > 0 {
        v.outgoing().iter().map(|s| s.clone()).collect()
    } else if rel.dir < 0 {
        v.incoming().iter().map(|s| s.clone()).collect()
    } else {
        let mut both = v.outgoing().iter().map(|s| s.clone()).collect::<Vec<_>>();
        both.extend(v.incoming().iter().map(|s| s.clone()));
        both
    };
    for eid in src.iter() {
        if let Some(ref tid) = rel.type_id {
            match g.edge(eid).and_then(|e| e.type_id().map(|s| s.to_string())) {
                Some(ref et) if et == tid => {}
                _ => continue,
            }
        }
        ids.push(eid.clone());
    }
    ids
}

fn filter_where(g: &Graph, src: QueryResult, preds: &Vec<(String, String, String)>) -> QueryResult {
    let mut r = QueryResult::ok_msg("WHERE");
    r.columns = src.columns.clone();
    for row in src.rows.iter() {
        let mut ok = true;
        for &(ref var, ref key, ref val) in preds.iter() {
            let col = match src.columns.iter().position(|c| c == var) {
                Some(i) => i,
                None => {
                    ok = false;
                    break;
                }
            };
            let vid = match row.get(col).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
                Some(id) => id,
                None => {
                    ok = false;
                    break;
                }
            };
            match g.vertex(vid).and_then(|v| v.get(key).map(|s| s.to_string())) {
                Some(ref got) if got == val => {}
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            r.rows.push(row.clone());
        }
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn project(src: &QueryResult, cols: &Vec<String>) -> QueryResult {
    let mut r = QueryResult::ok_msg("RETURN");
    let mut map = Vec::new();
    for c in cols.iter() {
        r.columns.push(c.clone());
        map.push(src.columns.iter().position(|x| x == c));
    }
    for row in src.rows.iter() {
        let mut nr = Vec::new();
        for m in map.iter() {
            match *m {
                Some(i) => nr.push(row.get(i).cloned().unwrap_or(None)),
                None => nr.push(None),
            }
        }
        r.rows.push(nr);
    }
    r.message = format!("{} row", r.rows.len());
    r
}

fn exec_merge(g: &mut Graph, pat: &Pattern) -> Result<QueryResult> {
    let mut pat = pat.clone();
    resolve_types(g, &mut pat, false);
    for rel in pat.rels.iter() {
        if rel.min != 1 || rel.max != 1 {
            return Err(Error::new("MERGE length"));
        }
    }
    if pat.rels.is_empty() {
        return merge_node(g, &pat.nodes[0]);
    }
    let left = merge_node(g, &pat.nodes[0])?;
    let right = merge_node(g, &pat.nodes[1])?;
    let a = match left.rows.get(0).and_then(|r| r.get(0)).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
        Some(id) => id.to_string(),
        None => return Err(Error::new("MERGE nodes")),
    };
    let b = match right.rows.get(0).and_then(|r| r.get(0)).and_then(|x| x.as_ref()).and_then(|v| v.as_id()) {
        Some(id) => id.to_string(),
        None => return Err(Error::new("MERGE nodes")),
    };
    let rel = &pat.rels[0];
    let eids: Vec<String> = match g.vertex(&a) {
        Some(v) => v.outgoing().iter().map(|s| s.clone()).collect(),
        None => Vec::new(),
    };
    for eid in eids.iter() {
        if let Some(e) = g.edge(eid) {
            if e.target() == b {
                let ok_t = match rel.type_id {
                    Some(ref tid) => e.type_id() == Some(&tid[..]),
                    None => true,
                };
                if ok_t {
                    let mut r = QueryResult::ok_msg("exists");
                    r.rows.push(vec![Some(Val::Id(a)), Some(Val::Id(b))]);
                    return Ok(r);
                }
            }
        }
    }
    g.add_edge(&a, &b, rel.type_name.as_ref().map(|s| &s[..]))?;
    let mut r = QueryResult::ok_msg("created");
    r.rows.push(vec![Some(Val::Id(a)), Some(Val::Id(b))]);
    Ok(r)
}

fn merge_node(g: &mut Graph, n: &NodePat) -> Result<QueryResult> {
    let found = seeds(g, n);
    if !found.is_empty() {
        let mut r = QueryResult::ok_msg("exists");
        r.columns.push(n.var.clone().unwrap_or("n".to_string()));
        for id in found.iter() {
            r.rows.push(vec![Some(Val::Id(id.clone()))]);
        }
        return Ok(r);
    }
    let mut attrs = std::collections::HashMap::new();
    for &(ref k, ref v) in n.props.iter() {
        attrs.insert(k.clone(), v.clone());
    }
    let id = g.add_vertex(attrs, n.type_name.as_ref().map(|s| &s[..]))?;
    let mut r = QueryResult::ok_msg("created");
    r.columns.push(n.var.clone().unwrap_or("n".to_string()));
    r.rows.push(vec![Some(Val::Id(id))]);
    Ok(r)
}
