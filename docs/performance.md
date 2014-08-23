# Performance notes

Measured with `System.Diagnostics.Stopwatch` on the test host.
No BenchmarkDotNet. That is 2014.

- Identity, type and schema lookups are dictionaries.
- MATCH seeds from `Find(Type, key)` when a property map is present.
- BFS / DFS / Dijkstra scratch only the vertices they touch.
- Adjacency is a `List`. `OutgoingAt(i)` does not allocate.
- Clone is a KHG2 round trip. It is correct, not a memcpy.

A 200-vertex chain, one-hop MATCH on an indexed name, should
return in well under a second on a 2013 laptop. The test prints
the millisecond count.
