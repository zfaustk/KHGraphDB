# Home

A vertex lives on one shard. That shard is trusted
for it. The serial stays local. An address is
`(shard, khid)`. The letters are Display.

The log is the truth on a shard. A snapshot is a
compact arena. Index and a far cache of the other
end of an edge are derived: drop them, replay the
log, they come back.

A query that starts at an address runs at home.
A query that needs other homes asks once for
the far Addr set, then one round of stubs.
RETURN e.title is that stub. MATCH still
binds a Khid; the far end is not a hop.
Fill a far cache and that round is skipped. Fill
it everywhere and every shard holds the skeleton:
the read path of a full replica.

Time worst: hop by hop, each hop a home lookup.
Space worst: the skeleton on every shard.
The knobs are the meta and the far cache.

Identity off this box is the address, not the
print form of a local serial. KHID never leaves
its shard as a pointer. A far edge stores an
address. MATCH on one shard still binds a Khid.

A shard has a primary. A copy of the log can be
promoted. There is no consensus round on a write.
The primary appends, `sync_data` on commit, a
copy tails. Wrong primary: last un-acked append
may vanish. That is the deal.

This process may hold several shards. Hydrate
fills a stub from home in one round, also
over a socket. A copy of the log can be
promoted. A sentinel watches the beat; a
failed catch_up does not promote. Router
across processes is a Pos on TCP. MATCH
still binds a Khid on one graph.

The arena still clones for rollback. Store
commit is the durable write. khg `:commit`
on an opened dir appends the log. compact
rewrites one capture. Replica is a second
directory. Index still sits on the Graph.
Rebuild it from the arena after replay; do
not treat it as a second truth.

4.3 is the standby at year end. 5.0 is a
Pos on a socket: mutation log, generation,
pull, one-round hydrate. Commit does not wait.


