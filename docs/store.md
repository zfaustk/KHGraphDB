# Store

A directory holds one shard. `log` is KHL3.
Commit appends this tx, `sync_data`. Empty
pending writes the delta against the snapshot,
not the arena. compact is still one capture.

A lease fences the writer. Drop releases it.
Crash holds until expiry. Replica `follow`
is a Pos on a socket. FIND is a round.