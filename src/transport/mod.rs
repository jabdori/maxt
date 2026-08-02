//! Shared HTTP and WebSocket transport implementations.

#[cfg(target_arch = "wasm32")]
use std::sync::OnceLock;

#[cfg(any(test, target_arch = "wasm32"))]
use crate::error::{Error, Result};

pub(crate) mod http;
pub(crate) mod ws;

#[allow(unused_imports)]
pub(crate) use http::{HttpMethod, HttpRequest, HttpResponse, HttpTransport};
#[allow(unused_imports)]
pub(crate) use ws::{Heartbeat, HeartbeatFrame, WsCommand, WsConnect, WsSession, connect};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn ensure_crypto_provider() {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BrowserRelay {
    pub(crate) http: String,
    pub(crate) websocket: String,
}

#[cfg(target_arch = "wasm32")]
static BROWSER_RELAY: OnceLock<BrowserRelay> = OnceLock::new();

#[cfg(target_arch = "wasm32")]
pub(crate) fn configure_browser_relay(relay_url: &str) -> Result<()> {
    let relay = relay_endpoints(relay_url)?;
    if let Some(configured) = BROWSER_RELAY.get() {
        return if configured == &relay {
            Ok(())
        } else {
            Err(Error::invalid_request(
                "relay_url",
                "the browser relay is already configured to a different origin",
            ))
        };
    }

    match BROWSER_RELAY.set(relay) {
        Ok(()) => Ok(()),
        Err(relay) if BROWSER_RELAY.get() == Some(&relay) => Ok(()),
        Err(_) => Err(Error::invalid_request(
            "relay_url",
            "the browser relay is already configured to a different origin",
        )),
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn browser_relay() -> Option<&'static BrowserRelay> {
    BROWSER_RELAY.get()
}

#[cfg(any(test, target_arch = "wasm32"))]
fn relay_endpoints(relay_url: &str) -> Result<BrowserRelay> {
    if relay_url.trim() != relay_url {
        return Err(Error::invalid_request(
            "relay_url",
            "must not contain leading or trailing whitespace",
        ));
    }

    let url = reqwest::Url::parse(relay_url)
        .map_err(|error| Error::invalid_request("relay_url", error.to_string()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Error::invalid_request(
            "relay_url",
            "must use http or https",
        ));
    }
    if url.host().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(Error::invalid_request(
            "relay_url",
            "must be an origin without credentials, a path, query, or fragment",
        ));
    }

    let origin = url.origin().ascii_serialization();
    let websocket_scheme = if url.scheme() == "https" { "wss" } else { "ws" };
    let websocket_origin = origin.replacen(url.scheme(), websocket_scheme, 1);
    Ok(BrowserRelay {
        http: format!("{origin}/v1/http"),
        websocket: format!("{websocket_origin}/v1/ws"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_origin_builds_fixed_http_and_websocket_endpoints() {
        assert_eq!(
            relay_endpoints("https://relay.example:8443").unwrap(),
            BrowserRelay {
                http: "https://relay.example:8443/v1/http".into(),
                websocket: "wss://relay.example:8443/v1/ws".into(),
            }
        );
    }

    #[test]
    fn relay_configuration_accepts_only_a_plain_http_origin() {
        for invalid in [
            "wss://relay.example",
            "https://user@relay.example",
            "https://relay.example/base",
            "https://relay.example?query=yes",
            "https://relay.example#fragment",
            " https://relay.example",
        ] {
            assert!(
                matches!(relay_endpoints(invalid), Err(Error::InvalidRequest { .. })),
                "{invalid}"
            );
        }
    }
}
