//! Bithumb, a Korean spot exchange.

mod parse;
mod private;
mod rest;
mod stream;

use crate::adapter::{Adapter, BoxFuture};
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{CandleRequest, OrderRequest};
use crate::stream::{AccountStream, MarketStream};
use crate::transport::HttpTransport;
use crate::types::{
    Balance, Candle, Exchange, Market, MarketInfo, MarketKind, Order, OrderBook, StreamConfig,
    Subscription, Ticker, Timestamp, Trade,
};

pub(crate) const REST_BASE_URL: &str = "https://api.bithumb.com";
pub(crate) const WEBSOCKET_URL: &str = "wss://ws-api.bithumb.com/websocket/v1";
/// Private frames are served from the v2 socket even though the public feeds
/// are v1; the two version numbers are unrelated.
pub(crate) const PRIVATE_WEBSOCKET_URL: &str = "wss://ws-api.bithumb.com/websocket/v2/private";

/// How serious Bithumb calls one alert, ranked the way Bithumb ranks them.
///
/// Ordering follows severity, so `step >= BithumbAlertStep::Warning` is the
/// filter for "past the mildest".
///
/// `Caution` here is Bithumb's 주의, the gentlest of three steps. It is not the
/// `CAUTION` that [`BithumbAdapter::market_warnings`] returns: that spelling
/// belongs to the other designation entirely and means 유의.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum BithumbAlertStep {
    /// 주의. The step Bithumb raises first, and the one that corresponds to
    /// Upbit having any 주의 종목 criterion raised at all.
    Caution,
    /// 경고. The middle step, rare enough that a day's list can hold one.
    Warning,
    /// 위험. The gravest step Bithumb documents, and the commonest one in
    /// practice, because the volume and deposit criteria tend to arrive here.
    Danger,
    /// A step Bithumb has begun sending since, ranked above [`Danger`] so a
    /// severity threshold surfaces it instead of quietly passing it.
    ///
    /// [`Danger`]: BithumbAlertStep::Danger
    Unknown,
}

/// One alert Bithumb's 경보제 has raised on one market.
///
/// This is the counterpart of Upbit's 주의 종목: raised and cleared
/// automatically against published criteria, describing how a market is
/// trading rather than anything about the listing, and leaving the market
/// [`MarketStatus::Active`](crate::MarketStatus::Active). Bithumb states two
/// things Upbit does not publish at all, a severity step and the moment the
/// alert lapses. Read it from
/// [`BithumbAdapter::market_alerts`](BithumbAdapter::market_alerts).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BithumbMarketAlert {
    /// Bithumb's criterion, 경보 유형, verbatim as Bithumb spells it:
    /// `PRICE_SUDDEN_FLUCTUATION`, `PRICE_DIFFERENCE_HIGH`,
    /// `SPECIFIC_ACCOUNT_HIGH_TRANSACTION`,
    /// `TRADING_VOLUME_SUDDEN_FLUCTUATION` or
    /// `DEPOSIT_AMOUNT_SUDDEN_FLUCTUATION`.
    ///
    /// Text rather than an enum, because the list of criteria is the part an
    /// exchange extends. Match on the ones you act on.
    pub kind: String,
    /// How serious Bithumb calls this one.
    pub step: BithumbAlertStep,
    /// When Bithumb has said the alert runs out, 경보 종료 일시.
    ///
    /// Bithumb states it as a Korean wall clock carrying no zone marker; this
    /// is that same instant written in UTC. Alerts on one criterion tend to
    /// share an expiry across every market carrying them, because the criteria
    /// are re-read on a schedule rather than per market.
    pub ends_at: Timestamp,
}

/// Talks to Bithumb.
///
/// Spot only. Bithumb lists no derivatives, so the position, margin, and
/// funding half of [`Client`](crate::Client) reports
/// [`Error::Unsupported`](crate::Error::Unsupported) here.
///
/// Bithumb also publishes no candle stream. Trades, order books, and tickers
/// stream normally, and historical candles are available over REST, but
/// [`Feature::CandleStream`] is unsupported. Build candles from
/// [`Feed::Trades`](crate::Feed::Trades) if you need them live.
///
/// ```
/// use maxt::{Client, Feature, adapters::BithumbAdapter};
///
/// let client = Client::new(BithumbAdapter::new());
///
/// assert!(client.supports(Feature::Candles));      // REST: yes
/// assert!(client.supports(Feature::TradeStream));  // stream: yes
/// assert!(!client.supports(Feature::CandleStream)); // stream: not published
/// ```
#[derive(Debug, Clone)]
pub struct BithumbAdapter {
    credentials: Option<BithumbCredentials>,
    /// Built once at construction, and held as a `Result` because `new` is
    /// infallible by design. A client that cannot be built is a defect in the
    /// process, so the failure is reported at the first call that needs the
    /// network.
    http: Result<HttpTransport>,
}

#[derive(Debug, Clone)]
pub(crate) struct BithumbCredentials {
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
}

impl BithumbAdapter {
    /// An adapter for public market data.
    pub fn new() -> Self {
        Self {
            credentials: None,
            http: HttpTransport::new(REST_BASE_URL),
        }
    }

    /// Adds the API credentials that account, order, and private stream calls
    /// need.
    ///
    /// Bithumb issues an access key and a secret key together.
    #[must_use]
    pub fn with_credentials(
        mut self,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        self.credentials = Some(BithumbCredentials {
            access_key: access_key.into(),
            secret_key: secret_key.into(),
        });
        self
    }

    /// Every listed market paired with Bithumb's investment-warning
    /// designation, 유의 종목.
    ///
    /// Bithumb spells the flag `CAUTION` while documenting the field itself as
    /// 유의 종목 여부, and `NONE` is the only other value the enum holds. A
    /// designated market keeps trading, and the common
    /// [`MarketStatus`](crate::MarketStatus) has no value meaning "trading,
    /// but flagged", so [`Client::markets`](crate::Client::markets) reports it
    /// as [`MarketStatus::Unknown`](crate::MarketStatus::Unknown), which is
    /// where Upbit's warning lands too. The label itself is available only
    /// here, verbatim as Bithumb spells it.
    ///
    /// This is not Bithumb's 주의 종목. That designation is published by a
    /// separate alert system, read through
    /// [`market_alerts`](Self::market_alerts), and never reaches
    /// `MarketStatus`.
    pub async fn market_warnings(&self) -> Result<Vec<(Market, String)>> {
        rest::market_warnings(self.http()?).await
    }

    /// What Bithumb's alert system, 경보제, currently has raised, one entry per
    /// alert.
    ///
    /// A market flagged on several criteria appears once per criterion, and a
    /// market flagged on none is absent, so unlike
    /// [`market_warnings`](Self::market_warnings) this is no substitute for a
    /// market list. Every market here is
    /// [`MarketStatus::Active`](crate::MarketStatus::Active) unless it also
    /// carries the separate warning: an alert describes trading conditions,
    /// not the listing, and folding it into
    /// [`MarketStatus::Unknown`](crate::MarketStatus::Unknown) would bury the
    /// handful of warned markets inside it.
    pub async fn market_alerts(&self) -> Result<Vec<(Market, BithumbMarketAlert)>> {
        rest::market_alerts(self.http()?).await
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }

    /// The credentials a private call needs, or the error it should fail with.
    ///
    /// Missing credentials are an authentication failure, not a missing
    /// feature: Bithumb has these endpoints, this adapter just cannot reach
    /// them yet. Reporting `Unsupported` here would tell a caller to stop
    /// asking, when the fix is to supply a key.
    fn credentials(&self) -> Result<&BithumbCredentials> {
        self.credentials
            .as_ref()
            .ok_or_else(|| Error::auth("bithumb needs both an access key and a secret key"))
    }

    pub(crate) fn http(&self) -> Result<&HttpTransport> {
        self.http.as_ref().map_err(Clone::clone)
    }
}

impl Default for BithumbAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for BithumbAdapter {
    fn exchange(&self) -> Exchange {
        Exchange::Bithumb
    }

    fn supports(&self, feature: Feature) -> bool {
        if feature.is_derivatives_only() {
            return false;
        }
        // Bithumb's public WebSocket carries trades, order books, and tickers,
        // but no candles.
        if matches!(feature, Feature::CandleStream) {
            return false;
        }
        if feature.needs_credentials() {
            return self.is_authenticated();
        }
        true
    }

    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
        Box::pin(async move { rest::markets(self.http()?, kind).await })
    }

    fn trades(&self, market: &Market, limit: Option<u32>) -> BoxFuture<'_, Result<Vec<Trade>>> {
        let market = market.clone();
        Box::pin(async move { rest::trades(self.http()?, &market, limit).await })
    }

    fn order_book(&self, market: &Market, depth: Option<u32>) -> BoxFuture<'_, Result<OrderBook>> {
        let market = market.clone();
        Box::pin(async move { rest::order_book(self.http()?, &market, depth).await })
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        let market = market.clone();
        Box::pin(async move { rest::ticker(self.http()?, &market).await })
    }

    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, Result<Vec<Candle>>> {
        let request = request.clone();
        Box::pin(async move { rest::candles(self.http()?, &request).await })
    }

    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, Result<MarketStream>> {
        let subscription = subscription.clone();
        let config = config.clone();
        Box::pin(async move { stream::subscribe(&subscription, &config).await })
    }

    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        let config = config.clone();
        Box::pin(async move { stream::subscribe_account(self.credentials()?, &config).await })
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        Box::pin(async move { private::balances(self.http()?, self.credentials()?).await })
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        let market = market.cloned();
        Box::pin(async move {
            private::open_orders(self.http()?, self.credentials()?, market.as_ref()).await
        })
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        let request = request.clone();
        Box::pin(
            async move { private::place_order(self.http()?, self.credentials()?, &request).await },
        )
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<Order>> {
        let market = market.clone();
        let order_id = order_id.to_string();
        Box::pin(async move {
            private::cancel_order(self.http()?, self.credentials()?, &market, &order_id).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candles_are_available_over_rest_but_not_as_a_stream() {
        let adapter = BithumbAdapter::new();

        assert!(adapter.supports(Feature::Candles));
        assert!(!adapter.supports(Feature::CandleStream));
    }

    #[test]
    fn every_other_public_stream_is_available() {
        let adapter = BithumbAdapter::new();

        for feature in [
            Feature::TradeStream,
            Feature::OrderBookStream,
            Feature::TickerStream,
        ] {
            assert!(adapter.supports(feature), "{feature:?}");
        }
    }

    #[test]
    fn a_spot_exchange_never_claims_derivatives_features() {
        let adapter = BithumbAdapter::new().with_credentials("access", "secret");

        for feature in [Feature::Positions, Feature::Margin, Feature::FundingRates] {
            assert!(!adapter.supports(feature), "{feature:?}");
        }
    }

    #[tokio::test]
    async fn subscribing_to_candles_is_refused_before_a_socket_is_opened() {
        use crate::types::{Feed, Interval, Market, StreamConfig, Subscription};

        let subscription = Subscription::new()
            .market(Market::spot(Exchange::Bithumb, "BTC", "KRW"))
            .feed(Feed::Candles(Interval::Min1));

        let error = BithumbAdapter::new()
            .subscribe(&subscription, &StreamConfig::default())
            .await
            .expect_err("bithumb publishes no candle stream");

        assert!(matches!(
            error,
            Error::Unsupported {
                feature: Feature::CandleStream,
                exchange: "bithumb",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn a_private_call_without_credentials_is_an_auth_failure_not_a_missing_feature() {
        let error = BithumbAdapter::new()
            .balances()
            .await
            .expect_err("no credentials were supplied");

        // Bithumb has this endpoint; the adapter just cannot reach it yet.
        // `Unsupported` would tell a caller to stop asking, when the fix is to
        // supply a key, and it would disagree with the other three adapters.
        assert!(
            matches!(error, Error::Auth { .. }),
            "expected an auth failure, got {error:?}"
        );
    }

    #[test]
    fn credentials_are_what_unlock_the_private_half() {
        let public = BithumbAdapter::new();
        let private = BithumbAdapter::new().with_credentials("access", "secret");

        for feature in [Feature::Balances, Feature::Trading, Feature::AccountStream] {
            assert!(!public.supports(feature), "{feature:?}");
            assert!(private.supports(feature), "{feature:?}");
        }
    }
}
