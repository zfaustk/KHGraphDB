# Replica

Primary writes. Replica tails. `Role` is that
pair. `graph_mut` on a replica fails. `tail`
copies the log, `catch_up` appends new bytes
(or replaces after compact), `promote` makes
the copy home. commit on a replica fails.

A sentinel watches the beat file primary
touches on commit. Enough unchanged beats
and it promotes. One watcher. No quorum.
Two writers is a split brain. That is the
deal.

A term on the log lasted a week in May.
Promote does not need a vote. Dropped.

```
let mut p = Store::open(prim, "notes", 1)?;
p.commit()?;
let mut r = Store::tail(copy, prim, "notes")?;
assert_eq!(r.role(), Role::Replica);
r.catch_up(prim)?;
let mut w = Sentinel::new(prim, 2);
w.poll(&mut r);
```

Same process, two directories, or a socket.
Pull is a Pos. Commit does not wait.

