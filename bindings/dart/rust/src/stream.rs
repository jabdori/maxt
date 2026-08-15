use std::collections::VecDeque;
use std::fmt;
use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use flutter_rust_bridge::{BaseAsyncRuntime, rust_async::RwLock};
use futures_channel::oneshot;
use futures_core::Stream;
use maxt::{
    AccountEvent, AccountStream, Error, HyperliquidAccountEvent, HyperliquidAccountStream,
    HyperliquidMarketEvent, HyperliquidMarketStream, MarketEvent, MarketStream, Overflow, Result,
    StreamConfig,
};

use crate::convert::{
    NativeError, WireBalance, WireCandle, WireHyperliquidAccountEvent, WireHyperliquidMarketEvent,
    WireOrder, WireOrderBook, WireTicker, WireTrade,
};

const OPEN: u8 = 0;
const ENDED: u8 = 1;
const CANCELLED: u8 = 2;

pub(crate) type CancelFuture = Pin<Box<dyn Future<Output = Result<()>> + Send + 'static>>;
pub(crate) type CancelCallback = Arc<dyn Fn(String) -> CancelFuture + Send + Sync>;

/// Market-stream event that can be delivered to Dart.
#[derive(Debug)]
pub enum WireMarketEvent {
    /// Market trade event.
    Trade(WireTrade),
    /// Market order-book snapshot event.
    OrderBook(WireOrderBook),
    /// Market ticker event.
    Ticker(WireTicker),
    /// Market candle event.
    Candle(WireCandle),
    /// Indicates that receipt resumed after reconnection.
    Reconnected,
}

/// Account-stream event that can be delivered to Dart.
#[derive(Debug)]
pub enum WireAccountEvent {
    /// Account balance event.
    Balance(WireBalance),
    /// Account order event.
    Order(WireOrder),
    /// Indicates that receipt resumed after reconnection.
    Reconnected,
}

/// Tagged item sent to a market-stream sink.
#[derive(Debug)]
pub enum MarketStreamItem {
    /// Normal market event.
    Event(WireMarketEvent),
    /// Non-terminal stream error.
    Error(NativeError),
    /// Natural end of the market stream.
    End,
}

/// Tagged item sent to an account-stream sink.
#[derive(Debug)]
pub enum AccountStreamItem {
    /// Normal account event.
    Event(WireAccountEvent),
    /// Non-terminal stream error.
    Error(NativeError),
    /// Natural end of the account stream.
    End,
}

/// Tagged item read by Dart from a Rust market subscription.
#[derive(Debug)]
pub enum WireMarketStreamItem {
    /// Normal market event.
    Event(WireMarketEvent),
    /// Non-terminal stream error.
    Error(NativeError),
    /// Natural end or explicit close of the market stream.
    End,
}

/// Tagged item read by Dart from a Rust account subscription.
#[derive(Debug)]
pub enum WireAccountStreamItem {
    /// Normal account event.
    Event(WireAccountEvent),
    /// Non-terminal stream error.
    Error(NativeError),
    /// Natural end or explicit close of the account stream.
    End,
}

/// Event, error, or end item from a Hyperliquid-native market stream.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum WireHyperliquidMarketStreamItem {
    /// Normal Hyperliquid market event.
    Event(WireHyperliquidMarketEvent),
    /// Non-terminal stream error.
    Error(NativeError),
    /// Natural end or explicit close.
    End,
}

/// Event, error, or end item from a Hyperliquid-native account stream.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum WireHyperliquidAccountStreamItem {
    /// Normal Hyperliquid account event.
    Event(WireHyperliquidAccountEvent),
    /// Non-terminal stream error.
    Error(NativeError),
    /// Natural end or explicit close.
    End,
}

struct ForwardTask {
    abort: Mutex<Option<oneshot::Sender<()>>>,
    completion: Mutex<Option<oneshot::Receiver<Result<()>>>>,
    result: Mutex<Option<Result<()>>>,
}

impl ForwardTask {
    fn abort(&self) {
        if let Some(abort) = self.abort.lock().unwrap().take() {
            let _ = abort.send(());
        }
    }

    async fn close(&self) -> Result<()> {
        self.abort();
        if let Some(result) = self.result.lock().unwrap().clone() {
            return result;
        }
        let completion = { self.completion.lock().unwrap().take() };
        let result = match completion {
            Some(completion) => completion.await.unwrap_or_else(|_| {
                Err(Error::adapter(
                    "native stream forwarding task stopped before cleanup completed",
                ))
            }),
            None => Ok(()),
        };
        *self.result.lock().unwrap() = Some(result.clone());
        result
    }
}

struct PullState<T> {
    inner: Mutex<PullInner<T>>,
}

struct PullInner<T> {
    item: Option<T>,
    receiver_waker: Option<Waker>,
    producer_waker: Option<Waker>,
    closed: bool,
}

impl<T> PullState<T> {
    fn new() -> Self {
        Self {
            inner: Mutex::new(PullInner {
                item: None,
                receiver_waker: None,
                producer_waker: None,
                closed: false,
            }),
        }
    }

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<bool> {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed {
            Poll::Ready(false)
        } else if inner.item.is_none() {
            Poll::Ready(true)
        } else {
            inner.producer_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }

    fn send_ready(&self, item: T) -> bool {
        let receiver = {
            let mut inner = self.inner.lock().unwrap();
            if inner.closed || inner.item.is_some() {
                return false;
            }
            inner.item = Some(item);
            inner.receiver_waker.take()
        };
        if let Some(receiver) = receiver {
            receiver.wake();
        }
        true
    }

    async fn recv(&self) -> Option<T> {
        poll_fn(|cx| {
            let (item, producer) = {
                let mut inner = self.inner.lock().unwrap();
                if let Some(item) = inner.item.take() {
                    (Some(Some(item)), inner.producer_waker.take())
                } else if inner.closed {
                    (Some(None), None)
                } else {
                    inner.receiver_waker = Some(cx.waker().clone());
                    (None, None)
                }
            };
            if let Some(producer) = producer {
                producer.wake();
            }
            match item {
                Some(item) => Poll::Ready(item),
                None => Poll::Pending,
            }
        })
        .await
    }

    fn close(&self) {
        let (receiver, producer) = {
            let mut inner = self.inner.lock().unwrap();
            inner.closed = true;
            inner.item.take();
            (inner.receiver_waker.take(), inner.producer_waker.take())
        };
        if let Some(receiver) = receiver {
            receiver.wake();
        }
        if let Some(producer) = producer {
            producer.wake();
        }
    }
}

struct NativeSubscription<T> {
    output: Arc<PullState<T>>,
    receiver: RwLock<()>,
    task: ForwardTask,
}

impl<T> NativeSubscription<T> {
    async fn next(&self) -> Option<T> {
        let _receiver = self.receiver.write().await;
        self.output.recv().await
    }

    async fn close(&self) -> Result<()> {
        self.task.close().await
    }
}

impl<T> Drop for NativeSubscription<T> {
    fn drop(&mut self) {
        self.task.abort();
    }
}

enum ForwardPoll<T> {
    Source(Option<T>),
    Ready,
    Aborted,
}

trait AsyncClose {
    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

impl AsyncClose for MarketStream {
    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(MarketStream::close(self))
    }
}

impl AsyncClose for AccountStream {
    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(AccountStream::close(self))
    }
}

impl AsyncClose for HyperliquidMarketStream {
    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(HyperliquidMarketStream::close(self))
    }
}

impl AsyncClose for HyperliquidAccountStream {
    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(HyperliquidAccountStream::close(self))
    }
}

fn spawn_subscription<S, T, U>(
    source: S,
    mut convert: impl FnMut(Result<T>) -> U + Send + 'static,
) -> NativeSubscription<U>
where
    S: Stream<Item = Result<T>> + AsyncClose + Send + Unpin + 'static,
    T: Send + 'static,
    U: Send + 'static,
{
    let output = Arc::new(PullState::new());
    let (abort_sender, mut abort_receiver) = oneshot::channel();
    let (completion_sender, completion_receiver) = oneshot::channel();

    let forwarding_output = output.clone();
    let forwarding = async move {
        let mut source = source;
        let mut aborted = false;
        loop {
            let ready = poll_fn(|cx| {
                if Pin::new(&mut abort_receiver).poll(cx).is_ready() {
                    return Poll::Ready(ForwardPoll::<Result<T>>::Aborted);
                }
                forwarding_output.poll_ready(cx).map(|ready| {
                    if ready {
                        ForwardPoll::Ready
                    } else {
                        ForwardPoll::Aborted
                    }
                })
            })
            .await;
            if matches!(ready, ForwardPoll::Aborted) {
                aborted = true;
                break;
            }

            let polled = poll_fn(|cx| {
                if Pin::new(&mut abort_receiver).poll(cx).is_ready() {
                    return Poll::Ready(ForwardPoll::Aborted);
                }
                Pin::new(&mut source).poll_next(cx).map(ForwardPoll::Source)
            })
            .await;

            match polled {
                ForwardPoll::Source(Some(item)) => {
                    if !forwarding_output.send_ready(convert(item)) {
                        break;
                    }
                }
                ForwardPoll::Source(None) => break,
                ForwardPoll::Aborted => {
                    aborted = true;
                    break;
                }
                ForwardPoll::Ready => unreachable!(),
            }
        }

        let result = if aborted {
            source.close().await
        } else {
            Ok(())
        };
        forwarding_output.close();
        let _ = completion_sender.send(result);
    };
    std::mem::drop(
        crate::frb_generated::FLUTTER_RUST_BRIDGE_HANDLER
            .async_runtime()
            .spawn(forwarding),
    );

    NativeSubscription {
        output,
        receiver: RwLock::new(()),
        task: ForwardTask {
            abort: Mutex::new(Some(abort_sender)),
            completion: Mutex::new(Some(completion_receiver)),
            result: Mutex::new(None),
        },
    }
}

/// Handle that preserves Rust market-subscription lifetime and non-terminal errors for Dart.
#[flutter_rust_bridge::frb(opaque)]
pub struct NativeMarketSubscription {
    inner: NativeSubscription<WireMarketStreamItem>,
}

impl NativeMarketSubscription {
    pub(crate) fn new(stream: MarketStream) -> Self {
        Self {
            inner: spawn_subscription(stream, market_stream_item),
        }
    }

    /// Returns the next event or error, and `End` after natural completion or `close`.
    pub async fn next(&self) -> WireMarketStreamItem {
        self.inner.next().await.unwrap_or(WireMarketStreamItem::End)
    }

    /// Stops delivery and waits for the source Rust stream to be dropped.
    pub async fn close(&self) -> std::result::Result<(), NativeError> {
        self.inner.close().await.map_err(Into::into)
    }
}

/// Handle that preserves Rust account-subscription lifetime and non-terminal errors for Dart.
#[flutter_rust_bridge::frb(opaque)]
pub struct NativeAccountSubscription {
    inner: NativeSubscription<WireAccountStreamItem>,
}

/// Handle that preserves Hyperliquid-native market subscription lifetime and non-terminal errors for Dart.
#[flutter_rust_bridge::frb(opaque)]
pub struct NativeHyperliquidMarketSubscription {
    inner: NativeSubscription<WireHyperliquidMarketStreamItem>,
}

impl NativeHyperliquidMarketSubscription {
    pub(crate) fn new(stream: HyperliquidMarketStream) -> Self {
        Self {
            inner: spawn_subscription(stream, hyperliquid_market_stream_item),
        }
    }

    /// Returns the next event or error, and `End` after natural completion or `close`.
    pub async fn next(&self) -> WireHyperliquidMarketStreamItem {
        self.inner
            .next()
            .await
            .unwrap_or(WireHyperliquidMarketStreamItem::End)
    }

    /// Stops delivery and waits for the source Rust stream to be dropped.
    pub async fn close(&self) -> std::result::Result<(), NativeError> {
        self.inner.close().await.map_err(Into::into)
    }
}

/// Handle that preserves Hyperliquid-native account subscription lifetime and non-terminal errors for Dart.
#[flutter_rust_bridge::frb(opaque)]
pub struct NativeHyperliquidAccountSubscription {
    inner: NativeSubscription<WireHyperliquidAccountStreamItem>,
}

impl NativeHyperliquidAccountSubscription {
    pub(crate) fn new(stream: HyperliquidAccountStream) -> Self {
        Self {
            inner: spawn_subscription(stream, hyperliquid_account_stream_item),
        }
    }

    /// Returns the next event or error, and `End` after natural completion or `close`.
    pub async fn next(&self) -> WireHyperliquidAccountStreamItem {
        self.inner
            .next()
            .await
            .unwrap_or(WireHyperliquidAccountStreamItem::End)
    }

    /// Stops delivery and waits for the source Rust stream to be dropped.
    pub async fn close(&self) -> std::result::Result<(), NativeError> {
        self.inner.close().await.map_err(Into::into)
    }
}

impl NativeAccountSubscription {
    pub(crate) fn new(stream: AccountStream) -> Self {
        Self {
            inner: spawn_subscription(stream, account_stream_item),
        }
    }

    /// Returns the next event or error, and `End` after natural completion or `close`.
    pub async fn next(&self) -> WireAccountStreamItem {
        self.inner
            .next()
            .await
            .unwrap_or(WireAccountStreamItem::End)
    }

    /// Stops delivery and waits for the source Rust stream to be dropped.
    pub async fn close(&self) -> std::result::Result<(), NativeError> {
        self.inner.close().await.map_err(Into::into)
    }
}

fn market_stream_item(item: Result<MarketEvent>) -> WireMarketStreamItem {
    match item {
        Ok(event) => match WireMarketEvent::try_from(event) {
            Ok(event) => WireMarketStreamItem::Event(event),
            Err(error) => WireMarketStreamItem::Error(error.into()),
        },
        Err(error) => WireMarketStreamItem::Error(error.into()),
    }
}

fn account_stream_item(item: Result<AccountEvent>) -> WireAccountStreamItem {
    match item {
        Ok(event) => match WireAccountEvent::try_from(event) {
            Ok(event) => WireAccountStreamItem::Event(event),
            Err(error) => WireAccountStreamItem::Error(error.into()),
        },
        Err(error) => WireAccountStreamItem::Error(error.into()),
    }
}

fn hyperliquid_market_stream_item(
    item: Result<HyperliquidMarketEvent>,
) -> WireHyperliquidMarketStreamItem {
    match item {
        Ok(event) => WireHyperliquidMarketStreamItem::Event(event.into()),
        Err(error) => WireHyperliquidMarketStreamItem::Error(error.into()),
    }
}

fn hyperliquid_account_stream_item(
    item: Result<HyperliquidAccountEvent>,
) -> WireHyperliquidAccountStreamItem {
    match item {
        Ok(event) => WireHyperliquidAccountStreamItem::Event(event.into()),
        Err(error) => WireHyperliquidAccountStreamItem::Error(error.into()),
    }
}

include!("generated_provider_streams.rs");

impl TryFrom<MarketEvent> for WireMarketEvent {
    type Error = Error;

    fn try_from(value: MarketEvent) -> Result<Self> {
        match value {
            MarketEvent::Trade(value) => Ok(Self::Trade(value.into())),
            MarketEvent::OrderBook(value) => Ok(Self::OrderBook(value.into())),
            MarketEvent::Ticker(value) => Ok(Self::Ticker(value.into())),
            MarketEvent::Candle(value) => Ok(Self::Candle(value.into())),
            MarketEvent::Reconnected => Ok(Self::Reconnected),
            _ => Err(Error::adapter(
                "Rust market stream returned an event unknown to the Dart bridge",
            )),
        }
    }
}

impl TryFrom<AccountEvent> for WireAccountEvent {
    type Error = Error;

    fn try_from(value: AccountEvent) -> Result<Self> {
        match value {
            AccountEvent::Balance(value) => Ok(Self::Balance(value.into())),
            AccountEvent::Order(value) => Ok(Self::Order(value.into())),
            AccountEvent::Reconnected => Ok(Self::Reconnected),
            _ => Err(Error::adapter(
                "Rust account stream returned an event unknown to the Dart bridge",
            )),
        }
    }
}

struct PendingMarketSource;

impl Stream for PendingMarketSource {
    type Item = Result<MarketEvent>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}

pub(crate) fn pending_market_subscription_for_test() -> NativeMarketSubscription {
    NativeMarketSubscription::new(MarketStream::new(PendingMarketSource))
}

struct StreamState<T> {
    inner: Mutex<BoundedState<T>>,
    status: AtomicU8,
    capacity: usize,
    overflow: Overflow,
    producer: RwLock<()>,
}

struct BoundedState<T> {
    queue: VecDeque<Result<T>>,
    consumer_waker: Option<Waker>,
    producer_waker: Option<Waker>,
    closed: bool,
}

impl<T> StreamState<T> {
    async fn send(&self, item: Result<T>, protected: bool) -> bool {
        if self.status.load(Ordering::Acquire) != OPEN {
            return false;
        }
        let _producer = self.producer.write().await;
        let mut item = Some(item);
        poll_fn(|cx| {
            if self.status.load(Ordering::Acquire) != OPEN {
                return Poll::Ready(false);
            }

            let mut inner = self.inner.lock().unwrap();
            if inner.closed {
                return Poll::Ready(false);
            }
            if inner.queue.len() < self.capacity {
                inner.queue.push_back(item.take().unwrap());
                let consumer = inner.consumer_waker.take();
                drop(inner);
                if let Some(consumer) = consumer {
                    consumer.wake();
                }
                return Poll::Ready(true);
            }
            if self.overflow == Overflow::DropNewest && !protected {
                return Poll::Ready(true);
            }

            inner.producer_waker = Some(cx.waker().clone());
            Poll::Pending
        })
        .await
    }

    fn finish(&self, status: u8) -> bool {
        if self
            .status
            .compare_exchange(OPEN, status, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return false;
        }
        let (consumer, producer) = {
            let mut inner = self.inner.lock().unwrap();
            inner.closed = true;
            (inner.consumer_waker.take(), inner.producer_waker.take())
        };
        if let Some(consumer) = consumer {
            consumer.wake();
        }
        if let Some(producer) = producer {
            producer.wake();
        }
        true
    }
}

/// Sink through which a Dart market subscription sends events to Rust.
#[flutter_rust_bridge::frb(opaque)]
pub struct MarketStreamSink {
    id: String,
    state: Arc<StreamState<MarketEvent>>,
}

impl fmt::Debug for MarketStreamSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MarketStreamSink")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl MarketStreamSink {
    /// Sends events and errors as items; only `End` terminates the stream.
    pub async fn add(&self, item: MarketStreamItem) -> bool {
        match item {
            MarketStreamItem::Event(WireMarketEvent::Reconnected) => {
                self.state.send(Ok(MarketEvent::Reconnected), true).await
            }
            MarketStreamItem::Event(event) => {
                self.state.send(event.into_result("market"), false).await
            }
            MarketStreamItem::Error(error) => {
                self.state.send(Err(structured_error(error)), false).await
            }
            MarketStreamItem::End => self.state.finish(ENDED),
        }
    }
}

impl Drop for MarketStreamSink {
    fn drop(&mut self) {
        self.state.finish(ENDED);
    }
}

/// Sink through which a Dart account subscription sends events to Rust.
#[flutter_rust_bridge::frb(opaque)]
pub struct AccountStreamSink {
    id: String,
    state: Arc<StreamState<AccountEvent>>,
}

impl fmt::Debug for AccountStreamSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountStreamSink")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl AccountStreamSink {
    /// Sends events and errors as items; only `End` terminates the stream.
    pub async fn add(&self, item: AccountStreamItem) -> bool {
        match item {
            AccountStreamItem::Event(WireAccountEvent::Reconnected) => {
                self.state.send(Ok(AccountEvent::Reconnected), true).await
            }
            AccountStreamItem::Event(event) => {
                self.state.send(event.into_result("account"), false).await
            }
            AccountStreamItem::Error(error) => {
                self.state.send(Err(structured_error(error)), false).await
            }
            AccountStreamItem::End => self.state.finish(ENDED),
        }
    }
}

impl Drop for AccountStreamSink {
    fn drop(&mut self) {
        self.state.finish(ENDED);
    }
}

trait IntoStreamResult<T> {
    fn into_result(self, stream: &str) -> Result<T>;
}

impl IntoStreamResult<MarketEvent> for WireMarketEvent {
    fn into_result(self, stream: &str) -> Result<MarketEvent> {
        let result = match self {
            Self::Trade(value) => value.try_into().map(MarketEvent::Trade),
            Self::OrderBook(value) => value.try_into().map(MarketEvent::OrderBook),
            Self::Ticker(value) => value.try_into().map(MarketEvent::Ticker),
            Self::Candle(value) => value.try_into().map(MarketEvent::Candle),
            Self::Reconnected => return Ok(MarketEvent::Reconnected),
        };
        result.map_err(|error| invalid_event(stream, error))
    }
}

impl IntoStreamResult<AccountEvent> for WireAccountEvent {
    fn into_result(self, stream: &str) -> Result<AccountEvent> {
        let result = match self {
            Self::Balance(value) => value.try_into().map(AccountEvent::Balance),
            Self::Order(value) => value.try_into().map(AccountEvent::Order),
            Self::Reconnected => return Ok(AccountEvent::Reconnected),
        };
        result.map_err(|error| invalid_event(stream, error))
    }
}

fn invalid_event(stream: &str, error: NativeError) -> Error {
    Error::adapter(format!(
        "Dart adapter returned an invalid {stream} stream event: {error}"
    ))
}

fn structured_error(error: NativeError) -> Error {
    Error::try_from(error).unwrap_or_else(|error| error)
}

struct CancelOnDrop<T> {
    id: String,
    state: Arc<StreamState<T>>,
    cancel: CancelCallback,
}

impl<T> Stream for CancelOnDrop<T> {
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let (item, producer) = {
            let mut inner = self.state.inner.lock().unwrap();
            if let Some(item) = inner.queue.pop_front() {
                (Some(Some(item)), inner.producer_waker.take())
            } else if inner.closed {
                (Some(None), None)
            } else {
                inner.consumer_waker = Some(cx.waker().clone());
                (None, None)
            }
        };
        if let Some(producer) = producer {
            producer.wake();
        }
        match item {
            Some(item) => Poll::Ready(item),
            None => Poll::Pending,
        }
    }
}

impl<T> Drop for CancelOnDrop<T> {
    fn drop(&mut self) {
        if self.state.finish(CANCELLED) {
            let id = self.id.clone();
            let future = (self.cancel)(id.clone());
            std::mem::drop(
                crate::frb_generated::FLUTTER_RUST_BRIDGE_HANDLER
                    .async_runtime()
                    .spawn(async move {
                        if let Err(error) = future.await {
                            eprintln!("Dart adapter stream {id} cancellation failed: {error}");
                        }
                    }),
            );
        }
    }
}

fn channel<T>(config: &StreamConfig) -> Arc<StreamState<T>> {
    Arc::new(StreamState {
        inner: Mutex::new(BoundedState {
            queue: VecDeque::new(),
            consumer_waker: None,
            producer_waker: None,
            closed: false,
        }),
        status: AtomicU8::new(OPEN),
        capacity: config.buffer_size.max(1),
        overflow: config.overflow,
        producer: RwLock::new(()),
    })
}

pub(crate) fn market_stream_channel(
    id: String,
    config: &StreamConfig,
    cancel: CancelCallback,
) -> (MarketStreamSink, MarketStream) {
    let state = channel(config);
    let sink = MarketStreamSink {
        id: id.clone(),
        state: state.clone(),
    };
    let close_id = id.clone();
    let close_state = state.clone();
    let close_cancel = cancel.clone();
    let stream =
        MarketStream::new_with_close(CancelOnDrop { id, state, cancel }, move || async move {
            if close_state.finish(CANCELLED) {
                close_cancel(close_id).await
            } else {
                Ok(())
            }
        });
    (sink, stream)
}

pub(crate) fn account_stream_channel(
    id: String,
    config: &StreamConfig,
    cancel: CancelCallback,
) -> (AccountStreamSink, AccountStream) {
    let state = channel(config);
    let sink = AccountStreamSink {
        id: id.clone(),
        state: state.clone(),
    };
    let close_id = id.clone();
    let close_state = state.clone();
    let close_cancel = cancel.clone();
    let stream =
        AccountStream::new_with_close(CancelOnDrop { id, state, cancel }, move || async move {
            if close_state.finish(CANCELLED) {
                close_cancel(close_id).await
            } else {
                Ok(())
            }
        });
    (sink, stream)
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};
    use std::time::Duration;

    use futures_channel::oneshot;
    use futures_util::future::{Either, select};
    use futures_util::{StreamExt, stream};
    use maxt::{
        AccountEvent, AccountStream, Error, MarketEvent, MarketStream, Overflow, StreamConfig,
    };

    use crate::convert::{NativeError, NativeErrorKind};

    use super::{
        CancelCallback, MarketStreamItem, NativeAccountSubscription, NativeMarketSubscription,
        WireAccountEvent, WireAccountStreamItem, WireMarketEvent, WireMarketStreamItem,
        market_stream_channel, pending_market_subscription_for_test,
    };

    fn no_cancel() -> CancelCallback {
        Arc::new(|_| Box::pin(async { Ok(()) }))
    }

    struct PendingUntilDropped {
        dropped: Arc<AtomicBool>,
    }

    struct CountingMarketSource {
        polls: Arc<AtomicU32>,
    }

    impl futures_core::Stream for CountingMarketSource {
        type Item = maxt::Result<MarketEvent>;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.polls.fetch_add(1, Ordering::SeqCst);
            Poll::Ready(Some(Ok(MarketEvent::Reconnected)))
        }
    }

    impl futures_core::Stream for PendingUntilDropped {
        type Item = maxt::Result<MarketEvent>;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Pending
        }
    }

    impl Drop for PendingUntilDropped {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::SeqCst);
        }
    }

    async fn within_one_second<F: Future>(future: F) -> F::Output {
        let (timeout_sender, timeout_receiver) = oneshot::channel();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(1));
            let _ = timeout_sender.send(());
        });

        match select(Box::pin(future), Box::pin(timeout_receiver)).await {
            Either::Left((output, _)) => output,
            Either::Right(_) => panic!("비동기 작업이 1초 안에 끝나지 않았습니다"),
        }
    }

    #[tokio::test]
    async fn market_error_is_non_terminal_and_only_end_closes_the_stream() {
        let (sink, mut stream) =
            market_stream_channel("7".to_owned(), &StreamConfig::default(), no_cancel());

        assert!(
            sink.add(MarketStreamItem::Event(WireMarketEvent::Reconnected,))
                .await
        );
        assert!(
            sink.add(MarketStreamItem::Error(NativeError::from(Error::Decode {
                detail: "bad frame".to_owned(),
            },)))
                .await
        );
        assert!(
            sink.add(MarketStreamItem::Event(WireMarketEvent::Reconnected,))
                .await
        );
        assert!(sink.add(MarketStreamItem::End).await);

        assert!(matches!(
            stream.next().await,
            Some(Ok(MarketEvent::Reconnected)),
        ));
        assert!(matches!(
            stream.next().await,
            Some(Err(Error::Decode { detail })) if detail == "bad frame",
        ));
        assert!(matches!(
            stream.next().await,
            Some(Ok(MarketEvent::Reconnected)),
        ));
        assert!(stream.next().await.is_none());
        assert!(
            !sink
                .add(MarketStreamItem::Event(WireMarketEvent::Reconnected,))
                .await
        );
    }

    #[tokio::test]
    async fn dropping_the_rust_stream_requests_dart_cancellation_once() {
        let cancelled = Arc::new(Mutex::new(None));
        let observed = cancelled.clone();
        let (sink, stream) = market_stream_channel(
            "41".to_owned(),
            &StreamConfig::default(),
            Arc::new(move |id| {
                let observed = observed.clone();
                Box::pin(async move {
                    *observed.lock().unwrap() = Some(id);
                    Ok(())
                })
            }),
        );

        drop(stream);

        within_one_second(async {
            while cancelled.lock().unwrap().is_none() {
                tokio::task::yield_now().await;
            }
        })
        .await;
        assert_eq!(cancelled.lock().unwrap().as_deref(), Some("41"));
        assert!(
            !sink
                .add(MarketStreamItem::Event(WireMarketEvent::Reconnected,))
                .await
        );
        drop(sink);
        assert_eq!(cancelled.lock().unwrap().as_deref(), Some("41"));
    }

    #[tokio::test]
    async fn dart_end_closes_account_stream_without_requesting_cancellation_back() {
        let cancellation_count = Arc::new(AtomicU32::new(0));
        let observed = cancellation_count.clone();
        let (sink, mut stream) = super::account_stream_channel(
            "9".to_owned(),
            &StreamConfig::default(),
            Arc::new(move |_| {
                let observed = observed.clone();
                Box::pin(async move {
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
            }),
        );

        assert!(
            sink.add(super::AccountStreamItem::Event(
                super::WireAccountEvent::Reconnected,
            ))
            .await
        );
        assert!(
            sink.add(super::AccountStreamItem::Error(NativeError::from(
                Error::Transport {
                    detail: "socket closed".to_owned(),
                },
            )))
            .await
        );
        assert!(sink.add(super::AccountStreamItem::End).await);

        assert!(matches!(
            stream.next().await,
            Some(Ok(maxt::AccountEvent::Reconnected)),
        ));
        assert!(matches!(
            stream.next().await,
            Some(Err(Error::Transport { detail })) if detail == "socket closed",
        ));
        assert!(stream.next().await.is_none());
        drop(stream);
        assert_eq!(cancellation_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn native_market_subscription_keeps_errors_non_terminal_and_only_none_ends() {
        let source = MarketStream::new(stream::iter([
            Ok(MarketEvent::Reconnected),
            Err(Error::Decode {
                detail: "bad native frame".to_owned(),
            }),
            Ok(MarketEvent::Reconnected),
        ]));
        let subscription = NativeMarketSubscription::new(source);

        assert!(matches!(
            subscription.next().await,
            WireMarketStreamItem::Event(WireMarketEvent::Reconnected),
        ));
        assert!(matches!(
            subscription.next().await,
            WireMarketStreamItem::Error(NativeError {
                kind: NativeErrorKind::Decode,
                detail: Some(detail),
                ..
            }) if detail == "bad native frame",
        ));
        assert!(matches!(
            subscription.next().await,
            WireMarketStreamItem::Event(WireMarketEvent::Reconnected),
        ));
        assert!(matches!(
            subscription.next().await,
            WireMarketStreamItem::End,
        ));
    }

    #[tokio::test]
    async fn native_account_subscription_keeps_errors_non_terminal_and_only_none_ends() {
        let source = AccountStream::new(stream::iter([
            Ok(AccountEvent::Reconnected),
            Err(Error::Transport {
                detail: "private socket stalled".to_owned(),
            }),
            Ok(AccountEvent::Reconnected),
        ]));
        let subscription = NativeAccountSubscription::new(source);

        assert!(matches!(
            subscription.next().await,
            WireAccountStreamItem::Event(WireAccountEvent::Reconnected),
        ));
        assert!(matches!(
            subscription.next().await,
            WireAccountStreamItem::Error(NativeError {
                kind: NativeErrorKind::Transport,
                detail: Some(detail),
                ..
            }) if detail == "private socket stalled",
        ));
        assert!(matches!(
            subscription.next().await,
            WireAccountStreamItem::Event(WireAccountEvent::Reconnected),
        ));
        assert!(matches!(
            subscription.next().await,
            WireAccountStreamItem::End,
        ));
    }

    #[tokio::test]
    async fn close_does_not_wait_for_the_receiver_lock_and_unblocks_pending_next() {
        let dropped = Arc::new(AtomicBool::new(false));
        let subscription = Arc::new(NativeMarketSubscription::new(MarketStream::new(
            PendingUntilDropped {
                dropped: dropped.clone(),
            },
        )));
        let pending_next = tokio::spawn({
            let subscription = subscription.clone();
            async move { subscription.next().await }
        });
        tokio::task::yield_now().await;

        within_one_second(subscription.close()).await.unwrap();
        let item = within_one_second(pending_next).await.unwrap();

        assert!(matches!(item, WireMarketStreamItem::End));
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn close_and_drop_are_idempotent_and_abort_the_forwarding_task() {
        let closed_drop = Arc::new(AtomicBool::new(false));
        let subscription = NativeMarketSubscription::new(MarketStream::new(PendingUntilDropped {
            dropped: closed_drop.clone(),
        }));

        within_one_second(subscription.close()).await.unwrap();
        within_one_second(subscription.close()).await.unwrap();
        assert!(closed_drop.load(Ordering::SeqCst));

        let dropped_drop = Arc::new(AtomicBool::new(false));
        let subscription = NativeMarketSubscription::new(MarketStream::new(PendingUntilDropped {
            dropped: dropped_drop.clone(),
        }));
        drop(subscription);

        within_one_second(async {
            while !dropped_drop.load(Ordering::SeqCst) {
                tokio::task::yield_now().await;
            }
        })
        .await;
    }

    #[tokio::test]
    async fn generated_api_fixture_is_pending_until_close_then_returns_end() {
        let subscription = Arc::new(pending_market_subscription_for_test());
        let pending_next = tokio::spawn({
            let subscription = subscription.clone();
            async move { subscription.next().await }
        });
        tokio::task::yield_now().await;

        within_one_second(subscription.close()).await.unwrap();

        assert!(matches!(
            within_one_second(pending_next).await.unwrap(),
            WireMarketStreamItem::End,
        ));
    }

    #[tokio::test]
    async fn native_handoff_polls_only_one_item_ahead_of_dart() {
        let polls = Arc::new(AtomicU32::new(0));
        let subscription = NativeMarketSubscription::new(MarketStream::new(CountingMarketSource {
            polls: polls.clone(),
        }));
        within_one_second(async {
            while polls.load(Ordering::SeqCst) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await;
        for _ in 0..32 {
            tokio::task::yield_now().await;
        }
        assert_eq!(polls.load(Ordering::SeqCst), 1);

        assert!(matches!(
            subscription.next().await,
            WireMarketStreamItem::Event(WireMarketEvent::Reconnected),
        ));
        within_one_second(async {
            while polls.load(Ordering::SeqCst) < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await;
        assert_eq!(polls.load(Ordering::SeqCst), 2);
        subscription.close().await.unwrap();
    }

    #[tokio::test]
    async fn native_close_waits_for_source_cleanup_ack_before_returning() {
        let dropped = Arc::new(AtomicBool::new(false));
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let subscription = Arc::new(NativeMarketSubscription::new(MarketStream::new_with_close(
            PendingUntilDropped {
                dropped: dropped.clone(),
            },
            move || async move {
                let _ = started_tx.send(());
                let _ = release_rx.await;
                Ok(())
            },
        )));
        let closing = tokio::spawn({
            let subscription = subscription.clone();
            async move { subscription.close().await }
        });
        started_rx.await.unwrap();
        tokio::task::yield_now().await;

        assert!(!closing.is_finished());
        assert!(!dropped.load(Ordering::SeqCst));

        release_tx.send(()).unwrap();
        within_one_second(closing).await.unwrap().unwrap();
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn native_close_propagates_source_cleanup_error_after_dropping_it() {
        let dropped = Arc::new(AtomicBool::new(false));
        let subscription = NativeMarketSubscription::new(MarketStream::new_with_close(
            PendingUntilDropped {
                dropped: dropped.clone(),
            },
            || async { Err(Error::adapter("cleanup ack failed")) },
        ));

        let error = subscription.close().await.unwrap_err();

        assert_eq!(error.kind, NativeErrorKind::Adapter);
        assert!(error.message.contains("cleanup ack failed"));
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn custom_sink_backpressure_waits_for_bounded_capacity() {
        let config = StreamConfig {
            buffer_size: 1,
            overflow: Overflow::Backpressure,
            ..StreamConfig::default()
        };
        let (sink, mut stream) = market_stream_channel("11".to_owned(), &config, no_cancel());
        let sink = Arc::new(sink);
        assert!(
            sink.add(MarketStreamItem::Event(WireMarketEvent::Reconnected))
                .await
        );
        let blocked = tokio::spawn({
            let sink = sink.clone();
            async move {
                sink.add(MarketStreamItem::Error(NativeError::from(Error::Decode {
                    detail: "waited".to_owned(),
                })))
                .await
            }
        });
        tokio::task::yield_now().await;
        assert!(!blocked.is_finished());

        assert!(matches!(
            stream.next().await,
            Some(Ok(MarketEvent::Reconnected)),
        ));
        assert!(within_one_second(blocked).await.unwrap());
        assert!(matches!(
            stream.next().await,
            Some(Err(Error::Decode { detail })) if detail == "waited",
        ));
    }

    #[test]
    fn custom_channel_does_not_preallocate_its_declared_capacity() {
        let config = StreamConfig {
            buffer_size: usize::MAX,
            ..StreamConfig::default()
        };

        let (sink, stream) = market_stream_channel("large".to_owned(), &config, no_cancel());

        assert_eq!(sink.state.capacity, usize::MAX);
        assert_eq!(sink.state.inner.lock().unwrap().queue.capacity(), 0);
        drop(stream);
    }

    #[tokio::test]
    async fn custom_sink_drop_newest_is_bounded_but_never_drops_reconnected() {
        let config = StreamConfig {
            buffer_size: 1,
            overflow: Overflow::DropNewest,
            ..StreamConfig::default()
        };
        let (sink, mut stream) = market_stream_channel("12".to_owned(), &config, no_cancel());
        let sink = Arc::new(sink);
        assert!(
            sink.add(MarketStreamItem::Error(NativeError::from(Error::Decode {
                detail: "kept".to_owned(),
            })))
            .await
        );
        assert!(
            sink.add(MarketStreamItem::Error(NativeError::from(Error::Decode {
                detail: "dropped".to_owned(),
            })))
            .await
        );
        let protected = tokio::spawn({
            let sink = sink.clone();
            async move {
                sink.add(MarketStreamItem::Event(WireMarketEvent::Reconnected))
                    .await
            }
        });
        tokio::task::yield_now().await;
        assert!(!protected.is_finished());

        assert!(matches!(
            stream.next().await,
            Some(Err(Error::Decode { detail })) if detail == "kept",
        ));
        assert!(within_one_second(protected).await.unwrap());
        assert!(matches!(
            stream.next().await,
            Some(Ok(MarketEvent::Reconnected)),
        ));

        assert!(sink.add(MarketStreamItem::End).await);
        assert!(stream.next().await.is_none());
    }
}
