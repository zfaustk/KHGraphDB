# KHGraphDB

A graph database library. Vertex, Edge, Type. By kinghand.

C# 5 / .NET 4.5. No packages. The graph is directed.
Type is a first-class object, not a string on the vertex or the edge.

## What it is

- KHID identity map, O(1) lookup
- A vertex may wear many Types. An edge wears one Type.
- Schema index on `(Type, key)`. Unique constraints.
- BFS / DFS / Dijkstra scratch in AlgorithmObjs
- KHG2 snapshots. KHG1 still reads.
- Commands: `find Person` · `near Alice 2` · `path Alice Bob` · `shortest Alice Bob`

## Build

Open `KHGraphDB.sln` in Visual Studio 2013. Build. Run `KHGraphDB.Tests`.

## Use

```
Graph g = new Graph();
IType person = g.AddType("Person", null);
IType knows = g.AddType("KNOWS", null);
IVertex alice = g.AddVertex(new Dictionary<string, object> { { "name", "Alice" } }, person);
IVertex bob = g.AddVertex(new Dictionary<string, object> { { "name", "Bob" } }, person);
g.AddEdge(alice, bob, knows);
g.CreateUniqueConstraint("Person", "name");
```

The original homework lives in zfaustk/-GraphDB and is not modified.
