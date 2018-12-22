## 4.0.0 - 2018-12-16

- rustc 1.31, edition 2018, `crate::` paths.
- Val::Prop. RETURN a.age keeps the tag.
- In-memory Tx: clone, Drop rolls back, commit keeps.
- khg :begin :commit :rollback. :param.
- A number with a dot is Float.
- EXPLAIN names Project and Limit.
- Unique backfill refuses a duplicate.

## 3.6.0 - 2018-08-05

- MATCH compiles to Seed / Expand / Filter / Optional / Shortest.
- EXPLAIN prints the operator tree.
- Edge source, target, and type are Khid. Vertex types too.
- WHERE on a lone MATCH is Filter, not a second pass.
- Graph::vertex_k / edge_k / ty_k skip the print form.

## 3.5.0 - 2018-04-29

- Prop: Bool / Int / Float / Str. 1 is not "1".
- KHG4 snapshots keep the tag. KHG3 still reads.
- Khid is a u64. Display is still k then hex.
- The arena is a slot Vec. Index is the KHID.
- Adjacency and Type members store Khid.
- Query cases live under src/tests/data/.

## 3.4.0 - 2017-12-16

- Catalog: many graphs in one process. MATCH still
  takes a Graph. Type objects are per arena.
- khg: a stdin shell. `.load` / `.save` KHG3.
  `.use` picks a catalog name.

## 3.3.0 - 2017-11-18

- CREATE, SET, REMOVE, DELETE, DETACH DELETE
- MERGE edges; ON CREATE / ON MATCH SET
- WITH, UNWIND, ORDER BY, SKIP, LIMIT, DISTINCT
- COUNT, collect, length / nodes / relationships
- A second MATCH uses bound names
- Graph::named, clone, subgraph, clear
- Edge (Type, key) index; KHG3 snapshots
- EXPLAIN returns bound Type KHIDs
- MATCH starts from a keyed (Type, key) node
- Parse errors name the token
- examples/social.rs
- rustc 1.18, pub(crate)
- Type.attrs is gone

## 3.2.0 - 2016-12-24

- The walk uses Type's KHID, not the name string.
- Path is a value: hops, nodes, edges.
- Paint is not a vertex attribute.
- query/parse and query/walk.

## 3.1.0 - 2016-11-12

- rustc 1.13. `?` in place of `try!`.
- Variable-length MATCH `[:T*n..m]`. Cap 16.
- Path as a column: interleaved KHIDs.
- `shortestPath()` is hop-count BFS.
- MERGE still refuses a star.

## 3.0.0 - 2015-12-27

- Rust 1.5 kernel. KHID arena, no Rc.
- Type remains a first-class object.
- Schema index and unique constraints.
- KHG2 snapshots.
- BFS / DFS cycle / Dijkstra.
- MATCH / WHERE / OPTIONAL / RETURN / MERGE.
- C# 2.2.1 moved to csharp/ and frozen.

# Changelog

## 2.2.1 - 2015-01-31

- Command MATCH regression tests
- MERGE binds unique names instead of crashing on create
- Broken MATCH syntax is a failed query

## 2.2.0 - 2014-12-13

- Traversal builder, NODE_GLOBAL / NODE_PATH
- Graph.Clone, Subgraph, Clear
- OutgoingAt / IncomingAt
- Samples: Social (MATCH), Paths (Dijkstra)
- docs/type.md, docs/performance.md, CONTRIBUTING

## 2.1.0 - 2014-06-14

- MATCH / OPTIONAL MATCH / WHERE / RETURN
- MERGE vertex and edge
- Path uniqueness so a cycle cannot explode

## 2.0.0 - 2014-03-29

Start of the 2.x line. The 2013 kernel is 1.8.

- MIT license
- The library targets .NET 4.5 still. No packages.
- A vertex may wear more than one Type. Type is still an object.
- Edges wear a Type too. KNOWS is an object, not a string.
- KHG2 snapshots. KHG1 still reads.
- Schema index on (Type, key). Unique constraints.
