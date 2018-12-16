//! A stdin shell. Dot commands for the catalog.
//! MATCH still takes one graph.

use std::env;
use std::fs::File;
use std::io::{self, Write, BufRead};
use std::collections::HashMap;
use khgraphdb::{Catalog, Graph, query, io as khio, Prop};
use khgraphdb::query::{QueryResult, Val};

extern "C" {
    fn isatty(fd: i32) -> i32;
}

fn interactive() -> bool {
    unsafe { isatty(0) != 0 }
}

struct Shell {
    cat: Catalog,
    cur: String,
    params: HashMap<String, Prop>,
    snap: Option<Graph>,
}

impl Shell {
    fn new() -> Shell {
        let mut cat = Catalog::new();
        match cat.create("g1") {
            Ok(_) => {}
            Err(_) => {}
        }
        Shell {
            cat: cat,
            cur: "g1".to_string(),
            params: HashMap::new(),
            snap: None,
        }
    }

    fn graph_mut(&mut self) -> Result<&mut Graph, String> {
        match self.cat.graph_mut(&self.cur) {
            Some(g) => Ok(g),
            None => Err(format!("no graph {}", self.cur)),
        }
    }

    fn load(&mut self, path: &str) -> Result<(), String> {
        let mut f = File::open(path).map_err(|e| e.to_string())?;
        let g = khio::read_graph(&mut f).map_err(|e| e.to_string())?;
        let name = self.cat.put(g);
        self.cur = name;
        Ok(())
    }

    fn save(&mut self, path: &str) -> Result<(), String> {
        let g = self.graph_mut()?;
        let mut f = File::create(path).map_err(|e| e.to_string())?;
        khio::write_graph(g, &mut f).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn one(&mut self, line: &str) -> bool {
        if line == ".quit" || line == ".exit" || line == ".q" {
            return false;
        }
        if line == ".help" {
            print_help();
            return true;
        }
        if line == ".graphs" {
            let mut names = self.cat.names();
            names.sort();
            for n in names.iter() {
                if n == &self.cur {
                    println!("* {}", n);
                } else {
                    println!("  {}", n);
                }
            }
            return true;
        }
        if line.starts_with(".use ") {
            if self.snap.is_some() {
                println!("in a transaction");
                return true;
            }
            let name = line[5..].trim();
            if self.cat.graph(name).is_none() {
                println!("no graph {}", name);
            } else {
                self.cur = name.to_string();
            }
            return true;
        }
        if line.starts_with(".create ") {
            let name = line[8..].trim();
            match self.cat.create(name) {
                Ok(_) => self.cur = name.to_string(),
                Err(e) => println!("{}", e),
            }
            return true;
        }
        if line.starts_with(".drop ") {
            let name = line[6..].trim();
            if name == self.cur {
                println!("current graph");
                return true;
            }
            if !self.cat.drop(name) {
                println!("no graph {}", name);
            }
            return true;
        }
        if line.starts_with(".load ") {
            match self.load(line[6..].trim()) {
                Ok(()) => println!("loaded {}", self.cur),
                Err(e) => println!("{}", e),
            }
            return true;
        }
        if line.starts_with(".save ") {
            match self.save(line[6..].trim()) {
                Ok(()) => {}
                Err(e) => println!("{}", e),
            }
            return true;
        }
        if line.starts_with(":param ") {
            let rest = line[7..].trim();
            match parse_param_line(rest) {
                Ok((k, v)) => {
                    println!("{} = {}", k, v);
                    self.params.insert(k, v);
                }
                Err(e) => println!("{}", e),
            }
            return true;
        }
        if line == ":params" {
            let mut keys: Vec<String> = self.params.keys().map(|s| s.clone()).collect();
            keys.sort();
            for k in keys.iter() {
                println!("${} = {}", k, self.params[k]);
            }
            return true;
        }
        if line == ":begin" {
            if self.snap.is_some() {
                println!("already in a transaction");
                return true;
            }
            match self.cat.graph_mut(&self.cur) {
                Some(g) => {
                    self.snap = Some(g.snapshot());
                    println!("begin");
                }
                None => println!("no graph {}", self.cur),
            }
            return true;
        }
        if line == ":commit" {
            if self.snap.is_none() {
                println!("no transaction");
            } else {
                self.snap = None;
                println!("commit");
            }
            return true;
        }
        if line == ":rollback" {
            match self.snap.take() {
                Some(s) => {
                    match self.cat.graph_mut(&self.cur) {
                        Some(g) => {
                            *g = s;
                            println!("rollback");
                        }
                        None => println!("no graph {}", self.cur),
                    }
                }
                None => println!("no transaction"),
            }
            return true;
        }
        if line.starts_with('.') {
            println!("unknown command");
            return true;
        }
        let params = self.params.clone();
        let g = match self.graph_mut() {
            Ok(g) => g,
            Err(e) => {
                println!("{}", e);
                return true;
            }
        };
        let r = query::run_with(g, line, params);
        print_result(&r);
        true
    }
}

fn print_help() {
    println!(".load FILE   .save FILE");
    println!(".graphs      .use NAME      .create NAME   .drop NAME");
    println!(":param NAME VALUE    :params");
    println!(":begin :commit :rollback");
    println!(".help        .quit");
    println!("MATCH still takes the current graph.");
}

fn parse_param_line(rest: &str) -> Result<(String, Prop), String> {
    let rest = rest.trim();
    if rest.is_empty() {
        return Err("usage: :param NAME VALUE".to_string());
    }
    let mut i = 0;
    let b = rest.as_bytes();
    while i < b.len() && !(b[i] as char).is_whitespace() {
        i += 1;
    }
    if i == 0 {
        return Err("usage: :param NAME VALUE".to_string());
    }
    let name = rest[..i].to_string();
    let val = rest[i..].trim();
    if val.is_empty() {
        return Err("usage: :param NAME VALUE".to_string());
    }
    Ok((name, parse_prop_text(val)))
}

fn parse_prop_text(raw: &str) -> Prop {
    if raw.len() >= 2 {
        let b = raw.as_bytes();
        if (b[0] == b'\'' || b[0] == b'"') && b[b.len() - 1] == b[0] {
            return Prop::from_str(&raw[1..raw.len() - 1]);
        }
    }
    if raw == "true" {
        return Prop::from_bool(true);
    }
    if raw == "false" {
        return Prop::from_bool(false);
    }
    match raw.parse::<i64>() {
        Ok(n) => return Prop::from_int(n),
        Err(_) => {}
    }
    Prop::from_str(raw)
}

fn fmt_cell(v: &Option<Val>) -> String {
    match *v {
        None => String::new(),
        Some(Val::Id(ref s)) => s.clone(),
        Some(Val::Path(ref p)) => {
            let ids = p.ids();
            let mut s = String::new();
            let mut i = 0;
            while i < ids.len() {
                if i == 0 {
                    s.push_str(&format!("{}", ids[i]));
                } else if i % 2 == 1 {
                    s.push_str("-[");
                    s.push_str(&format!("{}", ids[i]));
                    s.push_str("]-");
                } else {
                    s.push_str(&format!("{}", ids[i]));
                }
                i += 1;
            }
            s
        }
        Some(Val::List(ref xs)) => {
            let mut s = String::from("[");
            let mut i = 0;
            while i < xs.len() {
                if i > 0 {
                    s.push_str(", ");
                }
                s.push_str(&fmt_cell(&Some(xs[i].clone())));
                i += 1;
            }
            s.push(']');
            s
        }
        Some(Val::Prop(ref p)) => p.as_display(),
    }
}

fn print_result(r: &QueryResult) {
    if !r.ok {
        println!("{}", r.message);
        return;
    }
    if r.columns.is_empty() {
        println!("{}", r.message);
        if r.created > 0 || r.deleted > 0 {
            println!("created {} deleted {}", r.created, r.deleted);
        }
        return;
    }
    let mut widths: Vec<usize> = r.columns.iter().map(|c| c.len()).collect();
    let mut cells: Vec<Vec<String>> = Vec::new();
    for row in r.rows.iter() {
        let mut cr = Vec::new();
        let mut i = 0;
        while i < r.columns.len() {
            let s = fmt_cell(row.get(i).unwrap_or(&None));
            if s.len() > widths[i] {
                widths[i] = s.len();
            }
            cr.push(s);
            i += 1;
        }
        cells.push(cr);
    }
    let mut head = String::new();
    let mut rule = String::new();
    let mut i = 0;
    while i < r.columns.len() {
        if i > 0 {
            head.push_str(" | ");
            rule.push_str("-+-");
        }
        let w = widths[i];
        head.push_str(&r.columns[i]);
        let mut p = r.columns[i].len();
        while p < w {
            head.push(' ');
            p += 1;
        }
        let mut d = 0;
        while d < w {
            rule.push('-');
            d += 1;
        }
        i += 1;
    }
    println!("{}", head);
    println!("{}", rule);
    for cr in cells.iter() {
        let mut line = String::new();
        let mut i = 0;
        while i < cr.len() {
            if i > 0 {
                line.push_str(" | ");
            }
            line.push_str(&cr[i]);
            let mut p = cr[i].len();
            while p < widths[i] {
                line.push(' ');
                p += 1;
            }
            i += 1;
        }
        println!("{}", line);
    }
    println!("{}", r.message);
}

fn main() {
    let mut sh = Shell::new();
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        match sh.load(&args[1]) {
            Ok(()) => {}
            Err(e) => {
                let mut err = io::stderr();
                let _ = writeln!(err, "{}", e);
                std::process::exit(1);
            }
        }
    }
    let stdin = io::stdin();
    let prompt = interactive();
    loop {
        if prompt {
            print!("khg> ");
            let _ = io::stdout().flush();
        }
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                break;
            }
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if !sh.one(line) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{fmt_cell, Shell};
    use khgraphdb::query::Val;

    #[test]
    fn fmt_id() {
        assert_eq!(fmt_cell(&Some(Val::Id("k1".to_string()))), "k1");
        assert_eq!(fmt_cell(&None), "");
    }

    #[test]
    fn fmt_list() {
        let v = Val::List(vec![Val::Id("a".to_string()), Val::Id("b".to_string())]);
        assert_eq!(fmt_cell(&Some(v)), "[a, b]");
    }

    #[test]
    fn create_and_use() {
        let mut sh = Shell::new();
        assert!(sh.one(".create social"));
        assert_eq!(sh.cur, "social");
        assert!(sh.one("CREATE (a:Person {name:'Ada'})"));
        assert!(sh.one(".create other"));
        assert_eq!(sh.cur, "other");
        assert!(sh.one(".use social"));
        assert_eq!(sh.cur, "social");
        let g = sh.graph_mut().unwrap();
        assert!(g.vertex_by_name("Ada").is_some());
    }

    #[test]
    fn begin_rollback() {
        let mut sh = Shell::new();
        assert!(sh.one("CREATE (a:Person {name:'Ada'})"));
        assert!(sh.one(":begin"));
        assert!(sh.one("CREATE (b:Person {name:'Bob'})"));
        {
            let g = sh.graph_mut().unwrap();
            assert_eq!(g.vertex_count(), 2);
        }
        assert!(sh.one(":rollback"));
        let g = sh.graph_mut().unwrap();
        assert_eq!(g.vertex_count(), 1);
        assert!(g.vertex_by_name("Bob").is_none());
    }

    #[test]
    fn begin_commit() {
        let mut sh = Shell::new();
        assert!(sh.one(":begin"));
        assert!(sh.one("CREATE (a:Person {name:'Ada'})"));
        assert!(sh.one(":commit"));
        let g = sh.graph_mut().unwrap();
        assert!(g.vertex_by_name("Ada").is_some());
    }

    #[test]
    fn param_then_match() {
        let mut sh = Shell::new();
        assert!(sh.one("CREATE (a:Person {name:'Ada'})"));
        assert!(sh.one(":param n Ada"));
        assert!(sh.one("MATCH (a:Person {name:$n})"));
        let g = sh.graph_mut().unwrap();
        assert!(g.vertex_by_name("Ada").is_some());
    }

    #[test]
    fn use_refused_in_tx() {
        let mut sh = Shell::new();
        assert!(sh.one(".create other"));
        assert!(sh.one(".use g1"));
        assert!(sh.one(":begin"));
        assert!(sh.one(".use other"));
        assert_eq!(sh.cur, "g1");
        assert!(sh.one(":rollback"));
    }
}
