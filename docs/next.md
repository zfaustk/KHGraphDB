# Next

The prefix is the database. Clone, a rewritten
meta file, and a mutable MATCH are leftover
prototypes. The clock already says so.

Transaction. Memory `Tx` still clones. It will
keep the inverse of each touch and put the
arena back that way. The store already forgets
a tail. One writer. Readers pin a `Pos`. A
notebook is a home: permission and crash sit
there, not on a row. A second writer waits on
the lease. Isolation is the pin, not a version
chain.

Meta. Every commit still rewrites the posting
file. `commit+1` in the bench is that rewrite
plus `fsync`. Touches already name the keys
that moved. Meta should append PUT and DEL,
the same shape as the log. Rebuild is compact
of meta, not the common path.

MATCH. A read pins a prefix. It should not
need `&mut Graph`. `query::run` is mutable
because CREATE shares the walk. Split the
read. Then a reader does not clone the
notebook to count, and `ask` on a socket
does not snapshot the whole home.

Durability. Every commit `fsync`s. That is
the tax, and it stays the default. A session
may group the sync. It does not skip the log.

Body. The page stays at home. A vector, if
it comes, is another posting list on the same
pin. Do not put the body in the index. Do not
start a second database for recall.

The bench is the gate. p95 of `commit+1` and
keyed MATCH first. Not LDBC. Not a lock table.
