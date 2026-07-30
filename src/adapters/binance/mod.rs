//! Binance, global spot and USD-margined perpetual futures.

mod parse;
mod private;
mod rest;
mod stream;

use std::sync::OnceLock;

use crate::adapter::{Adapter, BoxFuture};
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{CandleRequest, HistoryRequest, MarginRequest, OrderRequest};
use crate::stream::{AccountStream, MarketStream};
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{
    Balance, Candle, Cursor, Exchange, FundingPayment, FundingRate, Interval, MarginSummary,
    Market, MarketInfo, MarketKind, MarketStatus, Order, OrderBook, Page, Position, StreamConfig,
    Subscription, Ticker, Timestamp, Trade,
};

pub use private::{BinanceListenKey, BinanceSpotOrderDetail};
pub use rest::BinanceSymbolFilters;

pub(crate) const SPOT_REST_BASE_URL: &str = "https://api.binance.com";
pub(crate) const SPOT_WEBSOCKET_URL: &str = "wss://stream.binance.com:9443/stream";
/// Where a spot user data stream is subscribed to.
///
/// Not the market data host. Binance removed the spot listen key endpoints on
/// 2026-02-20 07:00 UTC, and an account subscription is now a signed request on
/// the WebSocket API, which answers on its own host. See
/// `private::spot_user_data_subscribe_frame` for the request.
pub(crate) const SPOT_WEBSOCKET_API_URL: &str = "wss://ws-api.binance.com:443/ws-api/v3";
pub(crate) const USD_M_REST_BASE_URL: &str = "https://fapi.binance.com";
/// The USD-M entry point carrying the streams the matching engine pushes.
///
/// Binance splits USD-M market data across two entry points on one host and
/// decommissioned the unrouted `/stream` and `/ws` paths on 2026-04-23. A
/// connection that names no entry point is served as if it had named
/// `/public`. The stream module decides which feed goes where.
pub(crate) const USD_M_PUBLIC_WEBSOCKET_URL: &str = "wss://fstream.binance.com/public/stream";
/// The USD-M entry point carrying the streams an aggregator produces.
pub(crate) const USD_M_MARKET_WEBSOCKET_URL: &str = "wss://fstream.binance.com/market/stream";

/// The header every authenticated Binance request carries its API key in.
pub(crate) const API_KEY_HEADER: &str = "X-MBX-APIKEY";

/// The name `maxt` reports Binance under in errors and market identities.
pub(crate) const EXCHANGE: &str = Exchange::Binance.id();

/// Which Binance venue an adapter talks to.
///
/// Spot and USD-M futures are separate APIs with separate hosts, separate
/// balances, and separate listings. One adapter talks to one of them, chosen at
/// construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum BinanceMarket {
    /// Binance Spot. The default.
    #[default]
    Spot,
    /// Binance USD-margined perpetual futures.
    UsdMFutures,
}

impl BinanceMarket {
    /// The kind of instrument this venue lists.
    ///
    /// Every market handed to the adapter is checked against it, so a spot
    /// market never reaches the futures host by accident. Binance lists
    /// `BTCUSDT` on both, at different prices.
    pub(crate) const fn market_kind(self) -> MarketKind {
        match self {
            Self::Spot => MarketKind::Spot,
            Self::UsdMFutures => MarketKind::Perpetual,
        }
    }

    pub(crate) const fn rest_base_url(self) -> &'static str {
        match self {
            Self::Spot => SPOT_REST_BASE_URL,
            Self::UsdMFutures => USD_M_REST_BASE_URL,
        }
    }

    /// Binance's own name for a candle interval on this venue.
    ///
    /// One-second candles exist on spot only; USD-M futures starts at one
    /// minute, which is the one interval the two venues disagree about.
    pub(crate) fn interval_code(self, interval: Interval) -> Result<&'static str> {
        Ok(match (self, interval) {
            (Self::UsdMFutures, Interval::Sec1) => {
                return Err(Error::unsupported(
                    Feature::Candles,
                    EXCHANGE,
                    "USD-M futures publishes no one-second candles; one minute is the shortest",
                ));
            }
            (_, Interval::Sec1) => "1s",
            (_, Interval::Min1) => "1m",
            (_, Interval::Min3) => "3m",
            (_, Interval::Min5) => "5m",
            (_, Interval::Min15) => "15m",
            (_, Interval::Min30) => "30m",
            (_, Interval::Hour1) => "1h",
            (_, Interval::Hour2) => "2h",
            (_, Interval::Hour4) => "4h",
            (_, Interval::Hour8) => "8h",
            (_, Interval::Hour12) => "12h",
            (_, Interval::Day1) => "1d",
            (_, Interval::Day3) => "3d",
            (_, Interval::Week1) => "1w",
            (_, Interval::Month1) => "1M",
        })
    }
}

/// Talks to Binance.
///
/// Pick the venue with [`BinanceAdapter::spot`] or
/// [`BinanceAdapter::usd_m_futures`]. USD-M carries [`Feature::Positions`],
/// [`Feature::Margin`] and [`Feature::FundingRates`]; the spot venue answers
/// `Error::Unsupported` for all three. Binance does sell a cross and isolated
/// margin product on spot, but it is a separate set of endpoints that `maxt`
/// does not reach, so [`Feature::Margin`] here means the USD-M contract
/// account and nothing else.
///
/// ```
/// use maxt::{Client, Feature, adapters::BinanceAdapter};
///
/// let spot = Client::new(BinanceAdapter::spot());
/// let perp = Client::new(BinanceAdapter::usd_m_futures());
///
/// assert!(!spot.supports(Feature::FundingRates));
/// assert!(perp.supports(Feature::FundingRates));
/// ```
#[derive(Debug, Clone)]
pub struct BinanceAdapter {
    venue: BinanceMarket,
    credentials: Option<BinanceCredentials>,
    /// Built on first use so the constructors stay infallible, and shared from
    /// then on so connections are reused across calls.
    http: OnceLock<HttpTransport>,
}

#[derive(Debug, Clone)]
pub(crate) struct BinanceCredentials {
    pub(crate) api_key: String,
    pub(crate) secret_key: String,
}

impl BinanceAdapter {
    /// An adapter for public Binance Spot market data.
    pub fn spot() -> Self {
        Self::for_venue(BinanceMarket::Spot)
    }

    /// An adapter for public Binance USD-M futures market data.
    pub fn usd_m_futures() -> Self {
        Self::for_venue(BinanceMarket::UsdMFutures)
    }

    fn for_venue(venue: BinanceMarket) -> Self {
        Self {
            venue,
            credentials: None,
            http: OnceLock::new(),
        }
    }

    /// Adds the API credentials that account, order, and private stream calls
    /// need.
    ///
    /// Binance issues an API key and a secret key together. A key restricted to
    /// spot will be rejected by the futures API and the other way round.
    #[must_use]
    pub fn with_credentials(
        mut self,
        api_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        self.credentials = Some(BinanceCredentials {
            api_key: api_key.into(),
            secret_key: secret_key.into(),
        });
        self
    }

    /// Which venue this adapter talks to.
    pub fn venue(&self) -> BinanceMarket {
        self.venue
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }

    pub(crate) fn credentials(&self) -> Result<&BinanceCredentials> {
        self.credentials
            .as_ref()
            .ok_or_else(|| Error::auth("binance needs both an API key and a secret key"))
    }

    /// The transport for this venue's REST host.
    pub(crate) fn http(&self) -> Result<&HttpTransport> {
        if let Some(transport) = self.http.get() {
            return Ok(transport);
        }
        let transport = HttpTransport::new(self.venue.rest_base_url())?;
        Ok(self.http.get_or_init(|| transport))
    }

    /// Sends a request and returns the body, or Binance's own verdict.
    pub(crate) async fn send(&self, request: HttpRequest) -> Result<String> {
        let response = self.http()?.send(&request).await?;
        if response.is_success() {
            return Ok(response.body);
        }
        Err(exchange_error(response.status, &response.body))
    }

    /// The Binance symbol for a market, after checking it belongs here.
    ///
    /// `BTCUSDT` alone does not say which venue it came from, so the market's
    /// kind is checked against the adapter's venue before the symbol is built.
    pub(crate) fn symbol(&self, market: &Market) -> Result<String> {
        if market.exchange != Exchange::Binance {
            return Err(Error::invalid_request(
                "market",
                format!("{market} is not a Binance market"),
            ));
        }
        if market.kind != self.venue.market_kind() {
            return Err(Error::invalid_request(
                "market",
                format!(
                    "this adapter trades {:?} markets; {market} is {:?}",
                    self.venue.market_kind(),
                    market.kind
                ),
            ));
        }
        check_asset("base", &market.base)?;
        check_asset("quote", &market.quote)?;
        Ok(format!("{}{}", market.base, market.quote))
    }

    /// The market a Binance symbol names, on this adapter's venue.
    ///
    /// Used wherever a payload identifies its market by symbol alone, which
    /// every stream frame does. See [`split_symbol`] for how the split is
    /// decided.
    pub(crate) fn market(&self, symbol: &str) -> Result<Market> {
        let (base, quote) = split_symbol(symbol).ok_or_else(|| {
            Error::decode(format!(
                "`{symbol}` does not end in an asset Binance quotes in"
            ))
        })?;
        Ok(Market::new(
            Exchange::Binance,
            self.venue.market_kind(),
            base,
            quote,
        ))
    }

    /// Reads the trading rules Binance attaches to one spot symbol.
    ///
    /// Tick size, lot step, and minimum notional decide whether an order is
    /// accepted at all. Every exchange expresses them differently enough that
    /// [`BinanceSymbolFilters`] stays Binance-shaped. See that type for why.
    ///
    /// Reports [`Error::Unsupported`] on a USD-M adapter, whose listing carries
    /// a different set of filters.
    pub async fn spot_symbol_filters(&self, market: &Market) -> Result<BinanceSymbolFilters> {
        rest::spot_symbol_filters(self, market).await
    }

    /// Looks one spot order up by the identifier Binance issued for it.
    ///
    /// Answers for filled and cancelled orders as well as resting ones, which
    /// [`Client::open_orders`](crate::Client::open_orders) by definition does
    /// not. See [`BinanceSpotOrderDetail`] for why the answer is Binance-shaped.
    ///
    /// Reports [`Error::Unsupported`] on a USD-M adapter.
    pub async fn spot_order(
        &self,
        market: &Market,
        order_id: &str,
    ) -> Result<BinanceSpotOrderDetail> {
        private::spot_order(self, market, order_id).await
    }

    /// Opens a USD-M user data stream and returns its listen key.
    ///
    /// [`Client::subscribe_account`](crate::Client::subscribe_account) does
    /// this for you and keeps the key alive. Reach for these three methods
    /// only when driving the socket yourself, such as sharing one key across
    /// two consumers or holding it across a process restart.
    ///
    /// Binance returns the account's existing key when it already has one, and
    /// extends it either way.
    pub async fn usd_m_create_listen_key(&self) -> Result<BinanceListenKey> {
        self.check_usd_m("listen keys")?;
        private::create_listen_key(self).await
    }

    /// Pushes a USD-M listen key's expiry another sixty minutes out.
    ///
    /// `key` names the stream being kept alive and is not sent: USD-M extends
    /// whichever key the API key currently owns and refuses a `listenKey`
    /// parameter. A key that has already lapsed is therefore not detectable
    /// here, and what comes back is Binance's own verdict on the extension
    /// under [`Error::Exchange`], not an [`Error::Auth`] of this crate's
    /// invention.
    pub async fn usd_m_keepalive_listen_key(&self, key: &BinanceListenKey) -> Result<()> {
        self.check_usd_m("listen keys")?;
        private::keepalive_listen_key(self, key).await
    }

    /// Closes a USD-M user data stream.
    ///
    /// The socket stays open for a short while afterwards but stops carrying
    /// events, so drop the stream as well.
    pub async fn usd_m_close_listen_key(&self, key: &BinanceListenKey) -> Result<()> {
        self.check_usd_m("listen keys")?;
        private::close_listen_key(self, key).await
    }

    fn check_usd_m(&self, what: &str) -> Result<()> {
        if self.venue == BinanceMarket::UsdMFutures {
            return Ok(());
        }
        Err(Error::unsupported(
            Feature::AccountStream,
            EXCHANGE,
            format!("these {what} are the USD-M ones; build the adapter with `usd_m_futures`"),
        ))
    }
}

impl Default for BinanceAdapter {
    fn default() -> Self {
        Self::spot()
    }
}

/// Rejects an asset code that would change meaning inside a query string.
///
/// Binance's own codes are uppercase ASCII letters and digits. Anything else
/// would let a `&` in a market name append a parameter, and would break the
/// signature that private calls hash the query string into.
fn check_asset(field: &'static str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::invalid_request(field, "must not be empty"));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    {
        return Err(Error::invalid_request(
            field,
            format!("`{value}` is not a Binance asset code: expected uppercase ASCII and digits"),
        ));
    }
    Ok(())
}

/// The assets Binance prices markets in.
///
/// Membership is what matters, not order: [`split_symbol`] takes the longest
/// matching suffix, and two different codes of the same length cannot both end
/// the same symbol.
const QUOTE_ASSETS: &[&str] = &[
    "FDUSD", "USDT", "USDC", "USD1", "BUSD", "TUSD", "USDP", "AEUR", "BIDR", "IDRT", "DOGE", "BTC",
    "ETH", "BNB", "XRP", "SOL", "TRX", "DAI", "TRY", "EUR", "GBP", "BRL", "ARS", "JPY", "MXN",
    "ZAR", "COP", "CZK", "PLN", "RON", "UAH", "NGN", "RUB", "AUD", "VAI", "PAX",
];

/// Splits a Binance symbol into base and quote.
///
/// Binance concatenates the two assets with no separator, so `BTCUSDT` read as
/// text alone is genuinely ambiguous. The split is resolved against the set of
/// assets Binance actually quotes in, taking the longest matching suffix, which
/// is what makes `ETHBTC`, `BTCUSDC`, and `USDCUSDT` come out right.
///
/// `None` when no known quote asset ends the symbol. Listing markets never
/// needs this: `exchangeInfo` publishes `baseAsset` and `quoteAsset`
/// separately, and that listing is the authority this table approximates.
fn split_symbol(symbol: &str) -> Option<(&str, &str)> {
    QUOTE_ASSETS
        .iter()
        .filter_map(|quote| {
            let base = symbol.strip_suffix(*quote)?;
            (!base.is_empty()).then_some((base, *quote))
        })
        .max_by_key(|(_, quote)| quote.len())
}

/// Turns a non-2xx response into Binance's own verdict.
///
/// Binance answers every failure with `{"code": -1121, "msg": "Invalid
/// symbol."}`, rate limits included. The numeric code is the part worth
/// branching on and is kept verbatim. The HTTP status carries the retry
/// classification, which lands 429 and Binance's 418 IP ban on
/// [`ExchangeErrorKind::RateLimited`](crate::ExchangeErrorKind::RateLimited).
pub(crate) fn exchange_error(status: u16, body: &str) -> Error {
    #[derive(serde::Deserialize)]
    struct RawError {
        code: i64,
        msg: String,
    }

    match serde_json::from_str::<RawError>(body) {
        Ok(raw) => Error::exchange_http(EXCHANGE, status, raw.code.to_string(), raw.msg),
        // A body that is not the error envelope is still a failure; report the
        // status and whatever arrived rather than inventing a code.
        Err(_) => Error::exchange_http(EXCHANGE, status, "unknown", body.trim()),
    }
}

/// Reads a market status out of an `exchangeInfo` listing.
pub(crate) fn market_status(raw: &str) -> MarketStatus {
    match raw {
        "TRADING" => MarketStatus::Active,
        // Halted but still listed: orders are rejected and the pair comes back.
        "BREAK" | "HALT" | "PENDING_TRADING" | "PRE_TRADING" | "POST_TRADING" | "AUCTION_MATCH" => {
            MarketStatus::Paused
        }
        "DELISTED" | "CLOSE" | "SETTLING" => MarketStatus::Delisted,
        _ => MarketStatus::Unknown,
    }
}

/// The current time in Binance's unit, for signing and for candle closing.
pub(crate) fn now_millis() -> i64 {
    Timestamp::now().as_millis()
}

/// Encodes one position in a paginated history.
///
/// Both of Binance's paginated histories are windowed by `startTime`, so a
/// cursor is one millisecond timestamp. It is tagged so a cursor from another
/// exchange, or from a later change of scheme, is refused instead of misread.
pub(crate) fn encode_cursor(resume_from_millis: i64) -> Cursor {
    Cursor(format!("t{resume_from_millis}"))
}

/// Reads back a cursor from [`encode_cursor`].
pub(crate) fn decode_cursor(cursor: &Cursor) -> Result<i64> {
    cursor
        .as_str()
        .strip_prefix('t')
        .and_then(|millis| millis.parse().ok())
        .ok_or_else(|| Error::invalid_request("cursor", "not a cursor this exchange produced"))
}

impl Adapter for BinanceAdapter {
    fn exchange(&self) -> Exchange {
        Exchange::Binance
    }

    fn supports(&self, feature: Feature) -> bool {
        if feature.is_derivatives_only() && self.venue == BinanceMarket::Spot {
            return false;
        }
        if feature.needs_credentials() {
            return self.is_authenticated();
        }
        true
    }

    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
        Box::pin(async move { rest::markets(self, kind).await })
    }

    fn trades(&self, market: &Market, limit: Option<u32>) -> BoxFuture<'_, Result<Vec<Trade>>> {
        let market = market.clone();
        Box::pin(async move { rest::trades(self, &market, limit).await })
    }

    fn order_book(&self, market: &Market, depth: Option<u32>) -> BoxFuture<'_, Result<OrderBook>> {
        let market = market.clone();
        Box::pin(async move { rest::order_book(self, &market, depth).await })
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        let market = market.clone();
        Box::pin(async move { rest::ticker(self, &market).await })
    }

    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, Result<Vec<Candle>>> {
        let request = request.clone();
        Box::pin(async move { rest::candles(self, &request).await })
    }

    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, Result<MarketStream>> {
        let subscription = subscription.clone();
        let config = config.clone();
        Box::pin(async move { stream::subscribe(self, &subscription, &config).await })
    }

    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        let config = config.clone();
        Box::pin(async move { stream::subscribe_account(self, &config).await })
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        Box::pin(async move { private::balances(self).await })
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        let market = market.cloned();
        Box::pin(async move { private::open_orders(self, market.as_ref()).await })
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        let request = request.clone();
        Box::pin(async move { private::place_order(self, &request).await })
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<Order>> {
        let market = market.clone();
        let order_id = order_id.to_string();
        Box::pin(async move { private::cancel_order(self, &market, &order_id).await })
    }

    fn positions(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Position>>> {
        let market = market.cloned();
        Box::pin(async move { private::positions(self, market.as_ref()).await })
    }

    fn margin_summary(&self) -> BoxFuture<'_, Result<MarginSummary>> {
        Box::pin(async move { private::margin_summary(self).await })
    }

    fn funding_rates(&self, request: &HistoryRequest) -> BoxFuture<'_, Result<Page<FundingRate>>> {
        let request = request.clone();
        Box::pin(async move { private::funding_rates(self, &request).await })
    }

    fn funding_payments(
        &self,
        request: &HistoryRequest,
    ) -> BoxFuture<'_, Result<Page<FundingPayment>>> {
        let request = request.clone();
        Box::pin(async move { private::funding_payments(self, &request).await })
    }

    fn set_margin(&self, request: &MarginRequest) -> BoxFuture<'_, Result<()>> {
        let request = request.clone();
        Box::pin(async move { private::set_margin(self, &request).await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ExchangeErrorKind;

    #[test]
    fn spot_has_no_derivatives_features_and_futures_has_all_of_them() {
        let spot = BinanceAdapter::spot().with_credentials("key", "secret");
        let futures = BinanceAdapter::usd_m_futures().with_credentials("key", "secret");

        for feature in [
            Feature::Positions,
            Feature::Margin,
            Feature::FundingRates,
            Feature::FundingPayments,
            Feature::MarginConfig,
            Feature::ReduceOnlyOrders,
        ] {
            assert!(!spot.supports(feature), "spot {feature:?}");
            assert!(futures.supports(feature), "futures {feature:?}");
        }
    }

    #[test]
    fn both_venues_serve_public_market_data_without_credentials() {
        for adapter in [BinanceAdapter::spot(), BinanceAdapter::usd_m_futures()] {
            for feature in [
                Feature::Markets,
                Feature::Trades,
                Feature::OrderBook,
                Feature::Ticker,
                Feature::Candles,
                Feature::CandleStream,
            ] {
                assert!(
                    adapter.supports(feature),
                    "{:?} {feature:?}",
                    adapter.venue()
                );
            }
        }
    }

    #[test]
    fn both_venues_read_balances_and_open_orders_once_credentials_are_set() {
        // Spot answers these on `/api/v3/account` and `/api/v3/openOrders`,
        // USD-M on `/fapi/v3/account` and `/fapi/v1/openOrders`. Both venues
        // answer both questions, so `supports` claims both on both.
        for adapter in [
            BinanceAdapter::spot().with_credentials("key", "secret"),
            BinanceAdapter::usd_m_futures().with_credentials("key", "secret"),
        ] {
            assert!(adapter.supports(Feature::Balances), "{:?}", adapter.venue());
            assert!(
                adapter.supports(Feature::OpenOrders),
                "{:?}",
                adapter.venue()
            );
        }
    }

    #[test]
    fn funding_rates_are_public_but_funding_payments_are_not() {
        let public = BinanceAdapter::usd_m_futures();

        assert!(public.supports(Feature::FundingRates));
        assert!(!public.supports(Feature::FundingPayments));
    }

    #[test]
    fn the_two_venues_are_separate_hosts() {
        assert_ne!(SPOT_REST_BASE_URL, USD_M_REST_BASE_URL);
        assert_ne!(SPOT_WEBSOCKET_URL, USD_M_PUBLIC_WEBSOCKET_URL);
        assert_ne!(SPOT_WEBSOCKET_URL, USD_M_MARKET_WEBSOCKET_URL);
        assert_eq!(BinanceAdapter::default().venue(), BinanceMarket::Spot);
    }

    #[test]
    fn a_symbol_splits_on_the_longest_quote_asset_that_ends_it() {
        assert_eq!(split_symbol("BTCUSDT"), Some(("BTC", "USDT")));
        assert_eq!(split_symbol("ETHBTC"), Some(("ETH", "BTC")));
        assert_eq!(split_symbol("BTCUSDC"), Some(("BTC", "USDC")));
        // The one a shortest-match or fixed-offset split gets wrong.
        assert_eq!(split_symbol("USDCUSDT"), Some(("USDC", "USDT")));
        assert_eq!(split_symbol("BTCFDUSD"), Some(("BTC", "FDUSD")));
        assert_eq!(split_symbol("USDTTRY"), Some(("USDT", "TRY")));
        assert_eq!(split_symbol("BNBETH"), Some(("BNB", "ETH")));
        assert_eq!(split_symbol("BTCTUSD"), Some(("BTC", "TUSD")));
    }

    #[test]
    fn a_symbol_with_no_known_quote_asset_is_refused_rather_than_guessed() {
        assert_eq!(split_symbol("BTCXYZ"), None);
        // A quote asset on its own has no base.
        assert_eq!(split_symbol("USDT"), None);
        assert_eq!(split_symbol(""), None);
    }

    #[test]
    fn a_symbol_round_trips_through_the_venues_market_kind() {
        let spot = BinanceAdapter::spot();
        let perp = BinanceAdapter::usd_m_futures();

        let spot_market = Market::spot(Exchange::Binance, "BTC", "USDT");
        let perp_market = Market::perpetual(Exchange::Binance, "BTC", "USDT");

        assert_eq!(spot.symbol(&spot_market).expect("a spot market"), "BTCUSDT");
        assert_eq!(perp.symbol(&perp_market).expect("a perp market"), "BTCUSDT");
        assert_eq!(
            spot.market("BTCUSDT").expect("a listed symbol"),
            spot_market
        );
        assert_eq!(
            perp.market("BTCUSDT").expect("a listed symbol"),
            perp_market
        );
    }

    #[test]
    fn a_market_from_the_wrong_venue_or_exchange_never_reaches_the_network() {
        let spot = BinanceAdapter::spot();

        assert!(matches!(
            spot.symbol(&Market::perpetual(Exchange::Binance, "BTC", "USDT")),
            Err(Error::InvalidRequest {
                field: "market",
                ..
            })
        ));
        assert!(matches!(
            spot.symbol(&Market::spot(Exchange::Upbit, "BTC", "KRW")),
            Err(Error::InvalidRequest {
                field: "market",
                ..
            })
        ));
    }

    #[test]
    fn a_symbol_that_could_smuggle_a_query_parameter_is_rejected() {
        let injected = Market::spot(Exchange::Binance, "BTC&limit=5000", "USDT");

        assert!(matches!(
            BinanceAdapter::spot().symbol(&injected),
            Err(Error::InvalidRequest { field: "base", .. })
        ));
    }

    #[test]
    fn one_second_candles_are_spot_only() {
        assert_eq!(
            BinanceMarket::Spot
                .interval_code(Interval::Sec1)
                .expect("spot serves one-second candles"),
            "1s"
        );
        assert_eq!(
            BinanceMarket::UsdMFutures
                .interval_code(Interval::Min1)
                .expect("one minute is the futures floor"),
            "1m"
        );
        assert!(matches!(
            BinanceMarket::UsdMFutures.interval_code(Interval::Sec1),
            Err(Error::Unsupported {
                feature: Feature::Candles,
                ..
            })
        ));
    }

    #[test]
    fn month_intervals_keep_binances_capital_m() {
        // `1m` and `1M` are a minute and a month; lowercasing the code silently
        // asks for the wrong candles.
        assert_eq!(
            BinanceMarket::Spot
                .interval_code(Interval::Month1)
                .expect("a served interval"),
            "1M"
        );
        assert_eq!(
            BinanceMarket::Spot
                .interval_code(Interval::Min1)
                .expect("a served interval"),
            "1m"
        );
    }

    #[test]
    fn an_error_body_keeps_binances_numeric_code() {
        // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/general-api-information
        let error = exchange_error(400, r#"{"code":-1121,"msg":"Invalid symbol."}"#);

        let Error::Exchange {
            code,
            message,
            status,
            kind,
            ..
        } = &error
        else {
            panic!("expected an exchange error");
        };
        assert_eq!(code, "-1121");
        assert_eq!(message, "Invalid symbol.");
        assert_eq!(*status, Some(400));
        assert_eq!(*kind, ExchangeErrorKind::Rejected);
        assert!(!error.is_retryable());
    }

    /// A credential Binance read and refused keeps Binance's own code instead
    /// of becoming [`Error::Auth`].
    ///
    /// `Auth` says `maxt` sent nothing. These three were sent, answered, and
    /// refused, and they are three different problems: the secret is wrong, the
    /// key is wrong or unpermitted, or no key was presented. Folding them into
    /// one variant would drop the only field that tells them apart, and the
    /// rule doing the folding would have to be right about Upbit, Bithumb and
    /// Hyperliquid too, which spell a refused credential in three further ways.
    ///
    /// Bodies and statuses captured verbatim on 2026-07-31 from
    /// `GET https://api.binance.com/api/v3/account`, signed with a live key and
    /// broken one field at a time.
    #[test]
    fn a_credential_binance_refused_keeps_binances_own_code() {
        let cases = [
            (
                400,
                r#"{"code":-1022,"msg":"Signature for this request is not valid."}"#,
                "-1022",
            ),
            (
                401,
                r#"{"code":-2015,"msg":"Invalid API-key, IP, or permissions for action."}"#,
                "-2015",
            ),
            (
                401,
                r#"{"code":-2014,"msg":"API-key format invalid."}"#,
                "-2014",
            ),
        ];

        for (status, body, expected) in cases {
            let error = exchange_error(status, body);

            let Error::Exchange { code, .. } = &error else {
                panic!("expected the exchange's own verdict for {expected}, got {error:?}");
            };
            assert_eq!(code, expected, "{error:?}");
            // Retrying the identical signed request cannot make a wrong secret
            // right, so the classification the status gives is the right one.
            assert!(!error.is_retryable(), "{error:?}");
        }
    }

    /// A clock outside Binance's receive window is a rejection, not an outage.
    ///
    /// Both bodies captured verbatim on 2026-07-31 from
    /// `GET https://api.binance.com/api/v3/account`, by signing a timestamp ten
    /// minutes behind and then ten seconds ahead of Binance's own clock. Both
    /// arrived as HTTP 400, which is what puts them here rather than under
    /// `Unavailable`: the identical request carries the identical timestamp and
    /// can only be further outside the window the second time.
    #[test]
    fn a_timestamp_outside_the_receive_window_is_rejected_rather_than_retried() {
        for body in [
            r#"{"code":-1021,"msg":"Timestamp for this request is outside of the recvWindow."}"#,
            r#"{"code":-1021,"msg":"Timestamp for this request was 1000ms ahead of the server's time."}"#,
        ] {
            let error = exchange_error(400, body);

            let Error::Exchange { code, kind, .. } = &error else {
                panic!("expected an exchange error, got {error:?}");
            };
            assert_eq!(code, "-1021", "{error:?}");
            assert_eq!(*kind, ExchangeErrorKind::Rejected, "{error:?}");
            assert!(!error.is_retryable(), "{error:?}");
            assert!(!error.is_rate_limited(), "{error:?}");
        }
    }

    #[test]
    fn rate_limits_and_ip_bans_classify_as_rate_limited() {
        // https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits
        let too_many = exchange_error(429, r#"{"code":-1003,"msg":"Too many requests."}"#);
        // 418 is what Binance answers with once a 429 has been ignored.
        let banned = exchange_error(
            418,
            r#"{"code":-1003,"msg":"Way too many requests; IP banned until 1499865549590."}"#,
        );

        assert!(too_many.is_rate_limited());
        assert!(banned.is_rate_limited());
        assert!(banned.is_retryable());
    }

    #[test]
    fn an_error_body_that_is_not_json_still_reports_the_status() {
        let error = exchange_error(502, "<html>Bad Gateway</html>");

        let Error::Exchange {
            kind,
            status,
            message,
            ..
        } = error
        else {
            panic!("expected an exchange error");
        };
        assert_eq!(status, Some(502));
        assert_eq!(kind, ExchangeErrorKind::Unavailable);
        assert_eq!(message, "<html>Bad Gateway</html>");
    }

    #[test]
    fn a_cursor_round_trips_without_the_caller_reading_it() {
        let cursor = encode_cursor(1_499_865_549_590);

        assert_eq!(
            decode_cursor(&cursor).expect("its own cursor reads back"),
            1_499_865_549_590
        );
        assert!(decode_cursor(&Cursor("page-2".to_string())).is_err());
        assert!(decode_cursor(&Cursor("t".to_string())).is_err());
    }

    #[test]
    fn listing_statuses_separate_a_halt_from_a_delisting() {
        assert_eq!(market_status("TRADING"), MarketStatus::Active);
        assert_eq!(market_status("BREAK"), MarketStatus::Paused);
        assert_eq!(market_status("SETTLING"), MarketStatus::Delisted);
        assert_eq!(market_status("SOMETHING_NEW"), MarketStatus::Unknown);
    }

    #[test]
    fn private_calls_refuse_to_run_without_credentials() {
        assert!(matches!(
            BinanceAdapter::spot().credentials(),
            Err(Error::Auth { .. })
        ));
        assert!(
            BinanceAdapter::spot()
                .with_credentials("key", "secret")
                .credentials()
                .is_ok()
        );
    }
}
