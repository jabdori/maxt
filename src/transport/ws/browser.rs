//! Browser WebSocket and timer primitives.

use std::cell::Cell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::Duration;

use gloo_timers::future::TimeoutFuture;
use js_sys::{ArrayBuffer, Uint8Array};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{BinaryType, Event, MessageEvent, WebSocket};

use super::{Heartbeat, HeartbeatFrame, SocketMessage, WsConnect};
use crate::error::{Error, Result};
use crate::transport::BrowserRelay;
use crate::types::{Overflow, StreamConfig};

enum BrowserEvent {
    Open,
    Message(SocketMessage),
    Disconnected,
}

pub(super) struct Socket {
    socket: WebSocket,
    events: mpsc::Receiver<BrowserEvent>,
    disconnected: Rc<Cell<bool>>,
    _on_open: Closure<dyn FnMut(Event)>,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_error: Closure<dyn FnMut(Event)>,
    _on_close: Closure<dyn FnMut(Event)>,
}

impl Socket {
    pub(super) async fn next(&mut self) -> Option<Result<SocketMessage>> {
        loop {
            if self.events.is_empty() && self.disconnected.get() {
                return Some(Ok(SocketMessage::Closed));
            }
            return match self.events.recv().await? {
                BrowserEvent::Open => continue,
                BrowserEvent::Message(message) => Some(Ok(message)),
                BrowserEvent::Disconnected => Some(Ok(SocketMessage::Closed)),
            };
        }
    }

    pub(super) async fn send_heartbeat(&mut self, frame: HeartbeatFrame) -> Result<()> {
        match frame {
            HeartbeatFrame::Text(text) => self
                .socket
                .send_with_str(text)
                .map_err(|error| js_error("could not send browser WebSocket heartbeat", error)),
            // Browsers answer protocol Ping frames automatically and do not expose
            // control-frame sending to JavaScript.
            HeartbeatFrame::Ping => Ok(()),
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.socket.set_onopen(None);
        self.socket.set_onmessage(None);
        self.socket.set_onerror(None);
        self.socket.set_onclose(None);
        let _ = self.socket.close();
    }
}

pub(super) async fn open(connect: &WsConnect, config: &StreamConfig) -> Result<Socket> {
    validate_config(config)?;

    validate_upstream_url(&connect.url)?;
    let relay = relay_for(connect)?;
    let socket = WebSocket::new(relay.map_or(connect.url.as_str(), |relay| &relay.websocket))
        .map_err(|error| js_error("could not open browser WebSocket", error))?;
    socket.set_binary_type(BinaryType::Arraybuffer);
    let (sender, events) = mpsc::channel(config.buffer_size.max(1));
    let disconnected = Rc::new(Cell::new(false));

    let opened = sender.clone();
    let on_open = Closure::new(move |_: Event| {
        let _ = opened.try_send(BrowserEvent::Open);
    });
    socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));

    let messages = sender.clone();
    let on_message = Closure::new(move |event: MessageEvent| {
        // ponytail: classic browser WebSocket has no inbound backpressure API;
        // move to WebSocketStream or a relay if queued browser frames become a limit.
        let data = event.data();
        let message = if let Some(text) = data.as_string() {
            SocketMessage::Text(text)
        } else if data.is_instance_of::<ArrayBuffer>() {
            SocketMessage::Binary(Uint8Array::new(&data).to_vec())
        } else {
            return;
        };
        let _ = messages.try_send(BrowserEvent::Message(message));
    });
    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

    let errors = sender.clone();
    let errored = Rc::clone(&disconnected);
    let on_error = Closure::new(move |_: Event| {
        errored.set(true);
        let _ = errors.try_send(BrowserEvent::Disconnected);
    });
    socket.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    let closed = Rc::clone(&disconnected);
    let on_close = Closure::new(move |_: Event| {
        closed.set(true);
        let _ = sender.try_send(BrowserEvent::Disconnected);
    });
    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));

    let mut socket = Socket {
        socket,
        events,
        disconnected,
        _on_open: on_open,
        _on_message: on_message,
        _on_error: on_error,
        _on_close: on_close,
    };

    match socket.events.recv().await {
        Some(BrowserEvent::Open) => {}
        Some(BrowserEvent::Disconnected) | None => {
            return Err(Error::transport("browser WebSocket connection failed"));
        }
        Some(BrowserEvent::Message(_)) => {
            return Err(Error::transport(
                "browser WebSocket delivered data before opening",
            ));
        }
    }

    if let Some(relay) = relay {
        let headers = connect
            .headers
            .as_ref()
            .expect("relay routing requires a header factory")()?;
        let subscribe = (connect.subscribe)()?;
        let init = serde_json::to_string(&RelayWsInit {
            url: &connect.url,
            headers: &headers,
            subscribe: &subscribe,
        })
        .map_err(|error| Error::transport(format!("could not encode relay init frame: {error}")))?;
        socket
            .socket
            .send_with_str(&init)
            .map_err(|error| js_error("could not send browser relay init frame", error))?;
        wait_for_relay_ready(&mut socket, relay).await?;
    } else {
        for frame in (connect.subscribe)()? {
            socket.socket.send_with_str(&frame).map_err(|error| {
                js_error("could not send browser WebSocket subscription", error)
            })?;
        }
    }

    Ok(socket)
}

fn validate_config(config: &StreamConfig) -> Result<()> {
    if matches!(config.overflow, Overflow::Backpressure) {
        return Err(Error::invalid_request(
            "overflow",
            "browser WebSocket cannot apply network backpressure; use Overflow::DropNewest",
        ));
    }
    Ok(())
}

#[derive(Serialize)]
struct RelayWsInit<'a> {
    url: &'a str,
    headers: &'a [(String, String)],
    subscribe: &'a [String],
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RelayWsControl {
    Ready,
    Error { detail: String },
}

fn relay_for(connect: &WsConnect) -> Result<Option<&'static BrowserRelay>> {
    if connect.headers.is_none() {
        return Ok(None);
    }

    crate::transport::browser_relay().map(Some).ok_or_else(|| {
        Error::invalid_request(
            "relay_url",
            "browser WebSocket custom headers require configure_browser_relay",
        )
    })
}

fn validate_upstream_url(url: &str) -> Result<()> {
    let url = reqwest::Url::parse(url)
        .map_err(|error| Error::invalid_request("url", error.to_string()))?;
    if !matches!(url.scheme(), "ws" | "wss")
        || url.host().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(Error::invalid_request(
            "url",
            "browser WebSocket URL must use ws or wss without credentials or a fragment",
        ));
    }
    Ok(())
}

async fn wait_for_relay_ready(socket: &mut Socket, relay: &BrowserRelay) -> Result<()> {
    match socket.events.recv().await {
        Some(BrowserEvent::Message(SocketMessage::Text(frame))) => {
            match serde_json::from_str::<RelayWsControl>(&frame) {
                Ok(RelayWsControl::Ready) => Ok(()),
                Ok(RelayWsControl::Error { detail }) => Err(Error::transport(format!(
                    "browser relay rejected WebSocket init: {detail}"
                ))),
                Err(error) => Err(Error::transport(format!(
                    "browser relay returned an invalid ready frame from {}: {error}",
                    relay.websocket
                ))),
            }
        }
        Some(BrowserEvent::Disconnected) | None => Err(Error::transport(
            "browser relay disconnected before its ready frame",
        )),
        Some(BrowserEvent::Message(SocketMessage::Binary(_))) => Err(Error::transport(
            "browser relay returned binary data before its ready frame",
        )),
        Some(BrowserEvent::Message(SocketMessage::Closed)) => Err(Error::transport(
            "browser relay closed before its ready frame",
        )),
        Some(BrowserEvent::Open) => Err(Error::transport(
            "browser relay emitted a duplicate open event before its ready frame",
        )),
        #[cfg(not(target_arch = "wasm32"))]
        Some(BrowserEvent::Message(SocketMessage::Activity)) => unreachable!(),
    }
}

fn js_error(context: &str, error: JsValue) -> Error {
    let detail = error.as_string().unwrap_or_else(|| format!("{error:?}"));
    Error::transport(format!("{context}: {detail}"))
}

pub(super) fn spawn(future: impl Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(future);
}

pub(super) async fn sleep(duration: Duration) {
    timer(duration).await;
}

type Timer = Pin<Box<dyn Future<Output = ()>>>;

pub(super) struct Idle {
    timer: Timer,
    disabled: bool,
}

impl Idle {
    pub(super) fn new(duration: Duration, heartbeat: Option<&Heartbeat>) -> Self {
        let disabled = heartbeat.is_some_and(|heartbeat| heartbeat.frame == HeartbeatFrame::Ping);
        Self {
            timer: if disabled {
                Box::pin(std::future::pending())
            } else {
                timer(duration)
            },
            disabled,
        }
    }

    pub(super) fn reset(&mut self, duration: Duration) {
        if !self.disabled {
            self.timer = timer(duration);
        }
    }
}

impl Future for Idle {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.timer.as_mut().poll(cx)
    }
}

fn timer(duration: Duration) -> Timer {
    Box::pin(async move {
        let mut remaining = duration.as_millis();
        while remaining > 0 {
            let step = remaining.min(u32::MAX.into()) as u32;
            TimeoutFuture::new(step).await;
            remaining -= u128::from(step);
        }
    })
}

pub(super) struct Pulse {
    frame: HeartbeatFrame,
    interval: Duration,
    timer: Timer,
}

pub(super) fn pulse(heartbeat: Option<&Heartbeat>) -> Option<Pulse> {
    heartbeat.and_then(|heartbeat| match heartbeat.frame {
        HeartbeatFrame::Text(_) => Some(Pulse {
            frame: heartbeat.frame,
            interval: heartbeat.interval,
            timer: timer(heartbeat.interval),
        }),
        HeartbeatFrame::Ping => None,
    })
}

pub(super) async fn due(pulse: Option<&mut Pulse>) -> HeartbeatFrame {
    match pulse {
        Some(pulse) => {
            pulse.timer.as_mut().await;
            pulse.timer = timer(pulse.interval);
            pulse.frame
        }
        None => std::future::pending().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_headers_without_a_relay_are_a_non_retryable_request_error() {
        let connect = WsConnect {
            url: "wss://example.invalid".into(),
            headers: Some(Box::new(|| Ok(Vec::new()))),
            subscribe: WsConnect::fixed(Vec::new()),
            heartbeat: None,
        };

        let error = relay_for(&connect).expect_err("headers need a relay");
        assert!(matches!(error, Error::InvalidRequest { .. }));
        assert!(!error.is_retryable());
    }

    #[test]
    fn browser_backpressure_is_rejected_before_a_socket_is_opened() {
        let error = validate_config(&StreamConfig::default()).expect_err("cannot pause WebSocket");

        assert!(matches!(error, Error::InvalidRequest { .. }));
    }

    #[test]
    fn relay_init_frame_preserves_fresh_headers_and_subscriptions() {
        let headers = vec![("authorization".to_string(), "Bearer token".to_string())];
        let subscribe = vec![r#"{"type":"myOrder"}"#.to_string()];
        let frame = serde_json::to_value(RelayWsInit {
            url: "wss://exchange.example/private",
            headers: &headers,
            subscribe: &subscribe,
        })
        .unwrap();

        assert!(frame.get("type").is_none());
        assert_eq!(frame["url"], "wss://exchange.example/private");
        assert_eq!(frame["headers"][0][0], "authorization");
        assert_eq!(frame["subscribe"][0], r#"{"type":"myOrder"}"#);
    }

    #[test]
    fn protocol_ping_uses_browser_control_frames_instead_of_a_timer() {
        let heartbeat = Heartbeat {
            interval: Duration::from_secs(1),
            frame: HeartbeatFrame::Ping,
            min_idle_timeout: Duration::from_secs(1),
        };

        assert!(pulse(Some(&heartbeat)).is_none());
        assert!(Idle::new(Duration::from_millis(1), Some(&heartbeat)).disabled);
    }
}
