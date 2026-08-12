//! Binance's public market data REST API, on both venues.
//!
//! Spot serves these under `/api/v3` and USD-M futures under `/fapi/v1`, with
//! the same path suffixes, the same parameters, and different caps. Request
//! building is kept as plain functions returning [`HttpRequest`] so that every
//! path, query, and rejection below is testable without a network.

use std::cmp::Reverse;

use rust_decimal::Decimal;

use crate::adapters::{
    candles as candle_pages, inclusive_millis_at_or_after, inclusive_millis_before,
};
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::CandleRequest;
use crate::transport::HttpRequest;
use crate::types::{Candle, Market, MarketInfo, MarketKind, OrderBook, Ticker, Timestamp, Trade};

use super::{
    BinanceAdapter, BinanceAggregateTrade, BinanceAggregateTradesRequest, BinanceMarkPrice,
    BinanceMarket, BinanceOpenInterest, EXCHANGE, now_millis, parse,
};

/// The most recent trades either venue will return in one call.
const MAX_TRADE_LIMIT: u32 = 1_000;
/// Binance's maximum number of compressed aggregate trades in one response.
const MAX_AGGREGATE_TRADE_LIMIT: u32 = 1_000;
const AGGREGATE_TRADE_WINDOW_NANOS: i64 = 60 * 60 * 1_000_000_000;

/// The largest spot depth Binance returns in one response.
const MAX_SPOT_DEPTH: u32 = 5_000;

/// The fixed depths USD-M accepts.
const USD_M_DEPTHS: &[u32] = &[5, 10, 20, 50, 100, 500, 1_000];

/// The candles each venue will return in one call. USD-M serves more.
const MAX_SPOT_CANDLES: u32 = 1_000;
const MAX_USD_M_CANDLES: u32 = 1_500;

impl BinanceMarket {
    /// The prefix this venue's public endpoints hang off.
    const fn public_prefix(self) -> &'static str {
        match self {
            Self::Spot => "/api/v3",
            Self::UsdMFutures => "/fapi/v1",
        }
    }

    const fn max_candles(self) -> u32 {
        match self {
            Self::Spot => MAX_SPOT_CANDLES,
            Self::UsdMFutures => MAX_USD_M_CANDLES,
        }
    }
}

/// Percent-encodes a query value.
///
/// Public requests are unsigned, but private ones sign the exact bytes of the
/// query, so both sides encode identically and this is the only encoder.
pub(super) fn encode(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(char::from(byte));
            }
            other => encoded.push_str(&format!("%{other:02X}")),
        }
    }
    encoded
}

/// Joins parameters in the order given.
///
/// Order is preserved, because Binance signs the query text verbatim. A
/// signature covers the bytes, not the set.
pub(super) fn query(params: &[(&str, String)]) -> String {
    params
        .iter()
        .map(|(name, value)| format!("{}={}", name, encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

pub(super) fn markets_request(venue: BinanceMarket) -> HttpRequest {
    HttpRequest::get(format!("{}/exchangeInfo", venue.public_prefix()))
}

pub(super) fn trades_request(
    adapter: &BinanceAdapter,
    market: &Market,
    limit: Option<u32>,
) -> Result<HttpRequest> {
    let mut params = vec![("symbol", adapter.symbol(market)?)];
    if let Some(limit) = limit {
        if !(1..=MAX_TRADE_LIMIT).contains(&limit) {
            return Err(Error::invalid_request(
                "limit",
                format!("binance serves 1 to {MAX_TRADE_LIMIT} trades per call, not {limit}"),
            ));
        }
        params.push(("limit", limit.to_string()));
    }

    Ok(
        HttpRequest::get(format!("{}/trades", adapter.venue().public_prefix()))
            .query(query(&params)),
    )
}

fn check_usd_m_aggregate_trades(adapter: &BinanceAdapter) -> Result<()> {
    if adapter.venue() == BinanceMarket::UsdMFutures {
        return Ok(());
    }
    Err(Error::unsupported(
        Feature::Trades,
        EXCHANGE,
        "compressed aggregate trades are available on the USD-M futures adapter only",
    ))
}

/// Builds one USD-M compressed aggregate-trade request.
pub(super) fn aggregate_trades_request(
    adapter: &BinanceAdapter,
    request: &BinanceAggregateTradesRequest,
) -> Result<HttpRequest> {
    check_usd_m_aggregate_trades(adapter)?;

    let limit = request.limit.unwrap_or(500);
    if !(1..=MAX_AGGREGATE_TRADE_LIMIT).contains(&limit) {
        return Err(Error::invalid_request(
            "limit",
            format!(
                "binance serves 1 to {MAX_AGGREGATE_TRADE_LIMIT} aggregate trades per call, not {limit}"
            ),
        ));
    }
    if request.from_id.is_some() && (request.start_time.is_some() || request.end_time.is_some()) {
        return Err(Error::invalid_request(
            "from_id",
            "Binance aggregate trades use either from_id or start/end time bounds, not both",
        ));
    }
    if request
        .from_id
        .is_some_and(|from_id| from_id > i64::MAX as u64)
    {
        return Err(Error::invalid_request(
            "from_id",
            "Binance aggregate trade IDs must fit the documented signed 64-bit range",
        ));
    }
    let start_millis = request.start_time.map(inclusive_millis_at_or_after);
    let end_millis = request.end_time.map(inclusive_millis_at_or_before);
    if let (Some(start), Some(end)) = (request.start_time, request.end_time) {
        let width = end
            .as_nanos()
            .checked_sub(start.as_nanos())
            .ok_or_else(|| Error::invalid_request("end_time", "must not precede start_time"))?;
        if width < 0 {
            return Err(Error::invalid_request(
                "end_time",
                "must not precede start_time",
            ));
        }
        if width >= AGGREGATE_TRADE_WINDOW_NANOS {
            return Err(Error::invalid_request(
                "end_time",
                "Binance aggregate trade time windows must be shorter than one hour",
            ));
        }
        if start_millis > end_millis {
            return Err(Error::invalid_request(
                "end_time",
                "start_time and end_time do not overlap at millisecond precision",
            ));
        }
    }

    let mut params = vec![("symbol", adapter.symbol(&request.market)?)];
    if let Some(from_id) = request.from_id {
        params.push(("fromId", from_id.to_string()));
    } else {
        if let Some(start) = start_millis {
            params.push(("startTime", start.to_string()));
        }
        if let Some(end) = end_millis {
            params.push(("endTime", end.to_string()));
        }
    }
    // Sending the documented default is explicit and keeps generated clients'
    // request snapshots stable across Binance default changes.
    params.push(("limit", limit.to_string()));

    Ok(HttpRequest::get(format!(
        "{}/aggTrades",
        BinanceMarket::UsdMFutures.public_prefix()
    ))
    .query(query(&params)))
}

fn inclusive_millis_at_or_before(value: Timestamp) -> i64 {
    value.as_nanos().div_euclid(1_000_000)
}

pub(super) fn order_book_request(
    adapter: &BinanceAdapter,
    market: &Market,
    depth: Option<u32>,
) -> Result<HttpRequest> {
    let venue = adapter.venue();
    let mut params = vec![("symbol", adapter.symbol(market)?)];
    if let Some(depth) = depth {
        let accepted = match venue {
            BinanceMarket::Spot => (1..=MAX_SPOT_DEPTH).contains(&depth),
            BinanceMarket::UsdMFutures => USD_M_DEPTHS.contains(&depth),
        };
        if !accepted {
            let expected = match venue {
                BinanceMarket::Spot => format!("1 to {MAX_SPOT_DEPTH}"),
                BinanceMarket::UsdMFutures => format!("one of {USD_M_DEPTHS:?}"),
            };
            return Err(Error::invalid_request(
                "depth",
                format!("binance serves book depths {expected} on this venue, not {depth}"),
            ));
        }
        params.push(("limit", depth.to_string()));
    }

    Ok(HttpRequest::get(format!("{}/depth", venue.public_prefix())).query(query(&params)))
}

pub(super) fn ticker_request(adapter: &BinanceAdapter, market: &Market) -> Result<HttpRequest> {
    let params = [("symbol", adapter.symbol(market)?)];

    Ok(
        HttpRequest::get(format!("{}/ticker/24hr", adapter.venue().public_prefix()))
            .query(query(&params)),
    )
}

fn check_usd_m_market_data(adapter: &BinanceAdapter, what: &str) -> Result<()> {
    if adapter.venue() == BinanceMarket::UsdMFutures {
        return Ok(());
    }
    Err(Error::unsupported(
        Feature::FundingRates,
        EXCHANGE,
        format!("{what} is available on the USD-M futures adapter only"),
    ))
}

pub(super) fn mark_price_request(adapter: &BinanceAdapter, market: &Market) -> Result<HttpRequest> {
    check_usd_m_market_data(adapter, "mark price")?;
    let params = [("symbol", adapter.symbol(market)?)];
    Ok(HttpRequest::get(format!(
        "{}/premiumIndex",
        BinanceMarket::UsdMFutures.public_prefix()
    ))
    .query(query(&params)))
}

pub(super) fn mark_prices_request(adapter: &BinanceAdapter) -> Result<HttpRequest> {
    check_usd_m_market_data(adapter, "mark prices")?;
    Ok(HttpRequest::get(format!(
        "{}/premiumIndex",
        BinanceMarket::UsdMFutures.public_prefix()
    )))
}

pub(super) fn open_interest_request(
    adapter: &BinanceAdapter,
    market: &Market,
) -> Result<HttpRequest> {
    check_usd_m_market_data(adapter, "open interest")?;
    let params = [("symbol", adapter.symbol(market)?)];
    Ok(HttpRequest::get(format!(
        "{}/openInterest",
        BinanceMarket::UsdMFutures.public_prefix()
    ))
    .query(query(&params)))
}

/// One page of candles, ending at `cursor` and reaching at most `count` back.
///
/// `startTime` is deliberately not sent. Binance reads a window from its start
/// when both ends are given, while the walk in
/// [`crate::adapters::candles::read`] needs the newest `count` before a cursor.
/// The start of the caller's window is honoured by where the walk stops.
pub(super) fn candles_request(
    adapter: &BinanceAdapter,
    request: &CandleRequest,
    cursor: Option<Timestamp>,
    count: u32,
) -> Result<HttpRequest> {
    let venue = adapter.venue();
    let mut params = vec![
        ("symbol", adapter.symbol(&request.market)?),
        (
            "interval",
            venue.interval_code(request.interval)?.to_string(),
        ),
    ];
    if let Some(cursor) = cursor {
        // Binance includes `endTime`; convert the exclusive nanosecond cursor
        // to the latest whole millisecond still inside the requested window.
        params.push(("endTime", inclusive_millis_before(cursor).to_string()));
    }
    params.push(("limit", count.to_string()));

    Ok(HttpRequest::get(format!("{}/klines", venue.public_prefix())).query(query(&params)))
}

pub(super) async fn markets(adapter: &BinanceAdapter, kind: MarketKind) -> Result<Vec<MarketInfo>> {
    // Each venue lists one kind. Asking a spot adapter for perpetuals is a
    // meaningful question whose answer is "none", so it is not an error.
    if kind != adapter.venue().market_kind() {
        return Ok(Vec::new());
    }

    let body = adapter.send(markets_request(adapter.venue())).await?;
    let listing: parse::RawExchangeInfo = parse::json(&body, "exchangeInfo")?;
    Ok(listing
        .symbols
        .iter()
        .filter_map(|symbol| parse::market_info(adapter.venue(), symbol))
        .collect())
}

pub(super) async fn trades(
    adapter: &BinanceAdapter,
    market: &Market,
    limit: Option<u32>,
) -> Result<Vec<Trade>> {
    let body = adapter
        .send(trades_request(adapter, market, limit)?)
        .await?;

    newest_first(market, parse::json(&body, "trades")?)
}

pub(super) async fn aggregate_trades(
    adapter: &BinanceAdapter,
    request: &BinanceAggregateTradesRequest,
) -> Result<Vec<BinanceAggregateTrade>> {
    let body = adapter
        .send(aggregate_trades_request(adapter, request)?)
        .await?;
    let raw: Vec<parse::RawAggregateTrade> = parse::json(&body, "aggregate trades")?;
    raw.iter()
        .map(|trade| parse::aggregate_trade(&request.market, trade))
        .collect()
}

/// Puts a trades payload in the order the common API promises.
///
/// Both `/api/v3/trades` and `/fapi/v1/trades` answer oldest-first, ascending
/// by trade id, and [`Client::trades`](crate::Client::trades) promises newest
/// first. Sorting on the id keeps the order exact inside one millisecond,
/// where several trades routinely share a timestamp, and does not assume the
/// payload arrived sorted.
fn newest_first(market: &Market, mut raw: Vec<parse::RawTrade>) -> Result<Vec<Trade>> {
    raw.sort_unstable_by_key(|trade| Reverse(trade.id));
    raw.iter().map(|raw| parse::trade(market, raw)).collect()
}

pub(super) async fn order_book(
    adapter: &BinanceAdapter,
    market: &Market,
    depth: Option<u32>,
) -> Result<OrderBook> {
    let body = adapter
        .send(order_book_request(adapter, market, depth)?)
        .await?;
    let raw: parse::RawDepth = parse::json(&body, "depth")?;
    // Spot depth carries no clock, so it uses the response read time. USD-M's
    // exchange timestamp overrides this fallback.
    parse::order_book(market, Timestamp::now(), &raw)
}

pub(super) async fn ticker(adapter: &BinanceAdapter, market: &Market) -> Result<Ticker> {
    let body = adapter.send(ticker_request(adapter, market)?).await?;
    let raw: parse::RawTicker = parse::json(&body, "ticker")?;
    parse::ticker(market, &raw)
}

pub(super) async fn mark_price(
    adapter: &BinanceAdapter,
    market: &Market,
) -> Result<BinanceMarkPrice> {
    let body = adapter.send(mark_price_request(adapter, market)?).await?;
    let raw: parse::RawMarkPrice = parse::json(&body, "mark price")?;
    parse::mark_price(market, &raw)
}

pub(super) async fn mark_prices(adapter: &BinanceAdapter) -> Result<Vec<BinanceMarkPrice>> {
    let request = mark_prices_request(adapter)?;
    let markets = markets(adapter, MarketKind::Perpetual).await?;
    let body = adapter.send(request).await?;
    let raw: Vec<parse::RawMarkPrice> = parse::json(&body, "mark prices")?;
    mark_price_list(&markets, &raw)
}

fn mark_price_list(
    markets: &[MarketInfo],
    raw: &[parse::RawMarkPrice],
) -> Result<Vec<BinanceMarkPrice>> {
    raw.iter()
        .filter_map(|entry| {
            markets
                .iter()
                .find(|market| market.native_symbol == entry.symbol)
                .map(|market| parse::mark_price(&market.market, entry))
        })
        .collect()
}

pub(super) async fn open_interest(
    adapter: &BinanceAdapter,
    market: &Market,
) -> Result<BinanceOpenInterest> {
    let body = adapter
        .send(open_interest_request(adapter, market)?)
        .await?;
    let raw: parse::RawOpenInterest = parse::json(&body, "open interest")?;
    parse::open_interest(market, &raw)
}

/// Reads candles, oldest first, paging when one response cannot hold the
/// answer.
///
/// Binance caps a klines response at [`BinanceMarket::max_candles`], which
/// differs per venue and bounds the response, not the request. A larger `limit`
/// becomes several calls, walked by [`crate::adapters::candles::read`] exactly
/// as on every other exchange.
pub(super) async fn candles(
    adapter: &BinanceAdapter,
    request: &CandleRequest,
) -> Result<Vec<Candle>> {
    let max = adapter.venue().max_candles();
    let now = now_millis();

    candle_pages::read(request, EXCHANGE, max, |cursor, count| async move {
        let body = adapter
            .send(candles_request(adapter, request, cursor, count)?)
            .await?;

        parse::json::<Vec<parse::RawCandle>>(&body, "klines")?
            .iter()
            .map(|raw| parse::candle(&request.market, request.interval, raw, now))
            .collect()
    })
    .await
}

/// Binance Spot order filters for one symbol.
///
/// Read through [`Client::adapter`](crate::Client::adapter). A field is `None`
/// when the symbol has no filter that supplies it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceSymbolFilters {
    /// The symbol these rules apply to, verbatim. For example `BTCUSDT`.
    pub symbol: String,
    /// Prices must be a whole multiple of this, from `PRICE_FILTER`.
    pub tick_size: Option<Decimal>,
    /// Lowest accepted price, from `PRICE_FILTER`.
    pub min_price: Option<Decimal>,
    /// Highest accepted price, from `PRICE_FILTER`.
    pub max_price: Option<Decimal>,
    /// Quantities must be a whole multiple of this, from `LOT_SIZE`.
    pub step_size: Option<Decimal>,
    /// Smallest accepted quantity, from `LOT_SIZE`.
    pub min_quantity: Option<Decimal>,
    /// Largest accepted quantity, from `LOT_SIZE`.
    pub max_quantity: Option<Decimal>,
    /// Smallest accepted price times quantity, from `NOTIONAL`.
    pub min_notional: Option<Decimal>,
}

/// Reads the filters off one `exchangeInfo` listing.
pub(super) fn symbol_filters(raw: &parse::RawSymbol) -> Result<BinanceSymbolFilters> {
    let mut filters = BinanceSymbolFilters {
        symbol: raw.symbol.clone(),
        tick_size: None,
        min_price: None,
        max_price: None,
        step_size: None,
        min_quantity: None,
        max_quantity: None,
        min_notional: None,
    };

    let read = |text: &Option<String>, field: &'static str| -> Result<Option<Decimal>> {
        text.as_deref()
            .map(|value| parse::decimal(value, field))
            .transpose()
    };

    for filter in &raw.filters {
        match filter.filter_type.as_str() {
            "PRICE_FILTER" => {
                filters.tick_size = read(&filter.tick_size, "tickSize")?;
                filters.min_price = read(&filter.min_price, "minPrice")?;
                filters.max_price = read(&filter.max_price, "maxPrice")?;
            }
            "LOT_SIZE" => {
                filters.step_size = read(&filter.step_size, "stepSize")?;
                filters.min_quantity = read(&filter.min_qty, "minQty")?;
                filters.max_quantity = read(&filter.max_qty, "maxQty")?;
            }
            // Binance renamed `MIN_NOTIONAL` to `NOTIONAL` and still serves the
            // old spelling on some symbols.
            "NOTIONAL" | "MIN_NOTIONAL" => {
                filters.min_notional = read(&filter.min_notional, "minNotional")?;
            }
            // Filters `maxt` does not carry: order counts, percent price bands,
            // iceberg parts. Skipping them is not a loss of a value the caller
            // asked for.
            _ => {}
        }
    }

    Ok(filters)
}

pub(super) async fn spot_symbol_filters(
    adapter: &BinanceAdapter,
    market: &Market,
) -> Result<BinanceSymbolFilters> {
    if adapter.venue() != BinanceMarket::Spot {
        return Err(Error::unsupported(
            Feature::Markets,
            EXCHANGE,
            "symbol filters are read here for spot; USD-M publishes a different filter set",
        ));
    }

    let symbol = adapter.symbol(market)?;
    let body = adapter
        .send(markets_request(BinanceMarket::Spot).query(query(&[("symbol", symbol.clone())])))
        .await?;
    let listing: parse::RawExchangeInfo = parse::json(&body, "exchangeInfo")?;
    let raw = listing
        .symbols
        .iter()
        .find(|entry| entry.symbol == symbol)
        .ok_or_else(|| Error::decode(format!("binance did not list `{symbol}`")))?;

    symbol_filters(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, Interval, MarketStatus};

    fn spot() -> BinanceAdapter {
        BinanceAdapter::spot()
    }

    fn perp() -> BinanceAdapter {
        BinanceAdapter::usd_m_futures()
    }

    fn btc_usdt() -> Market {
        Market::spot(Exchange::Binance, "BTC", "USDT")
    }

    fn btc_usdt_perp() -> Market {
        Market::perpetual(Exchange::Binance, "BTC", "USDT")
    }

    #[test]
    fn the_two_venues_serve_the_same_endpoints_under_different_prefixes() {
        assert_eq!(
            markets_request(BinanceMarket::Spot).target(),
            "/api/v3/exchangeInfo"
        );
        assert_eq!(
            markets_request(BinanceMarket::UsdMFutures).target(),
            "/fapi/v1/exchangeInfo"
        );
        assert_eq!(
            trades_request(&spot(), &btc_usdt(), Some(10))
                .expect("a valid limit")
                .target(),
            "/api/v3/trades?symbol=BTCUSDT&limit=10"
        );
        assert_eq!(
            trades_request(&perp(), &btc_usdt_perp(), Some(10))
                .expect("a valid limit")
                .target(),
            "/fapi/v1/trades?symbol=BTCUSDT&limit=10"
        );
        assert_eq!(
            ticker_request(&spot(), &btc_usdt())
                .expect("a market")
                .target(),
            "/api/v3/ticker/24hr?symbol=BTCUSDT"
        );
        assert_eq!(
            order_book_request(&spot(), &btc_usdt(), Some(100))
                .expect("a valid depth")
                .target(),
            "/api/v3/depth?symbol=BTCUSDT&limit=100"
        );
        assert_eq!(
            mark_price_request(&perp(), &btc_usdt_perp())
                .expect("a USD-M market")
                .target(),
            "/fapi/v1/premiumIndex?symbol=BTCUSDT"
        );
        assert_eq!(
            mark_prices_request(&perp())
                .expect("a USD-M adapter")
                .target(),
            "/fapi/v1/premiumIndex"
        );
        assert_eq!(
            open_interest_request(&perp(), &btc_usdt_perp())
                .expect("a USD-M market")
                .target(),
            "/fapi/v1/openInterest?symbol=BTCUSDT"
        );
    }

    #[test]
    fn aggregate_trades_use_usd_m_and_preserve_binances_query_order() {
        let request = BinanceAggregateTradesRequest::new(btc_usdt_perp())
            .with_from_id(26129)
            .limit(50);
        assert_eq!(
            aggregate_trades_request(&perp(), &request)
                .expect("a valid aggregate-trade request")
                .target(),
            "/fapi/v1/aggTrades?symbol=BTCUSDT&fromId=26129&limit=50"
        );

        let timed = BinanceAggregateTradesRequest::new(btc_usdt_perp())
            .start_time(Timestamp::from_nanos(1_623_319_461_670_500_000))
            .end_time(Timestamp::from_nanos(1_623_319_462_670_499_999));
        assert_eq!(
            aggregate_trades_request(&perp(), &timed)
                .expect("a valid time window")
                .target(),
            "/fapi/v1/aggTrades?symbol=BTCUSDT&startTime=1623319461671&endTime=1623319462670&limit=500"
        );
    }

    #[test]
    fn aggregate_trades_reject_wrong_venue_and_unsafe_windows_before_network() {
        let spot_request = BinanceAggregateTradesRequest::new(btc_usdt());
        assert!(matches!(
            aggregate_trades_request(&spot(), &spot_request),
            Err(Error::Unsupported {
                feature: Feature::Trades,
                ..
            })
        ));

        let mixed = BinanceAggregateTradesRequest::new(btc_usdt_perp())
            .with_from_id(10)
            .start_time(Timestamp::from_millis(1_000));
        assert!(matches!(
            aggregate_trades_request(&perp(), &mixed),
            Err(Error::InvalidRequest { field, .. }) if field == "from_id"
        ));

        let out_of_range =
            BinanceAggregateTradesRequest::new(btc_usdt_perp()).with_from_id(i64::MAX as u64 + 1);
        assert!(matches!(
            aggregate_trades_request(&perp(), &out_of_range),
            Err(Error::InvalidRequest { field, .. }) if field == "from_id"
        ));

        let too_wide = BinanceAggregateTradesRequest::new(btc_usdt_perp())
            .start_time(Timestamp::from_millis(1_000))
            .end_time(Timestamp::from_millis(3_601_000));
        assert!(matches!(
            aggregate_trades_request(&perp(), &too_wide),
            Err(Error::InvalidRequest { field, .. }) if field == "end_time"
        ));

        for request in [
            BinanceAggregateTradesRequest::new(btc_usdt_perp()).limit(0),
            BinanceAggregateTradesRequest::new(btc_usdt_perp()).limit(1_001),
        ] {
            assert!(matches!(
                aggregate_trades_request(&perp(), &request),
                Err(Error::InvalidRequest { .. })
            ));
        }
    }

    #[test]
    fn aggregate_trades_preserve_ids_rpi_quantity_and_taker_side() {
        let raw: Vec<parse::RawAggregateTrade> = parse::json(
            r#"[
              {"a":26129,"p":"0.01633102","q":"4.70443515","nq":"4.00000000",
               "f":27781,"l":27784,"T":1498793709153,"m":true},
              {"a":26130,"p":"0.01633103","q":"1.2",
               "f":27785,"l":27785,"T":1498793709253,"m":false}
            ]"#,
            "aggregate trades",
        )
        .expect("official aggregate-trade payload");
        let trades: Vec<_> = raw
            .iter()
            .map(|entry| parse::aggregate_trade(&btc_usdt_perp(), entry))
            .collect::<Result<_>>()
            .expect("aggregate trades");

        assert_eq!(trades[0].aggregate_id, 26129);
        assert_eq!(trades[0].first_trade_id, 27781);
        assert_eq!(trades[0].last_trade_id, 27784);
        assert_eq!(trades[0].quantity.to_string(), "4.70443515");
        assert_eq!(
            trades[0].normal_quantity.expect("nq").to_string(),
            "4.00000000"
        );
        assert_eq!(trades[0].taker_side, crate::types::Side::Sell);
        assert_eq!(trades[1].normal_quantity, None);
        assert_eq!(trades[1].taker_side, crate::types::Side::Buy);

        let invalid = parse::RawAggregateTrade {
            aggregate_id: 1,
            price: "1".to_owned(),
            quantity: "1".to_owned(),
            normal_quantity: None,
            first_trade_id: 3,
            last_trade_id: 2,
            time: 0,
            is_buyer_maker: false,
        };
        assert!(matches!(
            parse::aggregate_trade(&btc_usdt_perp(), &invalid),
            Err(Error::Decode { .. })
        ));
    }

    #[test]
    fn mark_price_and_open_interest_are_usd_m_only() {
        assert!(matches!(
            mark_price_request(&spot(), &btc_usdt()),
            Err(Error::Unsupported { feature, .. }) if feature == Feature::FundingRates
        ));
        assert!(matches!(
            mark_prices_request(&spot()),
            Err(Error::Unsupported { feature, .. }) if feature == Feature::FundingRates
        ));
        assert!(matches!(
            open_interest_request(&spot(), &btc_usdt()),
            Err(Error::Unsupported { feature, .. }) if feature == Feature::FundingRates
        ));
    }

    #[test]
    fn mark_price_list_keeps_only_exchange_info_perpetuals() {
        let raw: Vec<parse::RawMarkPrice> = parse::json(
            r#"[
              {"symbol":"BTCUSDT","markPrice":"1","indexPrice":"1","estimatedSettlePrice":"0","lastFundingRate":"0","interestRate":"0","nextFundingTime":0,"time":0},
              {"symbol":"BTCU","markPrice":"1","indexPrice":"1","estimatedSettlePrice":"0","lastFundingRate":"0","interestRate":"0","nextFundingTime":0,"time":0},
              {"symbol":"TRADIFIUSDT","markPrice":"1","indexPrice":"1","estimatedSettlePrice":"0","lastFundingRate":"0","interestRate":"0","nextFundingTime":0,"time":0},
              {"symbol":"BTCUSDT_260925","markPrice":"1","indexPrice":"1","estimatedSettlePrice":"0","lastFundingRate":"0","interestRate":"0","nextFundingTime":0,"time":0}
            ]"#,
            "mark prices",
        )
        .expect("Binance mark-price array");
        let markets = vec![
            MarketInfo {
                market: btc_usdt_perp(),
                native_symbol: "BTCUSDT".to_owned(),
                status: MarketStatus::Active,
                korean_name: None,
                english_name: None,
            },
            MarketInfo {
                market: Market::perpetual(Exchange::Binance, "BTC", "U"),
                native_symbol: "BTCU".to_owned(),
                status: MarketStatus::Active,
                korean_name: None,
                english_name: None,
            },
        ];

        let prices = mark_price_list(&markets, &raw).expect("perpetual prices");

        assert_eq!(
            prices.iter().map(|price| &price.market).collect::<Vec<_>>(),
            vec![
                &btc_usdt_perp(),
                &Market::perpetual(Exchange::Binance, "BTC", "U")
            ]
        );
    }

    #[test]
    fn spot_accepts_any_depth_through_5000_while_futures_keeps_its_fixed_set() {
        assert_eq!(
            order_book_request(&spot(), &btc_usdt(), Some(30))
                .expect("spot accepts every depth through 5000")
                .target(),
            "/api/v3/depth?symbol=BTCUSDT&limit=30"
        );
        // 5000 levels are a spot-only depth.
        assert!(order_book_request(&spot(), &btc_usdt(), Some(5_000)).is_ok());
        assert!(matches!(
            order_book_request(&spot(), &btc_usdt(), Some(5_001)),
            Err(Error::InvalidRequest { field, .. }) if field == "depth"
        ));
        assert!(matches!(
            order_book_request(&perp(), &btc_usdt_perp(), Some(30)),
            Err(Error::InvalidRequest { field, .. }) if field == "depth"
        ));
        assert!(matches!(
            order_book_request(&perp(), &btc_usdt_perp(), Some(5_000)),
            Err(Error::InvalidRequest { field, .. }) if field == "depth"
        ));
    }

    #[test]
    fn each_venue_asks_for_no_more_than_it_serves_in_one_response() {
        // The caps differ per venue and belong to one response, not to the
        // request: a larger `limit` is paged, so nothing here is refused.
        assert_eq!(spot().venue().max_candles(), 1_000);
        assert_eq!(perp().venue().max_candles(), 1_500);
    }

    #[test]
    fn a_candle_window_is_half_open_on_the_wire_as_well_as_in_the_request() {
        let request = CandleRequest::new(btc_usdt(), Interval::Min15);

        // `endTime` is inclusive on Binance, so it stops one millisecond short.
        assert_eq!(
            candles_request(
                &spot(),
                &request,
                Some(Timestamp::from_millis(1_499_644_800_000)),
                250
            )
            .expect("a valid request")
            .target(),
            "/api/v3/klines?symbol=BTCUSDT&interval=15m&endTime=1499644799999&limit=250"
        );

        assert_eq!(
            candles_request(
                &spot(),
                &request,
                Some(Timestamp::from_nanos(1_499_644_800_000_000_001)),
                250
            )
            .expect("a sub-millisecond exclusive end")
            .target(),
            "/api/v3/klines?symbol=BTCUSDT&interval=15m&endTime=1499644800000&limit=250"
        );
    }

    #[test]
    fn a_one_second_candle_request_reaches_spot_and_is_refused_on_futures() {
        let spot_request = CandleRequest::new(btc_usdt(), Interval::Sec1);
        let perp_request = CandleRequest::new(btc_usdt_perp(), Interval::Sec1);

        assert_eq!(
            candles_request(&spot(), &spot_request, None, 500)
                .expect("spot serves one-second candles")
                .target(),
            "/api/v3/klines?symbol=BTCUSDT&interval=1s&limit=500"
        );
        // One second is outside the baseline every adapter serves, and USD-M
        // does not aggregate it.
        assert!(matches!(
            candles_request(&perp(), &perp_request, None, 500),
            Err(Error::Unsupported {
                feature: Feature::Candles,
                ..
            })
        ));
    }

    #[test]
    fn every_interval_in_the_common_baseline_reaches_both_venues() {
        // What `supports(Feature::Candles) == true` is worth: the eight
        // intervals every `maxt` adapter serves, on either venue.
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
                candles_request(&spot(), &CandleRequest::new(btc_usdt(), interval), None, 1)
                    .is_ok(),
                "spot refused {interval:?}"
            );
            assert!(
                candles_request(
                    &perp(),
                    &CandleRequest::new(btc_usdt_perp(), interval),
                    None,
                    1
                )
                .is_ok(),
                "usd-m refused {interval:?}"
            );
        }
    }

    #[test]
    fn a_market_from_the_other_venue_never_reaches_the_wire() {
        assert!(trades_request(&spot(), &btc_usdt_perp(), None).is_err());
        assert!(ticker_request(&perp(), &btc_usdt()).is_err());
        assert!(
            order_book_request(&spot(), &Market::spot(Exchange::Upbit, "BTC", "KRW"), None)
                .is_err()
        );
    }

    #[test]
    fn every_limit_above_binances_cap_is_refused_rather_than_clamped() {
        assert!(matches!(
            trades_request(&spot(), &btc_usdt(), Some(1_001)),
            Err(Error::InvalidRequest { field, .. }) if field == "limit"
        ));
        assert!(matches!(
            trades_request(&spot(), &btc_usdt(), Some(0)),
            Err(Error::InvalidRequest { field, .. }) if field == "limit"
        ));
        assert!(trades_request(&spot(), &btc_usdt(), Some(MAX_TRADE_LIMIT)).is_ok());
    }

    #[test]
    fn a_query_value_is_percent_encoded_the_same_way_signing_will_see_it() {
        assert_eq!(encode("BTCUSDT"), "BTCUSDT");
        assert_eq!(encode("a b&c=d"), "a%20b%26c%3Dd");
        assert_eq!(encode("0.1"), "0.1");
        assert_eq!(
            query(&[("a", "1".to_string()), ("b", "x y".to_string())]),
            "a=1&b=x%20y"
        );
    }

    #[test]
    fn a_non_ascii_spot_asset_is_percent_encoded_in_public_rest_queries() {
        let market = Market::spot(Exchange::Binance, "币安人生", "USDT");

        assert_eq!(
            trades_request(&spot(), &market, Some(1))
                .expect("Binance lists UTF-8 asset names")
                .target(),
            "/api/v3/trades?symbol=%E5%B8%81%E5%AE%89%E4%BA%BA%E7%94%9FUSDT&limit=1"
        );
    }

    #[test]
    fn recent_trades_come_back_newest_first_whichever_order_binance_sent() {
        // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints
        // Binance answers ascending by trade id, which is the opposite of what
        // the common API promises, so the ids below run 28457, 28458, 28459 on
        // the wire and must come back 28459, 28458, 28457.
        const ASCENDING: &str = r#"[
          {"id":28457,"price":"4.00000100","qty":"12.00000000","quoteQty":"48.000012",
           "time":1499865549590,"isBuyerMaker":true,"isBestMatch":true},
          {"id":28458,"price":"4.00000200","qty":"1.00000000","quoteQty":"4.000002",
           "time":1499865549590,"isBuyerMaker":false,"isBestMatch":true},
          {"id":28459,"price":"4.00000300","qty":"2.00000000","quoteQty":"8.000006",
           "time":1499865549712,"isBuyerMaker":true,"isBestMatch":true}
        ]"#;

        let raw: Vec<parse::RawTrade> =
            parse::json(ASCENDING, "trades").expect("official trades payload");
        assert_eq!(
            raw.iter().map(|entry| entry.id).collect::<Vec<_>>(),
            vec![28457, 28458, 28459],
            "the payload under test must be in binance's own order"
        );

        let trades = newest_first(&btc_usdt(), raw).expect("three trades");

        assert_eq!(
            trades
                .iter()
                .map(|trade| trade.id.clone().unwrap_or_default())
                .collect::<Vec<_>>(),
            vec!["28459", "28458", "28457"]
        );
        // 28458 and 28457 share a millisecond, so the timestamps alone do not
        // pin the order and the trade id has to.
        assert!(
            trades
                .windows(2)
                .all(|pair| pair[0].timestamp >= pair[1].timestamp)
        );
    }

    #[test]
    fn spot_filters_are_read_off_the_listing_by_name() {
        // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-endpoints
        let listing: parse::RawExchangeInfo = parse::json(
            r#"{
              "symbols": [
                {
                  "symbol": "BTCUSDT",
                  "status": "TRADING",
                  "baseAsset": "BTC",
                  "quoteAsset": "USDT",
                  "filters": [
                    {
                      "filterType": "PRICE_FILTER",
                      "minPrice": "0.01000000",
                      "maxPrice": "1000000.00000000",
                      "tickSize": "0.01000000"
                    },
                    {
                      "filterType": "LOT_SIZE",
                      "minQty": "0.00001000",
                      "maxQty": "9000.00000000",
                      "stepSize": "0.00001000"
                    },
                    {
                      "filterType": "NOTIONAL",
                      "minNotional": "5.00000000",
                      "applyMinToMarket": true
                    },
                    { "filterType": "MAX_NUM_ORDERS", "maxNumOrders": 200 }
                  ]
                }
              ]
            }"#,
            "exchangeInfo",
        )
        .expect("official listing payload");

        let filters = symbol_filters(&listing.symbols[0]).expect("a filter set");

        assert_eq!(filters.symbol, "BTCUSDT");
        assert_eq!(filters.tick_size, Some(Decimal::new(1_000_000, 8)));
        assert_eq!(filters.step_size, Some(Decimal::new(1_000, 8)));
        assert_eq!(filters.min_notional, Some(Decimal::new(500_000_000, 8)));
        assert_eq!(filters.max_quantity, Some(Decimal::new(900_000_000_000, 8)));
        // A symbol carries no filter for the rules `maxt` does not model.
        assert_eq!(
            symbol_filters(&parse::json::<parse::RawExchangeInfo>(
                r#"{"symbols":[{"symbol":"NEWUSDT","status":"TRADING","baseAsset":"NEW","quoteAsset":"USDT","filters":[]}]}"#,
                "exchangeInfo"
            )
            .expect("a listing")
            .symbols[0])
                .expect("a filter set")
                .tick_size,
            None
        );
    }
}
