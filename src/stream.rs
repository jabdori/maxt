//! The streams returned by live subscriptions.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use crate::error::Result;
use crate::types::{AccountEvent, MarketEvent};

type CloseFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
type CloseHook = Box<dyn FnOnce() -> CloseFuture + Send + 'static>;

fn poll_stream<T>(
    inner: &mut Option<Pin<Box<dyn Stream<Item = Result<T>> + Send>>>,
    close: &mut Option<CloseHook>,
    closing: &mut Option<CloseFuture>,
    cx: &mut Context<'_>,
) -> Poll<Option<Result<T>>> {
    if closing.is_none() {
        match inner.as_mut() {
            Some(source) => match source.as_mut().poll_next(cx) {
                Poll::Ready(None) => {
                    inner.take();
                    *closing = close.take().map(|close| close());
                }
                polled => return polled,
            },
            None => return Poll::Ready(None),
        }
    }

    let Some(cleanup) = closing.as_mut() else {
        return Poll::Ready(None);
    };
    match cleanup.as_mut().poll(cx) {
        Poll::Pending => Poll::Pending,
        Poll::Ready(result) => {
            *closing = None;
            inner.take();
            match result {
                Ok(()) => Poll::Ready(None),
                Err(error) => Poll::Ready(Some(Err(error))),
            }
        }
    }
}

/// Shared close and polling state for a typed built-in stream.
///
/// This is crate-private so provider-specific streams can preserve the same
/// cancellation and reconnect cleanup semantics without widening the public
/// common-stream API.
pub(crate) struct TypedStream<T> {
    inner: Option<Pin<Box<dyn Stream<Item = Result<T>> + Send>>>,
    close: Option<CloseHook>,
    closing: Option<CloseFuture>,
}

impl<T> TypedStream<T> {
    pub(crate) fn new(inner: impl Stream<Item = Result<T>> + Send + 'static) -> Self {
        Self {
            inner: Some(Box::pin(inner)),
            close: None,
            closing: None,
        }
    }

    pub(crate) fn new_with_close<F, Fut>(
        inner: impl Stream<Item = Result<T>> + Send + 'static,
        close: F,
    ) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            inner: Some(Box::pin(inner)),
            close: Some(Box::new(move || Box::pin(close()))),
            closing: None,
        }
    }

    pub(crate) async fn close(&mut self) -> Result<()> {
        if self.closing.is_none() {
            self.closing = self.close.take().map(|close| close());
        }
        let result = match self.closing.as_mut() {
            Some(closing) => closing.await,
            None => Ok(()),
        };
        self.closing = None;
        self.inner.take();
        result
    }
}

impl<T> Stream for TypedStream<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();
        poll_stream(&mut this.inner, &mut this.close, &mut this.closing, cx)
    }
}

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
    inner: TypedStream<MarketEvent>,
}

impl MarketStream {
    /// Wraps an adapter's own event source as a `MarketStream`.
    ///
    /// The only way to build one, so an [`Adapter`](crate::Adapter) written
    /// outside this crate needs it to return from
    /// [`Adapter::subscribe`](crate::Adapter::subscribe).
    ///
    /// The inner stream is polled unchanged. Its producer handles reconnects
    /// and emits [`MarketEvent::Reconnected`].
    pub fn new(inner: impl Stream<Item = Result<MarketEvent>> + Send + 'static) -> Self {
        Self {
            inner: TypedStream::new(inner),
        }
    }

    /// Wraps an event source with cleanup that natural exhaustion and explicit
    /// [`Self::close`] await.
    ///
    /// Dropping the stream still drops `inner` immediately. Use `close` when the
    /// producer must confirm asynchronous cleanup, such as a foreign runtime
    /// cancelling its subscription task.
    pub fn new_with_close<F, Fut>(
        inner: impl Stream<Item = Result<MarketEvent>> + Send + 'static,
        close: F,
    ) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            inner: TypedStream::new_with_close(inner, close),
        }
    }

    /// Stops this stream and waits for adapter-provided asynchronous cleanup.
    ///
    /// The source is dropped even when cleanup returns an error. Repeated calls
    /// are no-ops. If the caller cancels this future, the next call resumes the
    /// same cleanup future.
    pub async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}

impl Stream for MarketStream {
    type Item = Result<MarketEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();
        Pin::new(&mut this.inner).poll_next(cx)
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
    inner: TypedStream<AccountEvent>,
}

impl AccountStream {
    /// Wraps an adapter's own event source as an `AccountStream`.
    ///
    /// The only way to build one, so an [`Adapter`](crate::Adapter) written
    /// outside this crate needs it to return from
    /// [`Adapter::subscribe_account`](crate::Adapter::subscribe_account).
    ///
    /// The inner stream is polled unchanged. Its producer handles reconnects,
    /// emits [`AccountEvent::Reconnected`], and reports credential-renewal
    /// failures as `Err` items.
    pub fn new(inner: impl Stream<Item = Result<AccountEvent>> + Send + 'static) -> Self {
        Self {
            inner: TypedStream::new(inner),
        }
    }

    /// Wraps an event source with cleanup that natural exhaustion and explicit
    /// [`Self::close`] await.
    ///
    /// Dropping the stream still drops `inner` immediately. Use `close` when the
    /// producer must confirm asynchronous cleanup, such as a foreign runtime
    /// cancelling its subscription task.
    pub fn new_with_close<F, Fut>(
        inner: impl Stream<Item = Result<AccountEvent>> + Send + 'static,
        close: F,
    ) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Self {
            inner: TypedStream::new_with_close(inner, close),
        }
    }

    /// Stops this stream and waits for adapter-provided asynchronous cleanup.
    ///
    /// The source is dropped even when cleanup returns an error. Repeated calls
    /// are no-ops. If the caller cancels this future, the next call resumes the
    /// same cleanup future.
    pub async fn close(&mut self) -> Result<()> {
        self.inner.close().await
    }
}

impl Stream for AccountStream {
    type Item = Result<AccountEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();
        Pin::new(&mut this.inner).poll_next(cx)
    }
}

impl fmt::Debug for AccountStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountStream").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    use super::*;
    use futures_util::StreamExt;
    use futures_util::stream;

    struct PendingUntilDrop(Arc<AtomicBool>);

    impl futures_core::Stream for PendingUntilDrop {
        type Item = Result<MarketEvent>;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Pending
        }
    }

    impl Drop for PendingUntilDrop {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    struct CompletesThenDrops<T>(Arc<AtomicBool>, std::marker::PhantomData<T>);

    impl<T> futures_core::Stream for CompletesThenDrops<T> {
        type Item = T;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Ready(None)
        }
    }

    impl<T> Drop for CompletesThenDrops<T> {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn natural_completion_drops_market_and_account_sources_immediately() {
        let market_dropped = Arc::new(AtomicBool::new(false));
        let account_dropped = Arc::new(AtomicBool::new(false));
        let mut market = MarketStream::new(CompletesThenDrops(
            Arc::clone(&market_dropped),
            std::marker::PhantomData,
        ));
        let mut account = AccountStream::new(CompletesThenDrops(
            Arc::clone(&account_dropped),
            std::marker::PhantomData,
        ));

        assert!(market.next().await.is_none());
        assert!(account.next().await.is_none());
        assert!(market_dropped.load(Ordering::SeqCst));
        assert!(account_dropped.load(Ordering::SeqCst));
    }

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

    #[tokio::test]
    async fn explicit_close_awaits_the_hook_then_drops_the_source() {
        let dropped = Arc::new(AtomicBool::new(false));
        let hook_calls = Arc::new(AtomicUsize::new(0));
        let (release, released) = tokio::sync::oneshot::channel();
        let observed_calls = Arc::clone(&hook_calls);
        let mut stream = MarketStream::new_with_close(
            PendingUntilDrop(Arc::clone(&dropped)),
            move || async move {
                observed_calls.fetch_add(1, Ordering::SeqCst);
                let _ = released.await;
                Ok(())
            },
        );

        let close = tokio::spawn(async move {
            let result = stream.close().await;
            (stream, result)
        });
        while hook_calls.load(Ordering::SeqCst) == 0 {
            tokio::task::yield_now().await;
        }
        assert!(!dropped.load(Ordering::SeqCst));

        release.send(()).unwrap();
        let (mut stream, result) = close.await.unwrap();

        assert!(result.is_ok());
        assert!(dropped.load(Ordering::SeqCst));
        assert!(stream.next().await.is_none());
        assert!(stream.close().await.is_ok());
        assert_eq!(hook_calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_failed_close_hook_still_drops_the_account_source() {
        struct PendingAccount(Arc<AtomicBool>);

        impl futures_core::Stream for PendingAccount {
            type Item = Result<AccountEvent>;

            fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                Poll::Pending
            }
        }

        impl Drop for PendingAccount {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let dropped = Arc::new(AtomicBool::new(false));
        let mut stream =
            AccountStream::new_with_close(PendingAccount(Arc::clone(&dropped)), || async {
                Err(crate::Error::adapter("close failed"))
            });

        assert!(stream.close().await.is_err());
        assert!(dropped.load(Ordering::SeqCst));
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn a_cancelled_close_can_resume_the_same_cleanup_future() {
        let dropped = Arc::new(AtomicBool::new(false));
        let hook_calls = Arc::new(AtomicUsize::new(0));
        let (release, released) = tokio::sync::oneshot::channel();
        let observed_calls = Arc::clone(&hook_calls);
        let mut stream = MarketStream::new_with_close(
            PendingUntilDrop(Arc::clone(&dropped)),
            move || async move {
                observed_calls.fetch_add(1, Ordering::SeqCst);
                let _ = released.await;
                Ok(())
            },
        );

        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(10), stream.close())
                .await
                .is_err()
        );
        assert!(!dropped.load(Ordering::SeqCst));
        assert_eq!(hook_calls.load(Ordering::SeqCst), 1);

        release.send(()).unwrap();
        assert!(stream.close().await.is_ok());
        assert!(dropped.load(Ordering::SeqCst));
        assert_eq!(hook_calls.load(Ordering::SeqCst), 1);
    }
}
