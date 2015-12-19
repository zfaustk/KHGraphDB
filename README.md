# KHGraphDB

A graph database. Vertex, Edge, Type. By kinghand.

**3.0.0** (2015). Rust 1.5. No crates.io dependencies.

Type is a first-class object, not a string label. KHID is identity
and the only pointer: the graph is an arena of HashMaps.

The C# 2.2.1 kernel is frozen in `csharp/`.

## Build

```
cargo test
```

Needs rustc 1.5 (December 2015). No `edition` key. No `?`.

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
MATCH (a)-[:KNOWS]->(b) WHERE a.name = 'Alice' RETURN b
OPTIONAL MATCH (a:Person {name:'Ada'})-[:KNOWS]->(b)
MERGE (p:Person {name:'Ada'})
```

See `docs/type.md`. Type is still not a string.
