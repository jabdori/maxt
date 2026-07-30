//! How requests actually leave the process.
//!
//! Private to the crate. Adapters describe *what* to send in exchange-neutral
//! terms; the implementations here decide how it goes over the wire, and own
//! reconnects, heartbeats, and backpressure so that four adapters do not each
//! reimplement them.

pub(crate) mod http;
pub(crate) mod ws;

#[allow(unused_imports)]
pub(crate) use http::{HttpMethod, HttpRequest, HttpResponse, HttpTransport};
#[allow(unused_imports)]
pub(crate) use ws::{Heartbeat, HeartbeatFrame, WsCommand, WsConnect, WsSession, connect};
