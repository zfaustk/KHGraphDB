# Next

The prefix is the database. What landed in 7.2,
and what is still a clock.

Transaction. Memory `Tx` keeps the inverse of
each touch. The store forgets a tail. One
writer. Readers pin a `Pos`. Not a lock table.

Meta. Commit appends PUT and DEL. Rebuild is
compact of meta, not the common path. The
clock on `commit+1` is now the `fsync`.

MATCH. `ask` takes `&Graph`. A write is
refused. A reader does not clone the notebook
to count. `ask` on a socket is the same.

Durability. Every commit `fsync`s. That is
the tax, and it stays the default. A session
may group the sync. It does not skip the log.

Body. The page stays at home. A vector, if
it comes, is another posting list on the same
pin. Do not put the body in the index. Do not
start a second database for recall.

The bench is the gate. p95 of `commit+1` and
keyed MATCH first. Not LDBC. Not a lock table.
