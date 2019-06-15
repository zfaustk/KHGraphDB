# Replica

A copy of the log. `tail` copies, `catch_up`
copies again. `promote` drops read-only.
commit on a replica fails.

A sentinel watches the beat file primary
touches on commit. Enough unchanged beats
and it promotes. One watcher. No quorum.
Two writers is a split brain. That is the
deal.

```
let mut p = Store::open(prim, "notes", 1)?;
p.commit()?;
let mut r = Store::tail(copy, prim, "notes")?;
r.catch_up(prim)?;
let mut w = Sentinel::new(prim, 2);
w.poll(&mut r);
```

Same process, two directories. Not a network.
