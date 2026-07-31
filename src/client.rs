//! The common API.

use crate::adapter::Adapter;
use crate::error::Result;
use crate::feature::Feature;
use crate::request::{CandleRequest, HistoryRequest, MarginRequest, OrderRequest};
use crate::stream::{AccountStream, MarketStream};
use crate::types::{
    Balance, Candle, Exchange, FundingPayment, FundingRate, MarginSummary, Market, MarketInfo,
    MarketKind, Order, OrderBook, Page, Position, StreamConfig, Subscription, Ticker, Trade,
};

/// Everything the supported exchanges have in common, in one type.
///
/// Wrap an adapter and the calls below mean the same thing whichever exchange
/// is underneath. Exchange-specific behaviour stays off the common API, where
/// flattening it would change what it means. To reach it, use
/// [`Client::adapter`] and call that adapter's own typed methods.
///
/// ```no_run
/// use maxt::{Client, Exchange, Market, adapters::UpbitAdapter};
///
/// # async fn run() -> maxt::Result<()> {
/// let client = Client::new(UpbitAdapter::new());
/// let ticker = client.ticker(&Market::spot(Exchange::Upbit, "BTC", "KRW")).await?;
///
/// println!("{}", ticker.last_price);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client<A> {
    adapter: A,
}

impl<A: Adapter> Client<A> {
    /// Wraps an adapter.
    ///
    /// Credentials belong on the adapter, since each exchange issues a
    /// different pair of them. An adapter built without any serves public
    /// market data.
    ///
    /// ```
    /// use maxt::{Client, Feature, adapters::UpbitAdapter};
    ///
    /// // Keys come from the environment. An absent key yields a public client.
    /// let adapter = match (std::env::var("UPBIT_ACCESS_KEY"), std::env::var("UPBIT_SECRET_KEY")) {
    ///     (Ok(access), Ok(secret)) => UpbitAdapter::new().with_credentials(access, secret),
    ///     _ => UpbitAdapter::new(),
    /// };
    /// let client = Client::new(adapter);
    ///
    /// // True either way: reading a ticker needs no key.
    /// assert!(client.supports(Feature::Ticker));
    /// ```
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    /// Which exchange this client talks to.
    ///
    /// This names the exchange, not the venue. Binance spot and Binance USD-M
    /// futures are two adapters that answer the same way here, and are told
    /// apart by the [`MarketKind`] of the markets they trade.
    ///
    /// ```
    /// use maxt::{Client, Exchange, adapters::BinanceAdapter};
    ///
    /// let spot = Client::new(BinanceAdapter::spot());
    /// let perp = Client::new(BinanceAdapter::usd_m_futures());
    ///
    /// assert_eq!(spot.exchange(), Exchange::Binance);
    /// assert_eq!(perp.exchange(), spot.exchange());
    /// assert_eq!(spot.exchange().id(), "binance");
    /// ```
    pub fn exchange(&self) -> Exchange {
        self.adapter.exchange()
    }

    /// Whether this client offers a feature.
    ///
    /// Ask here when the answer should change what your program does. Calling
    /// an unsupported feature returns
    /// [`Error::Unsupported`](crate::Error::Unsupported).
    ///
    /// ```
    /// use maxt::{Client, Feed, Feature, Interval, Subscription, adapters::BithumbAdapter};
    ///
    /// let client = Client::new(BithumbAdapter::new());
    ///
    /// // Bithumb publishes no candle stream, so live candles there have to be
    /// // aggregated from trades.
    /// let feed = if client.supports(Feature::CandleStream) {
    ///     Feed::Candles(Interval::Min1)
    /// } else {
    ///     Feed::Trades
    /// };
    /// let subscription = Subscription::new().feed(feed);
    ///
    /// assert_eq!(subscription.feeds(), [Feed::Trades]);
    /// ```
    pub fn supports(&self, feature: Feature) -> bool {
        self.adapter.supports(feature)
    }

    /// The underlying adapter, for the exchange's own typed calls.
    ///
    /// What the common API has no shape for stays here: Upbit's batched book
    /// reads, Bithumb's caution labels, Hyperliquid's ledger. Reaching for the
    /// adapter is the supported way to use them.
    ///
    /// ```
    /// use maxt::{Client, adapters::{BinanceAdapter, BinanceMarket}};
    ///
    /// let client = Client::new(BinanceAdapter::usd_m_futures());
    ///
    /// // `venue` exists on the adapter alone: no other exchange splits its
    /// // markets across two hosts this way.
    /// assert_eq!(client.adapter().venue(), BinanceMarket::UsdMFutures);
    /// ```
    pub fn adapter(&self) -> &A {
        &self.adapter
    }

    /// Unwraps back to the adapter.
    ///
    /// Adapters take their credentials by value, so this is how a public client
    /// becomes an authenticated one without being rebuilt from scratch.
    ///
    /// ```
    /// use maxt::{Client, Feature, adapters::BithumbAdapter};
    ///
    /// let public = Client::new(BithumbAdapter::new());
    /// assert!(!public.supports(Feature::Trading));
    ///
    /// let trading = Client::new(public.into_adapter().with_credentials("access", "secret"));
    /// assert!(trading.supports(Feature::Trading));
    /// ```
    pub fn into_adapter(self) -> A {
        self.adapter
    }

    /// Lists the exchange's markets of one kind.
    ///
    /// A spot-only exchange returns an empty list for
    /// [`MarketKind::Perpetual`]. The question is meaningful there, and the
    /// answer is "none".
    ///
    /// ```no_run
    /// use maxt::{Client, MarketKind, MarketStatus, adapters::UpbitAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(UpbitAdapter::new());
    ///
    /// let krw: Vec<_> = client
    ///     .markets(MarketKind::Spot)
    ///     .await?
    ///     .into_iter()
    ///     // A listing is not a promise that it trades right now.
    ///     .filter(|info| info.market.quote == "KRW" && info.status == MarketStatus::Active)
    ///     .collect();
    ///
    /// for info in &krw {
    ///     // `market` is what other calls take; `native_symbol` is what the
    ///     // exchange's own UI calls the same thing.
    ///     println!("{} is {} upstream", info.market, info.native_symbol);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn markets(&self, kind: MarketKind) -> Result<Vec<MarketInfo>> {
        self.adapter.markets(kind).await
    }

    /// Reads the most recent trades on a market, newest first.
    ///
    /// Newest first on every adapter that offers this at all, whatever order
    /// the exchange sent.
    ///
    /// # Errors
    ///
    /// A `limit` past what one call serves is
    /// [`Error::InvalidRequest`](crate::Error::InvalidRequest) on `limit` rather
    /// than a short answer, and the ceiling is not the same everywhere: 1 000 on
    /// Binance, 500 on Upbit and Bithumb, 10 on Hyperliquid, whose endpoint takes
    /// no count at all. Leave `limit` unset to take whatever the exchange sends.
    /// A window wider than that is [`Feed::Trades`](crate::Feed).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use maxt::{Client, Exchange, Market, Side, adapters::BinanceAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(BinanceAdapter::spot());
    /// let market = Market::spot(Exchange::Binance, "BTC", "USDT");
    ///
    /// let trades = client.trades(&market, Some(100)).await?;
    ///
    /// // Newest first, so the head is the last print. No sorting needed.
    /// if let Some(latest) = trades.first() {
    ///     println!("{} at {}", latest.price, latest.timestamp);
    /// }
    ///
    /// // `taker_side` is the aggressor's side, the same convention on every
    /// // supported exchange, so it compares across them.
    /// let bought = trades.iter().filter(|trade| trade.taker_side == Side::Buy).count();
    /// println!("{bought} of {} lifted an ask", trades.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn trades(&self, market: &Market, limit: Option<u32>) -> Result<Vec<Trade>> {
        self.adapter.trades(market, limit).await
    }

    /// Reads an order book snapshot.
    ///
    /// `depth` asks for that many levels per side.
    ///
    /// # Errors
    ///
    /// A depth the exchange cannot serve is refused with
    /// [`Error::InvalidRequest`](crate::Error::InvalidRequest). Binance serves
    /// a fixed set of depths, so asking it for 30 levels fails instead of
    /// returning 50. Each provider page lists what its exchange accepts.
    ///
    /// # Examples
    ///
    /// ```
    /// use maxt::{Client, Error, Exchange, Market, adapters::BinanceAdapter};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::new(BinanceAdapter::spot());
    ///     let market = Market::spot(Exchange::Binance, "BTC", "USDT");
    ///
    ///     // Refused locally, before a request is sent: 30 is not one of the
    ///     // depths Binance serves.
    ///     let rejected = client.order_book(&market, Some(30)).await;
    ///     assert!(matches!(rejected, Err(Error::InvalidRequest { field: "depth", .. })));
    /// }
    /// ```
    ///
    /// Pass `None` for the exchange's own default depth, which every exchange
    /// accepts.
    ///
    /// ```no_run
    /// use maxt::{Client, Exchange, Market, adapters::BinanceAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(BinanceAdapter::spot());
    /// let book = client
    ///     .order_book(&Market::spot(Exchange::Binance, "BTC", "USDT"), None)
    ///     .await?;
    ///
    /// // Both sides are best-first, so the top of book is the front of each.
    /// if let (Some(bid), Some(ask)) = (book.best_bid(), book.best_ask()) {
    ///     println!("{} / {}", bid.price, ask.price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn order_book(&self, market: &Market, depth: Option<u32>) -> Result<OrderBook> {
        self.adapter.order_book(market, depth).await
    }

    /// Reads a market's rolling 24-hour summary.
    ///
    /// ```no_run
    /// use maxt::{Client, Exchange, Market, adapters::HyperliquidAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(HyperliquidAdapter::new());
    /// let ticker = client
    ///     .ticker(&Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC"))
    ///     .await?;
    ///
    /// // Hyperliquid publishes no clock with its summaries, so `timestamp` is
    /// // the read time there and says nothing about when the price traded.
    /// // `last_trade_time` stays `None`.
    /// match ticker.last_trade_time {
    ///     Some(traded_at) => println!("{} as of {traded_at}", ticker.last_price),
    ///     None => println!("{} (age unknown, read at {})", ticker.last_price, ticker.timestamp),
    /// }
    ///
    /// // Anything the exchange does not report stays `None`. A market that
    /// // truly traded nothing reports `Some(0)`.
    /// if let Some(volume) = ticker.volume {
    ///     println!("{volume} traded");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ticker(&self, market: &Market) -> Result<Ticker> {
        self.adapter.ticker(market).await
    }

    /// Reads historical candles, oldest first.
    ///
    /// [`CandleRequest::limit`] is honoured past what one response can carry.
    /// `maxt` pages to reach it, so a five-hundred-candle backfill is one call
    /// here whatever the exchange's per-response cap is.
    ///
    /// Paging is bounded. A request needing more than a hundred calls, whether
    /// a `from` far in the past with no `limit` or a `limit` past a hundred
    /// pages, is refused as
    /// [`Error::InvalidRequest`](crate::Error::InvalidRequest) before the first
    /// call, naming the field and the ceiling. Read a long history by looping
    /// on `from` with a `limit`, as the example below does.
    ///
    /// ```no_run
    /// use maxt::{CandleRequest, Client, Exchange, Interval, Market, Timestamp, adapters::UpbitAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(UpbitAdapter::new());
    /// let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
    ///
    /// // With `from` set, `limit` counts forward: the oldest 500 candles at or
    /// // after that instant.
    /// let mut cursor = Timestamp::from_millis(1_700_000_000_000);
    /// let batch = client
    ///     .candles(&CandleRequest::new(market.clone(), Interval::Min1).from(cursor).limit(500))
    ///     .await?;
    ///
    /// // Oldest first, so the last candle is where the next batch starts.
    /// if let Some(newest) = batch.last() {
    ///     cursor = newest.open_time;
    /// }
    ///
    /// // Without `from`, the same `limit` counts back from now instead.
    /// let latest = client
    ///     .candles(&CandleRequest::new(market, Interval::Hour1).limit(24))
    ///     .await?;
    /// println!("{} candles, resuming at {cursor}", latest.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn candles(&self, request: &CandleRequest) -> Result<Vec<Candle>> {
        self.adapter.candles(request).await
    }

    /// Opens a live market data subscription with default connection settings.
    ///
    /// Use [`Client::subscribe_with`] to change reconnect and buffering
    /// behaviour.
    ///
    /// ```no_run
    /// use futures_util::StreamExt;
    /// use maxt::{Client, Exchange, Feed, Market, MarketEvent, Subscription, adapters::UpbitAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(UpbitAdapter::new());
    /// let subscription = Subscription::new()
    ///     .market(Market::spot(Exchange::Upbit, "BTC", "KRW"))
    ///     .feed(Feed::Trades);
    ///
    /// let mut stream = client.subscribe(&subscription).await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     match event {
    ///         Ok(MarketEvent::Trade(trade)) => println!("{} {}", trade.price, trade.quantity),
    ///         Ok(MarketEvent::Reconnected) => println!("gap: anything sent while down was missed"),
    ///         Ok(_) => {}
    ///         // An error is a report, not the end: the subscription may
    ///         // recover on its own. Only `None` means nothing more is coming.
    ///         Err(error) => eprintln!("{error}"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(&self, subscription: &Subscription) -> Result<MarketStream> {
        self.subscribe_with(subscription, &StreamConfig::default())
            .await
    }

    /// Opens a live market data subscription.
    ///
    /// ```no_run
    /// use maxt::{
    ///     Client, Exchange, Feed, Market, Overflow, StreamConfig, Subscription,
    ///     adapters::BinanceAdapter,
    /// };
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let config = StreamConfig {
    ///     // The default. A dropped trade is gone: no later event restates
    ///     // it, so a volume total computed from the rest is silently short.
    ///     // The cost is latency, and the risk is a socket Binance closes if
    ///     // the consumer stalls for long enough.
    ///     overflow: Overflow::Backpressure,
    ///     buffer_size: 16_384,
    ///     // Give up after ten attempts, so a supervisor above this can
    ///     // restart the whole process.
    ///     max_reconnect_attempts: Some(10),
    ///     ..StreamConfig::default()
    /// };
    ///
    /// let subscription = Subscription::new()
    ///     .market(Market::perpetual(Exchange::Binance, "BTC", "USDT"))
    ///     .feed(Feed::Trades);
    ///
    /// let stream = Client::new(BinanceAdapter::usd_m_futures())
    ///     .subscribe_with(&subscription, &config)
    ///     .await?;
    /// drop(stream); // dropping closes the connection
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_with(
        &self,
        subscription: &Subscription,
        config: &StreamConfig,
    ) -> Result<MarketStream> {
        self.adapter.subscribe(subscription, config).await
    }

    /// Reads the account's balances.
    ///
    /// Requires credentials.
    ///
    /// # Errors
    ///
    /// An adapter built without credentials fails with
    /// [`Error::Auth`](crate::Error::Auth) before anything is sent, on every
    /// exchange.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use maxt::{Client, adapters::BithumbAdapter};
    /// use rust_decimal::Decimal;
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(BithumbAdapter::new().with_credentials("access", "secret"));
    ///
    /// let holdings = client.balances().await?;
    ///
    /// // `available` is what can be spent now. `total` adds what open orders
    /// // have reserved, and is the figure a portfolio should report.
    /// let krw = holdings.iter().find(|balance| balance.asset == "KRW");
    /// println!("{}", krw.map_or(Decimal::ZERO, |balance| balance.total()));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn balances(&self) -> Result<Vec<Balance>> {
        self.adapter.balances().await
    }

    /// Reads the account's open orders across every market.
    ///
    /// Requires credentials.
    ///
    /// ```no_run
    /// use maxt::{Client, adapters::UpbitAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(UpbitAdapter::new().with_credentials("access", "secret"));
    ///
    /// for order in client.open_orders().await? {
    ///     // An order listed here may already be finished. Read the status
    ///     // before assuming it can still fill.
    ///     if order.status.is_live() {
    ///         println!("{} on {}: {} left", order.id, order.market, order.remaining_quantity);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn open_orders(&self) -> Result<Vec<Order>> {
        self.adapter.open_orders(None).await
    }

    /// Reads the account's open orders on one market.
    ///
    /// Requires credentials. The exchange does the filtering in one request,
    /// which matters on an account that trades many markets and on exchanges
    /// that charge the wider query more quota.
    ///
    /// ```no_run
    /// use maxt::{Client, Exchange, Market, adapters::UpbitAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(UpbitAdapter::new().with_credentials("access", "secret"));
    /// let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
    ///
    /// // Cancelling by identifier is the only safe way: the exchange's own id
    /// // is what `cancel_order` matches on.
    /// for order in client.open_orders_on(&market).await? {
    ///     client.cancel_order(&order.market, &order.id).await?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn open_orders_on(&self, market: &Market) -> Result<Vec<Order>> {
        self.adapter.open_orders(Some(market)).await
    }

    /// Opens a live private account subscription with default settings.
    ///
    /// Requires credentials.
    ///
    /// A Binance USD-M order arrives with no `created_at`: that stream
    /// publishes no creation time, so a USD-M order's age comes from the REST
    /// read.
    ///
    /// ```no_run
    /// use futures_util::StreamExt;
    /// use maxt::{AccountEvent, Client, adapters::BinanceAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(BinanceAdapter::spot().with_credentials("key", "secret"));
    /// let mut stream = client.subscribe_account().await?;
    ///
    /// while let Some(event) = stream.next().await {
    ///     match event {
    ///         Ok(AccountEvent::Order(order)) => println!("{} is {:?}", order.id, order.status),
    ///         Ok(AccountEvent::Balance(balance)) => println!("{} {}", balance.asset, balance.total()),
    ///         // Updates published while the socket was down were missed, so
    ///         // the local view is stale until it is read back over REST.
    ///         Ok(AccountEvent::Reconnected) => {
    ///             let _ = client.open_orders().await?;
    ///         }
    ///         Ok(_) => {}
    ///         Err(error) => eprintln!("{error}"),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_account(&self) -> Result<AccountStream> {
        self.subscribe_account_with(&StreamConfig::default()).await
    }

    /// Opens a live private account subscription.
    ///
    /// Requires credentials.
    ///
    /// ```no_run
    /// use maxt::{Client, StreamConfig, adapters::BinanceAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let config = StreamConfig {
    ///     // A quiet account is not a broken one. Binance pings a private
    ///     // socket every three minutes, so anything under that would tear
    ///     // down a healthy connection. The adapter raises a too-short value
    ///     // to what its exchange can meet, and honours a longer one.
    ///     idle_timeout_ms: 300_000,
    ///     ..StreamConfig::default()
    /// };
    ///
    /// let stream = Client::new(BinanceAdapter::spot().with_credentials("key", "secret"))
    ///     .subscribe_account_with(&config)
    ///     .await?;
    /// drop(stream);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe_account_with(&self, config: &StreamConfig) -> Result<AccountStream> {
        self.adapter.subscribe_account(config).await
    }

    /// Places an order.
    ///
    /// Requires credentials. The returned [`Order`] carries the exchange's own
    /// identifier, which is what [`Client::cancel_order`] takes.
    ///
    /// ```no_run
    /// use maxt::{
    ///     Client, Exchange, Market, OrderRequest, Side, Size, TimeInForce,
    ///     adapters::UpbitAdapter,
    /// };
    /// use rust_decimal::Decimal;
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(UpbitAdapter::new().with_credentials("access", "secret"));
    /// let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
    ///
    /// let order = client
    ///     .place_order(
    ///         &OrderRequest::limit(
    ///             market,
    ///             Side::Buy,
    ///             // Base sizing: 0.001 BTC. `Size::Quote` would mean 0.001 KRW.
    ///             Size::Base(Decimal::new(1, 3)),
    ///             Decimal::from(100_000_000),
    ///         )
    ///         // Post-only: if the price has moved, the order is rejected.
    ///         .time_in_force(TimeInForce::PostOnly),
    ///     )
    ///     .await?;
    ///
    /// // Accepted is not filled: the status says which, and the id is what
    /// // cancels it.
    /// println!("{} is {:?}", order.id, order.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn place_order(&self, request: &OrderRequest) -> Result<Order> {
        self.adapter.place_order(request).await
    }

    /// Cancels an order.
    ///
    /// Requires credentials.
    ///
    /// ```no_run
    /// use maxt::{Client, Exchange, Market, OrderStatus, adapters::UpbitAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(UpbitAdapter::new().with_credentials("access", "secret"));
    /// let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
    ///
    /// let cancelled = client.cancel_order(&market, "order-id-from-place-order").await?;
    ///
    /// // A cancel races the book. Part of the order may have filled on the way,
    /// // so trust the returned order.
    /// if cancelled.status == OrderStatus::Filled {
    ///     println!("filled before the cancel landed");
    /// } else {
    ///     println!("{} filled, {} withdrawn", cancelled.filled_quantity, cancelled.remaining_quantity);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel_order(&self, market: &Market, order_id: &str) -> Result<Order> {
        self.adapter.cancel_order(market, order_id).await
    }

    /// Reads every open position.
    ///
    /// Requires credentials. Derivatives markets only.
    ///
    /// A row with no size is not an open position and never reaches the caller,
    /// whatever the venue publishes and whichever adapter is underneath. The
    /// drop happens here rather than in each adapter, so an adapter written
    /// outside this crate answers the same way.
    ///
    /// ```no_run
    /// use maxt::{Client, adapters::HyperliquidAdapter};
    /// use rust_decimal::Decimal;
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(HyperliquidAdapter::new().with_wallet("0xaddress", "0xkey"));
    ///
    /// let mut exposure = Decimal::ZERO;
    /// for position in client.positions().await? {
    ///     // Every one carries size, so there is nothing to skip. `None` means
    ///     // the exchange did not publish the figure, and summing it as zero
    ///     // would understate the account's exposure.
    ///     exposure += position.notional.unwrap_or_default();
    /// }
    /// println!("{exposure} at risk");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn positions(&self) -> Result<Vec<Position>> {
        Ok(open_positions(self.adapter.positions(None).await?))
    }

    /// Reads the open position on one market.
    ///
    /// Requires credentials. Derivatives markets only.
    ///
    /// A market the account holds nothing on answers an empty list rather than
    /// one flat position, on the same terms as [`Client::positions`].
    ///
    /// ```
    /// use maxt::{Client, Error, Exchange, Feature, Market, adapters::UpbitAdapter};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::new(UpbitAdapter::new());
    ///     let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
    ///
    ///     // Upbit lists no derivatives at all, so this is structural: no key
    ///     // and no argument makes it work. That is what `Unsupported` reports.
    ///     assert!(!client.supports(Feature::Positions));
    ///     let answer = client.positions_on(&market).await;
    ///     assert!(matches!(answer, Err(Error::Unsupported { feature: Feature::Positions, .. })));
    /// }
    /// ```
    pub async fn positions_on(&self, market: &Market) -> Result<Vec<Position>> {
        Ok(open_positions(self.adapter.positions(Some(market)).await?))
    }

    /// Reads account-wide margin state.
    ///
    /// Requires credentials. Derivatives markets only.
    ///
    /// ```no_run
    /// use maxt::{Client, adapters::BinanceAdapter};
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(
    ///     BinanceAdapter::usd_m_futures().with_credentials("key", "secret"),
    /// );
    ///
    /// let margin = client.margin_summary().await?;
    ///
    /// // Every figure is optional because not every exchange publishes all of
    /// // them. A sizing rule that reads a missing one as zero opens nothing,
    /// // and one that reads it as unlimited opens far too much.
    /// match margin.available_balance {
    ///     Some(free) => println!("{free} {} free to commit", margin.asset),
    ///     None => println!("{} does not publish free margin", client.exchange()),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn margin_summary(&self) -> Result<MarginSummary> {
        self.adapter.margin_summary().await
    }

    /// Reads a market's funding rate history, one page at a time.
    ///
    /// Public: this is a property of the market, not of any account.
    ///
    /// ```no_run
    /// use maxt::{Client, Exchange, HistoryRequest, Market, Timestamp, adapters::BinanceAdapter};
    /// use rust_decimal::Decimal;
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(BinanceAdapter::usd_m_futures());
    /// let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
    ///
    /// let mut request = HistoryRequest::new(market)
    ///     .from(Timestamp::from_millis(1_700_000_000_000))
    ///     .limit(100);
    /// let mut total = Decimal::ZERO;
    ///
    /// // The cursor, not the item count, is what says whether more follows.
    /// loop {
    ///     let page = client.funding_rates(&request).await?;
    ///     total += page.items.iter().map(|rate| rate.rate).sum::<Decimal>();
    ///
    ///     let Some(next) = page.next else { break };
    ///     request = request.cursor(next);
    /// }
    /// println!("{total} paid per unit of notional over the window");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn funding_rates(&self, request: &HistoryRequest) -> Result<Page<FundingRate>> {
        self.adapter.funding_rates(request).await
    }

    /// Reads the account's funding payment history, one page at a time.
    ///
    /// Requires credentials. Unlike [`Client::funding_rates`], this is what the
    /// account was actually charged.
    ///
    /// ```no_run
    /// use maxt::{Client, Exchange, HistoryRequest, Market, adapters::HyperliquidAdapter};
    /// use rust_decimal::Decimal;
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(HyperliquidAdapter::new().with_wallet("0xaddress", "0xkey"));
    /// let market = Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC");
    ///
    /// let page = client.funding_payments(&HistoryRequest::new(market).limit(50)).await?;
    ///
    /// // Signed: negative is what the account paid out. Summing the absolute
    /// // values would turn a funding income into a funding cost.
    /// let net: Decimal = page.items.iter().map(|payment| payment.amount).sum();
    /// println!("{net} net funding over {} entries", page.items.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn funding_payments(&self, request: &HistoryRequest) -> Result<Page<FundingPayment>> {
        self.adapter.funding_payments(request).await
    }

    /// Sets leverage or margin mode on a market.
    ///
    /// Requires credentials. Derivatives markets only.
    ///
    /// ```no_run
    /// use maxt::{
    ///     Client, Exchange, MarginMode, MarginRequest, Market, adapters::BinanceAdapter,
    /// };
    /// use rust_decimal::Decimal;
    ///
    /// # async fn run() -> maxt::Result<()> {
    /// let client = Client::new(
    ///     BinanceAdapter::usd_m_futures().with_credentials("key", "secret"),
    /// );
    ///
    /// // Setting both in one request avoids the half-applied state that two
    /// // calls would leave behind.
    /// client
    ///     .set_margin(
    ///         &MarginRequest::new(Market::perpetual(Exchange::Binance, "BTC", "USDT"))
    ///             .leverage(Decimal::from(5))
    ///             .margin_mode(MarginMode::Isolated),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_margin(&self, request: &MarginRequest) -> Result<()> {
        self.adapter.set_margin(request).await
    }
}

impl<A: Adapter> From<A> for Client<A> {
    fn from(adapter: A) -> Self {
        Self::new(adapter)
    }
}

/// Drops the rows that carry no size.
///
/// [`Client::positions`] and [`Client::positions_on`] promise open positions,
/// and a zero-size row is something else. Binance opens one on
/// `/fapi/v3/positionRisk` for any symbol that merely carries a resting order,
/// measured 2026-07-31; Hyperliquid omits closed positions from
/// `assetPositions` and so has never been seen to publish one, but its parser
/// maps a zero `szi` rather than rejecting it, and a venue is free to change.
///
/// This runs on the common API rather than in each adapter, which is what makes
/// the promise hold for every adapter, including one written outside this crate
/// against the public [`Adapter`] trait. An adapter stays free to report what
/// its venue said.
///
/// A row that fails to parse is untouched: the adapter's `Result` has already
/// resolved to `Err` by here, so a malformed row is still reported rather than
/// filtered away.
pub(crate) fn open_positions(mut positions: Vec<Position>) -> Vec<Position> {
    positions.retain(|position| !position.is_flat());
    positions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapter::BoxFuture;
    use crate::{Decimal, Error, Side};

    #[derive(Debug, Clone)]
    struct PublicOnly;

    impl Adapter for PublicOnly {
        fn exchange(&self) -> Exchange {
            Exchange::Bithumb
        }

        fn supports(&self, feature: Feature) -> bool {
            !feature.needs_credentials()
        }

        fn markets(&self, kind: MarketKind) -> BoxFuture<'_, Result<Vec<MarketInfo>>> {
            let empty = matches!(kind, MarketKind::Perpetual);
            Box::pin(async move {
                Ok(if empty {
                    vec![]
                } else {
                    vec![MarketInfo {
                        market: Market::spot(Exchange::Bithumb, "BTC", "KRW"),
                        native_symbol: "BTC_KRW".to_string(),
                        status: crate::MarketStatus::Active,
                        korean_name: None,
                        english_name: None,
                    }]
                })
            })
        }
    }

    #[test]
    fn supports_answers_without_a_network_call() {
        let client = Client::new(PublicOnly);

        assert!(client.supports(Feature::Ticker));
        assert!(!client.supports(Feature::Trading));
        assert_eq!(client.exchange(), Exchange::Bithumb);
    }

    #[tokio::test]
    async fn a_spot_exchange_reports_no_perpetuals_rather_than_an_error() {
        let client = Client::new(PublicOnly);

        assert_eq!(client.markets(MarketKind::Spot).await.unwrap().len(), 1);
        assert!(
            client
                .markets(MarketKind::Perpetual)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn private_calls_on_a_public_client_name_the_missing_feature() {
        let client = Client::new(PublicOnly);

        let error = client.balances().await.unwrap_err();
        assert!(matches!(
            error,
            Error::Unsupported {
                feature: Feature::Balances,
                exchange: "bithumb",
                ..
            }
        ));
    }

    /// An adapter that hands back exactly what its venue published, flat rows
    /// included. Every shipped adapter maps a zero-size row rather than
    /// rejecting it, and the [`Adapter`] trait is implementable from outside
    /// this crate, so this is the shape the common API has to hold.
    #[derive(Debug, Clone)]
    struct ReportsWhatTheVenueSaid;

    impl ReportsWhatTheVenueSaid {
        fn market(quote: &str) -> Market {
            Market::perpetual(Exchange::Binance, "BTC", quote)
        }

        fn position(quantity: Decimal, quote: &str) -> Position {
            Position {
                market: Self::market(quote),
                side: if quantity.is_zero() {
                    None
                } else {
                    Some(Side::Buy)
                },
                quantity,
                entry_price: None,
                mark_price: None,
                notional: Some(Decimal::from(30_000)),
                unrealized_pnl: None,
                leverage: None,
                margin_mode: None,
            }
        }
    }

    impl Adapter for ReportsWhatTheVenueSaid {
        fn exchange(&self) -> Exchange {
            Exchange::Binance
        }

        fn supports(&self, _feature: Feature) -> bool {
            true
        }

        fn positions(&self, _market: Option<&Market>) -> BoxFuture<'_, Result<Vec<Position>>> {
            Box::pin(async move {
                Ok(vec![
                    Self::position(Decimal::ZERO, "USDT"),
                    Self::position(Decimal::ONE, "USDC"),
                ])
            })
        }
    }

    /// `positions()` promises open positions, and a row with no size is not
    /// one. The drop lives on the common API rather than in an adapter, so it
    /// holds for an adapter written outside this crate too, which is the only
    /// place a crate-wide guarantee can hold.
    #[tokio::test]
    async fn a_flat_row_an_adapter_reports_is_not_answered_as_an_open_position() {
        let client = Client::new(ReportsWhatTheVenueSaid);

        // The adapter really did report two, so an answer of one below is the
        // filter rather than the adapter having nothing to say.
        assert_eq!(client.adapter().positions(None).await.unwrap().len(), 2);

        let open = client.positions().await.unwrap();
        assert_eq!(open.len(), 1, "a flat row was answered as an open position");
        assert_eq!(open[0].quantity, Decimal::ONE);

        // The narrowed read is the same promise, not a looser one.
        let narrowed = client
            .positions_on(&ReportsWhatTheVenueSaid::market("USDT"))
            .await
            .unwrap();
        assert_eq!(narrowed.len(), 1, "{narrowed:?}");
        assert!(!narrowed[0].is_flat(), "{narrowed:?}");
    }

    #[tokio::test]
    async fn clients_over_boxed_adapters_share_one_type() {
        let clients: Vec<Client<Box<dyn Adapter>>> = vec![Client::new(Box::new(PublicOnly) as _)];

        for client in &clients {
            assert!(!client.markets(MarketKind::Spot).await.unwrap().is_empty());
        }
    }
}
