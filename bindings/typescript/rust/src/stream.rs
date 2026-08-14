use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use futures_util::StreamExt;
use maxt::{
    AccountStream, Error, HyperliquidAccountEvent, HyperliquidAccountStream,
    HyperliquidMarketEvent, HyperliquidMarketStream, MarketStream,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{Mutex, OnceCell, watch};

use crate::convert::{
    WireBalance, WireCandle, WireHyperliquidL2Book, WireHyperliquidRecentTrade,
    WireHyperliquidSpotBalance, WireOrder, WireOrderBook, WireTicker, WireTrade, WireError,
    account_stream_item, decimal_to_wire, market_stream_item, timestamp_to_wire,
};

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

    pub(crate) async fn insert_hyperliquid_market(
        &self,
        stream: HyperliquidMarketStream,
    ) -> maxt::Result<WireStreamHandle> {
        self.insert(NativeStream::HyperliquidMarket(stream), "hyperliquid_market")
            .await
    }

    pub(crate) async fn insert_hyperliquid_account(
        &self,
        stream: HyperliquidAccountStream,
    ) -> maxt::Result<WireStreamHandle> {
        self.insert(
            NativeStream::HyperliquidAccount(stream),
            "hyperliquid_account",
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
            NativeStream::HyperliquidMarket(stream) => {
                let item = tokio::select! {
                    biased;
                    _ = closed.changed() => return Ok(None),
                    item = stream.next() => item,
                };
                item.map(hyperliquid_market_stream_item)
                    .transpose()
                    .map_err(|error| {
                        Error::adapter(format!("could not serialize Hyperliquid stream item: {error}"))
                    })
            }
            NativeStream::HyperliquidAccount(stream) => {
                let item = tokio::select! {
                    biased;
                    _ = closed.changed() => return Ok(None),
                    item = stream.next() => item,
                };
                item.map(hyperliquid_account_stream_item)
                    .transpose()
                    .map_err(|error| {
                        Error::adapter(format!("could not serialize Hyperliquid stream item: {error}"))
                    })
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
            Some(NativeStream::HyperliquidMarket(stream)) => stream.close().await,
            Some(NativeStream::HyperliquidAccount(stream)) => stream.close().await,
            None => Ok(()),
        };
        inner.take();
        result
    }
}

enum NativeStream {
    Market(MarketStream),
    Account(AccountStream),
    HyperliquidMarket(HyperliquidMarketStream),
    HyperliquidAccount(HyperliquidAccountStream),
}

fn wire_value(value: impl Serialize) -> maxt::Result<Value> {
    serde_json::to_value(value)
        .map_err(|error| Error::adapter(format!("could not serialize native stream item: {error}")))
}

fn error_item(error: Error) -> Value {
    json!({"kind": "error", "error": WireError::from(error)})
}

fn hyperliquid_market_stream_item(item: maxt::Result<HyperliquidMarketEvent>) -> maxt::Result<Value> {
    match item {
        Ok(event) => hyperliquid_market_event_value(event)
            .map(|event| json!({"kind": "event", "event": event}))
            .or_else(|error| Ok(error_item(error))),
        Err(error) => Ok(error_item(error)),
    }
}

fn hyperliquid_account_stream_item(
    item: maxt::Result<HyperliquidAccountEvent>,
) -> maxt::Result<Value> {
    match item {
        Ok(event) => hyperliquid_account_event_value(event)
            .map(|event| json!({"kind": "event", "event": event}))
            .or_else(|error| Ok(error_item(error))),
        Err(error) => Ok(error_item(error)),
    }
}

fn hyperliquid_market_event_value(event: HyperliquidMarketEvent) -> maxt::Result<Value> {
    match event {
        HyperliquidMarketEvent::Trade(value) => Ok(json!({
            "kind": "trade",
            "value": {
                "common": wire_value(WireTrade::try_from(value.common)?)?,
                "provider": wire_value(WireHyperliquidRecentTrade::try_from(value.provider)?)?,
            },
        })),
        HyperliquidMarketEvent::OrderBook(value) => Ok(json!({
            "kind": "order_book",
            "value": {
                "common": wire_value(WireOrderBook::try_from(value.common)?)?,
                "provider": wire_value(WireHyperliquidL2Book::try_from(value.provider)?)?,
            },
        })),
        HyperliquidMarketEvent::Candle(value) => Ok(json!({
            "kind": "candle",
            "value": {
                "common": wire_value(WireCandle::try_from(value.common)?)?,
                "provider": wire_value(crate::convert::WireHyperliquidCandleSnapshot::try_from(value.provider)?)?,
            },
        })),
        HyperliquidMarketEvent::AssetContext(value) => Ok(json!({
            "kind": "asset_context",
            "value": {
                "common": wire_value(WireTicker::try_from(value.common)?)?,
                "coin": value.coin,
                "mid_price": value.mid_price.map(decimal_to_wire),
                "mark_price": value.mark_price.map(decimal_to_wire),
                "previous_day_price": value.previous_day_price.map(decimal_to_wire),
                "day_base_volume": value.day_base_volume.map(decimal_to_wire),
                "day_notional_volume": value.day_notional_volume.map(decimal_to_wire),
                "oracle_price": value.oracle_price.map(decimal_to_wire),
                "funding_rate": value.funding_rate.map(decimal_to_wire),
                "open_interest": value.open_interest.map(decimal_to_wire),
                "circulating_supply": value.circulating_supply.map(decimal_to_wire),
                "total_supply": value.total_supply.map(decimal_to_wire),
                "raw_json": value.raw_json,
            },
        })),
        HyperliquidMarketEvent::Reconnected => Ok(json!({"kind": "reconnected"})),
        _ => Err(Error::adapter(
            "native Hyperliquid market stream returned an unknown event variant",
        )),
    }
}

fn hyperliquid_account_event_value(event: HyperliquidAccountEvent) -> maxt::Result<Value> {
    match event {
        HyperliquidAccountEvent::OrderUpdate(value) => Ok(json!({
            "kind": "order_update",
            "value": {
                "common": wire_value(WireOrder::try_from(value.common)?)?,
                "coin": value.coin,
                "side": value.side,
                "limit_price": decimal_to_wire(value.limit_price),
                "remaining_size": decimal_to_wire(value.remaining_size),
                "original_size": decimal_to_wire(value.original_size),
                "order_id": value.order_id.to_string(),
                "accepted_at": timestamp_to_wire(value.accepted_at),
                "client_order_id": value.client_order_id,
                "status": value.status,
                "status_at": value.status_at.map(timestamp_to_wire),
                "raw_json": value.raw_json,
            },
        })),
        HyperliquidAccountEvent::SpotState(value) => {
            let balances = value
                .balances
                .into_iter()
                .map(|balance| {
                    Ok(json!({
                        "common": wire_value(WireBalance::try_from(balance.common)?)?,
                        "provider": wire_value(WireHyperliquidSpotBalance::try_from(balance.provider)?)?,
                    }))
                })
                .collect::<maxt::Result<Vec<_>>>()?;
            Ok(json!({
                "kind": "spot_state",
                "value": {
                    "user": value.user,
                    "balances": balances,
                    "raw_json": value.raw_json,
                },
            }))
        }
        HyperliquidAccountEvent::Reconnected => Ok(json!({"kind": "reconnected"})),
        _ => Err(Error::adapter(
            "native Hyperliquid account stream returned an unknown event variant",
        )),
    }
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
    use tokio::sync::oneshot;

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
