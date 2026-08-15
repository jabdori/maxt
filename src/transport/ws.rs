//! Shared WebSocket connection lifecycle for provider adapters.
//!
//! Headers and subscription frames are recreated for every handshake. Heartbeats
//! continue while backpressure waits for the consumer, and inbound silence
//! triggers reconnects. Backoff resets only after inbound traffic, while the
//! reconnect-attempt budget never resets. [`Overflow::DropNewest`] may discard
//! data or errors, but a reconnect notification remains pending until delivered.

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_core::Stream;
#[cfg(all(test, not(target_arch = "wasm32")))]
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot, watch};
#[cfg(all(test, not(target_arch = "wasm32")))]
use tokio_tungstenite::tungstenite::Message;

use crate::error::{Error, Result};
use crate::types::{Overflow, StreamConfig};

#[cfg(target_arch = "wasm32")]
#[path = "ws/browser.rs"]
mod platform;
#[cfg(not(target_arch = "wasm32"))]
#[path = "ws/native.rs"]
mod platform;

use platform::{Idle, Pulse, Socket};

/// Events emitted by a WebSocket session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WsCommand {
    /// A text frame arrived.
    Text(String),
    /// A binary frame arrived.
    Binary(Vec<u8>),
    /// The socket reconnected and subscription frames were sent again.
    Reconnected,
}

/// Creates fresh headers for each opening handshake.
pub(crate) type WsHeaders = Box<dyn Fn() -> Result<Vec<(String, String)>> + Send + Sync>;

/// Creates fresh subscription frames for each connection.
pub(crate) type WsSubscribe = Box<dyn Fn() -> Result<Vec<String>> + Send + Sync>;

/// How to open one connection.
pub(crate) struct WsConnect {
    /// The `wss://` URL to open.
    pub(crate) url: String,
    /// Per-handshake headers, or `None` for no custom headers.
    pub(crate) headers: Option<WsHeaders>,
    /// Frames created and sent immediately after each handshake.
    pub(crate) subscribe: WsSubscribe,
    /// Optional heartbeat and provider-specific idle-timeout floor.
    pub(crate) heartbeat: Option<Heartbeat>,
}

impl WsConnect {
    /// Reuses immutable subscription frames across connections.
    ///
    /// Signed, nonce-bearing, or time-sensitive frames require a custom
    /// [`WsSubscribe`] instead.
    pub(crate) fn fixed(frames: Vec<String>) -> WsSubscribe {
        Box::new(move || Ok(frames.clone()))
    }
}

/// Client heartbeat settings for one connection.
#[derive(Debug, Clone)]
pub(crate) struct Heartbeat {
    /// Delay between heartbeat frames.
    pub(crate) interval: Duration,
    /// What one heartbeat puts on the wire.
    pub(crate) frame: HeartbeatFrame,
    /// Minimum idle timeout supported by the provider's liveness cadence.
    ///
    /// This raises a smaller [`StreamConfig::idle_timeout_ms`] value.
    pub(crate) min_idle_timeout: Duration,
}

/// Frame kind used for a provider heartbeat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeartbeatFrame {
    /// An application-level text heartbeat.
    Text(&'static str),
    /// A protocol-level WebSocket ping.
    Ping,
}

#[cfg(all(test, not(target_arch = "wasm32")))]
impl HeartbeatFrame {
    fn message(self) -> Message {
        match self {
            Self::Text(text) => Message::Text(text.into()),
            Self::Ping => Message::Ping(Vec::new().into()),
        }
    }
}

/// Signals a WebSocket lifecycle task and waits until it has stopped.
#[derive(Clone)]
pub(crate) struct WsCloseHandle {
    cancel: watch::Sender<bool>,
    completed: watch::Receiver<bool>,
}

/// Sends one provider operation frame through an existing WebSocket session.
///
/// This stays crate-private: providers own their request/response protocols,
/// while the shared session only owns socket writes and lifecycle handling.
#[derive(Clone)]
pub(crate) struct WsSendHandle {
    sender: mpsc::Sender<WsOutbound>,
    connection_epoch: watch::Receiver<u64>,
}

impl WsSendHandle {
    /// Observes reconnects for an operation tied to this exact socket.
    pub(crate) fn connection_epoch(&self) -> watch::Receiver<u64> {
        self.connection_epoch.clone()
    }

    /// Waits until the lifecycle task has written `text` to the active socket.
    pub(crate) async fn send_text(&self, text: String) -> Result<()> {
        let (sent, received) = oneshot::channel();
        let connection_epoch = *self.connection_epoch.borrow();
        self.sender
            .send(WsOutbound {
                text,
                connection_epoch,
                sent,
            })
            .await
            .map_err(|_| {
                Error::transport("WebSocket session closed before sending an operation")
            })?;
        received
            .await
            .map_err(|_| Error::transport("WebSocket session stopped while sending an operation"))?
    }
}

struct WsOutbound {
    text: String,
    connection_epoch: u64,
    sent: oneshot::Sender<Result<()>>,
}

impl WsCloseHandle {
    fn signal(&self) {
        self.cancel.send_if_modified(|cancelled| {
            if *cancelled {
                false
            } else {
                *cancelled = true;
                true
            }
        });
    }

    pub(crate) async fn close(&self) -> Result<()> {
        self.signal();
        let mut completed = self.completed.clone();
        while !*completed.borrow_and_update() {
            completed.changed().await.map_err(|_| {
                Error::adapter("WebSocket task ended without publishing completion")
            })?;
        }
        Ok(())
    }
}

fn close_channels() -> (WsCloseHandle, watch::Receiver<bool>, watch::Sender<bool>) {
    let (cancel, cancelled) = watch::channel(false);
    let (completion, completed) = watch::channel(false);
    (WsCloseHandle { cancel, completed }, cancelled, completion)
}

/// Waits until cancellation has been signalled.
async fn wait_for_cancel(cancelled: &mut watch::Receiver<bool>) {
    if *cancelled.borrow_and_update() {
        return;
    }
    let _ = cancelled.changed().await;
}

/// A live WebSocket session exposed as a frame stream.
///
/// Dropping the receiver stops the background task and closes its socket.
pub(crate) struct WsSession {
    events: mpsc::Receiver<Result<WsCommand>>,
    close: WsCloseHandle,
    send: WsSendHandle,
}

impl WsSession {
    pub(crate) fn close_handle(&self) -> WsCloseHandle {
        self.close.clone()
    }

    /// Returns the internal operation sender for this exact socket session.
    pub(crate) fn send_handle(&self) -> WsSendHandle {
        self.send.clone()
    }
}

impl std::fmt::Debug for WsSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsSession").finish_non_exhaustive()
    }
}

impl Stream for WsSession {
    type Item = Result<WsCommand>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.events.poll_recv(cx)
    }
}

impl Drop for WsSession {
    fn drop(&mut self) {
        self.close.signal();
    }
}

/// Opens the initial connection and starts its lifecycle task.
#[cfg(target_arch = "wasm32")]
pub(crate) async fn connect(connect: WsConnect, config: &StreamConfig) -> Result<WsSession> {
    send_wrapper::SendWrapper::new(connect_inner(connect, config)).await
}

/// Opens the initial connection and starts its lifecycle task.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn connect(connect: WsConnect, config: &StreamConfig) -> Result<WsSession> {
    connect_inner(connect, config).await
}

async fn connect_inner(connect: WsConnect, config: &StreamConfig) -> Result<WsSession> {
    let (sender, events) = mpsc::channel(config.buffer_size.max(1));
    let (outbound, requests) = mpsc::channel(1);
    let (connection_epoch, current_epoch) = watch::channel(0_u64);
    let (close, cancelled, completed) = close_channels();
    let config = config.clone();

    // Initial connection failures are returned directly to the caller.
    let socket = open(&connect, &config).await?;

    platform::spawn(async move {
        run(
            connect,
            config,
            socket,
            sender,
            requests,
            connection_epoch,
            cancelled,
        )
        .await;
        completed.send_replace(true);
    });

    Ok(WsSession {
        events,
        close,
        send: WsSendHandle {
            sender: outbound,
            connection_epoch: current_epoch,
        },
    })
}

async fn open(connect: &WsConnect, config: &StreamConfig) -> Result<Socket> {
    platform::open(connect, config).await
}

enum SocketMessage {
    Text(String),
    Binary(Vec<u8>),
    /// Protocol control traffic that only resets the idle timer.
    #[cfg(not(target_arch = "wasm32"))]
    Activity,
    Closed,
}

/// Returns the caller timeout raised to the provider's minimum when necessary.
fn idle_timeout(config: &StreamConfig, heartbeat: Option<&Heartbeat>) -> Duration {
    let asked = Duration::from_millis(config.idle_timeout_ms);
    let floor = heartbeat.map_or(Duration::ZERO, |heartbeat| heartbeat.min_idle_timeout);

    asked.max(floor)
}

/// Returns whether consecutive mute reconnects should be reported.
fn worth_reporting(consecutive_failures: u32) -> bool {
    consecutive_failures >= RECONNECT_FAILURES_BEFORE_REPORTING
}

/// Consecutive mute reconnects tolerated before reporting transport errors.
const RECONNECT_FAILURES_BEFORE_REPORTING: u32 = 3;

async fn run(
    connect: WsConnect,
    config: StreamConfig,
    mut socket: Socket,
    sender: mpsc::Sender<Result<WsCommand>>,
    mut requests: mpsc::Receiver<WsOutbound>,
    connection_epoch: watch::Sender<u64>,
    mut cancelled: watch::Receiver<bool>,
) {
    let idle_timeout = idle_timeout(&config, connect.heartbeat.as_ref());
    // Total reconnect budget; inbound traffic does not reset it.
    let mut attempt = 0_u32;
    // Consecutive mute reconnects scale backoff and error reporting.
    let mut mute = 0_u32;
    // A reconnected socket must notify the consumer before new data.
    let mut reconnected = false;
    let mut pump_cancelled = cancelled.clone();

    loop {
        let current_epoch = *connection_epoch.borrow();
        let pumped = tokio::select! {
            biased;
            () = wait_for_cancel(&mut cancelled) => return,
            pumped = pump(
                &mut socket,
                &sender,
                idle_timeout,
                &connect,
                &config,
                std::mem::take(&mut reconnected),
                &mut requests,
                current_epoch,
                &mut pump_cancelled,
            ) => pumped,
        };
        let carried = match pumped {
            Pump::Cancelled | Pump::ConsumerGone => return,
            Pump::Disconnected { carried } => carried,
        };

        if carried {
            // Inbound traffic resets backoff but not the reconnect budget.
            mute = 0;
        } else if worth_reporting(mute) {
            // A repeatedly mute connection is distinguishable from a quiet feed.
            match deliver_until_cancelled(
                &mut cancelled,
                &sender,
                Err(Error::transport(format!(
                    "reconnected {mute} times without the exchange sending anything"
                ))),
                config.overflow,
            )
            .await
            {
                Some(Delivery::Sent | Delivery::Dropped) => {}
                Some(Delivery::ConsumerGone) | None => return,
            }
        }

        // Reconnect with capped exponential backoff; mute sockets keep the streak.
        loop {
            attempt += 1;
            mute += 1;
            if config
                .max_reconnect_attempts
                .is_some_and(|max| attempt > max)
            {
                // `DropNewest` may discard this final error; stream termination remains.
                let _ = deliver_until_cancelled(
                    &mut cancelled,
                    &sender,
                    Err(Error::transport(format!(
                        "gave up reconnecting after {} attempts",
                        attempt - 1
                    ))),
                    config.overflow,
                )
                .await;
                return;
            }

            let backoff = backoff_delay(&config, mute);
            tokio::select! {
                biased;
                () = wait_for_cancel(&mut cancelled) => return,
                () = platform::sleep(backoff) => {}
            }

            let reopened = tokio::select! {
                biased;
                () = wait_for_cancel(&mut cancelled) => return,
                reopened = open(&connect, &config) => reopened,
            };
            match reopened {
                Ok(reopened) => {
                    socket = reopened;
                    connection_epoch.send_modify(|epoch| *epoch += 1);
                    reconnected = true;
                    break;
                }
                Err(error) => {
                    if worth_reporting(mute) {
                        match deliver_until_cancelled(
                            &mut cancelled,
                            &sender,
                            Err(error),
                            config.overflow,
                        )
                        .await
                        {
                            Some(Delivery::Sent | Delivery::Dropped) => {}
                            Some(Delivery::ConsumerGone) | None => return,
                        }
                    }
                    continue;
                }
            }
        }
    }
}

enum Pump {
    /// The socket ended; `carried` records whether any data frame arrived.
    Disconnected {
        carried: bool,
    },
    ConsumerGone,
    Cancelled,
}

#[allow(clippy::too_many_arguments)]
async fn pump(
    socket: &mut Socket,
    sender: &mpsc::Sender<Result<WsCommand>>,
    idle_timeout: Duration,
    connect: &WsConnect,
    config: &StreamConfig,
    reconnected: bool,
    requests: &mut mpsc::Receiver<WsOutbound>,
    connection_epoch: u64,
    cancelled: &mut watch::Receiver<bool>,
) -> Pump {
    // The first heartbeat is one full interval after the handshake.
    let mut pulse = platform::pulse(connect.heartbeat.as_ref());
    // Only inbound traffic resets the deadline; consumer waits do not count as idle.
    let mut idle = Idle::new(idle_timeout, connect.heartbeat.as_ref());
    // This controls backoff only; reconnect budget accounting is separate.
    let mut carried = false;
    // Retry a dropped reconnect notice before delivering post-gap data.
    let mut owed = reconnected;

    if owed {
        match hand_over(
            socket,
            sender,
            Ok(WsCommand::Reconnected),
            config.overflow,
            &mut pulse,
            requests,
            connection_epoch,
        )
        .await
        {
            Handover::Delivered => owed = false,
            Handover::Dropped => {}
            Handover::ConsumerGone => return Pump::ConsumerGone,
            Handover::SocketDead => return Pump::Disconnected { carried },
        }
        idle.reset(idle_timeout);
    }

    loop {
        let next = tokio::select! {
            biased;
            () = wait_for_cancel(cancelled) => return Pump::Cancelled,
            // Treat inbound silence as a disconnected socket.
            () = &mut idle => return Pump::Disconnected { carried },
            frame = platform::due(pulse.as_mut()) => {
                if socket.send_heartbeat(frame).await.is_err() {
                    return Pump::Disconnected { carried };
                }
                continue;
            }
            request = requests.recv() => {
                let Some(request) = request else {
                    continue;
                };
                if request.connection_epoch != connection_epoch {
                    let _ = request.sent.send(Err(Error::transport(
                        "WebSocket reconnected before sending an operation",
                    )));
                    continue;
                }
                match socket.send_text(&request.text).await {
                    Ok(()) => {
                        let _ = request.sent.send(Ok(()));
                        continue;
                    }
                    Err(error) => {
                        let _ = request.sent.send(Err(error));
                        return Pump::Disconnected { carried };
                    }
                }
            }
            next = socket.next() => next,
        };

        let message = match next {
            None => return Pump::Disconnected { carried },
            Some(Err(_)) => return Pump::Disconnected { carried },
            Some(Ok(message)) => message,
        };
        idle.reset(idle_timeout);

        let event = match message {
            SocketMessage::Text(text) => WsCommand::Text(text),
            SocketMessage::Binary(bytes) => WsCommand::Binary(bytes),
            #[cfg(not(target_arch = "wasm32"))]
            SocketMessage::Activity => continue,
            SocketMessage::Closed => return Pump::Disconnected { carried },
        };
        carried = true;

        // A reconnect notice must precede every post-gap data frame.
        if owed {
            match hand_over(
                socket,
                sender,
                Ok(WsCommand::Reconnected),
                config.overflow,
                &mut pulse,
                requests,
                connection_epoch,
            )
            .await
            {
                Handover::Delivered => owed = false,
                Handover::Dropped => {}
                Handover::ConsumerGone => return Pump::ConsumerGone,
                Handover::SocketDead => return Pump::Disconnected { carried },
            }
            idle.reset(idle_timeout);
            if owed {
                // Post-gap data cannot overtake a pending reconnect notice.
                continue;
            }
        }

        match hand_over(
            socket,
            sender,
            Ok(event),
            config.overflow,
            &mut pulse,
            requests,
            connection_epoch,
        )
        .await
        {
            Handover::Delivered | Handover::Dropped => {}
            Handover::ConsumerGone => return Pump::ConsumerGone,
            Handover::SocketDead => return Pump::Disconnected { carried },
        }
        // Exclude time spent waiting on a backpressured consumer from idle time.
        idle.reset(idle_timeout);
    }
}

/// Result of transferring one event while servicing heartbeats.
enum Handover {
    /// The consumer has it.
    Delivered,
    /// [`Overflow::DropNewest`] discarded the event from a full buffer.
    Dropped,
    /// The consumer receiver was dropped.
    ConsumerGone,
    /// A heartbeat write failed after the event was delivered.
    SocketDead,
}

/// Transfers one event without suspending heartbeat writes.
///
/// Under [`Overflow::Backpressure`], reservation and heartbeat waits race. If a
/// heartbeat write fails, the already-read event is still delivered before the
/// socket is reported dead.
async fn hand_over(
    socket: &mut Socket,
    sender: &mpsc::Sender<Result<WsCommand>>,
    event: Result<WsCommand>,
    overflow: Overflow,
    pulse: &mut Option<Pulse>,
    requests: &mut mpsc::Receiver<WsOutbound>,
    connection_epoch: u64,
) -> Handover {
    if !matches!(overflow, Overflow::Backpressure) {
        return match deliver(sender, event, overflow).await {
            Delivery::Sent => Handover::Delivered,
            Delivery::Dropped => Handover::Dropped,
            Delivery::ConsumerGone => Handover::ConsumerGone,
        };
    }

    loop {
        tokio::select! {
            // Reserving is cancellation-safe when a heartbeat wins the race.
            reserved = sender.reserve() => return match reserved {
                Ok(permit) => {
                    permit.send(event);
                    Handover::Delivered
                }
                Err(_) => Handover::ConsumerGone,
            },
            frame = platform::due(pulse.as_mut()) => {
                if socket.send_heartbeat(frame).await.is_err() {
                    // Backpressure still guarantees delivery of the already-read event.
                    return match sender.reserve().await {
                        Ok(permit) => {
                            permit.send(event);
                            Handover::SocketDead
                        }
                        Err(_) => Handover::ConsumerGone,
                    };
                }
            }
            request = requests.recv() => {
                let Some(request) = request else {
                    continue;
                };
                if request.connection_epoch != connection_epoch {
                    let _ = request.sent.send(Err(Error::transport(
                        "WebSocket reconnected before sending an operation",
                    )));
                    continue;
                }
                if let Err(error) = socket.send_text(&request.text).await {
                    let _ = request.sent.send(Err(error));
                    return Handover::SocketDead;
                }
                let _ = request.sent.send(Ok(()));
            }
        }
    }
}

enum Delivery {
    Sent,
    Dropped,
    ConsumerGone,
}

async fn deliver_until_cancelled(
    cancelled: &mut watch::Receiver<bool>,
    sender: &mpsc::Sender<Result<WsCommand>>,
    event: Result<WsCommand>,
    overflow: Overflow,
) -> Option<Delivery> {
    tokio::select! {
        biased;
        () = wait_for_cancel(cancelled) => None,
        delivered = deliver(sender, event, overflow) => Some(delivered),
    }
}

async fn deliver(
    sender: &mpsc::Sender<Result<WsCommand>>,
    event: Result<WsCommand>,
    overflow: Overflow,
) -> Delivery {
    match overflow {
        // Wait until the consumer catches up.
        Overflow::Backpressure => match sender.send(event).await {
            Ok(()) => Delivery::Sent,
            Err(_) => Delivery::ConsumerGone,
        },
        // Discard the new event instead of waiting on a full buffer.
        Overflow::DropNewest => match sender.try_send(event) {
            Ok(()) => Delivery::Sent,
            Err(mpsc::error::TrySendError::Full(_)) => Delivery::Dropped,
            Err(mpsc::error::TrySendError::Closed(_)) => Delivery::ConsumerGone,
        },
    }
}

/// Returns capped exponential backoff with a one-millisecond minimum.
fn backoff_delay(config: &StreamConfig, mute_run: u32) -> Duration {
    let doubling = mute_run.saturating_sub(1).min(16);
    let delay = config
        .initial_reconnect_delay_ms
        .max(1)
        .saturating_mul(1_u64 << doubling)
        .min(config.max_reconnect_delay_ms.max(1));
    Duration::from_millis(delay)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn config() -> StreamConfig {
        StreamConfig {
            initial_reconnect_delay_ms: 1_000,
            max_reconnect_delay_ms: 30_000,
            ..StreamConfig::default()
        }
    }

    #[tokio::test]
    async fn close_handle_signals_once_and_waits_for_task_completion() {
        let (handle, mut cancelled, completed) = close_channels();
        let close_one = tokio::spawn({
            let handle = handle.clone();
            async move { handle.close().await }
        });
        let close_two = tokio::spawn({
            let handle = handle.clone();
            async move { handle.close().await }
        });

        cancelled.changed().await.unwrap();
        assert!(*cancelled.borrow());
        assert!(!close_one.is_finished());
        assert!(!close_two.is_finished());

        completed.send_replace(true);
        close_one.await.unwrap().unwrap();
        close_two.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn close_handle_reports_a_task_that_omits_completion() {
        let (handle, _cancelled, completed) = close_channels();
        drop(completed);

        assert!(matches!(handle.close().await, Err(Error::Adapter { .. })));
    }

    /// Accepts one connection, sends a greeting, and optionally remains open.
    async fn one_shot_server(stay: bool) -> (std::net::SocketAddr, mpsc::Receiver<Message>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a free local port");
        let address = listener.local_addr().expect("a bound address");
        let (sent, received) = mpsc::channel(8);

        tokio::spawn(async move {
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            drop(listener);

            let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
                return;
            };
            if socket.send(Message::Text("hello".into())).await.is_err() {
                return;
            }
            if !stay {
                let _ = socket.close(None).await;
                return;
            }
            while let Some(Ok(message)) = socket.next().await {
                if sent.send(message).await.is_err() {
                    return;
                }
            }
        });

        (address, received)
    }

    /// Sends data continuously while counting client heartbeat frames.
    async fn chatty_server(every: Duration) -> (std::net::SocketAddr, Arc<AtomicUsize>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a free local port");
        let address = listener.local_addr().expect("a bound address");
        let heard = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&heard);

        tokio::spawn(async move {
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            drop(listener);

            let Ok(socket) = tokio_tungstenite::accept_async(stream).await else {
                return;
            };
            let (mut write, mut read) = socket.split();

            tokio::spawn(async move {
                while let Some(Ok(message)) = read.next().await {
                    if matches!(message, Message::Text(_) | Message::Ping(_)) {
                        counter.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });

            loop {
                if write.send(Message::Text("tick".into())).await.is_err() {
                    return;
                }
                tokio::time::sleep(every).await;
            }
        });

        (address, heard)
    }

    /// Accepts and immediately closes connections, optionally after one frame.
    ///
    /// The returned receiver counts accepted connections without blocking the
    /// server's reconnect churn.
    async fn flapping_server(
        greet: Option<&'static str>,
    ) -> (std::net::SocketAddr, mpsc::Receiver<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a free local port");
        let address = listener.local_addr().expect("a bound address");
        let (accepted, connections) = mpsc::channel(4_096);

        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                if matches!(
                    accepted.try_send(()),
                    Err(mpsc::error::TrySendError::Closed(()))
                ) {
                    return;
                }
                let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
                    continue;
                };
                if let Some(frame) = greet {
                    let _ = socket.send(Message::Text(frame.into())).await;
                }
                let _ = socket.close(None).await;
            }
        });

        (address, connections)
    }

    /// Closes the first connection, then keeps a second connection alive.
    ///
    /// The second connection starts sending only after observing a client
    /// heartbeat, providing a deterministic reconnect-notice ordering point.
    async fn flaps_once_then_stays(every: Duration) -> (std::net::SocketAddr, mpsc::Receiver<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a free local port");
        let address = listener.local_addr().expect("a bound address");
        let (beat, heard) = mpsc::channel(1);

        tokio::spawn(async move {
            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            if let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await {
                let _ = socket.send(Message::Text("hello".into())).await;
                let _ = socket.close(None).await;
            }

            let Ok((stream, _)) = listener.accept().await else {
                return;
            };
            let Ok(socket) = tokio_tungstenite::accept_async(stream).await else {
                return;
            };
            let (mut write, mut read) = socket.split();

            if read.next().await.is_none() || beat.send(()).await.is_err() {
                return;
            }
            loop {
                if write.send(Message::Text("tick".into())).await.is_err() {
                    return;
                }
                tokio::time::sleep(every).await;
            }
        });

        (address, heard)
    }

    /// Records the authorization header from each handshake before closing.
    #[allow(clippy::result_large_err)] // `tungstenite` fixes the callback's error response type.
    async fn header_recording_server() -> (std::net::SocketAddr, mpsc::Receiver<String>) {
        use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a free local port");
        let address = listener.local_addr().expect("a bound address");
        let (seen, presented) = mpsc::channel(16);

        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let seen = seen.clone();
                let socket = tokio_tungstenite::accept_hdr_async(
                    stream,
                    move |request: &Request, response: Response| {
                        let value = request
                            .headers()
                            .get("authorization")
                            .and_then(|value| value.to_str().ok())
                            .unwrap_or_default()
                            .to_string();
                        let _ = seen.try_send(value);
                        Ok(response)
                    },
                )
                .await;

                let Ok(mut socket) = socket else {
                    continue;
                };
                let _ = socket.close(None).await;
            }
        });

        (address, presented)
    }

    /// Records each connection's first subscription frame before closing.
    async fn subscribe_recording_server() -> (std::net::SocketAddr, mpsc::Receiver<String>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("a free local port");
        let address = listener.local_addr().expect("a bound address");
        let (seen, subscribed) = mpsc::channel(16);

        tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                let Ok(mut socket) = tokio_tungstenite::accept_async(stream).await else {
                    continue;
                };
                if let Some(Ok(Message::Text(frame))) = socket.next().await
                    && seen.send(frame.to_string()).await.is_err()
                {
                    return;
                }
                let _ = socket.close(None).await;
            }
        });

        (address, subscribed)
    }

    #[tokio::test]
    async fn every_connection_subscribes_with_frames_minted_for_it() {
        let (address, mut subscribed) = subscribe_recording_server().await;
        let signed = AtomicUsize::new(0);
        let config = StreamConfig {
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 1,
            idle_timeout_ms: 60_000,
            ..config()
        };

        let _session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                // Model a time-sensitive subscription signature.
                subscribe: Box::new(move || {
                    let nth = signed.fetch_add(1, Ordering::Relaxed);
                    Ok(vec![format!("subscribe {nth}")])
                }),
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        let mut seen = Vec::new();
        for _ in 0..3 {
            seen.push(
                tokio::time::timeout(Duration::from_secs(5), subscribed.recv())
                    .await
                    .expect("another connection before the deadline")
                    .expect("the server still listening"),
            );
        }

        assert_eq!(seen, ["subscribe 0", "subscribe 1", "subscribe 2"]);
    }

    #[tokio::test]
    async fn an_operation_frame_uses_the_existing_socket() {
        let (address, mut received) = one_shot_server(true).await;
        let session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(vec!["subscribe".to_owned()]),
                heartbeat: None,
            },
            &config(),
        )
        .await
        .expect("the initial connection");

        session
            .send_handle()
            .send_text("LIST_SUBSCRIPTIONS".to_owned())
            .await
            .expect("the operation frame is written");

        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(5), received.recv()).await,
            Ok(Some(Message::Text(text))) if text == "subscribe"
        ));
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(5), received.recv()).await,
            Ok(Some(Message::Text(text))) if text == "LIST_SUBSCRIPTIONS"
        ));
    }

    #[tokio::test]
    async fn a_subscription_that_cannot_be_minted_fails_the_connection_it_was_for() {
        let (address, _subscribed) = subscribe_recording_server().await;

        let error = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: Box::new(|| Err(Error::auth("no secret key to sign with"))),
                heartbeat: None,
            },
            &config(),
        )
        .await
        .expect_err("a subscription that cannot be signed");

        assert!(matches!(error, Error::Auth { .. }), "{error}");
    }

    #[test]
    fn an_exchange_that_needs_a_slower_idle_timer_than_the_caller_asked_for_gets_one() {
        let config = StreamConfig {
            idle_timeout_ms: 30_000,
            ..config()
        };
        let heartbeat = Heartbeat {
            interval: Duration::from_secs(15),
            frame: HeartbeatFrame::Ping,
            min_idle_timeout: Duration::from_secs(240),
        };

        assert_eq!(
            idle_timeout(&config, Some(&heartbeat)),
            Duration::from_secs(240)
        );
        let patient = StreamConfig {
            idle_timeout_ms: 600_000,
            ..config.clone()
        };
        assert_eq!(
            idle_timeout(&patient, Some(&heartbeat)),
            Duration::from_secs(600)
        );
        assert_eq!(idle_timeout(&config, None), Duration::from_secs(30));
    }

    #[test]
    fn a_heartbeat_goes_out_as_the_kind_of_frame_it_names() {
        assert_eq!(
            HeartbeatFrame::Text("PING").message(),
            Message::Text("PING".into())
        );
        assert!(matches!(
            HeartbeatFrame::Ping.message(),
            Message::Ping(payload) if payload.is_empty()
        ));
    }

    #[test]
    fn a_blip_stays_quiet_and_a_lasting_outage_does_not() {
        assert!(!worth_reporting(1));
        assert!(!worth_reporting(2));
        assert!(worth_reporting(3));
        assert!(worth_reporting(u32::MAX));
    }

    #[tokio::test]
    async fn a_quiet_connection_sends_the_heartbeat_on_its_own_interval() {
        // Exercise application-level and protocol-level heartbeats.
        for frame in [
            HeartbeatFrame::Text(r#"{"method":"ping"}"#),
            HeartbeatFrame::Ping,
        ] {
            let (address, mut received) = one_shot_server(true).await;
            let config = StreamConfig {
                idle_timeout_ms: 60_000,
                ..config()
            };

            let _session = connect(
                WsConnect {
                    url: format!("ws://{address}"),
                    headers: None,
                    subscribe: WsConnect::fixed(Vec::new()),
                    heartbeat: Some(Heartbeat {
                        interval: Duration::from_millis(50),
                        frame,
                        min_idle_timeout: Duration::from_millis(60_000),
                    }),
                },
                &config,
            )
            .await
            .expect("the first connection");

            for _ in 0..2 {
                let message = tokio::time::timeout(Duration::from_secs(5), received.recv())
                    .await
                    .expect("a heartbeat before the deadline")
                    .expect("the server still reading");

                assert_eq!(message, frame.message(), "{frame:?}");
            }
        }
    }

    #[tokio::test]
    async fn a_reconnect_that_never_succeeds_says_so_instead_of_going_quiet() {
        let (address, _received) = one_shot_server(false).await;
        let config = StreamConfig {
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 1,
            idle_timeout_ms: 60_000,
            ..config()
        };

        let mut session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        assert!(matches!(
            session.next().await,
            Some(Ok(WsCommand::Text(text))) if text == "hello"
        ));

        // Unlimited retries still report a persistent outage.
        let reported = tokio::time::timeout(Duration::from_secs(5), session.next())
            .await
            .expect("a report before the deadline");

        assert!(matches!(reported, Some(Err(Error::Transport { .. }))));
    }

    #[test]
    fn backoff_doubles_from_the_initial_delay() {
        let config = config();

        assert_eq!(backoff_delay(&config, 1), Duration::from_millis(1_000));
        assert_eq!(backoff_delay(&config, 2), Duration::from_millis(2_000));
        assert_eq!(backoff_delay(&config, 3), Duration::from_millis(4_000));
        assert_eq!(backoff_delay(&config, 4), Duration::from_millis(8_000));
    }

    #[test]
    fn backoff_stops_growing_at_the_cap() {
        let config = config();

        assert_eq!(backoff_delay(&config, 6), Duration::from_millis(30_000));
        assert_eq!(backoff_delay(&config, 60), Duration::from_millis(30_000));
        assert_eq!(
            backoff_delay(&config, u32::MAX),
            Duration::from_millis(30_000)
        );
    }

    #[test]
    fn a_zero_delay_still_sleeps_between_reconnects() {
        // Public zero values must not create a busy reconnect loop.
        let no_initial = StreamConfig {
            initial_reconnect_delay_ms: 0,
            ..config()
        };
        let no_ceiling = StreamConfig {
            max_reconnect_delay_ms: 0,
            ..config()
        };

        for attempt in [1, 2, 8, u32::MAX] {
            assert!(
                backoff_delay(&no_initial, attempt) >= Duration::from_millis(1),
                "attempt {attempt} from a zero initial delay"
            );
            assert!(
                backoff_delay(&no_ceiling, attempt) >= Duration::from_millis(1),
                "attempt {attempt} under a zero ceiling"
            );
        }

        assert_eq!(backoff_delay(&no_initial, 4), Duration::from_millis(8));
    }

    #[tokio::test]
    async fn backpressure_waits_instead_of_losing_events() {
        let (sender, mut receiver) = mpsc::channel(1);

        assert!(matches!(
            deliver(
                &sender,
                Ok(WsCommand::Text("first".into())),
                Overflow::Backpressure
            )
            .await,
            Delivery::Sent
        ));

        // A full buffer blocks the producer under backpressure.
        let blocked = tokio::time::timeout(
            Duration::from_millis(50),
            deliver(
                &sender,
                Ok(WsCommand::Text("second".into())),
                Overflow::Backpressure,
            ),
        )
        .await;
        assert!(blocked.is_err(), "backpressure should have blocked");

        assert!(matches!(
            receiver.recv().await,
            Some(Ok(WsCommand::Text(text))) if text == "first"
        ));
    }

    #[tokio::test]
    async fn a_full_buffer_drops_rather_than_stalling_when_asked_to() {
        let (sender, _receiver) = mpsc::channel(1);

        assert!(matches!(
            deliver(
                &sender,
                Ok(WsCommand::Text("first".into())),
                Overflow::DropNewest
            )
            .await,
            Delivery::Sent
        ));
        assert!(matches!(
            deliver(
                &sender,
                Ok(WsCommand::Text("second".into())),
                Overflow::DropNewest
            )
            .await,
            Delivery::Dropped
        ));
    }

    #[tokio::test]
    async fn a_dropped_consumer_stops_the_connection() {
        let (sender, receiver) = mpsc::channel(4);
        drop(receiver);

        assert!(matches!(
            deliver(
                &sender,
                Ok(WsCommand::Text("frame".into())),
                Overflow::Backpressure
            )
            .await,
            Delivery::ConsumerGone
        ));
        assert!(matches!(
            deliver(
                &sender,
                Ok(WsCommand::Text("frame".into())),
                Overflow::DropNewest
            )
            .await,
            Delivery::ConsumerGone
        ));
    }

    #[tokio::test]
    async fn every_handshake_is_opened_with_headers_minted_for_it() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let (address, mut presented) = header_recording_server().await;
        let signed = AtomicUsize::new(0);
        let config = StreamConfig {
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 1,
            idle_timeout_ms: 60_000,
            ..config()
        };

        let _session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                // Model a time-sensitive authentication header.
                headers: Some(Box::new(move || {
                    let nth = signed.fetch_add(1, Ordering::Relaxed);
                    Ok(vec![("authorization".to_string(), format!("Bearer {nth}"))])
                })),
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        let mut seen = Vec::new();
        for _ in 0..3 {
            seen.push(
                tokio::time::timeout(Duration::from_secs(5), presented.recv())
                    .await
                    .expect("another handshake before the deadline")
                    .expect("the server still listening"),
            );
        }

        assert_eq!(seen, ["Bearer 0", "Bearer 1", "Bearer 2"]);
    }

    #[tokio::test]
    async fn a_consumer_that_stalls_keeps_its_connection_and_its_heartbeat() {
        // Exercise both heartbeat kinds while delivery is backpressured.
        for frame in [
            HeartbeatFrame::Text(r#"{"method":"ping"}"#),
            HeartbeatFrame::Ping,
        ] {
            let (address, heard) = chatty_server(Duration::from_millis(5)).await;
            let config = StreamConfig {
                // Keep delivery blocked after the first frame.
                buffer_size: 1,
                overflow: Overflow::Backpressure,
                idle_timeout_ms: 60_000,
                ..config()
            };

            let mut session = connect(
                WsConnect {
                    url: format!("ws://{address}"),
                    headers: None,
                    subscribe: WsConnect::fixed(Vec::new()),
                    heartbeat: Some(Heartbeat {
                        interval: Duration::from_millis(20),
                        frame,
                        min_idle_timeout: Duration::ZERO,
                    }),
                },
                &config,
            )
            .await
            .expect("the first connection");

            assert!(matches!(session.next().await, Some(Ok(WsCommand::Text(_)))));

            // Heartbeats must continue while the consumer leaves the buffer full.
            for _ in 0..6 {
                let before = heard.load(Ordering::Relaxed);
                let kept_beating = tokio::time::timeout(Duration::from_secs(5), async {
                    while heard.load(Ordering::Relaxed) - before < 3 {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                })
                .await;
                assert!(
                    kept_beating.is_ok(),
                    "{frame:?}: {} heartbeats reached the server while the consumer was stalled",
                    heard.load(Ordering::Relaxed) - before
                );

                // Backpressure must preserve every data item.
                let item = tokio::time::timeout(Duration::from_secs(5), session.next())
                    .await
                    .expect("the connection to still be delivering");
                assert!(matches!(item, Some(Ok(WsCommand::Text(_)))), "{item:?}");
            }
        }
    }

    #[tokio::test]
    async fn the_idle_timer_does_not_count_time_spent_waiting_on_the_consumer() {
        let (address, _heard) = chatty_server(Duration::from_millis(5)).await;
        let config = StreamConfig {
            // Keep delivery blocked after the first frame.
            buffer_size: 1,
            overflow: Overflow::Backpressure,
            idle_timeout_ms: 500,
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 1,
            ..config()
        };

        let mut session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        assert!(matches!(session.next().await, Some(Ok(WsCommand::Text(_)))));

        // Consumer backpressure alone must not expire the inbound idle timer.
        tokio::time::sleep(Duration::from_millis(3_000)).await;

        for _ in 0..3 {
            let item = tokio::time::timeout(Duration::from_secs(5), session.next())
                .await
                .expect("the connection to still be delivering");
            assert!(matches!(item, Some(Ok(WsCommand::Text(_)))), "{item:?}");
        }
    }

    #[tokio::test]
    async fn a_connection_the_exchange_never_speaks_on_backs_off_and_says_so() {
        let (address, _connections) = flapping_server(None).await;
        let config = StreamConfig {
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 50,
            idle_timeout_ms: 60_000,
            ..config()
        };

        let mut session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        // Successful handshakes with no inbound frames still report an outage.
        let reported = tokio::time::timeout(Duration::from_secs(5), async {
            while let Some(event) = session.next().await {
                if let Err(error) = event {
                    return Some(error);
                }
            }
            None
        })
        .await
        .expect("a report before the deadline");

        assert!(
            matches!(
                &reported,
                Some(Error::Transport { detail })
                    if detail.contains("without the exchange sending anything")
            ),
            "{reported:?}"
        );
    }

    #[tokio::test]
    async fn the_attempt_limit_bounds_a_venue_that_sends_a_frame_on_every_connection() {
        // Raw frames cannot prove subscription success, so they do not reset the budget.
        let (address, mut connections) =
            flapping_server(Some(r#"{"code":-1121,"msg":"Invalid symbol."}"#)).await;
        let config = StreamConfig {
            buffer_size: 64,
            overflow: Overflow::DropNewest,
            max_reconnect_attempts: Some(3),
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 1,
            idle_timeout_ms: 60_000,
        };

        let mut session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        let ended = tokio::time::timeout(Duration::from_secs(20), async {
            while session.next().await.is_some() {}
        })
        .await;

        let mut opened = 0;
        while connections.try_recv().is_ok() {
            opened += 1;
        }

        assert!(
            ended.is_ok(),
            "the stream never ended; {opened} connections opened against a budget of 3"
        );
        // One initial connection plus exactly three reconnect attempts.
        assert_eq!(opened, 4, "connections opened against a budget of 3");
    }

    #[tokio::test]
    async fn a_venue_that_recycles_sockets_keeps_reconnecting_at_the_first_delay() {
        // Inbound traffic resets backoff for the next recycled connection.
        let (address, _connections) = flapping_server(Some("hello")).await;
        let config = StreamConfig {
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 30_000,
            idle_timeout_ms: 60_000,
            ..config()
        };

        let mut session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        let seen = tokio::time::timeout(Duration::from_secs(20), async {
            let mut reconnects = 0;
            while reconnects < 20 {
                match session.next().await {
                    Some(Ok(WsCommand::Reconnected)) => reconnects += 1,
                    Some(Ok(WsCommand::Text(_))) => {}
                    other => return Err(format!("the stream ended or faulted: {other:?}")),
                }
            }
            Ok(())
        })
        .await;

        assert!(
            matches!(seen, Ok(Ok(()))),
            "twenty reconnects at the first delay: {seen:?}"
        );
    }

    #[tokio::test]
    async fn the_attempt_limit_bounds_a_venue_that_accepts_and_says_nothing() {
        let (address, _connections) = flapping_server(None).await;
        let config = StreamConfig {
            buffer_size: 64,
            overflow: Overflow::Backpressure,
            max_reconnect_attempts: Some(2),
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 1,
            idle_timeout_ms: 60_000,
        };

        let mut session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        let ended = tokio::time::timeout(Duration::from_secs(10), async {
            while session.next().await.is_some() {}
        })
        .await;

        assert!(ended.is_ok(), "the stream retried past its attempt limit");
    }

    #[tokio::test]
    async fn the_news_of_a_reconnect_outlives_a_full_buffer() {
        let (address, mut heard) = flaps_once_then_stays(Duration::from_millis(20)).await;
        let config = StreamConfig {
            // The greeting fills the buffer before the reconnect notice arrives.
            buffer_size: 1,
            overflow: Overflow::DropNewest,
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 1,
            idle_timeout_ms: 60_000,
            ..config()
        };

        let mut session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: Some(Heartbeat {
                    interval: Duration::from_millis(20),
                    frame: HeartbeatFrame::Text("beat"),
                    min_idle_timeout: Duration::ZERO,
                }),
            },
            &config,
        )
        .await
        .expect("the first connection");

        // The observed heartbeat confirms the notice met the full buffer.
        tokio::time::timeout(Duration::from_secs(10), heard.recv())
            .await
            .expect("the reconnected socket to reach its heartbeat")
            .expect("the server still running");

        assert!(matches!(
            session.next().await,
            Some(Ok(WsCommand::Text(text))) if text == "hello"
        ));

        // The retained reconnect notice must precede post-gap data.
        let next = tokio::time::timeout(Duration::from_secs(10), session.next())
            .await
            .expect("another event before the deadline");
        assert!(matches!(next, Some(Ok(WsCommand::Reconnected))), "{next:?}");
    }

    #[tokio::test]
    async fn a_reconnect_does_not_wait_on_a_consumer_that_asked_never_to_be_waited_on() {
        let (address, mut connections) = flapping_server(Some("hello")).await;
        let config = StreamConfig {
            buffer_size: 1,
            overflow: Overflow::DropNewest,
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 1,
            idle_timeout_ms: 60_000,
            ..config()
        };

        let _session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        // A pending reconnect notice must not block `DropNewest` reconnects.
        for _ in 0..4 {
            tokio::time::timeout(Duration::from_secs(5), connections.recv())
                .await
                .expect("another connection before the deadline")
                .expect("the server still listening");
        }
    }

    #[tokio::test]
    async fn giving_up_does_not_wait_on_a_consumer_that_asked_never_to_be_waited_on() {
        let (address, _received) = one_shot_server(false).await;
        let config = StreamConfig {
            buffer_size: 1,
            overflow: Overflow::DropNewest,
            max_reconnect_attempts: Some(1),
            initial_reconnect_delay_ms: 1,
            max_reconnect_delay_ms: 1,
            idle_timeout_ms: 60_000,
        };

        let mut session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        // Keep the buffer full until the lifecycle task reaches its attempt limit.
        tokio::time::timeout(Duration::from_secs(5), async {
            while !session.events.is_closed() {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .expect("the connection to give up before the deadline");

        assert!(matches!(
            session.next().await,
            Some(Ok(WsCommand::Text(text))) if text == "hello"
        ));

        // The final error may be dropped, but the stream must still terminate.
        let ended = tokio::time::timeout(Duration::from_secs(5), session.next())
            .await
            .expect("the stream to end rather than wait on the consumer");
        assert!(ended.is_none(), "{ended:?}");
    }

    /// Returns a locally closed socket whose writes fail immediately.
    async fn write_dead_socket() -> Socket {
        let (address, _received) = one_shot_server(true).await;
        let mut socket = open(
            &WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &StreamConfig::default(),
        )
        .await
        .expect("a connection to the local server");

        socket.close_for_test().await;
        assert!(
            socket.send_heartbeat(HeartbeatFrame::Ping).await.is_err(),
            "a closed socket should refuse writes"
        );

        socket
    }

    #[tokio::test]
    async fn a_heartbeat_that_cannot_be_written_does_not_cost_the_event_in_hand() {
        let mut socket = write_dead_socket().await;
        // Fill the buffer so heartbeat failure wins the reservation race.
        let (sender, mut receiver) = mpsc::channel(1);
        sender
            .send(Ok(WsCommand::Text("already queued".into())))
            .await
            .expect("room in an empty buffer");

        let mut pulse = Some((
            HeartbeatFrame::Ping,
            tokio::time::interval(Duration::from_millis(1)),
        ));
        let (_outbound, mut requests) = mpsc::channel(1);

        let handed = tokio::spawn(async move {
            hand_over(
                &mut socket,
                &sender,
                Ok(WsCommand::Text("read before the socket died".into())),
                Overflow::Backpressure,
                &mut pulse,
                &mut requests,
                0,
            )
            .await
        });

        tokio::time::sleep(Duration::from_millis(200)).await;

        // Backpressure preserves the event already read from the failed socket.
        assert!(matches!(
            receiver.recv().await,
            Some(Ok(WsCommand::Text(text))) if text == "already queued"
        ));
        let kept = tokio::time::timeout(Duration::from_secs(5), receiver.recv())
            .await
            .expect("the event to be delivered rather than dropped with the socket");
        assert!(
            matches!(kept, Some(Ok(WsCommand::Text(ref text))) if text == "read before the socket died"),
            "{kept:?}"
        );

        assert!(matches!(
            handed.await.expect("the hand-over task"),
            Handover::SocketDead
        ));
    }

    #[tokio::test]
    async fn a_consumer_that_left_while_the_socket_died_ends_the_connection() {
        let mut socket = write_dead_socket().await;
        let (sender, receiver) = mpsc::channel(1);
        sender
            .send(Ok(WsCommand::Text("already queued".into())))
            .await
            .expect("room in an empty buffer");
        let mut pulse = Some((
            HeartbeatFrame::Ping,
            tokio::time::interval(Duration::from_millis(1)),
        ));
        let (_outbound, mut requests) = mpsc::channel(1);

        let handed = tokio::spawn(async move {
            hand_over(
                &mut socket,
                &sender,
                Ok(WsCommand::Text("nobody left to read it".into())),
                Overflow::Backpressure,
                &mut pulse,
                &mut requests,
                0,
            )
            .await
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
        // Dropping the receiver must cancel the pending reservation.
        drop(receiver);

        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(5), handed)
                .await
                .expect("the wait to end with the consumer")
                .expect("the hand-over task"),
            Handover::ConsumerGone
        ));
    }

    #[tokio::test]
    async fn an_unreachable_url_fails_at_connect_not_later_on_the_stream() {
        let error = connect(
            WsConnect {
                url: "not-a-websocket-url".to_string(),
                headers: None,
                subscribe: WsConnect::fixed(Vec::new()),
                heartbeat: None,
            },
            &config(),
        )
        .await
        .unwrap_err();

        assert!(matches!(error, Error::Transport { .. }));
    }
}
