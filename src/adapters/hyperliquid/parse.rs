//! Hyperliquid payload deserialization and domain mapping.
//!
//! Numeric strings and WebSocket candle numbers are converted directly to
//! [`Decimal`] without an `f64` round trip.

use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{Number, Value};

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::types::{
    Balance, Candle, Cursor, Exchange, Interval, Level, MarginMode, MarginSummary, Market,
    MarketInfo, MarketKind, MarketStatus, Order, OrderBook, OrderStatus, Position, Side, Ticker,
    Timestamp, Trade,
};

pub(crate) const EXCHANGE: &str = Exchange::Hyperliquid.id();

/// Settlement asset for default perpetuals and account-wide USDC values.
pub(crate) const SETTLE_ASSET: &str = "USDC";

/// Offset applied to spot pair indices in `/exchange` actions.
pub(crate) const SPOT_ASSET_ID_OFFSET: u32 = 10_000;

// ---------------------------------------------------------------------------
// Raw payloads
// ---------------------------------------------------------------------------

/// The `meta` response: the perpetual universe.
#[derive(Debug, Deserialize)]
pub(crate) struct RawPerpMeta {
    pub(crate) universe: Vec<RawPerpAsset>,
}

/// One perpetual. Its asset id is its position in `universe`, not a field.
#[derive(Debug, Deserialize)]
pub(crate) struct RawPerpAsset {
    pub(crate) name: String,
    #[serde(rename = "szDecimals")]
    pub(crate) sz_decimals: u32,
    #[serde(rename = "maxLeverage", default)]
    pub(crate) max_leverage: Option<u32>,
    #[serde(rename = "onlyIsolated", default)]
    pub(crate) only_isolated: bool,
    #[serde(rename = "isDelisted", default)]
    pub(crate) is_delisted: bool,
}

/// The `spotMeta` response: tokens, and the pairs built from them.
#[derive(Debug, Deserialize)]
pub(crate) struct RawSpotMeta {
    pub(crate) tokens: Vec<RawSpotToken>,
    pub(crate) universe: Vec<RawSpotPair>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawSpotToken {
    pub(crate) name: String,
    #[serde(rename = "szDecimals")]
    pub(crate) sz_decimals: u32,
    pub(crate) index: u32,
}

/// One spot pair. `tokens` indexes into [`RawSpotMeta::tokens`], base first.
#[derive(Debug, Deserialize)]
pub(crate) struct RawSpotPair {
    pub(crate) name: String,
    pub(crate) tokens: [u32; 2],
    pub(crate) index: u32,
}

/// The `l2Book` response, and the `l2Book` stream frame's `data`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawBook {
    pub(crate) coin: String,
    pub(crate) time: i64,
    /// Bids at index 0, asks at index 1.
    pub(crate) levels: [Vec<RawLevel>; 2],
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawLevel {
    pub(crate) px: String,
    pub(crate) sz: String,
}

/// One `candleSnapshot` item or `candle` stream item.
/// `t` is the opening millisecond and `T` is the inclusive closing millisecond.
#[derive(Debug, Deserialize)]
pub(crate) struct RawCandle {
    #[serde(rename = "t")]
    pub(crate) open_time: i64,
    #[serde(rename = "T")]
    pub(crate) close_time: i64,
    #[serde(rename = "s")]
    pub(crate) coin: String,
    #[serde(rename = "i")]
    pub(crate) interval: String,
    #[serde(rename = "o", deserialize_with = "number_or_string")]
    pub(crate) open: String,
    #[serde(rename = "h", deserialize_with = "number_or_string")]
    pub(crate) high: String,
    #[serde(rename = "l", deserialize_with = "number_or_string")]
    pub(crate) low: String,
    #[serde(rename = "c", deserialize_with = "number_or_string")]
    pub(crate) close: String,
    #[serde(rename = "v", deserialize_with = "number_or_string")]
    pub(crate) volume: String,
}

/// One entry of the `trades` stream frame.
#[derive(Debug, Deserialize)]
pub(crate) struct RawTrade {
    pub(crate) coin: String,
    /// `B` or `A`, naming the side the taker was on.
    pub(crate) side: String,
    pub(crate) px: String,
    pub(crate) sz: String,
    pub(crate) time: i64,
    pub(crate) tid: Number,
}

/// One context from `metaAndAssetCtxs` or `spotMetaAndAssetCtxs`.
///
/// Spot contexts are matched by `coin`. Default perpetual contexts omit `coin`
/// and are matched to the metadata array by position.
#[derive(Debug, Deserialize)]
pub(crate) struct RawAssetCtx {
    /// Native market name supplied by spot contexts.
    #[serde(default)]
    pub(crate) coin: Option<String>,
    #[serde(rename = "midPx", default)]
    pub(crate) mid_px: Option<String>,
    #[serde(rename = "markPx", default)]
    pub(crate) mark_px: Option<String>,
    #[serde(rename = "prevDayPx", default)]
    pub(crate) prev_day_px: Option<String>,
    #[serde(rename = "dayBaseVlm", default)]
    pub(crate) day_base_volume: Option<String>,
    #[serde(rename = "dayNtlVlm", default)]
    pub(crate) day_notional_volume: Option<String>,
    /// The external price funding is computed against. Perpetuals only.
    #[serde(rename = "oraclePx", default)]
    pub(crate) oracle_px: Option<String>,
    /// The current funding rate. Perpetuals only.
    #[serde(default)]
    pub(crate) funding: Option<String>,
    #[serde(rename = "openInterest", default)]
    pub(crate) open_interest: Option<String>,
}

/// The `spotClearinghouseState` response.
#[derive(Debug, Deserialize)]
pub(crate) struct RawSpotState {
    pub(crate) balances: Vec<RawSpotBalance>,
}

/// One spot balance. `total` includes `hold`, so the free amount is the
/// difference between them.
#[derive(Debug, Deserialize)]
pub(crate) struct RawSpotBalance {
    pub(crate) coin: String,
    pub(crate) hold: String,
    pub(crate) total: String,
}

/// The `clearinghouseState` response: perpetual positions and account margin.
#[derive(Debug, Deserialize)]
pub(crate) struct RawPerpState {
    #[serde(rename = "assetPositions")]
    pub(crate) asset_positions: Vec<RawAssetPosition>,
    #[serde(rename = "marginSummary")]
    pub(crate) margin_summary: RawMarginSummary,
    pub(crate) withdrawable: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawAssetPosition {
    pub(crate) position: RawPosition,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawPosition {
    pub(crate) coin: String,
    /// Signed position size: negative is short. `Position::quantity` is
    /// unsigned, so the sign becomes `Position::side`.
    pub(crate) szi: String,
    #[serde(rename = "entryPx", default)]
    pub(crate) entry_px: Option<String>,
    #[serde(rename = "positionValue")]
    pub(crate) position_value: String,
    #[serde(rename = "unrealizedPnl")]
    pub(crate) unrealized_pnl: String,
    pub(crate) leverage: RawLeverage,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawLeverage {
    #[serde(rename = "type")]
    pub(crate) margin_type: String,
    pub(crate) value: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawMarginSummary {
    #[serde(rename = "accountValue")]
    pub(crate) account_value: String,
    #[serde(rename = "totalMarginUsed")]
    pub(crate) total_margin_used: String,
}

/// One entry of `frontendOpenOrders`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawOpenOrder {
    pub(crate) coin: String,
    pub(crate) oid: u64,
    pub(crate) side: String,
    #[serde(rename = "limitPx")]
    pub(crate) limit_px: String,
    pub(crate) sz: String,
    #[serde(rename = "origSz")]
    pub(crate) orig_sz: String,
    pub(crate) timestamp: i64,
}

/// One entry of `userFunding`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawUserFunding {
    pub(crate) delta: RawFundingDelta,
    pub(crate) time: i64,
    pub(crate) hash: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawFundingDelta {
    pub(crate) coin: String,
    /// Signed USDC amount. Negative means the account paid funding.
    pub(crate) usdc: String,
    #[serde(rename = "fundingRate")]
    pub(crate) funding_rate: String,
}

/// One entry of `fundingHistory`.
/// This is a market-rate observation, not an account funding payment.
#[derive(Debug, Deserialize)]
pub(crate) struct RawFundingHistory {
    #[serde(rename = "fundingRate")]
    pub(crate) funding_rate: String,
    pub(crate) time: i64,
}

/// One entry of `userNonFundingLedgerUpdates`.
///
/// `delta` is left as raw JSON here: its shape depends on `delta.type`, and the
/// typed split happens in [`super::native`].
#[derive(Debug, Deserialize)]
pub(crate) struct RawLedgerUpdate {
    pub(crate) delta: Value,
    pub(crate) time: i64,
    pub(crate) hash: String,
}

/// An order as the `orderUpdates` private stream reports it.
#[derive(Debug, Deserialize)]
pub(crate) struct RawStreamOrder {
    pub(crate) order: RawStreamOrderBody,
    pub(crate) status: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RawStreamOrderBody {
    pub(crate) coin: String,
    pub(crate) side: String,
    #[serde(rename = "limitPx")]
    pub(crate) limit_px: String,
    pub(crate) sz: String,
    #[serde(rename = "origSz")]
    pub(crate) orig_sz: String,
    pub(crate) oid: u64,
    pub(crate) timestamp: i64,
}

/// The `spotState` private stream frame's `data`.
#[derive(Debug, Deserialize)]
pub(crate) struct RawStreamSpotState {
    #[serde(rename = "spotState")]
    pub(crate) spot_state: RawSpotState,
}

/// The envelope every `/exchange` action answers with.
///
/// `status` is `ok` or `err`, and it is the *only* thing that says which:
/// Hyperliquid returns HTTP 200 for a rejected action, so a status-code check
/// would read a rejection as a success.
#[derive(Debug, Deserialize)]
pub(crate) struct RawActionResponse {
    pub(crate) status: String,
    pub(crate) response: Value,
}

/// Reads a field represented as either a JSON string or number.
fn number_or_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;

    match Value::deserialize(deserializer)? {
        Value::String(text) => Ok(text),
        // `arbitrary_precision` keeps the digits, so this is not a round trip
        // through a float.
        Value::Number(number) => Ok(number.to_string()),
        other => Err(D::Error::custom(format!(
            "expected a number or a string, got {other}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Scalars
// ---------------------------------------------------------------------------

/// Reads a string-encoded number exactly or returns [`Error::Decode`].
pub(crate) fn decimal(text: &str, field: &str) -> Result<Decimal> {
    // Exponent notation is accepted without rounding.
    crate::adapters::decimal::exact(text)
        .map_err(|err| Error::decode(format!("`{field}` is not a decimal: {text} ({err})")))
}

/// Converts one of Hyperliquid's millisecond timestamps.
pub(crate) fn millis(millis: i64, field: &str) -> Result<Timestamp> {
    millis
        .checked_mul(1_000_000)
        .map(Timestamp::from_nanos)
        .ok_or_else(|| Error::decode(format!("`{field}` is out of range: {millis}ms")))
}

/// Reads Hyperliquid's one-letter side spelling.
///
/// `B` is the bid side and `A` the ask side. On a trade this names the taker;
/// on an order it names the order's own direction.
pub(crate) fn side(raw: &str) -> Result<Side> {
    match raw {
        "B" => Ok(Side::Buy),
        "A" => Ok(Side::Sell),
        _ => Err(Error::decode(format!("unknown Hyperliquid side `{raw}`"))),
    }
}

/// Maps a supported candle interval to Hyperliquid's spelling.
pub(crate) fn interval_name(interval: Interval) -> Option<&'static str> {
    Some(match interval {
        Interval::Min1 => "1m",
        Interval::Min3 => "3m",
        Interval::Min5 => "5m",
        Interval::Min15 => "15m",
        Interval::Min30 => "30m",
        Interval::Hour1 => "1h",
        Interval::Hour2 => "2h",
        Interval::Hour4 => "4h",
        Interval::Hour8 => "8h",
        Interval::Hour12 => "12h",
        Interval::Day1 => "1d",
        Interval::Day3 => "3d",
        Interval::Week1 => "1w",
        Interval::Month1 => "1M",
        Interval::Sec1 => return None,
    })
}

/// Reads Hyperliquid's interval spelling back.
pub(crate) fn interval_from_name(raw: &str) -> Option<Interval> {
    Some(match raw {
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
        "1M" => Interval::Month1,
        _ => return None,
    })
}

/// Reads a successful body, reporting a shape change as [`Error::Decode`].
pub(crate) fn json<T: for<'de> Deserialize<'de>>(body: &str) -> Result<T> {
    serde_json::from_str(body)
        .map_err(|err| Error::decode(format!("unreadable Hyperliquid response: {err}")))
}

/// Turns a non-2xx REST response into an [`Error::Exchange`].
///
/// Hyperliquid's transport-level failures answer with a bare string instead of
/// a JSON envelope, so the body is kept verbatim.
pub(crate) fn http_error(status: u16, body: &str) -> Error {
    Error::exchange_http(EXCHANGE, status, "unknown", body.trim())
}

/// Reads an `/exchange` envelope, which is where Hyperliquid hides its
/// rejections.
///
/// A rejected action still arrives as HTTP 200 with `status: "err"`, so this is
/// the only place the verdict can be read. Returns the `response` value of a
/// successful action.
pub(crate) fn action_response(body: &str) -> Result<Value> {
    let envelope: RawActionResponse = json(body)?;

    if envelope.status != "ok" {
        return Err(Error::exchange(
            EXCHANGE,
            envelope.status,
            response_message(&envelope.response),
        ));
    }
    Ok(envelope.response)
}

/// Pulls the human-readable half out of an error `response`, which Hyperliquid
/// sends as a bare string but has been known to nest.
fn response_message(response: &Value) -> String {
    match response {
        Value::String(message) => message.clone(),
        Value::Null => "hyperliquid rejected the action without saying why".to_string(),
        other => other.to_string(),
    }
}

/// Reads the per-order verdict out of an accepted `order` action response.
///
/// The envelope says `ok` even when the single order inside it was refused, so
/// `statuses[0]` is a second rejection point that a status check would miss.
pub(crate) fn order_ack_id(response: &Value) -> Result<(String, OrderStatus)> {
    let status = response
        .get("data")
        .and_then(|data| data.get("statuses"))
        .and_then(Value::as_array)
        .and_then(|statuses| statuses.first())
        .ok_or_else(|| Error::decode("hyperliquid order response carries no `data.statuses`"))?;

    if let Some(message) = status.get("error").and_then(Value::as_str) {
        return Err(Error::exchange(EXCHANGE, "order_rejected", message));
    }
    if let Some(oid) = status.get("resting").and_then(|resting| resting.get("oid")) {
        return Ok((oid_text(oid)?, OrderStatus::Open));
    }
    if let Some(filled) = status.get("filled") {
        let oid = filled
            .get("oid")
            .ok_or_else(|| Error::decode("hyperliquid filled order carries no `oid`"))?;
        return Ok((oid_text(oid)?, OrderStatus::Filled));
    }

    Err(Error::decode(format!(
        "unexpected hyperliquid order status `{status}`"
    )))
}

fn oid_text(oid: &Value) -> Result<String> {
    oid.as_u64()
        .map(|oid| oid.to_string())
        .or_else(|| oid.as_str().map(str::to_string))
        .ok_or_else(|| Error::decode(format!("hyperliquid order id `{oid}` is not a number")))
}

// ---------------------------------------------------------------------------
// Symbols
// ---------------------------------------------------------------------------

/// One tradable market, in both Hyperliquid's terms and `maxt`'s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Asset {
    pub(crate) market: Market,
    /// What Hyperliquid calls it on the wire.
    pub(crate) native: String,
    /// What `/exchange` actions call it. Perpetuals number from zero; spot pairs
    /// are offset by [`SPOT_ASSET_ID_OFFSET`].
    pub(crate) asset_id: u32,
    /// Decimal places allowed in an order size for this asset.
    pub(crate) size_decimals: u32,
    pub(crate) max_leverage: Option<u32>,
    /// Whether the asset refuses cross margin.
    pub(crate) only_isolated: bool,
    pub(crate) status: MarketStatus,
}

/// Default perpetual and spot markets loaded from `meta` and `spotMeta`.
///
/// Spot wire symbols are read from metadata rather than derived from asset
/// names.
#[derive(Debug, Clone, Default)]
pub(crate) struct Universe {
    assets: Vec<Asset>,
}

impl Universe {
    /// Builds the table from the `meta` and `spotMeta` responses.
    pub(crate) fn new(perp: &RawPerpMeta, spot: &RawSpotMeta) -> Result<Self> {
        let mut assets = Vec::with_capacity(perp.universe.len() + spot.universe.len());

        for (index, asset) in perp.universe.iter().enumerate() {
            // HIP-3 markets belong to separate DEX metadata and asset-id spaces.
            if asset.name.contains(':') {
                continue;
            }
            let asset_id = u32::try_from(index)
                .map_err(|_| Error::decode("hyperliquid perpetual universe is implausibly long"))?;
            assets.push(Asset {
                market: Market::perpetual(Exchange::Hyperliquid, &asset.name, SETTLE_ASSET),
                native: asset.name.clone(),
                asset_id,
                size_decimals: asset.sz_decimals,
                max_leverage: asset.max_leverage,
                only_isolated: asset.only_isolated,
                status: if asset.is_delisted {
                    MarketStatus::Delisted
                } else {
                    MarketStatus::Active
                },
            });
        }

        for pair in &spot.universe {
            let base = spot_token(spot, pair.tokens[0], "base")?;
            let quote = spot_token(spot, pair.tokens[1], "quote")?;
            let asset_id = pair
                .index
                .checked_add(SPOT_ASSET_ID_OFFSET)
                .ok_or_else(|| {
                    Error::decode("hyperliquid spot pair index overflows the spot asset id offset")
                })?;
            assets.push(Asset {
                market: Market::spot(Exchange::Hyperliquid, &base.name, &quote.name),
                native: pair.name.clone(),
                asset_id,
                size_decimals: base.sz_decimals,
                max_leverage: None,
                only_isolated: false,
                status: MarketStatus::Active,
            });
        }

        Ok(Self { assets })
    }

    /// The assets of one kind, in Hyperliquid's own listing order.
    pub(crate) fn of_kind(&self, kind: MarketKind) -> impl Iterator<Item = &Asset> {
        self.assets
            .iter()
            .filter(move |asset| asset.market.kind == kind)
    }

    /// Looks a market up by identity.
    pub(crate) fn asset(&self, market: &Market) -> Result<&Asset> {
        if market.exchange != Exchange::Hyperliquid {
            return Err(Error::invalid_request(
                "market",
                format!("{market} is not a Hyperliquid market"),
            ));
        }

        self.assets
            .iter()
            .find(|asset| &asset.market == market)
            .ok_or_else(|| {
                Error::invalid_request("market", format!("hyperliquid does not list {market}"))
            })
    }

    /// Hyperliquid's own name for a market.
    pub(crate) fn native_symbol(&self, market: &Market) -> Result<&str> {
        Ok(&self.asset(market)?.native)
    }

    /// Resolves either an indexed or slash-form native spot symbol.
    pub(crate) fn market_from_native_symbol(&self, native: &str) -> Result<&Market> {
        self.assets
            .iter()
            .find(|asset| asset.native == native || asset.index_symbol() == native)
            .map(|asset| &asset.market)
            .ok_or_else(|| Error::decode(format!("hyperliquid sent an unlisted market `{native}`")))
    }
}

impl Asset {
    /// Returns the indexed spot symbol used by market-data feeds.
    fn index_symbol(&self) -> String {
        match self.market.kind {
            MarketKind::Spot => {
                format!("@{}", self.asset_id.saturating_sub(SPOT_ASSET_ID_OFFSET))
            }
            MarketKind::Perpetual => self.native.clone(),
        }
    }

    /// Maximum decimal places allowed in a fractional order price.
    ///
    /// The cap is `6 - size_decimals` for perpetuals and
    /// `8 - size_decimals` for spot. Significant digits are validated
    /// separately when an order is built.
    pub(crate) fn price_decimals(&self) -> u32 {
        let max: u32 = match self.market.kind {
            MarketKind::Perpetual => 6,
            MarketKind::Spot => 8,
        };
        max.saturating_sub(self.size_decimals)
    }
}

fn spot_token<'a>(spot: &'a RawSpotMeta, index: u32, role: &str) -> Result<&'a RawSpotToken> {
    spot.tokens
        .iter()
        .find(|token| token.index == index)
        .ok_or_else(|| {
            Error::decode(format!(
                "hyperliquid spot pair names an unknown {role} token index {index}"
            ))
        })
}

// ---------------------------------------------------------------------------
// Raw to domain
// ---------------------------------------------------------------------------

pub(crate) fn market_info(asset: &Asset) -> MarketInfo {
    MarketInfo {
        market: asset.market.clone(),
        native_symbol: asset.native.clone(),
        status: asset.status,
        // Hyperliquid publishes no localized asset names.
        korean_name: None,
        english_name: None,
    }
}

pub(crate) fn order_book(raw: &RawBook, universe: &Universe) -> Result<OrderBook> {
    let mut bids = read_levels(&raw.levels[0])?;
    let mut asks = read_levels(&raw.levels[1])?;

    // Hyperliquid ships each side best-first already, but `OrderBook`'s ordering
    // is a guarantee to the caller and cheap to enforce, so it is enforced.
    bids.sort_by(|left, right| right.price.cmp(&left.price));
    asks.sort_by(|left, right| left.price.cmp(&right.price));

    Ok(OrderBook {
        market: universe.market_from_native_symbol(&raw.coin)?.clone(),
        timestamp: millis(raw.time, "time")?,
        bids,
        asks,
    })
}

fn read_levels(raw: &[RawLevel]) -> Result<Vec<Level>> {
    raw.iter()
        .map(|level| {
            Ok(Level {
                price: decimal(&level.px, "px")?,
                quantity: decimal(&level.sz, "sz")?,
            })
        })
        .collect()
}

pub(crate) fn trade(raw: &RawTrade, universe: &Universe) -> Result<Trade> {
    Ok(Trade {
        market: universe.market_from_native_symbol(&raw.coin)?.clone(),
        timestamp: millis(raw.time, "time")?,
        price: decimal(&raw.px, "px")?,
        quantity: decimal(&raw.sz, "sz")?,
        taker_side: side(&raw.side)?,
        id: Some(raw.tid.to_string()),
    })
}

/// Maps an asset context into a ticker summary.
///
/// [`Ticker::last_price`] uses `midPx`, falling back to `markPx`; neither field
/// is a recent execution price. [`Ticker::change`] and
/// [`Ticker::change_rate`] compare that selected price with `prevDayPx`.
/// Asset contexts contain no timestamp or trade time, so `at` becomes
/// [`Ticker::timestamp`] and [`Ticker::last_trade_time`] is `None`.
pub(crate) fn ticker(raw: &RawAssetCtx, market: &Market, at: Timestamp) -> Result<Ticker> {
    // `midPx` is optional; `markPx` is the provider-summary fallback.
    let last_price = match (&raw.mid_px, &raw.mark_px) {
        (Some(mid), _) => decimal(mid, "midPx")?,
        (None, Some(mark)) => decimal(mark, "markPx")?,
        (None, None) => {
            return Err(Error::decode(
                "hyperliquid asset context carries neither `midPx` nor `markPx`",
            ));
        }
    };
    let previous = raw
        .prev_day_px
        .as_deref()
        .map(|price| decimal(price, "prevDayPx"))
        .transpose()?;
    let change = previous.map(|previous| last_price - previous);

    Ok(Ticker {
        market: market.clone(),
        timestamp: at,
        last_trade_time: None,
        last_price,
        change,
        change_rate: match (change, previous) {
            (Some(change), Some(previous)) if !previous.is_zero() => Some(change / previous),
            _ => None,
        },
        // Hyperliquid's asset contexts carry no session high or low.
        high: None,
        low: None,
        volume: raw
            .day_base_volume
            .as_deref()
            .map(|volume| decimal(volume, "dayBaseVlm"))
            .transpose()?,
        quote_volume: raw
            .day_notional_volume
            .as_deref()
            .map(|volume| decimal(volume, "dayNtlVlm"))
            .transpose()?,
    })
}

/// Maps a candle and marks it closed when `now` is later than the inclusive
/// close millisecond `T`.
pub(crate) fn candle(raw: &RawCandle, universe: &Universe, now: Timestamp) -> Result<Candle> {
    let interval = interval_from_name(&raw.interval).ok_or_else(|| {
        Error::decode(format!(
            "hyperliquid sent an unmapped candle interval `{}`",
            raw.interval
        ))
    })?;

    Ok(Candle {
        market: universe.market_from_native_symbol(&raw.coin)?.clone(),
        interval,
        open_time: millis(raw.open_time, "t")?,
        open: decimal(&raw.open, "o")?,
        high: decimal(&raw.high, "h")?,
        low: decimal(&raw.low, "l")?,
        close: decimal(&raw.close, "c")?,
        volume: decimal(&raw.volume, "v")?,
        // Hyperliquid reports a candle's base volume only.
        quote_volume: None,
        closed: millis(raw.close_time, "T")? < now,
    })
}

pub(crate) fn balance(raw: &RawSpotBalance) -> Result<Balance> {
    let total = decimal(&raw.total, "total")?;
    let locked = decimal(&raw.hold, "hold")?;

    Ok(Balance {
        asset: raw.coin.to_ascii_uppercase(),
        // Hyperliquid reports the total and the held part; free is the rest.
        available: total - locked,
        locked,
    })
}

pub(crate) fn open_order(raw: &RawOpenOrder, universe: &Universe) -> Result<Order> {
    let remaining = decimal(&raw.sz, "sz")?;
    let original = decimal(&raw.orig_sz, "origSz")?;
    let filled = original - remaining;

    Ok(Order {
        id: raw.oid.to_string(),
        market: universe.market_from_native_symbol(&raw.coin)?.clone(),
        side: side(&raw.side)?,
        // Everything `frontendOpenOrders` returns is still working, so the only
        // question left is whether anything has filled yet.
        status: if filled.is_zero() {
            OrderStatus::Open
        } else {
            OrderStatus::PartiallyFilled
        },
        filled_quantity: filled,
        remaining_quantity: remaining,
        price: Some(decimal(&raw.limit_px, "limitPx")?),
        created_at: Some(millis(raw.timestamp, "timestamp")?),
    })
}

pub(crate) fn stream_order(raw: &RawStreamOrder, universe: &Universe) -> Result<Order> {
    let remaining = decimal(&raw.order.sz, "sz")?;
    let original = decimal(&raw.order.orig_sz, "origSz")?;
    let filled = original - remaining;

    Ok(Order {
        id: raw.order.oid.to_string(),
        market: universe.market_from_native_symbol(&raw.order.coin)?.clone(),
        side: side(&raw.order.side)?,
        status: order_status(&raw.status, filled),
        filled_quantity: filled,
        remaining_quantity: remaining,
        price: Some(decimal(&raw.order.limit_px, "limitPx")?),
        created_at: Some(millis(raw.order.timestamp, "timestamp")?),
    })
}

/// Maps an `orderUpdates` status.
///
/// Hyperliquid spells out several distinct ways an order can be taken off the
/// book, giving margin, reduce-only, and self-trade cancellations each their
/// own word. All of them mean cancelled to a caller.
pub(crate) fn order_status(status: &str, filled: Decimal) -> OrderStatus {
    match status {
        "open" if filled.is_zero() => OrderStatus::Open,
        "open" => OrderStatus::PartiallyFilled,
        "filled" => OrderStatus::Filled,
        "canceled"
        | "marginCanceled"
        | "reduceOnlyCanceled"
        | "vaultWithdrawalCanceled"
        | "openInterestCapCanceled"
        | "selfTradeCanceled"
        | "siblingFilledCanceled"
        | "delistedCanceled"
        | "liquidatedCanceled"
        | "scheduledCancel" => OrderStatus::Cancelled,
        "rejected"
        | "badAloPx"
        | "tickRejected"
        | "minTradeNtlRejected"
        | "perpMarginRejected"
        | "reduceOnlyRejected"
        | "insufficientSpotBalanceRejected"
        | "oracleRejected" => OrderStatus::Rejected,
        _ => OrderStatus::Unknown,
    }
}

/// Reads one `assetPositions` row, including a syntactically valid zero size.
/// The common client filters flat positions from the returned list.
pub(crate) fn position(raw: &RawPosition, universe: &Universe) -> Result<Position> {
    let signed = decimal(&raw.szi, "szi")?;

    Ok(Position {
        market: universe.market_from_native_symbol(&raw.coin)?.clone(),
        // Zero has no direction, so it is tested before the sign: an unsigned
        // zero would otherwise read as a long.
        side: match signed.is_sign_negative() {
            _ if signed.is_zero() => None,
            true => Some(Side::Sell),
            false => Some(Side::Buy),
        },
        quantity: signed.abs(),
        entry_price: raw
            .entry_px
            .as_deref()
            .map(|price| decimal(price, "entryPx"))
            .transpose()?,
        // `clearinghouseState` reports what the position is worth, not what the
        // exchange currently marks the market at.
        mark_price: None,
        notional: Some(decimal(&raw.position_value, "positionValue")?),
        unrealized_pnl: Some(decimal(&raw.unrealized_pnl, "unrealizedPnl")?),
        leverage: Some(Decimal::from(raw.leverage.value)),
        margin_mode: margin_mode(&raw.leverage.margin_type),
    })
}

/// Reads Hyperliquid's `leverage.type`, which is `cross` or `isolated`.
pub(crate) fn margin_mode(raw: &str) -> Option<MarginMode> {
    match raw {
        "cross" => Some(MarginMode::Cross),
        "isolated" => Some(MarginMode::Isolated),
        _ => None,
    }
}

pub(crate) fn margin_summary(raw: &RawPerpState) -> Result<MarginSummary> {
    Ok(MarginSummary {
        asset: SETTLE_ASSET.to_string(),
        equity: Some(decimal(&raw.margin_summary.account_value, "accountValue")?),
        margin_balance: Some(decimal(
            &raw.margin_summary.total_margin_used,
            "totalMarginUsed",
        )?),
        // `withdrawable` is what is free of every margin requirement, which is
        // exactly what can back a new position.
        available_balance: Some(decimal(&raw.withdrawable, "withdrawable")?),
    })
}

/// Builds the cursor for the next page of a time-ranged history.
///
/// Hyperliquid pages by time, so the cursor is the millisecond just past the
/// newest entry on this page. It is opaque to the caller, and only
/// [`super::rest`] reads it back.
pub(crate) fn time_cursor(newest_ms: i64) -> Result<Cursor> {
    newest_ms
        .checked_add(1)
        .map(|next| Cursor(next.to_string()))
        .ok_or_else(|| Error::decode("hyperliquid history cursor cannot move forward"))
}

/// Reads a cursor this adapter produced back into a start time.
pub(crate) fn cursor_start_ms(cursor: &Cursor) -> Result<i64> {
    cursor.as_str().parse().map_err(|_| {
        Error::invalid_request("cursor", "pass back the cursor `maxt` returned, unchanged")
    })
}

/// The exchange's own rejection for an interval it does not aggregate.
pub(crate) fn unsupported_interval(interval: Interval, feature: Feature) -> Error {
    Error::unsupported(
        feature,
        EXCHANGE,
        format!("hyperliquid aggregates candles from one minute upward, not {interval:?}"),
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals
    pub(crate) const META: &str = r#"{
      "universe": [
        {"name": "BTC", "szDecimals": 5, "maxLeverage": 50},
        {"name": "ETH", "szDecimals": 4, "maxLeverage": 50},
        {"name": "KPEPE", "szDecimals": 0, "maxLeverage": 10, "onlyIsolated": true},
        {"name": "test:ABC", "szDecimals": 2, "maxLeverage": 3}
      ]
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot
    pub(crate) const SPOT_META: &str = r#"{
      "tokens": [
        {
          "name": "USDC",
          "szDecimals": 8,
          "weiDecimals": 8,
          "index": 0,
          "tokenId": "0x6d1e7cde53ba9467b783cb7c530ce054",
          "isCanonical": true
        },
        {
          "name": "PURR",
          "szDecimals": 0,
          "weiDecimals": 5,
          "index": 1,
          "tokenId": "0xc1fb593aeffbeb02f85e0308e9956a90",
          "isCanonical": true
        },
        {
          "name": "HYPE",
          "szDecimals": 2,
          "weiDecimals": 8,
          "index": 150,
          "tokenId": "0x00000000000000000000000000000096",
          "isCanonical": true
        }
      ],
      "universe": [
        {"name": "PURR/USDC", "tokens": [1, 0], "index": 0, "isCanonical": true},
        {"name": "@107", "tokens": [150, 0], "index": 107, "isCanonical": false}
      ]
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint#l2-book-snapshot
    pub(crate) const L2_BOOK: &str = r#"{
      "coin": "BTC",
      "time": 1754450974231,
      "levels": [
        [
          {"px": "113376.0", "sz": "4.13714", "n": 8},
          {"px": "113377.0", "sz": "7.6699", "n": 17}
        ],
        [
          {"px": "113398.0", "sz": "0.20000", "n": 5},
          {"px": "113397.0", "sz": "0.11543", "n": 3}
        ]
      ]
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint#candle-snapshot
    pub(crate) const CANDLE_SNAPSHOT: &str = r#"[
      {
        "T": 1681924499999,
        "c": "29258.0",
        "h": "29309.0",
        "i": "15m",
        "l": "29250.0",
        "n": 189,
        "o": "29295.0",
        "s": "BTC",
        "t": 1681923600000,
        "v": "0.98639"
      }
    ]"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions
    pub(crate) const WS_TRADES: &str = r#"{
      "channel": "trades",
      "data": [
        {
          "coin": "BTC",
          "side": "B",
          "px": "29295.0",
          "sz": "0.98639",
          "hash": "0xa166e3fa63c25663024b03f2e0da011a00307e4017465df020210d3d432e7cb8",
          "time": 1681923600000,
          "tid": 118906512037719,
          "users": [
            "0x0000000000000000000000000000000000000001",
            "0x0000000000000000000000000000000000000002"
          ]
        }
      ]
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals#retrieve-users-perpetuals-account-summary
    pub(crate) const CLEARINGHOUSE_STATE: &str = r#"{
      "assetPositions": [
        {
          "position": {
            "coin": "ETH",
            "cumFunding": {"allTime": "514.085417", "sinceChange": "0.0", "sinceOpen": "0.0"},
            "entryPx": "2986.3",
            "leverage": {"rawUsd": "-95.059824", "type": "isolated", "value": 20},
            "liquidationPx": "2866.26936529",
            "marginUsed": "4.967826",
            "maxLeverage": 50,
            "positionValue": "100.02765",
            "returnOnEquity": "-0.0026789",
            "szi": "0.0335",
            "unrealizedPnl": "-0.0026789"
          },
          "type": "oneWay"
        }
      ],
      "crossMaintenanceMarginUsed": "0.0",
      "crossMarginSummary": {
        "accountValue": "13104.514502",
        "totalMarginUsed": "0.0",
        "totalNtlPos": "0.0",
        "totalRawUsd": "13104.514502"
      },
      "marginSummary": {
        "accountValue": "13104.514502",
        "totalMarginUsed": "4.967826",
        "totalNtlPos": "100.02765",
        "totalRawUsd": "13004.486852"
      },
      "time": 1708622398623,
      "withdrawable": "13104.514502"
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/spot#retrieve-a-users-token-balances
    pub(crate) const SPOT_STATE: &str = r#"{
      "balances": [
        {"coin": "USDC", "token": 0, "hold": "0.0", "total": "14.625485"},
        {"coin": "PURR", "token": 1, "hold": "3.0", "total": "2000.0"}
      ]
    }"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint#retrieve-a-users-open-orders-with-additional-frontend-info
    pub(crate) const FRONTEND_OPEN_ORDERS: &str = r#"[
      {
        "coin": "BTC",
        "isPositionTpsl": false,
        "isTrigger": false,
        "limitPx": "29792.0",
        "oid": 91490942,
        "orderType": "Limit",
        "origSz": "0.0",
        "reduceOnly": false,
        "side": "A",
        "sz": "0.0",
        "tif": "Gtc",
        "timestamp": 1681247412573,
        "triggerCondition": "N/A",
        "triggerPx": "0.0"
      }
    ]"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint/perpetuals#retrieve-historical-funding-rates
    pub(crate) const FUNDING_HISTORY: &str = r#"[
      {
        "coin": "BTC",
        "fundingRate": "-0.00022196",
        "premium": "-0.00052196",
        "time": 1683849600076
      },
      {
        "coin": "BTC",
        "fundingRate": "0.00001250",
        "premium": "0.00000000",
        "time": 1683853200000
      }
    ]"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint#retrieve-a-users-funding-history-or-non-funding-ledger-updates
    pub(crate) const USER_FUNDING: &str = r#"[
      {
        "delta": {
          "coin": "ETH",
          "fundingRate": "0.0000125",
          "szi": "49.1477",
          "type": "funding",
          "usdc": "-0.0568"
        },
        "hash": "0xa166e3fa63c25663024b03f2e0da011a00307e4017465df020210d3d432e7cb8",
        "time": 1681222254710
      }
    ]"#;

    pub(crate) fn universe() -> Universe {
        Universe::new(
            &json::<RawPerpMeta>(META).expect("official meta payload"),
            &json::<RawSpotMeta>(SPOT_META).expect("official spotMeta payload"),
        )
        .expect("a universe")
    }

    pub(crate) fn btc_perp() -> Market {
        Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC")
    }

    fn decimal_of(text: &str) -> Decimal {
        decimal(text, "test").expect("test literal is a decimal")
    }

    #[test]
    fn a_perpetual_is_its_bare_coin_name_settled_in_usdc() {
        let universe = universe();

        assert_eq!(universe.native_symbol(&btc_perp()).expect("listed"), "BTC");
        assert_eq!(
            universe.market_from_native_symbol("BTC").expect("listed"),
            &btc_perp()
        );
        assert_eq!(btc_perp().quote, SETTLE_ASSET);
    }

    #[test]
    fn a_spot_pair_is_named_by_index_and_maps_back_from_either_spelling() {
        let universe = universe();
        let hype = Market::spot(Exchange::Hyperliquid, "HYPE", "USDC");
        let purr = Market::spot(Exchange::Hyperliquid, "PURR", "USDC");

        // Indexed spot symbols resolve through `spotMeta`.
        assert_eq!(universe.native_symbol(&hype).expect("listed"), "@107");
        assert_eq!(
            universe.market_from_native_symbol("@107").expect("listed"),
            &hype
        );

        // Slash-form metadata symbols also resolve from their feed index.
        assert_eq!(universe.native_symbol(&purr).expect("listed"), "PURR/USDC");
        assert_eq!(
            universe
                .market_from_native_symbol("PURR/USDC")
                .expect("listed"),
            &purr
        );
        assert_eq!(
            universe.market_from_native_symbol("@0").expect("listed"),
            &purr
        );
    }

    #[test]
    fn spot_and_perpetual_on_one_coin_are_two_different_assets() {
        let universe = universe();
        let purr_spot = universe
            .asset(&Market::spot(Exchange::Hyperliquid, "PURR", "USDC"))
            .expect("listed");
        let btc_perp = universe.asset(&btc_perp()).expect("listed");

        // Spot and perpetual action ids occupy separate ranges.
        assert_eq!(btc_perp.asset_id, 0);
        assert_eq!(purr_spot.asset_id, SPOT_ASSET_ID_OFFSET);
        assert_eq!(purr_spot.market.kind, MarketKind::Spot);
        assert_eq!(btc_perp.market.kind, MarketKind::Perpetual);
    }

    #[test]
    fn a_builder_deployed_perpetual_is_left_out_of_the_main_universe() {
        // A separate DEX cannot share the default perpetual index space.
        let universe = universe();

        assert!(universe.market_from_native_symbol("test:ABC").is_err());
        assert_eq!(universe.of_kind(MarketKind::Perpetual).count(), 3);
        assert_eq!(universe.of_kind(MarketKind::Spot).count(), 2);
    }

    #[test]
    fn a_market_from_another_exchange_is_a_caller_mistake_not_a_lookup_miss() {
        let universe = universe();

        assert!(matches!(
            universe.asset(&Market::spot(Exchange::Upbit, "BTC", "KRW")),
            Err(Error::InvalidRequest { field, .. }) if field == "market"
        ));
        assert!(matches!(
            universe.asset(&Market::spot(Exchange::Hyperliquid, "NOPE", "USDC")),
            Err(Error::InvalidRequest { field, .. }) if field == "market"
        ));
    }

    #[test]
    fn decimals_keep_every_digit_hyperliquid_sent() {
        // Exact parsing preserves small rates and large balances.
        assert_eq!(decimal_of("0.0000125").to_string(), "0.0000125");
        assert_eq!(
            decimal_of("1386929.37231066771348207123").to_string(),
            "1386929.37231066771348207123"
        );
        assert_eq!(decimal_of("-0.00022196"), Decimal::new(-22_196, 8));
    }

    #[test]
    fn a_number_too_precise_to_hold_is_a_decode_error_not_a_rounded_price() {
        // Twenty-nine decimal places is one past what `Decimal` carries.
        let error = decimal("0.000000000000000000000000000001", "px").unwrap_err();

        assert!(matches!(error, Error::Decode { .. }));
    }

    #[test]
    fn a_book_comes_back_best_first_on_both_sides() {
        // The fixture is deliberately unsorted.
        let raw: RawBook = json(L2_BOOK).expect("official l2Book payload");
        let book = order_book(&raw, &universe()).expect("a book");

        assert_eq!(
            book.best_bid().expect("a bid").price,
            decimal_of("113377.0")
        );
        assert_eq!(
            book.best_ask().expect("an ask").price,
            decimal_of("113397.0")
        );
        assert_eq!(book.spread(), Some(decimal_of("20.0")));
        assert_eq!(book.timestamp, Timestamp::from_millis(1_754_450_974_231));
        assert_eq!(book.market, btc_perp());
    }

    #[test]
    fn a_trade_carries_hyperliquids_own_id_and_names_the_taker() {
        #[derive(Deserialize)]
        struct Frame {
            data: Vec<RawTrade>,
        }
        let frame: Frame = json(WS_TRADES).expect("official trades frame");
        let trade = trade(&frame.data[0], &universe()).expect("a trade");

        // `B` means the taker lifted an ask.
        assert_eq!(trade.taker_side, Side::Buy);
        assert_eq!(trade.id.as_deref(), Some("118906512037719"));
        assert_eq!(trade.price, decimal_of("29295.0"));
        assert_eq!(trade.timestamp, Timestamp::from_millis(1_681_923_600_000));
    }

    #[test]
    fn a_candle_is_closed_only_once_its_window_has_ended() {
        let raw: Vec<RawCandle> = json(CANDLE_SNAPSHOT).expect("official candleSnapshot payload");
        let universe = universe();
        let after = candle(
            &raw[0],
            &universe,
            Timestamp::from_millis(1_681_924_600_000),
        )
        .expect("a candle");
        let during = candle(
            &raw[0],
            &universe,
            Timestamp::from_millis(1_681_924_000_000),
        )
        .expect("a candle");

        assert_eq!(after.open_time, Timestamp::from_millis(1_681_923_600_000));
        assert_eq!(after.interval, Interval::Min15);
        assert_eq!(after.open, decimal_of("29295.0"));
        assert_eq!(after.volume, decimal_of("0.98639"));
        assert_eq!(after.quote_volume, None);
        assert!(after.closed);
        assert!(!during.closed);
    }

    #[test]
    fn the_last_millisecond_a_window_covers_is_still_inside_it() {
        // `T` is inclusive: the candle closes only after that millisecond.
        let raw: Vec<RawCandle> = json(CANDLE_SNAPSHOT).expect("official candleSnapshot payload");
        let universe = universe();
        let at_the_boundary = candle(
            &raw[0],
            &universe,
            Timestamp::from_millis(1_681_924_499_999),
        )
        .expect("a candle");
        let one_millisecond_later = candle(
            &raw[0],
            &universe,
            Timestamp::from_millis(1_681_924_500_000),
        )
        .expect("a candle");

        assert!(!at_the_boundary.closed);
        assert!(one_millisecond_later.closed);
    }

    #[test]
    fn every_interval_hyperliquid_aggregates_round_trips_through_its_name() {
        for interval in [
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
            let name = interval_name(interval).expect("an aggregated interval");
            assert_eq!(interval_from_name(name), Some(interval), "{name}");
        }
        assert_eq!(interval_name(Interval::Sec1), None);
    }

    #[test]
    fn a_ticker_derives_its_change_from_the_previous_day_close() {
        let raw: RawAssetCtx = json(
            r#"{
              "dayNtlVlm": "1169046.29406",
              "funding": "0.0000125",
              "impactPxs": ["14.3047", "14.3444"],
              "markPx": "14.3161",
              "midPx": "14.314",
              "openInterest": "688.11",
              "oraclePx": "14.325",
              "premium": "0.00031774",
              "prevDayPx": "15.322",
              "dayBaseVlm": "81584.5"
            }"#,
        )
        .expect("official asset context payload");
        let at = Timestamp::from_millis(1_700_000_000_000);
        let ticker = ticker(&raw, &btc_perp(), at).expect("a ticker");

        assert_eq!(ticker.last_price, decimal_of("14.314"));
        assert_eq!(ticker.change, Some(decimal_of("-1.008")));
        assert_eq!(ticker.volume, Some(decimal_of("81584.5")));
        assert_eq!(ticker.quote_volume, Some(decimal_of("1169046.29406")));
        // Asset contexts supply neither a context time nor a trade time.
        assert_eq!(ticker.timestamp, at);
        assert_eq!(ticker.last_trade_time, None);
    }

    #[test]
    fn a_ticker_falls_back_to_the_mark_price_on_a_one_sided_market() {
        let raw: RawAssetCtx =
            json(r#"{"markPx": "14.3161", "prevDayPx": "15.322"}"#).expect("a thin market");
        let ticker = ticker(&raw, &btc_perp(), Timestamp::default()).expect("a ticker");

        assert_eq!(ticker.last_price, decimal_of("14.3161"));
        assert_eq!(ticker.volume, None);
    }

    #[test]
    fn a_spot_balance_splits_the_total_into_free_and_held() {
        let raw: RawSpotState = json(SPOT_STATE).expect("official spot balances payload");
        let purr = balance(&raw.balances[1]).expect("a balance");

        assert_eq!(purr.asset, "PURR");
        assert_eq!(purr.locked, decimal_of("3.0"));
        assert_eq!(purr.available, decimal_of("1997.0"));
        assert_eq!(purr.total(), decimal_of("2000.0"));
    }

    #[test]
    fn a_short_position_keeps_its_size_unsigned_and_moves_the_sign_to_the_side() {
        let mut raw: RawPerpState =
            json(CLEARINGHOUSE_STATE).expect("official clearinghouseState payload");
        let universe = universe();

        let long = position(&raw.asset_positions[0].position, &universe).expect("a position");
        assert_eq!(long.side, Some(Side::Buy));
        assert_eq!(long.quantity, decimal_of("0.0335"));
        assert_eq!(long.margin_mode, Some(MarginMode::Isolated));
        assert_eq!(long.leverage, Some(Decimal::from(20)));

        raw.asset_positions[0].position.szi = "-0.0335".to_string();
        let short = position(&raw.asset_positions[0].position, &universe).expect("a position");
        assert_eq!(short.side, Some(Side::Sell));
        assert_eq!(short.quantity, decimal_of("0.0335"));
        assert!(!short.is_flat());
    }

    /// A synthetic zero-size row verifies the common flat-position filter.
    #[test]
    fn a_zero_size_row_maps_to_a_flat_position_the_common_api_drops() {
        let mut raw: RawPerpState =
            json(CLEARINGHOUSE_STATE).expect("official clearinghouseState payload");

        raw.asset_positions[0].position.szi = "0.0".to_string();
        let flat = position(&raw.asset_positions[0].position, &universe()).expect("a position");

        assert!(flat.is_flat());
        // The mapped placeholder side does not escape the common filter.
        assert_eq!(flat.side, None);

        assert_eq!(
            crate::client::open_positions(vec![flat]),
            Vec::new(),
            "a zero-size row was answered as an open position"
        );
    }

    #[test]
    fn margin_summary_reports_withdrawable_as_what_can_back_a_new_position() {
        let raw: RawPerpState =
            json(CLEARINGHOUSE_STATE).expect("official clearinghouseState payload");
        let summary = margin_summary(&raw).expect("a summary");

        assert_eq!(summary.asset, "USDC");
        assert_eq!(summary.equity, Some(decimal_of("13104.514502")));
        assert_eq!(summary.margin_balance, Some(decimal_of("4.967826")));
        assert_eq!(summary.available_balance, Some(decimal_of("13104.514502")));
    }

    #[test]
    fn an_open_order_reports_what_is_left_and_what_has_filled() {
        let mut raw: Vec<RawOpenOrder> =
            json(FRONTEND_OPEN_ORDERS).expect("official frontendOpenOrders payload");
        raw[0].sz = "0.4".to_string();
        raw[0].orig_sz = "1.0".to_string();

        let order = open_order(&raw[0], &universe()).expect("an order");

        assert_eq!(order.id, "91490942");
        assert_eq!(order.side, Side::Sell);
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert_eq!(order.filled_quantity, decimal_of("0.6"));
        assert_eq!(order.remaining_quantity, decimal_of("0.4"));
        assert_eq!(order.price, Some(decimal_of("29792.0")));
        assert_eq!(
            order.created_at,
            Some(Timestamp::from_millis(1_681_247_412_573))
        );
    }

    #[test]
    fn every_way_hyperliquid_spells_a_cancellation_reads_as_cancelled() {
        for status in [
            "canceled",
            "marginCanceled",
            "reduceOnlyCanceled",
            "selfTradeCanceled",
            "liquidatedCanceled",
        ] {
            assert_eq!(
                order_status(status, Decimal::ZERO),
                OrderStatus::Cancelled,
                "{status}"
            );
        }
        assert_eq!(order_status("open", Decimal::ZERO), OrderStatus::Open);
        assert_eq!(
            order_status("open", Decimal::ONE),
            OrderStatus::PartiallyFilled
        );
        assert_eq!(order_status("filled", Decimal::ONE), OrderStatus::Filled);
        assert_eq!(
            order_status("badAloPx", Decimal::ZERO),
            OrderStatus::Rejected
        );
        assert_eq!(
            order_status("somethingNew", Decimal::ZERO),
            OrderStatus::Unknown
        );
    }

    #[test]
    fn an_error_body_arriving_with_http_200_is_still_an_error() {
        // Hyperliquid answers a rejected action with a 200 and `status: "err"`.
        // A status-code check would read this as a success.
        let error = action_response(r#"{"status":"err","response":"Insufficient margin."}"#)
            .expect_err("a rejection");

        assert!(matches!(
            &error,
            Error::Exchange { exchange: "hyperliquid", code, message, status: None, .. }
                if code == "err" && message == "Insufficient margin."
        ));
    }

    #[test]
    fn a_rejected_order_inside_an_accepted_envelope_is_still_an_error() {
        // The envelope says `ok`; the order inside it was refused.
        let response = action_response(
            r#"{
              "status": "ok",
              "response": {
                "type": "order",
                "data": {"statuses": [{"error": "Order must have minimum value of $10."}]}
              }
            }"#,
        )
        .expect("an accepted envelope");
        let error = order_ack_id(&response).expect_err("a refused order");

        assert!(matches!(
            &error,
            Error::Exchange { code, message, .. }
                if code == "order_rejected" && message == "Order must have minimum value of $10."
        ));
    }

    #[test]
    fn an_accepted_order_reports_whether_it_rested_or_filled() {
        let resting = action_response(
            r#"{"status":"ok","response":{"type":"order","data":{"statuses":[{"resting":{"oid":77}}]}}}"#,
        )
        .expect("an accepted envelope");
        let filled = action_response(
            r#"{"status":"ok","response":{"type":"order","data":{"statuses":[{"filled":{"oid":88,"totalSz":"0.02","avgPx":"1891.4"}}]}}}"#,
        )
        .expect("an accepted envelope");

        assert_eq!(
            order_ack_id(&resting).expect("an ack"),
            ("77".to_string(), OrderStatus::Open)
        );
        assert_eq!(
            order_ack_id(&filled).expect("an ack"),
            ("88".to_string(), OrderStatus::Filled)
        );
    }

    #[test]
    fn a_cursor_round_trips_without_the_caller_reading_it() {
        let raw: Vec<RawFundingHistory> =
            json(FUNDING_HISTORY).expect("official fundingHistory payload");
        let newest = raw.last().expect("two entries").time;
        let cursor = time_cursor(newest).expect("a cursor");

        // The next page starts just past the newest entry, so nothing repeats.
        assert_eq!(cursor_start_ms(&cursor).expect("a start time"), newest + 1);
        assert_eq!(
            cursor_start_ms(&Cursor("not-a-cursor".to_string())),
            Err(Error::invalid_request(
                "cursor",
                "pass back the cursor `maxt` returned, unchanged"
            ))
        );
    }

    #[test]
    fn a_funding_payment_keeps_the_sign_that_says_who_paid() {
        let raw: Vec<RawUserFunding> = json(USER_FUNDING).expect("official userFunding payload");

        assert_eq!(
            decimal(&raw[0].delta.usdc, "usdc").expect("a decimal"),
            decimal_of("-0.0568")
        );
        assert_eq!(
            decimal(&raw[0].delta.funding_rate, "fundingRate").expect("a decimal"),
            decimal_of("0.0000125")
        );
        assert_eq!(raw[0].delta.coin, "ETH");
    }
}
