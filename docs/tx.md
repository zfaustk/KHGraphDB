# Transaction

A transaction is a prefix. Clone was a
prototype. It is not MVCC, not a copy-on-write
page tree, not a lock table.

Memory `Tx` keeps the inverse of each touch
and puts the arena back that way. The store
forgets a tail. One writer. Readers pin a
`Pos`. Commit is advancing the prefix. Drop
is discarding it.

A notebook is a home. Permission, crash, and
the body sit there. A second writer waits on
the lease. It does not take a row lock. If a
vector posting comes, it is another reader of
the same prefix. Isolation is the pin, not a
version chain.

```
let bm = store.commit()?;
let old = store.read_at(bm)?;
```

`.use` is refused while a transaction is open.
See `docs/store.md`, `docs/six.md`, `docs/next.md`.
