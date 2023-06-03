# Transaction

A transaction is a prefix. Memory `Tx`
keeps the inverse of each touch. The
store forgets a tail. One writer on
the arena. Readers of the engine hold
an Arc of the last commit. That is the
picture we measured in 2022 and dropped
for a notebook. It is back because ask
now shares a process with the writer.

Isolation for those readers is the
snapshot. Dirty read is not on the Arc.
A phantom arrives when apply publishes.
A pin of a Graph does not move.

Locks are 2PL on KHID. Deadlock is a
cycle. They do not replace the lease.
The lease is still the fence between
processes.

```
let e = Engine::open(dir, "notes", 1)?;
e.apply(|s| { s.query("CREATE (a:Doc {name:'Ada'})"); Ok(()) })?;
let r = e.ask("MATCH (a:Doc) RETURN a");
```

`.use` is refused while a store
transaction is open. See `docs/lock.md`,
`docs/six.md`, `docs/store.md`.
