//! The streams returned by live subscriptions.

use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use crate::error::Result;
use crate::types::{AccountEvent, MarketEvent};

/// A live market data subscription.
///
/// Yields [`MarketEvent`]s until it ends with `None`. Built-in adapters handle
/// reconnects and emit [`MarketEvent::Reconnected`]; custom producers define
/// their own reconnect behavior.
///
/// An `Err` item reports a failed frame or connection operation but does not by
/// itself end the stream. Consumers may continue polling after it.
///
/// `None` is the termination signal. It can follow exhaustion of
/// [`StreamConfig::max_reconnect_attempts`](crate::StreamConfig::max_reconnect_attempts).
/// Under [`Overflow::DropNewest`](crate::Overflow::DropNewest), the final error
/// may be dropped when the buffer is full, so consumers must also handle `None`
/// without a preceding error.
///
/// Dropping this value drops its inner stream. The built-in adapters use that
/// signal to stop their connection tasks; a custom stream controls its own
/// cleanup.
pub struct MarketStream {
    inner: Pin<Box<dyn Stream<Item = Result<MarketEvent>> + Send>>,
}

impl MarketStream {
    /// Wraps an adapter's own event source as a `MarketStream`.
    ///
    /// The only way to build one, so an [`Adapter`](crate::Adapter) written
    /// outside this crate needs it to return from
    /// [`Adapter::subscribe`](crate::Adapter::subscribe).
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
/// Yields [`AccountEvent`]s until it ends with `None`. Built-in adapters handle
/// reconnects and emit [`AccountEvent::Reconnected`]; custom producers define
/// their own reconnect behavior.
///
/// An `Err` item is a report, not the end, exactly as on [`MarketStream`].
/// Errors may include frame decoding, reconnect failures, and credential
/// renewal failures. `None` has the same termination semantics as
/// [`MarketStream`]. Dropping this value drops its inner stream; cleanup is the
/// producer's responsibility.
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
