# Contributing

KHGraphDB 3.x is Rust 1.13. The C# kernel in csharp/ is frozen.

- rustc 1.13. No edition key.
- No crates.io dependencies in the kernel.
- One idea per commit. A body that says why.
- Type is an object. KHID is the pointer. The walk compares KHID.
- `cargo test` is the gate.

Do not add C# 6. Do not flatten Type into a string.
