//! Language-neutral bridge for adapters implemented outside Rust.

mod contract;
mod foreign;

#[cfg(feature = "codegen")]
#[doc(hidden)]
pub mod schema;

pub use contract::{AdapterCall, AdapterReply, ForeignDispatcher};
pub use foreign::ForeignAdapter;
