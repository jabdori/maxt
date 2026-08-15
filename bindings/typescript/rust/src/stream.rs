use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_core::Stream;
use futures_util::StreamExt;
use maxt::{AccountStream, Error, MarketStream};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{Mutex, OnceCell, watch};

use crate::convert::{WireError, account_stream_item, market_stream_item};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WireStreamHandle {
    pub(crate) id: String,
    pub(crate) kind: String,
}

pub(crate) struct NativeStreamRegistry {
    next_id: AtomicU64,
    streams: Mutex<HashMap<String, Arc<NativeSubscription>>>,
}

impl Default for NativeStreamRegistry {
    fn default() -> Self {
        Self {
            next_id: AtomicU64::new(1),
            streams: Mutex::new(HashMap::new()),
        }
    }
}

impl NativeStreamRegistry {
    pub(crate) async fn insert_market(
        &self,
        stream: MarketStream,
    ) -> maxt::Result<WireStreamHandle> {
        self.insert(NativeStream::Market(stream), "market").await
    }

    pub(crate) async fn insert_account(
        &self,
        stream: AccountStream,
    ) -> maxt::Result<WireStreamHandle> {
        self.insert(NativeStream::Account(stream), "account").await
    }

    /// Registers a provider-specific stream without teaching this shared
    /// runtime about an exchange's event enum.
    pub(crate) async fn insert_provider<S, E>(
        &self,
        stream: S,
        kind: &'static str,
        item: fn(maxt::Result<E>) -> maxt::Result<Value>,
    ) -> maxt::Result<WireStreamHandle>
    where
        S: Stream<Item = maxt::Result<E>> + ProviderStreamClose + Send + Unpin + 'static,
        E: Send + 'static,
    {
        self.insert(
            NativeStream::Provider(Box::new(TypedProviderStream { stream, item })),
            kind,
        )
        .await
    }

    async fn insert(
        &self,
        stream: NativeStream,
        kind: &'static str,
    ) -> maxt::Result<WireStreamHandle> {
        let id = self
            .next_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| Error::adapter("native stream ID space is exhausted"))?
            .to_string();
        self.streams
            .lock()
            .await
            .insert(id.clone(), Arc::new(NativeSubscription::new(stream)));
        Ok(WireStreamHandle {
            id,
            kind: kind.to_owned(),
        })
    }

    pub(crate) async fn next(&self, id: &str) -> maxt::Result<Option<Value>> {
        let subscription =
            self.streams
                .lock()
                .await
                .get(id)
                .cloned()
                .ok_or_else(|| Error::InvalidRequest {
                    field: "stream_id".to_owned(),
                    detail: format!("unknown native stream `{id}`"),
                })?;
        let item = subscription.next().await?;
        if item.is_none() && !subscription.close_requested() {
            let mut streams = self.streams.lock().await;
            if streams
                .get(id)
                .is_some_and(|stored| Arc::ptr_eq(stored, &subscription))
            {
                streams.remove(id);
            }
        }
        Ok(item)
    }

    pub(crate) async fn close(&self, id: &str) -> maxt::Result<()> {
        let subscription = self.streams.lock().await.get(id).cloned();
        let Some(subscription) = subscription else {
            return Ok(());
        };
        let result = subscription.close().await;
        let mut streams = self.streams.lock().await;
        if streams
            .get(id)
            .is_some_and(|stored| Arc::ptr_eq(stored, &subscription))
        {
            streams.remove(id);
        }
        result
    }

    #[cfg(test)]
    async fn len(&self) -> usize {
        self.streams.lock().await.len()
    }
}

struct NativeSubscription {
    inner: Mutex<Option<NativeStream>>,
    closed: watch::Sender<bool>,
    close_result: OnceCell<maxt::Result<()>>,
}

impl NativeSubscription {
    fn new(stream: NativeStream) -> Self {
        let (closed, _) = watch::channel(false);
        Self {
            inner: Mutex::new(Some(stream)),
            closed,
            close_result: OnceCell::new(),
        }
    }

    async fn next(&self) -> maxt::Result<Option<Value>> {
        let mut closed = self.closed.subscribe();
        if *closed.borrow() {
            return Ok(None);
        }
        let mut inner = self.inner.lock().await;
        let Some(stream) = inner.as_mut() else {
            return Ok(None);
        };
        let item = match stream {
            NativeStream::Market(stream) => {
                let item = tokio::select! {
                    biased;
                    _ = closed.changed() => return Ok(None),
                    item = stream.next() => item,
                };
                item.map(market_stream_item)
                    .map(serde_json::to_value)
                    .transpose()
                    .map_err(|error| {
                        Error::adapter(format!("could not serialize stream item: {error}"))
                    })
            }
            NativeStream::Account(stream) => {
                let item = tokio::select! {
                    biased;
                    _ = closed.changed() => return Ok(None),
                    item = stream.next() => item,
                };
                item.map(account_stream_item)
                    .map(serde_json::to_value)
                    .transpose()
                    .map_err(|error| {
                        Error::adapter(format!("could not serialize stream item: {error}"))
                    })
            }
            NativeStream::Provider(stream) => {
                tokio::select! {
                    biased;
                    _ = closed.changed() => return Ok(None),
                    item = stream.next() => item,
                }
            }
        }?;
        if item.is_none() {
            inner.take();
        }
        Ok(item)
    }

    fn close_requested(&self) -> bool {
        *self.closed.borrow()
    }

    async fn close(&self) -> maxt::Result<()> {
        self.close_result
            .get_or_init(|| self.close_inner())
            .await
            .clone()
    }

    async fn close_inner(&self) -> maxt::Result<()> {
        self.closed.send_replace(true);
        let mut inner = self.inner.lock().await;
        let result = match inner.as_mut() {
            Some(NativeStream::Market(stream)) => stream.close().await,
            Some(NativeStream::Account(stream)) => stream.close().await,
            Some(NativeStream::Provider(stream)) => stream.close().await,
            None => Ok(()),
        };
        inner.take();
        result
    }
}

enum NativeStream {
    Market(MarketStream),
    Account(AccountStream),
    Provider(Box<dyn ProviderStream>),
}

/// The six exchange-specific detailed stream types all share this one native
/// lifecycle. Generated code supplies only the strongly typed event converter.
pub(crate) trait ProviderStreamClose {
    fn close_provider(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = maxt::Result<()>> + Send + '_>>;
}

trait ProviderStream: Send {
    fn next(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = maxt::Result<Option<Value>>> + Send + '_>>;

    fn close(&mut self) -> Pin<Box<dyn Future<Output = maxt::Result<()>> + Send + '_>>;
}

struct TypedProviderStream<S, E> {
    stream: S,
    item: fn(maxt::Result<E>) -> maxt::Result<Value>,
}

impl<S, E> ProviderStream for TypedProviderStream<S, E>
where
    S: Stream<Item = maxt::Result<E>> + ProviderStreamClose + Send + Unpin + 'static,
    E: Send + 'static,
{
    fn next(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = maxt::Result<Option<Value>>> + Send + '_>> {
        Box::pin(async move {
            match self.stream.next().await {
                Some(item) => (self.item)(item).map(Some),
                None => Ok(None),
            }
        })
    }

    fn close(&mut self) -> Pin<Box<dyn Future<Output = maxt::Result<()>> + Send + '_>> {
        self.stream.close_provider()
    }
}

fn error_item(error: Error) -> Value {
    json!({"kind": "error", "error": WireError::from(error)})
}

/// Converts a typed provider event into the JSON envelope shared by all
/// TypeScript detailed streams.
pub(crate) fn provider_stream_item<E, W>(item: maxt::Result<E>) -> maxt::Result<Value>
where
    W: TryFrom<E, Error = Error> + Serialize,
{
    match item {
        Ok(event) => serde_json::to_value(W::try_from(event)?)
            .map(|event| json!({"kind": "event", "event": event}))
            .map_err(|error| Error::adapter(format!("could not serialize provider stream item: {error}"))),
        Err(error) => Ok(error_item(error)),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::task::{Context, Poll};
    use std::time::Duration;

    use futures_core::Stream;
    use futures_util::stream;
    use maxt::{AccountEvent, AccountStream, Error, MarketEvent, MarketStream};
    use serde::Serialize;
    use tokio::sync::oneshot;

    use super::*;

    struct ProviderFixture {
        events: VecDeque<maxt::Result<String>>,
    }

    impl Stream for ProviderFixture {
        type Item = maxt::Result<String>;

        fn poll_next(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Self::Item>> {
            Poll::Ready(self.events.pop_front())
        }
    }

    impl ProviderStreamClose for ProviderFixture {
        fn close_provider(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = maxt::Result<()>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }
    }

    #[derive(Serialize)]
    struct ProviderWire(String);

    impl TryFrom<String> for ProviderWire {
        type Error = Error;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            Ok(Self(value))
        }
    }

    #[tokio::test]
    async fn generated_provider_streams_share_one_registry_lifecycle() {
        let registry = NativeStreamRegistry::default();
        let handle = registry
            .insert_provider(
                ProviderFixture {
                    events: VecDeque::from([
                        Ok("first".to_owned()),
                        Err(Error::Transport {
                            detail: "temporary".to_owned(),
                        }),
                        Ok("last".to_owned()),
                    ]),
                },
                "provider",
                provider_stream_item::<String, ProviderWire>,
            )
            .await
            .unwrap();

        assert_eq!(handle.kind, "provider");
        assert_eq!(registry.next(&handle.id).await.unwrap().unwrap()["kind"], "event");
        assert_eq!(registry.next(&handle.id).await.unwrap().unwrap()["kind"], "error");
        assert_eq!(registry.next(&handle.id).await.unwrap().unwrap()["kind"], "event");
        assert!(registry.next(&handle.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn registry_keeps_errors_non_terminal_and_removes_natural_end() {
        let registry = NativeStreamRegistry::default();
        let handle = registry
            .insert_market(MarketStream::new(stream::iter([
                Ok(MarketEvent::Reconnected),
                Err(Error::Transport {
                    detail: "socket closed".to_owned(),
                }),
                Ok(MarketEvent::Reconnected),
            ])))
            .await
            .unwrap();

        assert_eq!(handle.id, "1");
        assert_eq!(handle.kind, "market");
        assert_eq!(registry.len().await, 1);
        assert_eq!(
            registry.next(&handle.id).await.unwrap().unwrap()["kind"],
            "event"
        );
        assert_eq!(
            registry.next(&handle.id).await.unwrap().unwrap()["kind"],
            "error"
        );
        assert_eq!(
            registry.next(&handle.id).await.unwrap().unwrap()["kind"],
            "event"
        );
        assert!(registry.next(&handle.id).await.unwrap().is_none());
        assert_eq!(registry.len().await, 0);

        registry.close(&handle.id).await.unwrap();
        assert!(matches!(
            registry.next(&handle.id).await,
            Err(Error::InvalidRequest { field, .. }) if field == "stream_id"
        ));
    }

    struct PendingAccount;

    impl Stream for PendingAccount {
        type Item = maxt::Result<AccountEvent>;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Pending
        }
    }

    #[tokio::test]
    async fn close_wakes_pending_account_next_and_waits_for_source_ack() {
        let released = Arc::new(AtomicBool::new(false));
        let observed = Arc::clone(&released);
        let registry = Arc::new(NativeStreamRegistry::default());
        let handle = registry
            .insert_account(AccountStream::new_with_close(
                PendingAccount,
                move || async move {
                    observed.store(true, Ordering::SeqCst);
                    Ok(())
                },
            ))
            .await
            .unwrap();
        assert_eq!(handle.kind, "account");

        let pending = tokio::spawn({
            let registry = Arc::clone(&registry);
            let id = handle.id.clone();
            async move { registry.next(&id).await }
        });
        tokio::task::yield_now().await;

        tokio::time::timeout(Duration::from_millis(200), registry.close(&handle.id))
            .await
            .expect("streamClose must wake a pending streamNext")
            .unwrap();
        assert!(released.load(Ordering::SeqCst));
        assert!(pending.await.unwrap().unwrap().is_none());
        registry.close(&handle.id).await.unwrap();
        assert_eq!(registry.len().await, 0);
    }

    #[tokio::test]
    async fn concurrent_close_callers_wait_for_and_receive_the_same_cleanup_error() {
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let registry = Arc::new(NativeStreamRegistry::default());
        let handle = registry
            .insert_account(AccountStream::new_with_close(
                PendingAccount,
                move || async move {
                    let _ = started_tx.send(());
                    let _ = release_rx.await;
                    Err(Error::Transport {
                        detail: "cleanup failed".to_owned(),
                    })
                },
            ))
            .await
            .unwrap();

        let first = tokio::spawn({
            let registry = Arc::clone(&registry);
            let id = handle.id.clone();
            async move { registry.close(&id).await }
        });
        started_rx.await.unwrap();

        let mut second = tokio::spawn({
            let registry = Arc::clone(&registry);
            let id = handle.id.clone();
            async move { registry.close(&id).await }
        });
        assert!(
            tokio::time::timeout(Duration::from_millis(20), &mut second)
                .await
                .is_err(),
            "a concurrent close must wait for the in-flight cleanup"
        );

        release_tx.send(()).unwrap();
        let first_error = first.await.unwrap().unwrap_err();
        let second_error = second.await.unwrap().unwrap_err();
        assert_eq!(first_error, second_error);
        assert_eq!(registry.len().await, 0);
    }

    #[tokio::test]
    async fn cancelled_close_keeps_the_cleanup_available_for_retry() {
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let registry = Arc::new(NativeStreamRegistry::default());
        let handle = registry
            .insert_account(AccountStream::new_with_close(
                PendingAccount,
                move || async move {
                    let _ = started_tx.send(());
                    let _ = release_rx.await;
                    Ok(())
                },
            ))
            .await
            .unwrap();

        let first = tokio::spawn({
            let registry = Arc::clone(&registry);
            let id = handle.id.clone();
            async move { registry.close(&id).await }
        });
        started_rx.await.unwrap();
        first.abort();
        assert!(first.await.unwrap_err().is_cancelled());
        assert_eq!(registry.len().await, 1);

        let mut retry = tokio::spawn({
            let registry = Arc::clone(&registry);
            let id = handle.id.clone();
            async move { registry.close(&id).await }
        });
        assert!(
            tokio::time::timeout(Duration::from_millis(20), &mut retry)
                .await
                .is_err(),
            "the retry must resume and await the original cleanup"
        );
        release_tx.send(()).unwrap();
        retry.await.unwrap().unwrap();
        assert_eq!(registry.len().await, 0);
    }

    #[tokio::test]
    async fn close_wakeup_does_not_remove_the_registry_entry_before_cleanup_finishes() {
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let registry = Arc::new(NativeStreamRegistry::default());
        let handle = registry
            .insert_account(AccountStream::new_with_close(
                PendingAccount,
                move || async move {
                    let _ = started_tx.send(());
                    let _ = release_rx.await;
                    Ok(())
                },
            ))
            .await
            .unwrap();

        let pending = tokio::spawn({
            let registry = Arc::clone(&registry);
            let id = handle.id.clone();
            async move { registry.next(&id).await }
        });
        tokio::task::yield_now().await;
        let closing = tokio::spawn({
            let registry = Arc::clone(&registry);
            let id = handle.id.clone();
            async move { registry.close(&id).await }
        });

        started_rx.await.unwrap();
        assert!(pending.await.unwrap().unwrap().is_none());
        assert_eq!(registry.len().await, 1);

        release_tx.send(()).unwrap();
        closing.await.unwrap().unwrap();
        assert_eq!(registry.len().await, 0);
    }
}
