//! Binance account, order, margin, funding, and user-data requests.
//!
//! Private calls use an API key and HMAC-SHA-256 secret. The exact encoded query
//! bytes are signed and sent unchanged. RSA and Ed25519 keys are not supported.

use hmac::{Hmac, KeyInit, Mac};
use rust_decimal::Decimal;
use serde::Deserialize;
use sha2::Sha256;

use crate::adapters::{inclusive_millis_at_or_after, inclusive_millis_before};
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{HistoryRequest, MarginRequest, OrderRequest};
use crate::transport::{HttpMethod, HttpRequest};
use crate::types::{
    Balance, Cursor, FundingPayment, FundingRate, MarginMode, MarginSummary, Market, Order,
    OrderType, Page, Position, Side, Size, TimeInForce, Timestamp,
};

use super::{
    API_KEY_HEADER, BinanceAdapter, BinanceC2cTradeHistoryRequest, BinanceCredentials,
    BinanceMarket, BinanceTestOrderRequest, EXCHANGE, decode_cursor, encode_cursor, now_millis,
    parse,
    rest::{encode, query},
};

/// The most entries either paginated history returns in one call.
const MAX_HISTORY_LIMIT: u32 = 1_000;
/// What a history page holds when the caller does not say.
const DEFAULT_HISTORY_LIMIT: u32 = 100;
/// Binance's account-trade endpoints default to 500 entries.
const DEFAULT_ACCOUNT_TRADES_LIMIT: u32 = 500;
const SPOT_ACCOUNT_TRADE_WINDOW_MILLIS: i64 = 24 * 60 * 60 * 1_000;
const USD_M_ACCOUNT_TRADE_WINDOW_MILLIS: i64 = 7 * 24 * 60 * 60 * 1_000;
const C2C_DEFAULT_PAGE: u32 = 1;
const C2C_DEFAULT_ROWS: u32 = 100;
const C2C_MAX_ROWS: u32 = 100;
const C2C_MAX_WINDOW_MILLIS: i64 = 30 * 24 * 60 * 60 * 1_000;
const C2C_MAX_RECV_WINDOW: u64 = 60_000;

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
pub(super) fn signed(
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

/// The provider-only parts of `GET /api/v3/account`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSpotAccountInformation {
    maker_commission: u64,
    taker_commission: u64,
    buyer_commission: u64,
    seller_commission: u64,
    commission_rates: RawSpotCommissionRates,
    can_trade: bool,
    can_withdraw: bool,
    can_deposit: bool,
    update_time: i64,
    account_type: String,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    uid: Option<u64>,
    #[serde(default)]
    balances: Vec<RawSpotBalance>,
}

#[derive(Debug, Deserialize)]
struct RawSpotCommissionRates {
    maker: String,
    taker: String,
    buyer: String,
    seller: String,
}

/// A Spot cancellation report. Binance may return a richer order-list shape,
/// so every ordinary-order field is optional while the raw object is retained.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSpotCancelledOrder {
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default)]
    orig_client_order_id: Option<String>,
    #[serde(default)]
    order_id: Option<i64>,
    #[serde(default)]
    client_order_id: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    price: Option<String>,
    #[serde(default)]
    orig_qty: Option<String>,
    #[serde(default)]
    executed_qty: Option<String>,
    #[serde(default)]
    cummulative_quote_qty: Option<String>,
    #[serde(default)]
    transact_time: Option<i64>,
    #[serde(default)]
    order_list_id: Option<i64>,
    #[serde(default)]
    contingency_type: Option<String>,
    #[serde(default)]
    list_status_type: Option<String>,
    #[serde(default)]
    list_order_status: Option<String>,
    #[serde(default)]
    list_client_order_id: Option<String>,
    #[serde(default)]
    transaction_time: Option<i64>,
}

/// `GET /fapi/v3/account`, including fields the common balance/margin view
/// cannot represent.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawUsdMAccountInformation {
    total_initial_margin: String,
    total_maint_margin: String,
    total_wallet_balance: String,
    total_unrealized_profit: String,
    total_margin_balance: String,
    total_position_initial_margin: String,
    total_open_order_initial_margin: String,
    total_cross_wallet_balance: String,
    total_cross_un_pnl: String,
    available_balance: String,
    max_withdraw_amount: String,
    #[serde(default)]
    assets: Vec<RawUsdMAccountAsset>,
    #[serde(default)]
    positions: Vec<RawUsdMAccountPosition>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawUsdMAccountAsset {
    asset: String,
    wallet_balance: String,
    unrealized_profit: String,
    margin_balance: String,
    maint_margin: String,
    initial_margin: String,
    position_initial_margin: String,
    open_order_initial_margin: String,
    cross_wallet_balance: String,
    cross_un_pnl: String,
    available_balance: String,
    max_withdraw_amount: String,
    update_time: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawUsdMAccountPosition {
    symbol: String,
    position_side: String,
    position_amt: String,
    unrealized_profit: String,
    isolated_margin: String,
    notional: String,
    isolated_wallet: String,
    initial_margin: String,
    maint_margin: String,
    update_time: i64,
}

/// `GET /fapi/v3/positionRisk` with its full current public field set.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawUsdMPositionInformation {
    symbol: String,
    position_side: String,
    position_amt: String,
    entry_price: String,
    break_even_price: String,
    mark_price: String,
    #[serde(rename = "unRealizedProfit")]
    unrealized_profit: String,
    liquidation_price: String,
    isolated_margin: String,
    notional: String,
    margin_asset: String,
    isolated_wallet: String,
    initial_margin: String,
    maint_margin: String,
    position_initial_margin: String,
    open_order_initial_margin: String,
    adl: u64,
    bid_notional: String,
    ask_notional: String,
    update_time: i64,
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

/// `GET /api/v3/myTrades` and `GET /fapi/v1/userTrades`.
///
/// Spot names the buyer and maker flags with an `is` prefix; USD-M does not.
/// The three fields added to USD-M user trades remain optional so Spot and
/// older captured responses keep decoding.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawAccountTrade {
    id: i64,
    order_id: i64,
    order_list_id: Option<serde_json::Number>,
    side: Option<String>,
    price: String,
    qty: String,
    quote_qty: Option<String>,
    commission: String,
    commission_asset: String,
    time: i64,
    #[serde(alias = "isBuyer")]
    buyer: bool,
    #[serde(alias = "isMaker")]
    maker: bool,
    #[serde(rename = "isBestMatch")]
    best_match: Option<bool>,
    realized_pnl: Option<String>,
    position_side: Option<String>,
    pair: Option<String>,
    base_qty: Option<String>,
    margin_asset: Option<String>,
}

/// `GET /sapi/v1/c2c/orderMatch/listUserOrderHistory`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawC2cTradeHistoryPage {
    code: Option<String>,
    message: Option<String>,
    data: Option<Vec<RawC2cTrade>>,
    total: Option<u64>,
    success: Option<bool>,
}

/// One C2C order in Binance's history response.
///
/// The current official connector marks each field optional. The public Rust
/// type follows that contract so a newly omitted optional provider field does
/// not erase the rest of a successful page.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawC2cTrade {
    order_number: Option<String>,
    adv_no: Option<String>,
    trade_type: Option<String>,
    asset: Option<String>,
    fiat: Option<String>,
    fiat_symbol: Option<String>,
    amount: Option<String>,
    total_price: Option<String>,
    unit_price: Option<String>,
    order_status: Option<String>,
    create_time: Option<i64>,
    commission: Option<String>,
    counter_part_nick_name: Option<String>,
    pay_method_name: Option<String>,
    additional_kyc_verify: Option<u32>,
    taker_commission_rate: Option<String>,
    taker_commission: Option<String>,
    taker_amount: Option<String>,
    advertisement_role: Option<String>,
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

/// A create-or-cancel response shared by Binance Spot and USD-M.
///
/// Both venues return the common order fields, while the remaining fields
/// differ by venue and order type. Optional fields retain that distinction
/// without making a Spot response pretend to be a futures response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawProviderOrderResponse {
    #[serde(flatten)]
    order: parse::RawOrder,
    #[serde(default)]
    client_order_id: Option<String>,
    #[serde(default)]
    order_list_id: Option<i64>,
    #[serde(rename = "type", default)]
    order_type: Option<String>,
    #[serde(default)]
    time_in_force: Option<String>,
    #[serde(default)]
    cummulative_quote_qty: Option<String>,
    #[serde(default)]
    cum_qty: Option<String>,
    #[serde(default)]
    cum_quote: Option<String>,
    #[serde(default)]
    avg_price: Option<String>,
    #[serde(default)]
    reduce_only: Option<bool>,
    #[serde(default)]
    close_position: Option<bool>,
    #[serde(default)]
    position_side: Option<String>,
    #[serde(default)]
    stop_price: Option<String>,
    #[serde(default)]
    working_type: Option<String>,
    #[serde(default)]
    price_protect: Option<bool>,
    #[serde(default)]
    orig_type: Option<String>,
    #[serde(default)]
    price_match: Option<String>,
    #[serde(default)]
    self_trade_prevention_mode: Option<String>,
    #[serde(default)]
    good_till_date: Option<i64>,
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

fn account_trades_path(venue: BinanceMarket) -> &'static str {
    match venue {
        BinanceMarket::Spot => "/api/v3/myTrades",
        BinanceMarket::UsdMFutures => "/fapi/v1/userTrades",
    }
}

/// The USD-M endpoint for creating, extending, and closing listen keys.
/// Spot account streams use a signed WebSocket API request instead.
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

/// Builds a signed account-trade request for the selected venue.
pub(super) fn account_trades_request(
    adapter: &BinanceAdapter,
    request: &HistoryRequest,
) -> Result<HttpRequest> {
    if request.cursor.is_some() {
        return Err(Error::invalid_request(
            "cursor",
            "Binance account trades do not expose a safe cursor yet; request the first page without one",
        ));
    }

    let limit = request.limit.unwrap_or(DEFAULT_ACCOUNT_TRADES_LIMIT);
    if !(1..=MAX_HISTORY_LIMIT).contains(&limit) {
        return Err(Error::invalid_request(
            "limit",
            format!(
                "binance account-trade history serves 1 to {MAX_HISTORY_LIMIT} entries per page, not {limit}"
            ),
        ));
    }

    let start = request.from.map(inclusive_millis_at_or_after);
    let end = request.to.map(inclusive_millis_before);
    if let (Some(start), Some(end)) = (start, end) {
        if end < start {
            return Err(Error::invalid_request(
                "to",
                "must be later than `from` at Binance millisecond precision",
            ));
        }
        let maximum = match adapter.venue() {
            BinanceMarket::Spot => SPOT_ACCOUNT_TRADE_WINDOW_MILLIS,
            BinanceMarket::UsdMFutures => USD_M_ACCOUNT_TRADE_WINDOW_MILLIS,
        };
        if end - start > maximum {
            return Err(Error::invalid_request(
                "to",
                format!(
                    "Binance account-trade time windows may span at most {} days on this venue",
                    maximum / (24 * 60 * 60 * 1_000),
                ),
            ));
        }
    }

    let mut params = vec![("symbol", adapter.symbol(&request.market)?)];
    if let Some(start) = start {
        params.push(("startTime", start.to_string()));
    }
    if let Some(end) = end {
        params.push(("endTime", end.to_string()));
    }
    params.push(("limit", limit.to_string()));
    signed(
        adapter,
        HttpMethod::Get,
        account_trades_path(adapter.venue()),
        params,
    )
}

/// Builds the signed Spot/Funding C2C-history request.
pub(super) fn c2c_trade_history_request(
    adapter: &BinanceAdapter,
    request: &BinanceC2cTradeHistoryRequest,
) -> Result<HttpRequest> {
    check_c2c_venue(adapter)?;

    let page = request.page.unwrap_or(C2C_DEFAULT_PAGE);
    if page == 0 {
        return Err(Error::invalid_request(
            "page",
            "Binance C2C pages start at 1",
        ));
    }
    let rows = request.rows.unwrap_or(C2C_DEFAULT_ROWS);
    if !(1..=C2C_MAX_ROWS).contains(&rows) {
        return Err(Error::invalid_request(
            "rows",
            format!("Binance C2C history serves 1 to {C2C_MAX_ROWS} rows per page, not {rows}"),
        ));
    }
    if request
        .recv_window
        .is_some_and(|recv_window| recv_window > C2C_MAX_RECV_WINDOW)
    {
        return Err(Error::invalid_request(
            "recv_window",
            format!("Binance C2C recvWindow may not exceed {C2C_MAX_RECV_WINDOW} milliseconds"),
        ));
    }

    let start = request.start_timestamp.map(inclusive_millis_at_or_after);
    let end = request
        .end_timestamp
        .map(|end| end.as_nanos().div_euclid(1_000_000));
    if let (Some(start), Some(end)) = (start, end) {
        if end < start {
            return Err(Error::invalid_request(
                "end_timestamp",
                "must not precede `start_timestamp` at Binance millisecond precision",
            ));
        }
        if end - start > C2C_MAX_WINDOW_MILLIS {
            return Err(Error::invalid_request(
                "end_timestamp",
                "Binance C2C history time windows may span at most 30 days",
            ));
        }
    }

    let mut params = vec![("tradeType", request.trade_type.code().to_string())];
    if let Some(start) = start {
        params.push(("startTimestamp", start.to_string()));
    }
    if let Some(end) = end {
        params.push(("endTimestamp", end.to_string()));
    }
    params.push(("page", page.to_string()));
    params.push(("rows", rows.to_string()));
    if let Some(recv_window) = request.recv_window {
        params.push(("recvWindow", recv_window.to_string()));
    }

    signed(
        adapter,
        HttpMethod::Get,
        "/sapi/v1/c2c/orderMatch/listUserOrderHistory",
        params,
    )
}

fn check_c2c_venue(adapter: &BinanceAdapter) -> Result<()> {
    if adapter.venue() == BinanceMarket::Spot {
        return Ok(());
    }
    Err(Error::unsupported(
        Feature::OrderHistory,
        EXCHANGE,
        "C2C history operates on the Spot/Funding account; build a spot adapter",
    ))
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
    let size = match request.size {
        Size::Base(value) | Size::Quote(value) => value,
    };
    if size <= Decimal::ZERO {
        return Err(Error::invalid_request("size", "must be greater than zero"));
    }
    if let Some(price) = request.price
        && price <= Decimal::ZERO
    {
        return Err(Error::invalid_request("price", "must be greater than zero"));
    }

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
        (OrderType::Best, BinanceMarket::UsdMFutures, _) => {
            return Err(Error::unsupported(
                Feature::Trading,
                EXCHANGE,
                "Binance USD-M has no best-price order type; use a market or explicitly priced limit order",
            ));
        }
        (OrderType::Best, BinanceMarket::Spot, _) => "LIMIT",
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
        (Size::Quote(_), OrderType::Best, BinanceMarket::Spot) => {
            return Err(Error::invalid_request(
                "size",
                "a Binance Spot best-price order uses base quantity; use Size::Base",
            ));
        }
        (_, OrderType::Best, BinanceMarket::UsdMFutures) => {
            unreachable!("USD-M best orders returned above")
        }
    }

    if matches!(&request.order_type, OrderType::Best) {
        if request.price.is_some() {
            return Err(Error::invalid_request(
                "price",
                "a Binance Spot best-price order gets its price from the opposing book",
            ));
        }
        if !matches!(
            request.time_in_force,
            Some(TimeInForce::ImmediateOrCancel | TimeInForce::FillOrKill)
        ) {
            return Err(Error::invalid_request(
                "time_in_force",
                "a Binance Spot best-price order requires immediate-or-cancel or fill-or-kill",
            ));
        }
        params.push(("pegPriceType", "MARKET_PEG".to_string()));
    } else if let Some(price) = request.price {
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

fn order_params(
    adapter: &BinanceAdapter,
    request: &OrderRequest,
) -> Result<Vec<(&'static str, String)>> {
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
    if let Some(client_id) = &request.client_id {
        validate_client_order_id(client_id)?;
        params.push(("newClientOrderId", client_id.clone()));
    }

    Ok(params)
}

pub(super) fn place_order_request(
    adapter: &BinanceAdapter,
    request: &OrderRequest,
) -> Result<HttpRequest> {
    let venue = adapter.venue();
    let mut params = order_params(adapter, request)?;
    // Both venues answer with a bare acknowledgement by default; `RESULT` is
    // what makes the response describe the order that was actually placed.
    params.push(("newOrderRespType", "RESULT".to_string()));

    signed(adapter, HttpMethod::Post, order_path(venue), params)
}

/// Builds a test-order request without the placement response option.
pub(super) fn test_order_request(
    adapter: &BinanceAdapter,
    request: &BinanceTestOrderRequest,
) -> Result<HttpRequest> {
    let venue = adapter.venue();
    if request.compute_commission_rates && venue == BinanceMarket::UsdMFutures {
        return Err(Error::invalid_request(
            "compute_commission_rates",
            "Binance documents commission-rate calculation only for Spot test orders",
        ));
    }
    let mut params = order_params(adapter, &request.order)?;
    if request.compute_commission_rates {
        params.push(("computeCommissionRates", "true".to_string()));
    }
    signed(
        adapter,
        HttpMethod::Post,
        match venue {
            BinanceMarket::Spot => "/api/v3/order/test",
            BinanceMarket::UsdMFutures => "/fapi/v1/order/test",
        },
        params,
    )
}

/// Builds a request that cancels every open order for one symbol.
pub(super) fn cancel_all_open_orders_request(
    adapter: &BinanceAdapter,
    market: &Market,
) -> Result<HttpRequest> {
    signed(
        adapter,
        HttpMethod::Delete,
        match adapter.venue() {
            BinanceMarket::Spot => "/api/v3/openOrders",
            BinanceMarket::UsdMFutures => "/fapi/v1/allOpenOrders",
        },
        vec![("symbol", adapter.symbol(market)?)],
    )
}

fn validate_client_order_id(value: &str) -> Result<()> {
    if (1..=36).contains(&value.len())
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'/' | b':' | b'_' | b'-')
        })
    {
        return Ok(());
    }
    Err(Error::invalid_request(
        "client_id",
        "a Binance client order id must contain 1-36 ASCII letters, digits, '.', '/', ':', '_' or '-'",
    ))
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

pub(super) fn cancel_order_by_client_id_request(
    adapter: &BinanceAdapter,
    market: &Market,
    client_id: &str,
) -> Result<HttpRequest> {
    validate_client_order_id(client_id)?;
    let params = vec![
        ("symbol", adapter.symbol(market)?),
        ("origClientOrderId", client_id.to_string()),
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
        None => request.from.map(inclusive_millis_at_or_after),
    };

    let mut params = Vec::new();
    if let Some(start) = start {
        params.push(("startTime", start.to_string()));
    }
    if let Some(to) = request.to {
        // Convert the exclusive nanosecond boundary to Binance's inclusive
        // millisecond `endTime` without discarding a partial millisecond.
        params.push(("endTime", inclusive_millis_before(to).to_string()));
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

/// Reads Spot account information without flattening it into balances alone.
pub(super) async fn spot_account_information(
    adapter: &BinanceAdapter,
) -> Result<BinanceSpotAccountInformation> {
    if adapter.venue() != BinanceMarket::Spot {
        return Err(Error::unsupported(
            Feature::Balances,
            EXCHANGE,
            "Spot account information requires an adapter built with `spot`",
        ));
    }
    let body = adapter.send(balances_request(adapter)?).await?;
    spot_account_information_from_body(&body)
}

fn spot_account_information_from_body(body: &str) -> Result<BinanceSpotAccountInformation> {
    let response: serde_json::Value = parse::json(body, "account")?;
    if !response.is_object() {
        return Err(Error::decode(
            "Binance Spot account response is not an object",
        ));
    }
    let raw: RawSpotAccountInformation = serde_json::from_value(response.clone())
        .map_err(|error| Error::decode(format!("unreadable Spot account information: {error}")))?;

    Ok(BinanceSpotAccountInformation {
        maker_commission: raw.maker_commission,
        taker_commission: raw.taker_commission,
        buyer_commission: raw.buyer_commission,
        seller_commission: raw.seller_commission,
        commission_rates: BinanceSpotCommissionRates {
            maker: parse::decimal(&raw.commission_rates.maker, "commissionRates.maker")?,
            taker: parse::decimal(&raw.commission_rates.taker, "commissionRates.taker")?,
            buyer: parse::decimal(&raw.commission_rates.buyer, "commissionRates.buyer")?,
            seller: parse::decimal(&raw.commission_rates.seller, "commissionRates.seller")?,
        },
        can_trade: raw.can_trade,
        can_withdraw: raw.can_withdraw,
        can_deposit: raw.can_deposit,
        update_time: parse::millis(raw.update_time),
        account_type: raw.account_type,
        balances: raw
            .balances
            .iter()
            .map(|balance| {
                Ok(BinanceSpotAccountBalance {
                    asset: balance.asset.clone(),
                    free: parse::decimal(&balance.free, "free")?,
                    locked: parse::decimal(&balance.locked, "locked")?,
                })
            })
            .collect::<Result<Vec<_>>>()?,
        permissions: raw.permissions,
        uid: raw.uid,
        raw_json: parse::canonical_json(&response, "Spot account information")?,
    })
}

/// Reads the entire USD-M account snapshot, including per-asset and
/// per-position values omitted by the common balance and margin APIs.
pub(super) async fn usd_m_account_information(
    adapter: &BinanceAdapter,
) -> Result<BinanceUsdMAccountInformation> {
    check_futures_only(adapter, Feature::Margin)?;
    let body = adapter.send(balances_request(adapter)?).await?;
    usd_m_account_information_from_body(&body)
}

fn usd_m_account_information_from_body(body: &str) -> Result<BinanceUsdMAccountInformation> {
    let response: serde_json::Value = parse::json(body, "USD-M account")?;
    if !response.is_object() {
        return Err(Error::decode(
            "Binance USD-M account response is not an object",
        ));
    }
    let raw: RawUsdMAccountInformation = serde_json::from_value(response.clone())
        .map_err(|error| Error::decode(format!("unreadable USD-M account information: {error}")))?;

    Ok(BinanceUsdMAccountInformation {
        total_initial_margin: parse::decimal(&raw.total_initial_margin, "totalInitialMargin")?,
        total_maintenance_margin: parse::decimal(&raw.total_maint_margin, "totalMaintMargin")?,
        total_wallet_balance: parse::decimal(&raw.total_wallet_balance, "totalWalletBalance")?,
        total_unrealized_profit: parse::decimal(
            &raw.total_unrealized_profit,
            "totalUnrealizedProfit",
        )?,
        total_margin_balance: parse::decimal(&raw.total_margin_balance, "totalMarginBalance")?,
        total_position_initial_margin: parse::decimal(
            &raw.total_position_initial_margin,
            "totalPositionInitialMargin",
        )?,
        total_open_order_initial_margin: parse::decimal(
            &raw.total_open_order_initial_margin,
            "totalOpenOrderInitialMargin",
        )?,
        total_cross_wallet_balance: parse::decimal(
            &raw.total_cross_wallet_balance,
            "totalCrossWalletBalance",
        )?,
        total_cross_unrealized_profit: parse::decimal(&raw.total_cross_un_pnl, "totalCrossUnPnl")?,
        available_balance: parse::decimal(&raw.available_balance, "availableBalance")?,
        max_withdraw_amount: parse::decimal(&raw.max_withdraw_amount, "maxWithdrawAmount")?,
        assets: raw
            .assets
            .iter()
            .map(usd_m_account_asset)
            .collect::<Result<Vec<_>>>()?,
        positions: raw
            .positions
            .iter()
            .map(usd_m_account_position)
            .collect::<Result<Vec<_>>>()?,
        raw_json: parse::canonical_json(&response, "USD-M account information")?,
    })
}

fn usd_m_account_asset(raw: &RawUsdMAccountAsset) -> Result<BinanceUsdMAccountAsset> {
    Ok(BinanceUsdMAccountAsset {
        asset: raw.asset.clone(),
        wallet_balance: parse::decimal(&raw.wallet_balance, "walletBalance")?,
        unrealized_profit: parse::decimal(&raw.unrealized_profit, "unrealizedProfit")?,
        margin_balance: parse::decimal(&raw.margin_balance, "marginBalance")?,
        maintenance_margin: parse::decimal(&raw.maint_margin, "maintMargin")?,
        initial_margin: parse::decimal(&raw.initial_margin, "initialMargin")?,
        position_initial_margin: parse::decimal(
            &raw.position_initial_margin,
            "positionInitialMargin",
        )?,
        open_order_initial_margin: parse::decimal(
            &raw.open_order_initial_margin,
            "openOrderInitialMargin",
        )?,
        cross_wallet_balance: parse::decimal(&raw.cross_wallet_balance, "crossWalletBalance")?,
        cross_unrealized_profit: parse::decimal(&raw.cross_un_pnl, "crossUnPnl")?,
        available_balance: parse::decimal(&raw.available_balance, "availableBalance")?,
        max_withdraw_amount: parse::decimal(&raw.max_withdraw_amount, "maxWithdrawAmount")?,
        update_time: parse::millis(raw.update_time),
    })
}

fn usd_m_account_position(raw: &RawUsdMAccountPosition) -> Result<BinanceUsdMAccountPosition> {
    Ok(BinanceUsdMAccountPosition {
        symbol: raw.symbol.clone(),
        position_side: raw.position_side.clone(),
        position_amount: parse::decimal(&raw.position_amt, "positionAmt")?,
        unrealized_profit: parse::decimal(&raw.unrealized_profit, "unrealizedProfit")?,
        isolated_margin: parse::decimal(&raw.isolated_margin, "isolatedMargin")?,
        notional: parse::decimal(&raw.notional, "notional")?,
        isolated_wallet: parse::decimal(&raw.isolated_wallet, "isolatedWallet")?,
        initial_margin: parse::decimal(&raw.initial_margin, "initialMargin")?,
        maintenance_margin: parse::decimal(&raw.maint_margin, "maintMargin")?,
        update_time: parse::millis(raw.update_time),
    })
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

/// One account-owned execution from Spot `myTrades` or USD-M `userTrades`.
///
/// The provider-specific fields preserve USD-M's `pair`, `baseQty`, and
/// `marginAsset` additions rather than collapsing them into a Spot-shaped
/// trade. `next` is intentionally absent because the common timestamp cursor
/// cannot safely represent Binance's trade-ID cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceAccountTrade {
    /// The market requested from Binance.
    pub market: Market,
    /// Binance's account-trade identifier.
    pub id: String,
    /// The order that caused this execution.
    pub order_id: String,
    /// Spot order-list identifier, including Binance's `-1` non-list sentinel.
    pub order_list_id: Option<String>,
    /// Execution time in Binance milliseconds.
    pub timestamp: Timestamp,
    /// The account order's side.
    pub side: Side,
    /// Whether the account was the resting maker.
    pub maker: bool,
    /// Spot's `isBestMatch` flag, when Binance publishes it.
    pub best_match: Option<bool>,
    /// Execution price in quote units.
    pub price: Decimal,
    /// Executed quantity as Binance reports it.
    pub quantity: Decimal,
    /// Executed quote quantity, when the provider includes it.
    pub quote_quantity: Option<Decimal>,
    /// Commission charged for the execution.
    pub commission: Decimal,
    /// Asset in which Binance charged the commission.
    pub commission_asset: String,
    /// USD-M realized profit or loss, when published.
    pub realized_pnl: Option<Decimal>,
    /// USD-M position-side label, when published.
    pub position_side: Option<String>,
    /// USD-M underlying pair, when published.
    pub pair: Option<String>,
    /// USD-M base quantity, when published.
    pub base_quantity: Option<Decimal>,
    /// USD-M margin asset, when published.
    pub margin_asset: Option<String>,
}

pub(super) async fn account_trades(
    adapter: &BinanceAdapter,
    request: &HistoryRequest,
) -> Result<Page<BinanceAccountTrade>> {
    let body = adapter
        .send(account_trades_request(adapter, request)?)
        .await?;
    let raw: Vec<RawAccountTrade> = parse::json(&body, "account trades")?;
    account_trade_page(request, &raw)
}

fn account_trade_page(
    request: &HistoryRequest,
    raw: &[RawAccountTrade],
) -> Result<Page<BinanceAccountTrade>> {
    let items = raw
        .iter()
        .map(|entry| account_trade(request, entry))
        .collect::<Result<Vec<_>>>()?;

    Ok(Page { items, next: None })
}

fn account_trade(request: &HistoryRequest, raw: &RawAccountTrade) -> Result<BinanceAccountTrade> {
    let side = match raw.side.as_deref() {
        Some(value) => {
            let side = parse::side(value)?;
            let buyer_side = if raw.buyer { Side::Buy } else { Side::Sell };
            if side != buyer_side {
                return Err(Error::decode(format!(
                    "Binance account trade side `{value}` disagrees with buyer={}",
                    raw.buyer
                )));
            }
            side
        }
        None if raw.buyer => Side::Buy,
        None => Side::Sell,
    };

    Ok(BinanceAccountTrade {
        market: request.market.clone(),
        id: raw.id.to_string(),
        order_id: raw.order_id.to_string(),
        order_list_id: raw.order_list_id.as_ref().map(ToString::to_string),
        timestamp: parse::millis(raw.time),
        side,
        maker: raw.maker,
        best_match: raw.best_match,
        price: parse::decimal(&raw.price, "price")?,
        quantity: parse::decimal(&raw.qty, "qty")?,
        quote_quantity: raw
            .quote_qty
            .as_deref()
            .map(|value| parse::decimal(value, "quoteQty"))
            .transpose()?,
        commission: parse::decimal(&raw.commission, "commission")?,
        commission_asset: raw.commission_asset.to_ascii_uppercase(),
        realized_pnl: raw
            .realized_pnl
            .as_deref()
            .map(|value| parse::decimal(value, "realizedPnl"))
            .transpose()?,
        position_side: raw.position_side.clone(),
        pair: raw.pair.clone(),
        base_quantity: raw
            .base_qty
            .as_deref()
            .map(|value| parse::decimal(value, "baseQty"))
            .transpose()?,
        margin_asset: raw
            .margin_asset
            .as_ref()
            .map(|value| value.to_ascii_uppercase()),
    })
}

/// One C2C order returned by Binance.
///
/// C2C has fiat-specific status, payment, and KYC fields that do not fit a
/// Spot-market order. Every optional field mirrors the current official C2C
/// response model instead of turning a provider omission into a fake zero.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceC2cTrade {
    /// Binance's C2C order number.
    pub order_number: Option<String>,
    /// Advertisement number associated with the C2C order.
    pub adv_no: Option<String>,
    /// Provider C2C side under Binance's spelling.
    pub trade_type: Option<String>,
    /// Crypto asset exchanged in the C2C order.
    pub asset: Option<String>,
    /// Fiat currency of the C2C order.
    pub fiat: Option<String>,
    /// Fiat display symbol.
    pub fiat_symbol: Option<String>,
    /// Crypto quantity as an exact decimal.
    pub amount: Option<Decimal>,
    /// Fiat total as an exact decimal.
    pub total_price: Option<Decimal>,
    /// Fiat unit price as an exact decimal.
    pub unit_price: Option<Decimal>,
    /// Provider C2C lifecycle status under Binance's spelling.
    pub order_status: Option<String>,
    /// C2C order creation time.
    pub created_at: Option<Timestamp>,
    /// Crypto transaction fee as an exact decimal.
    pub commission: Option<Decimal>,
    /// Counterparty nickname as Binance masks or publishes it.
    pub counterparty_nickname: Option<String>,
    /// Provider payment-method identifier.
    pub pay_method_name: Option<String>,
    /// KYC state: 0 not required, 1 unverified, 2 verified.
    pub additional_kyc_verify: Option<u32>,
    /// Provider taker commission rate as an exact decimal.
    pub taker_commission_rate: Option<Decimal>,
    /// Provider taker commission amount as an exact decimal.
    pub taker_commission: Option<Decimal>,
    /// Provider taker trade amount as an exact decimal.
    pub taker_amount: Option<Decimal>,
    /// Provider advertisement role, such as `TAKER`.
    pub advertisement_role: Option<String>,
}

/// Binance's C2C-history response envelope.
///
/// C2C pages by an explicit one-based number. `total` remains the provider's
/// full matching-record count so callers can decide whether to request the
/// next numbered page without a lossy cursor conversion.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceC2cTradeHistoryPage {
    /// Binance response code; `000000` indicates a successful C2C response.
    pub code: Option<String>,
    /// Binance response message, when it provides one.
    pub message: Option<String>,
    /// C2C orders returned in this page.
    pub data: Option<Vec<BinanceC2cTrade>>,
    /// Provider count of all matching C2C orders.
    pub total: Option<u64>,
    /// Provider success flag retained from the response envelope.
    pub success: Option<bool>,
}

pub(super) async fn c2c_trade_history(
    adapter: &BinanceAdapter,
    request: &BinanceC2cTradeHistoryRequest,
) -> Result<BinanceC2cTradeHistoryPage> {
    let body = adapter
        .send_wallet(c2c_trade_history_request(adapter, request)?)
        .await?;
    c2c_trade_history_page(&body)
}

fn c2c_trade_history_page(body: &str) -> Result<BinanceC2cTradeHistoryPage> {
    let raw: RawC2cTradeHistoryPage = parse::json(body, "C2C order history")?;
    Ok(BinanceC2cTradeHistoryPage {
        code: raw.code,
        message: raw.message,
        data: raw
            .data
            .as_deref()
            .map(|items| items.iter().map(c2c_trade).collect::<Result<Vec<_>>>())
            .transpose()?,
        total: raw.total,
        success: raw.success,
    })
}

fn c2c_trade(raw: &RawC2cTrade) -> Result<BinanceC2cTrade> {
    Ok(BinanceC2cTrade {
        order_number: raw.order_number.clone(),
        adv_no: raw.adv_no.clone(),
        trade_type: raw.trade_type.clone(),
        asset: raw.asset.clone(),
        fiat: raw.fiat.clone(),
        fiat_symbol: raw.fiat_symbol.clone(),
        amount: decimal_option(raw.amount.as_deref(), "amount")?,
        total_price: decimal_option(raw.total_price.as_deref(), "totalPrice")?,
        unit_price: decimal_option(raw.unit_price.as_deref(), "unitPrice")?,
        order_status: raw.order_status.clone(),
        created_at: raw.create_time.map(parse::millis),
        commission: decimal_option(raw.commission.as_deref(), "commission")?,
        counterparty_nickname: raw.counter_part_nick_name.clone(),
        pay_method_name: raw.pay_method_name.clone(),
        additional_kyc_verify: raw.additional_kyc_verify,
        taker_commission_rate: decimal_option(
            raw.taker_commission_rate.as_deref(),
            "takerCommissionRate",
        )?,
        taker_commission: decimal_option(raw.taker_commission.as_deref(), "takerCommission")?,
        taker_amount: decimal_option(raw.taker_amount.as_deref(), "takerAmount")?,
        advertisement_role: raw.advertisement_role.clone(),
    })
}

fn decimal_option(value: Option<&str>, field: &'static str) -> Result<Option<Decimal>> {
    value.map(|value| parse::decimal(value, field)).transpose()
}

/// Binance Spot's signed account-information response.
///
/// Use the common balance API for a normalized balance list. This provider
/// result preserves Spot's commission, permission, and account-state fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceSpotAccountInformation {
    /// Legacy maker commission integer published by Binance.
    pub maker_commission: u64,
    /// Legacy taker commission integer published by Binance.
    pub taker_commission: u64,
    /// Legacy buyer commission integer published by Binance.
    pub buyer_commission: u64,
    /// Legacy seller commission integer published by Binance.
    pub seller_commission: u64,
    /// Current Spot commission rates as exact decimal strings converted to decimals.
    pub commission_rates: BinanceSpotCommissionRates,
    /// Whether Binance currently permits trading for this account.
    pub can_trade: bool,
    /// Whether Binance currently permits withdrawals for this account.
    pub can_withdraw: bool,
    /// Whether Binance currently permits deposits for this account.
    pub can_deposit: bool,
    /// Binance's account update time.
    pub update_time: Timestamp,
    /// Binance's provider-specific account type.
    pub account_type: String,
    /// Every asset balance Binance returned, including its exact free and locked amounts.
    pub balances: Vec<BinanceSpotAccountBalance>,
    /// Permissions Binance associates with the API key/account.
    pub permissions: Vec<String>,
    /// Binance account identifier when the endpoint provides it.
    pub uid: Option<u64>,
    /// Complete compact JSON response for forward-compatible provider fields.
    pub raw_json: String,
}

/// Spot commission rates returned within account information.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceSpotCommissionRates {
    /// Maker commission rate.
    pub maker: Decimal,
    /// Taker commission rate.
    pub taker: Decimal,
    /// Buyer commission rate.
    pub buyer: Decimal,
    /// Seller commission rate.
    pub seller: Decimal,
}

/// One asset balance returned inside Spot account information.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceSpotAccountBalance {
    /// Binance asset code.
    pub asset: String,
    /// Freely available amount.
    pub free: Decimal,
    /// Amount locked by Binance.
    pub locked: Decimal,
}

/// All cancellation reports returned by Spot's cancel-all operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceSpotCancelAllOpenOrders {
    /// One response entry for each cancellation or order-list cancellation.
    pub reports: Vec<BinanceSpotCancelledOrder>,
    /// Complete compact JSON array returned by Binance.
    pub raw_json: String,
}

/// A typed ordinary-order subset of one Spot cancellation report.
///
/// Binance can also return order-list reports with a different shape. Optional
/// fields distinguish that shape while `raw_json` retains all fields exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceSpotCancelledOrder {
    /// Native Binance symbol when this is an ordinary order report.
    pub symbol: Option<String>,
    /// Original client-supplied order identifier when Binance provides it.
    pub original_client_order_id: Option<String>,
    /// Numeric Binance order identifier when this is an ordinary order report.
    pub order_id: Option<String>,
    /// Client-supplied order identifier when Binance provides it.
    pub client_order_id: Option<String>,
    /// Provider order status when this is an ordinary order report.
    pub status: Option<String>,
    /// Limit price when Binance provides it.
    pub price: Option<Decimal>,
    /// Original order quantity when Binance provides it.
    pub original_quantity: Option<Decimal>,
    /// Filled quantity when Binance provides it.
    pub executed_quantity: Option<Decimal>,
    /// Cumulative filled quote quantity when Binance provides it.
    pub cumulative_quote_quantity: Option<Decimal>,
    /// Binance's cancellation transaction time when it provides one.
    pub transact_time: Option<Timestamp>,
    /// Binance order-list identifier when this report describes an order list.
    pub order_list_id: Option<String>,
    /// Provider order-list contingency type, such as `OCO`.
    pub contingency_type: Option<String>,
    /// Provider order-list execution status.
    pub list_status_type: Option<String>,
    /// Provider order-list order status.
    pub list_order_status: Option<String>,
    /// Provider client order-list identifier.
    pub list_client_order_id: Option<String>,
    /// Provider order-list transaction time.
    pub transaction_time: Option<Timestamp>,
    /// Complete compact JSON entry for provider-specific order-list fields.
    pub raw_json: String,
}

/// Binance USD-M account information beyond the normalized balance view.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceUsdMAccountInformation {
    /// Total initial margin across the account.
    pub total_initial_margin: Decimal,
    /// Total maintenance margin across the account.
    pub total_maintenance_margin: Decimal,
    /// Total wallet balance across margin assets.
    pub total_wallet_balance: Decimal,
    /// Total unrealized profit and loss across margin assets.
    pub total_unrealized_profit: Decimal,
    /// Total margin balance across margin assets.
    pub total_margin_balance: Decimal,
    /// Total initial margin allocated to positions.
    pub total_position_initial_margin: Decimal,
    /// Total initial margin allocated to open orders.
    pub total_open_order_initial_margin: Decimal,
    /// Total cross-wallet balance.
    pub total_cross_wallet_balance: Decimal,
    /// Total cross unrealized profit and loss.
    pub total_cross_unrealized_profit: Decimal,
    /// Amount Binance reports as available to trade.
    pub available_balance: Decimal,
    /// Amount Binance reports as withdrawable.
    pub max_withdraw_amount: Decimal,
    /// Per-asset account figures.
    pub assets: Vec<BinanceUsdMAccountAsset>,
    /// Per-position account figures.
    pub positions: Vec<BinanceUsdMAccountPosition>,
    /// Complete compact JSON response for forward-compatible provider fields.
    pub raw_json: String,
}

/// Per-asset USD-M account information.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceUsdMAccountAsset {
    /// Margin asset code.
    pub asset: String,
    /// Wallet balance in this asset.
    pub wallet_balance: Decimal,
    /// Unrealized profit and loss in this asset.
    pub unrealized_profit: Decimal,
    /// Margin balance in this asset.
    pub margin_balance: Decimal,
    /// Maintenance margin in this asset.
    pub maintenance_margin: Decimal,
    /// Initial margin in this asset.
    pub initial_margin: Decimal,
    /// Initial margin allocated to positions.
    pub position_initial_margin: Decimal,
    /// Initial margin allocated to open orders.
    pub open_order_initial_margin: Decimal,
    /// Cross-wallet balance in this asset.
    pub cross_wallet_balance: Decimal,
    /// Cross unrealized profit and loss in this asset.
    pub cross_unrealized_profit: Decimal,
    /// Amount Binance reports as available to trade.
    pub available_balance: Decimal,
    /// Amount Binance reports as withdrawable.
    pub max_withdraw_amount: Decimal,
    /// Binance's per-asset update time.
    pub update_time: Timestamp,
}

/// Per-position information embedded in USD-M account information.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceUsdMAccountPosition {
    /// Native Binance symbol.
    pub symbol: String,
    /// Binance position side, such as `BOTH`, `LONG`, or `SHORT`.
    pub position_side: String,
    /// Signed position amount.
    pub position_amount: Decimal,
    /// Unrealized profit and loss.
    pub unrealized_profit: Decimal,
    /// Isolated margin assigned to this position.
    pub isolated_margin: Decimal,
    /// Provider notional value.
    pub notional: Decimal,
    /// Isolated wallet balance.
    pub isolated_wallet: Decimal,
    /// Initial margin for this position.
    pub initial_margin: Decimal,
    /// Maintenance margin for this position.
    pub maintenance_margin: Decimal,
    /// Binance's position update time.
    pub update_time: Timestamp,
}

/// Binance USD-M `positionRisk` information for one returned symbol and side.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceUsdMPositionInformation {
    /// Native Binance symbol.
    pub symbol: String,
    /// Binance position side, such as `BOTH`, `LONG`, or `SHORT`.
    pub position_side: String,
    /// Signed position amount.
    pub position_amount: Decimal,
    /// Position entry price.
    pub entry_price: Decimal,
    /// Provider break-even price.
    pub break_even_price: Decimal,
    /// Current mark price.
    pub mark_price: Decimal,
    /// Unrealized profit and loss.
    pub unrealized_profit: Decimal,
    /// Provider liquidation price.
    pub liquidation_price: Decimal,
    /// Isolated margin assigned to this position.
    pub isolated_margin: Decimal,
    /// Provider notional value.
    pub notional: Decimal,
    /// Asset used to margin this position.
    pub margin_asset: String,
    /// Isolated wallet balance.
    pub isolated_wallet: Decimal,
    /// Initial margin for this position.
    pub initial_margin: Decimal,
    /// Maintenance margin for this position.
    pub maintenance_margin: Decimal,
    /// Initial margin allocated to the position.
    pub position_initial_margin: Decimal,
    /// Initial margin allocated to open orders.
    pub open_order_initial_margin: Decimal,
    /// Binance's automatic-deleveraging tier.
    pub adl: u64,
    /// Bid-side notional published by Binance.
    pub bid_notional: Decimal,
    /// Ask-side notional published by Binance.
    pub ask_notional: Decimal,
    /// Binance's position update time.
    pub update_time: Timestamp,
    /// Complete compact JSON entry for forward-compatible provider fields.
    pub raw_json: String,
}

/// The provider JSON Binance returns after a test-order validation.
///
/// Spot normally returns an empty object. USD-M may return a distinct
/// order-shaped object, whose fields are retained without forcing it into the
/// common [`Order`] model.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceTestOrder {
    /// Canonically serialized successful JSON object returned by Binance.
    pub response_json: String,
}

pub(super) async fn test_order(
    adapter: &BinanceAdapter,
    request: &BinanceTestOrderRequest,
) -> Result<BinanceTestOrder> {
    let body = adapter.send(test_order_request(adapter, request)?).await?;
    test_order_response(&body)
}

fn test_order_response(body: &str) -> Result<BinanceTestOrder> {
    let response: serde_json::Value = parse::json(body, "test order")?;
    if !response.is_object() {
        return Err(Error::decode(
            "Binance test-order response is not an object",
        ));
    }
    Ok(BinanceTestOrder {
        response_json: serde_json::to_string(&response)
            .map_err(|error| Error::decode(format!("could not serialize test order: {error}")))?,
    })
}

pub(super) async fn cancel_all_open_orders(
    adapter: &BinanceAdapter,
    market: &Market,
) -> Result<()> {
    let body = adapter
        .send(cancel_all_open_orders_request(adapter, market)?)
        .await?;
    cancel_all_open_orders_response(adapter.venue(), &body)
}

/// Cancels Spot orders while retaining Binance's cancellation report array.
pub(super) async fn spot_cancel_all_open_orders(
    adapter: &BinanceAdapter,
    market: &Market,
) -> Result<BinanceSpotCancelAllOpenOrders> {
    if adapter.venue() != BinanceMarket::Spot {
        return Err(Error::unsupported(
            Feature::OpenOrders,
            EXCHANGE,
            "Spot cancellation reports require an adapter built with `spot`",
        ));
    }
    let body = adapter
        .send(cancel_all_open_orders_request(adapter, market)?)
        .await?;
    spot_cancel_all_open_orders_from_body(&body)
}

fn spot_cancel_all_open_orders_from_body(body: &str) -> Result<BinanceSpotCancelAllOpenOrders> {
    let response: serde_json::Value = parse::json(body, "Spot cancel all open orders")?;
    let reports = response.as_array().ok_or_else(|| {
        Error::decode("Binance Spot cancel-all response is not the documented order-report array")
    })?;

    let reports = reports
        .iter()
        .map(|report| {
            let raw: RawSpotCancelledOrder =
                serde_json::from_value(report.clone()).map_err(|error| {
                    Error::decode(format!("unreadable Spot cancellation report: {error}"))
                })?;
            Ok(BinanceSpotCancelledOrder {
                symbol: raw.symbol,
                original_client_order_id: raw.orig_client_order_id,
                order_id: raw.order_id.map(|value| value.to_string()),
                client_order_id: raw.client_order_id,
                status: raw.status,
                price: decimal_option(raw.price.as_deref(), "price")?,
                original_quantity: decimal_option(raw.orig_qty.as_deref(), "origQty")?,
                executed_quantity: decimal_option(raw.executed_qty.as_deref(), "executedQty")?,
                cumulative_quote_quantity: decimal_option(
                    raw.cummulative_quote_qty.as_deref(),
                    "cummulativeQuoteQty",
                )?,
                transact_time: raw.transact_time.map(parse::millis),
                order_list_id: raw.order_list_id.map(|value| value.to_string()),
                contingency_type: raw.contingency_type,
                list_status_type: raw.list_status_type,
                list_order_status: raw.list_order_status,
                list_client_order_id: raw.list_client_order_id,
                transaction_time: raw.transaction_time.map(parse::millis),
                raw_json: parse::canonical_json(report, "Spot cancellation report")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BinanceSpotCancelAllOpenOrders {
        reports,
        raw_json: parse::canonical_json(&response, "Spot cancel-all response")?,
    })
}

fn cancel_all_open_orders_response(venue: BinanceMarket, body: &str) -> Result<()> {
    let response: serde_json::Value = parse::json(body, "cancel all open orders")?;
    let expected = match venue {
        BinanceMarket::Spot => response.is_array(),
        BinanceMarket::UsdMFutures => response.is_object(),
    };
    if expected {
        Ok(())
    } else {
        Err(Error::decode(match venue {
            BinanceMarket::Spot => {
                "Binance Spot cancel-all response is not the documented order-report array"
            }
            BinanceMarket::UsdMFutures => {
                "Binance USD-M cancel-all response is not the documented acknowledgement object"
            }
        }))
    }
}

pub(super) async fn place_order(adapter: &BinanceAdapter, request: &OrderRequest) -> Result<Order> {
    place_order_detail(adapter, request)
        .await
        .map(|response| response.order)
}

pub(super) async fn cancel_order(
    adapter: &BinanceAdapter,
    market: &Market,
    order_id: &str,
) -> Result<()> {
    cancel_order_detail(adapter, market, order_id)
        .await
        .map(drop)
}

/// Creates an order and keeps Binance's provider-specific response fields.
pub(super) async fn place_order_detail(
    adapter: &BinanceAdapter,
    request: &OrderRequest,
) -> Result<BinanceOrderResponse> {
    let body = adapter.send(place_order_request(adapter, request)?).await?;
    order_response_from_body(&request.market, &body)
}

/// Cancels one Binance order by exchange identifier and keeps its response.
pub(super) async fn cancel_order_detail(
    adapter: &BinanceAdapter,
    market: &Market,
    order_id: &str,
) -> Result<BinanceOrderResponse> {
    let body = adapter
        .send(cancel_order_request(adapter, market, order_id)?)
        .await?;
    order_response_from_body(market, &body)
}

pub(super) async fn cancel_order_by_client_id(
    adapter: &BinanceAdapter,
    market: &Market,
    client_id: &str,
) -> Result<()> {
    cancel_order_by_client_id_detail(adapter, market, client_id)
        .await
        .map(drop)
}

/// Cancels one Binance order by client identifier and keeps its response.
pub(super) async fn cancel_order_by_client_id_detail(
    adapter: &BinanceAdapter,
    market: &Market,
    client_id: &str,
) -> Result<BinanceOrderResponse> {
    let body = adapter
        .send(cancel_order_by_client_id_request(
            adapter, market, client_id,
        )?)
        .await?;
    order_response_from_body(market, &body)
}

pub(super) fn order_response_from_body(
    market: &Market,
    body: &str,
) -> Result<BinanceOrderResponse> {
    let value: serde_json::Value = parse::json(body, "order")?;
    let raw: RawProviderOrderResponse = serde_json::from_value(value.clone())
        .map_err(|error| Error::decode(format!("unreadable Binance order response: {error}")))?;

    Ok(BinanceOrderResponse {
        order: parse::order(market, &raw.order)?,
        client_order_id: raw.client_order_id,
        order_list_id: raw.order_list_id.map(|value| value.to_string()),
        order_type: raw.order_type,
        time_in_force: raw.time_in_force,
        cumulative_quote_quantity: decimal_option(
            raw.cummulative_quote_qty.as_deref(),
            "cummulativeQuoteQty",
        )?,
        cumulative_quantity: decimal_option(raw.cum_qty.as_deref(), "cumQty")?,
        cumulative_quote: decimal_option(raw.cum_quote.as_deref(), "cumQuote")?,
        average_price: decimal_option(raw.avg_price.as_deref(), "avgPrice")?,
        reduce_only: raw.reduce_only,
        close_position: raw.close_position,
        position_side: raw.position_side,
        stop_price: decimal_option(raw.stop_price.as_deref(), "stopPrice")?,
        working_type: raw.working_type,
        price_protect: raw.price_protect,
        original_type: raw.orig_type,
        price_match: raw.price_match,
        self_trade_prevention_mode: raw.self_trade_prevention_mode,
        good_till_date: raw
            .good_till_date
            .filter(|value| *value > 0)
            .map(parse::millis),
        raw_json: parse::canonical_json(&value, "order")?,
    })
}

/// Reads `/fapi/v3/positionRisk`, including Binance's zero-size rows.
/// [`Client::positions`](crate::Client::positions) filters flat rows from the
/// result exposed to callers.
pub(super) async fn positions(
    adapter: &BinanceAdapter,
    market: Option<&Market>,
) -> Result<Vec<Position>> {
    let body = adapter.send(positions_request(adapter, market)?).await?;
    let raw: Vec<RawPosition> = parse::json(&body, "positionRisk")?;
    raw.iter().map(|raw| position(adapter, raw)).collect()
}

/// Reads the full provider `positionRisk` contract without collapsing its
/// margin, liquidation, and side-specific risk fields into common positions.
pub(super) async fn usd_m_position_information(
    adapter: &BinanceAdapter,
    market: Option<&Market>,
) -> Result<Vec<BinanceUsdMPositionInformation>> {
    let body = adapter.send(positions_request(adapter, market)?).await?;
    usd_m_position_information_from_body(&body)
}

fn usd_m_position_information_from_body(body: &str) -> Result<Vec<BinanceUsdMPositionInformation>> {
    let response: serde_json::Value = parse::json(body, "positionRisk")?;
    let entries = response
        .as_array()
        .ok_or_else(|| Error::decode("Binance USD-M positionRisk response is not an array"))?;

    entries
        .iter()
        .map(|entry| {
            let raw: RawUsdMPositionInformation =
                serde_json::from_value(entry.clone()).map_err(|error| {
                    Error::decode(format!("unreadable USD-M position information: {error}"))
                })?;
            Ok(BinanceUsdMPositionInformation {
                symbol: raw.symbol,
                position_side: raw.position_side,
                position_amount: parse::decimal(&raw.position_amt, "positionAmt")?,
                entry_price: parse::decimal(&raw.entry_price, "entryPrice")?,
                break_even_price: parse::decimal(&raw.break_even_price, "breakEvenPrice")?,
                mark_price: parse::decimal(&raw.mark_price, "markPrice")?,
                unrealized_profit: parse::decimal(&raw.unrealized_profit, "unRealizedProfit")?,
                liquidation_price: parse::decimal(&raw.liquidation_price, "liquidationPrice")?,
                isolated_margin: parse::decimal(&raw.isolated_margin, "isolatedMargin")?,
                notional: parse::decimal(&raw.notional, "notional")?,
                margin_asset: raw.margin_asset,
                isolated_wallet: parse::decimal(&raw.isolated_wallet, "isolatedWallet")?,
                initial_margin: parse::decimal(&raw.initial_margin, "initialMargin")?,
                maintenance_margin: parse::decimal(&raw.maint_margin, "maintMargin")?,
                position_initial_margin: parse::decimal(
                    &raw.position_initial_margin,
                    "positionInitialMargin",
                )?,
                open_order_initial_margin: parse::decimal(
                    &raw.open_order_initial_margin,
                    "openOrderInitialMargin",
                )?,
                adl: raw.adl,
                bid_notional: parse::decimal(&raw.bid_notional, "bidNotional")?,
                ask_notional: parse::decimal(&raw.ask_notional, "askNotional")?,
                update_time: parse::millis(raw.update_time),
                raw_json: parse::canonical_json(entry, "USD-M position information")?,
            })
        })
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
        // `positionRisk` carries neither field; `None` means unpublished.
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

/// A Spot order lookup, including Binance-specific fields and terminal orders.
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

/// A Binance order response that keeps the normalized order and the
/// venue-specific response fields returned by create or cancel requests.
///
/// Spot and USD-M share the basic order shape but publish different optional
/// execution and risk fields. The complete response remains available through
/// [`Self::raw_json`] when Binance introduces additional fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceOrderResponse {
    /// The order projected onto maxt's portable order contract.
    pub order: Order,
    /// Binance's caller-assigned order identifier, when returned.
    pub client_order_id: Option<String>,
    /// Spot's order-list identifier, when this response belongs to one.
    pub order_list_id: Option<String>,
    /// Provider order type, including venue-specific values.
    pub order_type: Option<String>,
    /// Provider time-in-force value.
    pub time_in_force: Option<String>,
    /// Filled quote quantity from Spot's response.
    pub cumulative_quote_quantity: Option<Decimal>,
    /// Filled base quantity from USD-M's response.
    pub cumulative_quantity: Option<Decimal>,
    /// Filled quote quantity from USD-M's response.
    pub cumulative_quote: Option<Decimal>,
    /// USD-M average fill price.
    pub average_price: Option<Decimal>,
    /// USD-M reduce-only flag.
    pub reduce_only: Option<bool>,
    /// USD-M close-position flag.
    pub close_position: Option<bool>,
    /// USD-M position-side value.
    pub position_side: Option<String>,
    /// Provider stop price, when the order type has one.
    pub stop_price: Option<Decimal>,
    /// USD-M working-price source.
    pub working_type: Option<String>,
    /// USD-M price-protection flag.
    pub price_protect: Option<bool>,
    /// USD-M original order type.
    pub original_type: Option<String>,
    /// USD-M price-match mode.
    pub price_match: Option<String>,
    /// Binance self-trade-prevention mode.
    pub self_trade_prevention_mode: Option<String>,
    /// USD-M good-till timestamp, when the order uses one.
    pub good_till_date: Option<Timestamp>,
    /// Complete provider response object encoded as JSON.
    pub raw_json: String,
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

/// A bearer token for a USD-M user-data stream.
///
/// [`Debug`] redacts the value. Keep it alive with
/// [`BinanceAdapter::usd_m_keepalive_listen_key`], or let
/// [`Client::subscribe_account`](crate::Client::subscribe_account) manage it.
/// Spot account streams do not use listen keys.
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

/// Creates or extends the account's USD-M listen key.
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
pub(super) fn keepalive_listen_key_request(adapter: &BinanceAdapter) -> Result<HttpRequest> {
    api_key_only(adapter, HttpMethod::Put, USD_M_LISTEN_KEY_PATH, &[])
}

pub(super) async fn keepalive_listen_key(adapter: &BinanceAdapter) -> Result<()> {
    let request = keepalive_listen_key_request(adapter)?;
    adapter.send(request).await.map(|_| ())
}

pub(super) async fn close_listen_key(adapter: &BinanceAdapter) -> Result<()> {
    let request = close_listen_key_request(adapter)?;
    adapter.send(request).await.map(|_| ())
}

/// Builds the USD-M close request. Binance closes the key owned by the API key
/// and rejects a `listenKey` query parameter.
pub(super) fn close_listen_key_request(adapter: &BinanceAdapter) -> Result<HttpRequest> {
    api_key_only(adapter, HttpMethod::Delete, USD_M_LISTEN_KEY_PATH, &[])
}

/// USD-M account events requested in the socket URL.
///
/// These are the order, balance, and expiry events consumed by the decoder.
/// `eventStreamTerminated` is Spot-only and is not requested here.
pub(super) const USD_M_ACCOUNT_EVENTS: &str = "ORDER_TRADE_UPDATE/ACCOUNT_UPDATE/listenKeyExpired";

/// Builds the USD-M `/private` user-data URL.
///
/// The bearer key is percent-encoded and the requested event names retain `/`
/// as their separator.
pub(super) fn usd_m_user_data_stream_url(key: &BinanceListenKey) -> String {
    // The slash between event names is a separator, so it stays literal; only
    // the key is encoded.
    format!(
        "wss://fstream.binance.com/private/ws?listenKey={}&events={USD_M_ACCOUNT_EVENTS}",
        encode(&key.0)
    )
}

/// Builds a signed Spot account-subscription frame.
///
/// HMAC-SHA-256 covers the parameters sorted by name. A fresh timestamp and
/// signature are generated for every handshake, with Binance's 60-second
/// maximum `recvWindow`.
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

/// Binance's maximum receive window for the signed Spot subscribe frame.
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
    fn account_trades_use_the_venue_path_and_reject_an_unsafe_timestamp_cursor() {
        let spot_request = HistoryRequest::new(btc_usdt())
            .from(Timestamp::from_nanos(1_000_000_001))
            .to(Timestamp::from_millis(86_400_001))
            .limit(1_000);
        let futures_request = HistoryRequest::new(btc_usdt_perp())
            .from(Timestamp::from_millis(1_000))
            .to(Timestamp::from_millis(604_801_000));

        assert_eq!(
            signed_params(
                &account_trades_request(&spot(), &spot_request).expect("a spot account-trade page"),
            ),
            "/api/v3/myTrades?symbol=BTCUSDT&startTime=1001&endTime=86400000&limit=1000"
        );
        assert_eq!(
            signed_params(
                &account_trades_request(&perp(), &futures_request)
                    .expect("a USD-M account-trade page"),
            ),
            "/fapi/v1/userTrades?symbol=BTCUSDT&startTime=1000&endTime=604800999&limit=500"
        );

        for request in [
            HistoryRequest::new(btc_usdt())
                .from(Timestamp::from_millis(0))
                // `to` is exclusive, so one additional millisecond becomes
                // an inclusive endpoint exactly 24 hours away. Use two.
                .to(Timestamp::from_millis(SPOT_ACCOUNT_TRADE_WINDOW_MILLIS + 2)),
            HistoryRequest::new(btc_usdt_perp())
                .from(Timestamp::from_millis(0))
                .to(Timestamp::from_millis(
                    USD_M_ACCOUNT_TRADE_WINDOW_MILLIS + 2,
                )),
        ] {
            let adapter = if request.market.kind == MarketKind::Spot {
                spot()
            } else {
                perp()
            };
            assert!(matches!(
                account_trades_request(&adapter, &request),
                Err(Error::InvalidRequest { field, .. }) if field == "to"
            ));
        }

        assert!(matches!(
            account_trades_request(
                &spot(),
                &HistoryRequest::new(btc_usdt()).cursor(Cursor::new("t1000"))
            ),
            Err(Error::InvalidRequest { field, .. }) if field == "cursor"
        ));
        assert!(matches!(
            account_trades_request(&spot(), &HistoryRequest::new(btc_usdt()).limit(1_001)),
            Err(Error::InvalidRequest { field, .. }) if field == "limit"
        ));
        assert!(matches!(
            account_trades_request(&spot(), &HistoryRequest::new(btc_usdt_perp())),
            Err(Error::InvalidRequest { field, .. }) if field == "market"
        ));
        assert!(matches!(
            account_trades_request(&BinanceAdapter::spot(), &HistoryRequest::new(btc_usdt())),
            Err(Error::Auth { .. })
        ));
    }

    #[test]
    fn c2c_history_uses_wallet_sapi_defaults_and_preserves_query_order() {
        use super::super::{BinanceC2cTradeHistoryRequest, BinanceC2cTradeType};

        let defaults = BinanceC2cTradeHistoryRequest::new(BinanceC2cTradeType::Buy);
        let adapter = spot();
        let request =
            c2c_trade_history_request(&adapter, &defaults).expect("a default C2C history request");
        assert_eq!(
            signed_params(&request),
            "/sapi/v1/c2c/orderMatch/listUserOrderHistory?tradeType=BUY&page=1&rows=100"
        );
        let target = request.target();
        let (_, query) = target.split_once('?').expect("signed C2C query");
        let (payload, received_signature) = query
            .rsplit_once("&signature=")
            .expect("C2C request signature");
        assert_eq!(
            received_signature,
            signature(adapter.credentials().expect("C2C credentials"), payload)
                .expect("C2C HMAC signature")
        );
        assert!(
            request
                .headers
                .iter()
                .any(|(name, value)| name == API_KEY_HEADER && value == "key")
        );

        let filtered = BinanceC2cTradeHistoryRequest::new(BinanceC2cTradeType::Sell)
            .start_timestamp(Timestamp::from_nanos(1_000_000_001))
            .end_timestamp(Timestamp::from_nanos(2_000_000_999))
            .page(3)
            .rows(25)
            .recv_window(5_000);
        assert_eq!(
            signed_params(
                &c2c_trade_history_request(&spot(), &filtered)
                    .expect("a filtered C2C history request"),
            ),
            "/sapi/v1/c2c/orderMatch/listUserOrderHistory?tradeType=SELL&startTimestamp=1001&endTimestamp=2000&page=3&rows=25&recvWindow=5000"
        );
    }

    #[test]
    fn c2c_history_rejects_wrong_venue_invalid_limits_windows_and_missing_credentials() {
        use super::super::{BinanceC2cTradeHistoryRequest, BinanceC2cTradeType};

        let base = || BinanceC2cTradeHistoryRequest::new(BinanceC2cTradeType::Buy);
        assert!(matches!(
            c2c_trade_history_request(&perp(), &base()),
            Err(Error::Unsupported {
                feature: Feature::OrderHistory,
                ..
            })
        ));
        assert!(matches!(
            c2c_trade_history_request(&spot(), &base().page(0)),
            Err(Error::InvalidRequest { field, .. }) if field == "page"
        ));
        for request in [base().rows(0), base().rows(101)] {
            assert!(matches!(
                c2c_trade_history_request(&spot(), &request),
                Err(Error::InvalidRequest { field, .. }) if field == "rows"
            ));
        }
        assert!(matches!(
            c2c_trade_history_request(&spot(), &base().recv_window(60_001)),
            Err(Error::InvalidRequest { field, .. }) if field == "recv_window"
        ));
        for request in [
            base()
                .start_timestamp(Timestamp::from_millis(2_000))
                .end_timestamp(Timestamp::from_millis(1_999)),
            base()
                .start_timestamp(Timestamp::from_millis(0))
                .end_timestamp(Timestamp::from_millis(C2C_MAX_WINDOW_MILLIS + 1)),
        ] {
            assert!(matches!(
                c2c_trade_history_request(&spot(), &request),
                Err(Error::InvalidRequest { field, .. }) if field == "end_timestamp"
            ));
        }
        assert!(matches!(
            c2c_trade_history_request(&BinanceAdapter::spot(), &base()),
            Err(Error::Auth { .. })
        ));
    }

    #[test]
    fn order_test_and_cancel_all_use_distinct_venue_contracts() {
        let spot_order = OrderRequest::limit(
            btc_usdt(),
            Side::Buy,
            Size::Base(Decimal::new(1, 2)),
            Decimal::from(100_000),
        );
        let futures_order = OrderRequest::limit(
            btc_usdt_perp(),
            Side::Sell,
            Size::Base(Decimal::new(1, 2)),
            Decimal::from(100_000),
        );
        let spot_test = BinanceTestOrderRequest::new(spot_order.clone());
        let spot_commission_test =
            BinanceTestOrderRequest::new(spot_order.clone()).compute_commission_rates();
        let futures_test = BinanceTestOrderRequest::new(futures_order.clone());
        let futures_commission_test =
            BinanceTestOrderRequest::new(futures_order.clone()).compute_commission_rates();

        assert_eq!(
            signed_params(&test_order_request(&spot(), &spot_test).expect("a Spot test order")),
            "/api/v3/order/test?symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=0.01&price=100000&timeInForce=GTC"
        );
        assert_eq!(
            signed_params(&test_order_request(&perp(), &futures_test).expect("a USD-M test order"),),
            "/fapi/v1/order/test?symbol=BTCUSDT&side=SELL&type=LIMIT&quantity=0.01&price=100000&timeInForce=GTC"
        );
        assert_eq!(
            signed_params(
                &test_order_request(&spot(), &spot_commission_test)
                    .expect("a Spot commission test order"),
            ),
            "/api/v3/order/test?symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=0.01&price=100000&timeInForce=GTC&computeCommissionRates=true"
        );
        assert_eq!(
            signed_params(
                &cancel_all_open_orders_request(&spot(), &btc_usdt())
                    .expect("a Spot cancel-all request"),
            ),
            "/api/v3/openOrders?symbol=BTCUSDT"
        );
        assert_eq!(
            signed_params(
                &cancel_all_open_orders_request(&perp(), &btc_usdt_perp())
                    .expect("a USD-M cancel-all request"),
            ),
            "/fapi/v1/allOpenOrders?symbol=BTCUSDT"
        );

        assert!(matches!(
            test_order_request(&spot(), &BinanceTestOrderRequest::new(futures_order)),
            Err(Error::InvalidRequest { field, .. }) if field == "market"
        ));
        assert!(matches!(
            test_order_request(&perp(), &futures_commission_test),
            Err(Error::InvalidRequest { field, .. }) if field == "compute_commission_rates"
        ));
        assert!(matches!(
            cancel_all_open_orders_request(&perp(), &btc_usdt()),
            Err(Error::InvalidRequest { field, .. }) if field == "market"
        ));
        assert!(matches!(
            test_order_request(
                &BinanceAdapter::spot(),
                &BinanceTestOrderRequest::new(OrderRequest::market(
                    btc_usdt(),
                    Side::Buy,
                    Size::Base(Decimal::ONE),
                )),
            ),
            Err(Error::Auth { .. })
        ));
    }

    #[test]
    fn order_shapes_reject_non_positive_sizes_and_prices_before_signing() {
        let cases = [
            (
                OrderRequest::limit(
                    btc_usdt(),
                    Side::Buy,
                    Size::Base(Decimal::ZERO),
                    Decimal::ONE,
                ),
                "size",
            ),
            (
                OrderRequest::market(btc_usdt(), Side::Buy, Size::Quote(-Decimal::ONE)),
                "size",
            ),
            (
                OrderRequest::limit(
                    btc_usdt(),
                    Side::Buy,
                    Size::Base(Decimal::ONE),
                    Decimal::ZERO,
                ),
                "price",
            ),
            (
                OrderRequest::limit(
                    btc_usdt_perp(),
                    Side::Sell,
                    Size::Base(-Decimal::ONE),
                    -Decimal::ONE,
                ),
                "size",
            ),
        ];

        for (request, field) in cases {
            let venue = if request.market.kind == MarketKind::Spot {
                BinanceMarket::Spot
            } else {
                BinanceMarket::UsdMFutures
            };
            assert!(matches!(
                order_shape(venue, &request),
                Err(Error::InvalidRequest { field: actual, .. }) if actual == field
            ));
        }
    }

    #[test]
    fn account_trade_fixtures_preserve_usd_m_fields_and_never_make_a_timestamp_cursor() {
        // Spot: https://github.com/binance/binance-spot-api-docs/blob/master/rest-api.md#account-trade-list-user_data
        let spot: Vec<RawAccountTrade> = parse::json(
            r#"[{
              "symbol": "BNBBTC",
              "id": 28457,
              "orderId": 100234,
              "orderListId": -1,
              "price": "4.00000100",
              "qty": "12.00000000",
              "quoteQty": "48.000012",
              "commission": "10.10000000",
              "commissionAsset": "BNB",
              "time": 1499865549590,
              "isBuyer": true,
              "isMaker": false,
              "isBestMatch": true
            }]"#,
            "Spot account trades",
        )
        .expect("official Spot account-trade fixture");
        let spot_page = account_trade_page(&HistoryRequest::new(btc_usdt()).limit(1), &spot)
            .expect("a Spot account-trade page");
        assert_eq!(spot_page.next, None);
        assert_eq!(spot_page.items[0].side, Side::Buy);
        assert_eq!(
            spot_page.items[0].quote_quantity,
            Some(Decimal::new(48_000_012, 6))
        );
        assert_eq!(spot_page.items[0].pair, None);
        assert_eq!(spot_page.items[0].best_match, Some(true));
        assert_eq!(spot_page.items[0].order_list_id.as_deref(), Some("-1"));

        // USD-M: official connector's documented `userTrades` endpoint,
        // including fields added to the response contract.
        let usd_m: Vec<RawAccountTrade> = parse::json(
            r#"[{
              "buyer": false,
              "commission": "-0.07819010",
              "commissionAsset": "USDT",
              "id": 698759,
              "maker": false,
              "orderId": 25851813,
              "pair": "BTCUSDT",
              "side": "SELL",
              "price": "7819.01",
              "qty": "0.002",
              "quoteQty": "15.63802",
              "baseQty": "0.002",
              "realizedPnl": "-0.91539999",
              "marginAsset": "USDT",
              "positionSide": "SHORT",
              "symbol": "BTCUSDT",
              "time": 1569514978020
            }]"#,
            "USD-M account trades",
        )
        .expect("USD-M account-trade fixture");
        let usd_m_page = account_trade_page(&HistoryRequest::new(btc_usdt_perp()).limit(1), &usd_m)
            .expect("a USD-M account-trade page");
        assert_eq!(usd_m_page.next, None);
        let trade = &usd_m_page.items[0];
        assert_eq!(trade.side, Side::Sell);
        assert_eq!(trade.pair.as_deref(), Some("BTCUSDT"));
        assert_eq!(trade.base_quantity, Some(Decimal::new(2, 3)));
        assert_eq!(trade.margin_asset.as_deref(), Some("USDT"));
        assert_eq!(trade.realized_pnl, Some(Decimal::new(-91_539_999, 8)));
        assert_eq!(trade.best_match, None);
        assert_eq!(trade.order_list_id, None);
    }

    #[test]
    fn c2c_history_fixture_preserves_the_documented_envelope_and_optional_fields() {
        // Binance C2C's current official connector preserves this envelope
        // and its optional fields: https://github.com/binance/binance-connector-go/tree/main/clients/c2c
        let page = c2c_trade_history_page(
            r#"{
              "code":"000000",
              "message":"success",
              "data":[{
                "orderNumber":"20219644646554779648",
                "advNo":"11218246497340923904",
                "tradeType":"SELL",
                "asset":"BUSD",
                "fiat":"CNY",
                "fiatSymbol":"￥",
                "amount":"5000.00000000",
                "totalPrice":"33400.00000000",
                "unitPrice":"6.68",
                "orderStatus":"COMPLETED",
                "createTime":1619361369000,
                "commission":"0",
                "counterPartNickName":"ab***",
                "payMethodName":"Bank",
                "additionalKycVerify":2,
                "takerCommissionRate":"0.001",
                "takerCommission":"5",
                "takerAmount":"5000",
                "advertisementRole":"TAKER"
              }, {"orderNumber":"optional-only"}],
              "total":9,
              "success":true
            }"#,
        )
        .expect("official-shaped C2C fixture");

        assert_eq!(page.code.as_deref(), Some("000000"));
        assert_eq!(page.message.as_deref(), Some("success"));
        assert_eq!(page.total, Some(9));
        assert_eq!(page.success, Some(true));
        let data = page.data.expect("documented data array");
        assert_eq!(data.len(), 2);
        let order = &data[0];
        assert_eq!(order.order_number.as_deref(), Some("20219644646554779648"));
        assert_eq!(order.asset.as_deref(), Some("BUSD"));
        assert_eq!(order.amount, Some(Decimal::from(5_000)));
        assert_eq!(order.total_price, Some(Decimal::from(33_400)));
        assert_eq!(
            order.created_at,
            Some(Timestamp::from_millis(1_619_361_369_000))
        );
        assert_eq!(order.counterparty_nickname.as_deref(), Some("ab***"));
        assert_eq!(order.additional_kyc_verify, Some(2));
        assert_eq!(order.taker_commission_rate, Some(Decimal::new(1, 3)));
        assert_eq!(order.advertisement_role.as_deref(), Some("TAKER"));
        assert_eq!(data[1].asset, None);
        assert_eq!(data[1].amount, None);

        let nullable = c2c_trade_history_page(
            r#"{"code":null,"message":null,"data":null,"total":null,"success":null}"#,
        )
        .expect("official optional C2C envelope");
        assert_eq!(nullable.code, None);
        assert_eq!(nullable.message, None);
        assert_eq!(nullable.data, None);
        assert_eq!(nullable.total, None);
        assert_eq!(nullable.success, None);

        assert!(c2c_trade_history_page(r#"{"data":[{"additionalKycVerify":-1}]}"#,).is_err());
    }

    #[test]
    fn cancellation_and_test_order_fixtures_keep_the_two_response_shapes_separate() {
        // Spot returns order reports, including the array form even when no
        // report-specific field is needed by this provider method.
        assert!(
            cancel_all_open_orders_response(
                BinanceMarket::Spot,
                r#"[{"symbol":"BTCUSDT","orderId":11,"status":"CANCELED"}]"#
            )
            .is_ok()
        );
        // USD-M returns a single acknowledgement object instead of reports.
        assert!(
            cancel_all_open_orders_response(
                BinanceMarket::UsdMFutures,
                r#"{"code":200,"msg":"The operation of cancel all open orders is done."}"#
            )
            .is_ok()
        );
        assert!(cancel_all_open_orders_response(BinanceMarket::Spot, r#"{"code":200}"#).is_err());
        assert!(cancel_all_open_orders_response(BinanceMarket::UsdMFutures, "[]").is_err());

        let spot_test = test_order_response("{}").expect("a Spot test-order response");
        assert_eq!(spot_test.response_json, "{}");

        let spot_commissions = test_order_response(
            r#"{
              "standardCommissionForOrder": {"maker": "0.00000112", "taker": "0.00000114"},
              "specialCommissionForOrder": {"maker": "0.05000000", "taker": "0.06000000"},
              "taxCommissionForOrder": {"maker": "0.00000112", "taker": "0.00000114"},
              "discount": {
                "enabledForAccount": true,
                "enabledForSymbol": true,
                "discountAsset": "BNB",
                "discount": "0.25000000"
              }
            }"#,
        )
        .expect("a Spot commission test-order response");
        let commissions: serde_json::Value = serde_json::from_str(&spot_commissions.response_json)
            .expect("canonical Spot commission JSON");
        assert_eq!(
            commissions["standardCommissionForOrder"]["maker"],
            "0.00000112"
        );
        assert_eq!(commissions["discount"]["discountAsset"], "BNB");

        // USD-M documents the same order-shaped response as a new order, with
        // placeholder values because the test never reaches the matcher.
        let usd_m_test = test_order_response(
            r#"{
              "clientOrderId": "testOrder",
              "cumQty": "0",
              "cumQuote": "0",
              "executedQty": "0",
              "orderId": 22542179,
              "avgPrice": "0.00000",
              "origQty": "10",
              "price": "0",
              "reduceOnly": false,
              "side": "SELL",
              "positionSide": "SHORT",
              "status": "NEW",
              "stopPrice": "0",
              "closePosition": false,
              "symbol": "BTCUSDT",
              "timeInForce": "GTC",
              "type": "TRAILING_STOP_MARKET"
            }"#,
        )
        .expect("a USD-M test-order response");
        let parsed: serde_json::Value =
            serde_json::from_str(&usd_m_test.response_json).expect("canonical test-order JSON");
        assert_eq!(parsed["clientOrderId"], "testOrder");
        assert_eq!(parsed["orderId"], 22_542_179);
        assert_eq!(parsed["positionSide"], "SHORT");
        assert!(test_order_response("[]").is_err());
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
            Err(Error::InvalidRequest { field, .. }) if field == "size"
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
            Err(Error::InvalidRequest { field, .. }) if field == "time_in_force"
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
                    Err(Error::InvalidRequest { field, .. }) if field == "order_id"
                ),
                "{bad}"
            );
        }
    }

    #[test]
    fn a_client_order_id_cancels_through_its_own_parameter() {
        assert_eq!(
            signed_params(
                &cancel_order_by_client_id_request(&spot(), &btc_usdt(), "client-1")
                    .expect("a client order id"),
            ),
            "/api/v3/order?symbol=BTCUSDT&origClientOrderId=client-1"
        );
    }

    #[test]
    fn a_spot_best_order_uses_the_opposing_book_peg() {
        let request = OrderRequest::best(
            btc_usdt(),
            Side::Buy,
            Size::Base(Decimal::new(1, 2)),
            TimeInForce::ImmediateOrCancel,
        )
        .client_id("client/1");

        assert_eq!(
            signed_params(&place_order_request(&spot(), &request).expect("a pegged order")),
            "/api/v3/order?symbol=BTCUSDT&side=BUY&type=LIMIT&quantity=0.01&pegPriceType=MARKET_PEG&timeInForce=IOC&newClientOrderId=client%2F1&newOrderRespType=RESULT"
        );
        assert!(matches!(
            cancel_order_by_client_id_request(&spot(), &btc_usdt(), "bad&client"),
            Err(Error::InvalidRequest { field, .. }) if field == "client_id"
        ));
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

        let sub_millisecond_end = HistoryRequest::new(btc_usdt_perp())
            .to(Timestamp::from_nanos(1_570_636_800_000_000_001));
        assert!(
            funding_rates_request(&perp(), &sub_millisecond_end)
                .expect("a sub-millisecond exclusive end")
                .target()
                .contains("endTime=1570636800000")
        );

        let sub_millisecond_start = HistoryRequest::new(btc_usdt_perp())
            .from(Timestamp::from_nanos(1_570_608_000_000_000_001));
        assert!(
            funding_rates_request(&perp(), &sub_millisecond_start)
                .expect("a sub-millisecond inclusive start")
                .target()
                .contains("startTime=1570608000001")
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
            Err(Error::InvalidRequest { field, .. }) if field == "cursor"
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
            Err(Error::InvalidRequest { field, .. }) if field == "limit"
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
            Err(Error::InvalidRequest { field, .. }) if field == "leverage"
        ));
        assert!(matches!(
            set_margin_requests(
                &perp(),
                &MarginRequest::new(btc_usdt_perp()).leverage(Decimal::new(15, 1))
            ),
            Err(Error::InvalidRequest { field, .. }) if field == "leverage"
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
    fn provider_account_contracts_preserve_typed_fields_and_raw_json() {
        let spot = spot_account_information_from_body(
            r#"{
              "makerCommission":15,"takerCommission":15,"buyerCommission":0,"sellerCommission":0,
              "commissionRates":{"maker":"0.0015","taker":"0.0015","buyer":"0","seller":"0"},
              "canTrade":true,"canWithdraw":true,"canDeposit":true,"updateTime":1700000000000,
              "accountType":"SPOT","permissions":["SPOT"],"uid":42,
              "balances":[{"asset":"BTC","free":"1.2","locked":"0.3"}],"futureField":"kept"
            }"#,
        )
        .expect("official-shaped Spot account response");
        assert_eq!(spot.commission_rates.maker, Decimal::new(15, 4));
        assert_eq!(spot.permissions, ["SPOT"]);
        assert_eq!(spot.balances[0].locked, Decimal::new(3, 1));
        assert!(spot.raw_json.contains("futureField"));

        let usd_m = usd_m_account_information_from_body(
            r#"{
              "totalInitialMargin":"1","totalMaintMargin":"2","totalWalletBalance":"3",
              "totalUnrealizedProfit":"4","totalMarginBalance":"7","totalPositionInitialMargin":"1",
              "totalOpenOrderInitialMargin":"0","totalCrossWalletBalance":"3","totalCrossUnPnl":"4",
              "availableBalance":"6","maxWithdrawAmount":"5",
              "assets":[{"asset":"USDT","walletBalance":"3","unrealizedProfit":"4","marginBalance":"7","maintMargin":"2","initialMargin":"1","positionInitialMargin":"1","openOrderInitialMargin":"0","crossWalletBalance":"3","crossUnPnl":"4","availableBalance":"6","maxWithdrawAmount":"5","updateTime":1700000000000}],
              "positions":[{"symbol":"BTCUSDT","positionSide":"BOTH","positionAmt":"1","unrealizedProfit":"4","isolatedMargin":"0","notional":"100","isolatedWallet":"0","initialMargin":"1","maintMargin":"2","updateTime":1700000000000}],"futureField":true
            }"#,
        )
        .expect("official-shaped USD-M account response");
        assert_eq!(usd_m.total_maintenance_margin, Decimal::from(2));
        assert_eq!(usd_m.assets[0].cross_unrealized_profit, Decimal::from(4));
        assert_eq!(usd_m.positions[0].position_side, "BOTH");
        assert!(usd_m.raw_json.contains("futureField"));
    }

    #[test]
    fn provider_position_and_spot_cancel_contracts_preserve_reports() {
        let positions = usd_m_position_information_from_body(
            r#"[{
              "symbol":"BTCUSDT","positionSide":"LONG","positionAmt":"1","entryPrice":"100",
              "breakEvenPrice":"101","markPrice":"102","unRealizedProfit":"2",
              "liquidationPrice":"50","isolatedMargin":"3","notional":"102","marginAsset":"USDT",
              "isolatedWallet":"4","initialMargin":"5","maintMargin":"1","positionInitialMargin":"5",
              "openOrderInitialMargin":"0","adl":2,"bidNotional":"0","askNotional":"1",
              "updateTime":1700000000000,"futureField":"kept"
            }]"#,
        )
        .expect("official-shaped position response");
        assert_eq!(positions[0].break_even_price, Decimal::from(101));
        assert_eq!(positions[0].adl, 2);
        assert!(positions[0].raw_json.contains("futureField"));

        let cancelled = spot_cancel_all_open_orders_from_body(
            r#"[{
              "symbol":"BTCUSDT","origClientOrderId":"original","orderId":10,"clientOrderId":"client","status":"CANCELED",
              "price":"100","origQty":"2","executedQty":"1","cummulativeQuoteQty":"100",
              "transactTime":1700000000000,"futureField":"kept"
            }]"#,
        )
        .expect("official-shaped cancel response");
        assert_eq!(cancelled.reports[0].order_id.as_deref(), Some("10"));
        assert_eq!(
            cancelled.reports[0].original_client_order_id.as_deref(),
            Some("original")
        );
        assert_eq!(cancelled.reports[0].executed_quantity, Some(Decimal::ONE));
        assert!(cancelled.reports[0].raw_json.contains("futureField"));
    }

    #[test]
    fn a_margin_summary_reads_each_figure_off_the_field_that_means_it() {
        // https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Account-Information-V3
        //
        // Distinct totals make field swaps visible.
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
        // And a position with size survives the common API's filter, which
        // drops the zero-amount rows Binance opens for a symbol carrying only
        // an order.
        assert_eq!(crate::client::open_positions(vec![position]).len(), 1);
    }

    /// A flat `positionRisk` row associated with a resting order.
    const POSITION_RISK_WITH_A_RESTING_ORDER: &str = r#"[{
      "symbol": "XRPUSDT",
      "positionSide": "BOTH",
      "positionAmt": "0.0",
      "entryPrice": "0.0",
      "markPrice": "1.08710784",
      "unRealizedProfit": "0.00000000",
      "notional": "0"
    }]"#;

    /// Flat rows decode successfully and are filtered from open positions.
    #[test]
    fn a_symbol_with_only_a_resting_order_is_not_reported_as_a_position() {
        let raw: Vec<RawPosition> = parse::json(POSITION_RISK_WITH_A_RESTING_ORDER, "positionRisk")
            .expect("the captured payload");

        // The parser preserves the row; the common API removes it.
        let mapped = position(&perp(), &raw[0]).expect("a position");
        assert!(mapped.is_flat());
        assert_eq!(mapped.side, None);
        assert_eq!(mapped.market.kind, MarketKind::Perpetual);

        assert_eq!(
            crate::client::open_positions(vec![mapped]),
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

    /// No request builder uses the removed Spot listen-key endpoints.
    #[test]
    fn no_request_reaches_for_the_removed_spot_listen_key_endpoints() {
        let requests = [
            keepalive_listen_key_request(&spot()).expect("credentials are set"),
            keepalive_listen_key_request(&perp()).expect("credentials are set"),
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

    /// Spot subscribes with a per-request HMAC signature, not a listen-key URL.
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

        // Binance's documented maximum receive window.
        assert_eq!(parsed["params"]["recvWindow"], 60_000);
    }

    #[test]
    fn an_unauthenticated_adapter_never_subscribes_to_a_spot_user_data_stream() {
        assert!(matches!(
            spot_user_data_subscribe_frame(&BinanceAdapter::spot()),
            Err(Error::Auth { .. })
        ));
    }

    /// USD-M user data uses the `/private` entry point and a literal event separator.
    #[test]
    fn the_usd_m_account_socket_names_an_entry_point_and_leaves_the_separator_literal() {
        let key = BinanceListenKey("listen-key".to_string());
        let url = usd_m_user_data_stream_url(&key);

        assert!(
            url.starts_with("wss://fstream.binance.com/private/"),
            "{url}"
        );
        // The separator between event names is a separator, not part of one, so
        // it stays literal where the key beside it is encoded. This key carries
        // no slash of its own, so the one in the query is the separator.
        let query = url.split_once('?').expect("a query").1;
        assert!(query.contains('/'), "{url}");
        assert!(!query.contains("%2F"), "{url}");
    }

    #[test]
    fn keepalive_uses_put_and_never_names_the_key_binance_would_reject() {
        let request = keepalive_listen_key_request(&perp()).expect("credentials are set");

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
    fn close_uses_delete_and_never_names_the_key_binance_would_reject() {
        let request = close_listen_key_request(&perp()).expect("credentials are set");

        assert_eq!(request.method, HttpMethod::Delete);
        assert_eq!(request.target(), "/fapi/v1/listenKey");
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
