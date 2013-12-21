# KHGraphDB

A Graph Database. Vertex, Edge, Type. By kinghand crew.

C# 5 / .NET 4.5. No packages. The graph is directed.
Type is a first-class object, not a string on the vertex.

- KHID identity map, O(1) lookup
- Type indexed by name
- Vertices indexed by Attributes["name"]
- BFS / DFS / Dijkstra scratch in AlgorithmObjs, wiped on the vertices that ran
- Missing colour is white. Missing dist is infinity. The rest of the graph is left alone.
- KHG1 snapshot, the view is not the data
- Commands: `find Person` · `find Person name=Alice` · `near Alice 2` · `type Idea` · `path Alice Bob` · `shortest Alice Bob`

The original homework lives in zfaustk/-GraphDB and is not modified.
