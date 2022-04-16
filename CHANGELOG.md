## 8.0.0 - 2022-04-16

- The posting is a B-tree. Comparison is a range.
- Seed starts at the cheaper end. EXPLAIN prints cost.
- Meta fold rewrites a run. Compact is the merge.
- Replica lag is a Pos. Grouped sync is a session.
- `docs/kernel.md`: the subtractions.

## 7.2.0 - 2021-11-21

- Memory Tx keeps an inverse, not a clone.
- Store rollback forgets the tail.
- `ask` is MATCH on `&Graph`. A write is refused.
- Meta tails PUT/DEL. Compact still rebuilds.
- Bench: keyed MATCH does not clone.

## 7.1.0 - 2021-10-23

- begin does not clone. Commit writes touches.
- examples/bench: p50/p95/p99, torn loop, readers.
- performance.md: not LDBC, not YCSB.

## 7.0.0 - 2021-10-07

- Commit is only the delta. query is Cypher on the store.
- MATCH on a socket is a snapshot. Meta can forget.
- Delete, SET, compact, replica combinations agree after reopen.

## 6.4.0 - 2021-10-02

- Commit prefers the delta over pending.
- Restore of a Vertex is a replace.
- Attach type posts the index.

## 6.3.0 - 2021-09-25

- Remove unposts the index.
- Delete is a log record. Replay does not resurrect.
- Open truncates a torn tail before the next append.

## 6.2.0 - 2021-09-18

- Commit writes the delta, not the arena.
- Restore posts the index. Follow then FIND.

## 6.1.0 - 2021-08-28

- FIND on a socket. Route fans out once.
- Docs: clone is only the memory tx.

## 6.0.0 - 2021-07-10

- The log prefix is the database. `read_at(Pos)`
  does not see later commits. Rollback replays.
- KHL3: CRC on each record. A torn tail is dropped.
- Keyed MATCH runs at the homes meta names.
- A lease fences a writer. Drop releases it.
  Crash holds until expiry. No quorum.

## 5.1.0 - 2020-08-29

- A bookmark is a Pos. `honor` catch_up or fails.
- KHM1: posting only. Rebuild from the arena.
- FIND Type key value. Catalog `locate` is Addr.
- Replica copies meta. Compact keeps FIND.
- khg `.find` `.meta`. Commit still does not wait.

## 5.0.0 - 2020-04-25

- Commit writes this tx. graph_mut still may capture.
- KHL2: generation in the header. compact bumps it.
- Pos `(generation, offset)` is a bookmark.
- Pull over TCP: same epoch appends, new epoch replaces.
- Hydrate is one round of Addr → Stub.
- Sentinel does not promote when catch_up fails.
- Commit does not wait on a replica.
- RETURN e.title is the far stub after one fill
  round. khg `.listen` / `.follow`. Far MATCH
  does not collapse cites onto nil.

## 4.3.0 - 2019-12-21

- Replica `graph_mut` / `begin` fail. Writes stop
  at the arena, not at commit.
- catch_up appends new bytes. Compact on primary
  replaces the replica file.
- Sentinel catch_up before promote.
- khg `.tail DIR FROM` / `.promote`.

## 4.2.0 - 2019-06-15

- cite_title: a far edge shows the stub, not the page.
- Replica: tail / catch_up / promote. Read-only until
  promote. Split brain is the deal.
- Sentinel: missed beats promote. One watcher, no quorum.
- compact rewrites one capture. khg `:commit` writes the log.

## 4.1.0 - 2019-03-23

- A vertex lives on one shard. Addr is `(shard, khid)`.
- Type marks content keys. The index refuses a page.
- A far edge stores an Addr. MATCH still binds a Khid.
- Stub: a far title, hydrated in this process.
- Store: a directory, KHL1, commit captures and
  `sync_data`. Reopen replays. khg `.open`.

## 4.0.0 - 2018-12-16

- rustc 1.31, edition 2018, `crate::` paths.
- Val::Prop. RETURN a.age keeps the tag.
- In-memory Tx: clone, Drop rolls back, commit keeps.
- khg :begin :commit :rollback. :param.
- A number with a dot is Float.
- EXPLAIN names Project and Limit.
- Unique backfill refuses a duplicate.
- Graph lookups take Khid. Writes return Khid.
  vertex_k is gone: vertex is that lookup.
- Val::Id is a Khid. COUNT and UNWIND strings are Prop.
  EXPLAIN names too. The walk binds the serial.

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
