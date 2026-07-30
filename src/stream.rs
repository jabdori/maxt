//! The streams returned by live subscriptions.

use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use crate::error::Result;
use crate::types::{AccountEvent, MarketEvent};

/// A live market data subscription.
///
/// Yields [`MarketEvent`]s until it ends by yielding `None`. Reconnects are
/// handled internally and surface as [`MarketEvent::Reconnected`].
///
/// An `Err` item is a report, not the end. A frame that could not be read and
/// a reconnect that has stopped looking transient are both delivered as errors
/// while the stream goes on polling. A consumer that breaks out of its loop on
/// the first `Err` abandons a subscription that was about to recover.
///
/// `None` is the only thing that means nothing more is coming. It arrives after
/// [`StreamConfig::max_reconnect_attempts`](crate::StreamConfig::max_reconnect_attempts)
/// runs out, or once the connection has been dropped.
///
/// Dropping the stream closes the connection.
///
/// ```no_run
/// use futures_util::StreamExt;
/// use maxt::{Client, Exchange, Feed, Market, MarketEvent, MarketStream, Subscription};
/// use maxt::adapters::BithumbAdapter;
///
/// /// Prices from a live book, until the stream says it is finished.
/// async fn watch(mut stream: MarketStream) -> usize {
///     let mut books = 0;
///     let mut consecutive_errors = 0;
///
///     while let Some(event) = stream.next().await {
///         match event {
///             Ok(MarketEvent::OrderBook(book)) => {
///                 books += 1;
///                 consecutive_errors = 0;
///                 let _ = book.mid_price();
///             }
///             // The local book is missing whatever arrived while the socket
///             // was down, so it is rebuilt from this snapshot on.
///             Ok(MarketEvent::Reconnected) => books = 0,
///             Ok(_) => {}
///             Err(error) => {
///                 // A budget, not a `break`. The stream keeps trying, and
///                 // may recover on the next frame.
///                 consecutive_errors += 1;
///                 eprintln!("{consecutive_errors}: {error}");
///             }
///         }
///     }
///
///     // Reached only when the stream yielded `None`, the one signal that
///     // means nothing more is coming.
///     books
/// }
///
/// # async fn run() -> maxt::Result<()> {
/// let subscription = Subscription::new()
///     .market(Market::spot(Exchange::Bithumb, "BTC", "KRW"))
///     .feed(Feed::OrderBook);
/// let stream = Client::new(BithumbAdapter::new()).subscribe(&subscription).await?;
///
/// println!("{} books before the stream ended", watch(stream).await);
/// # Ok(())
/// # }
/// ```
pub struct MarketStream {
    inner: Pin<Box<dyn Stream<Item = Result<MarketEvent>> + Send>>,
}

impl MarketStream {
    /// Wraps an adapter's own event source as a `MarketStream`.
    ///
    /// The only way to build one, so an [`Adapter`](crate::Adapter) written
    /// outside this crate needs it to return from
    /// [`Adapter::subscribe`](crate::Adapter::subscribe). A mock, a backtester,
    /// and a harness replaying recorded frames all arrive here.
    ///
    /// What is handed over is polled unchanged. Reconnecting, and announcing it
    /// with [`MarketEvent::Reconnected`], belong to whatever produces the
    /// events; this type adds neither.
    pub fn new(inner: impl Stream<Item = Result<MarketEvent>> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(inner),
        }
    }
}

impl Stream for MarketStream {
    type Item = Result<MarketEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

impl fmt::Debug for MarketStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MarketStream").finish_non_exhaustive()
    }
}

/// A live private account subscription.
///
/// Yields [`AccountEvent`]s until it ends by yielding `None`. Reconnects are
/// handled internally and surface as [`AccountEvent::Reconnected`].
///
/// An `Err` item is a report, not the end, exactly as on [`MarketStream`].
/// Three things arrive as errors while the stream goes on polling: a frame that
/// could not be read, a reconnect that has stopped looking transient, and a
/// failed credential renewal on the exchanges whose private sockets hold one.
/// `None` is the only thing that means nothing more is coming.
///
/// Dropping the stream closes the connection.
///
/// ```no_run
/// use std::collections::HashMap;
///
/// use futures_util::StreamExt;
/// use maxt::{AccountEvent, AccountStream, Client, Order, StreamConfig};
/// use maxt::adapters::UpbitAdapter;
///
/// /// Keeps a table of live orders in step with the exchange.
/// async fn track(mut stream: AccountStream) -> HashMap<String, Order> {
///     let mut live = HashMap::new();
///
///     while let Some(event) = stream.next().await {
///         match event {
///             Ok(AccountEvent::Order(order)) if order.status.is_live() => {
///                 live.insert(order.id.clone(), order);
///             }
///             Ok(AccountEvent::Order(order)) => {
///                 live.remove(&order.id);
///             }
///             // Whatever changed while the socket was down never arrived, so
///             // the table no longer describes the account. Clear it and
///             // re-read over REST.
///             Ok(AccountEvent::Reconnected) => live.clear(),
///             Ok(_) => {}
///             // Includes a failed credential renewal on the exchanges whose
///             // private socket holds one. The stream keeps polling.
///             Err(error) => eprintln!("{error}"),
///         }
///     }
///
///     live
/// }
///
/// # async fn run() -> maxt::Result<()> {
/// let client = Client::new(UpbitAdapter::new().with_credentials("access", "secret"));
/// // With a reconnect budget the stream can end on its own. Without one it
/// // ends only when it is dropped.
/// let config = StreamConfig { max_reconnect_attempts: Some(5), ..StreamConfig::default() };
///
/// let remaining = track(client.subscribe_account_with(&config).await?).await;
/// println!("{} orders still open when the stream ended", remaining.len());
/// # Ok(())
/// # }
/// ```
pub struct AccountStream {
    inner: Pin<Box<dyn Stream<Item = Result<AccountEvent>> + Send>>,
}

impl AccountStream {
    /// Wraps an adapter's own event source as an `AccountStream`.
    ///
    /// The only way to build one, so an [`Adapter`](crate::Adapter) written
    /// outside this crate needs it to return from
    /// [`Adapter::subscribe_account`](crate::Adapter::subscribe_account).
    ///
    /// What is handed over is polled unchanged. Reconnecting, renewing a
    /// credential the exchange's private socket holds, and announcing either
    /// with [`AccountEvent::Reconnected`], belong to whatever produces the
    /// events; this type adds none of them.
    pub fn new(inner: impl Stream<Item = Result<AccountEvent>> + Send + 'static) -> Self {
        Self {
            inner: Box::pin(inner),
        }
    }
}

impl Stream for AccountStream {
    type Item = Result<AccountEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

impl fmt::Debug for AccountStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountStream").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use futures_util::stream;

    #[tokio::test]
    async fn a_market_stream_yields_what_the_adapter_produced() {
        let events = vec![Ok(MarketEvent::Reconnected), Ok(MarketEvent::Reconnected)];
        let mut market_stream = MarketStream::new(stream::iter(events));

        assert!(matches!(
            market_stream.next().await,
            Some(Ok(MarketEvent::Reconnected))
        ));
        assert!(market_stream.next().await.is_some());
        assert!(market_stream.next().await.is_none());
    }

    #[tokio::test]
    async fn an_err_is_an_item_the_stream_polls_past_rather_than_its_end() {
        // The documented contract: only `None` ends a stream, so an error in
        // the middle must not swallow what comes after it.
        let events = vec![
            Err(crate::Error::decode("a frame that could not be read")),
            Ok(AccountEvent::Reconnected),
        ];
        let mut account_stream = AccountStream::new(stream::iter(events));

        assert!(matches!(account_stream.next().await, Some(Err(_))));
        assert!(matches!(
            account_stream.next().await,
            Some(Ok(AccountEvent::Reconnected))
        ));
        assert!(account_stream.next().await.is_none());
    }
}
