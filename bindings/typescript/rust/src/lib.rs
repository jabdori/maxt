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

#[cfg(not(target_arch = "wasm32"))]
#[napi]
pub const NATIVE_API_VERSION: u32 = 15;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = "export const NATIVE_API_VERSION = 15;")]
extern "C" {
    #[wasm_bindgen(thread_local_v2, reexport)]
    static NATIVE_API_VERSION: JsValue;
}
