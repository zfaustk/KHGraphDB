//! A stdin shell. Dot commands for the catalog.
//! MATCH still takes one graph.

extern crate khgraphdb;

use std::env;
use std::fs::File;
use std::io::{self, Write, BufRead};
use khgraphdb::{Catalog, Graph, query, io as khio};
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
        if line.starts_with('.') {
            println!("unknown command");
            return true;
        }
        let g = match self.graph_mut() {
            Ok(g) => g,
            Err(e) => {
                println!("{}", e);
                return true;
            }
        };
        let r = query::run(g, line);
        print_result(&r);
        true
    }
}

fn print_help() {
    println!(".load FILE   .save FILE");
    println!(".graphs      .use NAME      .create NAME   .drop NAME");
    println!(".help        .quit");
    println!("MATCH still takes the current graph.");
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
                    s.push_str(&ids[i]);
                } else if i % 2 == 1 {
                    s.push_str("-[");
                    s.push_str(&ids[i]);
                    s.push_str("]-");
                } else {
                    s.push_str(&ids[i]);
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
}
