//! What this kernel is for: identity lookup, a fat
//! page at home, a log that is the delta. Not LDBC.
//! Not YCSB. Those names would lie.
//!
//!   cargo run --release --example bench -- 5000

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Instant;

use khgraphdb::{query, Prop, Store};

fn tmp() -> PathBuf {
    let p = env::temp_dir().join(format!("khg-bench-{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    p
}

fn ns(t: Instant) -> u64 {
    let d = t.elapsed();
    d.as_secs() * 1_000_000_000 + d.subsec_nanos() as u64
}

fn pct(xs: &mut [u64], p: u32) -> u64 {
    if xs.is_empty() {
        return 0;
    }
    xs.sort();
    let i = ((p as u64) * (xs.len() as u64 - 1)) / 100;
    xs[i as usize]
}

fn report(name: &str, xs: &mut [u64]) {
    if xs.is_empty() {
        println!("{:<16} empty", name);
        return;
    }
    let n = xs.len() as u64;
    let mut sum = 0u64;
    for x in xs.iter() {
        sum += *x;
    }
    let p50 = pct(xs, 50);
    let p95 = pct(xs, 95);
    let p99 = pct(xs, 99);
    println!("{:<16} n={:<6} p50={:<8} p95={:<8} p99={:<8} mean={}",
             name, n, p50, p95, p99, sum / n);
}

fn doc(i: u64) -> HashMap<String, Prop> {
    let mut m = HashMap::new();
    m.insert("name".to_string(), Prop::from_str(&format!("n{}", i)));
    m.insert("body".to_string(), Prop::from_str("a notebook page"));
    m
}

fn main() {
    let n: u64 = env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(3000);
    println!("KHGraphDB bench  N={}  (identity mix, not LDBC)", n);
    let dir = tmp();
    let mut s = Store::open(&dir, "notes", 1).unwrap();
    s.graph_mut().unwrap().create_index("Doc", "name");
    s.commit().unwrap();

    let t = Instant::now();
    let mut i = 0u64;
    while i < n {
        s.put_vertex(doc(i), Some("Doc")).unwrap();
        i += 1;
    }
    s.commit().unwrap();
    let load_ns = ns(t);
    println!("{:<16} {} vtx in {} ns  ({:.0}/s)",
             "load", n, load_ns,
             (n as f64) * 1e9 / (load_ns as f64));

    let mut keyed = Vec::new();
    i = 0;
    while i < 400 {
        let q = format!("MATCH (a:Doc {{name:'n{}'}}) RETURN a", i % n);
        let t = Instant::now();
        let r = query::ask(s.graph(), &q);
        keyed.push(ns(t));
        assert!(r.ok);
        i += 1;
    }
    report("keyed MATCH", &mut keyed);

    let mut scan = Vec::new();
    i = 0;
    while i < 40 {
        let t = Instant::now();
        let r = query::ask(s.graph(), "MATCH (a:Doc) RETURN count(a)");
        scan.push(ns(t));
        assert!(r.ok);
        i += 1;
    }
    report("count scan", &mut scan);

    s.graph_mut().unwrap().create_index("Doc", "n");
    i = 0;
    while i < n {
        if let Some(v) = s.graph().vertex_by_name(&format!("n{}", i)) {
            let id = v.khid();
            s.graph_mut().unwrap().set_prop(id, "n", khgraphdb::Prop::from_int(i as i64)).unwrap();
        }
        i += 1;
    }
    s.commit().unwrap();
    let mut ranged = Vec::new();
    i = 0;
    while i < 200 {
        let t = Instant::now();
        let r = query::ask(s.graph(), "MATCH (a:Doc) WHERE a.n > 10 RETURN a");
        ranged.push(ns(t));
        assert!(r.ok);
        i += 1;
    }
    report("range MATCH", &mut ranged);

    let mut commits = Vec::new();
    i = 0;
    while i < 200 {
        let t = Instant::now();
        s.put_vertex(doc(n + i), Some("Doc")).unwrap();
        s.commit().unwrap();
        commits.push(ns(t));
        i += 1;
    }
    report("commit+1", &mut commits);

    let mut sets = Vec::new();
    i = 0;
    while i < 100 {
        let q = format!("MATCH (a:Doc {{name:'n{}'}}) SET a.body = 'x'", i % 50);
        let t = Instant::now();
        let r = s.query(&q);
        assert!(r.ok);
        s.commit().unwrap();
        sets.push(ns(t));
        i += 1;
    }
    report("SET+commit", &mut sets);

    drop(s);
    let t = Instant::now();
    let s = Store::open(&dir, "notes", 1).unwrap();
    let replay_ns = ns(t);
    println!("{:<16} {} ns  {} vtx", "replay", replay_ns, s.graph().vertex_count());
    drop(s);

    let mut s = Store::open(&dir, "notes", 1).unwrap();
    let mut k = 0u32;
    while k < 50 {
        s.put_vertex(doc(900000 + k as u64), Some("Doc")).unwrap();
        s.commit().unwrap();
        drop(s);
        {
            use std::fs::OpenOptions;
            let mut f = OpenOptions::new().write(true).open(dir.join("log")).unwrap();
            f.seek(SeekFrom::End(0)).unwrap();
            f.write_all(&[9, 9, 9, 9, 9, 9, 9, 9]).unwrap();
        }
        s = Store::open(&dir, "notes", 1).unwrap();
        k += 1;
    }
    println!("{:<16} {} cycles, still {} vtx", "torn loop", 50, s.graph().vertex_count());

    let g = s.graph().clone();
    let mut handles = Vec::new();
    let mut r = 0;
    while r < 4 {
        let gg = g.clone();
        handles.push(std::thread::spawn(move || {
            let mut j = 0;
            while j < 100 {
                let q = format!("MATCH (a:Doc {{name:'n{}'}}) RETURN a", j);
                let _ = query::ask(&gg, &q);
                j += 1;
            }
        }));
        r += 1;
    }
    let t = Instant::now();
    for h in handles {
        let _ = h.join();
    }
    println!("{:<16} 4 readers × 100 MATCH in {} ns", "readers", ns(t));

    let _ = fs::remove_dir_all(&dir);
}
