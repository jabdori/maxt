#[cfg(not(target_arch = "wasm32"))]
use std::future::Future;
#[cfg(not(target_arch = "wasm32"))]
use std::pin::Pin;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use std::sync::atomic::AtomicU64;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(all(not(test), not(target_arch = "wasm32")))]
use futures_util::stream;
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use maxt::{AccountEvent, AccountStream, MarketEvent, MarketStream};
use maxt::{Error, Exchange, Feature, MarketKind};
use maxt_bindings_common::AdapterCall;
#[cfg(not(test))]
use maxt_bindings_common::{AdapterReply, ForeignAdapter, ForeignDispatcher};
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use napi::bindgen_prelude::{Function, Promise};
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use napi::threadsafe_function::ThreadsafeFunction;
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use napi_derive::napi;
#[cfg(not(test))]
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Handle;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::watch;

#[cfg(not(test))]
use crate::client::NativeClient;
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use crate::convert::outcome;
#[cfg(not(test))]
use crate::convert::{
    WireAccountStreamItem, WireAssetNetwork, WireBalance, WireCancelOrdersResult, WireCandle,
    WireDeposit, WireDepositAddress, WireFundingPayment, WireFundingRate, WireMarginSummary,
    WireMarketInfo, WireMarketStreamItem, WireOrder, WireOrderBook, WireOrderRules, WirePage,
    WirePosition, WireTicker, WireTrade, WireWithdrawal, WireWithdrawalQuote,
};
use crate::convert::{
    WireCancelOrdersRequest, WireCandleRequest, WireDepositAddressRequest, WireError,
    WireHistoryRequest, WireMarginRequest, WireMarket, WireOrderHistoryRequest,
    WireOrderLookupRequest, WireOrderRequest, WireStreamConfig, WireSubscription,
    WireTransferHistoryRequest, WireWithdrawRequest, feature_from_id, from_wire_text,
    from_wire_value,
};

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum WireAdapterCall {
    Markets {
        market_kind: &'static str,
    },
    Trades {
        market: WireMarket,
        limit: Option<u32>,
    },
    OrderBook {
        market: WireMarket,
        depth: Option<u32>,
    },
    Ticker {
        market: WireMarket,
    },
    Candles {
        request: WireCandleRequest,
    },
    Subscribe {
        stream_id: String,
        subscription: WireSubscription,
        config: WireStreamConfig,
    },
    Balances,
    OrderRules {
        market: WireMarket,
    },
    AssetNetworks {
        asset: String,
    },
    DepositAddress {
        request: WireDepositAddressRequest,
    },
    PrepareWithdrawal {
        request: WireWithdrawRequest,
    },
    Withdraw {
        request: WireWithdrawRequest,
    },
    Deposits {
        request: WireTransferHistoryRequest,
    },
    Withdrawals {
        request: WireTransferHistoryRequest,
    },
    OpenOrders {
        market: Option<WireMarket>,
    },
    Order {
        market: WireMarket,
        order_id: String,
    },
    OrderByClientId {
        market: WireMarket,
        client_id: String,
    },
    OrdersByIds {
        request: WireOrderLookupRequest,
    },
    OrderHistory {
        request: WireOrderHistoryRequest,
    },
    SubscribeAccount {
        stream_id: String,
        config: WireStreamConfig,
    },
    PlaceOrder {
        request: WireOrderRequest,
    },
    CancelOrder {
        market: WireMarket,
        order_id: String,
    },
    CancelOrderByClientId {
        market: WireMarket,
        client_id: String,
    },
    CancelOrders {
        request: WireCancelOrdersRequest,
    },
    Positions {
        market: Option<WireMarket>,
    },
    MarginSummary,
    FundingRates {
        request: WireHistoryRequest,
    },
    FundingPayments {
        request: WireHistoryRequest,
    },
    SetMargin {
        request: WireMarginRequest,
    },
}

#[cfg(not(test))]
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum WireAdapterReply {
    Markets { value: Vec<WireMarketInfo> },
    Trades { value: Vec<WireTrade> },
    OrderBook { value: WireOrderBook },
    Ticker { value: WireTicker },
    Candles { value: Vec<WireCandle> },
    MarketStream { stream_id: String },
    Balances { value: Vec<WireBalance> },
    OrderRules { value: Box<WireOrderRules> },
    AssetNetworks { value: Vec<WireAssetNetwork> },
    DepositAddress { value: WireDepositAddress },
    WithdrawalQuote { value: WireWithdrawalQuote },
    Withdrawal { value: WireWithdrawal },
    Deposits { value: WirePage<WireDeposit> },
    Withdrawals { value: WirePage<WireWithdrawal> },
    OpenOrders { value: Vec<WireOrder> },
    Order { value: WireOrder },
    OrdersByIds { value: Vec<WireOrder> },
    OrderHistory { value: WirePage<WireOrder> },
    AccountStream { stream_id: String },
    PlaceOrder { value: WireOrder },
    CancelOrders { value: WireCancelOrdersResult },
    Positions { value: Vec<WirePosition> },
    MarginSummary { value: WireMarginSummary },
    FundingRates { value: WirePage<WireFundingRate> },
    FundingPayments { value: WirePage<WireFundingPayment> },
    Unit,
}

fn call_to_wire(call: AdapterCall, stream_id: Option<String>) -> maxt::Result<WireAdapterCall> {
    match call {
        AdapterCall::Markets { kind } => Ok(WireAdapterCall::Markets {
            market_kind: market_kind_to_wire(kind)?,
        }),
        AdapterCall::Trades { market, limit } => Ok(WireAdapterCall::Trades {
            market: market.try_into()?,
            limit,
        }),
        AdapterCall::OrderBook { market, depth } => Ok(WireAdapterCall::OrderBook {
            market: market.try_into()?,
            depth,
        }),
        AdapterCall::Ticker { market } => Ok(WireAdapterCall::Ticker {
            market: market.try_into()?,
        }),
        AdapterCall::Candles { request } => Ok(WireAdapterCall::Candles {
            request: request.try_into()?,
        }),
        AdapterCall::Subscribe {
            subscription,
            config,
        } => Ok(WireAdapterCall::Subscribe {
            stream_id: required_stream_id(stream_id)?,
            subscription: subscription.try_into()?,
            config: config.try_into()?,
        }),
        AdapterCall::Balances => Ok(WireAdapterCall::Balances),
        AdapterCall::OrderRules { market } => Ok(WireAdapterCall::OrderRules {
            market: market.try_into()?,
        }),
        AdapterCall::AssetNetworks { asset } => Ok(WireAdapterCall::AssetNetworks { asset }),
        AdapterCall::DepositAddress { request } => Ok(WireAdapterCall::DepositAddress {
            request: request.try_into()?,
        }),
        AdapterCall::PrepareWithdrawal { request } => Ok(WireAdapterCall::PrepareWithdrawal {
            request: request.try_into()?,
        }),
        AdapterCall::Withdraw { request } => Ok(WireAdapterCall::Withdraw {
            request: request.try_into()?,
        }),
        AdapterCall::Deposits { request } => Ok(WireAdapterCall::Deposits {
            request: request.try_into()?,
        }),
        AdapterCall::Withdrawals { request } => Ok(WireAdapterCall::Withdrawals {
            request: request.try_into()?,
        }),
        AdapterCall::OpenOrders { market } => Ok(WireAdapterCall::OpenOrders {
            market: market.map(TryInto::try_into).transpose()?,
        }),
        AdapterCall::Order { market, order_id } => Ok(WireAdapterCall::Order {
            market: market.try_into()?,
            order_id,
        }),
        AdapterCall::OrderByClientId { market, client_id } => {
            Ok(WireAdapterCall::OrderByClientId {
                market: market.try_into()?,
                client_id,
            })
        }
        AdapterCall::OrdersByIds { request } => Ok(WireAdapterCall::OrdersByIds {
            request: request.try_into()?,
        }),
        AdapterCall::OrderHistory { request } => Ok(WireAdapterCall::OrderHistory {
            request: request.try_into()?,
        }),
        AdapterCall::SubscribeAccount { config } => Ok(WireAdapterCall::SubscribeAccount {
            stream_id: required_stream_id(stream_id)?,
            config: config.try_into()?,
        }),
        AdapterCall::PlaceOrder { request } => Ok(WireAdapterCall::PlaceOrder {
            request: request.try_into()?,
        }),
        AdapterCall::CancelOrder { market, order_id } => Ok(WireAdapterCall::CancelOrder {
            market: market.try_into()?,
            order_id,
        }),
        AdapterCall::CancelOrderByClientId { market, client_id } => {
            Ok(WireAdapterCall::CancelOrderByClientId {
                market: market.try_into()?,
                client_id,
            })
        }
        AdapterCall::CancelOrders { request } => Ok(WireAdapterCall::CancelOrders {
            request: request.try_into()?,
        }),
        AdapterCall::Positions { market } => Ok(WireAdapterCall::Positions {
            market: market.map(TryInto::try_into).transpose()?,
        }),
        AdapterCall::MarginSummary => Ok(WireAdapterCall::MarginSummary),
        AdapterCall::FundingRates { request } => Ok(WireAdapterCall::FundingRates {
            request: request.try_into()?,
        }),
        AdapterCall::FundingPayments { request } => Ok(WireAdapterCall::FundingPayments {
            request: request.try_into()?,
        }),
        AdapterCall::SetMargin { request } => Ok(WireAdapterCall::SetMargin {
            request: request.try_into()?,
        }),
        _ => Err(binding_contract("AdapterCall")),
    }
}

fn required_stream_id(stream_id: Option<String>) -> maxt::Result<String> {
    stream_id.ok_or_else(|| binding_contract("stream ID"))
}

fn market_kind_to_wire(kind: MarketKind) -> maxt::Result<&'static str> {
    match kind {
        MarketKind::Spot => Ok("spot"),
        MarketKind::Perpetual => Ok("perpetual"),
        _ => Err(binding_contract("MarketKind")),
    }
}

fn decode_outcome<T: DeserializeOwned>(text: &str, field: &str) -> maxt::Result<T> {
    let value: Value = from_wire_text(text, field)?;
    let Value::Object(mut object) = value else {
        return Err(invalid_callback(field, "must be a NativeOutcome object"));
    };
    let ok = object
        .remove("ok")
        .and_then(|value| value.as_bool())
        .ok_or_else(|| invalid_callback(field, "must contain boolean `ok`"))?;
    if ok {
        let value = object
            .remove("value")
            .ok_or_else(|| invalid_callback(field, "successful outcome must contain `value`"))?;
        if !object.is_empty() {
            return Err(invalid_callback(
                field,
                "successful outcome has unknown fields",
            ));
        }
        from_wire_value(value, field)
    } else {
        let error = object
            .remove("error")
            .ok_or_else(|| invalid_callback(field, "failed outcome must contain `error`"))?;
        if !object.is_empty() {
            return Err(invalid_callback(field, "failed outcome has unknown fields"));
        }
        Err(Error::try_from(from_wire_value::<WireError>(
            error, field,
        )?)?)
    }
}

fn invalid_callback(field: &str, detail: &str) -> Error {
    Error::InvalidRequest {
        field: field.to_owned(),
        detail: detail.to_owned(),
    }
}

fn binding_contract(type_name: &str) -> Error {
    Error::adapter(format!(
        "maxt binding contract does not map a new {type_name} variant"
    ))
}

fn parse_exchange(value: &str) -> maxt::Result<Exchange> {
    Exchange::ALL
        .into_iter()
        .find(|exchange| exchange.id() == value)
        .ok_or_else(|| Error::InvalidRequest {
            field: "exchange".to_owned(),
            detail: format!("unknown value `{value}`"),
        })
}

fn parse_features(values: Vec<String>) -> maxt::Result<Vec<Feature>> {
    values
        .into_iter()
        .map(|value| {
            feature_from_id(&value).ok_or_else(|| Error::InvalidRequest {
                field: "features".to_owned(),
                detail: format!("unknown value `{value}`"),
            })
        })
        .collect()
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
type AsyncCallback =
    ThreadsafeFunction<String, Promise<String>, String, napi::Status, false, true, 0>;

#[cfg(not(target_arch = "wasm32"))]
type CloseFuture = Pin<Box<dyn Future<Output = maxt::Result<()>> + Send + 'static>>;
#[cfg(not(target_arch = "wasm32"))]
type CloseCallback = Arc<dyn Fn(String) -> CloseFuture + Send + Sync>;

#[cfg(not(target_arch = "wasm32"))]
struct StreamClose {
    id: String,
    runtime: Handle,
    callback: CloseCallback,
    started: AtomicBool,
    result: watch::Sender<Option<maxt::Result<()>>>,
}

#[cfg(not(target_arch = "wasm32"))]
impl StreamClose {
    fn new(id: String, runtime: Handle, callback: CloseCallback) -> Arc<Self> {
        let (result, _) = watch::channel(None);
        Arc::new(Self {
            id,
            runtime,
            callback,
            started: AtomicBool::new(false),
            result,
        })
    }

    fn request(self: &Arc<Self>) {
        if self
            .started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let close = Arc::clone(self);
        self.runtime.spawn(async move {
            let result = (close.callback)(close.id.clone()).await;
            close.result.send_replace(Some(result));
        });
    }

    async fn wait(self: &Arc<Self>) -> maxt::Result<()> {
        let mut result = self.result.subscribe();
        self.request();
        loop {
            if let Some(result) = result.borrow().clone() {
                return result;
            }
            result.changed().await.map_err(|_| {
                Error::adapter(format!(
                    "foreign stream `{}` close result channel ended",
                    self.id
                ))
            })?;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct StreamLease {
    close: Arc<StreamClose>,
}

#[cfg(not(target_arch = "wasm32"))]
impl StreamLease {
    fn new(id: String, runtime: Handle, callback: CloseCallback) -> Self {
        Self {
            close: StreamClose::new(id, runtime, callback),
        }
    }

    fn id(&self) -> &str {
        &self.close.id
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for StreamLease {
    fn drop(&mut self) {
        self.close.request();
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
struct JsForeignCallbacks {
    dispatch: AsyncCallback,
    stream_next: AsyncCallback,
    stream_close: AsyncCallback,
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
impl JsForeignCallbacks {
    async fn invoke(callback: &AsyncCallback, value: String, name: &str) -> maxt::Result<String> {
        let promise = callback.call_async_catch(value).await.map_err(|error| {
            Error::adapter(format!("JavaScript {name} callback failed: {error}"))
        })?;
        promise.await.map_err(|error| {
            Error::adapter(format!("JavaScript {name} callback rejected: {error}"))
        })
    }

    async fn next_market(&self, id: &str) -> maxt::Result<Option<MarketEvent>> {
        let text = Self::invoke(&self.stream_next, id.to_owned(), "streamNext").await?;
        let value: Value = decode_outcome(&text, "streamNext result")?;
        if value.is_null() {
            return Ok(None);
        }
        let item: WireMarketStreamItem = from_wire_value(value, "streamNext result.value")?;
        Ok(Some(match item {
            WireMarketStreamItem::Event { event } => (*event).try_into(),
            WireMarketStreamItem::Error { error } => Err(Error::try_from(error)?),
        }?))
    }

    async fn next_account(&self, id: &str) -> maxt::Result<Option<AccountEvent>> {
        let text = Self::invoke(&self.stream_next, id.to_owned(), "streamNext").await?;
        let value: Value = decode_outcome(&text, "streamNext result")?;
        if value.is_null() {
            return Ok(None);
        }
        let item: WireAccountStreamItem = from_wire_value(value, "streamNext result.value")?;
        Ok(Some(match item {
            WireAccountStreamItem::Event { event } => event.try_into(),
            WireAccountStreamItem::Error { error } => Err(Error::try_from(error)?),
        }?))
    }

    async fn close(&self, id: &str) -> maxt::Result<()> {
        let text = Self::invoke(&self.stream_close, id.to_owned(), "streamClose").await?;
        let value: Value = decode_outcome(&text, "streamClose result")?;
        if value.is_null() {
            Ok(())
        } else {
            Err(invalid_callback("streamClose result.value", "must be null"))
        }
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
struct JsForeignDispatcher {
    callbacks: Arc<JsForeignCallbacks>,
    next_stream_id: AtomicU64,
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
impl JsForeignDispatcher {
    fn allocate_stream_lease(&self) -> maxt::Result<StreamLease> {
        let id = self
            .next_stream_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map(|id| id.to_string())
            .map_err(|_| Error::adapter("foreign stream ID space is exhausted"))?;
        let callbacks = Arc::clone(&self.callbacks);
        let close: CloseCallback = Arc::new(move |id| {
            let callbacks = Arc::clone(&callbacks);
            Box::pin(async move { callbacks.close(&id).await })
        });
        Ok(StreamLease::new(id, Handle::current(), close))
    }

    fn market_stream(&self, lease: StreamLease) -> MarketStream {
        let callbacks = Arc::clone(&self.callbacks);
        let close = Arc::clone(&lease.close);
        let source = stream::unfold((callbacks, lease), |(callbacks, lease)| async move {
            match callbacks.next_market(lease.id()).await {
                Ok(Some(item)) => Some((Ok(item), (callbacks, lease))),
                Ok(None) => None,
                Err(error) => Some((Err(error), (callbacks, lease))),
            }
        });
        MarketStream::new_with_close(source, move || async move { close.wait().await })
    }

    fn account_stream(&self, lease: StreamLease) -> AccountStream {
        let callbacks = Arc::clone(&self.callbacks);
        let close = Arc::clone(&lease.close);
        let source = stream::unfold((callbacks, lease), |(callbacks, lease)| async move {
            match callbacks.next_account(lease.id()).await {
                Ok(Some(item)) => Some((Ok(item), (callbacks, lease))),
                Ok(None) => None,
                Err(error) => Some((Err(error), (callbacks, lease))),
            }
        });
        AccountStream::new_with_close(source, move || async move { close.wait().await })
    }

    async fn decode_reply(
        &self,
        text: &str,
        mut lease: Option<StreamLease>,
    ) -> maxt::Result<AdapterReply> {
        let reply: WireAdapterReply = decode_outcome(text, "dispatch result")?;
        match reply {
            WireAdapterReply::Markets { value } => value
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()
                .map(AdapterReply::Markets),
            WireAdapterReply::Trades { value } => value
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()
                .map(AdapterReply::Trades),
            WireAdapterReply::OrderBook { value } => value.try_into().map(AdapterReply::OrderBook),
            WireAdapterReply::Ticker { value } => value.try_into().map(AdapterReply::Ticker),
            WireAdapterReply::Candles { value } => value
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()
                .map(AdapterReply::Candles),
            WireAdapterReply::MarketStream { stream_id } => {
                let lease = lease
                    .take()
                    .ok_or_else(|| unexpected_stream_id(&stream_id))?;
                verify_stream_id(&stream_id, Some(lease.id()))?;
                Ok(AdapterReply::MarketStream(self.market_stream(lease)))
            }
            WireAdapterReply::Balances { value } => value
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()
                .map(AdapterReply::Balances),
            WireAdapterReply::OrderRules { value } => (*value)
                .try_into()
                .map(Box::new)
                .map(AdapterReply::OrderRules),
            WireAdapterReply::AssetNetworks { value } => value
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()
                .map(AdapterReply::AssetNetworks),
            WireAdapterReply::DepositAddress { value } => {
                value.try_into().map(AdapterReply::DepositAddress)
            }
            WireAdapterReply::WithdrawalQuote { value } => {
                value.try_into().map(AdapterReply::WithdrawalQuote)
            }
            WireAdapterReply::Withdrawal { value } => {
                value.try_into().map(AdapterReply::Withdrawal)
            }
            WireAdapterReply::Deposits { value } => value.try_into().map(AdapterReply::Deposits),
            WireAdapterReply::Withdrawals { value } => {
                value.try_into().map(AdapterReply::Withdrawals)
            }
            WireAdapterReply::OpenOrders { value } => value
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()
                .map(AdapterReply::OpenOrders),
            WireAdapterReply::Order { value } => value.try_into().map(AdapterReply::Order),
            WireAdapterReply::OrdersByIds { value } => value
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()
                .map(AdapterReply::OrdersByIds),
            WireAdapterReply::OrderHistory { value } => {
                value.try_into().map(AdapterReply::OrderHistory)
            }
            WireAdapterReply::AccountStream { stream_id } => {
                let lease = lease
                    .take()
                    .ok_or_else(|| unexpected_stream_id(&stream_id))?;
                verify_stream_id(&stream_id, Some(lease.id()))?;
                Ok(AdapterReply::AccountStream(self.account_stream(lease)))
            }
            WireAdapterReply::PlaceOrder { value } => {
                value.try_into().map(AdapterReply::PlaceOrder)
            }
            WireAdapterReply::CancelOrders { value } => {
                value.try_into().map(AdapterReply::CancelOrdersResult)
            }
            WireAdapterReply::Positions { value } => value
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<_>>()
                .map(AdapterReply::Positions),
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
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
impl ForeignDispatcher for JsForeignDispatcher {
    fn dispatch(&self, call: AdapterCall) -> maxt::BoxFuture<'_, maxt::Result<AdapterReply>> {
        Box::pin(async move {
            let lease = match call {
                AdapterCall::Subscribe { .. } | AdapterCall::SubscribeAccount { .. } => {
                    Some(self.allocate_stream_lease()?)
                }
                _ => None,
            };
            let wire = call_to_wire(call, lease.as_ref().map(|lease| lease.id().to_owned()))?;
            let text = serde_json::to_string(&wire).map_err(|error| {
                Error::adapter(format!("could not serialize adapter call: {error}"))
            })?;
            let reply =
                JsForeignCallbacks::invoke(&self.callbacks.dispatch, text, "dispatch").await?;
            self.decode_reply(&reply, lease).await
        })
    }
}

fn unexpected_stream_id(actual: &str) -> Error {
    Error::adapter(format!(
        "foreign adapter returned unexpected stream ID `{actual}`"
    ))
}

fn verify_stream_id(actual: &str, expected: Option<&str>) -> maxt::Result<()> {
    match expected {
        Some(expected) if actual == expected => Ok(()),
        Some(expected) => Err(Error::adapter(format!(
            "foreign adapter returned stream ID `{actual}` for allocated stream `{expected}`"
        ))),
        None => Err(unexpected_stream_id(actual)),
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
fn async_callback(function: Function<'_, String, Promise<String>>) -> napi::Result<AsyncCallback> {
    function
        .build_threadsafe_function::<String>()
        .weak::<true>()
        .build()
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
fn factory_error(error: Error) -> napi::Error {
    napi::Error::from_reason(outcome::<Value>(Err(error)).to_string())
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
#[napi(
    js_name = "createCustomClient",
    ts_args_type = "exchange: string, features: string[], dispatch: (call: string) => Promise<string>, streamNext: (id: string) => Promise<string>, streamClose: (id: string) => Promise<string>"
)]
pub fn create_custom_client(
    exchange: String,
    features: Vec<String>,
    dispatch: Function<'_, String, Promise<String>>,
    stream_next: Function<'_, String, Promise<String>>,
    stream_close: Function<'_, String, Promise<String>>,
) -> napi::Result<NativeClient> {
    let exchange = parse_exchange(&exchange).map_err(factory_error)?;
    let features = parse_features(features).map_err(factory_error)?;
    let callbacks = Arc::new(JsForeignCallbacks {
        dispatch: async_callback(dispatch)?,
        stream_next: async_callback(stream_next)?,
        stream_close: async_callback(stream_close)?,
    });
    let dispatcher = Arc::new(JsForeignDispatcher {
        callbacks,
        next_stream_id: AtomicU64::new(1),
    });
    Ok(NativeClient::from_boxed(Box::new(ForeignAdapter::new(
        exchange, features, dispatcher,
    ))))
}

#[cfg(target_arch = "wasm32")]
#[path = "foreign_web.rs"]
mod web;

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;

    use futures_util::{StreamExt, future, stream};
    use maxt::{
        CandleRequest, ChainDestination, Decimal, DepositAddressRequest, HistoryRequest,
        MarginRequest, Market, MarketEvent, MarketStream, Network, OrderHistoryRequest,
        OrderRequest, Side, Size, StreamConfig, Subscription, TransferDestination,
        TransferHistoryRequest, WithdrawRequest,
    };

    use super::*;

    fn market() -> Market {
        Market::spot(Exchange::Binance, "BTC", "USDT")
    }

    fn test_lease(id: &str, calls: Arc<AtomicUsize>) -> StreamLease {
        let callback: CloseCallback = Arc::new(move |_| {
            let calls = Arc::clone(&calls);
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        });
        let lease = StreamLease::new(id.to_owned(), Handle::current(), callback);
        assert_eq!(lease.id(), id);
        lease
    }

    async fn wait_for_close(calls: &AtomicUsize) {
        tokio::time::timeout(std::time::Duration::from_millis(200), async {
            while calls.load(Ordering::SeqCst) == 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("stream close callback must be scheduled");
    }

    #[test]
    fn all_adapter_calls_have_the_expected_wire_kind() {
        let market = market();
        let destination = TransferDestination::Chain(ChainDestination {
            asset: "BTC".to_owned(),
            network: Network::Bitcoin,
            address: "bc1qdestination".to_owned(),
            memo: None,
        });
        let withdraw_request =
            WithdrawRequest::new("BTC", Network::Bitcoin, Decimal::ONE, destination);
        let calls = [
            AdapterCall::Markets {
                kind: MarketKind::Spot,
            },
            AdapterCall::Trades {
                market: market.clone(),
                limit: Some(10),
            },
            AdapterCall::OrderBook {
                market: market.clone(),
                depth: Some(20),
            },
            AdapterCall::Ticker {
                market: market.clone(),
            },
            AdapterCall::Candles {
                request: CandleRequest::new(market.clone(), maxt::Interval::Min1),
            },
            AdapterCall::Subscribe {
                subscription: Subscription::new()
                    .market(market.clone())
                    .feed(maxt::Feed::Trades),
                config: StreamConfig::default(),
            },
            AdapterCall::Balances,
            AdapterCall::OrderRules {
                market: market.clone(),
            },
            AdapterCall::AssetNetworks {
                asset: "BTC".to_owned(),
            },
            AdapterCall::DepositAddress {
                request: DepositAddressRequest::new("BTC", Network::Bitcoin),
            },
            AdapterCall::PrepareWithdrawal {
                request: withdraw_request.clone(),
            },
            AdapterCall::Withdraw {
                request: withdraw_request,
            },
            AdapterCall::Deposits {
                request: TransferHistoryRequest::new(),
            },
            AdapterCall::Withdrawals {
                request: TransferHistoryRequest::new(),
            },
            AdapterCall::OpenOrders {
                market: Some(market.clone()),
            },
            AdapterCall::Order {
                market: market.clone(),
                order_id: "order-1".to_owned(),
            },
            AdapterCall::OrderByClientId {
                market: market.clone(),
                client_id: "client-1".to_owned(),
            },
            AdapterCall::OrderHistory {
                request: OrderHistoryRequest::new().market(market.clone()),
            },
            AdapterCall::SubscribeAccount {
                config: StreamConfig::default(),
            },
            AdapterCall::PlaceOrder {
                request: OrderRequest::market(market.clone(), Side::Buy, Size::Base(Decimal::ONE)),
            },
            AdapterCall::CancelOrder {
                market: market.clone(),
                order_id: "order-1".to_owned(),
            },
            AdapterCall::CancelOrderByClientId {
                market: market.clone(),
                client_id: "client-1".to_owned(),
            },
            AdapterCall::Positions {
                market: Some(market.clone()),
            },
            AdapterCall::MarginSummary,
            AdapterCall::FundingRates {
                request: HistoryRequest::new(market.clone()),
            },
            AdapterCall::FundingPayments {
                request: HistoryRequest::new(market.clone()),
            },
            AdapterCall::SetMargin {
                request: MarginRequest::new(market),
            },
        ];
        let expected = [
            "markets",
            "trades",
            "order_book",
            "ticker",
            "candles",
            "subscribe",
            "balances",
            "order_rules",
            "asset_networks",
            "deposit_address",
            "prepare_withdrawal",
            "withdraw",
            "deposits",
            "withdrawals",
            "open_orders",
            "order",
            "order_by_client_id",
            "order_history",
            "subscribe_account",
            "place_order",
            "cancel_order",
            "cancel_order_by_client_id",
            "positions",
            "margin_summary",
            "funding_rates",
            "funding_payments",
            "set_margin",
        ];

        let actual = calls.into_iter().map(|call| {
            let stream_id = matches!(
                call,
                AdapterCall::Subscribe { .. } | AdapterCall::SubscribeAccount { .. }
            )
            .then(|| "7".to_owned());
            serde_json::to_value(call_to_wire(call, stream_id).unwrap()).unwrap()["kind"]
                .as_str()
                .unwrap()
                .to_owned()
        });
        assert_eq!(actual.collect::<Vec<_>>(), expected);
    }

    #[test]
    fn outcome_requires_exact_envelope_and_preserves_structured_errors() {
        assert_eq!(
            decode_outcome::<Value>(r#"{"ok":true,"value":null}"#, "callback").unwrap(),
            Value::Null
        );
        assert!(matches!(
            decode_outcome::<Value>(
                r#"{"ok":false,"error":{"kind":"transport","detail":"offline"}}"#,
                "callback"
            ),
            Err(Error::Transport { detail }) if detail == "offline"
        ));
        assert!(matches!(
            decode_outcome::<Value>(r#"{"ok":true,"value":null,"extra":1}"#, "callback"),
            Err(Error::InvalidRequest { field, .. }) if field == "callback"
        ));
    }

    #[test]
    fn metadata_and_stream_ids_are_strict() {
        assert_eq!(parse_exchange("binance").unwrap(), Exchange::Binance);
        assert!(parse_exchange("Binance").is_err());
        assert_eq!(
            parse_features(vec!["ticker".to_owned()]).unwrap(),
            [Feature::Ticker]
        );
        assert!(parse_features(vec!["future_feature".to_owned()]).is_err());
        assert!(verify_stream_id("3", Some("3")).is_ok());
        assert!(verify_stream_id("4", Some("3")).is_err());
        assert!(verify_stream_id("3", None).is_err());
    }

    #[tokio::test]
    async fn dropping_pending_subscribe_owner_schedules_close_once() {
        let calls = Arc::new(AtomicUsize::new(0));
        let lease = test_lease("1", Arc::clone(&calls));
        let pending = tokio::spawn(async move {
            let _lease = lease;
            future::pending::<()>().await;
        });
        tokio::task::yield_now().await;
        pending.abort();
        let _ = pending.await;

        wait_for_close(&calls).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn dropping_returned_stream_schedules_close_once() {
        let calls = Arc::new(AtomicUsize::new(0));
        let lease = test_lease("2", Arc::clone(&calls));
        let close = Arc::clone(&lease.close);
        let source = stream::unfold(lease, |_lease| async move {
            future::pending::<Option<(maxt::Result<MarketEvent>, StreamLease)>>().await
        });
        let returned =
            MarketStream::new_with_close(source, move || async move { close.wait().await });
        drop(returned);

        wait_for_close(&calls).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn explicit_close_is_awaited_and_runs_once() {
        let calls = Arc::new(AtomicUsize::new(0));
        let lease = test_lease("3", Arc::clone(&calls));
        let close = Arc::clone(&lease.close);
        let source = stream::pending::<maxt::Result<MarketEvent>>();
        let mut returned =
            MarketStream::new_with_close(source, move || async move { close.wait().await });
        drop(lease);

        returned.close().await.unwrap();
        returned.close().await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn natural_end_awaits_close_once() {
        let calls = Arc::new(AtomicUsize::new(0));
        let lease = test_lease("4", Arc::clone(&calls));
        let close = Arc::clone(&lease.close);
        let source = stream::unfold(Some(lease), |lease| async move {
            drop(lease);
            None::<(maxt::Result<MarketEvent>, Option<StreamLease>)>
        });
        let mut returned =
            MarketStream::new_with_close(source, move || async move { close.wait().await });

        assert!(returned.next().await.is_none());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
