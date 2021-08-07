# Transaction

Memory `Tx` still clones the arena. khg without
`.open` lives there. Drop restores the clone.

Store does not. `rollback` replays the log.
A reader pins a `Pos` and does not see later
commits. One writer holds the lease. There is
no lock manager.

```
let bm = store.commit()?;
let old = store.read_at(bm)?;
```

`.use` is refused while a transaction is open.
See `docs/store.md` and `docs/six.md`.
