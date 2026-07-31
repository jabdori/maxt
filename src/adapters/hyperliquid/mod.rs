//! Hyperliquid spot and perpetual market adapter.

mod native;
mod parse;
mod rest;
mod sign;
mod stream;

use futures_util::StreamExt;
use tokio::sync::OnceCell;

use crate::adapter::{Adapter, BoxFuture};
use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{CandleRequest, HistoryRequest, MarginRequest, OrderRequest};
use crate::stream::{AccountStream, MarketStream};
use crate::transport::{HttpTransport, WsCommand, WsConnect, ws};
use crate::types::{
    AccountEvent, Balance, Candle, Cursor, Exchange, FundingPayment, FundingRate, MarginSummary,
    Market, MarketEvent, MarketInfo, MarketKind, Order, OrderBook, Page, Position, StreamConfig,
    Subscription, Ticker, Timestamp, Trade,
};

use parse::Universe;

pub use native::{HyperliquidAssetContext, HyperliquidLedgerEntry, HyperliquidLedgerKind};

pub(crate) const MAINNET_REST_BASE_URL: &str = "https://api.hyperliquid.xyz";
pub(crate) const MAINNET_WEBSOCKET_URL: &str = "wss://api.hyperliquid.xyz/ws";
pub(crate) const TESTNET_REST_BASE_URL: &str = "https://api.hyperliquid-testnet.xyz";
pub(crate) const TESTNET_WEBSOCKET_URL: &str = "wss://api.hyperliquid-testnet.xyz/ws";

/// Adapter for Hyperliquid spot and default perpetual markets.
///
/// Public market-data calls do not require a wallet. Account data, orders,
/// margin changes, and private streams require
/// [`HyperliquidAdapter::with_wallet`].
///
/// Markets are loaded from `meta` and `spotMeta`. HIP-3 DEXes and outcome
/// markets are not exposed by this adapter.
///
/// [`Client::trades`](crate::Client::trades) uses `recentTrades`, which exposes
/// at most ten recent executions and no historical range.
/// [`Feed::Trades`](crate::Feed::Trades) provides live updates but may have a
/// gap after reconnection.
///
/// [`Ticker::last_price`](crate::Ticker::last_price) uses `midPx`, falling back
/// to `markPx`; it is not the latest execution price. Use the trades API when
/// an execution price and trade time are required.
///
/// Provider-specific data is available through
/// [`HyperliquidAdapter::non_funding_ledger`] and
/// [`HyperliquidAdapter::asset_context`].
#[derive(Debug, Clone)]
pub struct HyperliquidAdapter {
    network: HyperliquidNetwork,
    wallet: Option<HyperliquidWallet>,
    connection: OnceCell<Connection>,
}

/// Lazily initialized HTTP transport and cached `meta`/`spotMeta` symbol table.
/// A new adapter is required to refresh listed markets.
#[derive(Debug, Clone)]
struct Connection {
    http: HttpTransport,
    universe: Universe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) enum HyperliquidNetwork {
    #[default]
    Mainnet,
    Testnet,
}

#[derive(Clone)]
pub(crate) struct HyperliquidWallet {
    pub(crate) address: String,
    pub(crate) private_key: String,
}

// Keeps the signing key out of logs and panic messages.
impl std::fmt::Debug for HyperliquidWallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HyperliquidWallet")
            .field("address", &self.address)
            .field("private_key", &"<redacted>")
            .finish()
    }
}

impl HyperliquidAdapter {
    /// An adapter for public mainnet market data.
    pub fn new() -> Self {
        Self::on(HyperliquidNetwork::Mainnet)
    }

    /// An adapter for public testnet market data.
    pub fn testnet() -> Self {
        Self::on(HyperliquidNetwork::Testnet)
    }

    fn on(network: HyperliquidNetwork) -> Self {
        Self {
            network,
            wallet: None,
            connection: OnceCell::new(),
        }
    }

    /// Configures the account and signing key used by private calls.
    ///
    /// `address` is the account acted on. `private_key` is either that account's
    /// key or an approved API wallet key. The key is used only for local signing
    /// and is redacted from [`Debug`] output.
    ///
    /// The values are validated when a private call is made. Invalid values
    /// return [`Error::Auth`](crate::Error::Auth). Feature discovery treats the
    /// private features as configured as soon as a wallet is present.
    #[must_use]
    pub fn with_wallet(
        mut self,
        address: impl Into<String>,
        private_key: impl Into<String>,
    ) -> Self {
        self.wallet = Some(HyperliquidWallet {
            address: address.into(),
            private_key: private_key.into(),
        });
        self
    }

    /// Whether this adapter talks to testnet.
    pub fn is_testnet(&self) -> bool {
        self.network == HyperliquidNetwork::Testnet
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.wallet.is_some()
    }

    pub(crate) fn rest_base_url(&self) -> &'static str {
        match self.network {
            HyperliquidNetwork::Mainnet => MAINNET_REST_BASE_URL,
            HyperliquidNetwork::Testnet => TESTNET_REST_BASE_URL,
        }
    }

    pub(crate) fn websocket_url(&self) -> &'static str {
        match self.network {
            HyperliquidNetwork::Mainnet => MAINNET_WEBSOCKET_URL,
            HyperliquidNetwork::Testnet => TESTNET_WEBSOCKET_URL,
        }
    }

    /// Reads account-wide non-funding ledger entries.
    ///
    /// Entries include deposits, withdrawals, transfers, and liquidations. They
    /// are distinct from market-scoped [`FundingPayment`](crate::FundingPayment)
    /// records. See [`HyperliquidLedgerEntry`].
    ///
    /// Pass [`Page::next`] back as `cursor` until it is `None`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Auth`](crate::Error::Auth) when no valid wallet is
    /// configured. Invalid cursors, transport failures, exchange errors, and
    /// payload changes are also returned.
    pub async fn non_funding_ledger(
        &self,
        from: Option<Timestamp>,
        to: Option<Timestamp>,
        cursor: Option<&Cursor>,
        limit: Option<u32>,
    ) -> Result<Page<HyperliquidLedgerEntry>> {
        let (user, _) = self.account()?;
        let connection = self.connect().await?;

        rest::ledger(&connection.http, &user, from, to, cursor, limit).await
    }

    /// Reads the current asset context for one market.
    ///
    /// The context includes mid, mark, and oracle prices; the current funding
    /// rate; open interest; and order precision. Historical market-rate
    /// observations use [`FundingRate`](crate::FundingRate), while actual
    /// account charges use [`FundingPayment`](crate::FundingPayment). See
    /// [`HyperliquidAssetContext`].
    ///
    /// # Errors
    ///
    /// Returns an error for an unlisted or non-Hyperliquid market, a transport
    /// or exchange failure, or an invalid response payload.
    pub async fn asset_context(&self, market: &Market) -> Result<HyperliquidAssetContext> {
        let connection = self.connect().await?;
        let raw = rest::context(&connection.http, &connection.universe, market).await?;
        let asset = connection.universe.asset(market)?;

        native::asset_context(&raw, asset)
    }

    /// Initializes the HTTP client and symbol table once.
    async fn connect(&self) -> Result<&Connection> {
        self.connection
            .get_or_try_init(|| async {
                let http = HttpTransport::new(self.rest_base_url())?;
                let universe = rest::universe(&http).await?;

                Ok(Connection { http, universe })
            })
            .await
    }

    /// Returns the normalized account address and validated signing key.
    fn account(&self) -> Result<(String, &str)> {
        let wallet = self.wallet.as_ref().ok_or_else(sign::missing_wallet)?;
        let address = sign::check_wallet(&wallet.address, &wallet.private_key)?;

        Ok((address, &wallet.private_key))
    }

    fn signing_key(&self) -> Result<&str> {
        self.wallet
            .as_ref()
            .map(|wallet| wallet.private_key.as_str())
            .ok_or_else(sign::missing_wallet)
    }
}

impl Default for HyperliquidAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Adapter for HyperliquidAdapter {
    fn exchange(&self) -> Exchange {
        Exchange::Hyperliquid
    }

    fn supports(&self, feature: Feature) -> bool {
        if feature.needs_credentials() {
            return self.is_authenticated();
        }
        true
    }

    fn trades(&self, market: &Market, limit: Option<u32>) -> BoxFuture<'_, Result<Vec<Trade>>> {
        let market = market.clone();
        Box::pin(async move {
            let connection = self.connect().await?;

            rest::trades(&connection.http, &connection.universe, &market, limit).await
        })
    }

    fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
        Box::pin(async move {
            let connection = self.connect().await?;

            Ok(rest::markets(&connection.universe, kind))
        })
    }

    fn order_book(&self, market: &Market, depth: Option<u32>) -> BoxFuture<'_, Result<OrderBook>> {
        let market = market.clone();
        Box::pin(async move {
            let connection = self.connect().await?;

            rest::order_book(&connection.http, &connection.universe, &market, depth).await
        })
    }

    fn ticker(&self, market: &Market) -> BoxFuture<'_, Result<Ticker>> {
        let market = market.clone();
        Box::pin(async move {
            let connection = self.connect().await?;

            rest::ticker(&connection.http, &connection.universe, &market).await
        })
    }

    fn candles(&self, request: &CandleRequest) -> BoxFuture<'_, Result<Vec<Candle>>> {
        let request = request.clone();
        Box::pin(async move {
            let connection = self.connect().await?;

            rest::candles(
                &connection.http,
                &connection.universe,
                &request,
                Timestamp::now(),
            )
            .await
        })
    }

    fn subscribe(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> BoxFuture<'_, Result<MarketStream>> {
        let subscription = subscription.clone();
        let config = config.clone();
        let url = self.websocket_url();

        Box::pin(async move {
            let connection = self.connect().await?;
            let frames = stream::subscribe_frames(&subscription, &connection.universe)?;
            let session = ws::connect(
                WsConnect {
                    url: url.to_string(),
                    headers: None,
                    subscribe: WsConnect::fixed(frames),
                    heartbeat: Some(stream::HEARTBEAT),
                },
                &config,
            )
            .await?;

            let universe = connection.universe.clone();
            // Candle state is scoped to one connection and cleared on reconnect.
            let mut decoder = stream::Decoder::default();

            Ok(MarketStream::new(session.flat_map(move |command| {
                futures_util::stream::iter(market_events(command, &universe, &mut decoder))
            })))
        })
    }

    fn balances(&self) -> BoxFuture<'_, Result<Vec<Balance>>> {
        Box::pin(async move {
            let (user, _) = self.account()?;
            let connection = self.connect().await?;

            rest::balances(&connection.http, &user).await
        })
    }

    fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Order>>> {
        let market = market.cloned();
        Box::pin(async move {
            let (user, _) = self.account()?;
            let connection = self.connect().await?;

            rest::open_orders(
                &connection.http,
                &connection.universe,
                &user,
                market.as_ref(),
            )
            .await
        })
    }

    fn subscribe_account(&self, config: &StreamConfig) -> BoxFuture<'_, Result<AccountStream>> {
        let config = config.clone();
        let url = self.websocket_url();

        Box::pin(async move {
            let (user, _) = self.account()?;
            let connection = self.connect().await?;
            let session = ws::connect(
                WsConnect {
                    url: url.to_string(),
                    headers: None,
                    subscribe: WsConnect::fixed(stream::account_subscribe_frames(&user)),
                    heartbeat: Some(stream::HEARTBEAT),
                },
                &config,
            )
            .await?;

            let universe = connection.universe.clone();
            Ok(AccountStream::new(session.flat_map(move |command| {
                futures_util::stream::iter(account_events(command, &universe))
            })))
        })
    }

    fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, Result<Order>> {
        let request = request.clone();
        Box::pin(async move {
            // Validate the wallet before building the request.
            self.account()?;
            let connection = self.connect().await?;

            rest::place_order(
                &connection.http,
                &connection.universe,
                self.signing_key()?,
                self.network,
                &request,
                rest::nonce(Timestamp::now()),
            )
            .await
        })
    }

    fn cancel_order(&self, market: &Market, order_id: &str) -> BoxFuture<'_, Result<Order>> {
        let market = market.clone();
        let order_id = order_id.to_string();

        Box::pin(async move {
            self.account()?;
            let connection = self.connect().await?;

            rest::cancel_order(
                &connection.http,
                &connection.universe,
                self.signing_key()?,
                self.network,
                &market,
                &order_id,
                rest::nonce(Timestamp::now()),
            )
            .await
        })
    }

    fn positions(&self, market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Position>>> {
        let market = market.cloned();
        Box::pin(async move {
            let (user, _) = self.account()?;
            let connection = self.connect().await?;

            rest::positions(
                &connection.http,
                &connection.universe,
                &user,
                market.as_ref(),
            )
            .await
        })
    }

    fn margin_summary(&self) -> BoxFuture<'_, Result<MarginSummary>> {
        Box::pin(async move {
            let (user, _) = self.account()?;
            let connection = self.connect().await?;

            rest::margin_summary(&connection.http, &user).await
        })
    }

    fn funding_rates(&self, request: &HistoryRequest) -> BoxFuture<'_, Result<Page<FundingRate>>> {
        let request = request.clone();
        Box::pin(async move {
            let connection = self.connect().await?;

            rest::funding_rates(&connection.http, &connection.universe, &request).await
        })
    }

    fn funding_payments(
        &self,
        request: &HistoryRequest,
    ) -> BoxFuture<'_, Result<Page<FundingPayment>>> {
        let request = request.clone();
        Box::pin(async move {
            let (user, _) = self.account()?;
            let connection = self.connect().await?;

            rest::funding_payments(&connection.http, &connection.universe, &user, &request).await
        })
    }

    fn set_margin(&self, request: &MarginRequest) -> BoxFuture<'_, Result<()>> {
        let request = request.clone();
        Box::pin(async move {
            self.account()?;
            let connection = self.connect().await?;

            rest::set_margin(
                &connection.http,
                &connection.universe,
                self.signing_key()?,
                self.network,
                &request,
                rest::nonce(Timestamp::now()),
            )
            .await
        })
    }
}

/// Decodes one connection event into zero or more market events.
///
/// The decoder preserves candle state between frames and clears it on
/// reconnect, because a gap prevents the prior window from being settled.
fn market_events(
    command: Result<WsCommand>,
    universe: &Universe,
    decoder: &mut stream::Decoder,
) -> Vec<Result<MarketEvent>> {
    let text = match command {
        Ok(WsCommand::Text(text)) => text,
        // Accept UTF-8 binary frames from intermediaries.
        Ok(WsCommand::Binary(bytes)) => match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(err) => return vec![Err(Error::decode(format!("frame is not UTF-8: {err}")))],
        },
        Ok(WsCommand::Reconnected) => {
            decoder.reconnected();
            return vec![Ok(MarketEvent::Reconnected)];
        }
        Err(err) => return vec![Err(err)],
    };

    match decoder.decode(&text, universe, Timestamp::now()) {
        Ok(events) => events.into_iter().map(Ok).collect(),
        Err(err) => vec![Err(err)],
    }
}

/// Decodes one private connection event into zero or more account events.
fn account_events(command: Result<WsCommand>, universe: &Universe) -> Vec<Result<AccountEvent>> {
    let text = match command {
        Ok(WsCommand::Text(text)) => text,
        Ok(WsCommand::Binary(bytes)) => match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(err) => return vec![Err(Error::decode(format!("frame is not UTF-8: {err}")))],
        },
        Ok(WsCommand::Reconnected) => return vec![Ok(AccountEvent::Reconnected)],
        Err(err) => return vec![Err(err)],
    };

    match stream::decode_account(&text, universe) {
        Ok(events) => events.into_iter().map(Ok).collect(),
        Err(err) => vec![Err(err)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parse::tests::universe;

    /// Builds a one-minute BTC candle frame opening at `open_ms`.
    fn candle_frame(open_ms: i64) -> String {
        format!(
            r#"{{"channel":"candle","data":{{"T":{},"c":100,"h":100,"i":"1m","l":100,"n":12,"o":100,"s":"BTC","t":{},"v":1}}}}"#,
            open_ms + 59_999,
            open_ms
        )
    }

    #[test]
    fn a_reconnect_drops_the_held_window_instead_of_settling_it_across_the_gap() {
        // A pre-gap candle cannot be settled from a post-gap frame.
        const WINDOW_ONE_MS: i64 = 1_785_397_500_000;
        const WINDOW_TWO_MS: i64 = 1_785_397_560_000;

        let universe = universe();
        let mut decoder = stream::Decoder::default();
        let text = |frame: String| Ok(WsCommand::Text(frame));

        let events = market_events(text(candle_frame(WINDOW_ONE_MS)), &universe, &mut decoder);
        assert_eq!(events.len(), 1, "the first frame of a window is forming");

        let events = market_events(Ok(WsCommand::Reconnected), &universe, &mut decoder);
        assert!(matches!(events.as_slice(), [Ok(MarketEvent::Reconnected)]));

        // Reconnect cleared the held candle, so only the new forming one emits.
        let events = market_events(text(candle_frame(WINDOW_TWO_MS)), &universe, &mut decoder);
        assert_eq!(
            events.len(),
            1,
            "a window from before the gap is not settled by a frame after it: {events:?}"
        );
        let [Ok(MarketEvent::Candle(forming))] = events.as_slice() else {
            panic!("expected one forming candle: {events:?}");
        };
        assert!(!forming.closed);
        assert_eq!(forming.open_time, Timestamp::from_millis(WINDOW_TWO_MS));

        // Candle settlement resumes within the new connection.
        let events = market_events(
            text(candle_frame(WINDOW_TWO_MS + 60_000)),
            &universe,
            &mut decoder,
        );
        assert_eq!(events.len(), 2, "{events:?}");
        let [Ok(MarketEvent::Candle(settled)), _] = events.as_slice() else {
            panic!("expected a settled window: {events:?}");
        };
        assert!(settled.closed);
        assert_eq!(settled.open_time, Timestamp::from_millis(WINDOW_TWO_MS));
    }

    #[test]
    fn trades_are_served_both_live_and_over_rest() {
        let adapter = HyperliquidAdapter::new();

        assert!(adapter.supports(Feature::TradeStream));
        assert!(adapter.supports(Feature::Trades));
    }

    #[test]
    fn one_adapter_serves_both_spot_and_perpetual_markets() {
        let adapter = HyperliquidAdapter::new().with_wallet("0xabc", "0xdef");

        for feature in [
            Feature::Positions,
            Feature::Margin,
            Feature::FundingRates,
            Feature::MarginConfig,
            Feature::Trading,
        ] {
            assert!(adapter.supports(feature), "{feature:?}");
        }
    }

    #[test]
    fn a_wallet_is_what_unlocks_the_private_half() {
        let public = HyperliquidAdapter::new();
        let signed = HyperliquidAdapter::new().with_wallet("0xabc", "0xdef");

        for feature in [Feature::Balances, Feature::Trading, Feature::Positions] {
            assert!(!public.supports(feature), "{feature:?}");
            assert!(signed.supports(feature), "{feature:?}");
        }
    }

    #[test]
    fn the_signing_key_never_appears_in_debug_output() {
        let adapter = HyperliquidAdapter::new().with_wallet("0xabc", "0xdeadbeef");
        let rendered = format!("{adapter:?}");

        assert!(!rendered.contains("0xdeadbeef"));
        assert!(rendered.contains("<redacted>"));
        assert!(rendered.contains("0xabc"));
    }

    #[tokio::test]
    async fn a_private_call_without_a_wallet_says_so_before_it_reaches_the_network() {
        // Authentication fails before a network connection is attempted.
        let public = HyperliquidAdapter::new();
        let market = Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC");

        assert!(matches!(public.balances().await, Err(Error::Auth { .. })));
        assert!(matches!(
            public.positions(None).await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.margin_summary().await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.cancel_order(&market, "1").await,
            Err(Error::Auth { .. })
        ));
        assert!(matches!(
            public.non_funding_ledger(None, None, None, None).await,
            Err(Error::Auth { .. })
        ));
    }

    #[tokio::test]
    async fn a_wallet_that_cannot_sign_is_rejected_before_the_network() {
        let broken = HyperliquidAdapter::new().with_wallet("0xabc", "not-a-key");

        assert!(matches!(broken.balances().await, Err(Error::Auth { .. })));
    }

    #[test]
    fn testnet_and_mainnet_are_separate_hosts() {
        let mainnet = HyperliquidAdapter::new();
        let testnet = HyperliquidAdapter::testnet();

        assert!(!mainnet.is_testnet());
        assert!(testnet.is_testnet());
        assert_ne!(mainnet.rest_base_url(), testnet.rest_base_url());
        assert_ne!(mainnet.websocket_url(), testnet.websocket_url());
    }
}
