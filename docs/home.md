# Home

A vertex lives on one shard. That shard is trusted
for it. The serial stays local. An address is
`(shard, khid)`. The letters are Display.

The log is the truth on a shard. A snapshot is a
compact arena. Index and a far cache of the other
end of an edge are derived: drop them, replay the
log, they come back.

A query that starts at an address runs at home.
A query that needs other homes asks a meta view
for the id set, then one round to those shards.
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
fills a stub from home in one round. A copy
of the log can be promoted. A sentinel watches
the beat. Router across processes, far fill
across a network: not this year.

The arena still clones for rollback. Store
commit is the durable write. khg `:commit`
on an opened dir appends the log. compact
rewrites one capture. Replica is a second
directory. Index still sits on the Graph.
Rebuild it from the arena after replay; do
not treat it as a second truth.

4.2 froze in June. 4.3 is the standby at
year end: the copy cannot write, catch_up
is a tail, the sentinel does not promote
a stale log.

