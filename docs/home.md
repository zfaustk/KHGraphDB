# Home

A vertex lives on one shard. That shard is trusted
for it. The serial stays local. An address is
`(shard, khid)`. The letters are Display.

The log is the truth on a shard. A snapshot is a
compact arena. Index and a far cache of the other
end of an edge are derived: drop them, replay the
log, they come back.

A query that starts at an address runs at home.
FIND asks meta for the Addr set, then one
round. MATCH still binds a Khid on one graph.
A bookmark is a Pos. A replica that cannot
honor it does not answer.
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

The arena still clones when there is no store.
That is a prototype. A transaction is a
prefix: the store already pins a Pos. Memory
Tx will keep an inverse, not a second arena.
Store rollback replays the log. A lease fences
the writer. FIND on a socket is one round.
commit is the durable write. khg `:commit`
on an opened dir appends the log. compact
rewrites one capture. Replica is a second
directory. Index still sits on the Graph.
Rebuild it from the arena after replay; do
not treat it as a second truth.

4.3 is the standby at year end. 5.0 is a
Pos on a socket. 5.1 is meta. 6.0 is the
prefix: `docs/six.md`. What is still a
prototype is `docs/next.md`.




5.1 froze in December. Vectors are another
index, not topology. Not this year.
