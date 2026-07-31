//! Bithumb public REST requests and response handling.

use serde_json::Value;

use crate::adapters::candles as candle_pages;
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::CandleRequest;
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{
    Candle, Interval, Market, MarketInfo, MarketKind, OrderBook, Ticker, Timestamp, Trade,
};

use super::parse::{self, EXCHANGE};

/// Maximum recent trades in one Bithumb response.
const MAX_TRADE_COUNT: u32 = 500;
/// Maximum order-book levels per side in one Bithumb response.
const MAX_BOOK_DEPTH: u32 = 30;
/// Maximum candles in one Bithumb response.
const MAX_CANDLE_COUNT: u32 = 200;

/// Percent-encodes one query value.
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

pub(crate) fn query(params: &[(&str, String)]) -> String {
    params
        .iter()
        .map(|(key, value)| format!("{key}={}", encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

pub(crate) fn markets_request() -> HttpRequest {
    // `isDetails` adds the investment-warning flag used by `MarketStatus`.
    HttpRequest::get("/v1/market/all").query("isDetails=true")
}

/// Builds the separate alert-system request (경보제).
pub(crate) fn market_alerts_request() -> HttpRequest {
    HttpRequest::get("/v1/market/virtual_asset_warning")
}

pub(crate) fn trades_request(market: &Market, limit: Option<u32>) -> Result<HttpRequest> {
    let mut params = vec![("market", parse::native_symbol(market)?)];
    if let Some(limit) = limit {
        if !(1..=MAX_TRADE_COUNT).contains(&limit) {
            return Err(Error::invalid_request(
                "limit",
                format!("bithumb serves 1 to {MAX_TRADE_COUNT} trades per call, not {limit}"),
            ));
        }
        params.push(("count", limit.to_string()));
    }

    Ok(HttpRequest::get("/v1/trades/ticks").query(query(&params)))
}

pub(crate) fn order_book_request(market: &Market, depth: Option<u32>) -> Result<HttpRequest> {
    if let Some(depth) = depth {
        if !(1..=MAX_BOOK_DEPTH).contains(&depth) {
            return Err(Error::invalid_request(
                "depth",
                format!("bithumb serves 1 to {MAX_BOOK_DEPTH} levels per side, not {depth}"),
            ));
        }
    }

    // Bithumb has no depth parameter; the parsed book is truncated locally.
    let params = [("markets", parse::native_symbol(market)?)];

    Ok(HttpRequest::get("/v1/orderbook").query(query(&params)))
}

pub(crate) fn ticker_request(market: &Market) -> Result<HttpRequest> {
    let params = [("markets", parse::native_symbol(market)?)];

    Ok(HttpRequest::get("/v1/ticker").query(query(&params)))
}

/// Returns the endpoint for an interval supported by both Bithumb and `Interval`.
///
/// Bithumb also serves ten-minute candles, which `Interval` cannot represent.
pub(crate) fn candle_path(interval: Interval) -> Option<&'static str> {
    Some(match interval {
        Interval::Min1 => "/v1/candles/minutes/1",
        Interval::Min3 => "/v1/candles/minutes/3",
        Interval::Min5 => "/v1/candles/minutes/5",
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

/// Builds one candle page ending before the optional exclusive cursor.
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
                "bithumb publishes candles at 1m, 3m, 5m, 10m, 15m, 30m, 1h, 4h, 1d, 1w and 1M, \
                 and none at {:?}; every one of those it serves and `Interval` can name is \
                 mapped, so this is absent from bithumb rather than missing from `maxt`",
                request.interval
            ),
        ));
    };

    let mut params = vec![("market", parse::native_symbol(&request.market)?)];
    if let Some(cursor) = cursor {
        params.push(("to", kst_wall_clock(cursor)?));
    }
    params.push(("count", count.to_string()));

    Ok(HttpRequest::get(path).query(query(&params)))
}

/// Formats an exclusive UTC cursor as Bithumb's second-resolution KST wall clock.
///
/// Subsecond values round up so the exclusive bound is preserved.
fn kst_wall_clock(cursor: Timestamp) -> Result<String> {
    const KST_OFFSET_SECS: i64 = 9 * 3_600;

    let nanos = cursor.as_nanos();
    let secs = nanos.div_euclid(1_000_000_000) + i64::from(nanos.rem_euclid(1_000_000_000) != 0);

    secs.checked_add(KST_OFFSET_SECS)
        .and_then(|secs| chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0))
        .map(|kst| kst.format("%Y-%m-%dT%H:%M:%S").to_string())
        .ok_or_else(|| Error::invalid_request("to", format!("{cursor} is not a calendar date")))
}

fn read_response(status: u16, body: &str) -> Result<Value> {
    if !(200..300).contains(&status) {
        return Err(parse::exchange_error(status, body));
    }

    let value = parse::body(body)?;
    if value.get("error").is_some() {
        return Err(parse::exchange_error(status, body));
    }

    Ok(value)
}

/// Sends a request and maps Bithumb error envelopes to [`Error::Exchange`],
/// including envelopes returned with a successful HTTP status.
pub(crate) async fn send(http: &HttpTransport, request: &HttpRequest) -> Result<Value> {
    let response = http.send(request).await?;

    read_response(response.status, &response.body)
}

pub(crate) async fn markets(http: &HttpTransport, kind: MarketKind) -> Result<Vec<MarketInfo>> {
    // Bithumb lists spot markets only.
    if kind != MarketKind::Spot {
        return Ok(Vec::new());
    }

    parse::markets(&send(http, &markets_request()).await?)
}

pub(crate) async fn market_warnings(http: &HttpTransport) -> Result<Vec<(Market, String)>> {
    parse::market_warnings(&send(http, &markets_request()).await?)
}

pub(crate) async fn market_alerts(
    http: &HttpTransport,
) -> Result<Vec<(Market, super::BithumbMarketAlert)>> {
    parse::market_alerts(&send(http, &market_alerts_request()).await?)
}

pub(crate) async fn trades(
    http: &HttpTransport,
    market: &Market,
    limit: Option<u32>,
) -> Result<Vec<Trade>> {
    parse::trades(&send(http, &trades_request(market, limit)?).await?)
}

pub(crate) async fn order_book(
    http: &HttpTransport,
    market: &Market,
    depth: Option<u32>,
) -> Result<OrderBook> {
    let body = send(http, &order_book_request(market, depth)?).await?;
    let entry = only(&body, market)?;
    let timestamp = parse::millis(entry, "timestamp")?;

    parse::order_book(entry, market.clone(), timestamp, depth)
}

pub(crate) async fn ticker(http: &HttpTransport, market: &Market) -> Result<Ticker> {
    let body = send(http, &ticker_request(market)?).await?;

    parse::ticker(only(&body, market)?, market.clone())
}

/// Extracts the only entry expected from a single-market request.
fn only<'a>(body: &'a Value, market: &Market) -> Result<&'a Value> {
    match body.as_array().map(Vec::as_slice) {
        Some([entry]) => Ok(entry),
        Some(entries) => Err(Error::decode(format!(
            "bithumb answered with {} entries for {market}, expected 1",
            entries.len()
        ))),
        None => Err(Error::decode("expected a JSON array")),
    }
}

/// Reads candles oldest-first, paging backward from Bithumb's exclusive cursor.
///
/// Each response is capped at [`MAX_CANDLE_COUNT`]. Paging follows Bithumb's
/// UTC+09:00 candle grid.
pub(crate) async fn candles(http: &HttpTransport, request: &CandleRequest) -> Result<Vec<Candle>> {
    let now = Timestamp::now();
    let interval = request.interval;

    candle_pages::read_on_grid(
        request,
        EXCHANGE,
        MAX_CANDLE_COUNT,
        move |at, count| parse::advance_open(interval, at, count),
        |end, count| async move {
            let body = send(http, &candles_request(request, end, count)?).await?;

            parse::candles(&body, request.interval, now)
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Exchange;

    fn btc_krw() -> Market {
        Market::spot(Exchange::Bithumb, "BTC", "KRW")
    }

    fn candle_request(interval: Interval) -> CandleRequest {
        CandleRequest::new(btc_krw(), interval)
    }

    #[test]
    fn public_requests_target_the_documented_paths() {
        assert_eq!(markets_request().target(), "/v1/market/all?isDetails=true");
        assert_eq!(
            trades_request(&btc_krw(), Some(10))
                .expect("a valid limit")
                .target(),
            "/v1/trades/ticks?market=KRW-BTC&count=10"
        );
        assert_eq!(
            order_book_request(&btc_krw(), Some(5))
                .expect("a valid depth")
                .target(),
            "/v1/orderbook?markets=KRW-BTC"
        );
        assert_eq!(
            ticker_request(&btc_krw()).expect("a market").target(),
            "/v1/ticker?markets=KRW-BTC"
        );
    }

    #[test]
    fn each_interval_reaches_the_endpoint_that_serves_it() {
        // Shape references: https://apidocs.bithumb.com/reference/분minute-캔들-조회.md
        // and https://apidocs.bithumb.com/reference/월month-캔들-조회.md
        let cases = [
            (Interval::Min1, "/v1/candles/minutes/1"),
            (Interval::Min3, "/v1/candles/minutes/3"),
            (Interval::Min5, "/v1/candles/minutes/5"),
            (Interval::Min15, "/v1/candles/minutes/15"),
            (Interval::Min30, "/v1/candles/minutes/30"),
            (Interval::Hour1, "/v1/candles/minutes/60"),
            (Interval::Hour4, "/v1/candles/minutes/240"),
            (Interval::Day1, "/v1/candles/days"),
            (Interval::Week1, "/v1/candles/weeks"),
            (Interval::Month1, "/v1/candles/months"),
        ];

        for (interval, path) in cases {
            assert_eq!(
                candles_request(&candle_request(interval), None, 200)
                    .expect("a served interval")
                    .target(),
                format!("{path}?market=KRW-BTC&count=200"),
                "{interval:?}"
            );
        }
    }

    #[test]
    fn an_interval_bithumb_does_not_aggregate_is_refused_as_the_exchanges_gap() {
        // Unsupported intervals must not be mapped to a neighboring endpoint.
        for interval in [
            Interval::Sec1,
            Interval::Hour2,
            Interval::Hour8,
            Interval::Hour12,
            Interval::Day3,
        ] {
            let Err(Error::Unsupported {
                feature: Feature::Candles,
                exchange: "bithumb",
                detail,
            }) = candles_request(&candle_request(interval), None, 1)
            else {
                panic!("{interval:?} should not silently become another interval");
            };
            assert!(
                detail.contains("bithumb publishes candles at"),
                "{interval:?} was refused without saying what bithumb serves: {detail}"
            );
            assert!(
                !detail.contains("no endpoint mapped"),
                "{interval:?} was refused as a gap in `maxt`: {detail}"
            );
        }
    }

    #[test]
    fn a_cursor_is_sent_as_korean_wall_clock_and_percent_encoded() {
        // 2017-07-03T00:00:00Z is 09:00 KST.
        let cursor = Timestamp::from_secs(1_499_040_000);

        assert_eq!(
            kst_wall_clock(cursor).expect("a representable date"),
            "2017-07-03T09:00:00"
        );
        assert_eq!(
            candles_request(&candle_request(Interval::Min5), Some(cursor), 2)
                .expect("a served interval")
                .target(),
            "/v1/candles/minutes/5?market=KRW-BTC&to=2017-07-03T09%3A00%3A00&count=2"
        );
    }

    #[test]
    fn a_sub_second_cursor_is_rounded_up_before_kst_formatting() {
        let half_past = Timestamp::from_millis(1_785_394_320_500);

        assert_eq!(
            kst_wall_clock(half_past).expect("a representable date"),
            "2026-07-30T15:52:01"
        );
        assert_eq!(
            candles_request(&candle_request(Interval::Min1), Some(half_past), 1)
                .expect("a served interval")
                .target(),
            "/v1/candles/minutes/1?market=KRW-BTC&to=2026-07-30T15%3A52%3A01&count=1"
        );

        let whole_second = Timestamp::from_secs(1_785_394_320);
        assert_eq!(
            kst_wall_clock(whole_second).expect("a representable date"),
            "2026-07-30T15:52:00"
        );
    }

    #[test]
    fn every_limit_outside_bithumbs_range_is_refused_rather_than_clamped() {
        for limit in [0, 501, u32::MAX] {
            assert!(
                matches!(
                    trades_request(&btc_krw(), Some(limit)),
                    Err(Error::InvalidRequest { field, .. }) if field == "limit"
                ),
                "{limit}"
            );
        }
        assert!(trades_request(&btc_krw(), Some(MAX_TRADE_COUNT)).is_ok());
        assert!(matches!(
            order_book_request(&btc_krw(), Some(0)),
            Err(Error::InvalidRequest { field, .. }) if field == "depth"
        ));
        assert!(matches!(
            order_book_request(&btc_krw(), Some(31)),
            Err(Error::InvalidRequest { field, .. }) if field == "depth"
        ));
        assert!(order_book_request(&btc_krw(), Some(30)).is_ok());
    }

    #[test]
    fn the_caps_are_the_ones_bithumb_documents() {
        assert_eq!(MAX_TRADE_COUNT, 500);
        assert_eq!(MAX_BOOK_DEPTH, 30);
        assert_eq!(MAX_CANDLE_COUNT, 200);
    }

    #[test]
    fn a_market_from_another_exchange_never_reaches_the_wire() {
        let elsewhere = Market::spot(Exchange::Upbit, "BTC", "KRW");

        assert!(trades_request(&elsewhere, None).is_err());
        assert!(ticker_request(&elsewhere).is_err());
        assert!(order_book_request(&elsewhere, None).is_err());
    }

    #[test]
    fn an_error_envelope_on_a_success_status_is_an_exchange_error() {
        let error = read_response(200, r#"{"error":{"name":404,"message":"Code not found"}}"#)
            .expect_err("Bithumb returned an error envelope");

        let Error::Exchange {
            code,
            message,
            status,
            ..
        } = error
        else {
            panic!("expected an exchange error");
        };
        assert_eq!(code, "404");
        assert_eq!(message, "Code not found");
        assert_eq!(status, Some(200));
    }

    #[test]
    fn a_multi_market_answer_to_a_single_market_question_is_a_decode_error() {
        let one = parse::body(r#"[{"market":"KRW-BTC"}]"#).expect("JSON");
        let two = parse::body(r#"[{"market":"KRW-BTC"},{"market":"KRW-ETH"}]"#).expect("JSON");
        let neither = parse::body(r#"{"market":"KRW-BTC"}"#).expect("JSON");

        assert!(only(&one, &btc_krw()).is_ok());
        assert!(matches!(only(&two, &btc_krw()), Err(Error::Decode { .. })));
        assert!(matches!(
            only(&neither, &btc_krw()),
            Err(Error::Decode { .. })
        ));
    }
}
