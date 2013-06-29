# KHGraphDB

A Graph Database. Vertex, Edge, Type. By kinghand crew.

C# 5 / .NET 4.5. No packages. The graph is directed.
Type is a first-class object, not a string on the vertex.

BFS stores color / dist / pred in AlgorithmObjs and wipes them in
EndAlgorithm. The walk uses a queue and a visited color so a cycle
does not spin.

