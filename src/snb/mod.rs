//! LDBC SNB Interactive, cut for a notebook.
//! The generator is deterministic. The reads are
//! Cypher. The updates are arena puts. Not the
//! official driver. The shape is the workload.

pub mod gen;
pub mod is;
pub mod ic;
pub mod iu;
pub mod bi;
pub mod check;

pub use gen::{generate, Social, Rng};

