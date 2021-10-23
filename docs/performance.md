# Performance

This kernel is a notebook at home: sparse nodes,
fat bodies, keyed lookup, a log that is the delta.
It is not a TPC number. There is no YCSB, no LDBC,
no BenchmarkDotNet. Those would describe a different
database.

`cargo run --release --example bench -- 5000`
prints load rate, keyed MATCH, count scan,
commit+1, SET+commit, replay, a torn-tail loop,
and concurrent readers on a snapshot. Times are
nanoseconds. p50 / p95 / p99 are the sorted
samples, not a fitted model.

What the clock is for:

- Identity MATCH should beat a Type scan.
- A one-vertex commit must not walk the arena.
  7.1 records touches; `begin` does not clone.
- Open truncates a torn tail and writes again.
- Readers pin a snapshot. One writer holds the
  lease. Concurrent clients are readers, not
  a lock manager.

What it already points at:

- `commit+1` is still dominated by rewriting
  meta and `fsync`, not by the delta. Meta
  should tail, like the log. See `docs/next.md`.
- A MATCH that only reads should not clone
  the arena. `query::run` is `&mut` because
  CREATE shares the walk.
- Memory `Tx` still clones. Inverse of a
  touch is the next cut, not a lock manager.

A 200-vertex chain (2014, C#) still lives in
`csharp/`. It is a unit test, not evidence.
