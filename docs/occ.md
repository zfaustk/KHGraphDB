# OCC

A session does not clone the arena.
It pins an Arc and writes a set.
Commit sorts the keys, takes X, checks
the recent log, applies, publishes.
Two sessions may prepare at once.

First committer wins on a KHID. Lost
update dies. Write skew lives. That is
SI, not SSI. Predicate locks would
close skew. They are not here.

The lock table is no longer a sidecar.
Wait yields. A cycle aborts. Order of
keys is the KHID order, so two commits
do not deadlock.

Ask in a writer materialises the set
onto a copy of the pin. Readers of the
engine never do. The clone is the
price of seeing own writes, not of
isolation.

```
let mut a = e.session();
let mut b = e.session();
a.set(id, "n", 1);
b.set(id, "n", 2);
a.commit()?;  // wins
b.commit();   // conflict
```
