//! SNB Interactive clock. Scale is people.
//! cargo run --example snb -- 200

use std::env;
use std::time::Instant;
use khgraphdb::snb;

fn ns(t: Instant) -> u64 {
    let d = t.elapsed();
    d.as_secs() * 1_000_000_000 + d.subsec_nanos() as u64
}

fn pct(xs: &mut [u64], p: u32) -> u64 {
    xs.sort();
    if xs.is_empty() {
        return 0;
    }
    let i = ((p as u64) * (xs.len() as u64 - 1)) / 100;
    xs[i as usize]
}

fn report(name: &str, xs: &mut [u64]) {
    println!(
        "{:<12} n={} p50={}ns p95={}ns p99={}ns",
        name,
        xs.len(),
        pct(xs, 50),
        pct(xs, 95),
        pct(xs, 99)
    );
}

fn main() {
    let n: usize = env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(120);
    let t = Instant::now();
    let s = snb::generate(n, 1);
    println!(
        "scale={} people={} posts={} forums={} knows={} likes={} gen={}ms verts={}",
        n,
        s.people.len(),
        s.posts.len(),
        s.forums.len(),
        s.knows,
        s.likes,
        ns(t) / 1_000_000,
        s.g.vertex_count()
    );
    let mut is = Vec::new();
    let mut i = 0;
    while i < 40 {
        let t = Instant::now();
        for (_, r) in snb::is::run_all(&s) {
            assert!(r.ok);
        }
        is.push(ns(t));
        i += 1;
    }
    report("IS1-7", &mut is);
    let mut ic = Vec::new();
    i = 0;
    while i < 20 {
        let t = Instant::now();
        for (_, r) in snb::ic::run_all(&s) {
            assert!(r.ok);
        }
        ic.push(ns(t));
        i += 1;
    }
    report("IC mix", &mut ic);
}
