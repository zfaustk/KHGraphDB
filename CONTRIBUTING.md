# Contributing

KHGraphDB 3.x is Rust 1.18. The C# kernel in csharp/ is frozen.

- rustc 1.18. No edition key until 4.0.
- rustc 1.26 (May 2018) has `impl Trait` and `..=`. This
  tree does not need them yet.
- rustc 1.27 named `dyn Trait`. The engine is an enum.
  There is no trait object to miss.
- rustc 1.31 (December 2018) is edition 2018. Wait for
  that release before adding `edition` or `crate::`.
- `pub(crate)` for the walk and the parser.
- No crates.io dependencies in the kernel.
- One idea per commit. A body that says why.
- Type is an object. KHID is the pointer. The walk compares KHID.
- Khid is a u64. The letters are Display.
- Catalog names graphs. It is not a query language.
- A transaction clones the arena. Drop rolls back.
- `cargo test` is the gate.

Do not add C# 6. Do not flatten Type into a string.
