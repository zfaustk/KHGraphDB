use super::super::error::{Error, Result};
use super::super::graph::Graph;
use super::{Expr, NodePat, Pattern, QueryResult, RelPat, RetItem, Val};
use super::walk::{distinct_rows, exec_create, exec_delete, exec_explain, exec_match, exec_merge, exec_remove, exec_set, exec_unwind, filter_where, limit_rows, order_by, project, skip_rows};

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
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
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
        if c == '!' && self.i + 1 < self.s.len() && self.s[self.i + 1] == '=' {
            self.i += 2;
            return Ok(tok(TokenKind::Ne, "!="));
        }
        if c == '<' {
            if self.i + 1 < self.s.len() && self.s[self.i + 1] == '-' {
                self.i += 2;
                return Ok(tok(TokenKind::LArrow, "<-"));
            }
            if self.i + 1 < self.s.len() && self.s[self.i + 1] == '=' {
                self.i += 2;
                return Ok(tok(TokenKind::Le, "<="));
            }
            if self.i + 1 < self.s.len() && self.s[self.i + 1] == '>' {
                self.i += 2;
                return Ok(tok(TokenKind::Ne, "<>"));
            }
            self.i += 1;
            return Ok(tok(TokenKind::Lt, "<"));
        }
        if c == '>' {
            if self.i + 1 < self.s.len() && self.s[self.i + 1] == '=' {
                self.i += 2;
                return Ok(tok(TokenKind::Ge, ">="));
            }
            self.i += 1;
            return Ok(tok(TokenKind::Gt, ">"));
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
        Err(Error::near("bad char", &c.to_string()))
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
        Err(Error::near("unterminated string", "'"))
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


const STAR_CAP: usize = 16;

pub(crate) fn run_inner(g: &mut Graph, text: &str) -> Result<QueryResult> {
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
    fn err_here(&self, msg: &str) -> Error {
        Error::near(msg, &self.text())
    }
    fn expect(&mut self, k: TokenKind) -> Result<()> {
        if self.kind() != k {
            return Err(self.err_here("unexpected token"));
        }
        self.next();
        Ok(())
    }
    fn expect_ident(&mut self) -> Result<String> {
        if self.kind() != TokenKind::Ident {
            return Err(self.err_here("expected identifier"));
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
        if self.ident_is("EXPLAIN") {
            self.next();
            if self.ident_is("OPTIONAL") {
                self.next();
                if !self.ident_is("MATCH") {
                    return Err(self.err_here("expected MATCH"));
                }
                self.next();
            } else if self.ident_is("MATCH") {
                self.next();
            } else {
                return Err(self.err_here("EXPLAIN expected MATCH"));
            }
            let pat = self.parse_match()?;
            return exec_explain(g, &pat);
        }
        while self.kind() != TokenKind::Eof {
            if self.ident_is("OPTIONAL") {
                self.next();
                if !self.ident_is("MATCH") {
                    return Err(self.err_here("expected MATCH"));
                }
                self.next();
                let mut pat = self.parse_match()?;
                pat.optional = true;
                last = Some(exec_match(g, &pat, last));
            } else if self.ident_is("MATCH") {
                self.next();
                let pat = self.parse_match()?;
                last = Some(exec_match(g, &pat, last));
            } else if self.ident_is("WHERE") {
                self.next();
                let preds = self.parse_where()?;
                match last {
                    Some(src) => last = Some(filter_where(g, src, &preds)),
                    None => return Err(self.err_here("WHERE without MATCH")),
                }
            } else if self.ident_is("UNWIND") {
                self.next();
                let (col, lits) = self.parse_unwind_src()?;
                if !self.ident_is("AS") {
                    return Err(self.err_here("expected AS"));
                }
                self.next();
                let alias = self.expect_ident()?;
                last = Some(exec_unwind(last, col, lits, alias));
            } else if self.ident_is("WITH") {
                self.next();
                let distinct = if self.ident_is("DISTINCT") {
                    self.next();
                    true
                } else {
                    false
                };
                let cols = self.parse_return()?;
                match last {
                    Some(src) => {
                        let mut r = project(&src, &cols);
                        if distinct {
                            r = distinct_rows(r);
                        }
                        last = Some(self.parse_return_tail(g, r)?);
                    }
                    None => return Err(self.err_here("WITH without MATCH")),
                }
            } else if self.ident_is("RETURN") {
                self.next();
                let distinct = if self.ident_is("DISTINCT") {
                    self.next();
                    true
                } else {
                    false
                };
                let cols = self.parse_return()?;
                match last {
                    Some(src) => {
                        let mut r = project(&src, &cols);
                        if distinct {
                            r = distinct_rows(r);
                        }
                        last = Some(self.parse_return_tail(g, r)?);
                        break;
                    }
                    None => return Err(self.err_here("RETURN without MATCH")),
                }
            } else if self.ident_is("MERGE") {
                self.next();
                let mut pat = self.parse_pattern()?;
                self.parse_merge_tail(&mut pat)?;
                last = Some(exec_merge(g, &pat)?);
            } else if self.ident_is("CREATE") {
                self.next();
                loop {
                    let pat = self.parse_pattern()?;
                    last = Some(exec_create(g, &pat, last.as_ref())?);
                    if self.kind() == TokenKind::Comma {
                        self.next();
                    } else {
                        break;
                    }
                }
            } else if self.ident_is("SET") {
                self.next();
                let items = self.parse_set()?;
                match last {
                    Some(src) => last = Some(exec_set(g, src, &items)?),
                    None => return Err(self.err_here("SET without MATCH")),
                }
            } else if self.ident_is("REMOVE") {
                self.next();
                let items = self.parse_remove()?;
                match last {
                    Some(src) => last = Some(exec_remove(g, src, &items)?),
                    None => return Err(self.err_here("REMOVE without MATCH")),
                }
            } else if self.ident_is("DETACH") {
                self.next();
                if !self.ident_is("DELETE") {
                    return Err(self.err_here("expected DELETE"));
                }
                self.next();
                let names = self.parse_names()?;
                match last {
                    Some(src) => last = Some(exec_delete(g, src, &names, true)?),
                    None => return Err(self.err_here("DELETE without MATCH")),
                }
            } else if self.ident_is("DELETE") {
                self.next();
                let names = self.parse_names()?;
                match last {
                    Some(src) => last = Some(exec_delete(g, src, &names, false)?),
                    None => return Err(self.err_here("DELETE without MATCH")),
                }
            } else {
                break;
            }
        }
        match last {
            Some(r) => Ok(r),
            None => Err(self.err_here("expected MATCH")),
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
            on_create: Vec::new(),
            on_match: Vec::new(),
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

    fn parse_return_tail(&mut self, g: &Graph, mut r: QueryResult) -> Result<QueryResult> {
        if self.ident_is("ORDER") {
            self.next();
            if !self.ident_is("BY") {
                return Err(self.err_here("expected BY"));
            }
            self.next();
            let keys = self.parse_order()?;
            r = order_by(g, r, &keys);
        }
        if self.ident_is("SKIP") {
            self.next();
            if self.kind() != TokenKind::Number {
                return Err(self.err_here("expected number"));
            }
            let n = parse_usize(&self.text())?;
            self.next();
            r = skip_rows(r, n);
        }
        if self.ident_is("LIMIT") {
            self.next();
            if self.kind() != TokenKind::Number {
                return Err(self.err_here("expected number"));
            }
            let n = parse_usize(&self.text())?;
            self.next();
            r = limit_rows(r, n);
        }
        Ok(r)
    }

    fn parse_order(&mut self) -> Result<Vec<(String, Option<String>, bool)>> {
        let mut keys = Vec::new();
        keys.push(self.parse_order_key()?);
        while self.kind() == TokenKind::Comma {
            self.next();
            keys.push(self.parse_order_key()?);
        }
        Ok(keys)
    }

    fn parse_order_key(&mut self) -> Result<(String, Option<String>, bool)> {
        let var = self.expect_ident()?;
        let mut key = None;
        if self.kind() == TokenKind::Dot {
            self.next();
            key = Some(self.expect_ident()?);
        }
        let mut desc = false;
        if self.ident_is("DESC") {
            self.next();
            desc = true;
        } else if self.ident_is("ASC") {
            self.next();
        }
        Ok((var, key, desc))
    }

    fn parse_merge_tail(&mut self, pat: &mut Pattern) -> Result<()> {
        loop {
            if !self.ident_is("ON") {
                return Ok(());
            }
            self.next();
            if self.ident_is("CREATE") {
                self.next();
                if !self.ident_is("SET") {
                    return Err(self.err_here("expected SET"));
                }
                self.next();
                pat.on_create = self.parse_set()?;
            } else if self.ident_is("MATCH") {
                self.next();
                if !self.ident_is("SET") {
                    return Err(self.err_here("expected SET"));
                }
                self.next();
                pat.on_match = self.parse_set()?;
            } else {
                return Err(self.err_here("expected CREATE or MATCH"));
            }
        }
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
            var: None,
            type_name: None,
            type_id: None,
            dir: 1,
            min: 1,
            max: 1,
            star: false,
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
                r.var = Some(self.text());
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
        r.star = true;
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
            return Err(self.err_here("bad length"));
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
                _ => return Err(self.err_here("bad property value")),
            }
            if self.kind() == TokenKind::Comma {
                self.next();
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(d)
    }

    fn parse_where(&mut self) -> Result<Expr> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut e = self.parse_and()?;
        while self.ident_is("OR") {
            self.next();
            let r = self.parse_and()?;
            e = Expr::Or(Box::new(e), Box::new(r));
        }
        Ok(e)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut e = self.parse_not()?;
        while self.ident_is("AND") {
            self.next();
            let r = self.parse_not()?;
            e = Expr::And(Box::new(e), Box::new(r));
        }
        Ok(e)
    }

    fn parse_not(&mut self) -> Result<Expr> {
        if self.ident_is("NOT") {
            self.next();
            let inner = self.parse_not()?;
            return Ok(Expr::Not(Box::new(inner)));
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Expr> {
        if self.kind() == TokenKind::LParen {
            self.next();
            let e = self.parse_or()?;
            self.expect(TokenKind::RParen)?;
            return Ok(e);
        }
        self.parse_pred()
    }

    fn parse_pred(&mut self) -> Result<Expr> {
        let var = self.expect_ident()?;
        self.expect(TokenKind::Dot)?;
        let key = self.expect_ident()?;
        if self.ident_is("IN") {
            self.next();
            self.expect(TokenKind::LBrack)?;
            let mut vals = Vec::new();
            while self.kind() != TokenKind::RBrack && self.kind() != TokenKind::Eof {
                match self.kind() {
                    TokenKind::String | TokenKind::Number | TokenKind::Ident => {
                        vals.push(self.text());
                        self.next();
                    }
                    _ => return Err(self.err_here("bad IN value")),
                }
                if self.kind() == TokenKind::Comma {
                    self.next();
                }
            }
            self.expect(TokenKind::RBrack)?;
            return Ok(Expr::In(var, key, vals));
        }
        let op = match self.kind() {
            TokenKind::Eq => 0,
            TokenKind::Lt => -1,
            TokenKind::Gt => 1,
            TokenKind::Le => -2,
            TokenKind::Ge => 2,
            TokenKind::Ne => 3,
            _ => return Err(self.err_here("bad WHERE op")),
        };
        self.next();
        match self.kind() {
            TokenKind::String | TokenKind::Number | TokenKind::Ident => {
                let val = self.text();
                self.next();
                if op == 0 {
                    Ok(Expr::Eq(var, key, val))
                } else {
                    Ok(Expr::Cmp(var, key, op, val))
                }
            }
            _ => Err(self.err_here("bad WHERE value")),
        }
    }

    fn parse_return(&mut self) -> Result<Vec<RetItem>> {
        let mut cols = Vec::new();
        cols.push(self.parse_return_item()?);
        while self.kind() == TokenKind::Comma {
            self.next();
            cols.push(self.parse_return_item()?);
        }
        Ok(cols)
    }

    fn parse_return_item(&mut self) -> Result<RetItem> {
        if self.ident_is("COUNT") {
            self.next();
            self.expect(TokenKind::LParen)?;
            let name = if self.kind() == TokenKind::Star {
                self.next();
                "*".to_string()
            } else {
                self.expect_ident()?
            };
            self.expect(TokenKind::RParen)?;
            let alias = if self.ident_is("AS") {
                self.next();
                self.expect_ident()?
            } else {
                "count".to_string()
            };
            return Ok(RetItem {
                kind: 1,
                name: name,
                alias: alias,
            });
        }
        if self.ident_is("COLLECT") {
            self.next();
            self.expect(TokenKind::LParen)?;
            let name = self.expect_ident()?;
            self.expect(TokenKind::RParen)?;
            let alias = if self.ident_is("AS") {
                self.next();
                self.expect_ident()?
            } else {
                "collect".to_string()
            };
            return Ok(RetItem {
                kind: 2,
                name: name,
                alias: alias,
            });
        }
        if self.ident_is("LENGTH") || self.ident_is("NODES") || self.ident_is("RELATIONSHIPS") {
            let fname = self.text().to_lowercase();
            self.next();
            self.expect(TokenKind::LParen)?;
            let name = self.expect_ident()?;
            self.expect(TokenKind::RParen)?;
            let kind = if fname == "length" {
                3
            } else if fname == "nodes" {
                4
            } else {
                5
            };
            let alias = if self.ident_is("AS") {
                self.next();
                self.expect_ident()?
            } else {
                fname
            };
            return Ok(RetItem {
                kind: kind,
                name: name,
                alias: alias,
            });
        }
        let name = self.expect_ident()?;
        if self.ident_is("AS") {
            self.next();
            let alias = self.expect_ident()?;
            Ok(RetItem {
                kind: 0,
                name: name,
                alias: alias,
            })
        } else {
            Ok(RetItem {
                kind: 0,
                name: name.clone(),
                alias: name,
            })
        }
    }

    fn parse_unwind_src(&mut self) -> Result<(Option<String>, Vec<Val>)> {
        if self.kind() == TokenKind::LBrack {
            self.next();
            let mut lits = Vec::new();
            while self.kind() != TokenKind::RBrack && self.kind() != TokenKind::Eof {
                match self.kind() {
                    TokenKind::String | TokenKind::Number | TokenKind::Ident => {
                        lits.push(Val::Id(self.text()));
                        self.next();
                    }
                    _ => return Err(self.err_here("bad UNWIND value")),
                }
                if self.kind() == TokenKind::Comma {
                    self.next();
                }
            }
            self.expect(TokenKind::RBrack)?;
            return Ok((None, lits));
        }
        let col = self.expect_ident()?;
        Ok((Some(col), Vec::new()))
    }

    fn parse_set(&mut self) -> Result<Vec<(String, String, String)>> {
        let mut items = Vec::new();
        items.push(self.parse_set_item()?);
        while self.kind() == TokenKind::Comma {
            self.next();
            items.push(self.parse_set_item()?);
        }
        Ok(items)
    }

    fn parse_set_item(&mut self) -> Result<(String, String, String)> {
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
            _ => Err(self.err_here("bad SET value")),
        }
    }

    fn parse_remove(&mut self) -> Result<Vec<(String, String)>> {
        let mut items = Vec::new();
        items.push(self.parse_remove_item()?);
        while self.kind() == TokenKind::Comma {
            self.next();
            items.push(self.parse_remove_item()?);
        }
        Ok(items)
    }

    fn parse_remove_item(&mut self) -> Result<(String, String)> {
        let var = self.expect_ident()?;
        self.expect(TokenKind::Dot)?;
        let key = self.expect_ident()?;
        Ok((var, key))
    }

    fn parse_names(&mut self) -> Result<Vec<String>> {
        let mut names = Vec::new();
        names.push(self.expect_ident()?);
        while self.kind() == TokenKind::Comma {
            self.next();
            names.push(self.expect_ident()?);
        }
        Ok(names)
    }
}

