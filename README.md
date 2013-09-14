# KHGraphDB

A Graph Database. Vertex, Edge, Type. By kinghand crew.

C# 5 / .NET 4.5. No packages. The graph is directed.
Type is a first-class object, not a string on the vertex.

- KHID identity map, O(1) lookup
- Type indexed by name
- BFS scratch in AlgorithmObjs, wiped in EndAlgorithm
- KHG1 snapshot, the view is not the data
- Commands: `find Person` · `find Person name=Alice` · `near Alice 2` · `type Idea`

The original homework lives in zfaustk/-GraphDB and is not modified.
