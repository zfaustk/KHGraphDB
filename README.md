# KHGraphDB

A graph database. Vertex, Edge, Type. By kinghand.

**5.0.0** (2020). A Pos on a socket. Rust 1.31, edition 2018. No crates.io dependencies.

Type is a first-class object, not a string label. KHID is identity
and the only pointer: a u64, printed `k` then hex. Vertices, edges
and types live in a slot Vec. Slot 0 is nil.
An edge stores Khid ends. A vertex wears Type as Khid.
The arena returns a Khid. MATCH binds that serial.
Val::Id is a Khid. A name or a count is a Prop.
MATCH compiles to Seed, Expand, Filter. EXPLAIN prints the tree.

A vertex lives on one shard. That shard is trusted for it.
Off this box the name is an address: `(shard, khid)`.
Content is not topology. A far cite is an Addr.
Store opens a directory; the log is truth.
A copy of the log can be promoted. A sentinel
watches the beat. See `docs/home.md`.

The C# 2.2.1 kernel is frozen in `csharp/`.

## Vision

Property graphs are the right model for multi-hop questions.
A small declarative surface (MATCH, MERGE) will travel further
than vendor-specific APIs. Type should stay an object, not a
string tag, so schema and identity remain first-class.

The kernel is Rust. Systems code that lives for years needs
compile-time ownership more than a GC. std only: if the model
needs a framework, the model is wrong.

## Build

```
cargo test
cargo run --example social
cargo run --example notes
cargo run --example shards
cargo run --bin khg
```

Needs rustc 1.31 (December 2018). `edition = "2018"`.

## Shell

```
cargo run --bin khg
cargo run --bin khg -- graph.khg
```

Dot commands: `.load` `.save` `.open` `.tail` `.promote` `.compact` `.graphs` `.use` `.create` `.drop`.
`:param` `:params` `:begin` `:commit` `:rollback`.
A line that is not a dot is MATCH (or CREATE, MERGE, …) on
the current graph. Catalog holds the rest.

## Use

```
let mut g = Graph::new();
let alice = g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
let bob = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
g.add_edge(alice, bob, Some("KNOWS")).unwrap();
g.create_unique("Person", "name");
let r = khgraphdb::query::run(&mut g, "MATCH (a:Person)-[:KNOWS]->(b) WHERE a.name = 'Alice'");
```

## Language

```
MATCH (n:Person)
MATCH (a:Person {name:'Alice'})-[:KNOWS]->(b)
MATCH (a:Person {name:'Alice'})-[e:KNOWS*1..2]->(b)
MATCH p = shortestPath((a)-[:KNOWS*]->(b))
MATCH (a)-[:KNOWS]->(b) WHERE a.name = 'Alice' RETURN b
OPTIONAL MATCH (a:Person {name:'Ada'})-[:KNOWS]->(b)
CREATE (a:Person {name:'Ada'})-[:KNOWS]->(b:Person {name:'Bob'})
MERGE (p:Person {name:'Ada'}) ON CREATE SET p.born = 1815
WITH, UNWIND, ORDER BY, SKIP, LIMIT, DISTINCT, count, collect
RETURN a.age
EXPLAIN MATCH (a:Person)-[:KNOWS]->(b) RETURN b LIMIT 1
$param
```

See `docs/type.md`, `docs/language.md`, `docs/pipeline.md`,
`docs/tx.md`, `docs/home.md`, `docs/content.md`, `docs/store.md`,
and `docs/replica.md`. Type is still not a string. MATCH binds
it by KHID. Path is a value. A graph can be named, cloned,
or cut down to a subgraph. A Catalog holds several graphs;
the query still takes one. A transaction clones the arena.

