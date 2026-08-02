//! Native WebSocket and timer primitives.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use super::{Heartbeat, HeartbeatFrame, SocketMessage, WsConnect};
use crate::error::{Error, Result};
use crate::types::StreamConfig;

pub(super) struct Socket(
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
);

impl Socket {
    pub(super) async fn next(&mut self) -> Option<Result<SocketMessage>> {
        self.0.next().await.map(|message| {
            message
                .map(|message| match message {
                    Message::Text(text) => SocketMessage::Text(text.to_string()),
                    Message::Binary(bytes) => SocketMessage::Binary(bytes.to_vec()),
                    Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => {
                        SocketMessage::Activity
                    }
                    Message::Close(_) => SocketMessage::Closed,
                })
                .map_err(|error| Error::transport(error.to_string()))
        })
    }

    pub(super) async fn send_heartbeat(&mut self, frame: HeartbeatFrame) -> Result<()> {
        let message = match frame {
            HeartbeatFrame::Text(text) => Message::Text(text.into()),
            HeartbeatFrame::Ping => Message::Ping(Vec::new().into()),
        };
        self.0
            .send(message)
            .await
            .map_err(|error| Error::transport(error.to_string()))
    }

    #[cfg(test)]
    pub(super) async fn close_for_test(&mut self) {
        let _ = self.0.close(None).await;
    }
}

pub(super) async fn open(connect: &WsConnect, _config: &StreamConfig) -> Result<Socket> {
    crate::transport::ensure_crypto_provider();

    let mut request = connect
        .url
        .as_str()
        .into_client_request()
        .map_err(|error| Error::transport(format!("invalid WebSocket URL: {error}")))?;

    // Recreate time-sensitive headers for every handshake.
    if let Some(headers) = &connect.headers {
        for (name, value) in headers()? {
            let parsed: http::HeaderName = name
                .parse()
                .map_err(|_| Error::transport(format!("invalid header name `{name}`")))?;
            let value: http::HeaderValue = value
                .parse()
                .map_err(|_| Error::transport(format!("invalid value for header `{name}`")))?;
            request.headers_mut().insert(parsed, value);
        }
    }

    let (socket, _) = tokio_tungstenite::connect_async(request)
        .await
        .map_err(|error| Error::transport(error.to_string()))?;
    let mut socket = Socket(socket);

    for frame in (connect.subscribe)()? {
        socket
            .0
            .send(Message::Text(frame.into()))
            .await
            .map_err(|error| Error::transport(error.to_string()))?;
    }

    Ok(socket)
}

pub(super) fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(future);
}

pub(super) async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

pub(super) struct Idle(Pin<Box<tokio::time::Sleep>>);

impl Idle {
    pub(super) fn new(duration: Duration, _heartbeat: Option<&Heartbeat>) -> Self {
        Self(Box::pin(tokio::time::sleep(duration)))
    }

    pub(super) fn reset(&mut self, duration: Duration) {
        self.0
            .as_mut()
            .reset(tokio::time::Instant::now() + duration);
    }
}

impl Future for Idle {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}

pub(super) type Pulse = (HeartbeatFrame, tokio::time::Interval);

pub(super) fn pulse(heartbeat: Option<&Heartbeat>) -> Option<Pulse> {
    heartbeat.map(|heartbeat| {
        let mut ticks = tokio::time::interval_at(
            tokio::time::Instant::now() + heartbeat.interval,
            heartbeat.interval,
        );
        ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        (heartbeat.frame, ticks)
    })
}

pub(super) async fn due(pulse: Option<&mut Pulse>) -> HeartbeatFrame {
    match pulse {
        Some((frame, ticks)) => {
            ticks.tick().await;
            *frame
        }
        None => std::future::pending().await,
    }
}
