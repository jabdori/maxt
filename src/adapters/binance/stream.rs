//! Binance public and account WebSocket streams.
//!
//! Public data uses combined-stream wrappers so every payload has a stream
//! name. USD-M routes feeds across `/public` and `/market` sockets.

use std::collections::HashMap;
use std::time::Duration;

use futures_core::Stream;
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

/// Protocol-level keepalive and the minimum safe idle timeout.
///
/// Text frames are API commands, so the adapter sends protocol pings. The idle
/// floor stays above Binance's server-ping interval to avoid reconnecting a
/// healthy but quiet account stream.
const HEARTBEAT: Heartbeat = Heartbeat {
    interval: Duration::from_secs(60),
    frame: HeartbeatFrame::Ping,
    // Longer than the three minutes between Binance's own pings, so a quiet
    // account is not mistaken for a dead socket.
    min_idle_timeout: Duration::from_secs(240),
};

/// Binance's stream suffix for one public feed.
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

/// The public endpoint that carries a feed.
///
/// Spot uses one endpoint. USD-M routes trades and order books to `/public`,
/// and tickers and candles to `/market`.
fn entry_point_url(venue: BinanceMarket, feed: Feed) -> &'static str {
    match (venue, feed) {
        (BinanceMarket::Spot, _) => SPOT_WEBSOCKET_URL,
        (BinanceMarket::UsdMFutures, Feed::Trades | Feed::OrderBook) => USD_M_PUBLIC_WEBSOCKET_URL,
        (BinanceMarket::UsdMFutures, Feed::Ticker | Feed::Candles(_)) => USD_M_MARKET_WEBSOCKET_URL,
    }
}

/// Groups subscription names by endpoint; each group opens one socket.
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

/// Markets keyed exactly as the combined-stream wrapper names them.
fn subscribed_markets(
    adapter: &BinanceAdapter,
    subscription: &Subscription,
) -> Result<HashMap<String, Market>> {
    subscription
        .markets()
        .iter()
        .map(|market| Ok((adapter.symbol(market)?.to_ascii_lowercase(), market.clone())))
        .collect()
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
pub(super) fn decode(
    markets: &HashMap<String, Market>,
    frame: &str,
) -> Result<Option<MarketEvent>> {
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
    let market = markets.get(symbol).cloned().ok_or_else(|| {
        Error::decode(format!(
            "binance frame names `{symbol}`, which was not in this socket's subscription"
        ))
    })?;

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

/// Merges split endpoint sessions and ends when any session ends.
fn merge_until_any_ends<S>(sessions: Vec<S>) -> impl Stream<Item = S::Item>
where
    S: Stream + Send + Unpin,
{
    select_all(
        sessions
            .into_iter()
            .map(|session| session.map(Some).chain(futures_util::stream::iter([None]))),
    )
    .take_while(|item| std::future::ready(item.is_some()))
    .filter_map(std::future::ready)
}

/// Opens one socket per endpoint the subscription reaches and merges them.
///
/// A USD-M subscription that names both a book feed and an aggregated one
/// spans two entry points, and one socket cannot serve both. Each socket
/// reconnects on its own, so such a subscription reports
/// [`MarketEvent::Reconnected`] once per socket that comes back rather than
/// once per outage. If either socket ends, the merged stream ends and drops the
/// other socket instead of silently losing half its feeds.
pub(super) async fn subscribe(
    adapter: &BinanceAdapter,
    subscription: &Subscription,
    config: &StreamConfig,
) -> Result<MarketStream> {
    let markets = subscribed_markets(adapter, subscription)?;
    let mut sessions = Vec::new();
    let mut closes = Vec::new();
    for (url, frame) in subscribe_frames(adapter, subscription)? {
        let session = connect(
            WsConnect {
                url: url.to_string(),
                headers: None,
                subscribe: WsConnect::fixed(vec![frame]),
                heartbeat: Some(HEARTBEAT),
            },
            config,
        )
        .await?;
        closes.push(session.close_handle());
        sessions.push(session);
    }

    Ok(MarketStream::new_with_close(
        merge_until_any_ends(sessions).filter_map(move |item| {
            let event = match item {
                Ok(WsCommand::Text(frame)) => decode(&markets, &frame).transpose(),
                // Binance's public streams are text only; a binary frame means
                // something changed on their side that `maxt` has not read.
                Ok(WsCommand::Binary(_)) => Some(Err(Error::decode(
                    "binance sent an unexpected binary frame",
                ))),
                Ok(WsCommand::Reconnected) => Some(Ok(MarketEvent::Reconnected)),
                Err(error) => Some(Err(error)),
            };
            std::future::ready(event)
        }),
        move || async move {
            for result in
                futures_util::future::join_all(closes.iter().map(|close| close.close())).await
            {
                result?;
            }
            Ok(())
        },
    ))
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
        /// Spot order creation time `O`. Transaction time `T` is not a
        /// substitute. Optional because the shared USD-M shape omits it.
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
    /// Spot: the account subscription ended.
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
    /// Order creation time. USD-M currently omits it; transaction time `T` is
    /// not substituted because it dates the update rather than the order.
    #[serde(rename = "O", default)]
    created_at: Option<i64>,
}

/// A Spot WebSocket API response or pushed account event.
/// Response fields and event fields are mutually exclusive, so they are
/// optional in the shared envelope.
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

/// Decodes one private frame. Balance frames can expand into several events;
/// unsupported event types return an empty list.
pub(super) fn decode_account(adapter: &BinanceAdapter, frame: &str) -> Result<Vec<AccountEvent>> {
    match adapter.venue() {
        BinanceMarket::Spot => decode_spot_account(adapter, frame),
        BinanceMarket::UsdMFutures => {
            decode_account_event(adapter, parse::json(frame, "user data frame")?)
        }
    }
}

/// Reads a Spot WebSocket API response or account event.
/// A refused subscription keeps Binance's HTTP status and exchange code.
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
                    // The stream omits free balance. `cw` is the closest value
                    // available but can include margin held by cross positions.
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
        // Termination events have no message; their event names remain distinct
        // exchange error codes.
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

/// Opens a Spot account stream on the WebSocket API.
/// Spot uses a signed subscription frame and no listen key.
async fn subscribe_spot_account(
    adapter: &BinanceAdapter,
    config: &StreamConfig,
) -> Result<AccountStream> {
    let session = connect(spot_account_connect(adapter)?, config).await?;
    let close = session.close_handle();

    Ok(AccountStream::new_with_close(
        account_events(adapter.clone(), session).boxed(),
        move || async move { close.close().await },
    ))
}

/// Builds a Spot account connection that signs a fresh frame per handshake.
fn spot_account_connect(adapter: &BinanceAdapter) -> Result<WsConnect> {
    adapter.credentials()?;

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
    let close = session.close_handle();

    // The channel forwards a refresh failure and stops the task when the stream
    // is dropped.
    let (failures, failed) = mpsc::channel(1);
    let refresh = tokio::spawn(refresh_listen_key(adapter.clone(), key, failures));

    let events = account_events(adapter.clone(), session).boxed();

    Ok(AccountStream::new_with_close(
        with_refresher_failures(events, failed),
        move || async move {
            let (socket, refresh) = tokio::join!(close.close(), stop_refresh_task(refresh));
            socket?;
            refresh
        },
    ))
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

/// Merges a listen-key refresh failure without keeping a finished socket open.
fn with_refresher_failures(
    events: futures_util::stream::BoxStream<'static, Result<AccountEvent>>,
    failed: mpsc::Receiver<Error>,
) -> impl futures_core::Stream<Item = Result<AccountEvent>> + Send {
    futures_util::stream::unfold((events, failed), |(mut events, mut failed)| async move {
        tokio::select! {
            // Report an already queued refresh failure before another frame.
            biased;
            Some(error) = failed.recv() => Some((Err(error), (events, failed))),
            item = events.next() => item.map(|item| (item, (events, failed))),
        }
    })
}

/// Waits for the refresh interval, or returns `false` when the stream is dropped.
async fn due(interval: Duration, failures: &mpsc::Sender<Error>) -> bool {
    tokio::select! {
        () = tokio::time::sleep(interval) => true,
        () = failures.closed() => false,
    }
}

/// Extends a USD-M listen key until the stream is dropped or a refresh fails.
/// The first failure is forwarded unchanged to the account stream.
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

/// Stops and joins the USD-M listen-key refresh task.
async fn stop_refresh_task(task: tokio::task::JoinHandle<()>) -> Result<()> {
    task.abort();
    match task.await {
        Ok(()) => Ok(()),
        Err(error) if error.is_cancelled() => Ok(()),
        Err(error) => Err(Error::adapter(format!(
            "Binance listen-key refresh task failed: {error}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use super::*;
    use crate::feature::Feature;
    use crate::types::{Exchange, OrderStatus, Side};

    struct RefreshStopped(Arc<AtomicBool>);

    impl Drop for RefreshStopped {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    fn spawn_refresh_fixture(stopped: Arc<AtomicBool>) -> AccountStream {
        let stopped = RefreshStopped(stopped);
        let task = tokio::spawn(async move {
            let _stopped = stopped;
            std::future::pending::<()>().await;
        });

        AccountStream::new_with_close(futures_util::stream::pending(), move || {
            stop_refresh_task(task)
        })
    }

    #[tokio::test]
    async fn closing_a_usd_m_account_source_joins_the_refresh_task() {
        let stopped = Arc::new(AtomicBool::new(false));
        let mut task = spawn_refresh_fixture(Arc::clone(&stopped));

        assert!(!stopped.load(Ordering::SeqCst));
        task.close().await.unwrap();
        assert!(stopped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn a_panicking_refresh_task_is_a_structured_adapter_error() {
        let task = tokio::spawn(async { panic!("refresh fixture panic") });
        while !task.is_finished() {
            tokio::task::yield_now().await;
        }

        assert!(matches!(
            stop_refresh_task(task).await,
            Err(Error::Adapter { detail }) if detail.contains("listen-key refresh task")
        ));
    }

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

    // A representative USD-M combined-stream trade frame.
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

    // USD-M can announce a consumed trade id with zero price and quantity.
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

    // Spot WebSocket API events use a subscription envelope.
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

    // Spot reports a terminated user-data subscription as an event.
    const SPOT_STREAM_TERMINATED: &str =
        r#"{"subscriptionId":0,"event":{"e":"eventStreamTerminated","E":1785442936161}}"#;

    // Successful Spot account-subscription response.
    const SPOT_SUBSCRIBE_ACCEPTED: &str = r#"{"id":"1785442705539","status":200,
      "result":{"subscriptionId":0},
      "rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":6000,"count":4}]}"#;

    // Spot response to a subscription whose timestamp is stale.
    const SPOT_SUBSCRIBE_STALE: &str = r#"{"id":"st1","status":400,
      "error":{"code":-1021,"msg":"Timestamp for this request is outside of the recvWindow."},
      "rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":6000,"count":16}]}"#;

    // Spot response when HMAC credentials are used with `session.logon`.
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

    fn decode_for(
        adapter: &BinanceAdapter,
        market: Market,
        frame: &str,
    ) -> Result<Option<MarketEvent>> {
        let subscription = Subscription::new().market(market).feed(Feed::Trades);
        decode(&subscribed_markets(adapter, &subscription)?, frame)
    }

    fn decode_spot(frame: &str) -> Result<Option<MarketEvent>> {
        decode_for(
            &spot(),
            Market::spot(Exchange::Binance, "BNB", "BTC"),
            frame,
        )
    }

    fn decode_perp(frame: &str) -> Result<Option<MarketEvent>> {
        decode_for(
            &perp(),
            Market::perpetual(Exchange::Binance, "DOGE", "USDT"),
            frame,
        )
    }

    #[tokio::test]
    async fn a_split_subscription_ends_when_either_socket_ends() {
        type TestStream = std::pin::Pin<Box<dyn futures_core::Stream<Item = u8> + Send + 'static>>;

        let ending: TestStream = Box::pin(futures_util::stream::iter([1]));
        let still_open: TestStream = Box::pin(futures_util::stream::pending());

        let items: Vec<_> = merge_until_any_ends(vec![ending, still_open])
            .collect()
            .await;

        assert_eq!(items, [1]);
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

    #[test]
    fn a_non_ascii_spot_asset_is_escaped_by_the_subscription_json() {
        let subscription = Subscription::new()
            .market(Market::spot(Exchange::Binance, "币安人生", "USDT"))
            .feed(Feed::Trades);

        let frames = subscribe_frames(&spot(), &subscription).expect("a valid subscription");
        let value: Value = serde_json::from_str(&frames[0].1).expect("valid JSON");

        assert_eq!(value["params"], serde_json::json!(["币安人生usdt@trade"]));
    }

    /// USD-M routes trades and books to `/public`, and tickers and candles to `/market`.
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

    /// Every USD-M feed uses an explicit `/public` or `/market` entry point.
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
        // `aggTrade` has a different id space and is not the common trade feed.
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
            Err(Error::InvalidRequest { field, .. }) if field == "feeds"
        ));
        assert!(matches!(
            stream_groups(&spot(), &no_market),
            Err(Error::InvalidRequest { field, .. }) if field == "markets"
        ));
    }

    #[test]
    fn a_trade_frame_becomes_a_trade_on_the_market_its_stream_names() {
        let Some(MarketEvent::Trade(trade)) = decode_spot(SPOT_TRADE_FRAME).expect("a frame")
        else {
            panic!("expected a trade event");
        };

        assert_eq!(trade.market, Market::spot(Exchange::Binance, "BNB", "BTC"));
        assert_eq!(trade.taker_side, Side::Sell);
        assert_eq!(trade.id.as_deref(), Some("12345"));
        assert_eq!(trade.price.to_string(), "0.001");
    }

    #[test]
    fn an_ambiguous_stream_symbol_uses_the_market_that_was_subscribed() {
        for (base, quote, symbol) in [("ADA", "EUR", "adaeur"), ("USDT", "USD", "usdtusd")] {
            let market = Market::spot(Exchange::Binance, base, quote);
            let frame = SPOT_TRADE_FRAME.replace("bnbbtc", symbol);
            let Some(MarketEvent::Trade(trade)) =
                decode_for(&spot(), market.clone(), &frame).expect("a trade frame")
            else {
                panic!("expected a trade event");
            };

            assert_eq!(trade.market, market);
        }
    }

    #[test]
    fn a_new_quote_asset_stream_uses_the_market_that_was_subscribed() {
        let frame = FUTURES_TRADE_FRAME.replace("dogeusdt@trade", "btcu@trade");
        let Some(MarketEvent::Trade(trade)) = decode_for(
            &perp(),
            Market::perpetual(Exchange::Binance, "BTC", "U"),
            &frame,
        )
        .expect("a trade frame") else {
            panic!("expected a trade event");
        };

        assert_eq!(
            trade.market,
            Market::perpetual(Exchange::Binance, "BTC", "U")
        );
    }

    #[test]
    fn a_non_ascii_stream_frame_uses_the_market_that_was_subscribed() {
        let market = Market::spot(Exchange::Binance, "币安人生", "USDT");
        let frame = SPOT_TRADE_FRAME.replace("bnbbtc", "币安人生usdt");
        let Some(MarketEvent::Trade(trade)) =
            decode_for(&spot(), market.clone(), &frame).expect("a trade frame")
        else {
            panic!("expected a trade event");
        };

        assert_eq!(trade.market, market);
    }

    #[test]
    fn a_futures_trade_lands_on_a_perpetual_market_under_its_rest_id() {
        let Some(MarketEvent::Trade(trade)) = decode_perp(FUTURES_TRADE_FRAME).expect("a frame")
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
            decode_for(
                &perp(),
                Market::perpetual(Exchange::Binance, "ETH", "USDT"),
                FUTURES_SPENT_TRADE_ID_FRAME,
            )
            .expect("a frame"),
            None
        );
    }

    #[test]
    fn a_streamed_candle_says_for_itself_whether_it_has_closed() {
        let Some(MarketEvent::Candle(candle)) = decode_spot(SPOT_KLINE_FRAME).expect("a frame")
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
        let Some(MarketEvent::OrderBook(book)) = decode_spot(SPOT_DEPTH_FRAME).expect("a frame")
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
            decode_spot(r#"{"result":null,"id":1}"#)
                .expect("a control frame")
                .is_none()
        );
    }

    #[test]
    fn an_error_frame_carries_binances_own_code_and_message() {
        let error = decode_spot(
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
            decode_spot(r#"{"data":{"e":"trade"}}"#),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(
            decode_spot(r#"{"stream":"bnbbtc","data":{}}"#),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(
            decode_spot(r#"{"stream":"bnbbtc@somethingNew","data":{}}"#),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(decode_spot("not json"), Err(Error::Decode { .. })));
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

    /// Spot account credentials are carried in the first frame, not the URL.
    #[test]
    fn a_spot_account_socket_requires_credentials_before_connecting() {
        let Err(error) = spot_account_connect(&spot()) else {
            panic!("an account socket without credentials was prepared");
        };

        assert!(matches!(error, Error::Auth { .. }));
    }

    /// Spot account credentials are carried in the first frame, not the URL.
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

    /// Each Spot account handshake signs a frame with a fresh timestamp.
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

    /// A USD-M listen-key expiry event; its unused `E` field may be a string.
    const FUTURES_LISTEN_KEY_EXPIRED: &str = r#"{"e": "listenKeyExpired","E": "1785449618659","listenKey": "0000000000000000000000000000000000000000000000000000000000000000"}"#;

    #[test]
    fn an_expired_listen_key_ends_the_stream_rather_than_going_quiet() {
        let error =
            decode_account(&perp(), FUTURES_LISTEN_KEY_EXPIRED).expect_err("an expired listen key");

        // The event name is the code, so the two ways Binance ends a
        // subscription stay apart.
        assert!(
            matches!(&error, Error::Exchange { code, .. } if code == "listenKeyExpired"),
            "{error:?}"
        );
    }

    /// The USD-M event filter and decoder must cover the same event set.
    #[test]
    fn the_usd_m_filter_and_the_decoder_name_the_same_events() {
        /// Acted on, and not asked for, because USD-M cannot send it.
        const SPOT_ONLY: &str = "eventStreamTerminated";

        // (event, one frame carrying it, whether `maxt` acts on it)
        let events = [
            ("ORDER_TRADE_UPDATE", FUTURES_ORDER_UPDATE, true),
            ("ACCOUNT_UPDATE", FUTURES_ACCOUNT_UPDATE, true),
            ("listenKeyExpired", FUTURES_LISTEN_KEY_EXPIRED, true),
            (SPOT_ONLY, r#"{"e":"eventStreamTerminated"}"#, true),
            ("TRADE_LITE", r#"{"e":"TRADE_LITE"}"#, false),
            ("MARGIN_CALL", r#"{"e":"MARGIN_CALL"}"#, false),
            (
                "ACCOUNT_CONFIG_UPDATE",
                r#"{"e":"ACCOUNT_CONFIG_UPDATE"}"#,
                false,
            ),
            (
                "CONDITIONAL_ORDER_TRIGGER_REJECT",
                r#"{"e":"CONDITIONAL_ORDER_TRIGGER_REJECT"}"#,
                false,
            ),
            ("STRATEGY_UPDATE", r#"{"e":"STRATEGY_UPDATE"}"#, false),
            ("GRID_UPDATE", r#"{"e":"GRID_UPDATE"}"#, false),
            ("ALGO_UPDATE", r#"{"e":"ALGO_UPDATE"}"#, false),
        ];

        for (name, frame, acts) in events {
            // Acted on means read into an event or raised as an error. A frame
            // `maxt` drops has nothing to gain from being subscribed to.
            let acted_on = match decode_account(&perp(), frame) {
                Ok(events) => !events.is_empty(),
                Err(_) => true,
            };
            assert_eq!(acted_on, acts, "what the decoder does with {name} moved");

            assert_eq!(
                private::USD_M_ACCOUNT_EVENTS.split('/').any(|e| e == name),
                acts && name != SPOT_ONLY,
                "the subscription and the decoder disagree about {name}: {}",
                private::USD_M_ACCOUNT_EVENTS
            );
        }

        for name in private::USD_M_ACCOUNT_EVENTS.split('/') {
            assert!(
                events.iter().any(|(event, ..)| *event == name),
                "the subscription asks for {name}, which this table does not \
                 weigh against the decoder"
            );
        }
    }

    /// A Spot termination event ends the account stream.
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

    /// A refused Spot subscription keeps Binance's status, code, and message.
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
            // The WebSocket frame carries the HTTP-style status.
            assert_eq!(*status, Some(400), "{error:?}");
            assert_eq!(*kind, crate::ExchangeErrorKind::Rejected, "{error:?}");
            // The message distinguishes signature and clock failures.
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
