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
    Param,
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
        if c == '$' {
            return self.read_param();
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
        if self.i < self.s.len() && self.s[self.i] == '.' {
            let nxt = self.i + 1;
            if nxt < self.s.len() && self.s[nxt].is_digit(10) {
                self.i += 1;
                while self.i < self.s.len() && self.s[self.i].is_digit(10) {
                    self.i += 1;
                }
            }
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

    fn read_param(&mut self) -> Result<Token> {
        self.i += 1;
        if self.i >= self.s.len() {
            return Err(Error::near("bad param", "$"));
        }
        let c = self.s[self.i];
        if !(c.is_alphabetic() || c == '_') {
            return Err(Error::near("bad param", "$"));
        }
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
        Ok(tok(TokenKind::Param, &t))
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


