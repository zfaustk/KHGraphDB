# Store

A directory holds one shard. `log` is KHL1.
Commit captures the arena, appends, `sync_data`.
Reopen replays. An uncommitted begin is not
on the log.

Index and content marks are records. Rebuild
postings from the arena after replay. A stub
is filled from another graph in this process.
The page is never copied.

```
let mut s = Store::open(dir, "notes", 1)?;
s.graph_mut()?.add_vertex_props(...)?;
s.commit()?;
s.compact()?;
```

`put_vertex` goes on the log as this tx.
`graph_mut` with empty pending captures.
compact bumps generation. Pos is a bookmark.

khg: `.open DIR`. `:commit` writes the log.
`.compact` rewrites it. MATCH still takes one graph.
The page stays at home.
