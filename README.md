# KHGraphDB

A graph database. Vertex, Edge, Type. By kinghand.

**3.2.0** (2016). Rust 1.13. No crates.io dependencies.

Type is a first-class object, not a string label. KHID is identity
and the only pointer: the graph is an arena of HashMaps.

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
```

Needs rustc 1.13 (November 2016). No `edition` key.

## Use

```
let mut g = Graph::new();
let alice = g.add_vertex(attrs("Alice"), Some("Person")).unwrap();
let bob = g.add_vertex(attrs("Bob"), Some("Person")).unwrap();
g.add_edge(&alice, &bob, Some("KNOWS")).unwrap();
g.create_unique("Person", "name");
let r = khgraphdb::query::run(&mut g, "MATCH (a:Person)-[:KNOWS]->(b) WHERE a.name = 'Alice'");
```

## Language

```
MATCH (n:Person)
MATCH (a:Person {name:'Alice'})-[:KNOWS]->(b)
MATCH (a:Person {name:'Alice'})-[:KNOWS*1..2]->(b)
MATCH p = shortestPath((a)-[:KNOWS*]->(b))
MATCH (a)-[:KNOWS]->(b) WHERE a.name = 'Alice' RETURN b
OPTIONAL MATCH (a:Person {name:'Ada'})-[:KNOWS]->(b)
MERGE (p:Person {name:'Ada'})
```

See `docs/type.md`. Type is still not a string. MATCH
binds it by KHID. Path is a value.
