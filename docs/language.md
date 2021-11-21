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
A one-hop MATCH whose right node names a value
starts there. `{k:v}` uses the (Type, key) index
when it exists, otherwise it scans the Type.

The cap on `*` is 16. MERGE still refuses a star.
shortestPath is hop-count BFS.

MATCH compiles to operators. EXPLAIN MATCH prints Seed,
Expand, Filter, Optional, Shortest. WHERE on a first MATCH
is Filter. RETURN and LIMIT are named on the plan.
A second MATCH still joins, then WHERE filters
the table.

`RETURN a.age` is a Prop. 1 is not "1". A number with
a dot is Float. `$name` is a parameter. MERGE ON MATCH
SET can RETURN the tagged property.

A transaction keeps an inverse of each touch.
On a store, rollback forgets the tail. MATCH
is a read: `ask` takes `&Graph`.

See `docs/pipeline.md`.

No crates.io. rustc 1.31, edition 2018.
