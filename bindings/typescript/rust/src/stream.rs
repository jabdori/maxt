use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::StreamExt;
use maxt::{AccountStream, Error, MarketStream};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::{Mutex, watch};

use crate::convert::{account_stream_item, market_stream_item};

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
        if item.is_none() {
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
        let subscription = self.streams.lock().await.remove(id);
        match subscription {
            Some(subscription) => subscription.close().await,
            None => Ok(()),
        }
    }

    #[cfg(test)]
    async fn len(&self) -> usize {
        self.streams.lock().await.len()
    }
}

struct NativeSubscription {
    inner: Mutex<Option<NativeStream>>,
    closed: watch::Sender<bool>,
}

impl NativeSubscription {
    fn new(stream: NativeStream) -> Self {
        let (closed, _) = watch::channel(false);
        Self {
            inner: Mutex::new(Some(stream)),
            closed,
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
            }
        }
        .map_err(|error| Error::adapter(format!("could not serialize stream item: {error}")))?;
        if item.is_none() {
            inner.take();
        }
        Ok(item)
    }

    async fn close(&self) -> maxt::Result<()> {
        self.closed.send_replace(true);
        let mut inner = self.inner.lock().await;
        let result = match inner.as_mut() {
            Some(NativeStream::Market(stream)) => stream.close().await,
            Some(NativeStream::Account(stream)) => stream.close().await,
            None => Ok(()),
        };
        inner.take();
        result
    }
}

enum NativeStream {
    Market(MarketStream),
    Account(AccountStream),
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::task::{Context, Poll};
    use std::time::Duration;

    use futures_core::Stream;
    use futures_util::stream;
    use maxt::{AccountEvent, AccountStream, Error, MarketEvent, MarketStream};

    use super::*;

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
}
