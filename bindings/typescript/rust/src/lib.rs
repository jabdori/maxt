#![deny(unsafe_code)]

#[cfg(not(target_arch = "wasm32"))]
use napi_derive::napi;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod builtins;
mod client;
mod convert;
mod foreign;
mod stream;
#[cfg(target_arch = "wasm32")]
mod web;

include!("generated_version.rs");
