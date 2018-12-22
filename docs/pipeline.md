# Pipeline

MATCH compiles to operators. The engine matches on
an enum. There is no trait object. rustc 1.31 still
has no reason to add `dyn`.

```
Seed → Expand → Expand → Filter
                 └ hop DFS, cap 16
```

- **Seed** starts from a Type, or from a (Type, key)
  index when `{k:v}` is present.
- **Expand** walks one relationship. `*` is a stack
  of depth at most 16, inside Expand, not a loop of
  operators.
- **Filter** is WHERE on that MATCH. A second MATCH
  still joins, then WHERE filters the table.
- **Optional** wraps. Empty inner yields a null row.
- **Shortest** is hop-count BFS.

EXPLAIN MATCH prints the tree: slot, Type name, Type
KHID, then plan and op rows.

Project and Limit exist on the enum. RETURN / SKIP /
LIMIT still take a table of rows after the walk.
A later edition can pull them into the tree.

KHID is the only pointer. Type is an object.
