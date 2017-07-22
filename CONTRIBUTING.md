# Contributing

KHGraphDB 3.x is Rust 1.18. The C# kernel in csharp/ is frozen.

- rustc 1.18. No edition key.
- `pub(crate)` for the walk and the parser.
- No crates.io dependencies in the kernel.
- One idea per commit. A body that says why.
- Type is an object. KHID is the pointer. The walk compares KHID.
- `cargo test` is the gate.
- No crates.io dependencies in the kernel.
- One idea per commit. A body that says why.
- Type is an object. KHID is the pointer. The walk compares KHID.
- `cargo test` is the gate.

Do not add C# 6. Do not flatten Type into a string.
