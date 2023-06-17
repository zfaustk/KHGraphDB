# SNB

LDBC Interactive, cut for a notebook.
The official driver is not here. The
shape is: a deterministic social graph,
short reads IS1–IS7, complex IC1–IC13
that the Cypher can say, updates IU1–IU8.

Scale is people. Posts are 2n. Forums
are n/4. Cosine is not this workload.
MATCH is. The example prints p50/p95.

`cargo run --example snb -- 200`

Not YCSB. Not Graphalytics. Those are
other clocks. This one is multi-hop
on a social kernel.
