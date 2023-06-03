# Lock

The key is a Khid. Shared or exclusive.
Wait-for is a graph. A cycle aborts the
requester. There is no page latch. There
is no lock escalation.

The engine takes a lock, then the store.
Two logical transactions that want the
same vertex refuse. The arena is still
one writer. The table is how they know.

Release wakes a compatible waiter.
Upgrade of S to X waits if another S
sits there. Reacquire of X is ok.

See `docs/tx.md`.
