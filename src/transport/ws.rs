//! Live WebSocket connections, with reconnects.
//!
//! One session owns one socket. It re-subscribes after every reconnect, keeps
//! a quiet socket alive with the heartbeat its exchange reads, drops a
//! connection that has stopped answering, and applies the caller's overflow
//! policy when the consumer falls behind. An adapter only has to say which URL
//! to open, which frames to send on connect, and what its exchange accepts as
//! a keepalive.
//!
//! Nothing authenticating is carried between connections. Headers and subscribe
//! frames are both minted per handshake, so a credential with a clock in it is
//! as fresh on the tenth reconnect as on the first, and no socket is opened with
//! a signature the exchange has already stopped accepting.
//!
//! Waiting on a consumer never stops the heartbeat. The two share one task, and
//! a wait that silenced the keepalive would turn a consumer that pauses into a
//! connection the exchange closes for looking dead, so a stalled consumer
//! stops the reads and nothing else.
//!
//! Opening a socket is not the same as having one. What resets the backoff is
//! a connection the exchange spoke on, not a handshake that succeeded, so an
//! endpoint that accepts and then says nothing is backed off and eventually
//! reported instead of being reconnected to as fast as the loop can manage.
//!
//! The attempt budget is a separate count, and nothing resets it. Nothing here
//! parses a frame: a venue's rejection of the subscription is a text frame like
//! its data, so "the exchange spoke" cannot be read as "the subscription
//! works", and a budget reset by it would leave a venue that answers every
//! connection with an error frame reconnecting forever. So
//! [`StreamConfig::max_reconnect_attempts`] bounds reconnects outright,
//! whatever came of them, and the price is that it bounds the healthy ones too.
//!
//! The one thing [`Overflow::DropNewest`] never discards is the news of a
//! reconnect. It is kept and offered again ahead of every event read after it
//! until the consumer has it, because a consumer that missed it goes on
//! trusting state the gap invalidated.

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures_core::Stream;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use crate::error::{Error, Result};
use crate::types::{Overflow, StreamConfig};

/// What a connection tells the adapter above it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WsCommand {
    /// A text frame arrived.
    Text(String),
    /// A binary frame arrived. Hyperliquid and Binance both use text only;
    /// Upbit and Bithumb answer some requests in binary.
    Binary(Vec<u8>),
    /// The socket dropped and was re-established, and the subscribe frames were
    /// sent again.
    Reconnected,
}

/// Mints the headers one opening handshake carries.
///
/// Called once per handshake, so every reconnect gets its own headers rather
/// than a replay of the first ones. A private stream whose credential is a
/// signed token with a clock in it is only authenticated for as long as that
/// token is fresh, and a connection that lives longer than one token has to
/// sign another.
pub(crate) type WsHeaders = Box<dyn Fn() -> Result<Vec<(String, String)>> + Send + Sync>;

/// Mints the frames one connection subscribes with.
///
/// Called once per handshake for the same reason [`WsHeaders`] is: a subscribe
/// frame that signs a clock is only good for as long as the exchange's receive
/// window, and a reconnect that replayed the frame the first connect was opened
/// with would be refused once an outage outlasted that window. The reconnect
/// loop has no way to notice, so it would go on replaying a dead frame onto
/// sockets that open and carry nothing.
pub(crate) type WsSubscribe = Box<dyn Fn() -> Result<Vec<String>> + Send + Sync>;

/// How to open one connection.
pub(crate) struct WsConnect {
    /// The `wss://` URL to open.
    pub(crate) url: String,
    /// Headers for the opening handshake, for private streams that
    /// authenticate there instead of in a frame.
    ///
    /// `None` for a public stream, which needs none. Signing failures surface
    /// as a failed connection attempt, which the reconnect loop then retries.
    pub(crate) headers: Option<WsHeaders>,
    /// Frames to send immediately on connect, minted again for every reconnect.
    ///
    /// [`WsConnect::fixed`] is the whole of it for a subscription named by
    /// nothing but a market and a feed. Minting failures surface as a failed
    /// connection attempt, which the reconnect loop then retries.
    pub(crate) subscribe: WsSubscribe,
    /// What to send while the exchange has nothing to say, and how long silence
    /// may last before the socket counts as dead.
    ///
    /// `None` leaves the connection entirely at the mercy of inbound traffic,
    /// which every exchange `maxt` speaks to will eventually cut off.
    pub(crate) heartbeat: Option<Heartbeat>,
}

impl WsConnect {
    /// Subscribe frames that are the same on every connection.
    ///
    /// What a public feed sends: a market and a stream name say the whole of
    /// it, so the frame a reconnect owes the exchange is the frame the first
    /// connect sent. A frame carrying a signature, a nonce, or a clock is not
    /// one of these and has to be minted per handshake instead.
    pub(crate) fn fixed(frames: Vec<String>) -> WsSubscribe {
        Box::new(move || Ok(frames.clone()))
    }
}

/// Client-initiated traffic that keeps a quiet connection open.
///
/// Every exchange here disconnects a socket it has heard nothing from, and
/// [`StreamConfig::idle_timeout_ms`] independently gives up on a socket that
/// has said nothing. A heartbeat answers both. The frame goes out on
/// `interval`, the exchange's reply is the inbound traffic the idle timer
/// needs, and `min_idle_timeout` keeps that timer above what the exchange's
/// own pace can satisfy.
#[derive(Debug, Clone)]
pub(crate) struct Heartbeat {
    /// How long to wait between heartbeats. Well under the silence the exchange
    /// disconnects for, so several go unanswered before anything is concluded.
    pub(crate) interval: Duration,
    /// What one heartbeat puts on the wire.
    pub(crate) frame: HeartbeatFrame,
    /// The shortest inbound silence this exchange's connections may be dropped
    /// for.
    ///
    /// Raises [`StreamConfig::idle_timeout_ms`] when the caller's value is
    /// under what the exchange's own liveness traffic can meet. A Binance user
    /// data stream on an account that never moves is server-pinged every three
    /// minutes and is healthy the whole time.
    pub(crate) min_idle_timeout: Duration,
}

/// What one heartbeat puts on the wire.
///
/// The two are not interchangeable: an exchange that reads every text frame as
/// a command answers an unknown one with an error, and an exchange whose
/// keepalive is defined at the application level may never see a protocol ping
/// at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeartbeatFrame {
    /// A text frame the exchange reads as an application-level ping, and
    /// answers with a control frame of its own.
    Text(&'static str),
    /// A WebSocket ping, answered with a pong by the server's protocol stack,
    /// below the exchange's API.
    Ping,
}

impl HeartbeatFrame {
    /// The message actually written to the socket.
    fn message(self) -> Message {
        match self {
            Self::Text(text) => Message::Text(text.into()),
            Self::Ping => Message::Ping(Vec::new().into()),
        }
    }
}

/// A live connection, as a stream of frames.
///
/// Dropping this closes the socket: the background task stops as soon as its
/// channel has no receiver.
pub(crate) struct WsSession {
    events: mpsc::Receiver<Result<WsCommand>>,
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

/// Opens a connection and starts the task that keeps it open.
pub(crate) async fn connect(connect: WsConnect, config: &StreamConfig) -> Result<WsSession> {
    let (sender, events) = mpsc::channel(config.buffer_size.max(1));
    let config = config.clone();

    // Fail the first connection in the caller's face rather than reporting a
    // healthy stream that immediately errors: a bad URL or a rejected
    // handshake is a caller mistake, not a transient fault.
    let socket = open(&connect).await?;

    tokio::spawn(run(connect, config, socket, sender));

    Ok(WsSession { events })
}

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn open(connect: &WsConnect) -> Result<Socket> {
    let mut request = connect
        .url
        .as_str()
        .into_client_request()
        .map_err(|err| Error::transport(format!("invalid WebSocket URL: {err}")))?;

    // Minted here rather than carried on `connect`, so that this handshake and
    // every reconnect after it present headers made for the moment they are
    // sent.
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

    let (mut socket, _) = tokio_tungstenite::connect_async(request)
        .await
        .map_err(|err| Error::transport(err.to_string()))?;

    // Minted here for the same reason the headers are: this handshake and every
    // reconnect after it subscribe with frames made for the moment they go out.
    for frame in (connect.subscribe)()? {
        socket
            .send(Message::Text(frame.into()))
            .await
            .map_err(|err| Error::transport(err.to_string()))?;
    }

    Ok(socket)
}

/// How long silence may last before the socket counts as dead.
///
/// The caller sets the timeout. An exchange whose healthy connections are
/// quieter than that would be torn down and rebuilt on a fixed cycle forever,
/// so the adapter's floor wins where it is higher.
fn idle_timeout(config: &StreamConfig, heartbeat: Option<&Heartbeat>) -> Duration {
    let asked = Duration::from_millis(config.idle_timeout_ms);
    let floor = heartbeat.map_or(Duration::ZERO, |heartbeat| heartbeat.min_idle_timeout);

    asked.max(floor)
}

/// Whether a reconnect that did not leave a working connection behind is worth
/// reporting.
///
/// The first few cover the ordinary case of an exchange restart or a route
/// that moved, where a reconnect succeeds a second later and the consumer
/// needs no warning. Past this the fault has stopped looking transient, and a
/// consumer told nothing cannot tell a dead stream from a quiet market.
fn worth_reporting(consecutive_failures: u32) -> bool {
    consecutive_failures >= RECONNECT_FAILURES_BEFORE_REPORTING
}

/// How many reconnects in a row may come to nothing before the connection says
/// so.
const RECONNECT_FAILURES_BEFORE_REPORTING: u32 = 3;

async fn run(
    connect: WsConnect,
    config: StreamConfig,
    mut socket: Socket,
    sender: mpsc::Sender<Result<WsCommand>>,
) {
    let idle_timeout = idle_timeout(&config, connect.heartbeat.as_ref());
    // Reconnects made, whatever came of them. What `max_reconnect_attempts`
    // bounds, and nothing resets it: judging a reconnect productive would mean
    // reading the frames it carried, which is the adapter's job and not
    // possible here, so a venue that answers every connection with a rejection
    // would reset this on every cycle and never be bounded at all.
    let mut attempt = 0_u32;
    // Reconnects since the exchange last sent anything. What scales the backoff
    // and decides what is worth reporting, kept apart from the budget so that a
    // venue recycling working sockets still reconnects at the first delay
    // rather than creeping to the ceiling.
    let mut mute = 0_u32;
    // Whether the current socket came from a reconnect, and so owes the
    // consumer word of the gap. Handed over inside `pump` rather than here,
    // because a consumer that is waited on for this news is waited on with a
    // socket already open and already owing the exchange its keepalive.
    let mut reconnected = false;

    loop {
        // Pump the current socket until it fails or goes quiet.
        let carried = match pump(
            &mut socket,
            &sender,
            idle_timeout,
            &connect,
            &config,
            std::mem::take(&mut reconnected),
        )
        .await
        {
            Pump::ConsumerGone => return,
            Pump::Disconnected { carried } => carried,
        };

        if carried {
            // The exchange spoke on this connection, so whatever ended it is a
            // fresh fault as far as the pace of retrying goes: the next backoff
            // starts from the first delay again. The budget is untouched, since
            // what it said is not something this layer read.
            mute = 0;
        } else if worth_reporting(mute) {
            // The reconnects keep succeeding onto a socket the exchange never
            // says anything on, which reaches the consumer as an unbroken run
            // of `Reconnected` unless it is said outright. Through the overflow
            // policy like every other report.
            match deliver(
                &sender,
                Err(Error::transport(format!(
                    "reconnected {mute} times without the exchange sending anything"
                ))),
                config.overflow,
            )
            .await
            {
                Delivery::Sent | Delivery::Dropped => {}
                Delivery::ConsumerGone => return,
            }
        }

        // Reconnect with exponential backoff, capped. The backoff counter
        // carries over from a connection the exchange never spoke on, so an
        // endpoint that accepts and stays mute backs off as if it had never
        // opened at all. The budget counter carries over from every connection.
        loop {
            attempt += 1;
            mute += 1;
            if config
                .max_reconnect_attempts
                .is_some_and(|max| attempt > max)
            {
                // Through the caller's policy like everything else: a consumer
                // that asked never to be waited on is not waited on for the
                // failure that ends its stream either. `DropNewest` may discard
                // it, and the stream ending is what is left to say so.
                let _ = deliver(
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
            tokio::time::sleep(backoff).await;

            match open(&connect).await {
                Ok(reopened) => {
                    socket = reopened;
                    reconnected = true;
                    break;
                }
                // Retrying forever in silence is indistinguishable from a market
                // with nothing to say, so once the failures stop looking
                // transient every one of them is reported.
                Err(error) => {
                    if worth_reporting(mute) {
                        match deliver(&sender, Err(error), config.overflow).await {
                            Delivery::Sent | Delivery::Dropped => {}
                            Delivery::ConsumerGone => return,
                        }
                    }
                    continue;
                }
            }
        }
    }
}

enum Pump {
    /// The socket ended. `carried` is whether the exchange sent anything on it
    /// first, which is all this layer can observe about a connection and is
    /// less than "it worked": the frame is raw, nothing here parses it, and a
    /// venue's rejection of the subscription arrives as one. So it sets how
    /// hard the next reconnect is backed off and never spends the attempt
    /// budget.
    Disconnected {
        carried: bool,
    },
    ConsumerGone,
}

async fn pump(
    socket: &mut Socket,
    sender: &mpsc::Sender<Result<WsCommand>>,
    idle_timeout: Duration,
    connect: &WsConnect,
    config: &StreamConfig,
    reconnected: bool,
) -> Pump {
    // The first heartbeat is one interval away, not immediate: the subscribe
    // frames have only just gone out.
    let mut pulse = connect.heartbeat.as_ref().map(|heartbeat| {
        let mut ticks = tokio::time::interval_at(
            tokio::time::Instant::now() + heartbeat.interval,
            heartbeat.interval,
        );
        // A heartbeat that came due while the consumer was being waited on is
        // not worth sending twice in a row to catch up.
        ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        (heartbeat.frame, ticks)
    });
    // Only inbound traffic pushes this out. Our own heartbeats must not, or a
    // socket that stopped answering would be kept alive by our writes alone.
    // It is re-armed both when a frame arrives and once that frame has reached
    // the consumer, so what it measures is inbound silence and never time spent
    // waiting on the consumer.
    let mut deadline = tokio::time::Instant::now() + idle_timeout;
    // Whether the exchange has said anything at all on this socket. What the
    // backoff is scaled by, so a mute connection is retried more gently than a
    // talkative one without a clock entering into it.
    let mut carried = false;
    // What the reconnect that opened this socket owes the consumer, if this is
    // not the first one. [`Overflow::DropNewest`] may find no room for it, and
    // this is the one event that policy must not discard: a consumer that never
    // hears of the gap goes on trusting a book and a balance the gap
    // invalidated, with nothing later to correct it. So it is kept and offered
    // again ahead of every event read after it, until the consumer has it.
    let mut owed = reconnected;

    if owed {
        match hand_over(
            socket,
            sender,
            Ok(WsCommand::Reconnected),
            config.overflow,
            &mut pulse,
        )
        .await
        {
            Handover::Delivered => owed = false,
            Handover::Dropped => {}
            Handover::ConsumerGone => return Pump::ConsumerGone,
            Handover::SocketDead => return Pump::Disconnected { carried },
        }
        deadline = tokio::time::Instant::now() + idle_timeout;
    }

    loop {
        let next = tokio::select! {
            // Nothing arrived for the whole idle window: treat the socket as
            // dead even though it never said so. Silent half-open connections
            // are the common failure here, not clean closes.
            () = tokio::time::sleep_until(deadline) => return Pump::Disconnected { carried },
            frame = due(pulse.as_mut()) => {
                // A write that fails is a dead socket found early, well before
                // the idle window would have said so.
                if socket.send(frame).await.is_err() {
                    return Pump::Disconnected { carried };
                }
                continue;
            }
            next = socket.next() => next,
        };

        let message = match next {
            None => return Pump::Disconnected { carried },
            Some(Err(_)) => return Pump::Disconnected { carried },
            Some(Ok(message)) => message,
        };
        deadline = tokio::time::Instant::now() + idle_timeout;

        let event = match message {
            Message::Text(text) => WsCommand::Text(text.to_string()),
            Message::Binary(bytes) => WsCommand::Binary(bytes.to_vec()),
            // tokio-tungstenite answers pings itself; the pong answering our own
            // heartbeat has already done its work by arriving.
            Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => continue,
            Message::Close(_) => return Pump::Disconnected { carried },
        };
        carried = true;

        // The gap notice goes first. Offered again here rather than only once
        // at the reconnect, because the buffer that had no room for it then may
        // have room now, and a consumer told of the gap after the data from the
        // far side of it resynchronizes onto state it is about to discard.
        if owed {
            match hand_over(
                socket,
                sender,
                Ok(WsCommand::Reconnected),
                config.overflow,
                &mut pulse,
            )
            .await
            {
                Handover::Delivered => owed = false,
                Handover::Dropped => {}
                Handover::ConsumerGone => return Pump::ConsumerGone,
                Handover::SocketDead => return Pump::Disconnected { carried },
            }
            deadline = tokio::time::Instant::now() + idle_timeout;
            if owed {
                // Still no room for the notice means no room for this event
                // either, and it must not overtake the notice. `DropNewest`
                // would have discarded it a line later regardless.
                continue;
            }
        }

        match hand_over(socket, sender, Ok(event), config.overflow, &mut pulse).await {
            Handover::Delivered | Handover::Dropped => {}
            Handover::ConsumerGone => return Pump::ConsumerGone,
            Handover::SocketDead => return Pump::Disconnected { carried },
        }
        // Re-armed after the hand-over, not only before it. Under
        // `Overflow::Backpressure` the line above waits for as long as the
        // consumer takes, and a deadline set before that wait is already spent
        // when it returns: a slow consumer would then tear down a socket that
        // had never stopped talking, and lose everything published across the
        // reconnect that policy exists to prevent losing.
        deadline = tokio::time::Instant::now() + idle_timeout;
    }
}

/// One connection's heartbeat clock: what to put on the wire, and when.
///
/// Made once per socket, so the first heartbeat of a reconnected connection is
/// one full interval after that reconnect rather than whenever the old socket
/// left the clock.
type Pulse = (HeartbeatFrame, tokio::time::Interval);

/// Waits until the next heartbeat is due, and answers with the frame to send.
///
/// A connection whose adapter named no heartbeat waits here forever instead, so
/// a `select!` arm holding this never fires and nothing that keeps one needs a
/// second case for the connection that has none.
async fn due(pulse: Option<&mut Pulse>) -> Message {
    match pulse {
        Some((frame, ticks)) => {
            ticks.tick().await;
            frame.message()
        }
        None => std::future::pending().await,
    }
}

/// What became of one event on its way to the consumer.
enum Handover {
    /// The consumer has it.
    Delivered,
    /// The buffer was full and [`Overflow::DropNewest`] discarded it. Told
    /// apart from `Delivered` because the news of a reconnect is offered again
    /// rather than lost, and nothing else here is.
    Dropped,
    /// The consumer is gone, so there is nothing left to read the socket for.
    ConsumerGone,
    /// A heartbeat that came due during the wait could not be written, so the
    /// socket is dead and reconnecting is what is left. The event was still
    /// handed to the consumer first: the socket dying is no reason to lose what
    /// had already been read off it.
    SocketDead,
}

/// Hands one event to the consumer without letting the socket's heartbeat lapse.
///
/// This is the one task that both waits on the consumer and owes the exchange
/// its keepalive, and under [`Overflow::Backpressure`] the wait is unbounded:
/// the consumer decides how long it lasts. A wait that also stopped the
/// heartbeat would let a consumer that pauses for longer than one interval be
/// read by the exchange as a dead peer and disconnected, turning a slow
/// consumer into a dropped connection, which is the exact failure the heartbeat
/// is there to prevent. So the heartbeat keeps going out while the consumer is
/// waited on, and the only thing a stalled consumer stops is reading.
///
/// [`Overflow::DropNewest`] never waits, so it never delayed a heartbeat in the
/// first place, and it takes the short path here.
///
/// A heartbeat that cannot be written means the socket is dead, and the event
/// read off it before it died is handed over anyway. Nothing is left to keep
/// alive at that point, so the wait for the consumer costs the connection
/// nothing, and [`Overflow::Backpressure`] loses nothing on the way out either.
async fn hand_over(
    socket: &mut Socket,
    sender: &mpsc::Sender<Result<WsCommand>>,
    event: Result<WsCommand>,
    overflow: Overflow,
    pulse: &mut Option<Pulse>,
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
            // Reserving room rather than sending outright, so that losing this
            // race to a heartbeat cannot cost the event: cancelling a reserve
            // gives up a place in the queue, and there is only one producer to
            // lose it to.
            reserved = sender.reserve() => return match reserved {
                Ok(permit) => {
                    permit.send(event);
                    Handover::Delivered
                }
                Err(_) => Handover::ConsumerGone,
            },
            frame = due(pulse.as_mut()) => {
                if socket.send(frame).await.is_err() {
                    // The socket is dead, the consumer is not, and the event is
                    // still in hand. Returning here would drop it, which is the
                    // one thing this policy promises never to happen. There is
                    // no heartbeat left to race, so the wait is plain.
                    return match sender.reserve().await {
                        Ok(permit) => {
                            permit.send(event);
                            Handover::SocketDead
                        }
                        Err(_) => Handover::ConsumerGone,
                    };
                }
            }
        }
    }
}

enum Delivery {
    Sent,
    Dropped,
    ConsumerGone,
}

async fn deliver(
    sender: &mpsc::Sender<Result<WsCommand>>,
    event: Result<WsCommand>,
    overflow: Overflow,
) -> Delivery {
    match overflow {
        // Stop reading the socket until the consumer catches up.
        Overflow::Backpressure => match sender.send(event).await {
            Ok(()) => Delivery::Sent,
            Err(_) => Delivery::ConsumerGone,
        },
        // A full buffer means the consumer is behind; discard rather than stall.
        Overflow::DropNewest => match sender.try_send(event) {
            Ok(()) => Delivery::Sent,
            Err(mpsc::error::TrySendError::Full(_)) => Delivery::Dropped,
            Err(mpsc::error::TrySendError::Closed(_)) => Delivery::ConsumerGone,
        },
    }
}

/// How long to wait before the next reconnect attempt.
///
/// Both delays are floored at a millisecond, because both fields are public and
/// zero in either one is a reconnect loop that never sleeps: doubling from zero
/// stays at zero however many attempts it takes, and a ceiling of zero flattens
/// every delay to nothing. A floor of one leaves the loop as fast as a caller
/// can ask for while keeping a socket that flaps from spending a core on it.
fn backoff_delay(config: &StreamConfig, mute_run: u32) -> Duration {
    let doubling = mute_run.saturating_sub(1).min(16);
    let delay = config
        .initial_reconnect_delay_ms
        .max(1)
        .saturating_mul(1_u64 << doubling)
        .min(config.max_reconnect_delay_ms.max(1));
    Duration::from_millis(delay)
}

#[cfg(test)]
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

    /// A server that accepts one connection and then stops existing, so every
    /// reconnect after the first is refused.
    ///
    /// It greets the client and, when `stay` is set, reads until that client
    /// goes away instead of hanging up on it. Returns the address it listened
    /// on and a receiver of everything the client sent.
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

    /// A server that accepts one connection, then talks without stopping and
    /// counts what the client says back.
    ///
    /// The only thing that can end this connection is the client, and it listens
    /// no further, so a client that hangs up on it cannot come back: a teardown
    /// shows as failed reconnects rather than as a gap in the frames.
    ///
    /// Talking and reading at once is what makes it usable for a stalled
    /// consumer: a server that only talked would fill the consumer's buffer but
    /// never see the heartbeats, and one that only read would never fill it. The
    /// count covers the two frames a heartbeat can be and nothing else the
    /// client sends, which on a subscription-free connection is nothing at all.
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

    /// A server that hangs up on every client as soon as it has accepted it, so
    /// a connection to it is a loop of connect, disconnect.
    ///
    /// `greet` is the one frame that goes out before the close, if any. `None`
    /// is an endpoint that accepts connections and never says anything on them.
    /// What the text says makes no difference to anything under test, which is
    /// the point: nothing at this layer parses a frame, so a venue's data and
    /// its rejection of the subscription are the same event here.
    ///
    /// Returns the address and a receiver that ticks once per accepted
    /// connection.
    ///
    /// The ticks are never waited on and the buffer is deeper than any test
    /// here fills, because a server that stalled on a full buffer would put a
    /// bound on the churn that the client had not put there, and one test
    /// counts exactly that churn. A dropped receiver still stops it, which is
    /// what keeps the task from outliving the test that started it.
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

    /// A server that greets its first client and hangs up, then keeps the
    /// second one and talks on it without stopping.
    ///
    /// Exactly one reconnect is what makes it useful: a server that flapped
    /// forever would hand the consumer a fresh notice after every drop, so a
    /// notice lost on one reconnect would be covered up by the next one.
    ///
    /// The second connection stays mute until it has heard the client's
    /// heartbeat, and the returned receiver ticks when it does. That is the
    /// ordering the test needs and cannot get from a clock: the client only
    /// sends a heartbeat from the loop it enters after the reconnect notice has
    /// been offered to the consumer and refused, so hearing one means the
    /// notice has already been through a full buffer once.
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

    /// A server that reports the `authorization` header each handshake arrived
    /// with, then hangs up so the client opens another one.
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

    /// A server that reports the first frame each connection subscribed with,
    /// then hangs up so the client opens another one.
    ///
    /// The frame is read before the close rather than after it, so what the
    /// receiver carries is what the client sent on that connection and not a
    /// frame left over from the previous one.
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

        // Held: dropping the session would stop the reconnects.
        let _session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                headers: None,
                // A stand-in for a frame that signs a clock. What subscribed
                // the first connection must not be what subscribes the next
                // one: Binance's spot user data frame carries a timestamp
                // inside its signature, so a replayed frame is refused once an
                // outage outlasts `recvWindow`, and the reconnect loop cannot
                // tell that from a socket that is merely quiet.
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
    async fn a_subscription_that_cannot_be_minted_fails_the_connection_it_was_for() {
        // Signing can fail: a key that is not usable as an HMAC key, a wallet
        // that is missing. The caller hears it rather than getting a stream
        // that opens and then carries nothing.
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
        // A caller who wants to wait longer than the floor still may.
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
        // Both kinds, because an adapter that names the wrong one gets its
        // connection closed rather than kept alive: three of the four exchanges
        // read a text frame as an application command, and Binance rejects one.
        for frame in [
            HeartbeatFrame::Text(r#"{"method":"ping"}"#),
            HeartbeatFrame::Ping,
        ] {
            let (address, mut received) = one_shot_server(true).await;
            let config = StreamConfig {
                idle_timeout_ms: 60_000,
                ..config()
            };

            // Held: dropping the session would close the socket under the server.
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

            // Twice, so this is an interval rather than a single frame on connect.
            for _ in 0..2 {
                let message = tokio::time::timeout(Duration::from_secs(5), received.recv())
                    .await
                    .expect("a heartbeat before the deadline")
                    .expect("the server still reading");

                // Byte for byte what the exchange would receive.
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

        // The server is gone and `max_reconnect_attempts` is `None`, so this
        // stream will retry forever. It must still be possible to tell that
        // from a market with nothing to say.
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
        // Both fields are public and neither is validated, and a zero in either
        // one is a reconnect loop that spins: doubling from zero never leaves
        // zero, and a ceiling of zero flattens every delay to nothing.
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

        // And doubling still escapes the floor rather than sticking to it.
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

        // The buffer is full; a second send must block rather than drop.
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

        // Held: dropping the session would stop the reconnects.
        let _session = connect(
            WsConnect {
                url: format!("ws://{address}"),
                // A stand-in for a token that is only good for a while. What
                // opened the first handshake must not be what opens the next
                // one, or an exchange that checks the clock inside it refuses
                // every reconnect and the loop retries a dead token forever.
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
        // Both frame kinds, because the wait for the consumer is where a
        // heartbeat is easiest to lose and neither exchange forgives losing it.
        for frame in [
            HeartbeatFrame::Text(r#"{"method":"ping"}"#),
            HeartbeatFrame::Ping,
        ] {
            let (address, heard) = chatty_server(Duration::from_millis(5)).await;
            let config = StreamConfig {
                // One event deep, so the buffer is full from the first frame on
                // and every delivery after it waits on the consumer.
                buffer_size: 1,
                // The default, spelled out: this is the ordinary path.
                overflow: Overflow::Backpressure,
                // Far longer than this test runs. What the idle timer counts
                // is the neighbouring test's business, and a window narrow
                // enough to be interesting there would be tripped here by a
                // busy machine rather than by anything the code did.
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
                        // This server drops nothing for being quiet, so the
                        // caller's own idle window is the one under test.
                        min_idle_timeout: Duration::ZERO,
                    }),
                },
                &config,
            )
            .await
            .expect("the first connection");

            assert!(matches!(session.next().await, Some(Ok(WsCommand::Text(_)))));

            // Six stalls. The buffer is one deep and full, so the connection
            // spends every one of them waiting on this consumer, and it is
            // still the exchange's keepalive that it owes: a real exchange
            // closes a connection it has heard nothing from, which is the one
            // thing the heartbeat exists to prevent, and the defect this
            // catches let the wait for the consumer stop it.
            //
            // How fast they arrive is not the assertion. A machine under load
            // sends them late, so counting what landed inside a fixed window
            // would measure that machine, while the defect sent none at all
            // however long it was given. So the wait is for three heartbeats
            // to arrive, with a deadline loose enough to mean nothing but
            // "never".
            for _ in 0..6 {
                let before = heard.load(Ordering::Relaxed);
                // Nothing is drained until this loop ends, so every heartbeat
                // counted here went out while the consumer was stalled.
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

                // And the stall cost nothing: `Overflow::Backpressure` loses
                // nothing, and a teardown instead reaches a server that has
                // stopped listening, so reconnect failures would arrive here
                // in place of the data.
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
            // One event deep, so the buffer is full from the first frame on and
            // the connection is waiting on this consumer for the whole stall.
            buffer_size: 1,
            overflow: Overflow::Backpressure,
            // Six of these fit in the stall below, which is what makes the
            // stall long enough to mean something, and each one is wide enough
            // that a machine under load cannot close it by descheduling the
            // connection for a moment. Both matter: a window this test could
            // trip by being slow would measure the scheduler instead of the
            // idle timer.
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
                // None, so the caller's window is the whole of the idle timer
                // and nothing this connection writes can be mistaken for the
                // inbound traffic that re-arms it.
                heartbeat: None,
            },
            &config,
        )
        .await
        .expect("the first connection");

        assert!(matches!(session.next().await, Some(Ok(WsCommand::Text(_)))));

        // Six idle windows during which the server has not been quiet for a
        // moment. The only silence is this consumer's, and the idle timer
        // measures the exchange.
        tokio::time::sleep(Duration::from_millis(3_000)).await;

        // Three frames, against a buffer that can bank one: the rest can only
        // come off a socket that is still open. A teardown reaches a server
        // that accepted once and stopped listening, so it arrives here as
        // reconnect failures rather than as data.
        for _ in 0..3 {
            let item = tokio::time::timeout(Duration::from_secs(5), session.next())
                .await
                .expect("the connection to still be delivering");
            assert!(matches!(item, Some(Ok(WsCommand::Text(_)))), "{item:?}");
        }
    }

    #[tokio::test]
    async fn a_connection_the_exchange_never_speaks_on_backs_off_and_says_so() {
        // Held: the server stops accepting once nothing is listening for its
        // connections, and a server that stopped accepting would make this pass
        // on failed reconnects instead of on the flapping under test.
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

        // Every reconnect succeeds and every socket it opens is mute, so none
        // of this is a failed attempt in the handshake sense: what the consumer
        // must not be left with is an endless run of `Reconnected` and no word
        // that the connection is not working.
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
        // A permanently broken subscription: a bad symbol, a retired stream
        // name, a revoked credential, an HTML error from a gateway. Every one
        // of them is a text frame followed by a close, byte for byte what a
        // working venue recycling a socket looks like from here, because
        // nothing at this layer parses either. A budget that any frame reset
        // would leave this reconnecting for as long as the process lives.
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

        // Drained to the end. `None` is the only thing that means over, and a
        // deadline this loose measures nothing but "never": the whole run is
        // four handshakes and three one-millisecond sleeps.
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
        // The first connection and one per attempt in the budget, and nothing
        // after the budget ran out. Exact rather than an upper bound, so a
        // future off-by-one in either direction is visible here.
        assert_eq!(opened, 4, "connections opened against a budget of 3");
    }

    #[tokio::test]
    async fn a_venue_that_recycles_sockets_keeps_reconnecting_at_the_first_delay() {
        // The other half of the same venue: it talks on every connection and
        // recycles it, and the caller left `max_reconnect_attempts` at `None`,
        // so this must go on forever at the pace a working connection earns.
        // Letting the backoff creep here would turn a venue that recycles every
        // few seconds into one reconnected to every thirty.
        let (address, _connections) = flapping_server(Some("hello")).await;
        let config = StreamConfig {
            initial_reconnect_delay_ms: 1,
            // Reached on the sixteenth consecutive mute reconnect from a
            // one-millisecond start, so a backoff that failed to reset could
            // not deliver twenty reconnects inside any deadline this test would
            // wait: the sleeps alone would come to over half a minute.
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

        // Waited for rather than timed. What is asserted is that twenty
        // reconnects arrive at all, which on a machine of any speed they do in
        // well under a second of sleeping.
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
            // Every field, because giving up needs an attempt limit and the
            // shared fixture leaves it `None`.
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

        // The documented promise: `Some(n)` ends the stream. Drained to the end
        // rather than counted, because what the consumer is told on the way out
        // is the neighbouring tests' business and `None` is the only thing that
        // means over.
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
            // One event deep, so the greeting fills it and the reconnect that
            // follows finds no room for its notice.
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
                    // Text, so the server sees it as an ordinary frame rather
                    // than something its protocol stack answers on its own.
                    frame: HeartbeatFrame::Text("beat"),
                    min_idle_timeout: Duration::ZERO,
                }),
            },
            &config,
        )
        .await
        .expect("the first connection");

        // Nothing is read until the reconnected socket has sent a heartbeat,
        // which the connection only does from the loop it enters once the
        // notice has been offered to this consumer and refused for want of
        // room. Waited for rather than slept through: a loaded machine gets
        // there later, and a fixed wait would be the difference between testing
        // the code and testing the machine.
        tokio::time::timeout(Duration::from_secs(10), heard.recv())
            .await
            .expect("the reconnected socket to reach its heartbeat")
            .expect("the server still running");

        assert!(matches!(
            session.next().await,
            Some(Ok(WsCommand::Text(text))) if text == "hello"
        ));

        // `DropNewest` discards data, and a consumer that asked for that knows
        // it. What it must not discard is the one event that says the data it
        // did keep is from the far side of a gap: a book rebuilt without this
        // notice stays wrong for as long as the connection lasts, and nothing
        // later restates it. So the next thing off this stream is the notice,
        // ahead of the ticks that have been arriving since the reconnect.
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

        // Held and never read, so the greeting fills the buffer and it stays
        // full for every reconnect after it.
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

        // The first connection and three reconnects, each of which had to
        // report `Reconnected` into a buffer that has been full since the first
        // frame. Waiting there is the one thing `DropNewest` exists to rule out.
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
            // Every field of `StreamConfig`, so unlike its neighbours this one
            // inherits nothing from `config()`: giving up needs an attempt
            // limit, which nothing else here sets.
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

        // Nothing is read while the connection gives up, so the buffer the
        // greeting filled is still full when the failure is reported. Waited
        // for rather than allowed a fixed window: a machine under load gives up
        // later, and reading the greeting early would free the room that makes
        // the report a drop rather than a wait.
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

        // The failure was discarded rather than waited on, which is what
        // `DropNewest` asks for. The stream ending is what is left to say the
        // connection is over.
        let ended = tokio::time::timeout(Duration::from_secs(5), session.next())
            .await
            .expect("the stream to end rather than wait on the consumer");
        assert!(ended.is_none(), "{ended:?}");
    }

    /// A socket that is open to a live server but refuses every write.
    ///
    /// Closing it from this side puts it in a state where `send` fails without
    /// waiting on the network, which is a dead socket reproduced exactly and
    /// with no timing in it.
    async fn write_dead_socket() -> Socket {
        let (address, _received) = one_shot_server(true).await;
        let mut socket = open(&WsConnect {
            url: format!("ws://{address}"),
            headers: None,
            subscribe: WsConnect::fixed(Vec::new()),
            heartbeat: None,
        })
        .await
        .expect("a connection to the local server");

        let _ = socket.close(None).await;
        assert!(
            socket.send(Message::Ping(Vec::new().into())).await.is_err(),
            "a closed socket should refuse writes"
        );

        socket
    }

    #[tokio::test]
    async fn a_heartbeat_that_cannot_be_written_does_not_cost_the_event_in_hand() {
        let mut socket = write_dead_socket().await;
        // One deep and already full, so the reserve inside `hand_over` is
        // pending and the heartbeat arm is the only one that can fire.
        let (sender, mut receiver) = mpsc::channel(1);
        sender
            .send(Ok(WsCommand::Text("already queued".into())))
            .await
            .expect("room in an empty buffer");

        // Due immediately, so the failing write happens before anything else.
        let mut pulse = Some((
            HeartbeatFrame::Ping,
            tokio::time::interval(Duration::from_millis(1)),
        ));

        let handed = tokio::spawn(async move {
            hand_over(
                &mut socket,
                &sender,
                Ok(WsCommand::Text("read before the socket died".into())),
                Overflow::Backpressure,
                &mut pulse,
            )
            .await
        });

        // The heartbeat above is due on the first poll of that task and the
        // reserve cannot complete until the buffer is drained below, so this
        // waits for the task to be scheduled at all rather than for a rate. It
        // is generous because a machine under load schedules it later, not
        // because anything here takes this long.
        tokio::time::sleep(Duration::from_millis(200)).await;

        // `Overflow::Backpressure` loses nothing, and a socket dying under the
        // wait is not an exception to that: the event was already off the wire,
        // and the reconnect that follows cannot fetch it again.
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

        // Still a dead socket, so the caller still reconnects.
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

        let handed = tokio::spawn(async move {
            hand_over(
                &mut socket,
                &sender,
                Ok(WsCommand::Text("nobody left to read it".into())),
                Overflow::Backpressure,
                &mut pulse,
            )
            .await
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
        // Waiting for room that will never come is the failure this rules out:
        // there is nothing left to deliver to, and reconnecting for a consumer
        // that is gone is work for no one.
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
