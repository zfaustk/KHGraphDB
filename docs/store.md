# Store

A directory holds one shard. `log` is KHL3.
Commit appends this tx, `sync_data`. Empty
pending falls back to a capture. Reopen
replays. A torn CRC is a dropped tail.

A lease fences the writer. Drop releases it.
Crash holds until expiry. Replica `follow`
is a Pos on a socket. FIND is a round.