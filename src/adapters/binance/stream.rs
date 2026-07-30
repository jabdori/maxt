//! Binance's WebSocket streams, public and private.
//!
//! Both venues are addressed through their *combined* stream endpoint, where
//! every frame arrives wrapped as `{"stream": "btcusdt@trade", "data": {…}}`.
//! The plain endpoint is one byte shorter to type and unusable here: a partial
//! depth frame on spot carries no symbol of its own, so without the wrapper
//! there is no way to say which book changed.
//!
//! USD-M futures splits those streams across two entry points on one host, and
//! a subscription that spans both is opened as two sockets and merged. See
//! [`entry_point_url`].

use std::time::Duration;

use futures_util::StreamExt;
use futures_util::stream::select_all;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::error::{Error, Result};
use crate::stream::{AccountStream, MarketStream};
use crate::types::{
    AccountEvent, Balance, Candle, Feed, Interval, Market, MarketEvent, Order, StreamConfig,
    Subscription, Timestamp,
};

use super::{
    BinanceAdapter, BinanceMarket, EXCHANGE, SPOT_WEBSOCKET_API_URL, SPOT_WEBSOCKET_URL,
    USD_M_MARKET_WEBSOCKET_URL, USD_M_PUBLIC_WEBSOCKET_URL, parse, private,
};
use crate::transport::{Heartbeat, HeartbeatFrame, WsCommand, WsConnect, connect};

/// The book depth and refresh rate a subscription asks for.
///
/// Binance offers 5, 10, or 20 levels at 100ms or 1000ms. Twenty levels at
/// 100ms is the deepest and freshest of them, and the closest thing to the
/// snapshot [`MarketEvent::OrderBook`] describes.
const DEPTH_STREAM: &str = "depth20@100ms";

/// How long Binance keeps a listen key alive without being asked again.
const LISTEN_KEY_LIFETIME: Duration = Duration::from_secs(60 * 60);

/// What holds a Binance socket open when nothing is trading.
///
/// Binance is the one exchange here with no client-side text ping: its sockets
/// read every text frame as a command and answer an unknown one with an error,
/// so the keepalive has to be a protocol ping, which Binance's stack pongs
/// without the API ever seeing it.
///
/// The floor is what matters. Binance drives the liveness itself, pinging the
/// client every three minutes and hanging up only after ten minutes with no
/// pong back. On an account that never moves, three minutes of silence is a
/// perfectly healthy socket. Below that floor the idle timer would tear down
/// and rebuild a working connection on a fixed cycle forever.
const HEARTBEAT: Heartbeat = Heartbeat {
    interval: Duration::from_secs(60),
    frame: HeartbeatFrame::Ping,
    // Longer than the three minutes between Binance's own pings, so a quiet
    // account is not mistaken for a dead socket.
    min_idle_timeout: Duration::from_secs(240),
};

/// Binance's own name for a feed on one venue.
///
/// Both venues publish every fill on `@trade`, under the same trade id
/// `Client::trades` returns over REST. USD-M also offers `@aggTrade`, which
/// collapses the fills that matched one taker order at one price into a single
/// message carrying a second, unrelated id; `maxt` does not subscribe to it,
/// because that second id space is what forces a caller to reconcile the two
/// transports on something other than an id.
fn feed_name(venue: BinanceMarket, feed: Feed) -> Result<String> {
    Ok(match feed {
        Feed::Trades => "trade".to_string(),
        Feed::OrderBook => DEPTH_STREAM.to_string(),
        Feed::Ticker => "ticker".to_string(),
        Feed::Candles(interval) => format!("kline_{}", venue.interval_code(interval)?),
    })
}

/// Reads back the interval a `kline_*` stream carries.
fn interval_from_code(code: &str) -> Option<Interval> {
    Some(match code {
        "1s" => Interval::Sec1,
        "1m" => Interval::Min1,
        "3m" => Interval::Min3,
        "5m" => Interval::Min5,
        "15m" => Interval::Min15,
        "30m" => Interval::Min30,
        "1h" => Interval::Hour1,
        "2h" => Interval::Hour2,
        "4h" => Interval::Hour4,
        "8h" => Interval::Hour8,
        "12h" => Interval::Hour12,
        "1d" => Interval::Day1,
        "3d" => Interval::Day3,
        "1w" => Interval::Week1,
        // Capitalised, and a different span from `1m`.
        "1M" => Interval::Month1,
        _ => return None,
    })
}

/// The endpoint that carries one feed on one venue.
///
/// Binance serves USD-M market data from two entry points on
/// `fstream.binance.com`, and decommissioned the unrouted `/stream` and `/ws`
/// paths on 2026-04-23. `/public` carries what the matching engine pushes on
/// change: `@trade`, `@depth*`, `@bookTicker`. `/market` carries what an
/// aggregator produces: `@kline_*`, `@ticker`, `@aggTrade`, `@markPrice`,
/// `@forceOrder`. A connection naming neither is served as if it had named
/// `/public`.
///
/// Nothing rejects a mismatch. A socket on one entry point accepts a
/// `SUBSCRIBE` for the other's streams, acknowledges it with
/// `{"result": null, "id": 1}`, and then never sends a frame for it, which is
/// why the endpoint has to be chosen per feed rather than per venue.
///
/// Measured 2026-07-30 over 25 s on BTCUSDT, one `SUBSCRIBE` naming all seven
/// streams on each endpoint, counting frames by `stream` name:
///
/// | Endpoint | `@trade` | `@depth20@100ms` | `@bookTicker` | `@aggTrade` | `@kline_1m` | `@ticker` | `@markPrice@1s` |
/// | --- | --- | --- | --- | --- | --- | --- | --- |
/// | `/stream` | 141 | 229 | 896 | 0 | 0 | 0 | 0 |
/// | `/public/stream` | 149 | 229 | 1993 | 0 | 0 | 0 | 0 |
/// | `/market/stream` | 0 | 0 | 0 | 117 | 47 | 12 | 25 |
///
/// Spot is not split. Both entry points name the one spot endpoint, so a spot
/// subscription always groups into a single socket.
fn entry_point_url(venue: BinanceMarket, feed: Feed) -> &'static str {
    match (venue, feed) {
        (BinanceMarket::Spot, _) => SPOT_WEBSOCKET_URL,
        (BinanceMarket::UsdMFutures, Feed::Trades | Feed::OrderBook) => USD_M_PUBLIC_WEBSOCKET_URL,
        (BinanceMarket::UsdMFutures, Feed::Ticker | Feed::Candles(_)) => USD_M_MARKET_WEBSOCKET_URL,
    }
}

/// The stream names one subscription covers, grouped by the endpoint that
/// carries them.
///
/// One group is one socket. Spot yields one; USD-M yields one or two,
/// depending on whether the subscription spans both entry points. Groups come
/// back in the order their endpoint was first named, and names within a group
/// keep the subscription's market-then-feed order.
pub(super) fn stream_groups(
    adapter: &BinanceAdapter,
    subscription: &Subscription,
) -> Result<Vec<(&'static str, Vec<String>)>> {
    if subscription.markets().is_empty() {
        return Err(Error::invalid_request(
            "markets",
            "subscribe to at least one market",
        ));
    }
    if subscription.feeds().is_empty() {
        return Err(Error::invalid_request(
            "feeds",
            "subscribe to at least one feed",
        ));
    }

    let mut groups: Vec<(&'static str, Vec<String>)> = Vec::new();
    for market in subscription.markets() {
        // Binance names its streams in lowercase and rejects anything else.
        let symbol = adapter.symbol(market)?.to_ascii_lowercase();
        for feed in subscription.feeds() {
            let name = format!("{symbol}@{}", feed_name(adapter.venue(), *feed)?);
            let url = entry_point_url(adapter.venue(), *feed);
            match groups.iter_mut().find(|(known, _)| *known == url) {
                Some((_, names)) => names.push(name),
                None => groups.push((url, vec![name])),
            }
        }
    }

    Ok(groups)
}

/// The frame each socket sends on connect and again after every reconnect,
/// paired with the endpoint it is sent to.
pub(super) fn subscribe_frames(
    adapter: &BinanceAdapter,
    subscription: &Subscription,
) -> Result<Vec<(&'static str, String)>> {
    stream_groups(adapter, subscription)?
        .into_iter()
        .map(|(url, names)| {
            let payload = serde_json::json!({
                "method": "SUBSCRIBE",
                "params": names,
                // Binance echoes this back on the acknowledgement. One
                // connection sends one subscribe frame, so it never needs to
                // be more than a constant.
                "id": 1,
            });
            let frame = serde_json::to_string(&payload).map_err(|err| {
                Error::decode(format!("could not build binance subscribe frame: {err}"))
            })?;
            Ok((url, frame))
        })
        .collect()
}

/// A `kline` frame's payload.
#[derive(Debug, Deserialize)]
struct RawStreamCandle {
    k: RawKline,
}

#[derive(Debug, Deserialize)]
struct RawKline {
    /// Open time.
    t: i64,
    /// The interval, in Binance's own spelling.
    i: String,
    o: String,
    h: String,
    l: String,
    c: String,
    v: String,
    /// Quote asset volume.
    q: String,
    /// Whether the interval has closed.
    x: bool,
}

/// Reads one public frame.
///
/// `None` means the frame carried no market data. Binance acknowledges a
/// subscribe with `{"result": null, "id": 1}`, and USD-M announces a trade id
/// it published no fill for as a `@trade` frame priced and sized at zero.
/// Neither is something a caller should see.
pub(super) fn decode(adapter: &BinanceAdapter, frame: &str) -> Result<Option<MarketEvent>> {
    let value: Value = serde_json::from_str(frame)
        .map_err(|err| Error::decode(format!("unreadable binance frame: {err}")))?;

    if let Some(error) = value.get("error") {
        return Err(frame_error(error));
    }
    // The acknowledgement, and the answer to a LIST_SUBSCRIPTIONS.
    if value.get("result").is_some() {
        return Ok(None);
    }

    let Some(name) = value.get("stream").and_then(Value::as_str) else {
        return Err(Error::decode(
            "binance frame carries no `stream` to place it by".to_string(),
        ));
    };
    let Some((symbol, feed)) = name.split_once('@') else {
        return Err(Error::decode(format!(
            "`{name}` is not a binance stream name: expected symbol@feed"
        )));
    };
    let market = adapter.market(&symbol.to_ascii_uppercase())?;

    let Some(data) = value.get("data") else {
        return Err(Error::decode(format!(
            "binance frame for `{name}` carries no `data`"
        )));
    };
    let body = serde_json::to_string(data)
        .map_err(|err| Error::decode(format!("could not re-read binance frame: {err}")))?;

    Ok(Some(match feed {
        "trade" => {
            let trade = parse::trade(&market, &parse::json(&body, "trade frame")?)?;
            // USD-M spends trade ids it never publishes a fill for, and
            // announces each one on `@trade` as a frame whose price and
            // quantity are both `"0"` and whose `X` reads `NA`. Those ids are
            // absent from `/fapi/v1/trades`, so forwarding the frame would
            // invent a trade at a price of zero that the exchange itself does
            // not list. Read off the quantity rather than off `X`, because
            // spot publishes no `X` at all and a trade of nothing is not a
            // trade on either venue.
            if trade.quantity.is_zero() {
                return Ok(None);
            }
            MarketEvent::Trade(trade)
        }
        "ticker" => MarketEvent::Ticker(parse::ticker(
            &market,
            &parse::json(&body, "ticker frame")?,
        )?),
        depth if depth.starts_with("depth") => MarketEvent::OrderBook(parse::order_book(
            &market,
            Timestamp::now(),
            &parse::json(&body, "depth frame")?,
        )?),
        kline if kline.starts_with("kline_") => {
            MarketEvent::Candle(stream_candle(&market, &parse::json(&body, "kline frame")?)?)
        }
        other => {
            return Err(Error::decode(format!(
                "binance sent an unmapped feed `{other}`"
            )));
        }
    }))
}

fn stream_candle(market: &Market, raw: &RawStreamCandle) -> Result<Candle> {
    let interval = interval_from_code(&raw.k.i).ok_or_else(|| {
        Error::decode(format!(
            "binance sent an unmapped candle interval `{}`",
            raw.k.i
        ))
    })?;

    Ok(Candle {
        market: market.clone(),
        interval,
        open_time: parse::millis(raw.k.t),
        open: parse::decimal(&raw.k.o, "o")?,
        high: parse::decimal(&raw.k.h, "h")?,
        low: parse::decimal(&raw.k.l, "l")?,
        close: parse::decimal(&raw.k.c, "c")?,
        volume: parse::decimal(&raw.k.v, "v")?,
        quote_volume: Some(parse::decimal(&raw.k.q, "q")?),
        // Unlike the REST candles, a streamed one says so itself.
        closed: raw.k.x,
    })
}

/// Reads Binance's WebSocket error frame, which carries a code and a message
/// like the REST envelope but arrives without an HTTP status.
fn frame_error(error: &Value) -> Error {
    let code = error
        .get("code")
        .map(ToString::to_string)
        .unwrap_or_else(|| "unknown".to_string());
    let message = error
        .get("msg")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("binance closed the subscription without saying why");

    Error::exchange(EXCHANGE, code, message)
}

/// Opens one socket per endpoint the subscription reaches and merges them.
///
/// A USD-M subscription that names both a book feed and an aggregated one
/// spans two entry points, and one socket cannot serve both. Each socket
/// reconnects on its own, so such a subscription reports
/// [`MarketEvent::Reconnected`] once per socket that comes back rather than
/// once per outage. Dropping the stream closes every socket under it.
pub(super) async fn subscribe(
    adapter: &BinanceAdapter,
    subscription: &Subscription,
    config: &StreamConfig,
) -> Result<MarketStream> {
    let mut sessions = Vec::new();
    for (url, frame) in subscribe_frames(adapter, subscription)? {
        sessions.push(
            connect(
                WsConnect {
                    url: url.to_string(),
                    headers: None,
                    subscribe: WsConnect::fixed(vec![frame]),
                    heartbeat: Some(HEARTBEAT),
                },
                config,
            )
            .await?,
        );
    }

    let adapter = adapter.clone();
    Ok(MarketStream::new(select_all(sessions).filter_map(
        move |item| {
            let event = match item {
                Ok(WsCommand::Text(frame)) => decode(&adapter, &frame).transpose(),
                // Binance's public streams are text only; a binary frame means
                // something changed on their side that `maxt` has not read.
                Ok(WsCommand::Binary(_)) => Some(Err(Error::decode(
                    "binance sent an unexpected binary frame",
                ))),
                Ok(WsCommand::Reconnected) => Some(Ok(MarketEvent::Reconnected)),
                Err(error) => Some(Err(error)),
            };
            std::future::ready(event)
        },
    )))
}

// ---------------------------------------------------------------------------
// Account stream
// ---------------------------------------------------------------------------

/// One event off a user data stream, in either venue's spelling.
///
/// Only the events that change a balance or an order are named. Binance also
/// publishes list statuses, external lock updates, and margin calls, none of
/// which [`AccountEvent`] carries.
#[derive(Debug, Deserialize)]
#[serde(tag = "e")]
enum RawAccountEvent {
    /// Spot: the account's balances after a change.
    #[serde(rename = "outboundAccountPosition")]
    SpotBalances {
        #[serde(rename = "B")]
        balances: Vec<RawSpotStreamBalance>,
    },
    /// Spot: an order was placed, filled, cancelled, or rejected.
    #[serde(rename = "executionReport")]
    SpotOrder {
        #[serde(rename = "s")]
        symbol: String,
        #[serde(rename = "S")]
        side: String,
        #[serde(rename = "X")]
        status: String,
        #[serde(rename = "i")]
        order_id: serde_json::Number,
        #[serde(rename = "q")]
        quantity: String,
        #[serde(rename = "z")]
        filled: String,
        #[serde(rename = "p")]
        price: String,
        /// When the order was created, which is what
        /// [`Order::created_at`] means.
        ///
        /// Spot publishes this alongside `T`, the transaction time of the
        /// event in hand. They are equal on the report that opens an order
        /// and diverge from the first fill on, so reading `T` would date a
        /// resting order to whenever it last traded and disagree with the
        /// same order read back over REST.
        ///
        /// Optional only because USD-M shares this reading and publishes no
        /// `O` at all. Every spot `executionReport` carries one.
        #[serde(rename = "O", default)]
        created_at: Option<i64>,
    },
    /// USD-M: balances and positions after a change.
    #[serde(rename = "ACCOUNT_UPDATE")]
    FuturesAccount {
        #[serde(rename = "a")]
        account: RawFuturesAccountUpdate,
    },
    /// USD-M: an order changed.
    #[serde(rename = "ORDER_TRADE_UPDATE")]
    FuturesOrder {
        #[serde(rename = "o")]
        order: RawFuturesOrderUpdate,
    },
    /// USD-M: the listen key this socket was opened with has lapsed.
    #[serde(rename = "listenKeyExpired")]
    ListenKeyExpired,
    /// Spot: this socket's subscription is over.
    ///
    /// Binance sends it when the subscription is unsubscribed, when the session
    /// logs out, and when the subscription expires. It is the only frame a spot
    /// user data socket is guaranteed to produce on an account that never
    /// trades, which is what makes it the proof that one is really connected.
    #[serde(rename = "eventStreamTerminated")]
    EventStreamTerminated,
    /// Anything else Binance publishes on the same socket.
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct RawSpotStreamBalance {
    #[serde(rename = "a")]
    asset: String,
    #[serde(rename = "f")]
    free: String,
    #[serde(rename = "l")]
    locked: String,
}

#[derive(Debug, Deserialize)]
struct RawFuturesAccountUpdate {
    #[serde(rename = "B", default)]
    balances: Vec<RawFuturesStreamBalance>,
}

#[derive(Debug, Deserialize)]
struct RawFuturesStreamBalance {
    #[serde(rename = "a")]
    asset: String,
    #[serde(rename = "wb")]
    wallet_balance: String,
    #[serde(rename = "cw")]
    cross_wallet_balance: String,
}

#[derive(Debug, Deserialize)]
struct RawFuturesOrderUpdate {
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "S")]
    side: String,
    #[serde(rename = "X")]
    status: String,
    #[serde(rename = "i")]
    order_id: serde_json::Number,
    #[serde(rename = "q")]
    quantity: String,
    #[serde(rename = "z")]
    filled: String,
    #[serde(rename = "p")]
    price: String,
    /// When the order was created, which USD-M does not publish and this
    /// therefore never carries.
    ///
    /// The only clock in an `ORDER_TRADE_UPDATE` is `T`, the transaction time of
    /// the event in hand, and on an order that rests and later fills that is the
    /// fill. Reporting it as a creation time gave one order two different ages
    /// depending on whether it was read here or over REST, drifting further
    /// apart with every amendment. So this stays `None`, which
    /// [`Order::created_at`] already carries the meaning of: the exchange did
    /// not say. A USD-M order's age comes from the REST read, which does.
    ///
    /// Named after the field spot publishes rather than dropped, so that a
    /// venue which starts sending one is read without a change here.
    #[serde(rename = "O", default)]
    created_at: Option<i64>,
}

/// One frame off a spot WebSocket API socket.
///
/// The socket carries two kinds of frame and every field here is optional
/// because no frame has them all. An answer to a request carries `status` and
/// either `result` or `error`; a pushed account event carries `subscriptionId`
/// and `event`. `maxt` reads the answer only to find out whether the
/// subscription took, and reads nothing from `result` beyond that.
///
/// Captured 2026-07-31 off `wss://ws-api.binance.com:443/ws-api/v3`:
///
/// ```text
/// {"id":"1785442705539","status":200,"result":{"subscriptionId":0},"rateLimits":[…]}
/// {"subscriptionId":0,"event":{"e":"eventStreamTerminated","E":1785442715366}}
/// {"id":"st1","status":400,"error":{"code":-1021,"msg":"Timestamp for this request is outside of the recvWindow."}}
/// ```
#[derive(Debug, Deserialize)]
struct RawWsApiFrame {
    /// Present on an answer, absent on a pushed event.
    status: Option<u16>,
    error: Option<RawWsApiError>,
    /// Present on a pushed event, absent on an answer.
    event: Option<RawAccountEvent>,
}

#[derive(Debug, Deserialize)]
struct RawWsApiError {
    code: i64,
    msg: String,
}

/// Reads one private frame into the events it carries.
///
/// One frame can describe several balances, so this returns a list. An empty
/// list means the frame was about something `maxt` does not model.
///
/// The two venues are framed differently and neither wrapper is optional. Spot
/// speaks the WebSocket API, which wraps every event and answers the subscribe
/// request on the same socket; USD-M pushes bare events down a socket that was
/// authenticated by its URL.
pub(super) fn decode_account(adapter: &BinanceAdapter, frame: &str) -> Result<Vec<AccountEvent>> {
    match adapter.venue() {
        BinanceMarket::Spot => decode_spot_account(adapter, frame),
        BinanceMarket::UsdMFutures => {
            decode_account_event(adapter, parse::json(frame, "user data frame")?)
        }
    }
}

/// Reads one frame off a spot WebSocket API socket.
///
/// A refused subscription is an error rather than silence. Binance answers a
/// rejected `userDataStream.subscribe.signature` with a frame and then leaves
/// the socket open and empty, and an account that never trades is empty on a
/// working socket too, so a consumer told nothing could not tell them apart.
///
/// The refusal keeps Binance's own code rather than becoming
/// [`Error::Auth`](crate::Error::Auth): the frame's `status` and `code` are
/// what say whether the secret is wrong (`-1022`), the key type is
/// (`-2028`), or only the clock is (`-1021`), and the three want different
/// things done about them.
fn decode_spot_account(adapter: &BinanceAdapter, frame: &str) -> Result<Vec<AccountEvent>> {
    let raw: RawWsApiFrame = parse::json(frame, "user data frame")?;

    if let Some(status) = raw.status.filter(|status| *status != 200) {
        let (code, message) = raw.error.map_or_else(
            || ("unknown".to_string(), "no reason given".to_string()),
            |error| (error.code.to_string(), error.msg),
        );
        return Err(Error::exchange_http(EXCHANGE, status, code, message));
    }

    match raw.event {
        Some(event) => decode_account_event(adapter, event),
        None => Ok(Vec::new()),
    }
}

/// Turns one decoded user data event into what `maxt` models.
fn decode_account_event(
    adapter: &BinanceAdapter,
    raw: RawAccountEvent,
) -> Result<Vec<AccountEvent>> {
    Ok(match raw {
        RawAccountEvent::SpotBalances { balances } => balances
            .iter()
            .map(|raw| {
                Ok(AccountEvent::Balance(Balance {
                    asset: raw.asset.to_ascii_uppercase(),
                    available: parse::decimal(&raw.free, "f")?,
                    locked: parse::decimal(&raw.locked, "l")?,
                }))
            })
            .collect::<Result<Vec<_>>>()?,
        RawAccountEvent::FuturesAccount { account } => account
            .balances
            .iter()
            .map(|raw| {
                let wallet = parse::decimal(&raw.wallet_balance, "wb")?;
                let cross = parse::decimal(&raw.cross_wallet_balance, "cw")?;
                Ok(AccountEvent::Balance(Balance {
                    asset: raw.asset.to_ascii_uppercase(),
                    // This event carries only `wb` and `cw`, never the free
                    // balance the REST account read reports. `cw` is the wallet
                    // less whatever is ring-fenced by isolated positions, so it
                    // still counts margin held by open *cross* positions and is
                    // therefore an upper bound on what is free to trade: the
                    // same account read over REST will report less. Re-read
                    // there when the exact figure matters; Binance publishes
                    // nothing here that would narrow it.
                    available: cross,
                    locked: (wallet - cross).max(rust_decimal::Decimal::ZERO),
                }))
            })
            .collect::<Result<Vec<_>>>()?,
        RawAccountEvent::SpotOrder {
            symbol,
            side,
            status,
            order_id,
            quantity,
            filled,
            price,
            created_at,
        }
        | RawAccountEvent::FuturesOrder {
            order:
                RawFuturesOrderUpdate {
                    symbol,
                    side,
                    status,
                    order_id,
                    quantity,
                    filled,
                    price,
                    created_at,
                },
        } => {
            let filled = parse::decimal(&filled, "z")?;
            let total = parse::decimal(&quantity, "q")?;
            let status = parse::status(&status);

            vec![AccountEvent::Order(Order {
                id: order_id.to_string(),
                market: adapter.market(&symbol)?,
                side: parse::side(&side)?,
                status,
                filled_quantity: filled,
                remaining_quantity: if status.is_live() {
                    (total - filled).max(rust_decimal::Decimal::ZERO)
                } else {
                    rust_decimal::Decimal::ZERO
                },
                price: parse::decimal_or_none(&price, "p")?,
                created_at: created_at.map(parse::millis),
            })]
        }
        // Both are Binance ending the subscription, pushed as an event rather
        // than answered as an error, and neither carries a message of its own.
        // The event name is the code, because that is the whole of what Binance
        // said and a caller telling the two apart is telling apart a key that
        // lapsed from a stream Binance closed.
        RawAccountEvent::ListenKeyExpired => {
            return Err(Error::exchange(
                EXCHANGE,
                "listenKeyExpired",
                "the listen key behind this stream expired; subscribe again",
            ));
        }
        RawAccountEvent::EventStreamTerminated => {
            return Err(Error::exchange(
                EXCHANGE,
                "eventStreamTerminated",
                "binance ended the spot user data subscription behind this stream; subscribe again",
            ));
        }
        RawAccountEvent::Other => Vec::new(),
    })
}

pub(super) async fn subscribe_account(
    adapter: &BinanceAdapter,
    config: &StreamConfig,
) -> Result<AccountStream> {
    match adapter.venue() {
        BinanceMarket::Spot => subscribe_spot_account(adapter, config).await,
        BinanceMarket::UsdMFutures => subscribe_usd_m_account(adapter, config).await,
    }
}

/// Opens a spot user data stream over the WebSocket API.
///
/// There is no listen key and so nothing to keep alive: Binance removed
/// `POST /api/v3/userDataStream` on 2026-02-20 07:00 UTC and the host has
/// answered it `410 Gone` ever since, measured 2026-07-31. The socket opens
/// unauthenticated and the first frame out names the account.
///
/// Verified 2026-07-31 against a live HMAC-SHA-256 key: the subscribe was
/// answered `{"status":200,"result":{"subscriptionId":0}}`, the socket stayed
/// open through 150 s of silence on an account with no balances and no open
/// orders, and a `userDataStream.unsubscribe` at the end of it was answered by
/// a pushed `eventStreamTerminated`, which is what proves the socket was still
/// carrying that account's events rather than merely still open.
async fn subscribe_spot_account(
    adapter: &BinanceAdapter,
    config: &StreamConfig,
) -> Result<AccountStream> {
    let session = connect(spot_account_connect(adapter)?, config).await?;

    Ok(AccountStream::new(
        account_events(adapter.clone(), session).boxed(),
    ))
}

/// How a spot account socket is opened, and how every one of them is
/// authenticated.
///
/// The credential is in the first frame out rather than in the URL or a header,
/// which is what the WebSocket API's `userDataStream.subscribe.signature` takes,
/// and that frame signs the millisecond clock it was built at. One signature
/// therefore subscribes one socket: a reconnect that re-sent the frame it first
/// subscribed with would be refused `-1021` once the outage outlasted
/// `recvWindow`, and the reconnect loop would replay that dead frame onto every
/// socket after it. So the frame is signed per handshake, and what a reconnect
/// presents is as fresh as what the first connection did.
fn spot_account_connect(adapter: &BinanceAdapter) -> Result<WsConnect> {
    // Cloned into the signing closure below, which outlives this call: it is
    // called again for every reconnect.
    let adapter = adapter.clone();

    Ok(WsConnect {
        url: SPOT_WEBSOCKET_API_URL.to_string(),
        headers: None,
        subscribe: Box::new(move || Ok(vec![private::spot_user_data_subscribe_frame(&adapter)?])),
        heartbeat: Some(HEARTBEAT),
    })
}

/// Opens a USD-M user data stream with a listen key.
async fn subscribe_usd_m_account(
    adapter: &BinanceAdapter,
    config: &StreamConfig,
) -> Result<AccountStream> {
    let key = private::create_listen_key(adapter).await?;
    let session = connect(
        WsConnect {
            // USD-M authenticates a user data stream by URL. Nothing is sent on
            // connect: the socket carries whatever the key's account does.
            url: private::usd_m_user_data_stream_url(&key),
            headers: None,
            subscribe: WsConnect::fixed(Vec::new()),
            heartbeat: Some(HEARTBEAT),
        },
        config,
    )
    .await?;

    // A listen key lapses an hour after it was last extended, which would end
    // the stream mid-session with nothing said. This channel is both halves of
    // the refresher's contract with the stream: it carries a failed extension
    // out to the consumer, and its closing is how dropping the stream stops the
    // refresher, at once, rather than at the end of whatever sleep the
    // refresher happened to be in.
    let (failures, failed) = mpsc::channel(1);
    tokio::spawn(refresh_listen_key(adapter.clone(), key, failures));

    let events = account_events(adapter.clone(), session).boxed();

    Ok(AccountStream::new(with_refresher_failures(events, failed)))
}

/// Reads a private socket's frames as account events.
fn account_events(
    adapter: BinanceAdapter,
    session: impl futures_core::Stream<Item = Result<WsCommand>> + Send + 'static,
) -> impl futures_core::Stream<Item = Result<AccountEvent>> + Send {
    session.flat_map(move |item| {
        let events = match item {
            Ok(WsCommand::Text(frame)) => match decode_account(&adapter, &frame) {
                Ok(events) => events.into_iter().map(Ok).collect(),
                Err(error) => vec![Err(error)],
            },
            Ok(WsCommand::Binary(_)) => vec![Err(Error::decode(
                "binance sent an unexpected binary frame",
            ))],
            Ok(WsCommand::Reconnected) => vec![Ok(AccountEvent::Reconnected)],
            Err(error) => vec![Err(error)],
        };
        futures_util::stream::iter(events)
    })
}

/// Merges the refresher's one possible failure into the socket's own events.
///
/// The socket alone decides when the stream is over. A refresher still sleeping
/// on a healthy key must not hold a subscription open past the connection
/// giving up, or the consumer waits forever on a stream that has nothing left
/// to say.
fn with_refresher_failures(
    events: futures_util::stream::BoxStream<'static, Result<AccountEvent>>,
    failed: mpsc::Receiver<Error>,
) -> impl futures_core::Stream<Item = Result<AccountEvent>> + Send {
    futures_util::stream::unfold((events, failed), |(mut events, mut failed)| async move {
        tokio::select! {
            // A failure already waiting is reported before the next frame: it
            // says the socket is on borrowed time, which is worth knowing early.
            biased;
            Some(error) = failed.recv() => Some((Err(error), (events, failed))),
            item = events.next() => item.map(|item| (item, (events, failed))),
        }
    })
}

/// Waits out one refresh interval, unless the stream goes first.
///
/// `false` once the consumer has dropped the stream and taken the receiving
/// half of `failures` with it, which is the whole reason to stop refreshing.
/// Waiting out the half-hour before noticing would leave a task alive, holding
/// a cloned adapter, for up to that long after the stream it serves is gone,
/// and a process that opens and drops account streams accumulates one per
/// stream.
async fn due(interval: Duration, failures: &mpsc::Sender<Error>) -> bool {
    tokio::select! {
        () = tokio::time::sleep(interval) => true,
        () = failures.closed() => false,
    }
}

/// Keeps the listen key behind an open user data stream from lapsing.
///
/// Runs until the stream is dropped or an extension fails. A failure is
/// reported to the consumer, because the socket stays up until the key actually
/// lapses. Binance then rejects every reconnect, and a consumer told nothing
/// would await a stream that is never going to speak again.
///
/// The failure is forwarded as it arrived rather than restated as an
/// [`Error::Auth`](crate::Error::Auth). Binance's own verdict is what says
/// whether the key is gone, the key was never this account's, or the network
/// was merely down for a moment, and a stream that flattened the three would
/// leave a caller unable to tell a lost subscription from a blip. What it costs
/// is stated on the provider page: an extension that keeps failing ends the
/// stream's usefulness within the hour.
async fn refresh_listen_key(
    adapter: BinanceAdapter,
    key: super::BinanceListenKey,
    failures: mpsc::Sender<Error>,
) {
    // Half the lifetime, so one failed refresh still leaves time for the next.
    let interval = LISTEN_KEY_LIFETIME / 2;

    loop {
        if !due(interval, &failures).await {
            return;
        }
        if let Err(error) = private::keepalive_listen_key(&adapter, &key).await {
            let _ = failures.send(error).await;
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::Feature;
    use crate::types::{Exchange, OrderStatus, Side};

    // https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams
    const SPOT_TRADE_FRAME: &str = r#"{
      "stream": "bnbbtc@trade",
      "data": {
        "e": "trade",
        "E": 1672515782136,
        "s": "BNBBTC",
        "t": 12345,
        "p": "0.001",
        "q": "100",
        "T": 1672515782136,
        "m": true,
        "M": true
      }
    }"#;

    // https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams
    const SPOT_KLINE_FRAME: &str = r#"{
      "stream": "bnbbtc@kline_1m",
      "data": {
        "e": "kline",
        "E": 1672515782136,
        "s": "BNBBTC",
        "k": {
          "t": 1672515780000,
          "T": 1672515839999,
          "s": "BNBBTC",
          "i": "1m",
          "f": 100,
          "L": 200,
          "o": "0.0010",
          "c": "0.0020",
          "h": "0.0025",
          "l": "0.0015",
          "v": "1000",
          "n": 100,
          "x": false,
          "q": "1.0000",
          "V": "500",
          "Q": "0.500",
          "B": "123456"
        }
      }
    }"#;

    // https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams
    const SPOT_DEPTH_FRAME: &str = r#"{
      "stream": "bnbbtc@depth20@100ms",
      "data": {
        "lastUpdateId": 160,
        "bids": [["0.0024", "10"], ["0.0023", "5"]],
        "asks": [["0.0026", "100"], ["0.0027", "50"]]
      }
    }"#;

    // Captured 2026-07-30 off wss://fstream.binance.com/stream, one SUBSCRIBE
    // frame for `dogeusdt@trade`. Trade 3417626319 is in
    // https://fapi.binance.com/fapi/v1/trades?symbol=DOGEUSDT with the same
    // price, quantity, time and `isBuyerMaker`.
    const FUTURES_TRADE_FRAME: &str = r#"{
      "stream": "dogeusdt@trade",
      "data": {
        "e": "trade",
        "E": 1785407770046,
        "T": 1785407770046,
        "s": "DOGEUSDT",
        "t": 3417626319,
        "p": "0.070180",
        "q": "700",
        "X": "MARKET",
        "m": true,
        "st": 1
      }
    }"#;

    // Captured 2026-07-30 off the same socket, `ethusdt@trade`. USD-M spends
    // trade ids it publishes no fill for and announces each one like this.
    // Trade 8520420588 falls inside the window
    // https://fapi.binance.com/fapi/v1/trades?symbol=ETHUSDT returned
    // (8520419586 to 8520420589) and is not in it.
    const FUTURES_SPENT_TRADE_ID_FRAME: &str = r#"{
      "stream": "ethusdt@trade",
      "data": {
        "e": "trade",
        "E": 1785407792240,
        "T": 1785407792239,
        "s": "ETHUSDT",
        "t": 8520420588,
        "p": "0",
        "q": "0",
        "X": "NA",
        "m": true,
        "st": 1
      }
    }"#;

    // Spot pushes every user data event down the WebSocket API socket inside
    // this envelope. The envelope is not optional: an event on its own is what
    // the removed listen key transport sent, and Binance's current page shows
    // every example wrapped.
    // https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream
    const SPOT_BALANCE_FRAME: &str = r#"{
      "subscriptionId": 0,
      "event": {
        "e": "outboundAccountPosition",
        "E": 1564034571105,
        "u": 1564034571073,
        "B": [
          { "a": "ETH", "f": "10000.000000", "l": "0.000000" }
        ]
      }
    }"#;

    // Captured verbatim on 2026-07-31 off `wss://ws-api.binance.com:443/ws-api/v3`,
    // by subscribing with `userDataStream.subscribe.signature` and then sending
    // `userDataStream.unsubscribe`. On an account with no balances and no open
    // orders this is the only frame the socket will produce, which is what makes
    // it the proof that one is connected rather than merely open.
    const SPOT_STREAM_TERMINATED: &str =
        r#"{"subscriptionId":0,"event":{"e":"eventStreamTerminated","E":1785442936161}}"#;

    // Captured verbatim on 2026-07-31 off the same socket: what Binance answers
    // a `userDataStream.subscribe.signature` that took.
    const SPOT_SUBSCRIBE_ACCEPTED: &str = r#"{"id":"1785442705539","status":200,
      "result":{"subscriptionId":0},
      "rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":6000,"count":4}]}"#;

    // Captured verbatim on 2026-07-31 off the same socket, by sending a frame
    // signed ninety seconds earlier: what a replayed subscribe is answered with.
    const SPOT_SUBSCRIBE_STALE: &str = r#"{"id":"st1","status":400,
      "error":{"code":-1021,"msg":"Timestamp for this request is outside of the recvWindow."},
      "rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":6000,"count":16}]}"#;

    // Captured verbatim on 2026-07-31 off the same socket, by sending
    // `session.logon` with this HMAC-SHA-256 key.
    const SPOT_LOGON_REFUSED: &str = r#"{"id":"l1","status":400,
      "error":{"code":-2028,"msg":"HMAC-SHA-256 API key is not supported."}}"#;

    // https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream
    const SPOT_EXECUTION_REPORT: &str = r#"{
      "subscriptionId": 0,
      "event": {
      "e": "executionReport",
      "E": 1499405658658,
      "s": "ETHBTC",
      "c": "mUvoqJxFIILMdfAW5iGSOW",
      "S": "BUY",
      "o": "LIMIT",
      "f": "GTC",
      "q": "1.00000000",
      "p": "0.10264410",
      "P": "0.00000000",
      "F": "0.00000000",
      "g": -1,
      "C": "",
      "x": "NEW",
      "X": "NEW",
      "r": "NONE",
      "i": 4293153,
      "l": "0.00000000",
      "z": "0.00000000",
      "L": "0.00000000",
      "n": "0",
      "N": null,
      "T": 1499405658657,
      "t": -1,
      "I": 8641984,
      "w": true,
      "m": false,
      "M": false,
      "O": 1499405658657,
      "Z": "0.00000000",
      "Y": "0.00000000",
      "Q": "0.00000000"
      }
    }"#;

    // The same order as above, filled five hours after it was placed: `O` is
    // when Binance accepted it and `T` is when this fill happened. Binance's own
    // documented example, the one above, has the two equal, which is the single
    // case that cannot tell them apart.
    // https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream
    const SPOT_EXECUTION_REPORT_FILL: &str = r#"{
      "subscriptionId": 0,
      "event": {
      "e": "executionReport",
      "E": 1499423658658,
      "s": "ETHBTC",
      "c": "mUvoqJxFIILMdfAW5iGSOW",
      "S": "BUY",
      "o": "LIMIT",
      "f": "GTC",
      "q": "1.00000000",
      "p": "0.10264410",
      "P": "0.00000000",
      "F": "0.00000000",
      "g": -1,
      "C": "",
      "x": "TRADE",
      "X": "FILLED",
      "r": "NONE",
      "i": 4293153,
      "l": "1.00000000",
      "z": "1.00000000",
      "L": "0.10264410",
      "n": "0",
      "N": null,
      "T": 1499423658657,
      "t": 92,
      "I": 8641984,
      "w": false,
      "m": true,
      "M": true,
      "O": 1499405658657,
      "Z": "0.10264410",
      "Y": "0.10264410",
      "Q": "0.00000000"
      }
    }"#;

    // https://developers.binance.com/docs/derivatives/usds-margined-futures/user-data-streams/Event-Balance-and-Position-Update
    const FUTURES_ACCOUNT_UPDATE: &str = r#"{
      "e": "ACCOUNT_UPDATE",
      "E": 1564745798939,
      "T": 1564745798938,
      "a": {
        "m": "ORDER",
        "B": [
          { "a": "USDT", "wb": "122624.12345678", "cw": "100.12345678", "bc": "50.12345678" }
        ],
        "P": []
      }
    }"#;

    // https://developers.binance.com/docs/derivatives/usds-margined-futures/user-data-streams/Event-Order-Update
    const FUTURES_ORDER_UPDATE: &str = r#"{
      "e": "ORDER_TRADE_UPDATE",
      "E": 1568879465651,
      "T": 1568879465650,
      "o": {
        "s": "BTCUSDT",
        "c": "TEST",
        "S": "SELL",
        "o": "TRAILING_STOP_MARKET",
        "f": "GTC",
        "q": "0.001",
        "p": "0",
        "ap": "0",
        "sp": "7103.04",
        "x": "NEW",
        "X": "NEW",
        "i": 8886774,
        "l": "0",
        "z": "0",
        "L": "0",
        "T": 1568879465651,
        "t": 0,
        "b": "0",
        "a": "9.91",
        "m": false,
        "R": false,
        "wt": "CONTRACT_PRICE",
        "ot": "TRAILING_STOP_MARKET",
        "ps": "LONG",
        "cp": false,
        "rp": "0"
      }
    }"#;

    fn spot() -> BinanceAdapter {
        BinanceAdapter::spot()
    }

    fn perp() -> BinanceAdapter {
        BinanceAdapter::usd_m_futures()
    }

    fn subscription(market: Market) -> Subscription {
        Subscription::new()
            .market(market)
            .feed(Feed::Trades)
            .feed(Feed::OrderBook)
            .feed(Feed::Candles(Interval::Min1))
    }

    #[test]
    fn spot_names_every_market_and_feed_in_lowercase_on_one_socket() {
        let subscription = subscription(Market::spot(Exchange::Binance, "BNB", "BTC"))
            .market(Market::spot(Exchange::Binance, "ETH", "BTC"));

        let frames = subscribe_frames(&spot(), &subscription).expect("a valid subscription");
        assert_eq!(frames.len(), 1, "spot is not split across entry points");
        let (url, frame) = &frames[0];
        assert_eq!(*url, SPOT_WEBSOCKET_URL);
        let value: Value = serde_json::from_str(frame).expect("valid JSON");

        assert_eq!(value["method"], "SUBSCRIBE");
        assert_eq!(
            value["params"],
            serde_json::json!([
                "bnbbtc@trade",
                "bnbbtc@depth20@100ms",
                "bnbbtc@kline_1m",
                "ethbtc@trade",
                "ethbtc@depth20@100ms",
                "ethbtc@kline_1m",
            ])
        );
    }

    /// USD-M serves `@trade` and `@depth*` only under `/public`, and
    /// `@kline_*` and `@ticker` only under `/market`. Measured 2026-07-30: a
    /// `/public/stream` socket subscribed to all four returned 149 `@trade`
    /// and 229 `@depth20@100ms` frames in 25 s and zero `@kline_1m` or
    /// `@ticker`; a `/market/stream` socket returned 47 and 12 of those and
    /// zero of the first two. Both acknowledged the whole subscription.
    #[test]
    fn usd_m_sends_each_feed_to_the_entry_point_that_carries_it() {
        let subscription = Subscription::new()
            .market(Market::perpetual(Exchange::Binance, "BTC", "USDT"))
            .feed(Feed::Trades)
            .feed(Feed::OrderBook)
            .feed(Feed::Ticker)
            .feed(Feed::Candles(Interval::Min1));

        let groups = stream_groups(&perp(), &subscription).expect("a valid subscription");

        assert_eq!(
            groups,
            vec![
                (
                    USD_M_PUBLIC_WEBSOCKET_URL,
                    vec![
                        "btcusdt@trade".to_string(),
                        "btcusdt@depth20@100ms".to_string(),
                    ],
                ),
                (
                    USD_M_MARKET_WEBSOCKET_URL,
                    vec!["btcusdt@ticker".to_string(), "btcusdt@kline_1m".to_string(),],
                ),
            ]
        );
    }

    /// Binance decommissioned the unrouted USD-M paths on 2026-04-23. A socket
    /// that names no entry point is served as if it had named `/public`, so it
    /// acknowledges a `@kline_*` or `@ticker` subscription and then delivers
    /// nothing for it, with no error and no close.
    #[test]
    fn no_usd_m_socket_opens_on_a_path_that_names_no_entry_point() {
        for feed in [
            Feed::Trades,
            Feed::OrderBook,
            Feed::Ticker,
            Feed::Candles(Interval::Min1),
        ] {
            let url = entry_point_url(BinanceMarket::UsdMFutures, feed);
            assert!(
                url.starts_with("wss://fstream.binance.com/public/")
                    || url.starts_with("wss://fstream.binance.com/market/"),
                "{feed:?} routes to `{url}`"
            );
        }
    }

    #[test]
    fn both_venues_stream_every_fill_off_the_same_stream_name() {
        assert_eq!(
            feed_name(BinanceMarket::Spot, Feed::Trades).expect("a feed"),
            "trade"
        );
        // Not `aggTrade`: it numbers its messages out of a second id space
        // that no REST call answers, and on 2026-07-30 it delivered nothing.
        assert_eq!(
            feed_name(BinanceMarket::UsdMFutures, Feed::Trades).expect("a feed"),
            "trade"
        );
    }

    #[test]
    fn a_candle_feed_futures_does_not_stream_is_refused() {
        assert!(matches!(
            feed_name(BinanceMarket::UsdMFutures, Feed::Candles(Interval::Sec1)),
            Err(Error::Unsupported {
                feature: Feature::Candles,
                ..
            })
        ));
        assert_eq!(
            feed_name(BinanceMarket::Spot, Feed::Candles(Interval::Sec1)).expect("a feed"),
            "kline_1s"
        );
    }

    #[test]
    fn every_candle_stream_name_round_trips_through_its_interval() {
        for interval in [
            Interval::Sec1,
            Interval::Min1,
            Interval::Min3,
            Interval::Min5,
            Interval::Min15,
            Interval::Min30,
            Interval::Hour1,
            Interval::Hour2,
            Interval::Hour4,
            Interval::Hour8,
            Interval::Hour12,
            Interval::Day1,
            Interval::Day3,
            Interval::Week1,
            Interval::Month1,
        ] {
            let code = BinanceMarket::Spot
                .interval_code(interval)
                .expect("a served interval");
            assert_eq!(interval_from_code(code), Some(interval), "{code}");
        }
        // A minute and a month differ by case alone.
        assert_eq!(interval_from_code("1M"), Some(Interval::Month1));
        assert_eq!(interval_from_code("1m"), Some(Interval::Min1));
    }

    #[test]
    fn subscribing_to_nothing_is_refused_before_the_socket_opens() {
        let no_feed = Subscription::new().market(Market::spot(Exchange::Binance, "BTC", "USDT"));
        let no_market = Subscription::new().feed(Feed::Trades);

        assert!(matches!(
            stream_groups(&spot(), &no_feed),
            Err(Error::InvalidRequest { field: "feeds", .. })
        ));
        assert!(matches!(
            stream_groups(&spot(), &no_market),
            Err(Error::InvalidRequest {
                field: "markets",
                ..
            })
        ));
    }

    #[test]
    fn a_trade_frame_becomes_a_trade_on_the_market_its_stream_names() {
        let Some(MarketEvent::Trade(trade)) = decode(&spot(), SPOT_TRADE_FRAME).expect("a frame")
        else {
            panic!("expected a trade event");
        };

        assert_eq!(trade.market, Market::spot(Exchange::Binance, "BNB", "BTC"));
        assert_eq!(trade.taker_side, Side::Sell);
        assert_eq!(trade.id.as_deref(), Some("12345"));
        assert_eq!(trade.price.to_string(), "0.001");
    }

    #[test]
    fn a_futures_trade_lands_on_a_perpetual_market_under_its_rest_id() {
        let Some(MarketEvent::Trade(trade)) =
            decode(&perp(), FUTURES_TRADE_FRAME).expect("a frame")
        else {
            panic!("expected a trade event");
        };

        assert_eq!(
            trade.market,
            Market::perpetual(Exchange::Binance, "DOGE", "USDT")
        );
        // `t`, the same id `/fapi/v1/trades` returns for this fill, so a caller
        // can deduplicate the stream against REST on the id alone.
        assert_eq!(trade.id.as_deref(), Some("3417626319"));
        // The individual fill, not a sum over the fills a taker order swept.
        assert_eq!(trade.quantity.to_string(), "700");
        // `T`, the match time, not the later `E` the frame was published at.
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_785_407_770_046));
    }

    #[test]
    fn a_spent_futures_trade_id_is_not_reported_as_a_trade_at_zero() {
        assert_eq!(
            decode(&perp(), FUTURES_SPENT_TRADE_ID_FRAME).expect("a frame"),
            None
        );
    }

    #[test]
    fn a_streamed_candle_says_for_itself_whether_it_has_closed() {
        let Some(MarketEvent::Candle(candle)) = decode(&spot(), SPOT_KLINE_FRAME).expect("a frame")
        else {
            panic!("expected a candle event");
        };

        assert_eq!(candle.interval, Interval::Min1);
        assert_eq!(candle.open_time, Timestamp::from_millis(1_672_515_780_000));
        assert_eq!(candle.close.to_string(), "0.0020");
        assert_eq!(
            candle.quote_volume.expect("a quote volume").to_string(),
            "1.0000"
        );
        // Unlike a REST candle, this is Binance's own answer rather than a
        // comparison against the clock.
        assert!(!candle.closed);
    }

    #[test]
    fn a_partial_depth_frame_becomes_a_sorted_book() {
        let Some(MarketEvent::OrderBook(book)) =
            decode(&spot(), SPOT_DEPTH_FRAME).expect("a frame")
        else {
            panic!("expected an order book event");
        };

        assert_eq!(book.best_bid().expect("a bid").price.to_string(), "0.0024");
        assert_eq!(book.best_ask().expect("an ask").price.to_string(), "0.0026");
        assert_eq!(book.bids.len(), 2);
        assert_eq!(book.market, Market::spot(Exchange::Binance, "BNB", "BTC"));
    }

    #[test]
    fn a_subscribe_acknowledgement_is_not_reported_as_market_data() {
        assert!(
            decode(&spot(), r#"{"result":null,"id":1}"#)
                .expect("a control frame")
                .is_none()
        );
    }

    #[test]
    fn an_error_frame_carries_binances_own_code_and_message() {
        let error = decode(
            &spot(),
            r#"{"error":{"code":2,"msg":"Invalid request: request ID must be an unsigned integer"},"id":1}"#,
        )
        .expect_err("an error frame");

        assert!(matches!(
            &error,
            Error::Exchange { exchange: "binance", code, status: None, .. } if code == "2"
        ));
    }

    #[test]
    fn a_frame_maxt_cannot_place_is_an_error_not_a_silent_drop() {
        assert!(matches!(
            decode(&spot(), r#"{"data":{"e":"trade"}}"#),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(
            decode(&spot(), r#"{"stream":"bnbbtc","data":{}}"#),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(
            decode(&spot(), r#"{"stream":"bnbbtc@somethingNew","data":{}}"#),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(
            decode(&spot(), "not json"),
            Err(Error::Decode { .. })
        ));
    }

    #[test]
    fn a_spot_balance_frame_becomes_one_event_per_asset() {
        let events = decode_account(&spot(), SPOT_BALANCE_FRAME).expect("a balance frame");

        assert_eq!(events.len(), 1);
        let AccountEvent::Balance(balance) = &events[0] else {
            panic!("expected a balance event");
        };
        assert_eq!(balance.asset, "ETH");
        assert_eq!(balance.available.to_string(), "10000.000000");
        assert_eq!(balance.locked.to_string(), "0.000000");
    }

    #[test]
    fn a_futures_balance_frame_splits_the_wallet_at_the_cross_boundary() {
        let events = decode_account(&perp(), FUTURES_ACCOUNT_UPDATE).expect("a balance frame");

        let AccountEvent::Balance(balance) = &events[0] else {
            panic!("expected a balance event");
        };
        assert_eq!(balance.asset, "USDT");
        assert_eq!(balance.available.to_string(), "100.12345678");
        assert_eq!(balance.locked.to_string(), "122524.00000000");
    }

    #[test]
    fn both_venues_order_updates_read_into_the_same_order() {
        let spot_events = decode_account(&spot(), SPOT_EXECUTION_REPORT).expect("an order frame");
        let futures_events = decode_account(&perp(), FUTURES_ORDER_UPDATE).expect("an order frame");

        let AccountEvent::Order(spot_order) = &spot_events[0] else {
            panic!("expected an order event");
        };
        let AccountEvent::Order(futures_order) = &futures_events[0] else {
            panic!("expected an order event");
        };

        assert_eq!(spot_order.id, "4293153");
        assert_eq!(
            spot_order.market,
            Market::spot(Exchange::Binance, "ETH", "BTC")
        );
        assert_eq!(spot_order.status, OrderStatus::Open);
        assert_eq!(spot_order.remaining_quantity.to_string(), "1.00000000");
        assert_eq!(spot_order.price.expect("a price").to_string(), "0.10264410");

        assert_eq!(futures_order.id, "8886774");
        assert_eq!(
            futures_order.market,
            Market::perpetual(Exchange::Binance, "BTC", "USDT")
        );
        assert_eq!(futures_order.side, Side::Sell);
        // A trailing stop has no limit price, and Binance writes that as zero.
        assert_eq!(futures_order.price, None);
    }

    #[test]
    fn an_order_is_dated_to_when_it_was_created_or_not_at_all() {
        let filled = decode_account(&spot(), SPOT_EXECUTION_REPORT_FILL).expect("an order frame");
        let futures = decode_account(&perp(), FUTURES_ORDER_UPDATE).expect("an order frame");

        let AccountEvent::Order(filled) = &filled[0] else {
            panic!("expected an order event");
        };
        let AccountEvent::Order(futures) = &futures[0] else {
            panic!("expected an order event");
        };

        assert_eq!(filled.status, OrderStatus::Filled);
        // `O`, the creation time, and not `T`, the time of this fill. Reading
        // the same order back over REST reports the creation time, and one
        // order may not have two ages depending on which transport saw it.
        assert_eq!(
            filled.created_at,
            Some(Timestamp::from_millis(1_499_405_658_657))
        );

        // USD-M publishes no creation time, and the transaction time it does
        // publish is not one: on this frame it is the same clock the REST read
        // calls `updateTime`, and on an order that rests and later fills it is
        // the fill. Reporting it would give the one order two ages that drift
        // apart with every amendment, so nothing is reported.
        assert!(FUTURES_ORDER_UPDATE.contains("\"T\": 1568879465651"));
        assert_eq!(futures.created_at, None);
    }

    #[test]
    fn the_heartbeat_is_a_protocol_ping_because_a_text_one_would_be_read_as_a_command() {
        // Binance answers an unrecognised text frame with an error rather than
        // ignoring it, so the keepalive has to live below the API entirely.
        assert_eq!(HEARTBEAT.frame, HeartbeatFrame::Ping);

        // The floor is the part that matters. Binance pings the client every
        // three minutes and only hangs up after ten with no pong, so on an
        // account that never moves three minutes of quiet is a healthy socket.
        // A floor under that reconnects a working stream forever.
        assert!(HEARTBEAT.min_idle_timeout > Duration::from_secs(3 * 60));
        assert!(HEARTBEAT.interval * 3 <= Duration::from_secs(3 * 60));
    }

    #[tokio::test]
    async fn a_listen_key_that_could_not_be_extended_reaches_the_consumer() {
        let (failures, failed) = mpsc::channel(1);
        // A socket with nothing to say, as a quiet account's really is.
        let quiet = futures_util::stream::pending().boxed();
        let mut stream = Box::pin(with_refresher_failures(quiet, failed));

        // What `keepalive_listen_key` hands back, forwarded rather than
        // restated: Binance's own verdict on the key is what tells a lapsed one
        // from a network blip, and the refresher adds nothing to it.
        failures
            .send(Error::exchange_http(
                EXCHANGE,
                400,
                "-1125",
                "This listenKey does not exist.",
            ))
            .await
            .expect("the stream is still listening");

        // Without this the socket stays up, goes quiet when the key lapses, and
        // then fails every reconnect, none of which the consumer ever hears.
        let reported = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .expect("a report before the deadline");
        assert!(
            matches!(&reported, Some(Err(Error::Exchange { code, .. })) if code == "-1125"),
            "{reported:?}"
        );
    }

    #[tokio::test]
    async fn a_refresher_that_is_still_going_does_not_hold_a_finished_stream_open() {
        // The socket has given up; the refresher is mid-sleep on a key that is
        // still perfectly good, so it keeps its end of the channel open.
        let (_failures, failed) = mpsc::channel::<Error>(1);
        let done = futures_util::stream::empty().boxed();
        let mut stream = Box::pin(with_refresher_failures(done, failed));

        // The stream has to end anyway. Waiting on the refresher instead would
        // leave the consumer awaiting a stream that will never speak again,
        // the same fault the refresher's reporting exists to prevent.
        let ended = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .expect("the stream to end rather than wait on the refresher");
        assert!(ended.is_none());
    }

    #[tokio::test]
    async fn a_refresher_that_never_failed_adds_nothing_to_the_stream() {
        let (failures, failed) = mpsc::channel::<Error>(1);
        let events = futures_util::stream::iter(vec![Ok(AccountEvent::Reconnected)]).boxed();
        let mut stream = Box::pin(with_refresher_failures(events, failed));

        drop(failures);
        assert!(matches!(
            stream.next().await,
            Some(Ok(AccountEvent::Reconnected))
        ));
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn dropping_the_stream_stops_the_refresher_without_waiting_out_its_sleep() {
        let (failures, failed) = mpsc::channel::<Error>(1);
        // The interval the refresher really uses. A refresher that notices the
        // drop only when this elapses keeps a cloned adapter alive for half an
        // hour after the stream it serves is gone, and a process that opens and
        // drops account streams collects one such task per stream.
        let waiting = tokio::spawn(async move { due(LISTEN_KEY_LIFETIME / 2, &failures).await });

        drop(failed);

        let carry_on = tokio::time::timeout(Duration::from_secs(5), waiting)
            .await
            .expect("the wait to end with the stream")
            .expect("no panic");
        assert!(!carry_on);
    }

    /// A spot account socket opens on the WebSocket API host with no
    /// credential in the URL, and names the account in its first frame.
    ///
    /// The old shape put a listen key in the URL of the market data host and
    /// minted it at `POST /api/v3/userDataStream`, which Binance removed on
    /// 2026-02-20 07:00 UTC and now answers `410 Gone`. Restoring either half
    /// fails here.
    #[test]
    fn a_spot_account_socket_opens_on_the_websocket_api_and_names_the_account_in_a_frame() {
        let adapter = spot().with_credentials("key", "secret");
        let connect = spot_account_connect(&adapter).expect("credentials are set");

        assert_eq!(connect.url, "wss://ws-api.binance.com:443/ws-api/v3");
        // No listen key anywhere: not in the URL, and not in the frame.
        assert!(!connect.url.contains("listenKey"), "{}", connect.url);
        assert!(
            !connect.url.contains("stream.binance.com"),
            "{}",
            connect.url
        );
        assert!(connect.headers.is_none());

        let minted = (connect.subscribe)().expect("a signed subscribe frame");
        let [frame] = minted.as_slice() else {
            panic!("expected exactly one subscribe frame");
        };
        let parsed: serde_json::Value = serde_json::from_str(frame).expect("a JSON frame");
        assert_eq!(parsed["method"], "userDataStream.subscribe.signature");
        assert!(!frame.contains("listenKey"), "{frame}");

        // Binance pings this socket itself, so the keepalive only has to keep
        // the idle timer above its pace.
        assert!(connect.heartbeat.is_some());
    }

    /// Every socket a spot account stream opens subscribes with a signature
    /// minted for it.
    ///
    /// The frame signs the millisecond clock it was built at, and Binance
    /// refuses it `-1021` once `recvWindow` has passed, measured 2026-07-31 at
    /// 90 s against a window of 60 000 ms. A stream that kept the frame it
    /// first subscribed with would replay a dead signature onto every socket
    /// after a longer outage, and the refusal reaches the consumer over and
    /// over instead of the account's events.
    ///
    /// What is asserted is that the frames differ eventually, never how fast
    /// the clock moves: the deadline is loose enough to mean nothing but
    /// "never", and a frame captured once at construction never differs however
    /// long it is given.
    #[test]
    fn every_spot_account_socket_signs_its_own_subscribe_frame() {
        let adapter = spot().with_credentials("key", "secret");
        let connect = spot_account_connect(&adapter).expect("credentials are set");
        let first = (connect.subscribe)().expect("a signed subscribe frame");

        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        let differed = loop {
            if (connect.subscribe)().expect("another signed frame") != first {
                break true;
            }
            if std::time::Instant::now() >= deadline {
                break false;
            }
        };

        assert!(differed, "every mint returned the same frame: {first:?}");
    }

    #[test]
    fn an_expired_listen_key_ends_the_stream_rather_than_going_quiet() {
        let error = decode_account(&perp(), r#"{"e":"listenKeyExpired","E":1699596037418}"#)
            .expect_err("an expired listen key");

        // The event name is the code, so the two ways Binance ends a
        // subscription stay apart.
        assert!(
            matches!(&error, Error::Exchange { code, .. } if code == "listenKeyExpired"),
            "{error:?}"
        );
    }

    /// The one frame a spot user data socket is guaranteed to produce on an
    /// account that never trades, and so the only proof available that the
    /// socket is carrying that account rather than merely open. A stream that
    /// swallowed it would leave a consumer waiting on a subscription Binance
    /// has already ended.
    #[test]
    fn a_terminated_spot_subscription_ends_the_stream_rather_than_going_quiet() {
        let error =
            decode_account(&spot(), SPOT_STREAM_TERMINATED).expect_err("a terminated subscription");

        assert!(
            matches!(&error, Error::Exchange { code, .. } if code == "eventStreamTerminated"),
            "{error:?}"
        );
        assert!(error.to_string().contains("subscribe again"), "{error}");
    }

    /// Binance answers a refused subscription and then leaves the socket open
    /// and silent, and an account with no balances and no open orders is silent
    /// on a working socket too. A consumer told nothing could not tell the two
    /// apart, so the refusal is an error on the stream.
    ///
    /// It carries Binance's own code rather than becoming
    /// [`Error::Auth`](crate::Error::Auth), and these two are why: a stale
    /// timestamp and a key of the wrong type are both refusals of the same
    /// frame, and a caller who cannot tell them apart cannot tell a clock
    /// worth fixing from a key worth replacing.
    #[test]
    fn a_refused_spot_subscription_reaches_the_consumer_with_binances_own_code() {
        for (frame, expected) in [
            (SPOT_SUBSCRIBE_STALE, "-1021"),
            (SPOT_LOGON_REFUSED, "-2028"),
        ] {
            let error = decode_account(&spot(), frame).expect_err("a refusal");

            let Error::Exchange {
                code,
                status,
                kind,
                message,
                ..
            } = &error
            else {
                panic!("expected the exchange's own verdict, got {error:?}");
            };
            assert_eq!(code, expected, "{error:?}");
            // The status is inside the frame on this socket, not on an HTTP
            // response, and it is what classifies the refusal.
            assert_eq!(*status, Some(400), "{error:?}");
            assert_eq!(*kind, crate::ExchangeErrorKind::Rejected, "{error:?}");
            // Binance's sentence, unedited: it is the difference between a
            // signature problem and a clock problem.
            assert!(!message.is_empty(), "{error:?}");
            assert!(error.to_string().contains(expected), "{error}");
        }

        // The subscription that took carries no event, and says nothing.
        assert!(
            decode_account(&spot(), SPOT_SUBSCRIBE_ACCEPTED)
                .expect("an accepted subscription")
                .is_empty()
        );
    }

    #[test]
    fn an_account_event_maxt_does_not_model_is_dropped_rather_than_failing() {
        // Binance publishes these on the same socket; none of them changes a
        // balance or an order. Spot wraps them and USD-M does not.
        for frame in [
            r#"{"subscriptionId":0,"event":{"e":"balanceUpdate","E":1573200697110,"a":"BTC","d":"100.00000000","T":1}}"#,
            r#"{"subscriptionId":0,"event":{"e":"listStatus","E":1,"s":"ETHBTC"}}"#,
        ] {
            assert!(
                decode_account(&spot(), frame)
                    .expect("a known frame")
                    .is_empty(),
                "{frame}"
            );
        }
        assert!(
            decode_account(&perp(), r#"{"e":"MARGIN_CALL","E":1587727187525,"p":[]}"#)
                .expect("a known frame")
                .is_empty()
        );
    }
}
