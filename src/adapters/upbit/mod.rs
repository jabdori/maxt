//! Upbit spot adapter for Korea, Singapore, Indonesia, and Thailand.

mod parse;
mod private;
mod rest;
mod stream;

use futures_core::Stream;
use futures_util::StreamExt;

use crate::adapter::{Adapter, BoxFuture};
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{CandleRequest, OrderRequest};
use crate::stream::{AccountStream, MarketStream};
use crate::transport::{HttpTransport, WsCommand, WsConnect, WsSession, ws};
use crate::types::{
    AccountEvent, Balance, Candle, Exchange, Market, MarketEvent, MarketInfo, MarketKind, Order,
    OrderBook, StreamConfig, Subscription, Ticker, Trade,
};

/// Selects an Upbit regional deployment.
///
/// Listings, order books, accounts, and credentials are isolated by region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum UpbitRegion {
    /// Upbit Korea. The default.
    #[default]
    Korea,
    /// Upbit Singapore.
    Singapore,
    /// Upbit Indonesia.
    Indonesia,
    /// Upbit Thailand.
    Thailand,
}

impl UpbitRegion {
    pub(crate) const fn rest_base_url(self) -> &'static str {
        match self {
            Self::Korea => "https://api.upbit.com",
            Self::Singapore => "https://sg-api.upbit.com",
            Self::Indonesia => "https://id-api.upbit.com",
            Self::Thailand => "https://th-api.upbit.com",
        }
    }

    pub(crate) const fn websocket_url(self) -> &'static str {
        match self {
            Self::Korea => "wss://api.upbit.com/websocket/v1",
            Self::Singapore => "wss://sg-api.upbit.com/websocket/v1",
            Self::Indonesia => "wss://id-api.upbit.com/websocket/v1",
            Self::Thailand => "wss://th-api.upbit.com/websocket/v1",
        }
    }
}

/// Warning and caution data for one listing.
///
/// Returned by [`UpbitAdapter::market_events`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct UpbitMarketEvent {
    /// Whether Upbit marks the listing with an investment warning (`유의 종목`).
    ///
    /// [`Client::markets`](crate::Client::markets) maps this to
    /// [`MarketStatus::Unknown`](crate::MarketStatus::Unknown). The value does
    /// not state whether new orders are currently accepted.
    pub warning: bool,
    /// Active investment-caution (`주의 종목`) criteria, sorted by Upbit's
    /// criterion name.
    ///
    /// These criteria do not change [`MarketStatus`](crate::MarketStatus).
    /// The list is empty outside [`UpbitRegion::Korea`], whose payload is the
    /// only regional payload that includes the criteria.
    pub cautions: Vec<String>,
}

/// Adapter for Upbit spot markets.
///
/// Derivative features return [`Error::Unsupported`](crate::Error::Unsupported).
#[derive(Debug, Clone)]
pub struct UpbitAdapter {
    region: UpbitRegion,
    credentials: Option<UpbitCredentials>,
    /// Cached transport initialization result, reported on first use.
    http: std::result::Result<HttpTransport, Error>,
}

#[derive(Debug, Clone)]
pub(crate) struct UpbitCredentials {
    pub(crate) access_key: String,
    pub(crate) secret_key: String,
}

impl UpbitAdapter {
    /// Creates an unauthenticated adapter for Upbit Korea.
    pub fn new() -> Self {
        Self::with_region(UpbitRegion::Korea)
    }

    /// Creates an unauthenticated adapter for `region`.
    pub fn with_region(region: UpbitRegion) -> Self {
        Self {
            region,
            credentials: None,
            http: HttpTransport::new(region.rest_base_url()),
        }
    }

    /// Adds credentials for account, order, and private-stream calls.
    ///
    /// The key pair must be issued by the adapter's selected region.
    #[must_use]
    pub fn with_credentials(
        mut self,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        self.credentials = Some(UpbitCredentials {
            access_key: access_key.into(),
            secret_key: secret_key.into(),
        });
        self
    }

    /// Returns the selected region.
    pub fn region(&self) -> UpbitRegion {
        self.region
    }

    /// Fetches order books for one or more markets in one REST request.
    ///
    /// `depth` is the number of levels per side and must be from 1 through 30.
    /// `None` uses Upbit's 30-level default.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidRequest`](crate::Error::InvalidRequest) for an
    /// empty market list, an invalid depth, a different exchange, or an invalid
    /// asset code. Non-spot markets return
    /// [`Error::Unsupported`](crate::Error::Unsupported). Transport, exchange,
    /// and decoding errors are propagated.
    pub async fn order_books(
        &self,
        markets: &[Market],
        depth: Option<u32>,
    ) -> Result<Vec<OrderBook>> {
        rest::order_books(self.http()?, markets, depth).await
    }

    /// Fetches tickers for one or more markets in one REST request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidRequest`](crate::Error::InvalidRequest) for an
    /// empty market list, a different exchange, or an invalid asset code.
    /// Non-spot markets return [`Error::Unsupported`](crate::Error::Unsupported).
    /// Transport, exchange, and decoding errors are propagated.
    pub async fn tickers(&self, markets: &[Market]) -> Result<Vec<Ticker>> {
        rest::tickers(self.http()?, markets).await
    }

    /// Fetches warning and caution data for every listed market.
    ///
    /// Caution criteria are empty outside [`UpbitRegion::Korea`].
    ///
    /// # Errors
    ///
    /// Propagates transport, exchange, and decoding errors.
    pub async fn market_events(&self) -> Result<Vec<(Market, UpbitMarketEvent)>> {
        rest::market_events(self.http()?).await
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.credentials.is_some()
    }

    fn http(&self) -> Result<&HttpTransport> {
        self.http.as_ref().map_err(Clone::clone)
    }

    fn credentials(&self) -> Result<&UpbitCredentials> {
        self.credentials.as_ref().ok_or_else(|| {
            Error::auth(
                "this Upbit adapter has no credentials; add them with \
                 `UpbitAdapter::with_credentials`",
            )
        })
    }
}

impl Default for UpbitAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for UpbitAdapter {
    fn exchange(&self) -> Exchange {
        Exchange::Upbit
    }

    fn supports(&self, feature: Feature) -> bool {
        if feature.is_derivatives_only() {
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
        Box::pin(async move {
            let books = self
                .order_books(std::slice::from_ref(&market), depth)
                .await?;
            rest::only(books, &market)
        })
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        let market = market.clone();
        Box::pin(async move {
            let tickers = self.tickers(std::slice::from_ref(&market)).await?;
            rest::only(tickers, &market)
        })
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
        let frame = stream::subscribe_frame(subscription, &ticket());
        let url = self.region.websocket_url().to_string();
        let config = config.clone();

        Box::pin(async move {
            let session = ws::connect(
                WsConnect {
                    url,
                    headers: None,
                    subscribe: WsConnect::fixed(vec![frame?]),
                    heartbeat: Some(stream::HEARTBEAT),
                },
                &config,
            )
            .await?;

            // Candle completion state belongs to one WebSocket connection.
            let mut decoder = stream::Decoder::default();

            Ok(MarketStream::new(events(
                session,
                move |frame| decoder.decode(frame),
                MarketEvent::Reconnected,
            )))
        })
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        Box::pin(async move { private::balances(self.credentials()?, self.http()?).await })
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        let market = market.cloned();
        Box::pin(async move {
            private::open_orders(self.credentials()?, self.http()?, market.as_ref()).await
        })
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        let request = request.clone();
        Box::pin(
            async move { private::place_order(self.credentials()?, self.http()?, &request).await },
        )
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<Order>> {
        let market = market.clone();
        let order_id = order_id.to_string();
        Box::pin(async move {
            private::cancel_order(self.credentials()?, self.http()?, &market, &order_id).await
        })
    }

    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        let url = format!("{}/private", self.region.websocket_url());
        let config = config.clone();

        Box::pin(async move {
            // The reconnect callback owns the credentials it signs with.
            let credentials = self.credentials()?.clone();
            let session = ws::connect(
                WsConnect {
                    url,
                    // Mint a fresh authorization value for every handshake.
                    headers: Some(Box::new(move || {
                        Ok(vec![(
                            private::AUTHORIZATION.to_string(),
                            private::authorization(&credentials, "")?,
                        )])
                    })),
                    subscribe: WsConnect::fixed(vec![private::subscribe_frame(&ticket())?]),
                    heartbeat: Some(stream::HEARTBEAT),
                },
                &config,
            )
            .await?;

            Ok(AccountStream::new(events(
                session,
                private::account_events,
                AccountEvent::Reconnected,
            )))
        })
    }
}

/// Generates a unique subscription ticket.
fn ticket() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Decodes frames in arrival order and flattens each into zero or more events.
fn events<T: Clone + Send + 'static>(
    session: WsSession,
    mut decode: impl FnMut(&str) -> Result<Vec<T>> + Send + 'static,
    reconnected: T,
) -> impl Stream<Item = Result<T>> + Send {
    session.flat_map(move |item| {
        let items = match item {
            Ok(WsCommand::Text(text)) => split(decode(&text)),
            Ok(WsCommand::Binary(bytes)) => match String::from_utf8(bytes) {
                Ok(text) => split(decode(&text)),
                Err(err) => vec![Err(Error::decode(format!(
                    "upbit sent a frame that is not UTF-8: {err}"
                )))],
            },
            Ok(WsCommand::Reconnected) => vec![Ok(reconnected.clone())],
            Err(err) => vec![Err(err)],
        };

        futures_util::stream::iter(items)
    })
}

/// Converts one decoded frame into stream items.
fn split<T>(decoded: Result<Vec<T>>) -> Vec<Result<T>> {
    match decoded {
        Ok(items) => items.into_iter().map(Ok).collect(),
        Err(err) => vec![Err(err)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spot_exchange_never_claims_derivatives_features() {
        let adapter = UpbitAdapter::new().with_credentials("access", "secret");

        for feature in [
            Feature::Positions,
            Feature::Margin,
            Feature::FundingRates,
            Feature::FundingPayments,
            Feature::MarginConfig,
            Feature::ReduceOnlyOrders,
        ] {
            assert!(!adapter.supports(feature), "{feature:?}");
        }
    }

    #[test]
    fn credentials_are_what_unlock_the_private_half() {
        let public = UpbitAdapter::new();
        let private = UpbitAdapter::new().with_credentials("access", "secret");

        for feature in [Feature::Balances, Feature::Trading, Feature::AccountStream] {
            assert!(!public.supports(feature), "{feature:?}");
            assert!(private.supports(feature), "{feature:?}");
        }
    }

    #[test]
    fn public_market_data_works_without_credentials() {
        let public = UpbitAdapter::new();

        for feature in [
            Feature::Markets,
            Feature::Trades,
            Feature::OrderBook,
            Feature::Ticker,
            Feature::Candles,
            Feature::CandleStream,
        ] {
            assert!(public.supports(feature), "{feature:?}");
        }
    }

    #[tokio::test]
    async fn an_account_call_without_credentials_fails_before_the_network() {
        let public = UpbitAdapter::new();
        let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
        let order = crate::request::OrderRequest::market(
            market.clone(),
            crate::types::Side::Sell,
            crate::types::Size::Base(rust_decimal::Decimal::ONE),
        );

        assert!(matches!(public.balances().await, Err(Error::Auth { .. })));
        assert!(matches!(
            public.open_orders(None).await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.place_order(&order).await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.cancel_order(&market, "an-order").await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.subscribe_account(&StreamConfig::default()).await,
            Err(Error::Auth { .. })
        ));
    }

    #[tokio::test]
    async fn the_derivatives_half_stays_at_the_trait_default() {
        let adapter = UpbitAdapter::new().with_credentials("access", "secret");
        let request =
            crate::request::HistoryRequest::new(Market::perpetual(Exchange::Upbit, "BTC", "KRW"));

        assert!(matches!(
            adapter.positions(None).await,
            Err(Error::Unsupported { .. })
        ));
        assert!(matches!(
            adapter.margin_summary().await,
            Err(Error::Unsupported { .. })
        ));
        assert!(matches!(
            adapter.funding_rates(&request).await,
            Err(Error::Unsupported { .. })
        ));
    }

    #[tokio::test]
    async fn upbit_lists_no_derivatives_and_says_so_with_an_empty_answer() {
        let markets = UpbitAdapter::new()
            .markets(MarketKind::Perpetual)
            .await
            .expect("a listable kind");

        assert!(markets.is_empty());
    }

    #[test]
    fn a_frame_that_carries_no_events_yields_none_and_a_bad_one_yields_one_error() {
        let mut decoder = stream::Decoder::default();

        assert!(
            decoder
                .decode(r#"{"status":"UP"}"#)
                .expect("a control frame")
                .is_empty()
        );
        assert_eq!(split(decoder.decode("not json")).len(), 1);
        assert_eq!(split(decoder.decode(r#"{"status":"UP"}"#)).len(), 0);
    }

    #[test]
    fn each_region_is_a_separate_deployment() {
        assert_eq!(UpbitAdapter::new().region(), UpbitRegion::Korea);
        assert_ne!(
            UpbitRegion::Korea.rest_base_url(),
            UpbitRegion::Singapore.rest_base_url()
        );
        assert!(UpbitRegion::Thailand.websocket_url().starts_with("wss://"));
    }
}
