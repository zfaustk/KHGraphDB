//! Query cases. A file is a graph, a query, and a table.
//! Checkout this tree and `cargo test` covers the language.

use crate::Graph;
use crate::query;
use crate::query::Val;
use crate::Prop;

struct Case {
    name: String,
    graph: String,
    query: String,
    expect: String,
    sort: bool,
    params: Vec<(String, String)>,
}

fn parse_cases(src: &str) -> Vec<Case> {
    let mut out = Vec::new();
    let mut cur_name = String::new();
    let mut graph = String::new();
    let mut query = String::new();
    let mut expect = String::new();
    let mut sort = true;
    let mut params: Vec<(String, String)> = Vec::new();
    let mut sec = 0; // 0 name, 1 graph, 2 query, 3 expect
    for raw in src.lines() {
        let line = raw.trim_end();
        if line.starts_with("## ") {
            if !cur_name.is_empty() {
                out.push(Case {
                    name: cur_name.clone(),
                    graph: graph.clone(),
                    query: query.clone(),
                    expect: expect.clone(),
                    sort: sort,
                    params: params.clone(),
                });
            }
            cur_name = line[3..].trim().to_string();
            graph.clear();
            query.clear();
            expect.clear();
            sort = true;
            params.clear();
            sec = 0;
            continue;
        }
        if line == ".stable" {
            sort = false;
            continue;
        }
        if line.starts_with(".param ") {
            let rest = line[7..].trim();
            let kv: Vec<&str> = rest.splitn(2, '=').collect();
            if kv.len() == 2 {
                params.push((kv[0].trim().to_string(), kv[1].trim().to_string()));
            }
            continue;
        }
        if line == ".graph" {
            sec = 1;
            continue;
        }
        if line == ".query" {
            sec = 2;
            continue;
        }
        if line == ".expect" {
            sec = 3;
            continue;
        }
        if line.is_empty() && sec != 2 {
            continue;
        }
        match sec {
            1 => {
                graph.push_str(line);
                graph.push('\n');
            }
            2 => {
                query.push_str(line);
                query.push('\n');
            }
            3 => {
                expect.push_str(line);
                expect.push('\n');
            }
            _ => {}
        }
    }
    if !cur_name.is_empty() {
        out.push(Case {
            name: cur_name,
            graph: graph,
            query: query,
            expect: expect,
            sort: sort,
            params: params,
        });
    }
    out
}

fn parse_prop(raw: &str) -> Prop {
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
    let mut digits = true;
    let mut dot = false;
    let mut i = 0;
    let b = raw.as_bytes();
    if !b.is_empty() && (b[0] == b'-' || b[0] == b'+') {
        i = 1;
    }
    if i >= b.len() {
        digits = false;
    }
    while i < b.len() {
        if b[i] == b'.' && !dot {
            dot = true;
        } else if b[i] < b'0' || b[i] > b'9' {
            digits = false;
            break;
        }
        i += 1;
    }
    if digits && dot {
        match raw.parse::<f64>() {
            Ok(n) => return Prop::from_float(n),
            Err(_) => {}
        }
    }
    if digits && !dot {
        match raw.parse::<i64>() {
            Ok(n) => return Prop::from_int(n),
            Err(_) => {}
        }
    }
    Prop::from_str(raw)
}

fn load_graph(text: &str) -> Graph {
    let mut g = Graph::new();
    g.create_index("Person", "name");
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[0] == "T" {
            g.add_type(parts[1]).unwrap();
        } else if parts.len() >= 3 && parts[0] == "N" {
            // N Type name [k=v ...]
            let ty = parts[1];
            let name = parts[2];
            let mut attrs = std::collections::HashMap::new();
            attrs.insert("name".to_string(), name.to_string());
            let id = g.add_vertex(attrs, Some(ty)).unwrap();
            let mut i = 3;
            while i < parts.len() {
                let kv: Vec<&str> = parts[i].splitn(2, '=').collect();
                if kv.len() == 2 {
                    g.set_prop(id, kv[0], parse_prop(kv[1])).unwrap();
                }
                i += 1;
            }
        } else if parts.len() >= 4 && parts[0] == "E" {
            // E Type srcName dstName [k=v ...]
            let ty = parts[1];
            let a = g.vertex_by_name(parts[2]).unwrap().khid();
            let b = g.vertex_by_name(parts[3]).unwrap().khid();
            let eid = g.add_edge(a, b, Some(ty)).unwrap();
            let mut i = 4;
            while i < parts.len() {
                let kv: Vec<&str> = parts[i].splitn(2, '=').collect();
                if kv.len() == 2 {
                    g.set_edge_prop(eid, kv[0], parse_prop(kv[1]));
                }
                i += 1;
            }
        }
    }
    g
}

fn cell_text(g: &Graph, v: &Option<Val>) -> String {
    match *v {
        None => String::new(),
        Some(Val::Id(id)) => {
            match g.vertex(id).and_then(|x| x.get("name")).map(|s| s.to_string()) {
                Some(n) => n,
                None => format!("{}", id),
            }
        }
        Some(Val::Path(ref p)) => format!("path:{}", p.hops()),
        Some(Val::List(ref xs)) => format!("list:{}", xs.len()),
        Some(Val::Prop(ref p)) => p.as_display(),
    }
}

fn run_case(c: &Case) {
    let mut g = load_graph(&c.graph);
    let mut map = std::collections::HashMap::new();
    for &(ref k, ref v) in c.params.iter() {
        map.insert(k.clone(), parse_prop(v));
    }
    let r = if map.is_empty() {
        query::run(&mut g, c.query.trim())
    } else {
        query::run_with(&mut g, c.query.trim(), map)
    };
    let mut got = String::new();
    if !r.ok {
        got.push_str("ERR ");
        got.push_str(&r.message);
        got.push('\n');
    } else {
        got.push_str(&format!("rows {}\n", r.rows.len()));
        if !r.columns.is_empty() {
            got.push_str(&r.columns.join(" | "));
            got.push('\n');
            let mut lines = Vec::new();
            for row in r.rows.iter() {
                let mut cells = Vec::new();
                let mut i = 0;
                while i < r.columns.len() {
                    cells.push(cell_text(&g, row.get(i).unwrap_or(&None)));
                    i += 1;
                }
                lines.push(cells.join(" | "));
            }
            if c.sort {
                lines.sort();
            }
            for line in lines.iter() {
                got.push_str(line);
                got.push('\n');
            }
        }
    }
    let exp = trim_lines(c.expect.trim());
    let got_t = trim_lines(got.trim());
    if got_t != exp {
        panic!("case {}:\nquery: {}\ngot:\n{}\nexpect:\n{}\n",
               c.name,
               c.query.trim(),
               got_t,
               exp);
    }
}

fn trim_lines(s: &str) -> String {
    let mut out = String::new();
    let mut first = true;
    for line in s.lines() {
        if !first {
            out.push('\n');
        }
        first = false;
        out.push_str(line.trim_end());
    }
    out
}

fn run_src(src: &str) {
    let cases = parse_cases(src);
    assert!(!cases.is_empty());
    for c in cases.iter() {
        run_case(c);
    }
}

#[test]
fn cases_match() {
    run_src(include_str!("data/match.txt"));
}

#[test]
fn cases_write() {
    run_src(include_str!("data/write.txt"));
}

#[test]
fn cases_where() {
    run_src(include_str!("data/where.txt"));
}

#[test]
fn cases_path() {
    run_src(include_str!("data/path.txt"));
}

#[test]
fn cases_optional() {
    run_src(include_str!("data/optional.txt"));
}

#[test]
fn cases_return() {
    run_src(include_str!("data/return.txt"));
}

#[test]
fn cases_with() {
    run_src(include_str!("data/with.txt"));
}

#[test]
fn cases_param() {
    run_src(include_str!("data/param.txt"));
}

#[test]
fn cases_error() {
    run_src(include_str!("data/error.txt"));
}

#[test]
fn cases_merge() {
    run_src(include_str!("data/merge.txt"));
}

#[test]
fn cases_explain() {
    run_src(include_str!("data/explain.txt"));
}

#[test]
fn cases_prop() {
    run_src(include_str!("data/prop.txt"));
}
