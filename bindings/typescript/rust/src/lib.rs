#![deny(unsafe_code)]

use napi_derive::napi;

#[napi]
pub const NATIVE_API_VERSION: u32 = 1;
