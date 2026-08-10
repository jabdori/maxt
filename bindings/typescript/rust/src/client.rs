use std::sync::Arc;

#[cfg(all(not(test), not(target_arch = "wasm32")))]
use std::future::Future;

use maxt::{Adapter, Client, Error};
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use napi::bindgen_prelude::{Either, PromiseRaw};
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use napi::{Env, Unknown};
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use napi_derive::napi;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::convert::{
    WireAssetNetwork, WireBalance, WireCandle, WireCandleRequest, WireChainTransferRequest,
    WireDeposit, WireDepositAddress, WireDepositAddressRequest, WireExchangeTransferRequest,
    WireFundingPayment, WireFundingRate, WireHistoryRequest, WireMarginRequest, WireMarginSummary,
    WireMarket, WireMarketInfo, WireOrder, WireOrderBook, WireOrderRequest, WirePage, WirePosition,
    WireStreamConfig, WireSubscription, WireTicker, WireTrade, WireTransferHistoryRequest,
    WireTransferPlan, WireWithdrawRequest, WireWithdrawal, WireWithdrawalQuote, feature_from_id,
    from_wire_text, market_kind_from_wire, outcome,
};
use crate::stream::NativeStreamRegistry;

#[cfg_attr(all(not(test), not(target_arch = "wasm32")), napi)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct NativeClient {
    inner: Arc<Client<Box<dyn Adapter>>>,
    streams: Arc<NativeStreamRegistry>,
}

impl NativeClient {
    pub fn from_boxed(adapter: Box<dyn Adapter>) -> Self {
        Self {
            inner: Arc::new(Client::new(adapter)),
            streams: Arc::new(NativeStreamRegistry::default()),
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(adapter: Box<dyn Adapter>) -> Self {
        Self::from_boxed(adapter)
    }

    async fn markets(&self, kind: maxt::Result<String>) -> Value {
        let kind = kind
            .and_then(|kind| from_wire_text::<String>(&kind, "kind"))
            .and_then(|kind| market_kind_from_wire(&kind, "kind"));
        match kind {
            Ok(kind) => outcome(wire_vec::<_, WireMarketInfo>(
                self.inner.markets(kind).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }
}

impl NativeClient {
    fn exchange(&self) -> String {
        self.inner.exchange().id().to_owned()
    }

    fn supports(&self, feature: String) -> bool {
        feature_from_id(&feature).is_some_and(|feature| self.inner.supports(feature))
    }

    async fn trades(&self, market: maxt::Result<String>, limit: maxt::Result<String>) -> Value {
        let market = parse_wire::<maxt::Market, WireMarket>(market, "market");
        let limit = parse_wire_text::<Option<u32>>(limit, "limit");
        match (market, limit) {
            (Ok(market), Ok(limit)) => outcome(wire_vec::<_, WireTrade>(
                self.inner.trades(&market, limit).await,
            )),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    async fn order_book(&self, market: maxt::Result<String>, depth: maxt::Result<String>) -> Value {
        let market = parse_wire::<maxt::Market, WireMarket>(market, "market");
        let depth = parse_wire_text::<Option<u32>>(depth, "depth");
        match (market, depth) {
            (Ok(market), Ok(depth)) => outcome(wire_one::<_, WireOrderBook>(
                self.inner.order_book(&market, depth).await,
            )),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    async fn ticker(&self, market: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::Market, WireMarket>(market, "market") {
            Ok(market) => outcome(wire_one::<_, WireTicker>(self.inner.ticker(&market).await)),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn candles(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::CandleRequest, WireCandleRequest>(request, "request") {
            Ok(request) => outcome(wire_vec::<_, WireCandle>(
                self.inner.candles(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn balances(&self) -> Value {
        outcome(wire_vec::<_, WireBalance>(self.inner.balances().await))
    }

    async fn asset_networks(&self, asset: maxt::Result<String>) -> Value {
        match parse_wire_text::<String>(asset, "asset") {
            Ok(asset) => outcome(wire_vec::<_, WireAssetNetwork>(
                self.inner.asset_networks(&asset).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn deposit_address(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::DepositAddressRequest, WireDepositAddressRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_one::<_, WireDepositAddress>(
                self.inner.deposit_address(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn prepare_withdrawal(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::WithdrawRequest, WireWithdrawRequest>(request, "request") {
            Ok(request) => outcome(wire_one::<_, WireWithdrawalQuote>(
                self.inner.prepare_withdrawal(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn withdraw(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::WithdrawRequest, WireWithdrawRequest>(request, "request") {
            Ok(request) => outcome(wire_one::<_, WireWithdrawal>(
                self.inner.withdraw(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn deposits(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::TransferHistoryRequest, WireTransferHistoryRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_one::<_, WirePage<WireDeposit>>(
                self.inner.deposits(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn withdrawals(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::TransferHistoryRequest, WireTransferHistoryRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_one::<_, WirePage<WireWithdrawal>>(
                self.inner.withdrawals(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(test, allow(dead_code))]
    async fn prepare_transfer_to(
        &self,
        destination: &Self,
        request: maxt::Result<String>,
    ) -> Value {
        match parse_wire::<maxt::ExchangeTransferRequest, WireExchangeTransferRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_one::<_, WireTransferPlan>(
                maxt::prepare_exchange_transfer(
                    self.inner.adapter().as_ref(),
                    destination.inner.adapter().as_ref(),
                    &request,
                )
                .await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(test, allow(dead_code))]
    async fn prepare_transfer_to_chain(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::ChainTransferRequest, WireChainTransferRequest>(request, "request")
        {
            Ok(request) => outcome(wire_one::<_, WireTransferPlan>(
                maxt::prepare_chain_transfer(self.inner.adapter().as_ref(), &request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    #[cfg_attr(test, allow(dead_code))]
    async fn execute_transfer(&self, plan: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::TransferPlan, WireTransferPlan>(plan, "plan") {
            Ok(plan) => outcome(wire_one::<_, WireWithdrawal>(
                maxt::execute_transfer_plan(self.inner.adapter().as_ref(), &plan).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn open_orders(&self) -> Value {
        outcome(wire_vec::<_, WireOrder>(self.inner.open_orders().await))
    }

    async fn open_orders_on(&self, market: maxt::Result<String>) -> Value {
        let market = parse_wire_text::<WireMarket>(market, "market").and_then(TryInto::try_into);
        match market {
            Ok(market) => outcome(wire_vec::<_, WireOrder>(
                self.inner.open_orders_on(&market).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn positions(&self) -> Value {
        outcome(wire_vec::<_, WirePosition>(self.inner.positions().await))
    }

    async fn place_order(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::OrderRequest, WireOrderRequest>(request, "request") {
            Ok(request) => outcome(wire_one::<_, WireOrder>(
                self.inner.place_order(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn cancel_order(
        &self,
        market: maxt::Result<String>,
        order_id: maxt::Result<String>,
    ) -> Value {
        let market = parse_wire::<maxt::Market, WireMarket>(market, "market");
        let order_id = parse_wire_text::<String>(order_id, "order_id");
        match (market, order_id) {
            (Ok(market), Ok(order_id)) => outcome(
                self.inner
                    .cancel_order(&market, &order_id)
                    .await
                    .map(|()| Value::Null),
            ),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    async fn cancel_order_by_client_id(
        &self,
        market: maxt::Result<String>,
        client_id: maxt::Result<String>,
    ) -> Value {
        let market = parse_wire::<maxt::Market, WireMarket>(market, "market");
        let client_id = parse_wire_text::<String>(client_id, "client_id");
        match (market, client_id) {
            (Ok(market), Ok(client_id)) => outcome(
                self.inner
                    .cancel_order_by_client_id(&market, &client_id)
                    .await
                    .map(|()| Value::Null),
            ),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    async fn positions_on(&self, market: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::Market, WireMarket>(market, "market") {
            Ok(market) => outcome(wire_vec::<_, WirePosition>(
                self.inner.positions_on(&market).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn margin_summary(&self) -> Value {
        outcome(wire_one::<_, WireMarginSummary>(
            self.inner.margin_summary().await,
        ))
    }

    async fn funding_rates(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::HistoryRequest, WireHistoryRequest>(request, "request") {
            Ok(request) => outcome(wire_one::<_, WirePage<WireFundingRate>>(
                self.inner.funding_rates(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn funding_payments(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::HistoryRequest, WireHistoryRequest>(request, "request") {
            Ok(request) => outcome(wire_one::<_, WirePage<WireFundingPayment>>(
                self.inner.funding_payments(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn set_margin(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::MarginRequest, WireMarginRequest>(request, "request") {
            Ok(request) => outcome(self.inner.set_margin(&request).await.map(|()| Value::Null)),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn subscribe(&self, subscription: maxt::Result<String>) -> Value {
        let subscription = parse_wire_text::<WireSubscription>(subscription, "subscription")
            .and_then(TryInto::try_into);
        let result = match subscription {
            Ok(subscription) => match self.inner.subscribe(&subscription).await {
                Ok(stream) => self.streams.insert_market(stream).await,
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        };
        outcome(result)
    }

    async fn subscribe_with(
        &self,
        subscription: maxt::Result<String>,
        config: maxt::Result<String>,
    ) -> Value {
        let subscription = parse_wire_text::<WireSubscription>(subscription, "subscription")
            .and_then(TryInto::try_into);
        let config =
            parse_wire_text::<WireStreamConfig>(config, "config").and_then(TryInto::try_into);
        let result = match (subscription, config) {
            (Ok(subscription), Ok(config)) => {
                match self.inner.subscribe_with(&subscription, &config).await {
                    Ok(stream) => self.streams.insert_market(stream).await,
                    Err(error) => Err(error),
                }
            }
            (Err(error), _) | (_, Err(error)) => Err(error),
        };
        outcome(result)
    }

    async fn subscribe_account(&self) -> Value {
        let result = match self.inner.subscribe_account().await {
            Ok(stream) => self.streams.insert_account(stream).await,
            Err(error) => Err(error),
        };
        outcome(result)
    }

    async fn subscribe_account_with(&self, config: maxt::Result<String>) -> Value {
        let config =
            parse_wire_text::<WireStreamConfig>(config, "config").and_then(TryInto::try_into);
        let result = match config {
            Ok(config) => match self.inner.subscribe_account_with(&config).await {
                Ok(stream) => self.streams.insert_account(stream).await,
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        };
        outcome(result)
    }

    async fn stream_next(&self, id: maxt::Result<String>) -> Value {
        let result = match parse_wire_text::<String>(id, "stream_id") {
            Ok(id) => self.streams.next(&id).await,
            Err(error) => Err(error),
        };
        outcome(result)
    }

    async fn stream_close(&self, id: maxt::Result<String>) -> Value {
        let result = match parse_wire_text::<String>(id, "stream_id") {
            Ok(id) => self.streams.close(&id).await,
            Err(error) => Err(error),
        };
        outcome(result.map(|()| Value::Null))
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
#[napi]
impl NativeClient {
    #[napi(js_name = "exchange")]
    pub fn exchange_native(&self) -> String {
        self.exchange()
    }

    #[napi(js_name = "supports")]
    pub fn supports_native(&self, feature: String) -> bool {
        self.supports(feature)
    }

    #[napi(js_name = "markets", ts_args_type = "kind: string")]
    pub fn markets_native<'env>(
        &self,
        env: &'env Env,
        kind: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let kind = native_json_text(kind, "kind");
        spawn_native(env, async move { client.markets(kind).await })
    }

    #[napi(js_name = "trades", ts_args_type = "market: string, limit: string")]
    pub fn trades_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
        limit: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let market = native_json_text(market, "market");
        let limit = native_json_text(limit, "limit");
        spawn_native(env, async move { client.trades(market, limit).await })
    }

    #[napi(js_name = "orderBook", ts_args_type = "market: string, depth: string")]
    pub fn order_book_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
        depth: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let market = native_json_text(market, "market");
        let depth = native_json_text(depth, "depth");
        spawn_native(env, async move { client.order_book(market, depth).await })
    }

    #[napi(js_name = "ticker", ts_args_type = "market: string")]
    pub fn ticker_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let market = native_json_text(market, "market");
        spawn_native(env, async move { client.ticker(market).await })
    }

    #[napi(js_name = "candles", ts_args_type = "request: string")]
    pub fn candles_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { client.candles(request).await })
    }

    #[napi(js_name = "balances")]
    pub async fn balances_native(&self) -> Value {
        self.balances().await
    }

    #[napi(js_name = "assetNetworks", ts_args_type = "asset: string")]
    pub fn asset_networks_native<'env>(
        &self,
        env: &'env Env,
        asset: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let asset = native_json_text(asset, "asset");
        spawn_native(env, async move { client.asset_networks(asset).await })
    }

    #[napi(js_name = "depositAddress", ts_args_type = "request: string")]
    pub fn deposit_address_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { client.deposit_address(request).await })
    }

    #[napi(js_name = "prepareWithdrawal", ts_args_type = "request: string")]
    pub fn prepare_withdrawal_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { client.prepare_withdrawal(request).await })
    }

    #[napi(js_name = "withdraw", ts_args_type = "request: string")]
    pub fn withdraw_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { client.withdraw(request).await })
    }

    #[napi(js_name = "deposits", ts_args_type = "request: string")]
    pub fn deposits_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { client.deposits(request).await })
    }

    #[napi(js_name = "withdrawals", ts_args_type = "request: string")]
    pub fn withdrawals_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { client.withdrawals(request).await })
    }

    #[napi(
        js_name = "prepareTransferTo",
        ts_args_type = "destination: NativeClient, request: string"
    )]
    pub fn prepare_transfer_to_native<'env>(
        &self,
        env: &'env Env,
        destination: &NativeClient,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let source = self.clone();
        let destination = destination.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move {
            source.prepare_transfer_to(&destination, request).await
        })
    }

    #[napi(js_name = "prepareTransferToChain", ts_args_type = "request: string")]
    pub fn prepare_transfer_to_chain_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move {
            client.prepare_transfer_to_chain(request).await
        })
    }

    #[napi(js_name = "executeTransfer", ts_args_type = "plan: string")]
    pub fn execute_transfer_native<'env>(
        &self,
        env: &'env Env,
        plan: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let plan = native_json_text(plan, "plan");
        spawn_native(env, async move { client.execute_transfer(plan).await })
    }

    #[napi(js_name = "openOrders")]
    pub async fn open_orders_native(&self) -> Value {
        self.open_orders().await
    }

    #[napi(js_name = "openOrdersOn", ts_args_type = "market: string")]
    pub fn open_orders_on_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let market = native_json_text(market, "market");
        spawn_native(env, async move { client.open_orders_on(market).await })
    }

    #[napi(js_name = "positions")]
    pub async fn positions_native(&self) -> Value {
        self.positions().await
    }

    #[napi(js_name = "placeOrder", ts_args_type = "request: string")]
    pub fn place_order_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { client.place_order(request).await })
    }

    #[napi(
        js_name = "cancelOrder",
        ts_args_type = "market: string, orderId: string"
    )]
    pub fn cancel_order_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
        order_id: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let market = native_json_text(market, "market");
        let order_id = native_json_text(order_id, "order_id");
        spawn_native(
            env,
            async move { client.cancel_order(market, order_id).await },
        )
    }

    #[napi(
        js_name = "cancelOrderByClientId",
        ts_args_type = "market: string, clientId: string"
    )]
    pub fn cancel_order_by_client_id_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
        client_id: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let market = native_json_text(market, "market");
        let client_id = native_json_text(client_id, "client_id");
        spawn_native(env, async move {
            client.cancel_order_by_client_id(market, client_id).await
        })
    }

    #[napi(js_name = "positionsOn", ts_args_type = "market: string")]
    pub fn positions_on_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let market = native_json_text(market, "market");
        spawn_native(env, async move { client.positions_on(market).await })
    }

    #[napi(js_name = "marginSummary")]
    pub async fn margin_summary_native(&self) -> Value {
        self.margin_summary().await
    }

    #[napi(js_name = "fundingRates", ts_args_type = "request: string")]
    pub fn funding_rates_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { client.funding_rates(request).await })
    }

    #[napi(js_name = "fundingPayments", ts_args_type = "request: string")]
    pub fn funding_payments_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { client.funding_payments(request).await })
    }

    #[napi(js_name = "setMargin", ts_args_type = "request: string")]
    pub fn set_margin_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { client.set_margin(request).await })
    }

    #[napi(js_name = "subscribe", ts_args_type = "subscription: string")]
    pub fn subscribe_native<'env>(
        &self,
        env: &'env Env,
        subscription: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let subscription = native_json_text(subscription, "subscription");
        spawn_native(env, async move { client.subscribe(subscription).await })
    }

    #[napi(
        js_name = "subscribeWith",
        ts_args_type = "subscription: string, config: string"
    )]
    pub fn subscribe_with_native<'env>(
        &self,
        env: &'env Env,
        subscription: NativeJsonText<'env>,
        config: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let subscription = native_json_text(subscription, "subscription");
        let config = native_json_text(config, "config");
        spawn_native(env, async move {
            client.subscribe_with(subscription, config).await
        })
    }

    #[napi(js_name = "subscribeAccount")]
    pub async fn subscribe_account_native(&self) -> Value {
        self.subscribe_account().await
    }

    #[napi(js_name = "subscribeAccountWith", ts_args_type = "config: string")]
    pub fn subscribe_account_with_native<'env>(
        &self,
        env: &'env Env,
        config: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let config = native_json_text(config, "config");
        spawn_native(
            env,
            async move { client.subscribe_account_with(config).await },
        )
    }

    #[napi(js_name = "streamNext", ts_args_type = "id: string")]
    pub fn stream_next_native<'env>(
        &self,
        env: &'env Env,
        id: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let id = native_json_text(id, "stream_id");
        spawn_native(env, async move { client.stream_next(id).await })
    }

    #[napi(js_name = "streamClose", ts_args_type = "id: string")]
    pub fn stream_close_native<'env>(
        &self,
        env: &'env Env,
        id: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let client = self.clone();
        let id = native_json_text(id, "stream_id");
        spawn_native(env, async move { client.stream_close(id).await })
    }
}

impl Clone for NativeClient {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            streams: Arc::clone(&self.streams),
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl NativeClient {
    #[wasm_bindgen(js_name = "exchange")]
    pub fn exchange_wasm(&self) -> String {
        self.exchange()
    }

    #[wasm_bindgen(js_name = "supports")]
    pub fn supports_wasm(&self, feature: String) -> bool {
        self.supports(feature)
    }

    #[wasm_bindgen(js_name = "markets")]
    pub async fn markets_wasm(&self, kind: String) -> JsValue {
        crate::web::value(self.markets(Ok(kind)).await)
    }

    #[wasm_bindgen(js_name = "trades")]
    pub async fn trades_wasm(&self, market: String, limit: String) -> JsValue {
        crate::web::value(self.trades(Ok(market), Ok(limit)).await)
    }

    #[wasm_bindgen(js_name = "orderBook")]
    pub async fn order_book_wasm(&self, market: String, depth: String) -> JsValue {
        crate::web::value(self.order_book(Ok(market), Ok(depth)).await)
    }

    #[wasm_bindgen(js_name = "ticker")]
    pub async fn ticker_wasm(&self, market: String) -> JsValue {
        crate::web::value(self.ticker(Ok(market)).await)
    }

    #[wasm_bindgen(js_name = "candles")]
    pub async fn candles_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.candles(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "balances")]
    pub async fn balances_wasm(&self) -> JsValue {
        crate::web::value(self.balances().await)
    }

    #[wasm_bindgen(js_name = "assetNetworks")]
    pub async fn asset_networks_wasm(&self, asset: String) -> JsValue {
        crate::web::value(self.asset_networks(Ok(asset)).await)
    }

    #[wasm_bindgen(js_name = "depositAddress")]
    pub async fn deposit_address_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.deposit_address(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "prepareWithdrawal")]
    pub async fn prepare_withdrawal_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.prepare_withdrawal(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "withdraw")]
    pub async fn withdraw_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.withdraw(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "deposits")]
    pub async fn deposits_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.deposits(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "withdrawals")]
    pub async fn withdrawals_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.withdrawals(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "prepareTransferTo")]
    pub async fn prepare_transfer_to_wasm(
        &self,
        destination: &NativeClient,
        request: String,
    ) -> JsValue {
        crate::web::value(self.prepare_transfer_to(destination, Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "prepareTransferToChain")]
    pub async fn prepare_transfer_to_chain_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.prepare_transfer_to_chain(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "executeTransfer")]
    pub async fn execute_transfer_wasm(&self, plan: String) -> JsValue {
        crate::web::value(self.execute_transfer(Ok(plan)).await)
    }

    #[wasm_bindgen(js_name = "openOrders")]
    pub async fn open_orders_wasm(&self) -> JsValue {
        crate::web::value(self.open_orders().await)
    }

    #[wasm_bindgen(js_name = "openOrdersOn")]
    pub async fn open_orders_on_wasm(&self, market: String) -> JsValue {
        crate::web::value(self.open_orders_on(Ok(market)).await)
    }

    #[wasm_bindgen(js_name = "positions")]
    pub async fn positions_wasm(&self) -> JsValue {
        crate::web::value(self.positions().await)
    }

    #[wasm_bindgen(js_name = "placeOrder")]
    pub async fn place_order_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.place_order(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "cancelOrder")]
    pub async fn cancel_order_wasm(&self, market: String, order_id: String) -> JsValue {
        crate::web::value(self.cancel_order(Ok(market), Ok(order_id)).await)
    }

    #[wasm_bindgen(js_name = "cancelOrderByClientId")]
    pub async fn cancel_order_by_client_id_wasm(
        &self,
        market: String,
        client_id: String,
    ) -> JsValue {
        crate::web::value(
            self.cancel_order_by_client_id(Ok(market), Ok(client_id))
                .await,
        )
    }

    #[wasm_bindgen(js_name = "positionsOn")]
    pub async fn positions_on_wasm(&self, market: String) -> JsValue {
        crate::web::value(self.positions_on(Ok(market)).await)
    }

    #[wasm_bindgen(js_name = "marginSummary")]
    pub async fn margin_summary_wasm(&self) -> JsValue {
        crate::web::value(self.margin_summary().await)
    }

    #[wasm_bindgen(js_name = "fundingRates")]
    pub async fn funding_rates_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.funding_rates(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "fundingPayments")]
    pub async fn funding_payments_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.funding_payments(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "setMargin")]
    pub async fn set_margin_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.set_margin(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "subscribe")]
    pub async fn subscribe_wasm(&self, subscription: String) -> JsValue {
        crate::web::value(self.subscribe(Ok(subscription)).await)
    }

    #[wasm_bindgen(js_name = "subscribeWith")]
    pub async fn subscribe_with_wasm(&self, subscription: String, config: String) -> JsValue {
        crate::web::value(self.subscribe_with(Ok(subscription), Ok(config)).await)
    }

    #[wasm_bindgen(js_name = "subscribeAccount")]
    pub async fn subscribe_account_wasm(&self) -> JsValue {
        crate::web::value(self.subscribe_account().await)
    }

    #[wasm_bindgen(js_name = "subscribeAccountWith")]
    pub async fn subscribe_account_with_wasm(&self, config: String) -> JsValue {
        crate::web::value(self.subscribe_account_with(Ok(config)).await)
    }

    #[wasm_bindgen(js_name = "streamNext")]
    pub async fn stream_next_wasm(&self, id: String) -> JsValue {
        crate::web::value(self.stream_next(Ok(id)).await)
    }

    #[wasm_bindgen(js_name = "streamClose")]
    pub async fn stream_close_wasm(&self, id: String) -> JsValue {
        crate::web::value(self.stream_close(Ok(id)).await)
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
type NativeJsonText<'env> = Either<String, Unknown<'env>>;

#[cfg(all(not(test), not(target_arch = "wasm32")))]
fn native_json_text(value: NativeJsonText<'_>, field: &str) -> maxt::Result<String> {
    match value {
        Either::A(text) => Ok(text),
        Either::B(value) => {
            let value_type = value.get_type().map_err(|error| {
                Error::adapter(format!("could not inspect native `{field}` input: {error}"))
            })?;
            Err(Error::InvalidRequest {
                field: field.to_owned(),
                detail: format!("must be a JSON text string, got {value_type}"),
            })
        }
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
fn spawn_native<'env, F>(env: &'env Env, future: F) -> napi::Result<PromiseRaw<'env, Value>>
where
    F: Future<Output = Value> + Send + 'static,
{
    env.spawn_future(async move { Ok(future.await) })
}

fn parse_wire<T, W>(value: maxt::Result<String>, field: &str) -> maxt::Result<T>
where
    W: DeserializeOwned,
    T: TryFrom<W, Error = Error>,
{
    parse_wire_text::<W>(value, field).and_then(TryInto::try_into)
}

fn parse_wire_text<T: DeserializeOwned>(
    value: maxt::Result<String>,
    field: &str,
) -> maxt::Result<T> {
    value.and_then(|text| from_wire_text(&text, field))
}

fn wire_one<T, W>(result: maxt::Result<T>) -> maxt::Result<W>
where
    W: TryFrom<T, Error = Error> + Serialize,
{
    result.and_then(TryInto::try_into)
}

fn wire_vec<T, W>(result: maxt::Result<Vec<T>>) -> maxt::Result<Vec<W>>
where
    W: TryFrom<T, Error = Error> + Serialize,
{
    result.and_then(|values| {
        values
            .into_iter()
            .map(TryInto::try_into)
            .collect::<maxt::Result<_>>()
    })
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use maxt::{
        AccountEvent, AccountStream, Adapter, AssetNetwork, Balance, BoxFuture, Candle,
        CandleRequest, Decimal, Deposit, DepositAddress, DepositAddressRequest, Exchange, Feature,
        FundingPayment, FundingRate, HistoryRequest, MarginRequest, MarginSummary, Market,
        MarketEvent, MarketInfo, MarketKind, MarketStream, Order, OrderBook, OrderRequest,
        OrderStatus, Page, Position, Side, StreamConfig, Subscription, Ticker, Timestamp, Trade,
        TransferHistoryRequest, TravelRuleRequirement, WithdrawRequest, Withdrawal,
        WithdrawalQuote, WithdrawalStatus,
    };

    use super::*;

    struct RecordingAdapter {
        calls: Arc<Mutex<Vec<&'static str>>>,
    }

    impl RecordingAdapter {
        fn order(market: Market, side: Side) -> Order {
            Order {
                id: "order-1".to_owned(),
                market,
                side,
                status: OrderStatus::Open,
                filled_quantity: Decimal::ZERO,
                remaining_quantity: Decimal::ONE,
                price: None,
                created_at: None,
            }
        }
    }

    impl Adapter for RecordingAdapter {
        fn exchange(&self) -> Exchange {
            Exchange::Binance
        }

        fn supports(&self, _feature: Feature) -> bool {
            true
        }

        fn markets(&self, _kind: MarketKind) -> BoxFuture<'_, maxt::Result<Vec<MarketInfo>>> {
            self.calls.lock().unwrap().push("markets");
            Box::pin(async { Ok(vec![]) })
        }

        fn trades(
            &self,
            _market: &Market,
            _limit: Option<u32>,
        ) -> BoxFuture<'_, maxt::Result<Vec<Trade>>> {
            self.calls.lock().unwrap().push("trades");
            Box::pin(async { Ok(vec![]) })
        }

        fn order_book(
            &self,
            market: &Market,
            _depth: Option<u32>,
        ) -> BoxFuture<'_, maxt::Result<OrderBook>> {
            self.calls.lock().unwrap().push("order_book");
            let market = market.clone();
            Box::pin(async move {
                Ok(OrderBook {
                    market,
                    timestamp: Timestamp::from_nanos(1),
                    bids: vec![],
                    asks: vec![],
                })
            })
        }

        fn ticker(&self, market: &Market) -> BoxFuture<'_, maxt::Result<Ticker>> {
            self.calls.lock().unwrap().push("ticker");
            let market = market.clone();
            Box::pin(async move {
                Ok(Ticker {
                    market,
                    timestamp: Timestamp::from_nanos(1),
                    last_trade_time: None,
                    last_price: Decimal::ONE,
                    change: None,
                    change_rate: None,
                    high: None,
                    low: None,
                    volume: None,
                    quote_volume: None,
                })
            })
        }

        fn candles(&self, _request: &CandleRequest) -> BoxFuture<'_, maxt::Result<Vec<Candle>>> {
            self.calls.lock().unwrap().push("candles");
            Box::pin(async { Ok(vec![]) })
        }

        fn balances(&self) -> BoxFuture<'_, maxt::Result<Vec<Balance>>> {
            self.calls.lock().unwrap().push("balances");
            Box::pin(async { Ok(vec![]) })
        }

        fn asset_networks(&self, _asset: &str) -> BoxFuture<'_, maxt::Result<Vec<AssetNetwork>>> {
            self.calls.lock().unwrap().push("asset_networks");
            Box::pin(async { Ok(vec![]) })
        }

        fn deposit_address(
            &self,
            request: &DepositAddressRequest,
        ) -> BoxFuture<'_, maxt::Result<DepositAddress>> {
            self.calls.lock().unwrap().push("deposit_address");
            let result = DepositAddress {
                exchange: Exchange::Binance,
                asset: request.asset.clone(),
                network: request.network.clone(),
                address: Some("bc1qdestination".to_owned()),
                memo: None,
            };
            Box::pin(async move { Ok(result) })
        }

        fn prepare_withdrawal(
            &self,
            _request: &WithdrawRequest,
        ) -> BoxFuture<'_, maxt::Result<WithdrawalQuote>> {
            self.calls.lock().unwrap().push("prepare_withdrawal");
            Box::pin(async {
                Ok(WithdrawalQuote {
                    fee: Some(Decimal::new(1, 4)),
                    expected_receive: None,
                    minimum_amount: None,
                    maximum_amount: None,
                    address_allowed: Some(true),
                    travel_rule: TravelRuleRequirement::NotRequired,
                    expires_at: None,
                })
            })
        }

        fn withdraw(&self, request: &WithdrawRequest) -> BoxFuture<'_, maxt::Result<Withdrawal>> {
            self.calls.lock().unwrap().push("withdraw");
            let result = Withdrawal {
                id: "withdrawal-1".to_owned(),
                asset: request.asset.clone(),
                network: Some(request.network.clone()),
                provider_network: Some(request.network.id().to_owned()),
                amount: request.amount,
                fee: None,
                destination: Some(request.destination.clone()),
                status: WithdrawalStatus::Pending,
                provider_status: "accepted".to_owned(),
                tx_id: None,
                created_at: None,
            };
            Box::pin(async move { Ok(result) })
        }

        fn deposits(
            &self,
            _request: &TransferHistoryRequest,
        ) -> BoxFuture<'_, maxt::Result<Page<Deposit>>> {
            self.calls.lock().unwrap().push("deposits");
            Box::pin(async {
                Ok(Page {
                    items: vec![],
                    next: None,
                })
            })
        }

        fn withdrawals(
            &self,
            _request: &TransferHistoryRequest,
        ) -> BoxFuture<'_, maxt::Result<Page<Withdrawal>>> {
            self.calls.lock().unwrap().push("withdrawals");
            Box::pin(async {
                Ok(Page {
                    items: vec![],
                    next: None,
                })
            })
        }

        fn open_orders(&self, market: Option<&Market>) -> BoxFuture<'_, maxt::Result<Vec<Order>>> {
            self.calls.lock().unwrap().push(if market.is_some() {
                "open_orders:some"
            } else {
                "open_orders:none"
            });
            Box::pin(async { Ok(vec![]) })
        }

        fn positions(&self, market: Option<&Market>) -> BoxFuture<'_, maxt::Result<Vec<Position>>> {
            self.calls.lock().unwrap().push(if market.is_some() {
                "positions:some"
            } else {
                "positions:none"
            });
            let market = Market::perpetual(Exchange::Binance, "BTC", "USDT");
            Box::pin(async move {
                Ok([Decimal::ZERO, Decimal::ONE]
                    .into_iter()
                    .map(|quantity| Position {
                        market: market.clone(),
                        side: (!quantity.is_zero()).then_some(Side::Buy),
                        quantity,
                        entry_price: None,
                        mark_price: None,
                        notional: None,
                        unrealized_pnl: None,
                        leverage: None,
                        margin_mode: None,
                    })
                    .collect())
            })
        }

        fn place_order(&self, request: &OrderRequest) -> BoxFuture<'_, maxt::Result<Order>> {
            self.calls.lock().unwrap().push("place_order");
            let order = Self::order(request.market.clone(), request.side);
            Box::pin(async move { Ok(order) })
        }

        fn cancel_order(
            &self,
            _market: &Market,
            _order_id: &str,
        ) -> BoxFuture<'_, maxt::Result<()>> {
            self.calls.lock().unwrap().push("cancel_order");
            Box::pin(async { Ok(()) })
        }

        fn cancel_order_by_client_id(
            &self,
            _market: &Market,
            _client_id: &str,
        ) -> BoxFuture<'_, maxt::Result<()>> {
            self.calls.lock().unwrap().push("cancel_order_by_client_id");
            Box::pin(async { Ok(()) })
        }

        fn margin_summary(&self) -> BoxFuture<'_, maxt::Result<MarginSummary>> {
            self.calls.lock().unwrap().push("margin_summary");
            Box::pin(async {
                Ok(MarginSummary {
                    asset: "USDT".to_owned(),
                    equity: None,
                    margin_balance: None,
                    available_balance: None,
                })
            })
        }

        fn funding_rates(
            &self,
            _request: &HistoryRequest,
        ) -> BoxFuture<'_, maxt::Result<Page<FundingRate>>> {
            self.calls.lock().unwrap().push("funding_rates");
            Box::pin(async {
                Ok(Page {
                    items: vec![],
                    next: None,
                })
            })
        }

        fn funding_payments(
            &self,
            _request: &HistoryRequest,
        ) -> BoxFuture<'_, maxt::Result<Page<FundingPayment>>> {
            self.calls.lock().unwrap().push("funding_payments");
            Box::pin(async {
                Ok(Page {
                    items: vec![],
                    next: None,
                })
            })
        }

        fn set_margin(&self, _request: &MarginRequest) -> BoxFuture<'_, maxt::Result<()>> {
            self.calls.lock().unwrap().push("set_margin");
            Box::pin(async { Ok(()) })
        }

        fn subscribe(
            &self,
            _subscription: &Subscription,
            config: &StreamConfig,
        ) -> BoxFuture<'_, maxt::Result<MarketStream>> {
            self.calls
                .lock()
                .unwrap()
                .push(if config.buffer_size == 4_096 {
                    "subscribe:default"
                } else {
                    "subscribe:custom"
                });
            Box::pin(async {
                Ok(MarketStream::new(futures_util::stream::iter([Ok(
                    MarketEvent::Reconnected,
                )])))
            })
        }

        fn subscribe_account(
            &self,
            config: &StreamConfig,
        ) -> BoxFuture<'_, maxt::Result<AccountStream>> {
            self.calls
                .lock()
                .unwrap()
                .push(if config.buffer_size == 4_096 {
                    "subscribe_account:default"
                } else {
                    "subscribe_account:custom"
                });
            Box::pin(async {
                Ok(AccountStream::new(futures_util::stream::iter([Ok(
                    AccountEvent::Reconnected,
                )])))
            })
        }
    }

    #[tokio::test]
    async fn native_client_uses_core_filtering_and_optional_market_calls() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let client = NativeClient::for_test(Box::new(RecordingAdapter {
            calls: Arc::clone(&calls),
        }));

        let value = client.positions().await;
        assert_eq!(value["ok"], true);
        assert_eq!(value["value"].as_array().unwrap().len(), 1);

        assert_eq!(client.open_orders().await["ok"], true);
        assert_eq!(
            client.open_orders_on(json_text(market_wire())).await["ok"],
            true
        );
        assert_eq!(
            *calls.lock().unwrap(),
            vec!["positions:none", "open_orders:none", "open_orders:some"]
        );
    }

    #[tokio::test]
    async fn native_client_rejects_invalid_json_text_before_adapter_dispatch() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let client = NativeClient::for_test(Box::new(RecordingAdapter {
            calls: Arc::clone(&calls),
        }));

        let deep = format!("{}null{}", "[".repeat(65), "]".repeat(65));
        let non_string = Err(Error::InvalidRequest {
            field: "kind".to_owned(),
            detail: "must be a JSON text string, got Undefined".to_owned(),
        });
        for input in [Ok("undefined".to_owned()), Ok(deep), non_string] {
            let value = client.markets(input).await;
            assert_eq!(value["ok"], false);
            assert_eq!(value["error"]["kind"], "invalid_request");
            assert_eq!(value["error"]["field"], "kind");
        }
        assert!(calls.lock().unwrap().is_empty());
    }

    fn market_wire() -> serde_json::Value {
        serde_json::json!({
            "exchange": "binance",
            "kind": "perpetual",
            "base": "BTC",
            "quote": "USDT"
        })
    }

    fn json_text(value: Value) -> maxt::Result<String> {
        Ok(serde_json::to_string(&value).unwrap())
    }

    #[tokio::test]
    async fn native_client_uses_core_stream_defaults_and_its_own_registry() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let client = NativeClient::for_test(Box::new(RecordingAdapter {
            calls: Arc::clone(&calls),
        }));
        let subscription = serde_json::json!({
            "markets": [market_wire()],
            "feeds": [{ "kind": "trades" }]
        });
        let config = serde_json::json!({
            "max_reconnect_attempts": null,
            "initial_reconnect_delay_ms": "1",
            "max_reconnect_delay_ms": "2",
            "idle_timeout_ms": "3",
            "buffer_size": "1",
            "overflow": "backpressure"
        });

        let market_default = client.subscribe(json_text(subscription.clone())).await;
        let market_custom = client
            .subscribe_with(json_text(subscription), json_text(config.clone()))
            .await;
        let account_default = client.subscribe_account().await;
        let account_custom = client.subscribe_account_with(json_text(config)).await;

        for handle in [
            &market_default["value"],
            &market_custom["value"],
            &account_default["value"],
            &account_custom["value"],
        ] {
            assert_eq!(
                client.stream_next(json_text(handle["id"].clone())).await["ok"],
                true
            );
            assert_eq!(
                client.stream_close(json_text(handle["id"].clone())).await["ok"],
                true
            );
        }
        assert_eq!(
            *calls.lock().unwrap(),
            vec![
                "subscribe:default",
                "subscribe:custom",
                "subscribe_account:default",
                "subscribe_account:custom",
            ]
        );
    }

    #[tokio::test]
    async fn native_client_forwards_every_non_stream_operation() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let client = NativeClient::for_test(Box::new(RecordingAdapter {
            calls: Arc::clone(&calls),
        }));
        let market = market_wire();
        let candle_request = serde_json::json!({
            "market": market,
            "interval": "min1",
            "from": null,
            "to": null,
            "limit": null
        });
        let history_request = serde_json::json!({
            "market": market,
            "from": null,
            "to": null,
            "cursor": null,
            "limit": null
        });
        let order_request = serde_json::json!({
            "market": market,
            "side": "buy",
            "order_type": "market",
            "size": { "kind": "base", "value": "1.00" },
            "price": null,
            "time_in_force": null,
            "reduce_only": false
        });
        let margin_request = serde_json::json!({
            "market": market,
            "leverage": "10.0",
            "margin_mode": null
        });
        let deposit_address_request = serde_json::json!({
            "asset": "BTC",
            "network": "bitcoin",
            "amount": null
        });
        let withdraw_request = serde_json::json!({
            "asset": "BTC",
            "network": "bitcoin",
            "amount": "1.0",
            "destination": {
                "kind": "chain",
                "value": {
                    "asset": "BTC",
                    "network": "bitcoin",
                    "address": "bc1qdestination",
                    "memo": null
                }
            },
            "client_id": null
        });
        let transfer_history_request = serde_json::json!({
            "asset": null,
            "network": null,
            "cursor": null,
            "limit": null
        });

        assert_eq!(client.exchange(), "binance");
        assert!(client.supports("ticker".to_owned()));
        let results = [
            client
                .markets(Ok(serde_json::json!("spot").to_string()))
                .await,
            client
                .trades(json_text(market.clone()), json_text(Value::Null))
                .await,
            client
                .order_book(json_text(market.clone()), json_text(Value::Null))
                .await,
            client.ticker(json_text(market.clone())).await,
            client.candles(json_text(candle_request)).await,
            client.balances().await,
            client
                .asset_networks(json_text(serde_json::json!("BTC")))
                .await,
            client
                .deposit_address(json_text(deposit_address_request))
                .await,
            client
                .prepare_withdrawal(json_text(withdraw_request.clone()))
                .await,
            client.withdraw(json_text(withdraw_request)).await,
            client
                .deposits(json_text(transfer_history_request.clone()))
                .await,
            client
                .withdrawals(json_text(transfer_history_request))
                .await,
            client.open_orders().await,
            client.open_orders_on(json_text(market.clone())).await,
            client.place_order(json_text(order_request)).await,
            client
                .cancel_order(
                    json_text(market.clone()),
                    json_text(serde_json::json!("order-1")),
                )
                .await,
            client
                .cancel_order_by_client_id(
                    json_text(market.clone()),
                    json_text(serde_json::json!("client-1")),
                )
                .await,
            client.positions().await,
            client.positions_on(json_text(market)).await,
            client.margin_summary().await,
            client
                .funding_rates(json_text(history_request.clone()))
                .await,
            client.funding_payments(json_text(history_request)).await,
            client.set_margin(json_text(margin_request)).await,
        ];
        assert!(results.iter().all(|value| value["ok"] == true));
        assert_eq!(
            *calls.lock().unwrap(),
            vec![
                "markets",
                "trades",
                "order_book",
                "ticker",
                "candles",
                "balances",
                "asset_networks",
                "deposit_address",
                "prepare_withdrawal",
                "withdraw",
                "deposits",
                "withdrawals",
                "open_orders:none",
                "open_orders:some",
                "place_order",
                "cancel_order",
                "cancel_order_by_client_id",
                "positions:none",
                "positions:some",
                "margin_summary",
                "funding_rates",
                "funding_payments",
                "set_margin",
            ]
        );
    }
}
