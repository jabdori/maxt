//! Binance payloads shared by Spot and USD-M REST and streams.
//!
//! `serde` aliases cover REST names and abbreviated stream names. Prices and
//! quantities are parsed directly from strings into [`Decimal`] without an
//! `f64` conversion.

use std::cmp::Reverse;

use rust_decimal::Decimal;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::types::{
    Candle, Interval, Level, Market, MarketInfo, Order, OrderBook, OrderStatus, Side, Ticker,
    Timestamp, Trade,
};

use super::{BinanceMarket, market_status};

/// Reads a decimal out of the raw text Binance sent.
///
/// A number that `Decimal` cannot hold exactly is a decode failure. Rounding
/// it would silently lose the last digit of a price. See
/// [`crate::adapters::decimal::exact`], the same reader every other adapter
/// uses.
pub(super) fn decimal(text: &str, field: &'static str) -> Result<Decimal> {
    crate::adapters::decimal::exact(text)
        .map_err(|err| Error::decode(format!("`{field}`: {err}, in `{text}`")))
}

/// Reads a decimal, treating an exact zero as "not published".
///
/// Binance writes `"0.00000000"` for a field that does not apply, such as a
/// market order's limit price. Zero is not a price any listed market trades
/// at, so it is safe to read as absent.
pub(super) fn decimal_or_none(text: &str, field: &'static str) -> Result<Option<Decimal>> {
    let value = decimal(text, field)?;
    Ok((!value.is_zero()).then_some(value))
}

/// Deserializes a response body, naming the endpoint when it does not fit.
pub(super) fn json<T: DeserializeOwned>(body: &str, what: &'static str) -> Result<T> {
    serde_json::from_str(body).map_err(|err| Error::decode(format!("unreadable {what}: {err}")))
}

/// Binance timestamps are milliseconds since the epoch, everywhere.
pub(super) const fn millis(value: i64) -> Timestamp {
    Timestamp::from_millis(value)
}

// ---------------------------------------------------------------------------
// Raw payloads
// ---------------------------------------------------------------------------

/// One `[price, quantity]` pair from a depth payload.
///
/// Read as a list because Binance has historically appended ignored elements
/// to these arrays, and a tuple would reject them.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct RawLevel(Vec<String>);

impl RawLevel {
    fn level(&self) -> Result<Level> {
        let (Some(price), Some(quantity)) = (self.0.first(), self.0.get(1)) else {
            return Err(Error::decode("depth level is not a [price, quantity] pair"));
        };
        Ok(Level {
            price: decimal(price, "price")?,
            quantity: decimal(quantity, "quantity")?,
        })
    }
}

/// A depth payload, from either venue and from either transport.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct RawDepth {
    /// USD-M stamps its books; spot publishes no clock on a depth payload at
    /// all, over REST or over the stream.
    #[serde(rename = "E")]
    pub(super) event_time: Option<i64>,
    #[serde(rename = "bids", alias = "b")]
    pub(super) bids: Vec<RawLevel>,
    #[serde(rename = "asks", alias = "a")]
    pub(super) asks: Vec<RawLevel>,
}

/// One executed trade, from the recent-trades list or Spot's trade stream.
///
/// The identifier arrives as `id` over REST and as `t` on `@trade`.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct RawTrade {
    #[serde(rename = "id", alias = "t")]
    pub(super) id: i64,
    #[serde(rename = "price", alias = "p")]
    pub(super) price: String,
    #[serde(rename = "qty", alias = "q")]
    pub(super) quantity: String,
    #[serde(rename = "time", alias = "T")]
    pub(super) time: i64,
    #[serde(rename = "isBuyerMaker", alias = "m")]
    pub(super) is_buyer_maker: bool,
}

/// A rolling 24-hour summary, from REST or from a ticker stream.
///
/// The `mini` variant of the REST endpoint omits the change fields, so they are
/// optional; the full variant and the stream both fill them in.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct RawTicker {
    #[serde(rename = "priceChange", alias = "p")]
    pub(super) price_change: Option<String>,
    #[serde(rename = "priceChangePercent", alias = "P")]
    pub(super) price_change_percent: Option<String>,
    #[serde(rename = "lastPrice", alias = "c")]
    pub(super) last_price: String,
    #[serde(rename = "highPrice", alias = "h")]
    pub(super) high_price: String,
    #[serde(rename = "lowPrice", alias = "l")]
    pub(super) low_price: String,
    #[serde(rename = "volume", alias = "v")]
    pub(super) volume: String,
    #[serde(rename = "quoteVolume", alias = "q")]
    pub(super) quote_volume: String,
    /// The end of the 24-hour window, which is when Binance computed the
    /// figures. It is not the time of the last fill; Binance publishes no such
    /// field on either transport.
    #[serde(rename = "closeTime", alias = "C")]
    pub(super) close_time: i64,
}

/// One candle, as the fixed-order array Binance sends instead of an object.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct RawCandle(
    i64,
    String,
    String,
    String,
    String,
    String,
    i64,
    String,
    // Trade count and the taker-volume fields follow; none of them reach a
    // `Candle`, and `IgnoredAny` keeps the arity check without naming them.
    serde::de::IgnoredAny,
    serde::de::IgnoredAny,
    serde::de::IgnoredAny,
    serde::de::IgnoredAny,
);

/// An order, from any order endpoint on either venue.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawOrder {
    pub(super) symbol: String,
    pub(super) order_id: i64,
    pub(super) side: String,
    pub(super) status: String,
    pub(super) price: String,
    pub(super) orig_qty: String,
    pub(super) executed_qty: String,
    /// Present on order lookups and open-order lists.
    pub(super) time: Option<i64>,
    /// Present on place and cancel acknowledgements instead of `time`.
    pub(super) transact_time: Option<i64>,
    /// Present on USD-M acknowledgements instead of either of the above.
    pub(super) update_time: Option<i64>,
}

/// The `exchangeInfo` listing, on either venue.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct RawExchangeInfo {
    pub(super) symbols: Vec<RawSymbol>,
}

/// One listed symbol.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawSymbol {
    pub(super) symbol: String,
    pub(super) status: String,
    pub(super) base_asset: String,
    pub(super) quote_asset: String,
    /// USD-M only. `PERPETUAL` for the contracts this adapter covers; the same
    /// listing also carries dated futures, which `maxt` has no type for.
    pub(super) contract_type: Option<String>,
    #[serde(default)]
    pub(super) filters: Vec<RawFilter>,
}

/// One trading rule attached to a symbol.
///
/// Every filter Binance publishes shares `filterType` and nothing else, so the
/// fields are all optional and only the ones the named filter carries are set.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawFilter {
    pub(super) filter_type: String,
    pub(super) tick_size: Option<String>,
    pub(super) min_price: Option<String>,
    pub(super) max_price: Option<String>,
    pub(super) step_size: Option<String>,
    pub(super) min_qty: Option<String>,
    pub(super) max_qty: Option<String>,
    pub(super) min_notional: Option<String>,
}

// ---------------------------------------------------------------------------
// Raw to domain
// ---------------------------------------------------------------------------

/// Maps a listing, or `None` when the entry is not a market this venue trades.
///
/// The USD-M listing mixes perpetual and dated contracts. A dated one is
/// dropped, because reporting it as a perpetual would misprice it.
pub(super) fn market_info(venue: BinanceMarket, raw: &RawSymbol) -> Option<MarketInfo> {
    if venue == BinanceMarket::UsdMFutures && raw.contract_type.as_deref() != Some("PERPETUAL") {
        return None;
    }

    Some(MarketInfo {
        // Taken from the listing's own `baseAsset`/`quoteAsset` rather than by
        // splitting the symbol: this is the answer the split approximates.
        market: Market::new(
            crate::types::Exchange::Binance,
            venue.market_kind(),
            &raw.base_asset,
            &raw.quote_asset,
        ),
        native_symbol: raw.symbol.clone(),
        status: market_status(&raw.status),
        // Binance publishes no asset names in either language.
        korean_name: None,
        english_name: None,
    })
}

/// Builds a book with both sides ordered best-first.
///
/// Binance sorts its own depth payloads, but the [`OrderBook`] ordering is a
/// promise to the caller, so it is enforced here.
pub(super) fn order_book(
    market: &Market,
    fallback_time: Timestamp,
    raw: &RawDepth,
) -> Result<OrderBook> {
    let mut bids = raw
        .bids
        .iter()
        .map(RawLevel::level)
        .collect::<Result<Vec<_>>>()?;
    let mut asks = raw
        .asks
        .iter()
        .map(RawLevel::level)
        .collect::<Result<Vec<_>>>()?;

    bids.sort_by_key(|level| Reverse(level.price));
    asks.sort_by_key(|level| level.price);

    Ok(OrderBook {
        market: market.clone(),
        // Spot depth carries no clock at all, so the caller's read time stands
        // in for it. USD-M stamps the book and that stamp always wins.
        timestamp: raw.event_time.map_or(fallback_time, millis),
        bids,
        asks,
    })
}

/// Converts a trade, resolving the taker side.
///
/// `isBuyerMaker` describes the *maker*: when the buyer was resting, the taker
/// sold into the bid. `maxt` reports the taker, so the flag inverts.
pub(super) fn trade(market: &Market, raw: &RawTrade) -> Result<Trade> {
    Ok(Trade {
        market: market.clone(),
        timestamp: millis(raw.time),
        price: decimal(&raw.price, "price")?,
        quantity: decimal(&raw.quantity, "qty")?,
        taker_side: if raw.is_buyer_maker {
            Side::Sell
        } else {
            Side::Buy
        },
        id: Some(raw.id.to_string()),
    })
}

/// Converts a 24-hour summary.
pub(super) fn ticker(market: &Market, raw: &RawTicker) -> Result<Ticker> {
    let change = raw
        .price_change
        .as_deref()
        .map(|text| decimal(text, "priceChange"))
        .transpose()?;
    // Binance publishes the change as a percentage; `Ticker::change_rate` is a
    // ratio, so `-95.960` becomes `-0.95960`. Scaling by a hundredth rather
    // than dividing by a hundred keeps the conversion exact.
    let change_rate = raw
        .price_change_percent
        .as_deref()
        .map(|text| decimal(text, "priceChangePercent"))
        .transpose()?
        .map(|percent| percent * Decimal::new(1, 2));

    Ok(Ticker {
        market: market.clone(),
        timestamp: millis(raw.close_time),
        // The 24-hour summary has no last-trade timestamp.
        last_trade_time: None,
        last_price: decimal(&raw.last_price, "lastPrice")?,
        change,
        change_rate,
        high: Some(decimal(&raw.high_price, "highPrice")?),
        low: Some(decimal(&raw.low_price, "lowPrice")?),
        volume: Some(decimal(&raw.volume, "volume")?),
        quote_volume: Some(decimal(&raw.quote_volume, "quoteVolume")?),
    })
}

/// Converts a candle and marks it closed after its inclusive close millisecond.
pub(super) fn candle(
    market: &Market,
    interval: Interval,
    raw: &RawCandle,
    now_millis: i64,
) -> Result<Candle> {
    Ok(Candle {
        market: market.clone(),
        interval,
        open_time: millis(raw.0),
        open: decimal(&raw.1, "open")?,
        high: decimal(&raw.2, "high")?,
        low: decimal(&raw.3, "low")?,
        close: decimal(&raw.4, "close")?,
        volume: decimal(&raw.5, "volume")?,
        quote_volume: Some(decimal(&raw.7, "quoteVolume")?),
        closed: raw.6 < now_millis,
    })
}

/// Converts an order.
pub(super) fn order(market: &Market, raw: &RawOrder) -> Result<Order> {
    let filled = decimal(&raw.executed_qty, "executedQty")?;
    let total = decimal(&raw.orig_qty, "origQty")?;

    Ok(Order {
        id: raw.order_id.to_string(),
        market: market.clone(),
        side: side(&raw.side)?,
        status: status(&raw.status),
        filled_quantity: filled,
        // Binance reports the original quantity and what has filled, never the
        // remainder; a cancelled order leaves the difference unfillable, so it
        // is clamped rather than reported as still working.
        remaining_quantity: if status(&raw.status).is_live() {
            (total - filled).max(Decimal::ZERO)
        } else {
            Decimal::ZERO
        },
        price: decimal_or_none(&raw.price, "price")?,
        created_at: raw
            .time
            .or(raw.transact_time)
            .or(raw.update_time)
            .map(millis),
    })
}

/// Reads an order side.
pub(super) fn side(raw: &str) -> Result<Side> {
    match raw {
        "BUY" => Ok(Side::Buy),
        "SELL" => Ok(Side::Sell),
        other => Err(Error::decode(format!(
            "unknown Binance order side `{other}`"
        ))),
    }
}

/// Reads an order status.
///
/// `EXPIRED` is a cancellation from the caller's point of view: the order left
/// the book without the caller asking and nothing more will fill.
/// `EXPIRED_IN_MATCH` is the self-trade-prevention spelling of the same thing.
pub(super) fn status(raw: &str) -> OrderStatus {
    match raw {
        "NEW" => OrderStatus::Open,
        "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
        "FILLED" => OrderStatus::Filled,
        "CANCELED" | "EXPIRED" | "EXPIRED_IN_MATCH" => OrderStatus::Cancelled,
        "REJECTED" => OrderStatus::Rejected,
        "PENDING_NEW" | "PENDING_CANCEL" | "NEW_INSURANCE" | "NEW_ADL" => OrderStatus::Accepted,
        _ => OrderStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, MarketKind, MarketStatus};

    fn market() -> Market {
        Market::spot(Exchange::Binance, "BNB", "BTC")
    }

    // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints
    const SPOT_DEPTH: &str = r#"{
      "lastUpdateId": 1027024,
      "bids": [["4.00000000", "431.00000000"]],
      "asks": [["4.00000200", "12.00000000"]]
    }"#;

    // https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest/Order-Book
    const USD_M_DEPTH: &str = r#"{
      "lastUpdateId": 160,
      "E": 123456789,
      "T": 123456788,
      "bids": [["0.0024", "10"]],
      "asks": [["0.0026", "100"]]
    }"#;

    // https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams/Diff-Book-Depth-Streams
    const USD_M_STREAM_DEPTH: &str = r#"{
      "e": "depthUpdate",
      "E": 123456789,
      "T": 123456788,
      "s": "BNBUSDT",
      "U": 157,
      "u": 160,
      "pu": 149,
      "b": [["0.0024", "10"]],
      "a": [["0.0026", "100"]]
    }"#;

    // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints
    const SPOT_TRADES: &str = r#"[
      {
        "id": 28457,
        "price": "4.00000100",
        "qty": "12.00000000",
        "quoteQty": "48.000012",
        "time": 1499865549590,
        "isBuyerMaker": true,
        "isBestMatch": true
      }
    ]"#;

    // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints
    const SPOT_TICKER: &str = r#"{
      "symbol": "BNBBTC",
      "priceChange": "-94.99999800",
      "priceChangePercent": "-95.960",
      "weightedAvgPrice": "0.29628482",
      "prevClosePrice": "0.10002000",
      "lastPrice": "4.00000200",
      "lastQty": "200.00000000",
      "bidPrice": "4.00000000",
      "bidQty": "100.00000000",
      "askPrice": "4.00000200",
      "askQty": "100.00000000",
      "openPrice": "99.00000000",
      "highPrice": "100.00000000",
      "lowPrice": "0.10000000",
      "volume": "8913.30000000",
      "quoteVolume": "15.30000000",
      "openTime": 1499783499040,
      "closeTime": 1499869899040,
      "firstId": 28385,
      "lastId": 28460,
      "count": 76
    }"#;

    // https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams
    const SPOT_STREAM_TICKER: &str = r#"{
      "e": "24hrTicker",
      "E": 1672515782136,
      "s": "BNBBTC",
      "p": "0.0015",
      "P": "250.00",
      "w": "0.0018",
      "x": "0.0009",
      "c": "0.0025",
      "Q": "10",
      "b": "0.0024",
      "B": "10",
      "a": "0.0026",
      "A": "100",
      "o": "0.0010",
      "h": "0.0025",
      "l": "0.0010",
      "v": "10000",
      "q": "18",
      "O": 0,
      "C": 86400000,
      "F": 0,
      "L": 18150,
      "n": 18151
    }"#;

    // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints
    const SPOT_CANDLES: &str = r#"[
      [
        1499040000000,
        "0.01634790",
        "0.80000000",
        "0.01575800",
        "0.01577100",
        "148976.11427815",
        1499644799999,
        "2434.19055334",
        308,
        "1756.87402397",
        "28.46694368",
        "0"
      ]
    ]"#;

    // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints
    const SPOT_ORDER: &str = r#"{
      "symbol": "BTCUSDT",
      "orderId": 28,
      "orderListId": -1,
      "clientOrderId": "6gCrw2kRUAF9CvJDGP16IP",
      "transactTime": 1507725176595,
      "price": "0.00000000",
      "origQty": "10.00000000",
      "executedQty": "10.00000000",
      "cummulativeQuoteQty": "10.00000000",
      "status": "FILLED",
      "timeInForce": "GTC",
      "type": "MARKET",
      "side": "SELL",
      "workingTime": 1507725176595,
      "selfTradePreventionMode": "NONE"
    }"#;

    // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-endpoints
    const SPOT_EXCHANGE_INFO: &str = r#"{
      "timezone": "UTC",
      "serverTime": 1565246363776,
      "rateLimits": [],
      "exchangeFilters": [],
      "symbols": [
        {
          "symbol": "ETHBTC",
          "status": "TRADING",
          "baseAsset": "ETH",
          "baseAssetPrecision": 8,
          "quoteAsset": "BTC",
          "quotePrecision": 8,
          "quoteAssetPrecision": 8,
          "orderTypes": ["LIMIT", "MARKET"],
          "icebergAllowed": true,
          "isSpotTradingAllowed": true,
          "isMarginTradingAllowed": true,
          "filters": [
            {
              "filterType": "PRICE_FILTER",
              "minPrice": "0.00000100",
              "maxPrice": "100000.00000000",
              "tickSize": "0.00000100"
            },
            {
              "filterType": "LOT_SIZE",
              "minQty": "0.00100000",
              "maxQty": "100000.00000000",
              "stepSize": "0.00100000"
            },
            {
              "filterType": "NOTIONAL",
              "minNotional": "0.00010000",
              "applyMinToMarket": true,
              "maxNotional": "9000000.00000000"
            }
          ],
          "permissionSets": [["SPOT", "MARGIN"]]
        }
      ]
    }"#;

    // https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Exchange-Information
    const USD_M_EXCHANGE_INFO: &str = r#"{
      "serverTime": 1565613908500,
      "symbols": [
        {
          "symbol": "BTCUSDT",
          "pair": "BTCUSDT",
          "contractType": "PERPETUAL",
          "status": "TRADING",
          "baseAsset": "BTC",
          "quoteAsset": "USDT",
          "marginAsset": "USDT",
          "pricePrecision": 2,
          "quantityPrecision": 3,
          "filters": []
        },
        {
          "symbol": "ETHUSDT_260327",
          "pair": "ETHUSDT",
          "contractType": "CURRENT_QUARTER",
          "status": "TRADING",
          "baseAsset": "ETH",
          "quoteAsset": "USDT",
          "marginAsset": "USDT",
          "pricePrecision": 2,
          "quantityPrecision": 3,
          "filters": []
        }
      ],
      "timezone": "UTC"
    }"#;

    #[test]
    fn a_decimal_keeps_every_digit_binance_sent() {
        let parsed = decimal("0.00000001", "price").expect("a decimal");

        assert_eq!(parsed.to_string(), "0.00000001");
        assert_eq!(parsed.scale(), 8);
        // Trailing zeros are significant to Binance and survive the round trip.
        assert_eq!(
            decimal("4.00000000", "price")
                .expect("a decimal")
                .to_string(),
            "4.00000000"
        );
    }

    #[test]
    fn a_decimal_that_is_not_a_number_is_a_decode_error_not_a_zero() {
        assert!(matches!(decimal("", "price"), Err(Error::Decode { .. })));
        assert!(matches!(decimal("nan", "price"), Err(Error::Decode { .. })));
        // Twenty-nine decimal places is one past what `Decimal` carries.
        assert!(matches!(
            decimal("0.000000000000000000000000000001", "price"),
            Err(Error::Decode { .. })
        ));
    }

    #[test]
    fn a_zero_price_reads_as_no_price() {
        assert_eq!(
            decimal_or_none("0.00000000", "price").expect("a decimal"),
            None
        );
        assert_eq!(
            decimal_or_none("0.01000000", "price").expect("a decimal"),
            Some(Decimal::new(1, 2))
        );
    }

    #[test]
    fn a_depth_snapshot_lands_best_first_on_both_sides() {
        let raw: RawDepth = json(SPOT_DEPTH, "depth").expect("official depth payload");
        let book = order_book(&market(), millis(1_700_000_000_000), &raw).expect("a book");

        assert_eq!(
            book.best_bid().expect("a bid").price.to_string(),
            "4.00000000"
        );
        assert_eq!(
            book.best_ask().expect("an ask").price.to_string(),
            "4.00000200"
        );
        assert_eq!(
            book.best_bid().expect("a bid").quantity.to_string(),
            "431.00000000"
        );
    }

    #[test]
    fn a_book_arriving_out_of_order_is_re_sorted() {
        let raw: RawDepth = json(
            r#"{"bids":[["1","1"],["3","1"],["2","1"]],"asks":[["9","1"],["7","1"],["8","1"]]}"#,
            "depth",
        )
        .expect("a depth payload");
        let book = order_book(&market(), millis(0), &raw).expect("a book");

        let bids: Vec<String> = book.bids.iter().map(|l| l.price.to_string()).collect();
        let asks: Vec<String> = book.asks.iter().map(|l| l.price.to_string()).collect();
        assert_eq!(bids, ["3", "2", "1"]);
        assert_eq!(asks, ["7", "8", "9"]);
    }

    #[test]
    fn only_spot_books_fall_back_to_the_read_time() {
        let read_at = millis(1_700_000_000_000);
        let spot: RawDepth = json(SPOT_DEPTH, "depth").expect("official spot depth");
        let futures: RawDepth = json(USD_M_DEPTH, "depth").expect("official futures depth");
        let streamed: RawDepth =
            json(USD_M_STREAM_DEPTH, "depth").expect("official futures depth frame");

        // Spot publishes no clock on a book, over either transport.
        assert_eq!(
            order_book(&market(), read_at, &spot)
                .expect("a book")
                .timestamp,
            read_at
        );
        // USD-M does, and its own stamp wins over the read time.
        assert_eq!(
            order_book(&market(), read_at, &futures)
                .expect("a book")
                .timestamp,
            millis(123_456_789)
        );
        // The stream abbreviates `bids`/`asks` to `b`/`a` and nothing else.
        let streamed = order_book(&market(), read_at, &streamed).expect("a book");
        assert_eq!(streamed.timestamp, millis(123_456_789));
        assert_eq!(
            streamed.best_bid().expect("a bid").price.to_string(),
            "0.0024"
        );
        assert_eq!(
            streamed.best_ask().expect("an ask").price.to_string(),
            "0.0026"
        );
    }

    #[test]
    fn a_maker_buyer_means_the_taker_sold() {
        let raw: Vec<RawTrade> = json(SPOT_TRADES, "trades").expect("official trades payload");
        let trade = trade(&market(), &raw[0]).expect("a trade");

        assert_eq!(trade.taker_side, Side::Sell);
        assert_eq!(trade.id.as_deref(), Some("28457"));
        assert_eq!(trade.price.to_string(), "4.00000100");
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_499_865_549_590));
    }

    #[test]
    fn a_ticker_change_percentage_becomes_a_ratio() {
        let raw: RawTicker = json(SPOT_TICKER, "ticker").expect("official ticker payload");
        let ticker = ticker(&market(), &raw).expect("a ticker");

        assert_eq!(ticker.change_rate.expect("a rate").to_string(), "-0.95960");
        assert_eq!(ticker.change.expect("a change").to_string(), "-94.99999800");
        assert_eq!(ticker.last_price.to_string(), "4.00000200");
        assert_eq!(ticker.timestamp, Timestamp::from_millis(1_499_869_899_040));
        // Binance publishes no time for the trade behind `lastPrice`.
        assert_eq!(ticker.last_trade_time, None);
    }

    #[test]
    fn the_streamed_ticker_says_the_same_thing_in_shorter_words() {
        let raw: RawTicker = json(SPOT_STREAM_TICKER, "24hrTicker").expect("official ticker frame");
        let ticker = ticker(&market(), &raw).expect("a ticker");

        assert_eq!(ticker.last_price.to_string(), "0.0025");
        assert_eq!(ticker.change.expect("a change").to_string(), "0.0015");
        assert_eq!(ticker.change_rate.expect("a rate").to_string(), "2.5000");
        assert_eq!(ticker.volume.expect("a volume").to_string(), "10000");
        assert_eq!(ticker.quote_volume.expect("a volume").to_string(), "18");
        // `C`, the end of the statistics window, not `E`, the publish time.
        assert_eq!(ticker.timestamp, Timestamp::from_millis(86_400_000));
    }

    #[test]
    fn a_candle_is_closed_only_once_its_close_time_has_passed() {
        let raw: Vec<RawCandle> = json(SPOT_CANDLES, "klines").expect("official kline payload");

        let closed =
            candle(&market(), Interval::Min1, &raw[0], 1_499_644_800_000).expect("a candle");
        let still_open =
            candle(&market(), Interval::Min1, &raw[0], 1_499_644_799_000).expect("a candle");

        assert!(closed.closed);
        assert!(!still_open.closed);
        assert_eq!(closed.open_time, Timestamp::from_millis(1_499_040_000_000));
        assert_eq!(closed.high.to_string(), "0.80000000");
        assert_eq!(
            closed.quote_volume.expect("a quote volume").to_string(),
            "2434.19055334"
        );
    }

    #[test]
    fn the_last_millisecond_a_window_covers_is_still_inside_it() {
        // Binance close time is inclusive; the candle closes one millisecond later.
        let raw: Vec<RawCandle> = json(SPOT_CANDLES, "klines").expect("official kline payload");

        let at_the_boundary =
            candle(&market(), Interval::Min1, &raw[0], 1_499_644_799_999).expect("a candle");
        let one_millisecond_later =
            candle(&market(), Interval::Min1, &raw[0], 1_499_644_800_000).expect("a candle");

        assert!(!at_the_boundary.closed);
        assert!(one_millisecond_later.closed);
    }

    #[test]
    fn a_truncated_candle_array_is_rejected_rather_than_guessed() {
        assert!(matches!(
            json::<Vec<RawCandle>>(r#"[[1499040000000,"0.01634790"]]"#, "klines"),
            Err(Error::Decode { .. })
        ));
    }

    #[test]
    fn a_market_order_acknowledgement_has_no_price_and_no_remainder() {
        let raw: RawOrder = json(SPOT_ORDER, "order").expect("official order payload");
        let order = order(
            &Market::new(Exchange::Binance, MarketKind::Spot, "BTC", "USDT"),
            &raw,
        )
        .expect("an order");

        assert_eq!(order.id, "28");
        assert_eq!(order.price, None);
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.side, Side::Sell);
        assert_eq!(order.remaining_quantity, Decimal::ZERO);
        assert_eq!(
            order.created_at,
            Some(Timestamp::from_millis(1_507_725_176_595))
        );
    }

    #[test]
    fn a_partly_filled_cancellation_leaves_nothing_working() {
        let mut raw: RawOrder = json(SPOT_ORDER, "order").expect("official order payload");
        raw.status = "CANCELED".to_string();
        raw.executed_qty = "4.00000000".to_string();

        let cancelled = order(&market(), &raw).expect("an order");

        assert_eq!(cancelled.status, OrderStatus::Cancelled);
        assert_eq!(cancelled.filled_quantity.to_string(), "4.00000000");
        assert_eq!(cancelled.remaining_quantity, Decimal::ZERO);
    }

    #[test]
    fn order_statuses_that_end_an_order_never_read_as_live() {
        assert!(status("NEW").is_live());
        assert!(status("PARTIALLY_FILLED").is_live());
        for terminal in [
            "FILLED",
            "CANCELED",
            "EXPIRED",
            "EXPIRED_IN_MATCH",
            "REJECTED",
        ] {
            assert!(!status(terminal).is_live(), "{terminal}");
        }
        assert_eq!(status("SOMETHING_NEW"), OrderStatus::Unknown);
        assert!(matches!(side("HOLD"), Err(Error::Decode { .. })));
    }

    #[test]
    fn a_spot_listing_takes_its_assets_from_the_listing_not_from_the_symbol() {
        let raw: RawExchangeInfo =
            json(SPOT_EXCHANGE_INFO, "exchangeInfo").expect("official listing payload");
        let listing = market_info(BinanceMarket::Spot, &raw.symbols[0]).expect("a spot listing");

        assert_eq!(
            listing.market,
            Market::spot(Exchange::Binance, "ETH", "BTC")
        );
        assert_eq!(listing.native_symbol, "ETHBTC");
        assert_eq!(listing.status, MarketStatus::Active);
        assert_eq!(raw.symbols[0].filters.len(), 3);
    }

    #[test]
    fn the_futures_listing_drops_the_dated_contracts_it_carries_alongside() {
        let raw: RawExchangeInfo =
            json(USD_M_EXCHANGE_INFO, "exchangeInfo").expect("official listing payload");

        let listed: Vec<_> = raw
            .symbols
            .iter()
            .filter_map(|symbol| market_info(BinanceMarket::UsdMFutures, symbol))
            .collect();

        assert_eq!(listed.len(), 1);
        assert_eq!(
            listed[0].market,
            Market::perpetual(Exchange::Binance, "BTC", "USDT")
        );
        // The quarterly contract is dropped rather than mislabelled a perpetual.
        assert!(market_info(BinanceMarket::UsdMFutures, &raw.symbols[1]).is_none());
        // A spot adapter has no contract types to filter on.
        assert!(market_info(BinanceMarket::Spot, &raw.symbols[1]).is_some());
    }
}
