#![deny(unsafe_code)]

use napi_derive::napi;

mod builtins;
mod client;
mod convert;
mod foreign;
mod stream;

#[napi]
pub const NATIVE_API_VERSION: u32 = 1;
