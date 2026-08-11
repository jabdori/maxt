//! Upbit's public quotation REST API.

use std::cmp::Reverse;

use rust_decimal::Decimal;

use crate::adapters::candles as candle_pages;
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::CandleRequest;
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{
    Candle, Interval, Market, MarketInfo, MarketKind, OrderBook, Ticker, Timestamp, Trade,
};

use super::parse::{self, EXCHANGE};
use super::{UpbitOrderBookInstrument, UpbitYearCandle};

/// Upbit returns at most this many trade ticks per call.
const MAX_TRADE_COUNT: u32 = 500;
/// Upbit returns at most this many book levels per side.
const MAX_BOOK_DEPTH: u32 = 30;
/// Upbit returns at most this many candles per call.
pub(crate) const MAX_CANDLE_COUNT: u32 = 200;

/// Percent-encodes a query value using the RFC 3986 unreserved set.
fn encode(raw: &str) -> String {
    raw.bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                char::from(byte).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn query(params: &[(&str, String)]) -> String {
    params
        .iter()
        .map(|(key, value)| format!("{key}={}", encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

pub(crate) fn markets_request() -> HttpRequest {
    // Detailed listings include the region-specific warning fields.
    HttpRequest::get("/v1/market/all").query("is_details=true")
}

pub(crate) fn trades_request(market: &Market, limit: Option<u32>) -> Result<HttpRequest> {
    let mut params = vec![("market", parse::native_symbol(market)?)];
    if let Some(limit) = limit {
        if !(1..=MAX_TRADE_COUNT).contains(&limit) {
            return Err(Error::invalid_request(
                "limit",
                format!("upbit serves 1 to {MAX_TRADE_COUNT} trades per call, not {limit}"),
            ));
        }
        params.push(("count", limit.to_string()));
    }

    Ok(HttpRequest::get("/v1/trades/ticks").query(query(&params)))
}

pub(crate) fn order_book_request(markets: &[Market], depth: Option<u32>) -> Result<HttpRequest> {
    order_book_request_with_level(markets, None, depth)
}

/// Builds an Upbit Korea aggregated-orderbook request.
///
/// The provider accepts `0` for an unaggregated book. Positive values are live
/// market metadata and should come from `orderbook_instruments`.
pub(crate) fn order_book_with_level_request(
    markets: &[Market],
    level: Decimal,
    depth: Option<u32>,
) -> Result<HttpRequest> {
    if level < Decimal::ZERO {
        return Err(Error::invalid_request(
            "level",
            "Upbit order-book aggregation level must be zero or positive",
        ));
    }

    order_book_request_with_level(markets, Some(level), depth)
}

fn order_book_request_with_level(
    markets: &[Market],
    level: Option<Decimal>,
    depth: Option<u32>,
) -> Result<HttpRequest> {
    if markets.is_empty() {
        return Err(Error::invalid_request(
            "markets",
            "name at least one market",
        ));
    }

    let codes = markets
        .iter()
        .map(parse::native_symbol)
        .collect::<Result<Vec<_>>>()?
        .join(",");
    let mut params = vec![("markets", codes)];
    if let Some(level) = level {
        params.push(("level", level.normalize().to_string()));
    }
    if let Some(depth) = depth {
        if !(1..=MAX_BOOK_DEPTH).contains(&depth) {
            return Err(Error::invalid_request(
                "depth",
                format!("upbit serves 1 to {MAX_BOOK_DEPTH} book levels per side, not {depth}"),
            ));
        }
        params.push(("count", depth.to_string()));
    }

    Ok(HttpRequest::get("/v1/orderbook").query(query(&params)))
}

pub(crate) fn ticker_request(markets: &[Market]) -> Result<HttpRequest> {
    if markets.is_empty() {
        return Err(Error::invalid_request(
            "markets",
            "name at least one market",
        ));
    }

    let codes = markets
        .iter()
        .map(parse::native_symbol)
        .collect::<Result<Vec<_>>>()?
        .join(",");

    Ok(HttpRequest::get("/v1/ticker").query(query(&[("markets", codes)])))
}

pub(crate) fn tickers_by_quote_request(quote_currencies: &[String]) -> Result<HttpRequest> {
    let currencies = quote_currencies_parameter(quote_currencies, "quote_currencies")?;

    Ok(HttpRequest::get("/v1/ticker/all").query(query(&[("quote_currencies", currencies)])))
}

pub(crate) fn quote_currencies_parameter(
    quote_currencies: &[String],
    field: &'static str,
) -> Result<String> {
    if quote_currencies.is_empty() {
        return Err(Error::invalid_request(
            field,
            "name at least one quote currency",
        ));
    }

    Ok(quote_currencies
        .iter()
        .map(|currency| {
            let currency = currency.trim().to_ascii_uppercase();
            if currency.is_empty()
                || !currency
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
            {
                return Err(Error::invalid_request(
                    field,
                    "quote currencies must contain only uppercase ASCII letters and digits",
                ));
            }
            Ok(currency)
        })
        .collect::<Result<Vec<_>>>()?
        .join(","))
}

pub(crate) fn year_candles_request(
    market: &Market,
    to: Option<Timestamp>,
    count: Option<u32>,
) -> Result<HttpRequest> {
    let mut params = vec![("market", parse::native_symbol(market)?)];
    if let Some(to) = to {
        params.push(("to", parse::to_cursor(to)?));
    }
    if let Some(count) = count {
        if !(1..=MAX_CANDLE_COUNT).contains(&count) {
            return Err(Error::invalid_request(
                "count",
                format!(
                    "upbit serves 1 to {MAX_CANDLE_COUNT} yearly candles per call, not {count}"
                ),
            ));
        }
        params.push(("count", count.to_string()));
    }

    Ok(HttpRequest::get("/v1/candles/years").query(query(&params)))
}

pub(crate) fn orderbook_instruments_request(markets: &[Market]) -> Result<HttpRequest> {
    if markets.is_empty() {
        return Err(Error::invalid_request(
            "markets",
            "name at least one market",
        ));
    }

    let codes = markets
        .iter()
        .map(parse::native_symbol)
        .collect::<Result<Vec<_>>>()?
        .join(",");

    Ok(HttpRequest::get("/v1/orderbook/instruments").query(query(&[("markets", codes)])))
}

/// Returns the candle endpoint for an interval exposed by `maxt`.
///
/// Upbit also provides yearly candles, but [`Interval`] has no corresponding
/// variant. Other unmapped intervals are not available from Upbit. One-second
/// history is limited to the most recent three months.
pub(crate) fn candle_path(interval: Interval) -> Option<&'static str> {
    Some(match interval {
        Interval::Sec1 => "/v1/candles/seconds",
        Interval::Min1 => "/v1/candles/minutes/1",
        Interval::Min3 => "/v1/candles/minutes/3",
        Interval::Min5 => "/v1/candles/minutes/5",
        Interval::Min10 => "/v1/candles/minutes/10",
        Interval::Min15 => "/v1/candles/minutes/15",
        Interval::Min30 => "/v1/candles/minutes/30",
        Interval::Hour1 => "/v1/candles/minutes/60",
        Interval::Hour4 => "/v1/candles/minutes/240",
        Interval::Day1 => "/v1/candles/days",
        Interval::Week1 => "/v1/candles/weeks",
        Interval::Month1 => "/v1/candles/months",
        _ => return None,
    })
}

/// Builds one candle-page request ending before `cursor`.
///
/// `count` is the requested page size. An unmapped interval returns
/// [`Error::Unsupported`].
pub(crate) fn candles_request(
    request: &CandleRequest,
    cursor: Option<Timestamp>,
    count: u32,
) -> Result<HttpRequest> {
    let Some(path) = candle_path(request.interval) else {
        return Err(Error::unsupported(
            Feature::Candles,
            EXCHANGE,
            format!(
                "upbit publishes candles at 1s, 1m, 3m, 5m, 10m, 15m, 30m, 1h, 4h, 1d, 1w, 1M \
                 and 1y, and none at {:?}; every one of those it serves and `Interval` can name \
                 is mapped, so this is absent from upbit rather than missing from `maxt`",
                request.interval
            ),
        ));
    };

    let mut params = vec![("market", parse::native_symbol(&request.market)?)];
    if let Some(cursor) = cursor {
        params.push(("to", parse::to_cursor(cursor)?));
    }
    params.push(("count", count.to_string()));

    Ok(HttpRequest::get(path).query(query(&params)))
}

/// Returns a successful response body or maps the exchange error envelope.
pub(crate) async fn send(http: &HttpTransport, request: &HttpRequest) -> Result<String> {
    let response = http.send(request).await?;
    if response.is_success() {
        Ok(response.body)
    } else {
        Err(parse::exchange_error(response.status, &response.body))
    }
}

pub(crate) async fn markets(http: &HttpTransport, kind: MarketKind) -> Result<Vec<MarketInfo>> {
    // Upbit lists spot markets only.
    if kind != MarketKind::Spot {
        return Ok(Vec::new());
    }

    let body = send(http, &markets_request()).await?;
    parse::json::<Vec<parse::RawMarket>>(&body)?
        .iter()
        .map(parse::market_info)
        .collect()
}

pub(crate) async fn market_events(
    http: &HttpTransport,
) -> Result<Vec<(Market, super::UpbitMarketEvent)>> {
    let body = send(http, &markets_request()).await?;
    parse::market_events(&parse::json::<Vec<parse::RawMarket>>(&body)?)
}

pub(crate) async fn trades(
    http: &HttpTransport,
    market: &Market,
    limit: Option<u32>,
) -> Result<Vec<Trade>> {
    let body = send(http, &trades_request(market, limit)?).await?;

    newest_first(&parse::json::<Vec<parse::RawTrade>>(&body)?)
}

/// Sorts trades newest first.
///
/// The stable sort preserves server order for equal timestamps. Upbit's
/// `sequential_id` identifies a trade but is not an ordering key.
fn newest_first(raw: &[parse::RawTrade]) -> Result<Vec<Trade>> {
    let mut trades = raw.iter().map(parse::trade).collect::<Result<Vec<_>>>()?;
    trades.sort_by_key(|trade| Reverse(trade.timestamp));

    Ok(trades)
}

pub(crate) async fn order_books(
    http: &HttpTransport,
    markets: &[Market],
    depth: Option<u32>,
) -> Result<Vec<OrderBook>> {
    let body = send(http, &order_book_request(markets, depth)?).await?;
    parse::json::<Vec<parse::RawOrderBook>>(&body)?
        .iter()
        .map(parse::order_book)
        .collect()
}

pub(crate) async fn order_books_at_level(
    http: &HttpTransport,
    markets: &[Market],
    level: Decimal,
    depth: Option<u32>,
) -> Result<Vec<OrderBook>> {
    let body = send(http, &order_book_with_level_request(markets, level, depth)?).await?;
    parse::json::<Vec<parse::RawOrderBook>>(&body)?
        .iter()
        .map(parse::order_book)
        .collect()
}

pub(crate) async fn tickers(http: &HttpTransport, markets: &[Market]) -> Result<Vec<Ticker>> {
    let body = send(http, &ticker_request(markets)?).await?;
    parse::json::<Vec<parse::RawTicker>>(&body)?
        .iter()
        .map(parse::ticker)
        .collect()
}

pub(crate) async fn tickers_by_quote(
    http: &HttpTransport,
    quote_currencies: &[String],
) -> Result<Vec<Ticker>> {
    let body = send(http, &tickers_by_quote_request(quote_currencies)?).await?;
    parse::json::<Vec<parse::RawTicker>>(&body)?
        .iter()
        .map(parse::ticker)
        .collect()
}

pub(crate) async fn year_candles(
    http: &HttpTransport,
    market: &Market,
    to: Option<Timestamp>,
    count: Option<u32>,
) -> Result<Vec<UpbitYearCandle>> {
    let body = send(http, &year_candles_request(market, to, count)?).await?;
    oldest_first_year_candles(&parse::json::<Vec<parse::RawYearCandle>>(&body)?)
}

/// Upbit responds newest first; keep candle history consistent with `Client`.
fn oldest_first_year_candles(raw: &[parse::RawYearCandle]) -> Result<Vec<UpbitYearCandle>> {
    let mut candles = raw
        .iter()
        .map(parse::year_candle)
        .collect::<Result<Vec<_>>>()?;
    candles.sort_by_key(|candle| candle.open_time);
    Ok(candles)
}

pub(crate) async fn orderbook_instruments(
    http: &HttpTransport,
    markets: &[Market],
) -> Result<Vec<UpbitOrderBookInstrument>> {
    let body = send(http, &orderbook_instruments_request(markets)?).await?;
    parse::json::<Vec<parse::RawOrderBookInstrument>>(&body)?
        .iter()
        .map(parse::orderbook_instrument)
        .collect()
}

/// Reads candles oldest first, paging backward as needed.
///
/// Upbit returns at most [`MAX_CANDLE_COUNT`] newest-first rows per request and
/// provides only an exclusive end cursor.
pub(crate) async fn candles(http: &HttpTransport, request: &CandleRequest) -> Result<Vec<Candle>> {
    let now = Timestamp::now();

    candle_pages::read(
        request,
        EXCHANGE,
        MAX_CANDLE_COUNT,
        |end, count| async move {
            let body = send(http, &candles_request(request, end, count)?).await?;
            let mut page = parse::json::<Vec<parse::RawCandle>>(&body)?
                .iter()
                .map(|raw| parse::candle(raw, request.interval, now))
                .collect::<Result<Vec<_>>>()?;

            // The shared candle reader expects each page oldest first.
            page.reverse();
            Ok(page)
        },
    )
    .await
}

/// Extracts one market from a batch-shaped response.
pub(crate) fn only<T>(mut items: Vec<T>, market: &Market) -> Result<T> {
    if items.len() == 1 {
        Ok(items.remove(0))
    } else {
        Err(Error::decode(format!(
            "upbit answered with {} entries for {market}, expected 1",
            items.len()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Exchange;

    fn btc_krw() -> Market {
        Market::spot(Exchange::Upbit, "BTC", "KRW")
    }

    #[test]
    fn public_requests_target_the_documented_paths() {
        assert_eq!(markets_request().target(), "/v1/market/all?is_details=true");
        assert_eq!(
            trades_request(&btc_krw(), Some(10))
                .expect("a valid limit")
                .target(),
            "/v1/trades/ticks?market=KRW-BTC&count=10"
        );
        assert_eq!(
            order_book_request(&[btc_krw()], Some(5))
                .expect("a valid depth")
                .target(),
            "/v1/orderbook?markets=KRW-BTC&count=5"
        );
        assert_eq!(
            order_book_with_level_request(&[btc_krw()], Decimal::new(1_000_000, 1), Some(2),)
                .expect("a valid aggregation level")
                .target(),
            "/v1/orderbook?markets=KRW-BTC&level=100000&count=2"
        );
        assert_eq!(
            order_book_with_level_request(&[btc_krw()], Decimal::new(10, 2), None)
                .expect("a valid fractional aggregation level")
                .target(),
            "/v1/orderbook?markets=KRW-BTC&level=0.1"
        );
        assert_eq!(
            order_book_with_level_request(&[btc_krw()], Decimal::ZERO, None)
                .expect("the documented unaggregated level")
                .target(),
            "/v1/orderbook?markets=KRW-BTC&level=0"
        );
        assert_eq!(
            ticker_request(&[btc_krw()]).expect("a market").target(),
            "/v1/ticker?markets=KRW-BTC"
        );
        assert_eq!(
            tickers_by_quote_request(&["KRW".to_owned()])
                .expect("a quote currency")
                .target(),
            "/v1/ticker/all?quote_currencies=KRW"
        );
        assert_eq!(
            year_candles_request(
                &btc_krw(),
                Some(Timestamp::from_secs(1_499_040_000)),
                Some(2),
            )
            .expect("a documented candle request")
            .target(),
            "/v1/candles/years?market=KRW-BTC&to=2017-07-03T00%3A00%3A00Z&count=2"
        );
        assert_eq!(
            orderbook_instruments_request(&[btc_krw()])
                .expect("a market")
                .target(),
            "/v1/orderbook/instruments?markets=KRW-BTC"
        );
    }

    #[test]
    fn several_markets_go_into_one_call_as_a_comma_list() {
        let markets = [btc_krw(), Market::spot(Exchange::Upbit, "ETH", "KRW")];

        assert_eq!(
            ticker_request(&markets).expect("two markets").target(),
            "/v1/ticker?markets=KRW-BTC%2CKRW-ETH"
        );
    }

    #[test]
    fn each_interval_reaches_the_endpoint_that_serves_it() {
        let cases = [
            (Interval::Sec1, "/v1/candles/seconds"),
            (Interval::Min1, "/v1/candles/minutes/1"),
            (Interval::Min3, "/v1/candles/minutes/3"),
            (Interval::Min5, "/v1/candles/minutes/5"),
            (Interval::Min10, "/v1/candles/minutes/10"),
            (Interval::Min15, "/v1/candles/minutes/15"),
            (Interval::Min30, "/v1/candles/minutes/30"),
            (Interval::Hour1, "/v1/candles/minutes/60"),
            (Interval::Hour4, "/v1/candles/minutes/240"),
            (Interval::Day1, "/v1/candles/days"),
            (Interval::Week1, "/v1/candles/weeks"),
            (Interval::Month1, "/v1/candles/months"),
        ];

        for (interval, path) in cases {
            let request = CandleRequest::new(btc_krw(), interval);
            assert_eq!(
                candles_request(&request, None, 200)
                    .expect("a served interval")
                    .target(),
                format!("{path}?market=KRW-BTC&count=200"),
                "{interval:?}"
            );
        }
    }

    #[test]
    fn an_interval_upbit_does_not_aggregate_is_refused_as_the_exchanges_gap() {
        // Unavailable intervals must not be substituted with nearby intervals.
        for interval in [
            Interval::Hour2,
            Interval::Hour6,
            Interval::Hour8,
            Interval::Hour12,
            Interval::Day3,
        ] {
            let request = CandleRequest::new(btc_krw(), interval);
            let Err(Error::Unsupported {
                feature: Feature::Candles,
                exchange: "upbit",
                detail,
            }) = candles_request(&request, None, 1)
            else {
                panic!("{interval:?} should not silently become another interval");
            };
            assert!(
                detail.contains("upbit publishes candles at"),
                "{interval:?} was refused without saying what upbit serves: {detail}"
            );
            assert!(
                !detail.contains("no endpoint mapped"),
                "{interval:?} was refused as a gap in `maxt`: {detail}"
            );
        }
    }

    #[test]
    fn the_to_cursor_is_percent_encoded_into_the_query() {
        let request = CandleRequest::new(btc_krw(), Interval::Min5);

        assert_eq!(
            candles_request(&request, Some(Timestamp::from_secs(1_499_040_000)), 2)
                .expect("a served interval")
                .target(),
            "/v1/candles/minutes/5?market=KRW-BTC&to=2017-07-03T00%3A00%3A00Z&count=2"
        );
    }

    #[test]
    fn every_interval_in_the_common_baseline_reaches_an_endpoint() {
        // Every Upbit interval representable by `Interval` is mapped.
        for interval in [
            Interval::Sec1,
            Interval::Min1,
            Interval::Min3,
            Interval::Min5,
            Interval::Min10,
            Interval::Min15,
            Interval::Min30,
            Interval::Hour1,
            Interval::Hour4,
            Interval::Day1,
            Interval::Week1,
            Interval::Month1,
        ] {
            let request = CandleRequest::new(btc_krw(), interval)
                .from(Timestamp::from_secs(1_499_040_000))
                .limit(500);
            assert!(
                candles_request(&request, None, 200).is_ok(),
                "{interval:?} is in the baseline and must not be refused"
            );
        }
    }

    #[test]
    fn every_limit_above_upbits_cap_is_refused_rather_than_clamped() {
        assert!(matches!(
            trades_request(&btc_krw(), Some(501)),
            Err(Error::InvalidRequest { field, .. }) if field == "limit"
        ));
        assert!(matches!(
            trades_request(&btc_krw(), Some(0)),
            Err(Error::InvalidRequest { field, .. }) if field == "limit"
        ));
        assert!(matches!(
            order_book_request(&[btc_krw()], Some(31)),
            Err(Error::InvalidRequest { field, .. }) if field == "depth"
        ));
        assert!(matches!(
            order_book_request(&[btc_krw()], Some(0)),
            Err(Error::InvalidRequest { field, .. }) if field == "depth"
        ));
    }

    #[test]
    fn the_caps_are_the_ones_upbit_documents() {
        assert_eq!(MAX_TRADE_COUNT, 500);
        assert_eq!(MAX_BOOK_DEPTH, 30);
        assert_eq!(MAX_CANDLE_COUNT, 200);
        assert!(trades_request(&btc_krw(), Some(MAX_TRADE_COUNT)).is_ok());
        assert!(order_book_request(&[btc_krw()], Some(MAX_BOOK_DEPTH)).is_ok());
        assert!(
            candles_request(
                &CandleRequest::new(btc_krw(), Interval::Min1),
                None,
                MAX_CANDLE_COUNT
            )
            .is_ok()
        );
    }

    #[test]
    fn a_market_from_another_exchange_never_reaches_the_wire() {
        let elsewhere = Market::spot(Exchange::Binance, "BTC", "USDT");

        assert!(trades_request(&elsewhere, None).is_err());
        assert!(ticker_request(std::slice::from_ref(&elsewhere)).is_err());
        assert!(order_book_request(&[elsewhere], None).is_err());
    }

    #[test]
    fn an_empty_market_list_is_a_caller_mistake_not_an_empty_request() {
        assert!(matches!(
            ticker_request(&[]),
            Err(Error::InvalidRequest { field, .. }) if field == "markets"
        ));
        assert!(matches!(
            order_book_request(&[], None),
            Err(Error::InvalidRequest { field, .. }) if field == "markets"
        ));
        assert!(matches!(
            tickers_by_quote_request(&[]),
            Err(Error::InvalidRequest { field, .. }) if field == "quote_currencies"
        ));
        assert!(matches!(
            orderbook_instruments_request(&[]),
            Err(Error::InvalidRequest { field, .. }) if field == "markets"
        ));
        assert!(matches!(
            order_book_with_level_request(&[btc_krw()], Decimal::new(-1, 0), None),
            Err(Error::InvalidRequest { field, .. }) if field == "level"
        ));
    }

    #[test]
    fn quote_currency_and_year_candle_limits_are_validated_before_transport() {
        assert!(matches!(
            tickers_by_quote_request(&["KRW&markets=BTC".to_owned()]),
            Err(Error::InvalidRequest { field, .. }) if field == "quote_currencies"
        ));
        assert!(matches!(
            year_candles_request(&btc_krw(), None, Some(0)),
            Err(Error::InvalidRequest { field, .. }) if field == "count"
        ));
        assert!(matches!(
            year_candles_request(&btc_krw(), None, Some(MAX_CANDLE_COUNT + 1)),
            Err(Error::InvalidRequest { field, .. }) if field == "count"
        ));
    }

    #[test]
    fn recent_trades_come_back_newest_first() {
        // The fixture is already in Upbit's newest-first response order.
        let raw: Vec<parse::RawTrade> = parse::json(
            r#"[
              {"market":"KRW-BTC","trade_date_utc":"2024-01-01","trade_time_utc":"12:00:03",
               "timestamp":1704110403000,"trade_price":52000000,"trade_volume":0.1,
               "prev_closing_price":51000000,"change_price":1000000,"ask_bid":"BID",
               "sequential_id":1704110403000001},
              {"market":"KRW-BTC","trade_date_utc":"2024-01-01","trade_time_utc":"12:00:02",
               "timestamp":1704110402000,"trade_price":51900000,"trade_volume":0.2,
               "prev_closing_price":51000000,"change_price":900000,"ask_bid":"ASK",
               "sequential_id":1704110402000001},
              {"market":"KRW-BTC","trade_date_utc":"2024-01-01","trade_time_utc":"12:00:01",
               "timestamp":1704110401000,"trade_price":51800000,"trade_volume":0.3,
               "prev_closing_price":51000000,"change_price":800000,"ask_bid":"BID",
               "sequential_id":1704110401000001}
            ]"#,
        )
        .expect("official trades payload");

        let trades = newest_first(&raw).expect("three trades");

        assert_eq!(
            trades
                .iter()
                .map(|trade| trade.timestamp.as_millis())
                .collect::<Vec<_>>(),
            vec![1_704_110_403_000, 1_704_110_402_000, 1_704_110_401_000]
        );
    }

    #[test]
    fn trades_are_reordered_rather_than_trusted_to_arrive_sorted() {
        // Sorting is enforced even if the payload order changes.
        let raw: Vec<parse::RawTrade> = parse::json(
            r#"[
              {"market":"KRW-BTC","timestamp":1704110401000,"trade_price":51800000,
               "trade_volume":0.3,"ask_bid":"BID"},
              {"market":"KRW-BTC","timestamp":1704110403000,"trade_price":52000000,
               "trade_volume":0.1,"ask_bid":"BID"},
              {"market":"KRW-BTC","timestamp":1704110402000,"trade_price":51900000,
               "trade_volume":0.2,"ask_bid":"ASK"}
            ]"#,
        )
        .expect("a trades payload");

        let trades = newest_first(&raw).expect("three trades");

        assert!(
            trades
                .windows(2)
                .all(|pair| pair[0].timestamp > pair[1].timestamp),
            "trades came back {:?}",
            trades
                .iter()
                .map(|trade| trade.timestamp.as_millis())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn yearly_candles_are_reordered_oldest_first() {
        let raw: Vec<parse::RawYearCandle> = parse::json(
            r#"[
              {"market":"KRW-BTC","candle_date_time_utc":"2026-01-01T00:00:00",
               "timestamp":1767225600000,"opening_price":2,"high_price":2,"low_price":2,
               "trade_price":2,"candle_acc_trade_volume":2,"candle_acc_trade_price":2,
               "first_day_of_period":"2026-01-01"},
              {"market":"KRW-BTC","candle_date_time_utc":"2025-01-01T00:00:00",
               "timestamp":1735689600000,"opening_price":1,"high_price":1,"low_price":1,
               "trade_price":1,"candle_acc_trade_volume":1,"candle_acc_trade_price":1,
               "first_day_of_period":"2025-01-01"}
            ]"#,
        )
        .expect("a newest-first yearly payload");

        let candles = oldest_first_year_candles(&raw).expect("yearly candles");

        assert_eq!(
            candles
                .iter()
                .map(|candle| candle.first_day_of_period.as_str())
                .collect::<Vec<_>>(),
            ["2025-01-01", "2026-01-01"]
        );
    }

    #[test]
    fn a_multi_market_answer_to_a_single_market_question_is_a_decode_error() {
        assert_eq!(only(vec![1], &btc_krw()).expect("exactly one"), 1);
        assert!(matches!(
            only(Vec::<u8>::new(), &btc_krw()),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(
            only(vec![1, 2], &btc_krw()),
            Err(Error::Decode { .. })
        ));
    }
}
