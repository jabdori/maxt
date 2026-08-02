//! Language-neutral bridge for adapters implemented outside Rust.

mod contract;
mod foreign;

pub use contract::{AdapterCall, AdapterReply, ForeignDispatcher};
pub use foreign::ForeignAdapter;
