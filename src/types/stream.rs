//! Live subscriptions: what to subscribe to, and what arrives.

use crate::types::{Balance, Candle, Interval, Market, Order, OrderBook, Ticker, Trade};

/// A kind of live market data to subscribe to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Feed {
    /// Every executed trade.
    Trades,
    /// Order book updates.
    OrderBook,
    /// Provider ticker summaries.
    Ticker,
    /// Candles at one interval.
    Candles(Interval),
}

/// What to subscribe to, across markets and feeds.
///
/// Every feed applies to every market, so this is a cross product, not a list
/// of pairs. Duplicate markets and feeds are removed while insertion order is
/// preserved.
///
/// An adapter normally uses one connection, but may split one subscription
/// across multiple endpoints. Reconnect settings and `Reconnected` events then
/// apply to each underlying connection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Subscription {
    markets: Vec<Market>,
    feeds: Vec<Feed>,
}

impl Subscription {
    /// An empty subscription.
    ///
    /// Add at least one market and one feed before subscribing. An empty
    /// subscription is rejected as
    /// [`Error::InvalidRequest`](crate::Error::InvalidRequest).
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a market. Adding the same market twice has no additional effect.
    #[must_use]
    pub fn market(mut self, market: Market) -> Self {
        if !self.markets.contains(&market) {
            self.markets.push(market);
        }
        self
    }

    /// Adds several markets.
    #[must_use]
    pub fn markets_iter(mut self, markets: impl IntoIterator<Item = Market>) -> Self {
        for market in markets {
            self = self.market(market);
        }
        self
    }

    /// Adds a feed. Adding the same feed twice has no additional effect.
    #[must_use]
    pub fn feed(mut self, feed: Feed) -> Self {
        if !self.feeds.contains(&feed) {
            self.feeds.push(feed);
        }
        self
    }

    /// The markets subscribed to, in the order they were added.
    pub fn markets(&self) -> &[Market] {
        &self.markets
    }

    /// The feeds subscribed to, in the order they were added.
    pub fn feeds(&self) -> &[Feed] {
        &self.feeds
    }
}

/// Something that arrived on a market data subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MarketEvent {
    /// A trade executed.
    Trade(Trade),
    /// The order book changed.
    OrderBook(OrderBook),
    /// A ticker was published.
    Ticker(Ticker),
    /// A candle opened, updated, or closed. See [`Candle::closed`].
    Candle(Candle),
    /// The connection dropped and was re-established, and the subscription was
    /// restored.
    ///
    /// Anything the exchange published while the connection was down was
    /// missed. Order book consumers should discard their local book and rebuild
    /// from the next snapshot.
    ///
    /// [`Overflow::DropNewest`] delays this until buffer capacity is available.
    Reconnected,
}

/// Something that arrived on a private account subscription.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AccountEvent {
    /// A balance changed.
    Balance(Balance),
    /// An order was placed, filled, or cancelled.
    Order(Order),
    /// The connection dropped and was re-established, and the subscription was
    /// restored.
    ///
    /// Account state changes published while the connection was down were
    /// missed. Re-read balances and open orders over REST to resynchronize.
    ///
    /// [`Overflow::DropNewest`] delays this until buffer capacity is available.
    Reconnected,
}

/// How a live connection should behave when it degrades.
///
/// The defaults reconnect without a count limit, use exponential backoff, and
/// apply [`Overflow::Backpressure`] when the buffer fills.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamConfig {
    /// Maximum number of reconnect attempts.
    ///
    /// Every reconnect counts; successful traffic does not reset the total.
    /// `None`, the default, retries without a count limit. When a finite limit
    /// is exhausted, the producer attempts to send [`Error::Transport`](crate::Error::Transport)
    /// and then ends. [`Overflow::DropNewest`] may drop that final error when
    /// the buffer is full, leaving `None` as the only termination signal.
    pub max_reconnect_attempts: Option<u32>,
    /// Delay before the first reconnect attempt, in milliseconds.
    ///
    /// Values below one millisecond are raised to one millisecond.
    pub initial_reconnect_delay_ms: u64,
    /// Ceiling on the reconnect backoff, in milliseconds.
    ///
    /// Values below one millisecond are raised to one millisecond.
    pub max_reconnect_delay_ms: u64,
    /// Close and reconnect if nothing arrives for this long, in milliseconds.
    ///
    /// An adapter may raise this to the minimum its exchange's heartbeat can
    /// satisfy. A larger caller-provided value is preserved.
    pub idle_timeout_ms: u64,
    /// How many events to buffer for a consumer that has fallen behind.
    ///
    /// Zero is raised to one.
    pub buffer_size: usize,
    /// What to do when the buffer is full.
    pub overflow: Overflow,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            max_reconnect_attempts: None,
            initial_reconnect_delay_ms: 1_000,
            max_reconnect_delay_ms: 30_000,
            idle_timeout_ms: 30_000,
            buffer_size: 4_096,
            overflow: Overflow::Backpressure,
        }
    }
}

/// What a live connection does when its consumer falls behind.
///
/// There is no drop-oldest policy. A consumer that needs newest-wins behavior
/// must drain the stream and retain the latest complete snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Overflow {
    /// Stop reading from the socket until the consumer catches up.
    ///
    /// No events are intentionally dropped. Heartbeats continue while delivery
    /// waits, though a sufficiently long socket stall may still be closed by the
    /// exchange.
    Backpressure,
    /// Discard arriving events while the buffer is full.
    ///
    /// Suitable for complete snapshots such as [`Feed::Ticker`] and
    /// [`Feed::OrderBook`]. It is lossy for trades and candles: later events do
    /// not restate a trade or a candle's single settled emission.
    ///
    /// Reconnected events wait for capacity; other items, including errors, may
    /// be discarded.
    DropNewest,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Exchange;

    #[test]
    fn duplicate_markets_and_feeds_collapse() {
        let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
        let subscription = Subscription::new()
            .market(market.clone())
            .market(market)
            .feed(Feed::Trades)
            .feed(Feed::Trades);

        assert_eq!(subscription.markets().len(), 1);
        assert_eq!(subscription.feeds().len(), 1);
    }

    #[test]
    fn candle_feeds_at_different_intervals_are_different_feeds() {
        let subscription = Subscription::new()
            .feed(Feed::Candles(Interval::Min1))
            .feed(Feed::Candles(Interval::Hour1));

        assert_eq!(subscription.feeds().len(), 2);
    }

    #[test]
    fn insertion_order_is_preserved() {
        let btc = Market::spot(Exchange::Upbit, "BTC", "KRW");
        let eth = Market::spot(Exchange::Upbit, "ETH", "KRW");
        let subscription = Subscription::new()
            .markets_iter([btc.clone(), eth.clone()])
            .feed(Feed::Ticker)
            .feed(Feed::Trades);

        assert_eq!(subscription.markets(), [btc, eth]);
        assert_eq!(subscription.feeds(), [Feed::Ticker, Feed::Trades]);
    }

    #[test]
    fn defaults_reconnect_forever_and_lose_nothing() {
        let config = StreamConfig::default();

        assert_eq!(config.max_reconnect_attempts, None);
        assert_eq!(config.overflow, Overflow::Backpressure);
    }
}
