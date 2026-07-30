//! Binance's signed REST API: balances, orders, positions, margin, funding.
//!
//! Every call here carries the API key in a header and an HMAC-SHA256
//! signature over the query string. Binance signs the *bytes* of that query,
//! not the set of parameters, so the query is built once, hashed, and sent
//! verbatim. Re-encoding it between those two steps is the classic way to
//! produce a request the exchange rejects with `-1022`.

use hmac::{Hmac, KeyInit, Mac};
use rust_decimal::Decimal;
use serde::Deserialize;
use sha2::Sha256;

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{HistoryRequest, MarginRequest, OrderRequest};
use crate::transport::{HttpMethod, HttpRequest};
use crate::types::{
    Balance, Cursor, FundingPayment, FundingRate, MarginMode, MarginSummary, Market, Order,
    OrderType, Page, Position, Side, Size, TimeInForce, Timestamp,
};

use super::{
    API_KEY_HEADER, BinanceAdapter, BinanceCredentials, BinanceMarket, EXCHANGE, decode_cursor,
    encode_cursor, now_millis, parse,
    rest::{encode, query},
};

/// The most entries either paginated history returns in one call.
const MAX_HISTORY_LIMIT: u32 = 1_000;
/// What a history page holds when the caller does not say.
const DEFAULT_HISTORY_LIMIT: u32 = 100;

/// The HMAC-SHA-256 of a payload under the account's secret key, hex encoded.
///
/// The same primitive signs a REST query and a WebSocket API request. They
/// differ only in what they sign: REST signs the query string it is about to
/// send, the WebSocket API signs its `params` sorted by name.
fn signature(credentials: &BinanceCredentials, payload: &str) -> Result<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(credentials.secret_key.as_bytes())
        .map_err(|_| Error::auth("binance secret key cannot be used as an HMAC key"))?;
    mac.update(payload.as_bytes());

    Ok(hex::encode(mac.finalize().into_bytes()))
}

/// Signs a query string with the account's secret key.
///
/// Returns the query with `signature` appended, ready to send unchanged.
fn sign(credentials: &BinanceCredentials, payload: &str) -> Result<String> {
    let signature = signature(credentials, payload)?;

    Ok(if payload.is_empty() {
        format!("signature={signature}")
    } else {
        format!("{payload}&signature={signature}")
    })
}

/// Builds a signed request, stamping it with the current time.
///
/// `timestamp` is appended last so the signed text ends where Binance's own
/// examples end, which keeps the vector in the tests comparable to the docs.
fn signed(
    adapter: &BinanceAdapter,
    method: HttpMethod,
    path: &str,
    mut params: Vec<(&str, String)>,
) -> Result<HttpRequest> {
    let credentials = adapter.credentials()?;
    params.push(("timestamp", now_millis().to_string()));

    Ok(HttpRequest::new(method, path.to_string())
        .query(sign(credentials, &query(&params))?)
        .header(API_KEY_HEADER, credentials.api_key.clone()))
}

/// Builds a request that authenticates with the API key alone.
///
/// The USD-M listen key endpoints take no signature: the key names the account
/// and the listen key they return is the secret from then on.
fn api_key_only(
    adapter: &BinanceAdapter,
    method: HttpMethod,
    path: &str,
    params: &[(&str, String)],
) -> Result<HttpRequest> {
    let credentials = adapter.credentials()?;

    Ok(HttpRequest::new(method, path.to_string())
        .query(query(params))
        .header(API_KEY_HEADER, credentials.api_key.clone()))
}

// ---------------------------------------------------------------------------
// Raw payloads
// ---------------------------------------------------------------------------

/// `GET /api/v3/account`.
#[derive(Debug, Deserialize)]
struct RawSpotAccount {
    balances: Vec<RawSpotBalance>,
}

#[derive(Debug, Deserialize)]
struct RawSpotBalance {
    asset: String,
    free: String,
    locked: String,
}

/// `GET /fapi/v3/account`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFuturesAccount {
    total_margin_balance: String,
    total_initial_margin: String,
    available_balance: String,
    assets: Vec<RawFuturesAsset>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFuturesAsset {
    asset: String,
    wallet_balance: String,
    available_balance: String,
}

/// `GET /fapi/v3/positionRisk`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPosition {
    symbol: String,
    position_amt: String,
    entry_price: String,
    mark_price: String,
    #[serde(rename = "unRealizedProfit")]
    unrealized_profit: String,
    notional: String,
}

/// `GET /fapi/v1/fundingRate`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFundingRate {
    funding_time: i64,
    funding_rate: String,
    mark_price: Option<String>,
}

/// `GET /fapi/v1/income`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawIncome {
    income: String,
    time: i64,
    tran_id: serde_json::Number,
}

/// `POST /fapi/v1/listenKey`. Spot has no listen key; see
/// [`spot_user_data_subscribe_frame`].
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawListenKey {
    listen_key: String,
}

/// A single spot order as `GET /api/v3/order` reports it.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSpotOrderDetail {
    #[serde(flatten)]
    order: parse::RawOrder,
    client_order_id: String,
    #[serde(rename = "type")]
    order_type: String,
    time_in_force: String,
    cummulative_quote_qty: String,
    update_time: Option<i64>,
}

// ---------------------------------------------------------------------------
// Request building
// ---------------------------------------------------------------------------

/// The path an account read hangs off, per venue.
fn account_path(venue: BinanceMarket) -> &'static str {
    match venue {
        // Spot's account read carries balances; USD-M's carries balances,
        // account-wide margin, and positions in one payload.
        BinanceMarket::Spot => "/api/v3/account",
        BinanceMarket::UsdMFutures => "/fapi/v3/account",
    }
}

fn order_path(venue: BinanceMarket) -> &'static str {
    match venue {
        BinanceMarket::Spot => "/api/v3/order",
        BinanceMarket::UsdMFutures => "/fapi/v1/order",
    }
}

fn open_orders_path(venue: BinanceMarket) -> &'static str {
    match venue {
        BinanceMarket::Spot => "/api/v3/openOrders",
        BinanceMarket::UsdMFutures => "/fapi/v1/openOrders",
    }
}

/// Where USD-M mints, extends, and closes a listen key.
///
/// USD-M only, and there is no spot counterpart to pair it with. Binance
/// removed `POST`, `PUT`, and `DELETE /api/v3/userDataStream` on
/// 2026-02-20 07:00 UTC; the host answers all three with `410 Gone`, measured
/// 2026-07-31. Spot subscribes over the WebSocket API instead, which needs no
/// key at all: see [`spot_user_data_subscribe_frame`].
const USD_M_LISTEN_KEY_PATH: &str = "/fapi/v1/listenKey";

pub(super) fn balances_request(adapter: &BinanceAdapter) -> Result<HttpRequest> {
    signed(
        adapter,
        HttpMethod::Get,
        account_path(adapter.venue()),
        Vec::new(),
    )
}

pub(super) fn open_orders_request(
    adapter: &BinanceAdapter,
    market: Option<&Market>,
) -> Result<HttpRequest> {
    let mut params = Vec::new();
    if let Some(market) = market {
        params.push(("symbol", adapter.symbol(market)?));
    }
    signed(
        adapter,
        HttpMethod::Get,
        open_orders_path(adapter.venue()),
        params,
    )
}

/// The `side` Binance expects.
fn side_code(side: Side) -> &'static str {
    match side {
        Side::Buy => "BUY",
        Side::Sell => "SELL",
    }
}

/// Builds the type, size, price, and time-in-force parameters of an order.
///
/// The two venues disagree here more than anywhere else: spot sizes a market
/// buy in the quote asset, futures cannot; spot spells post-only as an order
/// *type*, futures as a time in force.
fn order_shape(
    venue: BinanceMarket,
    request: &OrderRequest,
) -> Result<Vec<(&'static str, String)>> {
    let post_only = request.time_in_force == Some(TimeInForce::PostOnly);
    let mut params: Vec<(&'static str, String)> = Vec::new();

    let order_type = match (&request.order_type, venue, post_only) {
        (OrderType::Market, _, true) => {
            return Err(Error::invalid_request(
                "time_in_force",
                "a market order takes liquidity by definition and cannot be post-only",
            ));
        }
        (OrderType::Market, _, false) => "MARKET",
        // Spot has no post-only time in force; it has a `LIMIT_MAKER` type
        // that rejects rather than crossing, which is the same guarantee.
        (OrderType::Limit, BinanceMarket::Spot, true) => "LIMIT_MAKER",
        (OrderType::Limit, _, _) => "LIMIT",
    };
    params.push(("type", order_type.to_string()));

    match (&request.size, &request.order_type, venue) {
        (Size::Base(quantity), _, _) => params.push(("quantity", quantity.to_string())),
        (Size::Quote(amount), OrderType::Market, BinanceMarket::Spot) => {
            params.push(("quoteOrderQty", amount.to_string()));
        }
        (Size::Quote(_), OrderType::Market, BinanceMarket::UsdMFutures) => {
            return Err(Error::invalid_request(
                "size",
                "USD-M futures sizes every order in contracts; use Size::Base",
            ));
        }
        (Size::Quote(_), OrderType::Limit, _) => {
            return Err(Error::invalid_request(
                "size",
                "binance sizes a limit order in the base asset; use Size::Base",
            ));
        }
    }

    if let Some(price) = request.price {
        params.push(("price", price.to_string()));
    }

    // `LIMIT_MAKER` carries no time in force: the type already says it.
    if order_type == "LIMIT"
        && let Some(code) = time_in_force_code(venue, request.time_in_force)
    {
        params.push(("timeInForce", code.to_string()));
    }

    Ok(params)
}

/// The `timeInForce` Binance expects, or `None` when the order needs none.
fn time_in_force_code(venue: BinanceMarket, tif: Option<TimeInForce>) -> Option<&'static str> {
    Some(match tif {
        // A limit order needs one, and `GTC` is what Binance itself defaults to.
        None | Some(TimeInForce::GoodTilCancelled) => "GTC",
        Some(TimeInForce::ImmediateOrCancel) => "IOC",
        Some(TimeInForce::FillOrKill) => "FOK",
        // Reached on USD-M only; spot turns post-only into `LIMIT_MAKER` above.
        Some(TimeInForce::PostOnly) => match venue {
            BinanceMarket::UsdMFutures => "GTX",
            BinanceMarket::Spot => return None,
        },
    })
}

pub(super) fn place_order_request(
    adapter: &BinanceAdapter,
    request: &OrderRequest,
) -> Result<HttpRequest> {
    let venue = adapter.venue();
    if request.reduce_only && venue == BinanceMarket::Spot {
        return Err(Error::unsupported(
            Feature::ReduceOnlyOrders,
            EXCHANGE,
            "spot holds no positions to reduce",
        ));
    }

    let mut params = vec![
        ("symbol", adapter.symbol(&request.market)?),
        ("side", side_code(request.side).to_string()),
    ];
    params.extend(order_shape(venue, request)?);
    if request.reduce_only {
        params.push(("reduceOnly", "true".to_string()));
    }
    // Both venues answer with a bare acknowledgement by default; `RESULT` is
    // what makes the response describe the order that was actually placed.
    params.push(("newOrderRespType", "RESULT".to_string()));

    signed(adapter, HttpMethod::Post, order_path(venue), params)
}

pub(super) fn cancel_order_request(
    adapter: &BinanceAdapter,
    market: &Market,
    order_id: &str,
) -> Result<HttpRequest> {
    check_order_id(order_id)?;
    let params = vec![
        ("symbol", adapter.symbol(market)?),
        ("orderId", order_id.to_string()),
    ];

    signed(
        adapter,
        HttpMethod::Delete,
        order_path(adapter.venue()),
        params,
    )
}

/// Rejects an order id that is not the integer Binance issues.
///
/// Binance's own ids are decimal integers. Anything else is either a client
/// order id, which cancels through a different parameter, or an injection.
fn check_order_id(order_id: &str) -> Result<()> {
    if !order_id.is_empty() && order_id.bytes().all(|byte| byte.is_ascii_digit()) {
        return Ok(());
    }
    Err(Error::invalid_request(
        "order_id",
        format!("`{order_id}` is not a Binance order id: expected decimal digits"),
    ))
}

pub(super) fn positions_request(
    adapter: &BinanceAdapter,
    market: Option<&Market>,
) -> Result<HttpRequest> {
    check_futures_only(adapter, Feature::Positions)?;
    let mut params = Vec::new();
    if let Some(market) = market {
        params.push(("symbol", adapter.symbol(market)?));
    }
    signed(adapter, HttpMethod::Get, "/fapi/v3/positionRisk", params)
}

pub(super) fn funding_rates_request(
    adapter: &BinanceAdapter,
    request: &HistoryRequest,
) -> Result<HttpRequest> {
    check_futures_only(adapter, Feature::FundingRates)?;
    let mut params = vec![("symbol", adapter.symbol(&request.market)?)];
    params.extend(history_window(request)?);

    // Funding rate history is public: no key, no signature.
    Ok(HttpRequest::get("/fapi/v1/fundingRate").query(query(&params)))
}

pub(super) fn funding_payments_request(
    adapter: &BinanceAdapter,
    request: &HistoryRequest,
) -> Result<HttpRequest> {
    check_futures_only(adapter, Feature::FundingPayments)?;
    let mut params = vec![
        ("symbol", adapter.symbol(&request.market)?),
        // `/fapi/v1/income` is the whole ledger: commissions, realized PnL,
        // transfers. Only the funding rows are funding payments.
        ("incomeType", "FUNDING_FEE".to_string()),
    ];
    params.extend(history_window(request)?);

    signed(adapter, HttpMethod::Get, "/fapi/v1/income", params)
}

/// The `startTime`, `endTime`, and `limit` a history page asks for.
///
/// A cursor supersedes `from`: it is the millisecond the previous page stopped
/// at, and resuming from anywhere else would repeat or skip entries.
fn history_window(request: &HistoryRequest) -> Result<Vec<(&'static str, String)>> {
    let limit = request.limit.unwrap_or(DEFAULT_HISTORY_LIMIT);
    if !(1..=MAX_HISTORY_LIMIT).contains(&limit) {
        return Err(Error::invalid_request(
            "limit",
            format!(
                "binance serves 1 to {MAX_HISTORY_LIMIT} history entries per page, not {limit}"
            ),
        ));
    }

    let start = match &request.cursor {
        Some(cursor) => Some(decode_cursor(cursor)?),
        None => request.from.map(Timestamp::as_millis),
    };

    let mut params = Vec::new();
    if let Some(start) = start {
        params.push(("startTime", start.to_string()));
    }
    if let Some(to) = request.to {
        // Binance's `endTime` is inclusive; `HistoryRequest::to` is not.
        params.push(("endTime", to.as_millis().saturating_sub(1).to_string()));
    }
    params.push(("limit", limit.to_string()));

    Ok(params)
}

pub(super) fn set_margin_requests(
    adapter: &BinanceAdapter,
    request: &MarginRequest,
) -> Result<Vec<HttpRequest>> {
    check_futures_only(adapter, Feature::MarginConfig)?;
    if request.leverage.is_none() && request.margin_mode.is_none() {
        return Err(Error::invalid_request(
            "leverage",
            "set at least one of leverage or margin mode",
        ));
    }

    let symbol = adapter.symbol(&request.market)?;
    let mut requests = Vec::with_capacity(2);

    if let Some(leverage) = request.leverage {
        requests.push(signed(
            adapter,
            HttpMethod::Post,
            "/fapi/v1/leverage",
            vec![
                ("symbol", symbol.clone()),
                ("leverage", leverage_code(leverage)?),
            ],
        )?);
    }
    if let Some(mode) = request.margin_mode {
        requests.push(signed(
            adapter,
            HttpMethod::Post,
            "/fapi/v1/marginType",
            vec![
                ("symbol", symbol),
                (
                    "marginType",
                    match mode {
                        // Binance spells cross margin `CROSSED`, not `CROSS`.
                        MarginMode::Cross => "CROSSED".to_string(),
                        MarginMode::Isolated => "ISOLATED".to_string(),
                    },
                ),
            ],
        )?);
    }

    Ok(requests)
}

/// Binance takes leverage as a whole multiplier.
fn leverage_code(leverage: Decimal) -> Result<String> {
    if leverage.fract() != Decimal::ZERO || leverage < Decimal::ONE {
        return Err(Error::invalid_request(
            "leverage",
            format!("binance takes whole leverage multipliers from 1 upwards, not {leverage}"),
        ));
    }
    Ok(leverage.trunc().to_string())
}

fn check_futures_only(adapter: &BinanceAdapter, feature: Feature) -> Result<()> {
    if adapter.venue() == BinanceMarket::UsdMFutures {
        return Ok(());
    }
    Err(Error::unsupported(
        feature,
        EXCHANGE,
        "binance spot is not a derivatives venue; build the adapter with `usd_m_futures`",
    ))
}

// ---------------------------------------------------------------------------
// Calls
// ---------------------------------------------------------------------------

pub(super) async fn balances(adapter: &BinanceAdapter) -> Result<Vec<Balance>> {
    let body = adapter.send(balances_request(adapter)?).await?;

    match adapter.venue() {
        BinanceMarket::Spot => parse::json::<RawSpotAccount>(&body, "account")?
            .balances
            .iter()
            .map(|raw| {
                Ok(Balance {
                    asset: raw.asset.to_ascii_uppercase(),
                    available: parse::decimal(&raw.free, "free")?,
                    locked: parse::decimal(&raw.locked, "locked")?,
                })
            })
            .collect(),
        BinanceMarket::UsdMFutures => parse::json::<RawFuturesAccount>(&body, "account")?
            .assets
            .iter()
            .map(futures_balance)
            .collect(),
    }
}

/// Maps one margin asset onto the available/locked split `Balance` promises.
///
/// USD-M reports a wallet balance and how much of it is free; what is not free
/// is posted against open positions and orders, which is exactly what
/// [`Balance::locked`] means.
fn futures_balance(raw: &RawFuturesAsset) -> Result<Balance> {
    let wallet = parse::decimal(&raw.wallet_balance, "walletBalance")?;
    let available = parse::decimal(&raw.available_balance, "availableBalance")?;

    Ok(Balance {
        asset: raw.asset.to_ascii_uppercase(),
        available,
        // A losing position can push the free balance above the wallet balance
        // for one asset in multi-asset mode; a negative lock would be nonsense.
        locked: (wallet - available).max(Decimal::ZERO),
    })
}

pub(super) async fn open_orders(
    adapter: &BinanceAdapter,
    market: Option<&Market>,
) -> Result<Vec<Order>> {
    let body = adapter.send(open_orders_request(adapter, market)?).await?;
    parse::json::<Vec<parse::RawOrder>>(&body, "openOrders")?
        .iter()
        .map(|raw| {
            // A list covering every market names each order's own symbol, and
            // the caller's market is not available to name it for us.
            let market = adapter.market(&raw.symbol)?;
            parse::order(&market, raw)
        })
        .collect()
}

pub(super) async fn place_order(adapter: &BinanceAdapter, request: &OrderRequest) -> Result<Order> {
    let body = adapter.send(place_order_request(adapter, request)?).await?;
    let raw: parse::RawOrder = parse::json(&body, "order")?;
    parse::order(&request.market, &raw)
}

pub(super) async fn cancel_order(
    adapter: &BinanceAdapter,
    market: &Market,
    order_id: &str,
) -> Result<Order> {
    let body = adapter
        .send(cancel_order_request(adapter, market, order_id)?)
        .await?;
    let raw: parse::RawOrder = parse::json(&body, "order")?;
    parse::order(market, &raw)
}

pub(super) async fn positions(
    adapter: &BinanceAdapter,
    market: Option<&Market>,
) -> Result<Vec<Position>> {
    let body = adapter.send(positions_request(adapter, market)?).await?;
    let raw: Vec<RawPosition> = parse::json(&body, "positionRisk")?;
    open_positions(adapter, &raw)
}

/// Keeps the rows that are positions.
///
/// `/fapi/v3/positionRisk` opens a zero-amount row for any symbol that merely
/// has a resting order on it, and reporting one as a position contradicts what
/// [`Client::positions`](crate::Client::positions) promises. Measured
/// 2026-07-31 on a funded USD-M account: with one resting XRPUSDT limit order
/// the endpoint returned one row with `positionAmt` `0.0`, and cancelling the
/// order emptied it again. Nothing else moved, so a resting order is the whole
/// of the trigger, and an empty account never shows it.
///
/// Dropping the row costs a caller nothing. What it carried beyond the mark
/// price, which is public, was that an order rests on the symbol, and
/// `open_orders` says that outright. Hyperliquid already answers an empty list
/// for a market it holds no position on, so this is the venues agreeing rather
/// than Binance being singled out.
///
/// A row that failed to parse is kept, so it is reported rather than hidden by
/// the filter.
fn open_positions(adapter: &BinanceAdapter, raw: &[RawPosition]) -> Result<Vec<Position>> {
    raw.iter()
        .map(|raw| position(adapter, raw))
        .filter(|position| !matches!(position, Ok(position) if position.is_flat()))
        .collect()
}

fn position(adapter: &BinanceAdapter, raw: &RawPosition) -> Result<Position> {
    let signed_quantity = parse::decimal(&raw.position_amt, "positionAmt")?;

    Ok(Position {
        market: adapter.market(&raw.symbol)?,
        // The sign of the size is the only place the direction is stated:
        // `positionSide` reads `BOTH` on a one-way account whichever way the
        // position points.
        side: if signed_quantity.is_zero() {
            None
        } else if signed_quantity.is_sign_negative() {
            Some(Side::Sell)
        } else {
            Some(Side::Buy)
        },
        quantity: signed_quantity.abs(),
        entry_price: parse::decimal_or_none(&raw.entry_price, "entryPrice")?,
        mark_price: parse::decimal_or_none(&raw.mark_price, "markPrice")?,
        notional: Some(parse::decimal(&raw.notional, "notional")?.abs()),
        unrealized_pnl: Some(parse::decimal(&raw.unrealized_profit, "unRealizedProfit")?),
        // `/fapi/v3/positionRisk` carries neither. Binance keeps a symbol's
        // configured leverage and margin mode on `/fapi/v1/symbolConfig`, which
        // this does not read: it answers per symbol rather than per position,
        // and fetching it would double the weight of every `positions()` call
        // for two fields most callers never look at. Reporting them as unknown
        // beats reporting a stale guess, and `None` is not `1`.
        leverage: None,
        margin_mode: None,
    })
}

pub(super) async fn margin_summary(adapter: &BinanceAdapter) -> Result<MarginSummary> {
    check_futures_only(adapter, Feature::Margin)?;
    let body = adapter.send(balances_request(adapter)?).await?;
    margin_summary_of(&parse::json(&body, "account")?)
}

/// Maps the USD-M account totals onto the common margin summary.
///
/// Split out from the call so the mapping can be checked against a payload
/// without a network, which is where the three figures are easiest to confuse:
/// Binance names two of them after "margin" and means something different by
/// each.
fn margin_summary_of(raw: &RawFuturesAccount) -> Result<MarginSummary> {
    Ok(MarginSummary {
        // Binance denominates the USD-M account totals in USDT, and values
        // every other margin asset into it before adding them up.
        asset: "USDT".to_string(),
        // Binance's `totalMarginBalance` is wallet balance plus unrealized
        // profit and loss, which is what `equity` means, not what
        // `margin_balance` means, despite sharing a word with it.
        equity: Some(parse::decimal(
            &raw.total_margin_balance,
            "totalMarginBalance",
        )?),
        // What is actually posted against open positions and orders.
        // `totalWalletBalance` is the whole account and would make a
        // free-margin ratio come out against the wrong denominator.
        margin_balance: Some(parse::decimal(
            &raw.total_initial_margin,
            "totalInitialMargin",
        )?),
        available_balance: Some(parse::decimal(&raw.available_balance, "availableBalance")?),
    })
}

pub(super) async fn funding_rates(
    adapter: &BinanceAdapter,
    request: &HistoryRequest,
) -> Result<Page<FundingRate>> {
    let body = adapter
        .send(funding_rates_request(adapter, request)?)
        .await?;
    let raw: Vec<RawFundingRate> = parse::json(&body, "fundingRate")?;

    let items = raw
        .iter()
        .map(|entry| {
            Ok(FundingRate {
                market: request.market.clone(),
                timestamp: parse::millis(entry.funding_time),
                rate: parse::decimal(&entry.funding_rate, "fundingRate")?,
                mark_price: entry
                    .mark_price
                    .as_deref()
                    .map(|price| parse::decimal_or_none(price, "markPrice"))
                    .transpose()?
                    .flatten(),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Page {
        next: next_cursor(
            request,
            items.len(),
            raw.last().map(|entry| entry.funding_time),
        ),
        items,
    })
}

pub(super) async fn funding_payments(
    adapter: &BinanceAdapter,
    request: &HistoryRequest,
) -> Result<Page<FundingPayment>> {
    let body = adapter
        .send(funding_payments_request(adapter, request)?)
        .await?;
    let raw: Vec<RawIncome> = parse::json(&body, "income")?;

    let items = raw
        .iter()
        .map(|entry| {
            Ok(FundingPayment {
                market: request.market.clone(),
                timestamp: parse::millis(entry.time),
                amount: parse::decimal(&entry.income, "income")?,
                // The income ledger records what was charged, never the rate it
                // was charged at; read `funding_rates` for that.
                rate: None,
                id: Some(entry.tran_id.to_string()),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Page {
        next: next_cursor(request, items.len(), raw.last().map(|entry| entry.time)),
        items,
    })
}

/// The cursor for the page after this one, or `None` at the end of the history.
///
/// A short page means the window is exhausted. A full one might be, and there
/// is no way to know without asking, so a cursor is offered and the next call
/// answers it. That costs one wasted request at the end of a history.
fn next_cursor(
    request: &HistoryRequest,
    returned: usize,
    last_millis: Option<i64>,
) -> Option<Cursor> {
    let limit = request.limit.unwrap_or(DEFAULT_HISTORY_LIMIT) as usize;
    if returned < limit {
        return None;
    }
    // Resume one millisecond past the last entry: `startTime` is inclusive, so
    // resuming at it would return the last entry again.
    last_millis.map(|millis| encode_cursor(millis.saturating_add(1)))
}

pub(super) async fn set_margin(adapter: &BinanceAdapter, request: &MarginRequest) -> Result<()> {
    // Binance changes leverage and margin mode through separate endpoints;
    // there is no way to make the pair atomic, so the first failure stops the
    // second rather than leaving both half-applied silently.
    for request in set_margin_requests(adapter, request)? {
        adapter.send(request).await?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Binance-shaped reads
// ---------------------------------------------------------------------------

/// One spot order, with the fields the common [`Order`] has no room for.
///
/// Two things keep this off the common API. `maxt` reads *open* orders
/// everywhere, and a lookup by id that also answers for a filled or cancelled
/// order exists on Binance but not on every exchange behind the common API.
/// The extra fields also have no counterpart on the other venues: Binance's own
/// order type spelling, its client order id, and the quote total it
/// accumulated.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceSpotOrderDetail {
    /// The order in `maxt` terms.
    pub order: Order,
    /// The identifier the caller supplied, or the one Binance generated.
    pub client_order_id: String,
    /// Binance's own order type, for example `LIMIT_MAKER` or `STOP_LOSS`.
    ///
    /// Includes the types [`OrderType`] does not model.
    pub order_type: String,
    /// Binance's own time in force, for example `GTC` or `FOK`.
    pub time_in_force: String,
    /// Total quote asset spent or received so far.
    pub filled_quote_quantity: Decimal,
    /// When Binance last changed the order.
    pub updated_at: Option<Timestamp>,
}

pub(super) async fn spot_order(
    adapter: &BinanceAdapter,
    market: &Market,
    order_id: &str,
) -> Result<BinanceSpotOrderDetail> {
    if adapter.venue() != BinanceMarket::Spot {
        return Err(Error::unsupported(
            Feature::OpenOrders,
            EXCHANGE,
            "this lookup reads a spot order; build the adapter with `spot`",
        ));
    }
    check_order_id(order_id)?;

    let request = signed(
        adapter,
        HttpMethod::Get,
        "/api/v3/order",
        vec![
            ("symbol", adapter.symbol(market)?),
            ("orderId", order_id.to_string()),
        ],
    )?;
    let body = adapter.send(request).await?;
    let raw: RawSpotOrderDetail = parse::json(&body, "order")?;

    Ok(BinanceSpotOrderDetail {
        order: parse::order(market, &raw.order)?,
        client_order_id: raw.client_order_id,
        order_type: raw.order_type,
        time_in_force: raw.time_in_force,
        filled_quote_quantity: parse::decimal(&raw.cummulative_quote_qty, "cummulativeQuoteQty")?,
        updated_at: raw.update_time.map(parse::millis),
    })
}

/// The token that opens a USD-M user data stream.
///
/// USD-M authenticates its user data stream by URL. The key is created over
/// REST and then becomes part of the socket address, which makes it a bearer
/// secret. [`Debug`] redacts it, so it does not reach a log through a `{:?}` on
/// the struct that holds it.
///
/// A key expires sixty minutes after it was created or last extended. Keep it
/// alive with [`BinanceAdapter::usd_m_keepalive_listen_key`], or let
/// [`Client::subscribe_account`](crate::Client::subscribe_account) manage its
/// own.
///
/// Spot never mints one. Its listen key endpoints were removed on
/// 2026-02-20 07:00 UTC and it subscribes over the WebSocket API instead.
#[derive(Clone, PartialEq, Eq)]
pub struct BinanceListenKey(String);

impl BinanceListenKey {
    /// The key itself, for building a stream URL by hand.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for BinanceListenKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BinanceListenKey")
            .field(&"<redacted>")
            .finish()
    }
}

/// Creates the USD-M listen key, or extends the account's existing one.
///
/// Binance answers `POST` with the account's current key when it already has
/// one, and pushes its expiry another sixty minutes out either way. The same
/// call therefore serves as the keepalive below.
///
/// Reached on USD-M only. Every caller checks the venue first:
/// [`BinanceAdapter::usd_m_create_listen_key`] through `check_usd_m`, and
/// `stream::subscribe_account` by dispatching spot to the WebSocket API.
pub(super) async fn create_listen_key(adapter: &BinanceAdapter) -> Result<BinanceListenKey> {
    let request = api_key_only(adapter, HttpMethod::Post, USD_M_LISTEN_KEY_PATH, &[])?;
    let body = adapter.send(request).await?;
    let raw: RawListenKey = parse::json(&body, "listenKey")?;

    if raw.listen_key.trim().is_empty() {
        return Err(Error::decode("binance returned an empty listen key"));
    }
    Ok(BinanceListenKey(raw.listen_key))
}

/// Extends the USD-M listen key the API key currently owns.
///
/// `key` names the stream being kept alive and is not sent: USD-M extends
/// whichever key the API key owns and rejects a `listenKey` parameter.
pub(super) fn keepalive_listen_key_request(
    adapter: &BinanceAdapter,
    _key: &BinanceListenKey,
) -> Result<HttpRequest> {
    api_key_only(adapter, HttpMethod::Put, USD_M_LISTEN_KEY_PATH, &[])
}

pub(super) async fn keepalive_listen_key(
    adapter: &BinanceAdapter,
    key: &BinanceListenKey,
) -> Result<()> {
    let request = keepalive_listen_key_request(adapter, key)?;
    adapter.send(request).await.map(|_| ())
}

pub(super) async fn close_listen_key(
    adapter: &BinanceAdapter,
    key: &BinanceListenKey,
) -> Result<()> {
    let request = api_key_only(
        adapter,
        HttpMethod::Delete,
        USD_M_LISTEN_KEY_PATH,
        &[("listenKey", key.0.clone())],
    )?;
    adapter.send(request).await.map(|_| ())
}

/// The user data events USD-M is asked for, slash-separated as Binance's own
/// example spells them.
///
/// The filter is exhaustive, measured 2026-07-31: a socket whose `events` named
/// only the two order and balance events was told nothing when the key behind
/// it was deleted, while sockets naming `listenKeyExpired`, and one naming no
/// `events` at all, all received the event. So an event `maxt` acts on and does
/// not name here is an event `maxt` can never receive.
///
/// Every event USD-M publishes, and why it is here or is not:
///
/// | Event | Named | Why |
/// | --- | --- | --- |
/// | `ORDER_TRADE_UPDATE` | yes | an order change, read into [`AccountEvent::Order`](crate::AccountEvent::Order) |
/// | `ACCOUNT_UPDATE` | yes | a balance change, read into [`AccountEvent::Balance`](crate::AccountEvent::Balance) |
/// | `listenKeyExpired` | yes | the key behind this socket lapsed, raised as an [`Error::Exchange`](crate::Error::Exchange) so the consumer stops waiting on a stream that has nothing left to say |
/// | `TRADE_LITE` | no | the same fill `ORDER_TRADE_UPDATE` already carries, sooner and with fewer fields. Naming it would report one fill twice, and the fields `maxt` reads are the ones it drops |
/// | `MARGIN_CALL` | no | risk guidance, which Binance's own page says not to trade on. `maxt` has no type for it and would drop it |
/// | `ACCOUNT_CONFIG_UPDATE` | no | leverage and multi-assets mode. `maxt` reports neither: `Position::leverage` and `margin_mode` are always `None` |
/// | `CONDITIONAL_ORDER_TRIGGER_REJECT` | no | a rejected TP/SL trigger, and `maxt` places no conditional orders |
/// | `STRATEGY_UPDATE` | no | Binance's own grid strategies, which `maxt` does not create |
/// | `GRID_UPDATE` | no | a sub-order of one of those strategies, and deprecated on Binance's page |
/// | `ALGO_UPDATE` | no | an algo order, and `maxt` places none |
///
/// The five `maxt` does not name are all dropped by `stream::decode_account`
/// today, so naming them would buy frames nothing reads. What the ones named
/// buy is pinned by `stream::the_usd_m_events_filter_names_every_frame_the_decoder_acts_on`.
pub(super) const USD_M_ACCOUNT_EVENTS: &str = "ORDER_TRADE_UPDATE/ACCOUNT_UPDATE/listenKeyExpired";

/// The URL a USD-M user data stream opens at.
///
/// The key rides in the query under the `/private` entry point, because Binance
/// decommissioned the unrouted `/ws` path on 2026-04-23 and states that a
/// connection naming no entry point stops receiving `/private` channels, in the
/// same sentence that says it stops receiving `/market` ones.
///
/// What is verified and what is not, precisely. Everything below was measured
/// on 2026-07-31 against a live USD-M account, by opening sockets on one shared
/// key and then sending `DELETE /fapi/v1/listenKey`, which is the one push a
/// socket produces on an account that holds nothing and trades nothing:
///
/// | Claim | Standing |
/// | --- | --- |
/// | The `/market` half of that sentence is true | measured; see `stream::entry_point_url` for frame counts |
/// | Binance publishes this `/private` form | quoted from the change notice's own worked example |
/// | This URL carries that account's events | **measured.** Deleting the key pushed `listenKeyExpired` down this exact URL, so the socket was carrying the account rather than merely open |
/// | The unrouted `/ws/<key>` path is dead | **measured.** The same key on `wss://fstream.binance.com/ws/<key>` was told nothing across the same deletion |
/// | `events` filters, and filters exhaustively | **measured**, four sockets on one key. No `events` parameter: received. `events=listenKeyExpired`: received. `events=ORDER_TRADE_UPDATE/ACCOUNT_UPDATE/listenKeyExpired`: received. `events=ORDER_TRADE_UPDATE/ACCOUNT_UPDATE`: **not** received |
/// | `ORDER_TRADE_UPDATE` and `ACCOUNT_UPDATE` arrive | **not tested.** The account holds no balance and nothing was traded, so neither has ever been observed. What the deletion proves is that this socket delivers the events its filter names |
///
/// The last row is why the filter is treated as exhaustive rather than as a
/// hint: the one event that could be provoked was provoked, and the filter
/// decided it.
///
/// A lapsed key is now reported twice over, and both are wanted.
/// `stream::refresh_listen_key` extends the key over REST and sends its own
/// failure down the stream, which covers a refresher that cannot reach Binance;
/// the event covers a key invalidated some other way, which no REST call of
/// `maxt`'s would notice.
pub(super) fn usd_m_user_data_stream_url(key: &BinanceListenKey) -> String {
    // The slash between event names is a separator, so it stays literal; only
    // the key is encoded.
    format!(
        "wss://fstream.binance.com/private/ws?listenKey={}&events={USD_M_ACCOUNT_EVENTS}",
        encode(&key.0)
    )
}

/// The request that subscribes a spot WebSocket API socket to its own account.
///
/// Binance removed the spot listen key on 2026-02-20 07:00 UTC, so there is no
/// URL to authenticate by any more: the socket opens unauthenticated at
/// [`SPOT_WEBSOCKET_API_URL`](super::SPOT_WEBSOCKET_API_URL) and this frame
/// names the account.
///
/// Two methods reach the same subscription and only one of them is open to this
/// key, measured 2026-07-31:
///
/// | Method | Result with this HMAC-SHA-256 key |
/// | --- | --- |
/// | `session.logon` then `userDataStream.subscribe` | `-2028 HMAC-SHA-256 API key is not supported`, then `-1193 WebSocket session not authenticated` |
/// | `userDataStream.subscribe.signature` | `{"status":200,"result":{"subscriptionId":0}}` |
///
/// So the per-request signature is the path, and session authentication stays
/// an Ed25519-only alternative `maxt` does not need.
///
/// The frame is signed over `params` sorted by name and carries an `id`. Both
/// are load-bearing: a frame with no `id` is answered by closing the connection
/// with code 1006, measured 2026-07-31.
///
/// Called once per handshake, so a reconnect subscribes with a signature minted
/// for it rather than the one the first socket used. One signature is only good
/// for `recvWindow`, and a reconnect loop re-sending an expired one opens socket
/// after socket that carries nothing: measured 2026-07-31, a frame replayed onto
/// a socket opened 75 s later was refused `-1021` where a freshly signed one was
/// answered `{"status":200,"result":{"subscriptionId":0}}`.
///
/// `recvWindow` still sets how much of a reconnect's own latency one signature
/// covers. Measured 2026-07-31 with a frame signed 50 s earlier:
///
/// | `recvWindow` | Answer |
/// | --- | --- |
/// | absent, so Binance's own 5 000 ms | `-1021 Timestamp for this request is outside of the recvWindow` |
/// | `60000` | `{"status":200,"result":{"subscriptionId":0}}` |
///
/// So it is set to the documented maximum, which is a twelvefold widening for
/// one field. Past it the frame is refused whatever this says: the same 60 000
/// against a frame signed 90 s earlier was `-1021` too. A stream that
/// reconnects after longer reports that refusal rather than resuming, which is
/// the outcome to subscribe again on.
pub(super) fn spot_user_data_subscribe_frame(adapter: &BinanceAdapter) -> Result<String> {
    let credentials = adapter.credentials()?;
    let timestamp = now_millis();
    // Sorted by parameter name, which is the order Binance signs, and the
    // values go in unencoded: this is a JSON body, not a query string.
    let payload = format!(
        "apiKey={}&recvWindow={SPOT_SUBSCRIBE_RECV_WINDOW_MS}&timestamp={timestamp}",
        credentials.api_key
    );

    Ok(serde_json::json!({
        "id": timestamp.to_string(),
        "method": "userDataStream.subscribe.signature",
        "params": {
            "apiKey": credentials.api_key,
            "recvWindow": SPOT_SUBSCRIBE_RECV_WINDOW_MS,
            "timestamp": timestamp,
            "signature": signature(credentials, &payload)?,
        },
    })
    .to_string())
}

/// How long Binance will still accept the spot subscribe frame's timestamp.
///
/// Binance's documented maximum. Nothing here benefits from a shorter one. The
/// frame is signed again per handshake, so this is not what bounds an outage any
/// more; what it bounds is the gap between signing a frame and Binance reading
/// it, and a shorter window only makes a slow handshake fail for no gain.
const SPOT_SUBSCRIBE_RECV_WINDOW_MS: u64 = 60_000;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, MarketKind, OrderStatus};

    /// The key, query, and signature Binance publishes as its worked example.
    ///
    /// https://developers.binance.com/docs/binance-spot-api-docs/rest-api/endpoint-security-type
    const DOC_SECRET_KEY: &str = "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j";
    const DOC_QUERY: &str = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559";
    const DOC_SIGNATURE: &str = "c8db56825ae71d6d79447849e617115f4a920fa2acdcab2b053c4b2838bd6b71";

    fn spot() -> BinanceAdapter {
        BinanceAdapter::spot().with_credentials("key", "secret")
    }

    fn perp() -> BinanceAdapter {
        BinanceAdapter::usd_m_futures().with_credentials("key", "secret")
    }

    fn btc_usdt() -> Market {
        Market::spot(Exchange::Binance, "BTC", "USDT")
    }

    fn btc_usdt_perp() -> Market {
        Market::perpetual(Exchange::Binance, "BTC", "USDT")
    }

    /// The query of a request, with the signature stripped off.
    ///
    /// The signature covers a timestamp read from the clock, so it differs on
    /// every run; the parameters before it are what the test is about.
    fn signed_params(request: &HttpRequest) -> String {
        let query = request.target();
        let (params, _) = query.split_once("&signature=").expect("a signed query");
        // The timestamp is the clock's, so it is dropped too.
        let (params, _) = params.rsplit_once("&timestamp=").expect("a stamped query");
        params.to_string()
    }

    #[test]
    fn the_signature_matches_binances_own_worked_example() {
        let credentials = BinanceCredentials {
            api_key: "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A".to_string(),
            secret_key: DOC_SECRET_KEY.to_string(),
        };

        let signed = sign(&credentials, DOC_QUERY).expect("a signature");

        assert_eq!(signed, format!("{DOC_QUERY}&signature={DOC_SIGNATURE}"));
    }

    #[test]
    fn an_empty_query_still_carries_a_signature() {
        let credentials = BinanceCredentials {
            api_key: "key".to_string(),
            secret_key: DOC_SECRET_KEY.to_string(),
        };

        let signed = sign(&credentials, "").expect("a signature");

        assert!(signed.starts_with("signature="));
        assert!(!signed.starts_with("signature=&"));
        assert_eq!(signed.len(), "signature=".len() + 64);
    }

    #[test]
    fn every_signed_request_carries_the_api_key_header() {
        let request = balances_request(&spot()).expect("credentials are set");

        assert!(
            request
                .headers
                .iter()
                .any(|(name, value)| name == API_KEY_HEADER && value == "key")
        );
        assert!(request.target().starts_with("/api/v3/account?"));
        assert!(request.target().contains("&signature="));
    }

    #[test]
    fn an_unauthenticated_adapter_never_builds_a_private_request() {
        let public = BinanceAdapter::spot();

        assert!(matches!(balances_request(&public), Err(Error::Auth { .. })));
        assert!(matches!(
            open_orders_request(&public, None),
            Err(Error::Auth { .. })
        ));
    }

    #[test]
    fn the_two_venues_read_the_same_questions_at_different_paths() {
        assert_eq!(account_path(BinanceMarket::Spot), "/api/v3/account");
        assert_eq!(account_path(BinanceMarket::UsdMFutures), "/fapi/v3/account");
        assert_eq!(open_orders_path(BinanceMarket::Spot), "/api/v3/openOrders");
        assert_eq!(
            open_orders_path(BinanceMarket::UsdMFutures),
            "/fapi/v1/openOrders"
        );
        assert!(
            balances_request(&perp())
                .expect("credentials are set")
                .target()
                .starts_with("/fapi/v3/account?")
        );
        assert!(
            open_orders_request(&perp(), Some(&btc_usdt_perp()))
                .expect("a futures market")
                .target()
                .starts_with("/fapi/v1/openOrders?symbol=BTCUSDT")
        );
    }

    #[test]
    fn a_spot_market_buy_can_be_sized_in_the_quote_asset_and_a_futures_one_cannot() {
        let quote_sized =
            OrderRequest::market(btc_usdt(), Side::Buy, Size::Quote(Decimal::from(10_000)));
        let futures_quote_sized = OrderRequest::market(
            btc_usdt_perp(),
            Side::Buy,
            Size::Quote(Decimal::from(10_000)),
        );

        assert_eq!(
            signed_params(&place_order_request(&spot(), &quote_sized).expect("a spot order")),
            "/api/v3/order?symbol=BTCUSDT&side=BUY&type=MARKET&quoteOrderQty=10000&newOrderRespType=RESULT"
        );
        assert!(matches!(
            place_order_request(&perp(), &futures_quote_sized),
            Err(Error::InvalidRequest { field: "size", .. })
        ));
    }

    #[test]
    fn post_only_is_an_order_type_on_spot_and_a_time_in_force_on_futures() {
        let spot_order = OrderRequest::limit(
            btc_usdt(),
            Side::Sell,
            Size::Base(Decimal::new(1, 2)),
            Decimal::from(100_000),
        )
        .time_in_force(TimeInForce::PostOnly);
        let futures_order = OrderRequest::limit(
            btc_usdt_perp(),
            Side::Sell,
            Size::Base(Decimal::new(1, 2)),
            Decimal::from(100_000),
        )
        .time_in_force(TimeInForce::PostOnly);

        assert_eq!(
            signed_params(&place_order_request(&spot(), &spot_order).expect("a spot order")),
            "/api/v3/order?symbol=BTCUSDT&side=SELL&type=LIMIT_MAKER&quantity=0.01&price=100000&newOrderRespType=RESULT"
        );
        assert_eq!(
            signed_params(&place_order_request(&perp(), &futures_order).expect("a futures order")),
            "/fapi/v1/order?symbol=BTCUSDT&side=SELL&type=LIMIT&quantity=0.01&price=100000&timeInForce=GTX&newOrderRespType=RESULT"
        );
    }

    #[test]
    fn a_limit_order_defaults_to_the_time_in_force_binance_itself_defaults_to() {
        let order = OrderRequest::limit(
            btc_usdt(),
            Side::Buy,
            Size::Base(Decimal::ONE),
            Decimal::from(50_000),
        );

        assert!(
            signed_params(&place_order_request(&spot(), &order).expect("a spot order"))
                .contains("&timeInForce=GTC")
        );
        assert_eq!(
            time_in_force_code(BinanceMarket::Spot, Some(TimeInForce::FillOrKill)),
            Some("FOK")
        );
        assert_eq!(
            time_in_force_code(BinanceMarket::Spot, Some(TimeInForce::ImmediateOrCancel)),
            Some("IOC")
        );
    }

    #[test]
    fn a_market_order_carries_no_time_in_force() {
        let order = OrderRequest::market(btc_usdt(), Side::Buy, Size::Base(Decimal::ONE));

        assert!(
            !signed_params(&place_order_request(&spot(), &order).expect("a spot order"))
                .contains("timeInForce")
        );
        assert!(matches!(
            place_order_request(&spot(), &order.clone().time_in_force(TimeInForce::PostOnly)),
            Err(Error::InvalidRequest {
                field: "time_in_force",
                ..
            })
        ));
    }

    #[test]
    fn reduce_only_reaches_futures_and_is_refused_on_spot() {
        let futures_order =
            OrderRequest::market(btc_usdt_perp(), Side::Sell, Size::Base(Decimal::ONE))
                .reduce_only();
        let spot_order =
            OrderRequest::market(btc_usdt(), Side::Sell, Size::Base(Decimal::ONE)).reduce_only();

        assert_eq!(
            signed_params(&place_order_request(&perp(), &futures_order).expect("a futures order")),
            "/fapi/v1/order?symbol=BTCUSDT&side=SELL&type=MARKET&quantity=1&reduceOnly=true&newOrderRespType=RESULT"
        );
        assert!(matches!(
            place_order_request(&spot(), &spot_order),
            Err(Error::Unsupported {
                feature: Feature::ReduceOnlyOrders,
                ..
            })
        ));
    }

    #[test]
    fn an_order_id_that_is_not_binances_never_reaches_a_cancel() {
        assert_eq!(
            signed_params(&cancel_order_request(&spot(), &btc_usdt(), "28").expect("a numeric id")),
            "/api/v3/order?symbol=BTCUSDT&orderId=28"
        );
        for bad in ["", "abc", "28&symbol=ETHUSDT", "-1", "2 8"] {
            assert!(
                matches!(
                    cancel_order_request(&spot(), &btc_usdt(), bad),
                    Err(Error::InvalidRequest {
                        field: "order_id",
                        ..
                    })
                ),
                "{bad}"
            );
        }
    }

    #[test]
    fn every_derivatives_call_is_refused_on_a_spot_adapter() {
        let history = HistoryRequest::new(btc_usdt());
        let margin = MarginRequest::new(btc_usdt()).leverage(Decimal::from(10));

        assert!(matches!(
            positions_request(&spot(), None),
            Err(Error::Unsupported {
                feature: Feature::Positions,
                ..
            })
        ));
        assert!(matches!(
            funding_rates_request(&spot(), &history),
            Err(Error::Unsupported {
                feature: Feature::FundingRates,
                ..
            })
        ));
        assert!(matches!(
            funding_payments_request(&spot(), &history),
            Err(Error::Unsupported {
                feature: Feature::FundingPayments,
                ..
            })
        ));
        assert!(matches!(
            set_margin_requests(&spot(), &margin),
            Err(Error::Unsupported {
                feature: Feature::MarginConfig,
                ..
            })
        ));
    }

    #[test]
    fn funding_rate_history_is_public_and_funding_payments_are_not() {
        let request = HistoryRequest::new(btc_usdt_perp())
            .from(Timestamp::from_millis(1_570_608_000_000))
            .to(Timestamp::from_millis(1_570_636_800_000))
            .limit(1_000);

        let rates = funding_rates_request(&perp(), &request).expect("a futures market");
        let payments = funding_payments_request(&perp(), &request).expect("a futures market");

        assert_eq!(
            rates.target(),
            "/fapi/v1/fundingRate?symbol=BTCUSDT&startTime=1570608000000&endTime=1570636799999&limit=1000"
        );
        assert!(rates.headers.is_empty());
        assert!(
            signed_params(&payments)
                .starts_with("/fapi/v1/income?symbol=BTCUSDT&incomeType=FUNDING_FEE")
        );
        assert!(
            payments
                .headers
                .iter()
                .any(|(name, _)| name == API_KEY_HEADER)
        );
    }

    #[test]
    fn a_cursor_supersedes_the_requested_start_and_resumes_past_the_last_entry() {
        let resumed = HistoryRequest::new(btc_usdt_perp())
            .from(Timestamp::from_millis(1))
            .cursor(encode_cursor(1_570_636_800_001));

        assert_eq!(
            funding_rates_request(&perp(), &resumed)
                .expect("a futures market")
                .target(),
            "/fapi/v1/fundingRate?symbol=BTCUSDT&startTime=1570636800001&limit=100"
        );
        assert!(matches!(
            funding_rates_request(
                &perp(),
                &HistoryRequest::new(btc_usdt_perp()).cursor(Cursor("page-2".to_string()))
            ),
            Err(Error::InvalidRequest {
                field: "cursor",
                ..
            })
        ));
    }

    #[test]
    fn a_short_page_ends_the_history_and_a_full_one_offers_another() {
        let request = HistoryRequest::new(btc_usdt_perp()).limit(2);

        assert_eq!(next_cursor(&request, 1, Some(1_000)), None);
        assert_eq!(
            next_cursor(&request, 2, Some(1_000)),
            Some(encode_cursor(1_001))
        );
        // The default limit applies when the caller states none.
        let defaulted = HistoryRequest::new(btc_usdt_perp());
        assert_eq!(next_cursor(&defaulted, 99, Some(1_000)), None);
        assert_eq!(
            next_cursor(&defaulted, 100, Some(1_000)),
            Some(encode_cursor(1_001))
        );
    }

    #[test]
    fn a_history_page_larger_than_binance_serves_is_refused() {
        let request = HistoryRequest::new(btc_usdt_perp()).limit(1_001);

        assert!(matches!(
            funding_rates_request(&perp(), &request),
            Err(Error::InvalidRequest { field: "limit", .. })
        ));
    }

    #[test]
    fn margin_changes_go_to_one_endpoint_each() {
        let both = MarginRequest::new(btc_usdt_perp())
            .leverage(Decimal::from(10))
            .margin_mode(MarginMode::Cross);

        let requests = set_margin_requests(&perp(), &both).expect("a futures market");

        assert_eq!(requests.len(), 2);
        assert_eq!(
            signed_params(&requests[0]),
            "/fapi/v1/leverage?symbol=BTCUSDT&leverage=10"
        );
        // Binance spells cross margin `CROSSED`.
        assert_eq!(
            signed_params(&requests[1]),
            "/fapi/v1/marginType?symbol=BTCUSDT&marginType=CROSSED"
        );
    }

    #[test]
    fn a_margin_request_that_changes_nothing_is_a_caller_mistake() {
        assert!(matches!(
            set_margin_requests(&perp(), &MarginRequest::new(btc_usdt_perp())),
            Err(Error::InvalidRequest {
                field: "leverage",
                ..
            })
        ));
        assert!(matches!(
            set_margin_requests(
                &perp(),
                &MarginRequest::new(btc_usdt_perp()).leverage(Decimal::new(15, 1))
            ),
            Err(Error::InvalidRequest {
                field: "leverage",
                ..
            })
        ));
        assert_eq!(
            leverage_code(Decimal::from(125)).expect("a multiplier"),
            "125"
        );
    }

    #[test]
    fn a_futures_balance_splits_the_wallet_into_free_and_posted() {
        // https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Account-Information-V3
        let raw: RawFuturesAccount = parse::json(
            r#"{
              "totalInitialMargin": "0.00000000",
              "totalMaintMargin": "0.00000000",
              "totalWalletBalance": "126.72469206",
              "totalUnrealizedProfit": "0.00000000",
              "totalMarginBalance": "126.72469206",
              "availableBalance": "100.12345678",
              "assets": [
                {
                  "asset": "USDT",
                  "walletBalance": "126.72469206",
                  "unrealizedProfit": "0.00000000",
                  "marginBalance": "126.72469206",
                  "availableBalance": "100.12345678",
                  "updateTime": 1625474304765
                }
              ],
              "positions": []
            }"#,
            "account",
        )
        .expect("official account payload");

        let balance = futures_balance(&raw.assets[0]).expect("a balance");

        assert_eq!(balance.asset, "USDT");
        assert_eq!(balance.available.to_string(), "100.12345678");
        assert_eq!(balance.locked.to_string(), "26.60123528");
        assert_eq!(balance.total().to_string(), "126.72469206");
    }

    #[test]
    fn a_margin_summary_reads_each_figure_off_the_field_that_means_it() {
        // https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Account-Information-V3
        //
        // Chosen so the four totals are all different: an account holding 1,000
        // USDT, 250 of it posted against open positions, 12.5 of unrealized
        // profit on top, leaving 750 free. Binance names three of them and the
        // right one for "posted as margin" is `totalInitialMargin`, the same
        // quantity Hyperliquid calls `totalMarginUsed`.
        let raw: RawFuturesAccount = parse::json(
            r#"{
              "totalInitialMargin": "250.00000000",
              "totalMaintMargin": "50.00000000",
              "totalWalletBalance": "1000.00000000",
              "totalUnrealizedProfit": "12.50000000",
              "totalMarginBalance": "1012.50000000",
              "availableBalance": "750.00000000",
              "assets": [],
              "positions": []
            }"#,
            "account",
        )
        .expect("official account payload");

        let summary = margin_summary_of(&raw).expect("a summary");

        assert_eq!(summary.asset, "USDT");
        // Wallet plus unrealized profit and loss.
        assert_eq!(summary.equity, Some(Decimal::new(101_250, 2)));
        // What is posted, not what is held: the wallet balance is 1000.
        assert_eq!(summary.margin_balance, Some(Decimal::from(250)));
        assert_eq!(summary.available_balance, Some(Decimal::from(750)));
        // A free-margin ratio computed off these is the one a caller expects.
        assert_eq!(
            summary
                .equity
                .zip(summary.margin_balance)
                .map(|(e, m)| e - m),
            Some(Decimal::new(76_250, 2))
        );
    }

    #[test]
    fn a_position_takes_its_direction_from_the_sign_of_its_size() {
        // https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Position-Information-V3
        let raw: Vec<RawPosition> = parse::json(
            r#"[
              {
                "symbol": "BTCUSDT",
                "positionSide": "BOTH",
                "positionAmt": "-1.000",
                "entryPrice": "0.00000",
                "breakEvenPrice": "0.0",
                "markPrice": "6679.50671178",
                "unRealizedProfit": "0.00000000",
                "liquidationPrice": "0",
                "isolatedMargin": "0.00000000",
                "notional": "-6679.50671178",
                "marginAsset": "USDT",
                "isolatedWallet": "0",
                "initialMargin": "0",
                "maintMargin": "0",
                "positionInitialMargin": "0",
                "openOrderInitialMargin": "0",
                "adl": 0,
                "bidNotional": "0",
                "askNotional": "0",
                "updateTime": 0
              }
            ]"#,
            "positionRisk",
        )
        .expect("official position payload");

        let position = position(&perp(), &raw[0]).expect("a position");

        // `positionSide` reads `BOTH`; only the sign says which way it points.
        assert_eq!(position.side, Some(Side::Sell));
        assert_eq!(position.quantity.to_string(), "1.000");
        assert_eq!(
            position.notional.expect("a notional").to_string(),
            "6679.50671178"
        );
        // A zero entry price is Binance's "not applicable", not a real price.
        assert_eq!(position.entry_price, None);
        assert_eq!(position.market.kind, MarketKind::Perpetual);
        // `/fapi/v3/positionRisk` publishes neither.
        assert_eq!(position.leverage, None);
        assert_eq!(position.margin_mode, None);
        // And a position with size survives the filter that drops the
        // zero-amount rows Binance opens for a symbol carrying only an order.
        assert_eq!(
            open_positions(&perp(), &raw)
                .expect("a position list")
                .len(),
            1
        );
    }

    /// Captured 2026-07-31 off `GET /fapi/v3/positionRisk` on a funded USD-M
    /// account holding no position and one resting XRPUSDT limit order. The
    /// same endpoint on the same account returned `[]` once the order was
    /// cancelled, and `[]` again on a later read naming the symbol outright, so
    /// the resting order is the whole of what puts this row there.
    const POSITION_RISK_WITH_A_RESTING_ORDER: &str = r#"[{
      "symbol": "XRPUSDT",
      "positionSide": "BOTH",
      "positionAmt": "0.0",
      "entryPrice": "0.0",
      "markPrice": "1.08710784",
      "unRealizedProfit": "0.00000000",
      "notional": "0"
    }]"#;

    /// Binance opens a position row for a symbol that has only an order on it,
    /// and `positions` promises open positions. The row maps cleanly, which is
    /// why nothing upstream rejects it: it has to be dropped on the way out.
    #[test]
    fn a_symbol_with_only_a_resting_order_is_not_reported_as_a_position() {
        let raw: Vec<RawPosition> = parse::json(POSITION_RISK_WITH_A_RESTING_ORDER, "positionRisk")
            .expect("the captured payload");

        // The row is real and maps: this is not a payload that decodes to
        // nothing, so an empty answer below is the filter and not the parser.
        let mapped = position(&perp(), &raw[0]).expect("a position");
        assert!(mapped.is_flat());
        assert_eq!(mapped.side, None);
        assert_eq!(mapped.market.kind, MarketKind::Perpetual);

        assert_eq!(
            open_positions(&perp(), &raw).expect("a position list"),
            Vec::new(),
            "a symbol carrying only a resting order was reported as a position"
        );
    }

    #[test]
    fn a_spot_order_lookup_keeps_the_fields_the_common_order_has_no_room_for() {
        // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints
        let raw: RawSpotOrderDetail = parse::json(
            r#"{
              "symbol": "LTCBTC",
              "orderId": 1,
              "orderListId": -1,
              "clientOrderId": "myOrder1",
              "price": "0.1",
              "origQty": "1.0",
              "executedQty": "0.5",
              "cummulativeQuoteQty": "0.05",
              "status": "PARTIALLY_FILLED",
              "timeInForce": "GTC",
              "type": "LIMIT",
              "side": "BUY",
              "stopPrice": "0.0",
              "icebergQty": "0.0",
              "time": 1499827319559,
              "updateTime": 1499827319559,
              "isWorking": true,
              "workingTime": 1499827319559,
              "origQuoteOrderQty": "0.000000",
              "selfTradePreventionMode": "NONE"
            }"#,
            "order",
        )
        .expect("official order payload");

        let order = parse::order(&btc_usdt(), &raw.order).expect("an order");

        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert_eq!(order.filled_quantity.to_string(), "0.5");
        assert_eq!(order.remaining_quantity.to_string(), "0.5");
        assert_eq!(raw.client_order_id, "myOrder1");
        assert_eq!(raw.cummulative_quote_qty, "0.05");
        assert_eq!(raw.time_in_force, "GTC");
    }

    #[test]
    fn a_listen_key_stays_out_of_a_debug_line_and_goes_into_the_stream_url() {
        let key = BinanceListenKey(
            "pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1".to_string(),
        );

        assert_eq!(format!("{key:?}"), r#"BinanceListenKey("<redacted>")"#);
        assert!(!format!("{key:?}").contains("pqia91"));
        assert!(key.as_str().starts_with("pqia91"));
        assert_eq!(
            usd_m_user_data_stream_url(&key),
            format!(
                "wss://fstream.binance.com/private/ws?listenKey={}\
                 &events=ORDER_TRADE_UPDATE/ACCOUNT_UPDATE/listenKeyExpired",
                key.as_str()
            )
        );
        assert_eq!(USD_M_LISTEN_KEY_PATH, "/fapi/v1/listenKey");
    }

    /// Binance removed `POST`, `PUT`, and `DELETE /api/v3/userDataStream` on
    /// 2026-02-20 07:00 UTC. `POST` was measured answering `410 Gone` with an
    /// nginx error page on 2026-07-31, before a socket was ever opened, so a
    /// spot account subscription that reaches for a listen key cannot work at
    /// all.
    ///
    /// This pins that no request `maxt` builds names that path, whichever venue
    /// it is built for.
    #[test]
    fn no_request_reaches_for_the_removed_spot_listen_key_endpoints() {
        let key = BinanceListenKey("listen-key".to_string());
        let requests = [
            keepalive_listen_key_request(&spot(), &key).expect("credentials are set"),
            keepalive_listen_key_request(&perp(), &key).expect("credentials are set"),
            api_key_only(&spot(), HttpMethod::Post, USD_M_LISTEN_KEY_PATH, &[])
                .expect("credentials are set"),
        ];

        for request in &requests {
            assert!(
                !request.target().contains("/api/v3/userDataStream"),
                "{}",
                request.target()
            );
        }
        assert!(!USD_M_LISTEN_KEY_PATH.contains("userDataStream"));
    }

    /// A spot subscription is a signed request on the WebSocket API, not a URL
    /// carrying a listen key.
    ///
    /// Every field here was measured against a live HMAC-SHA-256 key on
    /// 2026-07-31: the method, because `session.logon` answers `-2028
    /// HMAC-SHA-256 API key is not supported`; the `id`, because a frame
    /// without one is answered by closing the connection with code 1006; and
    /// the signature's payload order, because Binance signs `params` sorted by
    /// name and a wrong order is answered `-1022`.
    #[test]
    fn the_spot_subscribe_frame_is_signed_over_its_parameters_in_binances_own_order() {
        let frame = spot_user_data_subscribe_frame(&spot()).expect("credentials are set");
        let parsed: serde_json::Value = serde_json::from_str(&frame).expect("a JSON frame");

        assert_eq!(parsed["method"], "userDataStream.subscribe.signature");
        assert!(parsed["id"].is_string(), "{frame}");
        assert_eq!(parsed["params"]["apiKey"], "key");

        let timestamp = parsed["params"]["timestamp"]
            .as_i64()
            .expect("a numeric timestamp");
        // The id is the timestamp, so one subscribe is told from the next in a
        // log without either being invented.
        assert_eq!(parsed["id"], timestamp.to_string());

        let credentials = BinanceCredentials {
            api_key: "key".to_string(),
            secret_key: "secret".to_string(),
        };
        // Sorted by name, and no value is percent encoded: this is a JSON body,
        // not a query string.
        let expected = signature(
            &credentials,
            &format!("apiKey=key&recvWindow=60000&timestamp={timestamp}"),
        )
        .expect("a signature");
        assert_eq!(parsed["params"]["signature"], expected);

        // A frame signed 50 s earlier is refused without this and accepted with
        // it, so it sets how long an outage the stream can reconnect through.
        assert_eq!(parsed["params"]["recvWindow"], 60_000);
    }

    #[test]
    fn an_unauthenticated_adapter_never_subscribes_to_a_spot_user_data_stream() {
        assert!(matches!(
            spot_user_data_subscribe_frame(&BinanceAdapter::spot()),
            Err(Error::Auth { .. })
        ));
    }

    /// USD-M user data moved to `/private` with the key in the query. A URL on
    /// the decommissioned unrouted path opens and is never told anything, the
    /// same way `Feed::Ticker` was, so the shape is worth pinning rather than
    /// leaving to a string literal nobody rereads.
    ///
    /// Which events the query has to name is not pinned here, because a
    /// substring assertion cannot tell a complete filter from a truncated one.
    /// `stream::the_usd_m_events_filter_names_every_frame_the_decoder_acts_on`
    /// pins that against the decoder instead.
    #[test]
    fn the_usd_m_account_socket_names_an_entry_point_and_the_events_it_reads() {
        let key = BinanceListenKey("listen-key".to_string());
        let url = usd_m_user_data_stream_url(&key);

        assert!(
            url.starts_with("wss://fstream.binance.com/private/"),
            "{url}"
        );
        assert!(
            url.contains(&format!("&events={USD_M_ACCOUNT_EVENTS}")),
            "{url}"
        );
        // The separator is a separator, not part of an event name.
        assert!(!url.contains("%2F"), "{url}");
    }

    #[test]
    fn keepalive_uses_put_and_never_names_the_key_binance_would_reject() {
        let key = BinanceListenKey("listen-key".to_string());

        let request = keepalive_listen_key_request(&perp(), &key).expect("credentials are set");

        assert_eq!(request.method, HttpMethod::Put);
        // USD-M extends whichever key the API key owns, and sending `listenKey`
        // there is rejected.
        assert_eq!(request.target(), "/fapi/v1/listenKey");
        // Not a signed endpoint: the API key header alone authorises it.
        assert!(!request.target().contains("signature"));
        assert!(
            request
                .headers
                .iter()
                .any(|(name, _)| name == API_KEY_HEADER)
        );
    }

    #[test]
    fn a_listen_key_request_authenticates_with_the_key_alone() {
        let request = api_key_only(&perp(), HttpMethod::Post, USD_M_LISTEN_KEY_PATH, &[])
            .expect("credentials are set");

        assert_eq!(request.target(), "/fapi/v1/listenKey");
        assert!(!request.target().contains("signature"));
        assert!(
            request
                .headers
                .iter()
                .any(|(name, value)| name == API_KEY_HEADER && value == "key")
        );
    }
}
