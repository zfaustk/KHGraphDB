//! KHGraphDB. A directed property graph.
//! Type is a first-class object. KHID is identity.

pub use error::{Error, Result};
pub use graph::Graph;
pub use vertex::Vertex;
pub use edge::Edge;
pub use ty::Type;

pub mod error;
pub mod graph;
pub mod vertex;
pub mod edge;
pub mod ty;
