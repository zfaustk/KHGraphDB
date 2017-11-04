# Language

The kernel speaks a small Cypher. MATCH reads. CREATE,
SET, DELETE write. WITH is a row table: columns that
are not named are dropped.

Type is still an object. `:Person` resolves once to a
KHID. EXPLAIN shows that table. The walk never compares
the name string.

```
MATCH (a:Person {name:'Alice'})-[e:KNOWS]->(b)
WHERE e.weight > 1
RETURN b AS friend
ORDER BY friend.name
SKIP 0 LIMIT 10
```

```
CREATE (a:Person {name:'Ada'}), (b:Person {name:'Bob'}), (a)-[:KNOWS]->(b)
MERGE (p:Person {name:'Ada'}) ON CREATE SET p.born = '1815'
MATCH (n:Person {name:'Solo'}) DELETE n
MATCH (n:Person {name:'Alice'}) DETACH DELETE n
```

```
MATCH p = (a)-[:KNOWS*1..2]->(b)
RETURN length(p), nodes(p), collect(b)
```

A second MATCH starts from names already bound.
UNWIND turns a list into rows.

The cap on `*` is 16. MERGE still refuses a star.
shortestPath is hop-count BFS.

No crates.io. rustc 1.18.
