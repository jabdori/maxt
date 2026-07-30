//! Hyperliquid's REST API, which is two endpoints.
//!
//! Everything readable goes to `POST /info` and everything that changes state
//! goes to `POST /exchange`. Neither takes a path or a query string: the request
//! *is* the JSON body, and its `type` field is what a URL would be anywhere
//! else.
//!
//! Request building is kept as plain functions returning [`HttpRequest`] so that
//! every body and rejection below is testable without a network.

use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::adapters::candles as candle_pages;
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{CandleRequest, HistoryRequest, MarginRequest, OrderRequest};
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{
    Balance, Candle, Cursor, FundingPayment, FundingRate, Interval, MarginMode, MarginSummary,
    Market, MarketInfo, MarketKind, Order, OrderBook, OrderStatus, OrderType, Page, Position, Size,
    TimeInForce, Timestamp, Trade,
};

use super::HyperliquidNetwork;
use super::native;
use super::parse::{self, Asset, EXCHANGE, Universe};
use super::sign::{
    self, CancelAction, LeverageAction, LimitKind, OrderAction, OrderKind, OrderWire,
};

pub(crate) const INFO_PATH: &str = "/info";
pub(crate) const EXCHANGE_PATH: &str = "/exchange";

/// Hyperliquid returns at most this many book levels per side.
pub(crate) const MAX_BOOK_DEPTH: u32 = 20;

/// Hyperliquid returns at most this many candles per `candleSnapshot`.
pub(crate) const MAX_CANDLE_COUNT: u32 = 5_000;

/// Hyperliquid returns at most this many trades per `recentTrades`.
///
/// Not a parameter. `recentTrades` takes the market and nothing else, so this is
/// the whole window the endpoint offers and a larger `limit` cannot be served by
/// asking differently.
pub(crate) const MAX_TRADE_COUNT: u32 = 10;

/// Hyperliquid returns at most this many entries per time-ranged history call,
/// which is how a page knows another one may follow.
pub(crate) const MAX_HISTORY_PAGE: usize = 500;

/// A non-integer price carries at most this many significant figures.
const MAX_PRICE_SIGNIFICANT_FIGURES: usize = 5;

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

/// A read, addressed by the `type` inside its body instead of by a path.
pub(crate) fn info(body: Value) -> HttpRequest {
    HttpRequest::post(INFO_PATH).json_body(body.to_string())
}

pub(crate) fn meta_request() -> HttpRequest {
    info(json!({ "type": "meta" }))
}

pub(crate) fn spot_meta_request() -> HttpRequest {
    info(json!({ "type": "spotMeta" }))
}

/// The universe plus a live context per asset, which is as close as Hyperliquid
/// comes to a ticker endpoint.
pub(crate) fn asset_contexts_request(kind: MarketKind) -> HttpRequest {
    info(json!({
        "type": match kind {
            MarketKind::Perpetual => "metaAndAssetCtxs",
            MarketKind::Spot => "spotMetaAndAssetCtxs",
        }
    }))
}

pub(crate) fn book_request(native: &str) -> HttpRequest {
    info(json!({ "type": "l2Book", "coin": native }))
}

/// The recent-trades read, which Hyperliquid's info reference does not list but
/// its rate-limit page names.
///
/// The body carries the market and nothing else: `recentTrades` takes no count
/// and no time range. `limit` is therefore checked here and applied to the
/// response rather than sent, and one above [`MAX_TRADE_COUNT`] is refused before
/// a request is built, because no way of asking would serve it.
pub(crate) fn trades_request(native: &str, limit: Option<u32>) -> Result<HttpRequest> {
    if let Some(limit) = limit
        && !(1..=MAX_TRADE_COUNT).contains(&limit)
    {
        return Err(Error::invalid_request(
            "limit",
            format!("hyperliquid serves 1 to {MAX_TRADE_COUNT} trades per call, not {limit}"),
        ));
    }

    Ok(info(json!({ "type": "recentTrades", "coin": native })))
}

pub(crate) fn candles_request(
    native: &str,
    interval: &str,
    start_ms: i64,
    end_ms: Option<i64>,
) -> HttpRequest {
    let mut req = json!({
        "coin": native,
        "interval": interval,
        "startTime": start_ms,
    });
    if let Some(end_ms) = end_ms {
        req["endTime"] = json!(end_ms);
    }

    info(json!({ "type": "candleSnapshot", "req": req }))
}

pub(crate) fn spot_state_request(user: &str) -> HttpRequest {
    info(json!({ "type": "spotClearinghouseState", "user": user }))
}

pub(crate) fn perp_state_request(user: &str) -> HttpRequest {
    info(json!({ "type": "clearinghouseState", "user": user }))
}

pub(crate) fn open_orders_request(user: &str) -> HttpRequest {
    // The `frontend` variant is the one carrying the limit price and the
    // original size, which an `Order` needs to report progress.
    info(json!({ "type": "frontendOpenOrders", "user": user }))
}

pub(crate) fn funding_history_request(
    native: &str,
    start_ms: i64,
    end_ms: Option<i64>,
) -> HttpRequest {
    let mut body = json!({
        "type": "fundingHistory",
        "coin": native,
        "startTime": start_ms,
    });
    if let Some(end_ms) = end_ms {
        body["endTime"] = json!(end_ms);
    }

    info(body)
}

pub(crate) fn user_funding_request(user: &str, start_ms: i64, end_ms: Option<i64>) -> HttpRequest {
    time_ranged("userFunding", user, start_ms, end_ms)
}

pub(crate) fn ledger_request(user: &str, start_ms: i64, end_ms: Option<i64>) -> HttpRequest {
    time_ranged("userNonFundingLedgerUpdates", user, start_ms, end_ms)
}

fn time_ranged(request_type: &str, user: &str, start_ms: i64, end_ms: Option<i64>) -> HttpRequest {
    let mut body = json!({
        "type": request_type,
        "user": user,
        "startTime": start_ms,
    });
    if let Some(end_ms) = end_ms {
        body["endTime"] = json!(end_ms);
    }

    info(body)
}

/// Sends a request and hands back a 2xx body.
///
/// A 2xx is only half the answer here, because `/exchange` reports a rejected
/// action inside a 200. Callers that post an action must also read the envelope
/// with [`parse::action_response`].
pub(crate) async fn post(http: &HttpTransport, request: &HttpRequest) -> Result<String> {
    let response = http.send(request).await?;

    if response.is_success() {
        Ok(response.body)
    } else {
        Err(parse::http_error(response.status, &response.body))
    }
}

// ---------------------------------------------------------------------------
// Public reads
// ---------------------------------------------------------------------------

/// Loads both universes and pairs them into one symbol table.
pub(crate) async fn universe(http: &HttpTransport) -> Result<Universe> {
    let perp = post(http, &meta_request()).await?;
    let spot = post(http, &spot_meta_request()).await?;

    Universe::new(&parse::json(&perp)?, &parse::json(&spot)?)
}

pub(crate) fn markets(universe: &Universe, kind: MarketKind) -> Vec<MarketInfo> {
    universe.of_kind(kind).map(parse::market_info).collect()
}

pub(crate) async fn order_book(
    http: &HttpTransport,
    universe: &Universe,
    market: &Market,
    depth: Option<u32>,
) -> Result<OrderBook> {
    if let Some(depth) = depth
        && !(1..=MAX_BOOK_DEPTH).contains(&depth)
    {
        return Err(Error::invalid_request(
            "depth",
            format!("hyperliquid serves 1 to {MAX_BOOK_DEPTH} book levels per side, not {depth}"),
        ));
    }

    let native = universe.native_symbol(market)?.to_string();
    let body = post(http, &book_request(&native)).await?;
    let mut book = parse::order_book(&parse::json(&body)?, universe)?;

    // `l2Book` takes no depth parameter, so the trimming happens here.
    if let Some(depth) = depth {
        let depth = depth as usize;
        book.bids.truncate(depth);
        book.asks.truncate(depth);
    }
    Ok(book)
}

/// Reads a market's recent trades, newest first.
///
/// The same `coin` name the `trades` subscription uses, which is what makes one
/// [`parse::trade`] serve both paths: the perpetual coin name, or a spot pair's
/// `@107` index form or its legacy slash form. A never-traded market answers with
/// an empty list rather than an error.
///
/// `limit` trims the page, and [`trades_request`] refuses one the endpoint's
/// fixed window cannot reach.
pub(crate) async fn trades(
    http: &HttpTransport,
    universe: &Universe,
    market: &Market,
    limit: Option<u32>,
) -> Result<Vec<Trade>> {
    let native = universe.native_symbol(market)?.to_string();
    let body = post(http, &trades_request(&native, limit)?).await?;

    newest_first(&parse::json::<Vec<_>>(&body)?, universe, limit)
}

/// Puts a `recentTrades` payload in the order the common API promises and cuts it
/// to `limit`.
///
/// Hyperliquid already answers newest-first, and
/// [`Client::trades`](crate::Client::trades) promises it, so it is enforced here
/// instead of trusted. The sort is stable, which leaves trades sharing a
/// millisecond in the order Hyperliquid listed them; several routinely do,
/// because one aggressive order fills against several resting ones at once.
fn newest_first(
    raw: &[parse::RawTrade],
    universe: &Universe,
    limit: Option<u32>,
) -> Result<Vec<Trade>> {
    let mut trades = raw
        .iter()
        .map(|raw| parse::trade(raw, universe))
        .collect::<Result<Vec<_>>>()?;

    trades.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
    if let Some(limit) = limit {
        trades.truncate(limit as usize);
    }

    Ok(trades)
}

/// Reads a market's rolling summary out of its asset context.
///
/// `at` becomes the summary's timestamp. See [`parse::ticker`] for why the
/// exchange cannot supply one.
pub(crate) async fn ticker(
    http: &HttpTransport,
    universe: &Universe,
    market: &Market,
    at: Timestamp,
) -> Result<crate::types::Ticker> {
    parse::ticker(&context(http, universe, market).await?, market, at)
}

/// Fetches one market's live context.
pub(crate) async fn context(
    http: &HttpTransport,
    universe: &Universe,
    market: &Market,
) -> Result<parse::RawAssetCtx> {
    let native = universe.native_symbol(market)?.to_string();
    let body = post(http, &asset_contexts_request(market.kind)).await?;

    pick_context(&body, market.kind, &native)
}

/// Picks the context belonging to `native` out of a `[meta, contexts]` response.
///
/// The two endpoints identify a context differently, so this does too. Both
/// were read live on 2026-07-30:
///
/// | Endpoint | Universe | Contexts | What names a context |
/// | --- | --- | --- | --- |
/// | `spotMetaAndAssetCtxs` | 319 | 710 | every context carries `coin` |
/// | `metaAndAssetCtxs` | 232 | 232 | nothing; position is all there is |
///
/// Spot therefore matches on `coin` and never on position. The two arrays are
/// not the same length, and 248 of the 319 spot markets sit at a position
/// holding another market's prices: `@107`, which is `HYPE/USDC`, is at
/// universe index 105, and the context at index 105 is `@105`, quoted 450 times
/// lower.
///
/// Perpetuals have no name to match, so position is the only pairing available.
/// That it is the right one was checked against `allMids`: all 232 positional
/// pairs agreed to within 0.11 percent, which is the drift between two separate
/// calls, while shifting the pairing by one put them 12 million percent apart.
/// The equal lengths are the only evidence that alignment still holds, so a
/// response where they differ is refused rather than paired.
fn pick_context(body: &str, kind: MarketKind, native: &str) -> Result<parse::RawAssetCtx> {
    match kind {
        MarketKind::Spot => {
            let (_, contexts): (serde_json::Value, Vec<parse::RawAssetCtx>) = parse::json(body)?;

            contexts
                .into_iter()
                .find(|context| context.coin.as_deref() == Some(native))
                .ok_or_else(|| {
                    Error::decode(format!(
                        "hyperliquid sent no asset context naming `{native}`"
                    ))
                })
        }
        MarketKind::Perpetual => {
            let (meta, mut contexts): (parse::RawPerpMeta, Vec<parse::RawAssetCtx>) =
                parse::json(body)?;
            if meta.universe.len() != contexts.len() {
                return Err(Error::decode(format!(
                    "hyperliquid sent {} perpetual asset contexts for {} assets, so they no \
                     longer pair by position, and a perpetual context names no market",
                    contexts.len(),
                    meta.universe.len()
                )));
            }

            let index = meta
                .universe
                .iter()
                .position(|asset| asset.name == native)
                .ok_or_else(|| {
                    Error::decode(format!("hyperliquid's universe no longer lists `{native}`"))
                })?;
            Ok(contexts.swap_remove(index))
        }
    }
}

/// Reads candles, oldest first, paging when one snapshot cannot hold the
/// answer.
///
/// `candleSnapshot` takes a time range instead of a count, and caps what it
/// returns at [`MAX_CANDLE_COUNT`]. A page of "the newest `count` before this
/// instant" is therefore a window that many intervals wide, which is what
/// [`crate::adapters::candles::read`] asks for on every exchange.
///
/// The interval is looked up inside the page fetch, not before it, so a request
/// that is wrong in more than one way is answered the same way here as on the
/// other three exchanges: the checks
/// [`candle_pages::read`](crate::adapters::candles::read) makes on `limit` and
/// on the window come first, and an interval Hyperliquid does not publish is
/// reported after them.
pub(crate) async fn candles(
    http: &HttpTransport,
    universe: &Universe,
    request: &CandleRequest,
    now: Timestamp,
) -> Result<Vec<Candle>> {
    let native = universe.native_symbol(&request.market)?.to_string();

    candle_pages::read(request, EXCHANGE, MAX_CANDLE_COUNT, |cursor, count| {
        let native = native.clone();
        async move {
            let Some(interval) = parse::interval_name(request.interval) else {
                return Err(parse::unsupported_interval(
                    request.interval,
                    Feature::Candles,
                ));
            };
            let end_ms = cursor.unwrap_or(now).as_millis();
            let start_ms = candle_start_ms(request, end_ms, count);
            let body = post(
                http,
                &candles_request(
                    &native,
                    interval,
                    start_ms,
                    Some(candle_query_end_ms(request.interval, end_ms)),
                ),
            )
            .await?;

            let mut page = parse::json::<Vec<parse::RawCandle>>(&body)?
                .iter()
                .map(|raw| parse::candle(raw, universe, now))
                .collect::<Result<Vec<_>>>()?;
            // Back to the window that was asked for. `candle_query_end_ms`
            // reaches one interval past it, and on the buckets that answer
            // `endTime` by their open rather than by their close that brings
            // back one too many.
            page.retain(|candle| candle.open_time.as_millis() <= end_ms);

            Ok(page)
        }
    })
    .await
}

/// How long one Hyperliquid `1M` candle is.
///
/// `candleSnapshot` at interval `1M` answers on a fixed 30-day grid measured
/// from the Unix epoch, not on calendar months. Read live on 2026-07-30 for
/// BTC, ETH and SOL: every bucket of all three spans exactly 30 days and opens
/// at a whole multiple of 30 days after the epoch, so the three share the same
/// boundaries. Recent opens are 2026-05-07, 2026-06-06 and 2026-07-06, all at
/// 00:00 UTC. There is no June bucket, and none closes on 1 July.
const MONTH_MS: i64 = 30 * 24 * 60 * 60 * 1_000;

/// Where a bucket boundary `count` steps from `at_ms` falls, on Hyperliquid's
/// own grid.
///
/// The single answer to "how long is one interval here", used by both ends of
/// every `candleSnapshot` window. It is
/// [`Interval::advance`](crate::Interval::advance) at every interval whose
/// length Hyperliquid and `maxt` agree on.
/// [`Interval::Month1`](crate::Interval::Month1) is not one of them: `advance`
/// steps calendar months and Hyperliquid's `1M` is [`MONTH_MS`], so a monthly
/// window is measured in the buckets the exchange will actually send. Stepping
/// calendar months against a 30-day grid over-fetches across a 31-day month and
/// under-fetches across February, and a page short of what it asked for is what
/// [`candle_pages`](crate::adapters::candles) reads as the end of a market's
/// history.
///
/// `None` past the range a [`Timestamp`] can name. Each caller says what it
/// wants done about that, because the two ends want opposite things.
fn candle_step_ms(interval: Interval, at_ms: i64, count: i64) -> Option<i64> {
    match interval {
        Interval::Month1 => MONTH_MS.checked_mul(count)?.checked_add(at_ms),
        interval => interval
            .advance(Timestamp::from_millis(at_ms), count)
            .map(Timestamp::as_millis),
    }
}

/// Works out what `endTime` one `candleSnapshot` should carry.
///
/// [`candle_pages`](crate::adapters::candles) asks for the candles opening at or
/// before `end_ms`, and `endTime` does not mean that over the whole of
/// Hyperliquid's history. Read live on 2026-07-30, holding `startTime` well
/// before each bucket and moving `endTime`:
///
/// | Interval | Bucket opening at | Arrives at `endTime` = its open | Arrives at `endTime` = its close |
/// | --- | --- | --- | --- |
/// | `1M` | 2020-07-07 | no | yes |
/// | `1M` | 2022-12-24 | no | yes |
/// | `1M` | 2023-05-23 | yes | yes |
/// | `1M` | 2024-01-18 | yes | yes |
/// | `1w` | 2020-02-06 | no | yes |
/// | `1d` | 2023-01-19 | no | yes |
/// | `1d` | 2023-06-20 | yes | yes |
/// | `1m` | within the last hour | yes | yes |
///
/// So a bucket from before roughly mid-2023 arrives only once `endTime` reaches
/// its close, and a newer one as soon as `endTime` reaches its open. Asking one
/// interval further on covers both eras at every interval, and the only thing it
/// can add is a bucket opening inside that extra interval, which the caller
/// drops.
///
/// `end_ms` unchanged when a step past it leaves the range a [`Timestamp`] can
/// name: a window ending there needs no room past its end.
fn candle_query_end_ms(interval: Interval, end_ms: i64) -> i64 {
    candle_step_ms(interval, end_ms, 1).unwrap_or(end_ms)
}

/// Works out where one `candleSnapshot` should start.
///
/// Hyperliquid has no count parameter, so `count` candles is expressed as a
/// window `count` intervals wide ending at `end_ms`, measured by
/// [`candle_step_ms`].
///
/// A window reaching further back than a [`Timestamp`] can name starts at the
/// epoch instead, which Hyperliquid reads as the beginning of history. That is
/// the plain default page of monthly candles: `limit` unset asks for
/// [`MAX_CANDLE_COUNT`] of them, and 5000 buckets before now is the year 1615.
/// Refusing it named `to`, a field such a caller never set.
///
/// Hyperliquid reads both ends of the window inclusively, so the snapshot
/// carries the candle at `end_ms` as well and a full page is `count + 1` long.
/// Read live: a `1m` window one interval wide returns two candles and one five
/// wide returns six.
fn candle_start_ms(request: &CandleRequest, end_ms: i64, count: u32) -> i64 {
    // Hyperliquid reads a start at or below zero as no start at all.
    candle_step_ms(request.interval, end_ms, -i64::from(count))
        .unwrap_or(0)
        .max(0)
}

// ---------------------------------------------------------------------------
// Private reads
// ---------------------------------------------------------------------------

pub(crate) async fn balances(http: &HttpTransport, user: &str) -> Result<Vec<Balance>> {
    let body = post(http, &spot_state_request(user)).await?;

    parse::json::<parse::RawSpotState>(&body)?
        .balances
        .iter()
        .map(parse::balance)
        .collect()
}

pub(crate) async fn open_orders(
    http: &HttpTransport,
    universe: &Universe,
    user: &str,
    market: Option<&Market>,
) -> Result<Vec<Order>> {
    // Narrowing happens here because `frontendOpenOrders` answers for the whole
    // account and takes no market.
    let wanted = market.map(|market| universe.asset(market)).transpose()?;
    let body = post(http, &open_orders_request(user)).await?;

    parse::json::<Vec<parse::RawOpenOrder>>(&body)?
        .iter()
        .map(|raw| parse::open_order(raw, universe))
        .filter(|order| match (order, wanted) {
            (Ok(order), Some(wanted)) => order.market == wanted.market,
            _ => true,
        })
        .collect()
}

pub(crate) async fn positions(
    http: &HttpTransport,
    universe: &Universe,
    user: &str,
    market: Option<&Market>,
) -> Result<Vec<Position>> {
    let wanted = market.map(|market| universe.asset(market)).transpose()?;
    let body = post(http, &perp_state_request(user)).await?;

    parse::json::<parse::RawPerpState>(&body)?
        .asset_positions
        .iter()
        .map(|raw| parse::position(&raw.position, universe))
        .filter(|position| match (position, wanted) {
            (Ok(position), Some(wanted)) => position.market == wanted.market,
            _ => true,
        })
        .collect()
}

pub(crate) async fn margin_summary(http: &HttpTransport, user: &str) -> Result<MarginSummary> {
    let body = post(http, &perp_state_request(user)).await?;

    parse::margin_summary(&parse::json(&body)?)
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

pub(crate) async fn funding_rates(
    http: &HttpTransport,
    universe: &Universe,
    request: &HistoryRequest,
) -> Result<Page<FundingRate>> {
    let asset = perpetual_asset(universe, &request.market, Feature::FundingRates)?;
    let (start_ms, end_ms) = history_window(request)?;
    let body = post(
        http,
        &funding_history_request(&asset.native, start_ms, end_ms),
    )
    .await?;

    let raw: Vec<parse::RawFundingHistory> = parse::json(&body)?;
    let items = raw
        .iter()
        .map(|entry| {
            Ok((
                FundingRate {
                    market: asset.market.clone(),
                    timestamp: parse::millis(entry.time, "time")?,
                    rate: parse::decimal(&entry.funding_rate, "fundingRate")?,
                    // `premium` is the gap between mark and oracle, not a price,
                    // so there is no mark price to report here.
                    mark_price: None,
                },
                entry.time,
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    page(
        items,
        newest(raw.iter().map(|entry| entry.time)),
        request.limit,
    )
}

pub(crate) async fn funding_payments(
    http: &HttpTransport,
    universe: &Universe,
    user: &str,
    request: &HistoryRequest,
) -> Result<Page<FundingPayment>> {
    let asset = perpetual_asset(universe, &request.market, Feature::FundingPayments)?;
    let (start_ms, end_ms) = history_window(request)?;
    let body = post(http, &user_funding_request(user, start_ms, end_ms)).await?;

    // `userFunding` answers for the whole account, so the market narrowing is
    // ours to do.
    let raw: Vec<parse::RawUserFunding> = parse::json(&body)?;
    let items = raw
        .iter()
        .filter(|entry| entry.delta.coin == asset.native)
        .map(|entry| {
            Ok((
                FundingPayment {
                    market: asset.market.clone(),
                    timestamp: parse::millis(entry.time, "time")?,
                    amount: parse::decimal(&entry.delta.usdc, "usdc")?,
                    rate: Some(parse::decimal(&entry.delta.funding_rate, "fundingRate")?),
                    id: Some(entry.hash.clone()),
                },
                entry.time,
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    // The unfiltered page decides whether another one follows: a page made
    // entirely of some other market's funding is still a full page.
    page(
        items,
        newest(raw.iter().map(|entry| entry.time)),
        request.limit,
    )
}

/// Reads a page of the account's non-funding ledger.
///
/// Account-wide, not market-scoped, which is one reason it cannot be a
/// [`FundingPayment`]. See [`native::HyperliquidLedgerEntry`].
pub(crate) async fn ledger(
    http: &HttpTransport,
    user: &str,
    from: Option<Timestamp>,
    to: Option<Timestamp>,
    cursor: Option<&Cursor>,
    limit: Option<u32>,
) -> Result<Page<native::HyperliquidLedgerEntry>> {
    let start_ms = match cursor {
        Some(cursor) => parse::cursor_start_ms(cursor)?,
        None => from.map(Timestamp::as_millis).unwrap_or(0),
    };
    let body = post(
        http,
        &ledger_request(user, start_ms, to.map(Timestamp::as_millis)),
    )
    .await?;

    let raw: Vec<parse::RawLedgerUpdate> = parse::json(&body)?;
    let items = native::ledger_entries(&raw)?
        .into_iter()
        .zip(raw.iter().map(|entry| entry.time))
        .collect();

    page(items, newest(raw.iter().map(|entry| entry.time)), limit)
}

/// The newest entry time on a page, and whether the page came back full.
///
/// A full page is Hyperliquid's only signal that more history follows: these
/// endpoints report no total and no cursor of their own.
///
/// The largest time is taken, not the last one. Hyperliquid answers
/// oldest-first today, but a cursor built from a page that arrived in any other
/// order would move backwards, and a backwards cursor re-reads the same page
/// forever.
fn newest(times: impl Iterator<Item = i64>) -> (Option<i64>, bool) {
    let times: Vec<i64> = times.collect();

    (times.iter().max().copied(), times.len() >= MAX_HISTORY_PAGE)
}

/// Assembles a page and the cursor that continues it.
///
/// Each item carries the time of the entry it came from, so that trimming to a
/// limit moves the cursor back to the last item the caller actually saw. Every
/// item is paired, not counted, because a filtered page has fewer items than
/// the page it was read from. Funding payments for one market come out of an
/// account-wide answer.
fn page<T>(
    mut items: Vec<(T, i64)>,
    page_end: (Option<i64>, bool),
    limit: Option<u32>,
) -> Result<Page<T>> {
    let (page_newest, full) = page_end;
    let mut truncated = false;

    if let Some(limit) = limit
        && items.len() > limit as usize
    {
        items.truncate(millisecond_boundary(&items, limit as usize));
        truncated = true;
    }

    // A trimmed page resumes just past its own last item; an untrimmed full one
    // resumes past the raw page, which may end on another market's entry.
    let resume_from = match (truncated, full) {
        (true, _) => items.last().map(|(_, time)| *time),
        (false, true) => page_newest,
        (false, false) => None,
    };
    let next = resume_from.map(parse::time_cursor).transpose()?;

    Ok(Page {
        items: items.into_iter().map(|(item, _)| item).collect(),
        next,
    })
}

/// Where a page of `limit` entries may be cut without the cursor skipping any.
///
/// The next page resumes one millisecond past the last entry kept, so a cut
/// that lands inside a run of entries sharing one millisecond would strand the
/// rest of that run: the caller never saw them and the cursor has already moved
/// past them. Cutting back to the start of the straddling run avoids that and
/// stays under `limit`. When the run reaches the front of the page there is
/// nothing to cut back to, so the whole run is kept. That hands back a few more
/// entries than asked for, which a caller can drop, instead of fewer than
/// exist, which it cannot recover.
fn millisecond_boundary<T>(items: &[(T, i64)], limit: usize) -> usize {
    let Some(head) = items.get(..limit) else {
        return limit;
    };
    let Some(&(_, boundary)) = head.last() else {
        return limit;
    };
    if items.get(limit).is_none_or(|(_, time)| *time != boundary) {
        return limit;
    }

    match head.iter().rposition(|(_, time)| *time != boundary) {
        Some(previous) => previous + 1,
        None => items
            .iter()
            .take_while(|(_, time)| *time == boundary)
            .count(),
    }
}

/// Reads the start and end of a history window in milliseconds.
///
/// A cursor wins over `from`: it is where the previous page stopped, and
/// honouring `from` instead would fetch it a second time.
fn history_window(request: &HistoryRequest) -> Result<(i64, Option<i64>)> {
    let start = match &request.cursor {
        Some(cursor) => parse::cursor_start_ms(cursor)?,
        // Hyperliquid demands a start time. Zero is the earliest one it accepts
        // and means "as far back as you keep".
        None => request.from.map(Timestamp::as_millis).unwrap_or(0),
    };

    Ok((start, request.to.map(Timestamp::as_millis)))
}

fn perpetual_asset<'a>(
    universe: &'a Universe,
    market: &Market,
    feature: Feature,
) -> Result<&'a Asset> {
    let asset = universe.asset(market)?;

    if asset.market.kind != MarketKind::Perpetual {
        return Err(Error::unsupported(
            feature,
            EXCHANGE,
            format!("{market} is a spot market, and spot markets pay no funding"),
        ));
    }
    Ok(asset)
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/// Builds and signs an order, then reads the verdict out of the response.
pub(crate) async fn place_order(
    http: &HttpTransport,
    universe: &Universe,
    private_key: &str,
    network: HyperliquidNetwork,
    request: &OrderRequest,
    nonce: u64,
) -> Result<Order> {
    let asset = universe.asset(&request.market)?;
    let wire = order_wire(asset, request)?;
    let action = OrderAction::new(wire.clone());

    let body = sign::signed_body(&action, private_key, nonce, network)?;
    let response = post(http, &HttpRequest::post(EXCHANGE_PATH).json_body(body)).await?;
    let (id, status) = parse::order_ack_id(&parse::action_response(&response)?)?;

    let size = parse::decimal(&wire.s, "size")?;
    Ok(Order {
        id,
        market: request.market.clone(),
        side: request.side,
        status,
        // Hyperliquid's acknowledgement says whether the order rested or filled,
        // not how much of it filled, so a fill is reported as complete and a
        // rest as untouched.
        filled_quantity: if status == OrderStatus::Filled {
            size
        } else {
            Decimal::ZERO
        },
        remaining_quantity: if status == OrderStatus::Filled {
            Decimal::ZERO
        } else {
            size
        },
        price: request.price,
        // Hyperliquid's acknowledgement carries no time, and the nonce is this
        // process's clock rather than the exchange's.
        created_at: None,
    })
}

/// Turns an [`OrderRequest`] into the wire shape, rejecting anything Hyperliquid
/// cannot express.
pub(crate) fn order_wire(asset: &Asset, request: &OrderRequest) -> Result<OrderWire> {
    if request.order_type != OrderType::Limit {
        return Err(Error::unsupported(
            Feature::Trading,
            EXCHANGE,
            "hyperliquid has no market order type; send an immediate-or-cancel limit \
             order priced through the book instead",
        ));
    }
    let Size::Base(size) = request.size else {
        return Err(Error::invalid_request(
            "size",
            "hyperliquid sizes every order in the base asset; use `Size::Base`",
        ));
    };
    let price = request.price.ok_or_else(|| {
        Error::invalid_request("price", "a hyperliquid limit order needs a price")
    })?;
    if request.reduce_only && asset.market.kind != MarketKind::Perpetual {
        return Err(Error::unsupported(
            Feature::ReduceOnlyOrders,
            EXCHANGE,
            format!(
                "{} is a spot market, and has no position to reduce",
                asset.market
            ),
        ));
    }

    Ok(OrderWire {
        a: asset.asset_id,
        b: request.side == crate::types::Side::Buy,
        p: price_text(price, asset)?,
        s: size_text(size, asset)?,
        r: request.reduce_only,
        t: OrderKind {
            limit: LimitKind {
                tif: time_in_force(request.time_in_force)?,
            },
        },
    })
}

/// Hyperliquid's spelling of a time in force.
fn time_in_force(time_in_force: Option<TimeInForce>) -> Result<&'static str> {
    Ok(
        match time_in_force.unwrap_or(TimeInForce::GoodTilCancelled) {
            TimeInForce::GoodTilCancelled => "Gtc",
            TimeInForce::ImmediateOrCancel => "Ioc",
            // "Add liquidity only" is Hyperliquid's name for post-only.
            TimeInForce::PostOnly => "Alo",
            TimeInForce::FillOrKill => {
                return Err(Error::unsupported(
                    Feature::Trading,
                    EXCHANGE,
                    "hyperliquid offers good-til-cancelled, immediate-or-cancel, and \
                 post-only; it has no fill-or-kill",
                ));
            }
        },
    )
}

/// Formats a price, enforcing the two rules Hyperliquid applies to one.
///
/// Both are checked here, before anything is sent. A rejected order costs a
/// round trip and reads as a rate limit's worth of noise in a log.
fn price_text(price: Decimal, asset: &Asset) -> Result<String> {
    if price <= Decimal::ZERO {
        return Err(Error::invalid_request("price", "must be greater than zero"));
    }
    let text = wire_decimal(price);
    let fraction = text
        .split_once('.')
        .map(|(_, digits)| digits.len())
        .unwrap_or(0);

    if fraction as u32 > asset.price_decimals() {
        return Err(Error::invalid_request(
            "price",
            format!(
                "hyperliquid prices {} to {} decimal places, and {text} has {fraction}",
                asset.market,
                asset.price_decimals()
            ),
        ));
    }
    // Integers are exempt: Hyperliquid accepts any whole-number price.
    if fraction > 0 && significant_figures(&text) > MAX_PRICE_SIGNIFICANT_FIGURES {
        return Err(Error::invalid_request(
            "price",
            format!(
                "a hyperliquid price with a fractional part carries at most \
                 {MAX_PRICE_SIGNIFICANT_FIGURES} significant figures, and {text} has more"
            ),
        ));
    }
    Ok(text)
}

fn size_text(size: Decimal, asset: &Asset) -> Result<String> {
    if size <= Decimal::ZERO {
        return Err(Error::invalid_request("size", "must be greater than zero"));
    }
    let text = wire_decimal(size);
    let fraction = text
        .split_once('.')
        .map(|(_, digits)| digits.len())
        .unwrap_or(0);

    if fraction as u32 > asset.size_decimals {
        return Err(Error::invalid_request(
            "size",
            format!(
                "hyperliquid sizes {} to {} decimal places, and {text} has {fraction}",
                asset.market, asset.size_decimals
            ),
        ));
    }
    Ok(text)
}

/// Writes a decimal the way Hyperliquid wants it: plain digits, no exponent, no
/// trailing zeros.
///
/// Trailing zeros matter because the text is what gets hashed and signed, and
/// Hyperliquid compares it against its own canonical spelling: `1.20` and `1.2`
/// are the same number but two different signatures, and only one is accepted.
fn wire_decimal(value: Decimal) -> String {
    let text = value.to_string();

    match text.contains('.') {
        true => text.trim_end_matches('0').trim_end_matches('.').to_string(),
        false => text,
    }
}

fn significant_figures(text: &str) -> usize {
    let digits: String = text.chars().filter(char::is_ascii_digit).collect();
    let significant = digits.trim_start_matches('0');

    // Trailing zeros are already gone, so what is left is exactly the
    // significant part.
    significant.len().max(1)
}

pub(crate) async fn cancel_order(
    http: &HttpTransport,
    universe: &Universe,
    private_key: &str,
    network: HyperliquidNetwork,
    market: &Market,
    order_id: &str,
    nonce: u64,
) -> Result<Order> {
    let asset = universe.asset(market)?;
    let oid: u64 = order_id.parse().map_err(|_| {
        Error::invalid_request(
            "order_id",
            format!("a hyperliquid order id is a number, and `{order_id}` is not"),
        )
    })?;

    let action = CancelAction::new(asset.asset_id, oid);
    let body = sign::signed_body(&action, private_key, nonce, network)?;
    let response = post(http, &HttpRequest::post(EXCHANGE_PATH).json_body(body)).await?;
    let accepted = parse::action_response(&response)?;
    cancel_ack(&accepted)?;

    Ok(Order {
        id: order_id.to_string(),
        market: market.clone(),
        // Hyperliquid's cancel acknowledgement carries nothing but the verdict:
        // no side, no sizes, no price. Reading the order back is `open_orders`.
        side: crate::types::Side::Buy,
        status: OrderStatus::Cancelled,
        filled_quantity: Decimal::ZERO,
        remaining_quantity: Decimal::ZERO,
        price: None,
        created_at: None,
    })
}

/// Reads the per-cancel verdict, which is a second rejection point inside an
/// envelope that already said `ok`.
fn cancel_ack(response: &Value) -> Result<()> {
    let status = response
        .get("data")
        .and_then(|data| data.get("statuses"))
        .and_then(Value::as_array)
        .and_then(|statuses| statuses.first())
        .ok_or_else(|| Error::decode("hyperliquid cancel response carries no `data.statuses`"))?;

    if let Some(message) = status.get("error").and_then(Value::as_str) {
        return Err(Error::exchange(EXCHANGE, "cancel_rejected", message));
    }
    if status.as_str() == Some("success") {
        return Ok(());
    }

    Err(Error::decode(format!(
        "unexpected hyperliquid cancel status `{status}`"
    )))
}

pub(crate) async fn set_margin(
    http: &HttpTransport,
    universe: &Universe,
    private_key: &str,
    network: HyperliquidNetwork,
    request: &MarginRequest,
    nonce: u64,
) -> Result<()> {
    let asset = perpetual_asset(universe, &request.market, Feature::MarginConfig)?;

    // `updateLeverage` sets both at once, so neither can be changed alone
    // without first reading back the other and risking a stale value.
    let (Some(leverage), Some(mode)) = (request.leverage, request.margin_mode) else {
        return Err(Error::invalid_request(
            "leverage",
            "hyperliquid sets leverage and margin mode in one action; set both",
        ));
    };
    if leverage.fract() != Decimal::ZERO || leverage <= Decimal::ZERO {
        return Err(Error::invalid_request(
            "leverage",
            format!("hyperliquid leverage is a whole number above zero, not {leverage}"),
        ));
    }
    let leverage = u32::try_from(leverage.trunc()).map_err(|_| {
        Error::invalid_request(
            "leverage",
            format!("{leverage} is beyond any exchange's cap"),
        )
    })?;
    if let Some(max) = asset.max_leverage
        && leverage > max
    {
        return Err(Error::invalid_request(
            "leverage",
            format!(
                "hyperliquid caps {} at {max}x, not {leverage}x",
                asset.market
            ),
        ));
    }
    let is_cross = mode == MarginMode::Cross;
    if is_cross && asset.only_isolated {
        return Err(Error::invalid_request(
            "margin_mode",
            format!(
                "hyperliquid backs {} with isolated margin only",
                asset.market
            ),
        ));
    }

    let action = LeverageAction::new(asset.asset_id, is_cross, leverage);
    let body = sign::signed_body(&action, private_key, nonce, network)?;
    let response = post(http, &HttpRequest::post(EXCHANGE_PATH).json_body(body)).await?;
    parse::action_response(&response)?;

    Ok(())
}

/// A nonce, which Hyperliquid requires to be a millisecond timestamp near its
/// own clock and never reused.
pub(crate) fn nonce(now: Timestamp) -> u64 {
    u64::try_from(now.as_millis()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::super::parse::tests::{btc_perp, universe};
    use super::*;
    use crate::types::{Exchange, Side};

    /// A `spotMetaAndAssetCtxs` response, cut down from the live one read on
    /// 2026-07-30 with every number and name kept as it came back.
    ///
    /// The misalignment is the live one, not an invented shape. `@107` is
    /// `HYPE/USDC`; in the real response it is at universe index 105 while the
    /// context at index 105 is `@105`, and the contexts array is 710 long
    /// against a 319-entry universe. Here `@107` sits at universe index 1 and
    /// `@105` at context index 1, which is the same defect at a size a test can
    /// read.
    const SPOT_ASSET_CTXS: &str = r#"[
      {
        "tokens": [
          {"name": "USDC", "szDecimals": 8, "weiDecimals": 8, "index": 0, "isCanonical": true},
          {"name": "PURR", "szDecimals": 0, "weiDecimals": 5, "index": 1, "isCanonical": true},
          {"name": "HYPE", "szDecimals": 2, "weiDecimals": 8, "index": 150, "isCanonical": true}
        ],
        "universe": [
          {"name": "PURR/USDC", "tokens": [1, 0], "index": 0, "isCanonical": true},
          {"name": "@107", "tokens": [150, 0], "index": 107, "isCanonical": false}
        ]
      },
      [
        {
          "prevDayPx": "0.063091",
          "dayNtlVlm": "380143.3363749998",
          "markPx": "0.061848",
          "midPx": "0.061726",
          "circulatingSupply": "595169798.8345600367",
          "coin": "PURR/USDC",
          "totalSupply": "595169805.3573399782",
          "dayBaseVlm": "6100365.0"
        },
        {
          "prevDayPx": "0.14169",
          "dayNtlVlm": "349.9133733",
          "markPx": "0.11898",
          "midPx": "0.118805",
          "circulatingSupply": "722204.69446393",
          "coin": "@105",
          "totalSupply": "1198193.1844639301",
          "dayBaseVlm": "2469.57"
        },
        {
          "prevDayPx": "55.42",
          "dayNtlVlm": "50995282.010359928",
          "markPx": "53.687",
          "midPx": "53.6865",
          "circulatingSupply": "298668840.8729972243",
          "coin": "@107",
          "totalSupply": "999071991.9264007807",
          "dayBaseVlm": "938763.9000000007"
        }
      ]
    ]"#;

    /// A `metaAndAssetCtxs` response, cut down from the live one read on
    /// 2026-07-30. The universe and the contexts are the same length there, and
    /// no context carries a name, so the two are kept the same length here.
    const PERP_ASSET_CTXS: &str = r#"[
      {
        "universe": [
          {"name": "BTC", "szDecimals": 5, "maxLeverage": 40, "marginTableId": 50},
          {"name": "ETH", "szDecimals": 4, "maxLeverage": 25, "marginTableId": 51}
        ]
      },
      [
        {
          "dayNtlVlm": "1169046.29406",
          "funding": "0.0000125",
          "markPx": "63970.6",
          "midPx": "63969.5",
          "openInterest": "688.11",
          "oraclePx": "63970.0",
          "prevDayPx": "63000.0",
          "dayBaseVlm": "81584.5"
        },
        {
          "dayNtlVlm": "22.0",
          "funding": "0.0000125",
          "markPx": "3000.0",
          "midPx": "3000.5",
          "openInterest": "1.0",
          "oraclePx": "3000.0",
          "prevDayPx": "2900.0",
          "dayBaseVlm": "1.0"
        }
      ]
    ]"#;

    fn hype_spot() -> Market {
        Market::spot(Exchange::Hyperliquid, "HYPE", "USDC")
    }

    fn body_of(request: &HttpRequest) -> Value {
        serde_json::from_str(request.body.as_deref().expect("a JSON body")).expect("valid JSON")
    }

    #[test]
    fn every_read_is_a_post_to_one_path_addressed_by_its_type() {
        for (request, expected) in [
            (meta_request(), "meta"),
            (spot_meta_request(), "spotMeta"),
            (
                asset_contexts_request(MarketKind::Perpetual),
                "metaAndAssetCtxs",
            ),
            (
                asset_contexts_request(MarketKind::Spot),
                "spotMetaAndAssetCtxs",
            ),
            (book_request("BTC"), "l2Book"),
            (
                trades_request("BTC", None).expect("no limit is always servable"),
                "recentTrades",
            ),
            (spot_state_request("0xabc"), "spotClearinghouseState"),
            (perp_state_request("0xabc"), "clearinghouseState"),
            (open_orders_request("0xabc"), "frontendOpenOrders"),
        ] {
            assert_eq!(request.target(), INFO_PATH, "{expected}");
            assert_eq!(body_of(&request)["type"], expected);
        }
    }

    #[test]
    fn a_spot_context_is_the_one_naming_the_market_not_the_one_beside_it() {
        let hype = pick_context(SPOT_ASSET_CTXS, MarketKind::Spot, "@107").expect("a context");
        let purr = pick_context(SPOT_ASSET_CTXS, MarketKind::Spot, "PURR/USDC").expect("a context");

        // Pairing by universe position hands back `@105` here, which quotes
        // 0.118805 against HYPE/USDC's real 53.6865: a different market's price,
        // silently, 450 times out.
        assert_eq!(hype.coin.as_deref(), Some("@107"));
        assert_eq!(hype.mid_px.as_deref(), Some("53.6865"));
        // The pairs that predate the index scheme keep their slash name in both
        // the universe and the context, and resolve the same way.
        assert_eq!(purr.mid_px.as_deref(), Some("0.061726"));
    }

    #[test]
    fn a_spot_market_with_no_context_is_an_error_rather_than_a_neighbours_data() {
        let missing = pick_context(SPOT_ASSET_CTXS, MarketKind::Spot, "@999");

        assert!(matches!(missing, Err(Error::Decode { .. })), "{missing:?}");
    }

    #[test]
    fn a_perpetual_context_pairs_by_position_because_nothing_in_it_carries_a_name() {
        let btc = pick_context(PERP_ASSET_CTXS, MarketKind::Perpetual, "BTC").expect("a context");
        let eth = pick_context(PERP_ASSET_CTXS, MarketKind::Perpetual, "ETH").expect("a context");

        assert_eq!(btc.coin, None);
        assert_eq!(btc.mid_px.as_deref(), Some("63969.5"));
        assert_eq!(eth.mid_px.as_deref(), Some("3000.5"));
    }

    #[test]
    fn a_perpetual_response_whose_two_arrays_disagree_in_length_is_refused() {
        // The equal lengths are the only evidence the positional pairing still
        // holds, so losing them has to stop the read rather than shift it.
        let whole: Value = serde_json::from_str(PERP_ASSET_CTXS).expect("valid JSON");
        let one_short = json!([whole[0], [whole[1][0].clone()]]).to_string();

        let refused = pick_context(&one_short, MarketKind::Perpetual, "ETH");

        assert!(matches!(refused, Err(Error::Decode { .. })), "{refused:?}");
    }

    #[test]
    fn a_candle_request_names_the_window_hyperliquid_needs() {
        let body = body_of(&candles_request(
            "BTC",
            "15m",
            1_681_923_600_000,
            Some(1_681_924_500_000),
        ));

        assert_eq!(body["type"], "candleSnapshot");
        assert_eq!(body["req"]["coin"], "BTC");
        assert_eq!(body["req"]["interval"], "15m");
        assert_eq!(body["req"]["startTime"], 1_681_923_600_000_i64);
        assert_eq!(body["req"]["endTime"], 1_681_924_500_000_i64);
        // An open-ended window omits the end rather than inventing one.
        assert!(body_of(&candles_request("BTC", "1m", 0, None))["req"]["endTime"].is_null());
    }

    #[test]
    fn a_window_asks_one_interval_past_its_end_because_older_buckets_answer_by_their_close() {
        // Hyperliquid serves a bucket from before roughly mid-2023 only once
        // `endTime` has reached the bucket's close, and a newer one as soon as
        // `endTime` reaches its open. Read live on 2026-07-30 against
        // `candleSnapshot` for BTC, `startTime` 2020-01-01 throughout:
        //
        // | interval | endTime | newest bucket returned |
        // | --- | --- | --- |
        // | `1M` | 2020-07-07, the open of a bucket | 2020-06-07, the one before |
        // | `1M` | 2020-08-06, that bucket's close  | 2020-07-07 |
        // | `1w` | 2020-02-06, the open of a bucket | 2020-01-30, the one before |
        // | `1w` | 2020-02-13, that bucket's close  | 2020-02-06 |
        //
        // Asking one interval on turns both eras into the one thing
        // `candle_pages` asked for, the candles opening at or before the
        // cursor, and the page fetch cuts off whatever the extra interval
        // added.
        const JULY_7_2020: i64 = 1_594_080_000_000;
        const FEB_6_2020: i64 = 1_580_947_200_000;

        assert_eq!(
            candle_query_end_ms(Interval::Month1, JULY_7_2020),
            JULY_7_2020 + MONTH_MS,
            "2020-08-06, the close of the bucket opening at the cursor"
        );
        assert_eq!(
            candle_query_end_ms(Interval::Week1, FEB_6_2020),
            FEB_6_2020 + 7 * 86_400_000,
            "2020-02-13, the close of the week opening at the cursor"
        );
        // A calendar month is not what `1M` is, at either end of the window.
        assert_ne!(
            candle_query_end_ms(Interval::Month1, JULY_7_2020),
            Interval::Month1
                .advance(Timestamp::from_millis(JULY_7_2020), 1)
                .expect("a month on")
                .as_millis()
        );
        // A cursor at the end of what a `Timestamp` holds has no room past it,
        // and a window ending there does not need any.
        let end_of_time = Timestamp::from_nanos(i64::MAX).as_millis();
        assert_eq!(
            candle_query_end_ms(Interval::Min1, end_of_time),
            end_of_time
        );
    }

    #[test]
    fn a_count_becomes_a_window_that_many_intervals_wide() {
        const END: i64 = 1_700_000_000_000;
        let request = CandleRequest::new(btc_perp(), Interval::Min1);

        assert_eq!(candle_start_ms(&request, END, 100), END - 100 * 60 * 1_000);
        // Hyperliquid reads a negative start as no start at all, so a window
        // reaching past the epoch stops there.
        assert_eq!(candle_start_ms(&request, 60_000, 100), 0);
    }

    #[test]
    fn a_monthly_window_counts_back_hyperliquids_thirty_day_buckets() {
        const END: i64 = 1_700_000_000_000;
        let monthly = CandleRequest::new(btc_perp(), Interval::Month1);

        // Hyperliquid's `1M` sits on a 30-day grid measured from the epoch.
        // Read live for BTC, ETH and SOL on 2026-07-30: consecutive buckets
        // open 2026-05-07, 2026-06-06 and 2026-07-06 on all three, no June
        // bucket, nothing closing on 1 July, and every open a whole multiple of
        // 30 days after the epoch. A window measured in calendar months counts
        // something the exchange does not serve, and lands 92 days back here
        // where three of its buckets are 90.
        assert_eq!(candle_start_ms(&monthly, END, 3), END - 3 * MONTH_MS);
        assert_eq!((END - candle_start_ms(&monthly, END, 3)) / 86_400_000, 90);
        assert_eq!(1_783_296_000_000_i64 % MONTH_MS, 0, "2026-07-06T00:00:00Z");
        // The caller's own `from` is not consulted, so a monthly window is not
        // the whole span from `from` to `end` in one snapshot however wide the
        // caller's `limit` was.
        assert_eq!(
            candle_start_ms(
                &monthly.from(Timestamp::from_millis(1_600_000_000_000)),
                END,
                3
            ),
            END - 3 * MONTH_MS
        );
    }

    #[test]
    fn the_default_page_of_monthly_candles_reads_from_the_beginning_of_history() {
        // `CandleRequest::new(market, Interval::Month1)` with nothing else set
        // asks `candle_pages` for `MAX_CANDLE_COUNT` candles, and that many
        // months before now is the year 1615, which is not a `Timestamp`. It
        // used to be refused as an invalid `to`, a field this caller never set.
        // Hyperliquid reads a start of zero as the beginning of history, which
        // is the honest answer to "more candles than exist".
        const NOW: i64 = 1_785_000_000_000;
        let monthly = CandleRequest::new(btc_perp(), Interval::Month1);

        assert_eq!(candle_start_ms(&monthly, NOW, MAX_CANDLE_COUNT), 0);
        assert!(
            Interval::Month1
                .advance(Timestamp::from_millis(NOW), -i64::from(MAX_CANDLE_COUNT))
                .is_none()
        );
    }

    #[tokio::test]
    async fn limit_and_window_are_checked_before_the_interval_is_looked_up() {
        // The other three exchanges look the interval up inside the page fetch,
        // so `candle_pages` reports a `limit` of zero first. Hyperliquid used to
        // check the interval ahead of the walk and answer `Unsupported` to a
        // request that was also asking for no candles at all.
        let http = HttpTransport::new("http://127.0.0.1:1").expect("a transport");
        let request = CandleRequest::new(btc_perp(), Interval::Sec1).limit(0);

        // Nothing is fetched: `candle_pages` refuses before the first call, so
        // the unreachable host above is never dialled.
        let refused = candles(&http, &universe(), &request, Timestamp::default()).await;

        assert!(
            matches!(&refused, Err(Error::InvalidRequest { field, .. }) if *field == "limit"),
            "{refused:?}"
        );
    }

    #[test]
    fn every_interval_in_the_common_baseline_has_a_hyperliquid_spelling() {
        // What `supports(Feature::Candles) == true` is worth: the eight
        // intervals every `maxt` adapter serves.
        for interval in [
            Interval::Min1,
            Interval::Min5,
            Interval::Min15,
            Interval::Min30,
            Interval::Hour1,
            Interval::Hour4,
            Interval::Day1,
            Interval::Week1,
        ] {
            assert!(
                parse::interval_name(interval).is_some(),
                "{interval:?} is in the baseline and must not be refused"
            );
        }
    }

    #[test]
    fn one_second_candles_are_refused_rather_than_rounded_to_a_neighbour() {
        assert!(matches!(
            parse::unsupported_interval(Interval::Sec1, Feature::Candles),
            Error::Unsupported {
                feature: Feature::Candles,
                exchange: "hyperliquid",
                ..
            }
        ));
    }

    #[test]
    fn a_depth_beyond_what_hyperliquid_serves_is_refused_rather_than_clamped() {
        assert_eq!(MAX_BOOK_DEPTH, 20);
        // The check lives in `order_book`, which needs a transport; this proves
        // the boundary the check uses is the documented one.
        assert!(!(1..=MAX_BOOK_DEPTH).contains(&21));
        assert!(!(1..=MAX_BOOK_DEPTH).contains(&0));
    }

    #[test]
    fn a_trade_request_names_the_market_and_carries_no_count() {
        let body = body_of(&trades_request("@107", Some(5)).expect("five is inside the window"));

        assert_eq!(body["type"], "recentTrades");
        assert_eq!(body["coin"], "@107");
        // The count the caller asked for is not on the wire, because
        // `recentTrades` has nowhere to put it. `trades` trims the response.
        assert!(body.get("n").is_none() && body.get("limit").is_none());
    }

    /// Four consecutive trades as `recentTrades` sent them, newest first.
    ///
    /// The first two share a `hash` and a millisecond: one aggressive sell filled
    /// against two resting bids, and the fill hash names the order, not the
    /// trade. `tid` is the per-trade identifier, and it is what
    /// [`parse::trade`] reads.
    const RECENT_TRADES: &str = r#"[
      {"coin":"BTC","side":"A","px":"64307.0","sz":"0.06854","time":1785378501507,
       "hash":"0x0ba8656f473eef6f0d22044109ec2b0207ac0054e2320e41af7110c20632c959",
       "tid":732538781370075,"users":["0x4e60","0xd30a"]},
      {"coin":"BTC","side":"A","px":"64307.0","sz":"0.02739","time":1785378501507,
       "hash":"0x0ba8656f473eef6f0d22044109ec2b0207ac0054e2320e41af7110c20632c959",
       "tid":94854795233250,"users":["0xe57e","0xd30a"]},
      {"coin":"BTC","side":"A","px":"64307.0","sz":"0.00078","time":1785378501507,
       "hash":"0xab7225b999df4e83aceb044109ec2b020466009f34d26d554f3ad10c58d3286e",
       "tid":391208932656103,"users":["0xe57e","0xbe8d"]},
      {"coin":"BTC","side":"B","px":"64307.0","sz":"0.00622","time":1785378501087,
       "hash":"0xb1015560527b16f8b27b044109ec260207590045ed7e35ca54ca00b3117ef0e3",
       "tid":909117310751266,"users":["0xe57e","0xfb3f"]}
    ]"#;

    #[test]
    fn recent_trades_come_back_newest_first_with_one_id_per_trade() {
        let raw: Vec<parse::RawTrade> = parse::json(RECENT_TRADES).expect("a recentTrades payload");
        let trades = newest_first(&raw, &universe(), None).expect("four trades");

        assert_eq!(trades.len(), 4);
        assert_eq!(trades[0].market, btc_perp());
        assert!(
            trades
                .windows(2)
                .all(|pair| pair[0].timestamp >= pair[1].timestamp),
            "{trades:?}"
        );
        // A fill hash names the order that swept the book, so two of these four
        // carry the same one. Keying on it would drop a trade; `tid` does not.
        let ids: std::collections::BTreeSet<_> =
            trades.iter().filter_map(|trade| trade.id.clone()).collect();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains("732538781370075"));
    }

    #[test]
    fn a_trade_limit_cuts_the_page_hyperliquid_sent() {
        let raw: Vec<parse::RawTrade> = parse::json(RECENT_TRADES).expect("a recentTrades payload");
        let trades = newest_first(&raw, &universe(), Some(2)).expect("two trades");

        // Newest two, not the oldest two: the cut happens after the ordering.
        assert_eq!(trades.len(), 2);
        assert_eq!(
            trades[0].timestamp,
            crate::types::Timestamp::from_millis(1_785_378_501_507)
        );
    }

    #[test]
    fn a_trade_limit_past_the_recent_trades_window_is_refused_rather_than_under_served() {
        assert_eq!(MAX_TRADE_COUNT, 10);
        assert!(trades_request("BTC", Some(MAX_TRADE_COUNT)).is_ok());

        for limit in [0, MAX_TRADE_COUNT + 1, 500] {
            assert!(
                matches!(
                    trades_request("BTC", Some(limit)),
                    Err(Error::InvalidRequest { field: "limit", .. })
                ),
                "{limit} was accepted"
            );
        }
    }

    #[test]
    fn an_order_carries_the_asset_id_of_the_universe_it_belongs_to() {
        let universe = universe();
        let perp = order_wire(
            universe.asset(&btc_perp()).expect("listed"),
            &OrderRequest::limit(
                btc_perp(),
                Side::Buy,
                Size::Base(Decimal::new(12_345, 5)),
                Decimal::from(27_123),
            ),
        )
        .expect("a valid order");
        let spot = order_wire(
            universe.asset(&hype_spot()).expect("listed"),
            &OrderRequest::limit(
                hype_spot(),
                Side::Sell,
                Size::Base(Decimal::new(150, 2)),
                Decimal::new(4_231, 2),
            ),
        )
        .expect("a valid order");

        assert_eq!(perp.a, 0);
        assert!(perp.b);
        assert_eq!(perp.p, "27123");
        assert_eq!(perp.s, "0.12345");
        assert_eq!(perp.t.limit.tif, "Gtc");
        // Spot ids are the same numbers, pushed past the offset.
        assert_eq!(spot.a, 10_107);
        assert!(!spot.b);
        assert_eq!(spot.s, "1.5");
    }

    #[test]
    fn a_size_or_price_finer_than_the_asset_allows_is_refused_before_signing() {
        let universe = universe();
        let btc = universe.asset(&btc_perp()).expect("listed");

        // BTC carries five size decimals, so a sixth is not expressible.
        assert!(matches!(
            order_wire(
                btc,
                &OrderRequest::limit(
                    btc_perp(),
                    Side::Buy,
                    Size::Base(Decimal::new(123_456, 6)),
                    Decimal::from(27_123),
                ),
            ),
            Err(Error::InvalidRequest { field: "size", .. })
        ));
        // A fractional price is capped at five significant figures.
        assert!(matches!(
            order_wire(
                btc,
                &OrderRequest::limit(
                    btc_perp(),
                    Side::Buy,
                    Size::Base(Decimal::ONE),
                    Decimal::new(271_231, 1),
                ),
            ),
            Err(Error::InvalidRequest { field: "price", .. })
        ));
        // A whole-number price of any length is fine.
        assert!(
            order_wire(
                btc,
                &OrderRequest::limit(
                    btc_perp(),
                    Side::Buy,
                    Size::Base(Decimal::ONE),
                    Decimal::from(1_234_567),
                ),
            )
            .is_ok()
        );
    }

    #[test]
    fn trailing_zeros_are_stripped_because_the_text_is_what_gets_signed() {
        assert_eq!(wire_decimal(Decimal::new(1_200, 3)), "1.2");
        assert_eq!(wire_decimal(Decimal::new(2_712_300, 2)), "27123");
        assert_eq!(wire_decimal(Decimal::new(1, 5)), "0.00001");
        assert_eq!(significant_figures("27123"), 5);
        assert_eq!(significant_figures("0.00001"), 1);
        assert_eq!(significant_figures("271231"), 6);
    }

    #[test]
    fn a_market_order_is_refused_because_hyperliquid_has_no_such_type() {
        let universe = universe();
        let error = order_wire(
            universe.asset(&btc_perp()).expect("listed"),
            &OrderRequest::market(btc_perp(), Side::Buy, Size::Base(Decimal::ONE)),
        )
        .expect_err("no market orders");

        assert!(matches!(
            error,
            Error::Unsupported {
                feature: Feature::Trading,
                exchange: "hyperliquid",
                ..
            }
        ));
    }

    #[test]
    fn an_order_sized_in_the_quote_asset_is_refused() {
        let universe = universe();

        assert!(matches!(
            order_wire(
                universe.asset(&btc_perp()).expect("listed"),
                &OrderRequest::limit(
                    btc_perp(),
                    Side::Buy,
                    Size::Quote(Decimal::from(10_000)),
                    Decimal::from(27_123),
                ),
            ),
            Err(Error::InvalidRequest { field: "size", .. })
        ));
    }

    #[test]
    fn reduce_only_is_a_perpetual_idea_and_is_refused_on_spot() {
        let universe = universe();
        let request = OrderRequest::limit(
            hype_spot(),
            Side::Sell,
            Size::Base(Decimal::ONE),
            Decimal::new(4_231, 2),
        )
        .reduce_only();

        assert!(matches!(
            order_wire(universe.asset(&hype_spot()).expect("listed"), &request),
            Err(Error::Unsupported {
                feature: Feature::ReduceOnlyOrders,
                ..
            })
        ));
    }

    #[test]
    fn reduce_only_reaches_the_wire_as_the_flag_hyperliquid_reads() {
        let universe = universe();
        let request = OrderRequest::limit(
            btc_perp(),
            Side::Sell,
            Size::Base(Decimal::ONE),
            Decimal::from(27_123),
        )
        .reduce_only();

        assert!(
            order_wire(universe.asset(&btc_perp()).expect("listed"), &request)
                .expect("a valid order")
                .r
        );
    }

    #[test]
    fn each_time_in_force_reaches_hyperliquids_own_spelling() {
        assert_eq!(time_in_force(None).expect("a default"), "Gtc");
        assert_eq!(
            time_in_force(Some(TimeInForce::ImmediateOrCancel)).expect("mapped"),
            "Ioc"
        );
        assert_eq!(
            time_in_force(Some(TimeInForce::PostOnly)).expect("mapped"),
            "Alo"
        );
        assert!(matches!(
            time_in_force(Some(TimeInForce::FillOrKill)),
            Err(Error::Unsupported { .. })
        ));
    }

    #[test]
    fn a_history_cursor_wins_over_the_start_time_it_replaced() {
        let with_both = HistoryRequest::new(btc_perp())
            .from(Timestamp::from_millis(1_000))
            .cursor(parse::time_cursor(5_000).expect("a cursor"));

        assert_eq!(
            history_window(&with_both).expect("a window"),
            (5_001, None),
            "honouring `from` would repeat the previous page"
        );
        assert_eq!(
            history_window(&HistoryRequest::new(btc_perp())).expect("a window"),
            (0, None)
        );
    }

    #[test]
    fn a_cursor_appears_only_when_another_page_might_follow() {
        let items = vec![(1, 10), (2, 20), (3, 30)];

        let short = page(items.clone(), (Some(30), false), None).expect("a page");
        assert!(!short.has_more());

        let full = page(items.clone(), (Some(30), true), None).expect("a page");
        assert_eq!(full.next.expect("a cursor").as_str(), "31");

        let limited = page(items, (Some(30), false), Some(2)).expect("a page");
        assert_eq!(limited.items, vec![1, 2]);
        assert_eq!(limited.next.expect("a cursor").as_str(), "21");
    }

    #[test]
    fn a_filtered_page_resumes_past_the_whole_page_not_past_what_survived() {
        // `userFunding` answers for the whole account. If only one entry in a
        // full page belonged to the market asked about, resuming from that
        // entry's time would re-read every later entry for good.
        let one_survivor = page(vec![(1, 10)], (Some(500), true), None).expect("a page");
        assert_eq!(one_survivor.next.expect("a cursor").as_str(), "501");

        // Trimming is the one case where the caller has not seen the rest of
        // the page, so the cursor steps back to what they did see.
        let trimmed = page(vec![(1, 10), (2, 20)], (Some(500), true), Some(1)).expect("a page");
        assert_eq!(trimmed.items, vec![1]);
        assert_eq!(trimmed.next.expect("a cursor").as_str(), "11");
    }

    #[test]
    fn a_page_is_never_cut_through_the_middle_of_one_millisecond() {
        // Four entries, the last three stamped the same millisecond: funding
        // for several markets settles in one batch, so this is the ordinary
        // shape rather than a corner case. A cut at 2 would resume at 21 and
        // entries 3 and 4 would never be read by anyone.
        let batched = vec![(1, 10), (2, 20), (3, 20), (4, 20)];

        let page = page(batched, (Some(20), true), Some(2)).expect("a page");

        assert_eq!(page.items, vec![1]);
        assert_eq!(
            page.next.expect("a cursor").as_str(),
            "11",
            "the next page must still contain every entry stamped 20"
        );
    }

    #[test]
    fn a_page_that_is_one_long_millisecond_is_kept_whole_so_the_cursor_can_move() {
        // Cutting back to a boundary would empty this page, and an empty page
        // carries no cursor, so the caller would be stuck re-reading it.
        let one_batch = vec![(1, 20), (2, 20), (3, 20)];

        let page = page(one_batch, (Some(20), true), Some(2)).expect("a page");

        assert_eq!(page.items, vec![1, 2, 3], "a short page would lose entry 3");
        assert_eq!(page.next.expect("a cursor").as_str(), "21");
    }

    #[test]
    fn a_page_that_did_not_arrive_oldest_first_still_resumes_forwards() {
        // `newest` takes the largest time, not the last one: a cursor built
        // from the last entry of a descending page would move backwards, and
        // the same page would be fetched forever.
        let descending = [30, 20, 10];

        assert_eq!(newest(descending.into_iter()), (Some(30), false));
        assert_eq!(
            page(vec![(1, 30), (2, 20), (3, 10)], (Some(30), true), None)
                .expect("a page")
                .next
                .expect("a cursor")
                .as_str(),
            "31"
        );
    }

    #[test]
    fn funding_is_a_perpetual_idea_and_is_refused_on_spot() {
        let universe = universe();

        assert!(matches!(
            perpetual_asset(&universe, &hype_spot(), Feature::FundingRates),
            Err(Error::Unsupported {
                feature: Feature::FundingRates,
                exchange: "hyperliquid",
                ..
            })
        ));
        assert!(perpetual_asset(&universe, &btc_perp(), Feature::FundingRates).is_ok());
    }

    #[test]
    fn a_cancel_rejected_inside_an_accepted_envelope_is_still_an_error() {
        let accepted = parse::action_response(
            r#"{"status":"ok","response":{"type":"cancel","data":{"statuses":[{"error":"Order was never placed, already canceled, or filled."}]}}}"#,
        )
        .expect("an accepted envelope");

        assert!(matches!(
            cancel_ack(&accepted),
            Err(Error::Exchange { code, .. }) if code == "cancel_rejected"
        ));
        let succeeded = parse::action_response(
            r#"{"status":"ok","response":{"type":"cancel","data":{"statuses":["success"]}}}"#,
        )
        .expect("an accepted envelope");
        assert!(cancel_ack(&succeeded).is_ok());
    }

    #[test]
    fn a_nonce_is_the_millisecond_clock_hyperliquid_checks_against_its_own() {
        assert_eq!(
            nonce(Timestamp::from_millis(1_700_000_000_123)),
            1_700_000_000_123
        );
    }
}
