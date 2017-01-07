use super::super::error::{Error, Result};
use super::super::graph::Graph;
use super::{NodePat, Pattern, QueryResult, RelPat};
use super::walk::{exec_merge, exec_pattern, filter_where, project};

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


const STAR_CAP: usize = 16;

pub fn run_inner(g: &mut Graph, text: &str) -> Result<QueryResult> {
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
            var: None,
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

