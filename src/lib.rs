//! KHGraphDB. A directed property graph.
//! Type is a first-class object. KHID is identity
//! on a shard. Addr is identity off this box.

pub use error::{Error, Result};
pub use graph::Graph;
pub use catalog::Catalog;
pub use vertex::Vertex;
pub use edge::Edge;
pub use ty::Type;
pub use prop::Prop;
pub use khid::Khid;
pub use addr::Addr;
pub use stub::Stub;
pub use store::{Store, Role};
pub use pos::Pos;
pub use meta::Meta;
pub use wal::Head;
pub use sentinel::Sentinel;
pub use route::Route;
pub use query::{run as run_query, ask as ask_query, QueryResult, Val, Path};
pub use tx::Tx;

pub mod error;
pub mod khid;
pub mod addr;
pub mod stub;
pub mod prop;
pub mod graph;
pub mod catalog;
pub mod vertex;
pub mod edge;
pub mod ty;
pub mod index;
pub mod io;
pub mod algo;
pub mod query;
pub mod tx;
pub mod wal;
pub mod pos;
pub mod meta;
pub mod store;
pub mod sentinel;
pub mod wire;
pub mod route;

#[cfg(test)]
mod tests;
