use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::task::{Context, Poll};

use futures_channel::{mpsc, oneshot};
use futures_core::Stream;
use futures_util::StreamExt;
use js_sys::{Function, Promise};
use maxt::{AccountEvent, AccountStream, MarketEvent, MarketStream};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use super::*;

enum LocalRequest {
    Dispatch {
        call: AdapterCall,
        stream_id: Option<String>,
        requests: mpsc::UnboundedSender<LocalRequest>,
        reply: oneshot::Sender<maxt::Result<AdapterReply>>,
    },
    StreamNext {
        id: String,
        kind: StreamKind,
        reply: oneshot::Sender<maxt::Result<Option<ForeignStreamItem>>>,
    },
    StreamClose {
        id: String,
        reply: Option<oneshot::Sender<maxt::Result<()>>>,
    },
}

#[derive(Clone, Copy)]
enum StreamKind {
    Market,
    Account,
}

enum ForeignStreamItem {
    Market(MarketEvent),
    Account(AccountEvent),
}

struct JsCallbacks {
    dispatch: Function,
    stream_next: Function,
    stream_close: Function,
}

struct WebForeignDispatcher {
    requests: mpsc::UnboundedSender<LocalRequest>,
    next_stream_id: AtomicU64,
}

impl ForeignDispatcher for WebForeignDispatcher {
    fn dispatch(&self, call: AdapterCall) -> maxt::BoxFuture<'_, maxt::Result<AdapterReply>> {
        let stream_id = match &call {
            AdapterCall::Subscribe { .. } | AdapterCall::SubscribeAccount { .. } => self
                .next_stream_id
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
                .map(|id| id.to_string())
                .map_err(|_| Error::adapter("foreign stream ID space is exhausted")),
            _ => Ok(String::new()),
        };
        let requests = self.requests.clone();
        Box::pin(async move {
            let stream_id = stream_id?;
            let stream_id = (!stream_id.is_empty()).then_some(stream_id);
            let (reply, result) = oneshot::channel();
            requests
                .unbounded_send(LocalRequest::Dispatch {
                    call,
                    stream_id,
                    requests: requests.clone(),
                    reply,
                })
                .map_err(|_| Error::adapter("JavaScript Adapter dispatcher is closed"))?;
            result.await.map_err(|_| {
                Error::adapter("JavaScript Adapter dispatcher stopped before replying")
            })?
        })
    }
}

fn spawn_callback_pump(
    callbacks: JsCallbacks,
    mut requests: mpsc::UnboundedReceiver<LocalRequest>,
) {
    let callbacks = std::rc::Rc::new(callbacks);
    wasm_bindgen_futures::spawn_local(async move {
        while let Some(request) = requests.next().await {
            let callbacks = callbacks.clone();
            wasm_bindgen_futures::spawn_local(async move {
                handle_request(&callbacks, request).await;
            });
        }
    });
}

async fn handle_request(callbacks: &JsCallbacks, request: LocalRequest) {
    match request {
        LocalRequest::Dispatch {
            call,
            stream_id,
            requests,
            reply,
        } => {
            let result = dispatch(callbacks, requests, call, stream_id.as_deref()).await;
            if result.is_err()
                && let Some(id) = stream_id
            {
                let _ = invoke(&callbacks.stream_close, id).await;
            }
            let _ = reply.send(result);
        }
        LocalRequest::StreamNext { id, kind, reply } => {
            let result = stream_next(callbacks, &id, kind).await;
            let _ = reply.send(result);
        }
        LocalRequest::StreamClose { id, reply } => {
            let result = stream_close(callbacks, &id).await;
            if let Some(reply) = reply {
                let _ = reply.send(result);
            }
        }
    }
}

async fn invoke(callback: &Function, value: String) -> maxt::Result<String> {
    let returned = callback
        .call1(&JsValue::UNDEFINED, &JsValue::from_str(&value))
        .map_err(|_| callback_error())?;
    JsFuture::from(Promise::resolve(&returned))
        .await
        .map_err(|_| callback_error())?
        .as_string()
        .ok_or_else(callback_error)
}

fn callback_error() -> Error {
    Error::adapter("JavaScript Adapter callback rejected or returned an invalid outcome")
}

async fn dispatch(
    callbacks: &JsCallbacks,
    requests: mpsc::UnboundedSender<LocalRequest>,
    call: AdapterCall,
    stream_id: Option<&str>,
) -> maxt::Result<AdapterReply> {
    let wire = call_to_wire(call, stream_id.map(str::to_owned))?;
    let text = serde_json::to_string(&wire)
        .map_err(|error| Error::adapter(format!("could not serialize adapter call: {error}")))?;
    let reply = invoke(&callbacks.dispatch, text).await?;
    decode_reply(&reply, stream_id, requests)
}

fn decode_reply(
    text: &str,
    stream_id: Option<&str>,
    requests: mpsc::UnboundedSender<LocalRequest>,
) -> maxt::Result<AdapterReply> {
    let reply: WireAdapterReply = decode_outcome(text, "dispatch result")?;
    match reply {
        WireAdapterReply::Markets { value } => wire_vec(value).map(AdapterReply::Markets),
        WireAdapterReply::Trades { value } => wire_vec(value).map(AdapterReply::Trades),
        WireAdapterReply::OrderBook { value } => value.try_into().map(AdapterReply::OrderBook),
        WireAdapterReply::Ticker { value } => value.try_into().map(AdapterReply::Ticker),
        WireAdapterReply::Candles { value } => wire_vec(value).map(AdapterReply::Candles),
        WireAdapterReply::MarketStream {
            stream_id: returned,
        } => {
            verify_stream_id(&returned, stream_id)?;
            Ok(AdapterReply::MarketStream(market_stream(
                returned, requests,
            )))
        }
        WireAdapterReply::Balances { value } => wire_vec(value).map(AdapterReply::Balances),
        WireAdapterReply::OrderRules { value } => (*value)
            .try_into()
            .map(Box::new)
            .map(AdapterReply::OrderRules),
        WireAdapterReply::AssetNetworks { value } => {
            wire_vec(value).map(AdapterReply::AssetNetworks)
        }
        WireAdapterReply::DepositAddress { value } => {
            value.try_into().map(AdapterReply::DepositAddress)
        }
        WireAdapterReply::WithdrawalQuote { value } => {
            value.try_into().map(AdapterReply::WithdrawalQuote)
        }
        WireAdapterReply::Withdrawal { value } => value.try_into().map(AdapterReply::Withdrawal),
        WireAdapterReply::Deposits { value } => value.try_into().map(AdapterReply::Deposits),
        WireAdapterReply::Withdrawals { value } => value.try_into().map(AdapterReply::Withdrawals),
        WireAdapterReply::OpenOrders { value } => wire_vec(value).map(AdapterReply::OpenOrders),
        WireAdapterReply::Order { value } => value.try_into().map(AdapterReply::Order),
        WireAdapterReply::OrdersByIds { value } => wire_vec(value).map(AdapterReply::OrdersByIds),
        WireAdapterReply::OrderHistory { value } => {
            value.try_into().map(AdapterReply::OrderHistory)
        }
        WireAdapterReply::AccountStream {
            stream_id: returned,
        } => {
            verify_stream_id(&returned, stream_id)?;
            Ok(AdapterReply::AccountStream(account_stream(
                returned, requests,
            )))
        }
        WireAdapterReply::PlaceOrder { value } => value.try_into().map(AdapterReply::PlaceOrder),
        WireAdapterReply::CancelOrders { value } => {
            value.try_into().map(AdapterReply::CancelOrdersResult)
        }
        WireAdapterReply::Positions { value } => wire_vec(value).map(AdapterReply::Positions),
        WireAdapterReply::MarginSummary { value } => {
            value.try_into().map(AdapterReply::MarginSummary)
        }
        WireAdapterReply::FundingRates { value } => {
            value.try_into().map(AdapterReply::FundingRates)
        }
        WireAdapterReply::FundingPayments { value } => {
            value.try_into().map(AdapterReply::FundingPayments)
        }
        WireAdapterReply::Unit => Ok(AdapterReply::Unit),
    }
}

fn wire_vec<T, W>(values: Vec<W>) -> maxt::Result<Vec<T>>
where
    T: TryFrom<W, Error = Error>,
{
    values.into_iter().map(TryInto::try_into).collect()
}

async fn stream_next(
    callbacks: &JsCallbacks,
    id: &str,
    kind: StreamKind,
) -> maxt::Result<Option<ForeignStreamItem>> {
    let text = invoke(&callbacks.stream_next, id.to_owned()).await?;
    let value: Value = decode_outcome(&text, "streamNext result")?;
    if value.is_null() {
        return Ok(None);
    }
    match kind {
        StreamKind::Market => {
            let item: WireMarketStreamItem = from_wire_value(value, "streamNext result.value")?;
            match item {
                WireMarketStreamItem::Event { event } => {
                    Ok(Some(ForeignStreamItem::Market((*event).try_into()?)))
                }
                WireMarketStreamItem::Error { error } => Err(error.try_into()?),
            }
        }
        StreamKind::Account => {
            let item: WireAccountStreamItem = from_wire_value(value, "streamNext result.value")?;
            match item {
                WireAccountStreamItem::Event { event } => {
                    Ok(Some(ForeignStreamItem::Account(event.try_into()?)))
                }
                WireAccountStreamItem::Error { error } => Err(error.try_into()?),
            }
        }
    }
}

async fn stream_close(callbacks: &JsCallbacks, id: &str) -> maxt::Result<()> {
    let text = invoke(&callbacks.stream_close, id.to_owned()).await?;
    let value: Value = decode_outcome(&text, "streamClose result")?;
    if value.is_null() {
        Ok(())
    } else {
        Err(invalid_callback("streamClose result.value", "must be null"))
    }
}

trait FromForeignItem: Sized {
    const KIND: StreamKind;
    fn from_foreign(item: ForeignStreamItem) -> maxt::Result<Self>;
}

impl FromForeignItem for MarketEvent {
    const KIND: StreamKind = StreamKind::Market;

    fn from_foreign(item: ForeignStreamItem) -> maxt::Result<Self> {
        match item {
            ForeignStreamItem::Market(item) => Ok(item),
            ForeignStreamItem::Account(_) => Err(Error::adapter(
                "JavaScript Adapter returned an account item for a market stream",
            )),
        }
    }
}

impl FromForeignItem for AccountEvent {
    const KIND: StreamKind = StreamKind::Account;

    fn from_foreign(item: ForeignStreamItem) -> maxt::Result<Self> {
        match item {
            ForeignStreamItem::Account(item) => Ok(item),
            ForeignStreamItem::Market(_) => Err(Error::adapter(
                "JavaScript Adapter returned a market item for an account stream",
            )),
        }
    }
}

struct ForeignPullStream<T> {
    id: String,
    requests: mpsc::UnboundedSender<LocalRequest>,
    pending: Option<oneshot::Receiver<maxt::Result<Option<ForeignStreamItem>>>>,
    closed: Arc<AtomicBool>,
    ended: bool,
    marker: PhantomData<T>,
}

impl<T> ForeignPullStream<T> {
    fn new(
        id: String,
        requests: mpsc::UnboundedSender<LocalRequest>,
        closed: Arc<AtomicBool>,
    ) -> Self {
        Self {
            id,
            requests,
            pending: None,
            closed,
            ended: false,
            marker: PhantomData,
        }
    }
}

impl<T: FromForeignItem + Unpin> Stream for ForeignPullStream<T> {
    type Item = maxt::Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.ended {
            return Poll::Ready(None);
        }
        if self.pending.is_none() {
            let (reply, result) = oneshot::channel();
            if self
                .requests
                .unbounded_send(LocalRequest::StreamNext {
                    id: self.id.clone(),
                    kind: T::KIND,
                    reply,
                })
                .is_err()
            {
                self.ended = true;
                return Poll::Ready(Some(Err(Error::adapter(
                    "JavaScript Adapter dispatcher is closed",
                ))));
            }
            self.pending = Some(result);
        }
        match Pin::new(self.pending.as_mut().expect("pending receiver")).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                self.pending = None;
                match result {
                    Ok(Ok(Some(item))) => Poll::Ready(Some(T::from_foreign(item))),
                    Ok(Ok(None)) => {
                        self.ended = true;
                        Poll::Ready(None)
                    }
                    Ok(Err(error)) => Poll::Ready(Some(Err(error))),
                    Err(_) => {
                        self.ended = true;
                        Poll::Ready(Some(Err(Error::adapter(
                            "JavaScript Adapter stream callback stopped before replying",
                        ))))
                    }
                }
            }
        }
    }
}

impl<T> Drop for ForeignPullStream<T> {
    fn drop(&mut self) {
        if !self.closed.swap(true, Ordering::AcqRel) {
            let _ = self.requests.unbounded_send(LocalRequest::StreamClose {
                id: self.id.clone(),
                reply: None,
            });
        }
    }
}

async fn close_foreign_stream(
    id: String,
    requests: mpsc::UnboundedSender<LocalRequest>,
    closed: Arc<AtomicBool>,
) -> maxt::Result<()> {
    if closed.swap(true, Ordering::AcqRel) {
        return Ok(());
    }
    let (reply, result) = oneshot::channel();
    requests
        .unbounded_send(LocalRequest::StreamClose {
            id,
            reply: Some(reply),
        })
        .map_err(|_| Error::adapter("JavaScript Adapter dispatcher is closed"))?;
    result
        .await
        .map_err(|_| Error::adapter("JavaScript Adapter stream close stopped before replying"))?
}

fn market_stream(id: String, requests: mpsc::UnboundedSender<LocalRequest>) -> MarketStream {
    let closed = Arc::new(AtomicBool::new(false));
    let close_requests = requests.clone();
    let close_id = id.clone();
    let close_state = Arc::clone(&closed);
    MarketStream::new_with_close(
        ForeignPullStream::<MarketEvent>::new(id, requests, closed),
        move || async move { close_foreign_stream(close_id, close_requests, close_state).await },
    )
}

fn account_stream(id: String, requests: mpsc::UnboundedSender<LocalRequest>) -> AccountStream {
    let closed = Arc::new(AtomicBool::new(false));
    let close_requests = requests.clone();
    let close_id = id.clone();
    let close_state = Arc::clone(&closed);
    AccountStream::new_with_close(
        ForeignPullStream::<AccountEvent>::new(id, requests, closed),
        move || async move { close_foreign_stream(close_id, close_requests, close_state).await },
    )
}

#[wasm_bindgen(js_name = "createCustomClient")]
pub fn create_custom_client_wasm(
    exchange: String,
    features: Vec<String>,
    dispatch: Function,
    stream_next: Function,
    stream_close: Function,
) -> Result<NativeClient, JsValue> {
    let exchange = parse_exchange(&exchange).map_err(crate::web::factory_error)?;
    let features = parse_features(features).map_err(crate::web::factory_error)?;
    let (requests, receiver) = mpsc::unbounded();
    spawn_callback_pump(
        JsCallbacks {
            dispatch,
            stream_next,
            stream_close,
        },
        receiver,
    );
    let dispatcher = Arc::new(WebForeignDispatcher {
        requests,
        next_stream_id: AtomicU64::new(1),
    });
    Ok(NativeClient::from_boxed(Box::new(ForeignAdapter::new(
        exchange, features, dispatcher,
    ))))
}
