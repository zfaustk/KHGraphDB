# Store

A directory holds one shard. `log` is KHL3.
Commit writes the delta against the snapshot.
compact is still one capture. There is no
pending list. `query` is Cypher on the arena;
commit is a second call.

A lease fences the writer. Drop releases it.
Crash holds until expiry. Replica `follow`
is a Pos on a socket. FIND is a round.
MATCH on a socket is a snapshot: writes
do not stick. Delete is a record. Open
truncates a torn tail.
