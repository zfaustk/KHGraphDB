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
