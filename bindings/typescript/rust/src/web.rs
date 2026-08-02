use maxt::Error;
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::convert::outcome;

pub(crate) fn value(value: Value) -> JsValue {
    js_sys::JSON::parse(&value.to_string()).unwrap_or_else(|_| {
        JsValue::from_str(
            &outcome::<Value>(Err(Error::adapter(
                "could not construct WebAssembly binding outcome",
            )))
            .to_string(),
        )
    })
}

pub(crate) fn factory_error(error: Error) -> JsValue {
    js_sys::Error::new(&outcome::<Value>(Err(error)).to_string()).into()
}

#[wasm_bindgen(js_name = "configureRelay")]
pub fn configure_relay(relay_url: String) -> JsValue {
    value(outcome(
        maxt::configure_browser_relay(&relay_url).map(|()| Value::Null),
    ))
}
