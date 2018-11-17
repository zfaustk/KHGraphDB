# Type is not a label string

Neo4j 2.0 shipped labels as strings on the node.
KHGraphDB had Type as a first-class object in 2013.

2.0 of this library lets a vertex wear many Types. That
is the label-set idea. The Type itself is still an object
with a KHID and membership lists.

Edges wear a Type the same way. `:KNOWS` in MATCH looks
up `GetTypeByName("KNOWS")`, it does not intern a string
on the edge. The hop compares the Type's KHID. A typed
walk can start from the Type's own edge list. EXPLAIN
prints the bound Type KHID. Graph::named is the graph's
own KHID, not a Type.

An edge's source, target, and type are Khid. A vertex
wears Type as Khid. The print form is Display.

This is the DNA. Do not flatten Type into a string to
look more like Cypher.
