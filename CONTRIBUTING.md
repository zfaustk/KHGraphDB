# Contributing

KHGraphDB 4.2 is Rust 1.31, edition 2018. The C# kernel in csharp/ is frozen.

- rustc 1.31. `edition = "2018"`. `crate::` paths.
- No `dyn` trait objects. The engine is an enum.
- `pub(crate)` for the walk and the parser.
- No crates.io dependencies in the kernel.
- One idea per commit. A body that says why.
- Type is an object. KHID is the pointer. The walk compares KHID.
- Khid is a u64. The letters are Display.
- A graph is a shard. Addr is `(shard, khid)`. Shard 0 is here.
- Catalog names graphs and assigns shards. It is not a query language.
- The log is KHL1. Uncommitted records do not replay.
- Store commit captures the arena and `sync_data`.
- A replica tails the log. A sentinel promotes on missed beats.
- Type may mark a key as content. The index refuses it.
- A stub is a far title. Hydrate from home in this process.
- A transaction clones the arena. Drop rolls back.
- `cargo test` is the gate.

Do not add C# 6. Do not flatten Type into a string.
