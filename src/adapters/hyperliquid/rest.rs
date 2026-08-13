//! Hyperliquid REST request construction and response handling.
//!
//! Reads use `POST /info`; signed actions use `POST /exchange`. Request type and
//! parameters are encoded in the JSON body.

use std::cmp::Reverse;

use rust_decimal::Decimal;
use serde_json::{Value, json};

use crate::adapters::candles as candle_pages;
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{
    CandleRequest, HistoryRequest, MarginRequest, OrderRequest, TransferHistoryRequest,
};
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{
    Balance, Candle, Cursor, Deposit, FundingPayment, FundingRate, Interval, MarginMode,
    MarginSummary, Market, MarketInfo, MarketKind, Order, OrderBook, OrderStatus, OrderType, Page,
    Position, Size, TimeInForce, Timestamp, Trade, Withdrawal,
};

use super::native;
use super::parse::{self, Asset, EXCHANGE, Universe};
use super::sign::{
    self, CancelAction, LeverageAction, LimitKind, OrderAction, OrderKind, OrderWire,
};
use super::{HyperliquidMidPrice, HyperliquidNetwork};

pub(crate) const INFO_PATH: &str = "/info";
pub(crate) const EXCHANGE_PATH: &str = "/exchange";

/// Hyperliquid returns at most this many book levels per side.
pub(crate) const MAX_BOOK_DEPTH: u32 = 20;

/// Number of recent candles retained by `candleSnapshot` per interval.
pub(crate) const MAX_CANDLE_COUNT: u32 = 5_000;

/// Fixed number of recent trades exposed by `recentTrades`.
pub(crate) const MAX_TRADE_COUNT: u32 = 10;

/// Maximum entries returned by one time-ranged history request.
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

/// Builds the default-universe `allMids` request.
pub(crate) fn all_mids_request() -> HttpRequest {
    info(json!({ "type": "allMids" }))
}

/// Builds the metadata-plus-context request used for ticker summaries.
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

/// Builds a `recentTrades` request after validating its local result limit.
/// The endpoint accepts neither a count nor a time range.
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

/// Builds Hyperliquid's compact `openOrders` request.
pub(crate) fn basic_open_orders_request(user: &str) -> HttpRequest {
    user_info_request("openOrders", user)
}

/// Builds an `orderStatus` request after validating its provider identifier.
pub(crate) fn order_status_request(
    user: &str,
    reference: &native::HyperliquidOrderReference,
) -> Result<HttpRequest> {
    validate_order_reference(reference)?;
    let oid = match reference {
        native::HyperliquidOrderReference::OrderId(order_id) => json!(order_id),
        native::HyperliquidOrderReference::ClientOrderId(client_order_id) => json!(client_order_id),
    };

    Ok(info(
        json!({ "type": "orderStatus", "user": user, "oid": oid }),
    ))
}

pub(crate) fn validate_order_reference(
    reference: &native::HyperliquidOrderReference,
) -> Result<()> {
    match reference {
        native::HyperliquidOrderReference::OrderId(_) => Ok(()),
        native::HyperliquidOrderReference::ClientOrderId(client_order_id) => {
            validate_client_order_id(client_order_id)
        }
    }
}

/// Builds Hyperliquid's documented recent historical-orders request.
pub(crate) fn historical_orders_request(user: &str) -> HttpRequest {
    user_info_request("historicalOrders", user)
}

fn validate_client_order_id(client_order_id: &str) -> Result<()> {
    let digits = client_order_id.strip_prefix("0x").ok_or_else(|| {
        Error::invalid_request(
            "client_order_id",
            "a Hyperliquid client order id needs a `0x` prefix",
        )
    })?;
    if digits.len() != 32 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(Error::invalid_request(
            "client_order_id",
            "a Hyperliquid client order id is exactly 16 bytes of hexadecimal",
        ));
    }
    Ok(())
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

pub(crate) fn user_rate_limit_request(user: &str) -> HttpRequest {
    user_info_request("userRateLimit", user)
}

pub(crate) fn user_role_request(user: &str) -> HttpRequest {
    user_info_request("userRole", user)
}

pub(crate) fn referral_request(user: &str) -> HttpRequest {
    user_info_request("referral", user)
}

pub(crate) fn user_fees_request(user: &str) -> HttpRequest {
    user_info_request("userFees", user)
}

pub(crate) fn portfolio_request(user: &str) -> HttpRequest {
    user_info_request("portfolio", user)
}

pub(crate) fn sub_accounts_request(user: &str) -> HttpRequest {
    user_info_request("subAccounts", user)
}

pub(crate) fn user_vault_equities_request(user: &str) -> HttpRequest {
    user_info_request("userVaultEquities", user)
}

/// Builds the documented most-recent account fills request.
pub(crate) fn user_fills_request(user: &str, aggregate_by_time: bool) -> HttpRequest {
    info(json!({
        "type": "userFills",
        "user": user,
        "aggregateByTime": aggregate_by_time,
    }))
}

/// Builds the documented time-ranged account fills request.
pub(crate) fn user_fills_by_time_request(
    user: &str,
    start_ms: i64,
    end_ms: Option<i64>,
    aggregate_by_time: bool,
) -> HttpRequest {
    let mut body = json!({
        "type": "userFillsByTime",
        "user": user,
        "startTime": start_ms,
        "aggregateByTime": aggregate_by_time,
    });
    if let Some(end_ms) = end_ms {
        body["endTime"] = json!(end_ms);
    }

    info(body)
}

fn user_fills_by_time_range_request(
    user: &str,
    from: Timestamp,
    to: Option<Timestamp>,
    aggregate_by_time: bool,
) -> HttpRequest {
    user_fills_by_time_request(
        user,
        // `startTime` is inclusive. Round up so a nanosecond-precise caller
        // boundary never admits an older whole-millisecond fill.
        crate::adapters::inclusive_millis_at_or_after(from),
        // `endTime` is also inclusive in Hyperliquid's API, so flooring
        // preserves the last whole millisecond within the caller boundary.
        to.map(Timestamp::as_millis),
        aggregate_by_time,
    )
}

fn user_info_request(request_type: &str, user: &str) -> HttpRequest {
    info(json!({ "type": request_type, "user": user }))
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

/// Sends a request and returns its successful response body.
///
/// Signed-action callers must also inspect the `/exchange` envelope with
/// [`parse::action_response`], because action rejection can use HTTP 200.
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

/// Reads current mids for the markets represented by this adapter's universe.
pub(crate) async fn all_mids(
    http: &HttpTransport,
    universe: &Universe,
) -> Result<Vec<HyperliquidMidPrice>> {
    let body = post(http, &all_mids_request()).await?;

    parse::all_mids(&parse::json(&body)?, universe)
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

/// Reads up to ten recent executions, newest first.
/// `limit`, when set, must be in `1..=10` and is applied locally.
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

/// Sorts recent trades newest first and applies the local result limit.
/// Stable sorting preserves provider order for equal timestamps.
fn newest_first(
    raw: &[parse::RawTrade],
    universe: &Universe,
    limit: Option<u32>,
) -> Result<Vec<Trade>> {
    let mut trades = raw
        .iter()
        .map(|raw| parse::trade(raw, universe))
        .collect::<Result<Vec<_>>>()?;

    trades.sort_by_key(|trade| Reverse(trade.timestamp));
    if let Some(limit) = limit {
        trades.truncate(limit as usize);
    }

    Ok(trades)
}

/// Reads a ticker summary from the market's asset context.
pub(crate) async fn ticker(
    http: &HttpTransport,
    universe: &Universe,
    market: &Market,
) -> Result<crate::types::Ticker> {
    let context = context(http, universe, market).await?;
    parse::ticker(&context, market, Timestamp::now())
}

/// Fetches one market's current asset context.
pub(crate) async fn context(
    http: &HttpTransport,
    universe: &Universe,
    market: &Market,
) -> Result<parse::RawAssetCtx> {
    let native = universe.native_symbol(market)?.to_string();
    let body = post(http, &asset_contexts_request(market.kind)).await?;

    pick_context(&body, market.kind, &native)
}

/// Selects one context from a `[meta, contexts]` response.
///
/// Spot contexts are matched by `coin`. Default perpetual contexts omit a name,
/// so they are matched by metadata position after equal lengths are verified.
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

/// Reads candles oldest first using time-window pagination.
/// Only the provider's most recent [`MAX_CANDLE_COUNT`] candles per interval are
/// available.
pub(crate) async fn candles(
    http: &HttpTransport,
    universe: &Universe,
    request: &CandleRequest,
    now: Timestamp,
) -> Result<Vec<Candle>> {
    let native = universe.native_symbol(&request.market)?.to_string();

    let interval = request.interval;
    candle_pages::read_on_grid(
        request,
        EXCHANGE,
        MAX_CANDLE_COUNT,
        move |at, count| advance_candle_grid(interval, at, count),
        |cursor, count| {
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
                // Remove the possible extra bucket requested for endpoint compatibility.
                page.retain(|candle| candle.open_time.as_millis() <= end_ms);

                Ok(page)
            }
        },
    )
    .await
}

/// Duration used for Hyperliquid `1M` request windows.
const MONTH_MS: i64 = 30 * 24 * 60 * 60 * 1_000;

/// Moves a candle boundary by `count` provider intervals.
/// Returns `None` when the result is outside [`Timestamp`]'s range.
fn candle_step_ms(interval: Interval, at_ms: i64, count: i64) -> Option<i64> {
    match interval {
        Interval::Month1 => MONTH_MS.checked_mul(count)?.checked_add(at_ms),
        interval => interval
            .advance(Timestamp::from_millis(at_ms), count)
            .map(Timestamp::as_millis),
    }
}

/// Moves a timestamp on Hyperliquid's candle grid.
fn advance_candle_grid(interval: Interval, at: Timestamp, count: i64) -> Option<Timestamp> {
    match interval {
        Interval::Month1 => {
            let delta = MONTH_MS.checked_mul(count)?.checked_mul(1_000_000)?;
            at.as_nanos().checked_add(delta).map(Timestamp::from_nanos)
        }
        interval => interval.advance(at, count),
    }
}

/// Extends `endTime` by one interval so responses gated by candle close still
/// include the requested final bucket. The caller removes any extra bucket.
fn candle_query_end_ms(interval: Interval, end_ms: i64) -> i64 {
    candle_step_ms(interval, end_ms, 1).unwrap_or(end_ms)
}

/// Calculates a `candleSnapshot` start time `count` intervals before `end_ms`.
/// Values before the Unix epoch are clamped to zero.
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

pub(crate) async fn user_rate_limit(
    http: &HttpTransport,
    user: &str,
) -> Result<native::HyperliquidUserRateLimit> {
    let body = post(http, &user_rate_limit_request(user)).await?;

    native::user_rate_limit(&parse::json(&body)?)
}

pub(crate) async fn user_role(
    http: &HttpTransport,
    user: &str,
) -> Result<native::HyperliquidUserRole> {
    let body = post(http, &user_role_request(user)).await?;

    native::user_role(&parse::json(&body)?)
}

pub(crate) async fn referral(
    http: &HttpTransport,
    user: &str,
) -> Result<native::HyperliquidReferral> {
    let body = post(http, &referral_request(user)).await?;

    native::referral(&parse::json(&body)?)
}

pub(crate) async fn user_fees(
    http: &HttpTransport,
    user: &str,
) -> Result<native::HyperliquidUserFees> {
    let body = post(http, &user_fees_request(user)).await?;

    native::user_fees(&parse::json(&body)?)
}

pub(crate) async fn portfolio(
    http: &HttpTransport,
    user: &str,
) -> Result<Vec<native::HyperliquidPortfolioPeriod>> {
    let body = post(http, &portfolio_request(user)).await?;

    native::portfolio(&parse::json(&body)?)
}

pub(crate) async fn sub_accounts(
    http: &HttpTransport,
    user: &str,
) -> Result<Vec<native::HyperliquidSubAccount>> {
    let body = post(http, &sub_accounts_request(user)).await?;

    native::sub_accounts(&parse::json(&body)?)
}

pub(crate) async fn user_vault_equities(
    http: &HttpTransport,
    user: &str,
) -> Result<Vec<native::HyperliquidVaultEquity>> {
    let body = post(http, &user_vault_equities_request(user)).await?;

    native::vault_equities(&parse::json(&body)?)
}

pub(crate) async fn user_fills(
    http: &HttpTransport,
    user: &str,
    aggregate_by_time: bool,
) -> Result<Vec<native::HyperliquidUserFill>> {
    let body = post(http, &user_fills_request(user, aggregate_by_time)).await?;

    native::user_fills(&parse::json(&body)?)
}

pub(crate) async fn user_fills_by_time(
    http: &HttpTransport,
    user: &str,
    from: Timestamp,
    to: Option<Timestamp>,
    aggregate_by_time: bool,
) -> Result<Vec<native::HyperliquidUserFill>> {
    let body = post(
        http,
        &user_fills_by_time_range_request(user, from, to, aggregate_by_time),
    )
    .await?;

    native::user_fills(&parse::json(&body)?)
}

pub(crate) async fn basic_open_orders(
    http: &HttpTransport,
    user: &str,
) -> Result<Vec<native::HyperliquidOpenOrder>> {
    let body = post(http, &basic_open_orders_request(user)).await?;

    native::basic_open_orders(&parse::json(&body)?)
}

pub(crate) async fn order_status(
    http: &HttpTransport,
    user: &str,
    reference: &native::HyperliquidOrderReference,
) -> Result<native::HyperliquidOrderStatusResponse> {
    let body = post(http, &order_status_request(user, reference)?).await?;

    native::order_status(&parse::json(&body)?)
}

pub(crate) async fn historical_orders(
    http: &HttpTransport,
    user: &str,
) -> Result<Vec<native::HyperliquidOrderInfo>> {
    let body = post(http, &historical_orders_request(user)).await?;

    native::historical_orders(&parse::json(&body)?)
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

/// Reads historical market funding-rate observations.
/// These are rates, not amounts charged to an account.
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
                    // `fundingHistory` does not provide a mark price.
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

/// Reads funding amounts charged or credited to the configured account.
pub(crate) async fn funding_payments(
    http: &HttpTransport,
    universe: &Universe,
    user: &str,
    request: &HistoryRequest,
) -> Result<Page<FundingPayment>> {
    let asset = perpetual_asset(universe, &request.market, Feature::FundingPayments)?;
    let (start_ms, end_ms) = history_window(request)?;
    let body = post(http, &user_funding_request(user, start_ms, end_ms)).await?;

    // `userFunding` is account-wide; filter it to the requested market.
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

    // Pagination is based on the unfiltered account-wide response.
    page(
        items,
        newest(raw.iter().map(|entry| entry.time)),
        request.limit,
    )
}

/// Reads account-wide non-funding ledger entries.
pub(crate) async fn ledger(
    http: &HttpTransport,
    user: &str,
    from: Option<Timestamp>,
    to: Option<Timestamp>,
    cursor: Option<&Cursor>,
    limit: Option<u32>,
) -> Result<Page<native::HyperliquidLedgerEntry>> {
    validate_page_limit(limit)?;

    let start_ms = match cursor {
        Some(cursor) => parse::cursor_start_ms(cursor)?,
        None => from.map(Timestamp::as_millis).unwrap_or(0),
    };
    let raw = raw_ledger(http, user, start_ms, to.map(Timestamp::as_millis)).await?;
    let items = native::ledger_entries(&raw)?
        .into_iter()
        .zip(raw.iter().map(|entry| entry.time))
        .collect();

    page(items, newest(raw.iter().map(|entry| entry.time)), limit)
}

/// Reads one common deposit-history page from the account-wide ledger.
pub(crate) async fn deposits(
    http: &HttpTransport,
    user: &str,
    request: &TransferHistoryRequest,
) -> Result<Page<Deposit>> {
    let raw = transfer_ledger(http, user, request).await?;
    let mut items = native::deposits(&raw)?;
    filter_asset(&mut items, request.asset.as_deref(), |deposit| {
        &deposit.asset
    });

    page(
        items,
        newest(raw.iter().map(|entry| entry.time)),
        request.limit,
    )
}

/// Reads one common withdrawal-history page from the account-wide ledger.
pub(crate) async fn withdrawals(
    http: &HttpTransport,
    user: &str,
    request: &TransferHistoryRequest,
) -> Result<Page<Withdrawal>> {
    let raw = transfer_ledger(http, user, request).await?;
    let mut items = native::withdrawals(&raw)?;
    filter_asset(&mut items, request.asset.as_deref(), |withdrawal| {
        &withdrawal.asset
    });

    page(
        items,
        newest(raw.iter().map(|entry| entry.time)),
        request.limit,
    )
}

async fn transfer_ledger(
    http: &HttpTransport,
    user: &str,
    request: &TransferHistoryRequest,
) -> Result<Vec<parse::RawLedgerUpdate>> {
    validate_page_limit(request.limit)?;
    let start_ms = request
        .cursor
        .as_ref()
        .map(parse::cursor_start_ms)
        .transpose()?
        .unwrap_or(0);

    raw_ledger(http, user, start_ms, None).await
}

async fn raw_ledger(
    http: &HttpTransport,
    user: &str,
    start_ms: i64,
    end_ms: Option<i64>,
) -> Result<Vec<parse::RawLedgerUpdate>> {
    let body = post(http, &ledger_request(user, start_ms, end_ms)).await?;

    parse::json(&body)
}

fn filter_asset<T>(items: &mut Vec<(T, i64)>, requested: Option<&str>, asset: impl Fn(&T) -> &str) {
    if let Some(requested) = requested {
        items.retain(|(item, _)| asset(item).eq_ignore_ascii_case(requested));
    }
}

/// Validates the transfer-history fields Hyperliquid can honor.
pub(crate) fn validate_transfer_history(
    request: &TransferHistoryRequest,
    feature: Feature,
) -> Result<()> {
    validate_page_limit(request.limit)?;
    if let Some(network) = &request.network {
        return Err(Error::unsupported(
            feature,
            EXCHANGE,
            format!(
                "userNonFundingLedgerUpdates does not identify a network, so `{network}` cannot be filtered safely"
            ),
        ));
    }

    Ok(())
}

/// Returns the greatest entry time and whether the provider page was full.
/// History responses provide neither a total nor a native cursor.
fn newest(times: impl Iterator<Item = i64>) -> (Option<i64>, bool) {
    let times: Vec<i64> = times.collect();

    (times.iter().max().copied(), times.len() >= MAX_HISTORY_PAGE)
}

pub(crate) fn validate_page_limit(limit: Option<u32>) -> Result<()> {
    if limit == Some(0) {
        return Err(Error::invalid_request("limit", "must be greater than zero"));
    }

    Ok(())
}

/// Assembles a page and a time-based continuation cursor.
/// Filtering and local truncation preserve the timestamp of the last item
/// visible to the caller.
fn page<T>(
    mut items: Vec<(T, i64)>,
    page_end: (Option<i64>, bool),
    limit: Option<u32>,
) -> Result<Page<T>> {
    validate_page_limit(limit)?;

    let (page_newest, full) = page_end;
    let mut truncated = false;

    if let Some(limit) = limit
        && items.len() > limit as usize
    {
        items.truncate(millisecond_boundary(&items, limit as usize));
        truncated = true;
    }

    // Resume after the visible item when truncated, otherwise after the raw page.
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

/// Finds a cut that does not split entries sharing one millisecond.
/// A leading same-millisecond run may exceed `limit` because the next cursor
/// advances past the entire millisecond.
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

/// Converts a history request to provider millisecond boundaries.
///
/// `from` is inclusive and `to` is exclusive. Hyperliquid's `endTime` is
/// inclusive, so `to` is converted to the last millisecond strictly before it.
/// A cursor takes precedence over `from`.
fn history_window(request: &HistoryRequest) -> Result<(i64, Option<i64>)> {
    validate_page_limit(request.limit)?;

    let start = match &request.cursor {
        Some(cursor) => parse::cursor_start_ms(cursor)?,
        // The API requires `startTime`; zero requests the earliest retained data.
        None => request
            .from
            .map(crate::adapters::inclusive_millis_at_or_after)
            .unwrap_or(0),
    };
    // Convert the exclusive nanosecond boundary to an inclusive millisecond.
    let end = request.to.map(crate::adapters::inclusive_millis_before);

    Ok((start, end))
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
        // The acknowledgement exposes only resting versus filled status.
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
        // The acknowledgement has no exchange timestamp.
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
    if request.client_id.is_some() {
        return Err(Error::unsupported(
            Feature::Trading,
            EXCHANGE,
            "hyperliquid client order ids require the provider-specific cloid contract",
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

/// Formats a positive price and enforces decimal-place and significant-digit
/// limits before signing.
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

/// Formats a signed-action decimal as plain digits without trailing zeros.
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

    // Leading zeros are not significant; trailing zeros were removed above.
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
) -> Result<()> {
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
    cancel_ack(&accepted)
}

/// Reads the per-cancel verdict inside a successful action envelope.
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

    // `updateLeverage` requires leverage and margin mode in one action.
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

#[cfg(test)]
mod tests {
    use super::super::parse::tests::{btc_perp, universe};
    use super::*;
    use crate::types::{Exchange, Side};

    /// Spot context fixture whose metadata and context arrays are not aligned.
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

    /// Default perpetual context fixture with position-aligned arrays.
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
            (all_mids_request(), "allMids"),
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
            (basic_open_orders_request("0xabc"), "openOrders"),
            (
                order_status_request(
                    "0xabc",
                    &native::HyperliquidOrderReference::order_id(u64::MAX),
                )
                .expect("u64 order id"),
                "orderStatus",
            ),
            (historical_orders_request("0xabc"), "historicalOrders"),
            (user_rate_limit_request("0xabc"), "userRateLimit"),
            (user_role_request("0xabc"), "userRole"),
            (referral_request("0xabc"), "referral"),
            (user_fees_request("0xabc"), "userFees"),
            (portfolio_request("0xabc"), "portfolio"),
            (sub_accounts_request("0xabc"), "subAccounts"),
            (user_vault_equities_request("0xabc"), "userVaultEquities"),
            (user_fills_request("0xabc", false), "userFills"),
            (
                user_fills_by_time_request("0xabc", 1_681_222_254_710, None, true),
                "userFillsByTime",
            ),
            (
                ledger_request("0xabc", 1_681_222_254_710, None),
                "userNonFundingLedgerUpdates",
            ),
        ] {
            assert_eq!(request.target(), INFO_PATH, "{expected}");
            assert_eq!(body_of(&request)["type"], expected);
        }

        assert!(body_of(&all_mids_request()).get("dex").is_none());

        let ledger = body_of(&ledger_request("0xabc", 1_681_222_254_710, None));
        assert_eq!(ledger["user"], "0xabc");
        assert_eq!(ledger["startTime"], 1_681_222_254_710_i64);
        assert!(ledger["endTime"].is_null());
    }

    #[test]
    fn every_account_info_read_names_the_configured_user() {
        for request in [
            user_rate_limit_request("0xabc"),
            user_role_request("0xabc"),
            referral_request("0xabc"),
            user_fees_request("0xabc"),
            portfolio_request("0xabc"),
            sub_accounts_request("0xabc"),
            user_vault_equities_request("0xabc"),
            user_fills_request("0xabc", false),
            user_fills_by_time_request("0xabc", 1_681_222_254_710, None, true),
            basic_open_orders_request("0xabc"),
            order_status_request("0xabc", &native::HyperliquidOrderReference::order_id(1))
                .expect("order status"),
            historical_orders_request("0xabc"),
        ] {
            let body = body_of(&request);

            assert_eq!(request.target(), INFO_PATH);
            assert_eq!(body["user"], "0xabc");
            assert!(body["type"].is_string());
        }
    }

    #[test]
    fn user_fill_requests_keep_the_documented_filter_shape() {
        let recent = body_of(&user_fills_request("0xabc", true));
        assert_eq!(
            recent,
            json!({
                "type": "userFills",
                "user": "0xabc",
                "aggregateByTime": true,
            })
        );

        let ranged = body_of(&user_fills_by_time_request(
            "0xabc",
            1_681_222_254_710,
            Some(1_681_222_254_999),
            false,
        ));
        assert_eq!(
            ranged,
            json!({
                "type": "userFillsByTime",
                "user": "0xabc",
                "startTime": 1_681_222_254_710_i64,
                "endTime": 1_681_222_254_999_i64,
                "aggregateByTime": false,
            })
        );
    }

    #[test]
    fn order_query_requests_match_the_documented_json_shapes() {
        assert_eq!(
            body_of(&basic_open_orders_request("0xabc")),
            json!({"type":"openOrders", "user":"0xabc"})
        );
        assert_eq!(
            body_of(
                &order_status_request(
                    "0xabc",
                    &native::HyperliquidOrderReference::order_id(u64::MAX),
                )
                .expect("maximum order id")
            ),
            json!({"type":"orderStatus", "user":"0xabc", "oid":u64::MAX})
        );
        assert_eq!(
            body_of(
                &order_status_request(
                    "0xabc",
                    &native::HyperliquidOrderReference::client_order_id(
                        "0x0123456789abcdef0123456789ABCDEF",
                    ),
                )
                .expect("16-byte client order id")
            ),
            json!({
                "type":"orderStatus",
                "user":"0xabc",
                "oid":"0x0123456789abcdef0123456789ABCDEF"
            })
        );
        assert_eq!(
            body_of(&historical_orders_request("0xabc")),
            json!({"type":"historicalOrders", "user":"0xabc"})
        );
    }

    #[test]
    fn client_order_ids_must_be_exactly_sixteen_bytes_of_prefixed_hex() {
        for invalid in [
            "0123456789abcdef0123456789abcdef",
            "0x0123456789abcdef0123456789abcde",
            "0x0123456789abcdef0123456789abcdef0",
            "0x0123456789abcdef0123456789abcdeg",
        ] {
            let result = order_status_request(
                "0xabc",
                &native::HyperliquidOrderReference::client_order_id(invalid),
            );
            assert!(
                matches!(&result, Err(Error::InvalidRequest { field, .. }) if *field == "client_order_id"),
                "{invalid} was accepted: {result:?}"
            );
        }
    }

    #[test]
    fn user_fill_time_range_rounds_only_the_inclusive_start_up_to_a_millisecond() {
        let from = Timestamp::from_nanos(1_681_222_254_710_000_001);
        let to = Timestamp::from_nanos(1_681_222_254_999_999_999);

        let request = user_fills_by_time_range_request("0xabc", from, Some(to), false);
        let body = body_of(&request);

        assert_eq!(body["startTime"], 1_681_222_254_711_i64);
        assert_eq!(body["endTime"], 1_681_222_254_999_i64);
    }

    #[test]
    fn a_spot_context_is_the_one_naming_the_market_not_the_one_beside_it() {
        let hype = pick_context(SPOT_ASSET_CTXS, MarketKind::Spot, "@107").expect("a context");
        let purr = pick_context(SPOT_ASSET_CTXS, MarketKind::Spot, "PURR/USDC").expect("a context");

        // Position-based matching would select the unrelated `@105` context.
        assert_eq!(hype.coin.as_deref(), Some("@107"));
        assert_eq!(hype.mid_px.as_deref(), Some("53.6865"));
        // Slash-form contexts resolve by their native name.
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
        // Positional matching is unsafe when the arrays differ in length.
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
        // Open-ended requests omit `endTime`.
        assert!(body_of(&candles_request("BTC", "1m", 0, None))["req"]["endTime"].is_null());
    }

    #[test]
    fn a_window_asks_one_interval_past_its_end_because_older_buckets_answer_by_their_close() {
        // The request end advances one interval; the page reader trims extras.
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
        // `1M` request windows use the provider's fixed duration.
        assert_ne!(
            candle_query_end_ms(Interval::Month1, JULY_7_2020),
            Interval::Month1
                .advance(Timestamp::from_millis(JULY_7_2020), 1)
                .expect("a month on")
                .as_millis()
        );
        // Overflow leaves the requested end unchanged.
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
        // Starts before the epoch clamp to zero.
        assert_eq!(candle_start_ms(&request, 60_000, 100), 0);
    }

    #[test]
    fn a_monthly_window_counts_back_hyperliquids_thirty_day_buckets() {
        const END: i64 = 1_700_000_000_000;
        let monthly = CandleRequest::new(btc_perp(), Interval::Month1);

        // Three provider `1M` request intervals span ninety days.
        assert_eq!(candle_start_ms(&monthly, END, 3), END - 3 * MONTH_MS);
        assert_eq!((END - candle_start_ms(&monthly, END, 3)) / 86_400_000, 90);
        assert_eq!(1_783_296_000_000_i64 % MONTH_MS, 0, "2026-07-06T00:00:00Z");
        let boundary = Timestamp::from_millis(1_783_296_000_000);
        assert_eq!(
            advance_candle_grid(Interval::Month1, boundary, 12),
            Some(Timestamp::from_millis(boundary.as_millis() + 12 * MONTH_MS))
        );
        // Page width is determined by count; the outer walker applies `from`.
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
        // A request window wider than `Timestamp` starts at the epoch.
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
        // Common request validation runs before provider interval mapping.
        let http = HttpTransport::new("http://127.0.0.1:1").expect("a transport");
        let request = CandleRequest::new(btc_perp(), Interval::Sec1).limit(0);

        // The invalid limit prevents a network request.
        let refused = candles(&http, &universe(), &request, Timestamp::default()).await;

        assert!(
            matches!(&refused, Err(Error::InvalidRequest { field, .. }) if *field == "limit"),
            "{refused:?}"
        );
    }

    #[test]
    fn every_interval_in_the_common_baseline_has_a_hyperliquid_spelling() {
        // Every common baseline interval has a native spelling.
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
        // The documented depth boundary is inclusive.
        assert!(!(1..=MAX_BOOK_DEPTH).contains(&21));
        assert!(!(1..=MAX_BOOK_DEPTH).contains(&0));
    }

    #[test]
    fn a_trade_request_names_the_market_and_carries_no_count() {
        let body = body_of(&trades_request("@107", Some(5)).expect("five is inside the window"));

        assert_eq!(body["type"], "recentTrades");
        assert_eq!(body["coin"], "@107");
        // `recentTrades` has no wire-level count parameter.
        assert!(body.get("n").is_none() && body.get("limit").is_none());
    }

    /// Recent-trade fixture with equal timestamps and repeated transaction hashes.
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
        // `tid`, not the repeated transaction hash, identifies each trade.
        let ids: std::collections::BTreeSet<_> =
            trades.iter().filter_map(|trade| trade.id.clone()).collect();
        assert_eq!(ids.len(), 4);
        assert!(ids.contains("732538781370075"));
    }

    #[test]
    fn a_trade_limit_cuts_the_page_hyperliquid_sent() {
        let raw: Vec<parse::RawTrade> = parse::json(RECENT_TRADES).expect("a recentTrades payload");
        let trades = newest_first(&raw, &universe(), Some(2)).expect("two trades");

        // Apply the limit after newest-first ordering.
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
                    Err(Error::InvalidRequest { field, .. }) if field == "limit"
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
        // Spot action ids include the spot offset.
        assert_eq!(spot.a, 10_107);
        assert!(!spot.b);
        assert_eq!(spot.s, "1.5");
    }

    #[test]
    fn a_size_or_price_finer_than_the_asset_allows_is_refused_before_signing() {
        let universe = universe();
        let btc = universe.asset(&btc_perp()).expect("listed");

        // Size precision comes from `szDecimals`.
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
            Err(Error::InvalidRequest { field, .. }) if field == "size"
        ));
        // Fractional prices are capped at five significant digits.
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
            Err(Error::InvalidRequest { field, .. }) if field == "price"
        ));
        // Integer prices are exempt from the significant-digit cap.
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
            Err(Error::InvalidRequest { field, .. }) if field == "size"
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

        let inside_millisecond =
            HistoryRequest::new(btc_perp()).from(Timestamp::from_nanos(5_000_000_001));
        assert_eq!(
            history_window(&inside_millisecond).expect("an inclusive start"),
            (5_001, None)
        );
    }

    #[test]
    fn a_history_end_becomes_the_last_inclusive_millisecond_before_it() {
        let exact_millisecond = HistoryRequest::new(btc_perp()).to(Timestamp::from_millis(5_000));
        let inside_millisecond =
            HistoryRequest::new(btc_perp()).to(Timestamp::from_nanos(5_000_000_001));
        let negative_exact = HistoryRequest::new(btc_perp()).to(Timestamp::from_millis(-5_000));
        let negative_inside =
            HistoryRequest::new(btc_perp()).to(Timestamp::from_nanos(-4_999_999_999));

        assert_eq!(
            history_window(&exact_millisecond).expect("a window"),
            (0, Some(4_999))
        );
        assert_eq!(
            history_window(&inside_millisecond).expect("a window"),
            (0, Some(5_000))
        );
        assert_eq!(
            history_window(&negative_exact).expect("a window"),
            (0, Some(-5_001))
        );
        assert_eq!(
            history_window(&negative_inside).expect("a window"),
            (0, Some(-5_000))
        );
    }

    #[test]
    fn a_zero_history_limit_is_rejected_before_building_the_request_window() {
        let request = HistoryRequest::new(btc_perp()).limit(0);

        assert!(matches!(
            history_window(&request),
            Err(Error::InvalidRequest { field, .. }) if field == "limit"
        ));
    }

    #[tokio::test]
    async fn a_zero_ledger_limit_is_rejected_before_the_network() {
        let http = HttpTransport::new("http://127.0.0.1:1").expect("a transport");

        let refused = ledger(&http, "0xabc", None, None, None, Some(0)).await;

        assert!(
            matches!(&refused, Err(Error::InvalidRequest { field, .. }) if *field == "limit"),
            "{refused:?}"
        );
    }

    #[test]
    fn a_zero_page_limit_is_rejected_instead_of_losing_items_and_the_cursor() {
        let refused = page(vec![(1, 10)], (Some(10), true), Some(0));

        assert!(matches!(
            refused,
            Err(Error::InvalidRequest { field, .. }) if field == "limit"
        ));
    }

    #[test]
    fn valid_history_limits_preserve_page_and_timestamp_group_semantics() {
        let unlimited =
            page(vec![(1, 10), (2, 20)], (Some(20), true), None).expect("an unlimited page");
        assert_eq!(unlimited.items, vec![1, 2]);
        assert_eq!(unlimited.next.expect("a cursor").as_str(), "21");

        let one = page(vec![(1, 20), (2, 20), (3, 30)], (Some(30), false), Some(1))
            .expect("a one-item target");
        assert_eq!(one.items, vec![1, 2]);
        assert_eq!(one.next.expect("a cursor").as_str(), "21");

        let large = page(vec![(1, 10), (2, 20)], (Some(20), false), Some(u32::MAX))
            .expect("a large provider-independent limit");
        assert_eq!(large.items, vec![1, 2]);
        assert!(!large.has_more());
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
        // An untrimmed filtered page resumes after the raw account-wide page.
        let one_survivor = page(vec![(1, 10)], (Some(500), true), None).expect("a page");
        assert_eq!(one_survivor.next.expect("a cursor").as_str(), "501");

        // A locally trimmed page resumes after the last visible entry.
        let trimmed = page(vec![(1, 10), (2, 20)], (Some(500), true), Some(1)).expect("a page");
        assert_eq!(trimmed.items, vec![1]);
        assert_eq!(trimmed.next.expect("a cursor").as_str(), "11");
    }

    #[test]
    fn a_page_is_never_cut_through_the_middle_of_one_millisecond() {
        // Splitting a timestamp group would skip unseen entries on resume.
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
        // Keep a leading timestamp group whole so the cursor can advance.
        let one_batch = vec![(1, 20), (2, 20), (3, 20)];

        let page = page(one_batch, (Some(20), true), Some(2)).expect("a page");

        assert_eq!(page.items, vec![1, 2, 3], "a short page would lose entry 3");
        assert_eq!(page.next.expect("a cursor").as_str(), "21");
    }

    #[test]
    fn a_page_that_did_not_arrive_oldest_first_still_resumes_forwards() {
        // Cursor progress uses the greatest timestamp regardless of response order.
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
}
