# Replset

A primary and its copies, held open.
Commit is local. Then catch_up on the
live handle until `min_ack` honor the
Pos. Same generation appends the log
and copies only new blob files.

Failover promotes a copy. The old
handle is demoted. No vote. The lease
is the fence. Raft was a week in 2019.

Lag is subtraction of Pos. See
`docs/replica.md`, `docs/six.md`.
