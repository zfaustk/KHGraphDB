# Replset

A primary and its copies. Commit is
local. Then catch_up until `min_ack`
members honor the Pos. 1 is async.
2 is one copy.

Failover promotes a copy. The old
directory keeps a stale lease. The
new primary takes writes. No vote.
The lease is the fence. Raft was a
week in 2019.

Lag is subtraction of Pos, same
generation. blob/ and vec/ still
copy with the log.

See `docs/replica.md`, `docs/six.md`.
