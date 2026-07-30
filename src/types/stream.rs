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
    /// Rolling 24-hour summaries.
    Ticker,
    /// Candles at one interval.
    Candles(Interval),
}

/// What to subscribe to, across markets and feeds.
///
/// One subscription becomes one connection per exchange, whatever the number of
/// markets and feeds. Build it once and hand it to
/// [`Client::subscribe`](crate::Client::subscribe).
///
/// Every feed applies to every market, so this is a cross product, not a list
/// of pairs. Three markets and three feeds is nine streams over one socket.
///
/// ```
/// use maxt::{Exchange, Feed, Interval, Market, Subscription};
///
/// let majors = ["BTC", "ETH", "XRP"]
///     .map(|base| Market::spot(Exchange::Upbit, base, "KRW"));
///
/// let subscription = Subscription::new()
///     .markets_iter(majors)
///     .feed(Feed::Trades)
///     .feed(Feed::OrderBook)
///     .feed(Feed::Candles(Interval::Min1))
///     // Candles at another interval are a different feed, not a replacement.
///     .feed(Feed::Candles(Interval::Hour1))
///     // Adding the same feed twice is not an error and costs nothing.
///     .feed(Feed::Trades);
///
/// assert_eq!(subscription.markets().len(), 3);
/// assert_eq!(subscription.feeds().len(), 4);
/// // Insertion order is kept, which is the order the exchange is asked in.
/// assert_eq!(subscription.feeds()[0], Feed::Trades);
/// ```
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
    /// [`Overflow::DropNewest`] never discards this. A full buffer delays it to
    /// the first event that finds room, and it still arrives ahead of that
    /// event.
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
    /// [`Overflow::DropNewest`] never discards this. A full buffer delays it to
    /// the first event that finds room, and it still arrives ahead of that
    /// event, so the instruction to re-read is not one a slow consumer can miss.
    Reconnected,
}

/// How a live connection should behave when it degrades.
///
/// The defaults reconnect forever with exponential backoff and never drop
/// events. Override them when a stalled consumer is worse than a lossy one.
///
/// The struct is exhaustive, so all six fields may be named and
/// `..StreamConfig::default()` is a convenience rather than a requirement. The
/// price of that is paid on the crate's side: a seventh field would break every
/// construction that names them all, so one arrives with a new major version
/// and not before.
///
/// ```
/// use maxt::{Overflow, StreamConfig};
///
/// // Update the fields that differ and inherit the rest.
/// let config = StreamConfig {
///     // Ends the stream after five attempts, so a process supervisor sees
///     // the failure.
///     max_reconnect_attempts: Some(5),
///     // Which policy is right depends on the feed. See [`Overflow`].
///     overflow: Overflow::DropNewest,
///     ..StreamConfig::default()
/// };
///
/// assert_eq!(config.max_reconnect_attempts, Some(5));
/// // Untouched, and still the defaults: one second, backing off to thirty.
/// assert_eq!(config.initial_reconnect_delay_ms, 1_000);
/// assert_eq!(config.max_reconnect_delay_ms, 30_000);
/// assert_eq!(config.buffer_size, StreamConfig::default().buffer_size);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamConfig {
    /// Give up after this many reconnects, whatever came of them.
    ///
    /// Every reconnect counts and nothing resets the count: one that never
    /// opened, one that opened a socket the exchange stayed mute on, and one
    /// that opened a socket the exchange sent frames on. The connection reads
    /// raw frames and parses none of them, so a rejected symbol, a retired
    /// stream name, a revoked credential on a private stream and an HTML error
    /// from a gateway all arrive as frames it cannot tell from data. A budget
    /// any frame reset would put no bound at all on the venue that answers
    /// every connection with one.
    ///
    /// The price is that healthy reconnects spend it too. A venue that recycles
    /// working sockets on its own schedule runs a finite budget down doing so,
    /// which makes `Some(n)` a way to have a process supervisor see a failure
    /// rather than a way to survive a venue's housekeeping.
    ///
    /// No delay field enters into it: none of them changes what counts here.
    ///
    /// `None`, the default, retries forever, and that includes retrying a venue
    /// that rejects every connection. Such a venue keeps being reconnected to at
    /// [`initial_reconnect_delay_ms`](StreamConfig::initial_reconnect_delay_ms)
    /// for as long as the process lives, because the rejection is a frame and
    /// so resets the backoff. Reading it is the consumer's to do, and setting
    /// this field is what bounds it. When the limit is reached the stream ends
    /// with [`Error::Transport`](crate::Error::Transport).
    pub max_reconnect_attempts: Option<u32>,
    /// Delay before the first reconnect attempt, in milliseconds.
    ///
    /// Floored at one millisecond, because zero doubles to zero forever and
    /// leaves a flapping socket reconnecting as fast as a core allows.
    pub initial_reconnect_delay_ms: u64,
    /// Ceiling on the reconnect backoff, in milliseconds.
    ///
    /// Floored at one millisecond, for the reason
    /// [`initial_reconnect_delay_ms`](StreamConfig::initial_reconnect_delay_ms)
    /// is. Raising it makes the retries gentler and nothing else: it does not
    /// decide which reconnects count against
    /// [`max_reconnect_attempts`](StreamConfig::max_reconnect_attempts).
    pub max_reconnect_delay_ms: u64,
    /// Close and reconnect if nothing arrives for this long, in milliseconds.
    ///
    /// Treated as a minimum. An adapter whose exchange keeps healthy
    /// connections quieter than this raises it to what that exchange's own
    /// pace can meet. A Binance user data stream on an account that never
    /// moves is server-pinged every three minutes and is healthy the whole
    /// time, so the 30 seconds asked for by default would tear down and
    /// rebuild a working socket forever. Asking for longer than an adapter's
    /// floor is always honoured.
    pub idle_timeout_ms: u64,
    /// How many events to buffer for a consumer that has fallen behind.
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
/// There is no "drop the oldest" policy, because the sending side cannot evict
/// from the front of a full queue. A consumer that wants newest-wins semantics,
/// as a ticker or a book snapshot does, gets them by draining the stream in a
/// tight loop and keeping only the last event it saw.
///
/// The question to ask of a feed is not whether one event matters on its own,
/// but whether a later event on that same feed restates it. A ticker and a book
/// snapshot are restated by the next one. A trade never is. Neither is the one
/// candle emission that carries a window's final figures, which is why candles
/// belong with trades here and not with tickers.
///
/// # Examples
///
/// ```
/// use maxt::{Feed, Interval, Overflow, StreamConfig};
///
/// /// What a slow consumer should lose on each kind of feed.
/// fn overflow_for(feed: Feed) -> Overflow {
///     match feed {
///         // A ticker and a book event each restate one value in full, so
///         // dropping some costs staleness and the next event cures it.
///         // Stalling instead holds up every other feed on the connection.
///         Feed::Ticker | Feed::OrderBook => Overflow::DropNewest,
///         // A trade is a distinct fact no later event repeats, and a settled
///         // candle is the only emission carrying its window's final figures.
///         // Losing either is silent and permanent, so pay in latency.
///         _ => Overflow::Backpressure,
///     }
/// }
///
/// assert_eq!(overflow_for(Feed::Ticker), Overflow::DropNewest);
/// assert_eq!(overflow_for(Feed::Trades), Overflow::Backpressure);
/// assert_eq!(overflow_for(Feed::Candles(Interval::Min1)), Overflow::Backpressure);
/// // Losing nothing is the default: it cannot corrupt a total silently.
/// assert_eq!(StreamConfig::default().overflow, Overflow::Backpressure);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Overflow {
    /// Stop reading from the socket until the consumer catches up.
    ///
    /// Loses nothing, and reading is the only thing it stops: the connection's
    /// heartbeat keeps going out while the consumer is waited on, so a stall is
    /// not read by the exchange as a dead peer. A stall long enough to fill the
    /// buffers between the two ends is still one the exchange may act on.
    Backpressure,
    /// Discard arriving events while the buffer is full.
    ///
    /// Right for a feed whose next event restates the whole of what the dropped
    /// one said: [`Feed::Ticker`] and [`Feed::OrderBook`], where every event is
    /// a complete current value and staleness is the only cost.
    ///
    /// Wrong for [`Feed::Trades`]. Each trade is a distinct fact no later event
    /// repeats, so a total computed from what survives is silently short.
    ///
    /// Wrong for [`Feed::Candles`] too, which is less obvious. A forming bar is
    /// restated by the next frame, but the settled emission is not: a window
    /// gets exactly one event with [`Candle::closed`] set, carrying that
    /// window's own final figures, and everything after it belongs to the next
    /// window. Dropping it loses that bar outright, with no counter and no
    /// error, and the series is short one bar wherever it is stored. Use
    /// [`Overflow::Backpressure`] for trades and candles alike, and pay in
    /// latency.
    ///
    /// What this never discards is
    /// [`MarketEvent::Reconnected`](crate::MarketEvent::Reconnected) and
    /// [`AccountEvent::Reconnected`](crate::AccountEvent::Reconnected). A full
    /// buffer delays that notice rather than losing it: it is held and
    /// delivered ahead of the first event that finds room, because a consumer
    /// that never heard of the gap goes on trusting a book and a balance the
    /// gap invalidated, and nothing later corrects it. Everything else,
    /// including a reconnect failure reported as an error, is discarded
    /// silently.
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
