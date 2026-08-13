use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(all(not(test), not(target_arch = "wasm32")))]
use std::future::Future;

#[cfg(test)]
use maxt::Adapter;
use maxt::adapters::{
    BinanceAdapter, BinanceAggregateTradesRequest, BinanceC2cTradeHistoryRequest, BinanceListenKey,
    BinanceMarket, BinanceTestOrderRequest, BithumbAdapter, BithumbBatchOrdersRequest,
    BithumbClosedOrdersRequest, BithumbKrwDepositsRequest, BithumbKrwTransferRequest,
    BithumbKrwWithdrawalsRequest, BithumbOrderDetailRequest, BithumbOrderListRequest,
    BithumbPendingOrdersRequest, BithumbTwapOrderRequest, BithumbTwapOrdersRequest,
    HyperliquidAdapter, UpbitAdapter, UpbitCancelAndNewOrderRequest, UpbitClosedOrdersRequest,
    UpbitKrwTransferRequest, UpbitOrderDetailRequest, UpbitPocketApiKeysRequest,
    UpbitPocketTransferQuery, UpbitPocketTransferRequest, UpbitPocketUniversalTransferRequest,
    UpbitRegion,
};
use maxt::{Cursor, Error, HyperliquidOrderReference, Market};
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use napi::bindgen_prelude::{Either, PromiseRaw};
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use napi::{Env, Unknown};
#[cfg(all(not(test), not(target_arch = "wasm32")))]
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::client::NativeClient;
use crate::convert::{
    WireBinanceAccountTrade, WireBinanceAggregateTrade, WireBinanceAggregateTradesRequest,
    WireBinanceC2cTradeHistoryPage, WireBinanceC2cTradeHistoryRequest, WireBinanceMarkPrice,
    WireBinanceOpenInterest, WireBinanceSpotAveragePrice, WireBinanceSpotOrderDetail,
    WireBinanceSymbolFilters, WireBinanceTestOrder, WireBinanceTestOrderRequest, WireBithumbApiKey,
    WireBithumbAssetFee, WireBithumbBatchOrdersRequest, WireBithumbBatchOrdersResult,
    WireBithumbClosedOrder, WireBithumbClosedOrdersRequest, WireBithumbKrwDeposit,
    WireBithumbKrwDepositsRequest, WireBithumbKrwTransferRequest, WireBithumbKrwWithdrawal,
    WireBithumbKrwWithdrawalsRequest, WireBithumbMarketAlert, WireBithumbNotice,
    WireBithumbOrderDetail, WireBithumbOrderDetailRequest, WireBithumbOrderListItem,
    WireBithumbOrderListRequest, WireBithumbPendingOrdersRequest, WireBithumbTwapOrder,
    WireBithumbTwapOrderRequest, WireBithumbTwapOrdersRequest, WireBithumbWithdrawalAddress,
    WireCancelOrdersResult, WireHyperliquidAssetContext, WireHyperliquidLedgerEntry,
    WireHyperliquidMidPrice, WireHyperliquidOpenOrder, WireHyperliquidOrderInfo,
    WireHyperliquidOrderReference, WireHyperliquidOrderStatusResponse,
    WireHyperliquidPortfolioPeriod, WireHyperliquidReferral, WireHyperliquidSubAccount,
    WireHyperliquidUserFees, WireHyperliquidUserFill, WireHyperliquidUserRateLimit,
    WireHyperliquidUserRole, WireHyperliquidVaultEquity, WireMarket, WireOrder, WireOrderBook,
    WireOrderRequest, WirePage, WireTicker, WireUpbitApiKey, WireUpbitBatchCancelRequest,
    WireUpbitCancelAndNewOrderRequest, WireUpbitCancelAndNewOrderResult, WireUpbitClosedOrder,
    WireUpbitClosedOrdersRequest, WireUpbitDepositInfo, WireUpbitKrwDeposit,
    WireUpbitKrwTransferRequest, WireUpbitKrwWithdrawal, WireUpbitMarketEvent,
    WireUpbitOrderBookInstrument, WireUpbitOrderDetail, WireUpbitOrderDetailRequest,
    WireUpbitPocket, WireUpbitPocketApiKeyGroup, WireUpbitPocketApiKeysRequest,
    WireUpbitPocketBalance, WireUpbitPocketTransfer, WireUpbitPocketTransferQuery,
    WireUpbitPocketTransferRequest, WireUpbitPocketUniversalTransferRequest,
    WireUpbitSubscriptionList, WireUpbitTravelRuleVasp, WireUpbitTravelRuleVerification,
    WireUpbitYearCandle, decimal_from_wire, from_wire_text, network_from_wire, outcome,
    timestamp_from_wire,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpbitOptions {
    region: String,
    #[serde(deserialize_with = "explicit_option")]
    access_key: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    secret_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BithumbOptions {
    #[serde(deserialize_with = "explicit_option")]
    access_key: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    secret_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BinanceOptions {
    venue: String,
    #[serde(deserialize_with = "explicit_option")]
    api_key: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    secret_key: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HyperliquidOptions {
    testnet: bool,
    #[serde(deserialize_with = "explicit_option")]
    address: Option<String>,
    #[serde(deserialize_with = "explicit_option")]
    private_key: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(
    test,
    allow(dead_code, reason = "네이티브 생성 함수는 테스트 빌드에서 제외됩니다")
)]
struct WireBinanceListenKey {
    id: String,
    value: String,
}

fn explicit_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}

fn credential_pair(
    first: Option<String>,
    second: Option<String>,
    first_name: &str,
    second_name: &str,
) -> maxt::Result<Option<(String, String)>> {
    match (first, second) {
        (None, None) => Ok(None),
        (Some(first), Some(second)) => Ok(Some((first, second))),
        _ => Err(Error::InvalidRequest {
            field: "credentials".to_owned(),
            detail: format!("{first_name} and {second_name} must be provided together"),
        }),
    }
}

fn parse_options<T: for<'de> Deserialize<'de>>(options: maxt::Result<String>) -> maxt::Result<T> {
    options.and_then(|options| from_wire_text(&options, "options"))
}

#[cfg_attr(
    test,
    allow(dead_code, reason = "네이티브 메서드는 테스트 빌드에서 제외됩니다")
)]
fn parse_wire<T: TryFrom<W, Error = Error>, W: for<'de> Deserialize<'de>>(
    value: maxt::Result<String>,
    field: &str,
) -> maxt::Result<T> {
    value
        .and_then(|value| from_wire_text::<W>(&value, field))
        .and_then(TryInto::try_into)
}

fn parse_text<T: for<'de> Deserialize<'de>>(
    value: maxt::Result<String>,
    field: &str,
) -> maxt::Result<T> {
    value.and_then(|value| from_wire_text(&value, field))
}

fn wire_vec<T, W>(result: maxt::Result<Vec<T>>) -> maxt::Result<Vec<W>>
where
    W: TryFrom<T, Error = Error> + Serialize,
{
    result.and_then(|values| values.into_iter().map(TryInto::try_into).collect())
}

#[cfg_attr(
    test,
    allow(dead_code, reason = "네이티브 메서드는 테스트 빌드에서 제외됩니다")
)]
fn wire_pairs<T, W>(result: maxt::Result<Vec<(Market, T)>>) -> maxt::Result<Vec<(WireMarket, W)>>
where
    W: TryFrom<T, Error = Error> + Serialize,
{
    result.and_then(|values| {
        values
            .into_iter()
            .map(|(market, value)| Ok((market.try_into()?, value.try_into()?)))
            .collect()
    })
}

#[cfg_attr(all(not(test), not(target_arch = "wasm32")), napi)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct NativeUpbit {
    adapter: Arc<UpbitAdapter>,
}

#[cfg_attr(
    test,
    allow(dead_code, reason = "네이티브 메서드는 테스트 빌드에서 제외됩니다")
)]
impl NativeUpbit {
    fn create(options: maxt::Result<String>) -> maxt::Result<Self> {
        let options: UpbitOptions = parse_options(options)?;
        let region = match options.region.as_str() {
            "korea" => UpbitRegion::Korea,
            "singapore" => UpbitRegion::Singapore,
            "indonesia" => UpbitRegion::Indonesia,
            "thailand" => UpbitRegion::Thailand,
            value => return Err(invalid_enum("options.region", value)),
        };
        let mut adapter = UpbitAdapter::with_region(region);
        if let Some((access_key, secret_key)) = credential_pair(
            options.access_key,
            options.secret_key,
            "access_key",
            "secret_key",
        )? {
            adapter = adapter.with_credentials(access_key, secret_key);
        }
        Ok(Self {
            adapter: Arc::new(adapter),
        })
    }

    fn client(&self) -> NativeClient {
        NativeClient::from_boxed(Box::new((*self.adapter).clone()))
    }

    fn region(&self) -> &'static str {
        match self.adapter.region() {
            UpbitRegion::Korea => "korea",
            UpbitRegion::Singapore => "singapore",
            UpbitRegion::Indonesia => "indonesia",
            UpbitRegion::Thailand => "thailand",
            _ => "unknown",
        }
    }

    async fn order_books(
        &self,
        markets: maxt::Result<String>,
        depth: maxt::Result<String>,
    ) -> Value {
        let markets = parse_text::<Vec<WireMarket>>(markets, "markets").and_then(|markets| {
            markets
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<Vec<Market>>>()
        });
        let depth = parse_text::<Option<u32>>(depth, "depth");
        match (markets, depth) {
            (Ok(markets), Ok(depth)) => outcome(wire_vec::<_, WireOrderBook>(
                self.adapter.order_books(&markets, depth).await,
            )),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    async fn order_books_at_level(
        &self,
        markets: maxt::Result<String>,
        level: maxt::Result<String>,
        depth: maxt::Result<String>,
    ) -> Value {
        let markets = parse_text::<Vec<WireMarket>>(markets, "markets").and_then(|markets| {
            markets
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<Vec<Market>>>()
        });
        let level = parse_text::<String>(level, "level")
            .and_then(|level| decimal_from_wire(&level, "level"));
        let depth = parse_text::<Option<u32>>(depth, "depth");
        match (markets, level, depth) {
            (Ok(markets), Ok(level), Ok(depth)) => outcome(wire_vec::<_, WireOrderBook>(
                self.adapter
                    .order_books_at_level(&markets, level, depth)
                    .await,
            )),
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                outcome::<Value>(Err(error))
            }
        }
    }

    async fn tickers(&self, markets: maxt::Result<String>) -> Value {
        let markets = parse_text::<Vec<WireMarket>>(markets, "markets").and_then(|markets| {
            markets
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<Vec<Market>>>()
        });
        match markets {
            Ok(markets) => outcome(wire_vec::<_, WireTicker>(
                self.adapter.tickers(&markets).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn tickers_by_quote(&self, quote_currencies: maxt::Result<String>) -> Value {
        match parse_text::<Vec<String>>(quote_currencies, "quote_currencies") {
            Ok(quote_currencies) => outcome(wire_vec::<_, WireTicker>(
                self.adapter.tickers_by_quote(&quote_currencies).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn year_candles(
        &self,
        market: maxt::Result<String>,
        to: maxt::Result<String>,
        count: maxt::Result<String>,
    ) -> Value {
        let market = parse_wire::<Market, WireMarket>(market, "market");
        let to = parse_text::<Option<String>>(to, "to").and_then(|value| {
            value
                .as_deref()
                .map(|value| timestamp_from_wire(value, "to"))
                .transpose()
        });
        let count = parse_text::<Option<u32>>(count, "count");
        match (market, to, count) {
            (Ok(market), Ok(to), Ok(count)) => outcome(wire_vec::<_, WireUpbitYearCandle>(
                self.adapter.year_candles(&market, to, count).await,
            )),
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                outcome::<Value>(Err(error))
            }
        }
    }

    async fn orderbook_instruments(&self, markets: maxt::Result<String>) -> Value {
        let markets = parse_text::<Vec<WireMarket>>(markets, "markets").and_then(|markets| {
            markets
                .into_iter()
                .map(TryInto::try_into)
                .collect::<maxt::Result<Vec<Market>>>()
        });
        match markets {
            Ok(markets) => outcome(wire_vec::<_, WireUpbitOrderBookInstrument>(
                self.adapter.orderbook_instruments(&markets).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn market_events(&self) -> Value {
        outcome(wire_pairs::<_, WireUpbitMarketEvent>(
            self.adapter.market_events().await,
        ))
    }

    async fn list_subscriptions(&self, subscription: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::Subscription, crate::convert::WireSubscription>(
            subscription,
            "subscription",
        ) {
            Ok(subscription) => outcome(
                self.adapter
                    .list_subscriptions(&subscription)
                    .await
                    .and_then(TryInto::<WireUpbitSubscriptionList>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn test_order(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::OrderRequest, WireOrderRequest>(request, "request") {
            Ok(request) => outcome(
                self.adapter
                    .test_order(&request)
                    .await
                    .and_then(TryInto::<WireOrder>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn deposit_info(
        &self,
        asset: maxt::Result<String>,
        network: maxt::Result<String>,
    ) -> Value {
        let asset = parse_text::<String>(asset, "asset");
        let network =
            parse_text::<String>(network, "network").map(|network| network_from_wire(&network));
        match (asset, network) {
            (Ok(asset), Ok(network)) => outcome(
                self.adapter
                    .deposit_info(&asset, &network)
                    .await
                    .and_then(TryInto::<WireUpbitDepositInfo>::try_into),
            ),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    async fn travel_rule_vasps(&self) -> Value {
        outcome(wire_vec::<_, WireUpbitTravelRuleVasp>(
            self.adapter.travel_rule_vasps().await,
        ))
    }

    async fn verify_travel_rule_by_uuid(
        &self,
        deposit_uuid: maxt::Result<String>,
        vasp_uuid: maxt::Result<String>,
    ) -> Value {
        match (
            parse_text::<String>(deposit_uuid, "deposit_uuid"),
            parse_text::<String>(vasp_uuid, "vasp_uuid"),
        ) {
            (Ok(deposit_uuid), Ok(vasp_uuid)) => outcome(
                self.adapter
                    .verify_travel_rule_by_uuid(&deposit_uuid, &vasp_uuid)
                    .await
                    .and_then(TryInto::<WireUpbitTravelRuleVerification>::try_into),
            ),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    async fn verify_travel_rule_by_txid(
        &self,
        txid: maxt::Result<String>,
        vasp_uuid: maxt::Result<String>,
        currency: maxt::Result<String>,
        net_type: maxt::Result<String>,
    ) -> Value {
        let txid = parse_text::<String>(txid, "txid");
        let vasp_uuid = parse_text::<String>(vasp_uuid, "vasp_uuid");
        let currency = parse_text::<String>(currency, "currency");
        let net_type = parse_text::<String>(net_type, "net_type");
        match (txid, vasp_uuid, currency, net_type) {
            (Ok(txid), Ok(vasp_uuid), Ok(currency), Ok(net_type)) => outcome(
                self.adapter
                    .verify_travel_rule_by_txid(&txid, &vasp_uuid, &currency, &net_type)
                    .await
                    .and_then(TryInto::<WireUpbitTravelRuleVerification>::try_into),
            ),
            (Err(error), _, _, _)
            | (_, Err(error), _, _)
            | (_, _, Err(error), _)
            | (_, _, _, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    async fn batch_cancel_open_orders(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::UpbitBatchCancelRequest, WireUpbitBatchCancelRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .batch_cancel_open_orders(&request)
                    .await
                    .and_then(TryInto::<WireCancelOrdersResult>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn cancel_and_new_order(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<UpbitCancelAndNewOrderRequest, WireUpbitCancelAndNewOrderRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .cancel_and_new_order(&request)
                    .await
                    .and_then(TryInto::<WireUpbitCancelAndNewOrderResult>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn deposit_krw(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<UpbitKrwTransferRequest, WireUpbitKrwTransferRequest>(request, "request")
        {
            Ok(request) => outcome(
                self.adapter
                    .deposit_krw(&request)
                    .await
                    .and_then(TryInto::<WireUpbitKrwDeposit>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn withdraw_krw(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<UpbitKrwTransferRequest, WireUpbitKrwTransferRequest>(request, "request")
        {
            Ok(request) => outcome(
                self.adapter
                    .withdraw_krw(&request)
                    .await
                    .and_then(TryInto::<WireUpbitKrwWithdrawal>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn api_keys(&self) -> Value {
        outcome(wire_vec::<_, WireUpbitApiKey>(
            self.adapter.api_keys().await,
        ))
    }

    async fn list_pockets(&self) -> Value {
        outcome(wire_vec::<_, WireUpbitPocket>(
            self.adapter.list_pockets().await,
        ))
    }

    async fn list_pocket_api_keys(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<UpbitPocketApiKeysRequest, WireUpbitPocketApiKeysRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_vec::<_, WireUpbitPocketApiKeyGroup>(
                self.adapter.list_pocket_api_keys(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn sub_pocket_balances(&self, pocket_uuid: maxt::Result<String>) -> Value {
        match parse_text::<String>(pocket_uuid, "pocket_uuid") {
            Ok(pocket_uuid) => outcome(wire_vec::<_, WireUpbitPocketBalance>(
                self.adapter.sub_pocket_balances(&pocket_uuid).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn universal_transfer(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<
            UpbitPocketUniversalTransferRequest,
            WireUpbitPocketUniversalTransferRequest,
        >(request, "request")
        {
            Ok(request) => outcome(
                self.adapter
                    .universal_transfer(&request)
                    .await
                    .and_then(TryInto::<WireUpbitPocketTransfer>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn universal_transfers(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<UpbitPocketTransferQuery, WireUpbitPocketTransferQuery>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_vec::<_, WireUpbitPocketTransfer>(
                self.adapter.universal_transfers(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn sub_pocket_transfer(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<UpbitPocketTransferRequest, WireUpbitPocketTransferRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .sub_pocket_transfer(&request)
                    .await
                    .and_then(TryInto::<WireUpbitPocketTransfer>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn sub_pocket_transfers(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<UpbitPocketTransferQuery, WireUpbitPocketTransferQuery>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_vec::<_, WireUpbitPocketTransfer>(
                self.adapter.sub_pocket_transfers(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn order_detail(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<UpbitOrderDetailRequest, WireUpbitOrderDetailRequest>(request, "request")
        {
            Ok(request) => outcome(
                self.adapter
                    .order_detail(&request)
                    .await
                    .and_then(TryInto::<WireUpbitOrderDetail>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn closed_orders(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<UpbitClosedOrdersRequest, WireUpbitClosedOrdersRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_vec::<_, WireUpbitClosedOrder>(
                self.adapter.closed_orders(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
#[napi]
impl NativeUpbit {
    #[napi(js_name = "client")]
    pub fn client_native(&self) -> NativeClient {
        self.client()
    }

    #[napi(js_name = "region")]
    pub fn region_native(&self) -> String {
        self.region().to_owned()
    }

    #[napi(
        js_name = "orderBooks",
        ts_args_type = "markets: string, depth: string"
    )]
    pub fn order_books_native<'env>(
        &self,
        env: &'env Env,
        markets: NativeJsonText<'env>,
        depth: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let markets = native_json_text(markets, "markets");
        let depth = native_json_text(depth, "depth");
        spawn_native(env, async move { this.order_books(markets, depth).await })
    }

    #[napi(
        js_name = "orderBooksAtLevel",
        ts_args_type = "markets: string, level: string, depth: string"
    )]
    pub fn order_books_at_level_native<'env>(
        &self,
        env: &'env Env,
        markets: NativeJsonText<'env>,
        level: NativeJsonText<'env>,
        depth: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let markets = native_json_text(markets, "markets");
        let level = native_json_text(level, "level");
        let depth = native_json_text(depth, "depth");
        spawn_native(env, async move {
            this.order_books_at_level(markets, level, depth).await
        })
    }

    #[napi(js_name = "tickers", ts_args_type = "markets: string")]
    pub fn tickers_native<'env>(
        &self,
        env: &'env Env,
        markets: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let markets = native_json_text(markets, "markets");
        spawn_native(env, async move { this.tickers(markets).await })
    }

    #[napi(js_name = "tickersByQuote", ts_args_type = "quoteCurrencies: string")]
    pub fn tickers_by_quote_native<'env>(
        &self,
        env: &'env Env,
        quote_currencies: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let quote_currencies = native_json_text(quote_currencies, "quote_currencies");
        spawn_native(
            env,
            async move { this.tickers_by_quote(quote_currencies).await },
        )
    }

    #[napi(
        js_name = "yearCandles",
        ts_args_type = "market: string, to: string, count: string"
    )]
    pub fn year_candles_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
        to: NativeJsonText<'env>,
        count: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let market = native_json_text(market, "market");
        let to = native_json_text(to, "to");
        let count = native_json_text(count, "count");
        spawn_native(
            env,
            async move { this.year_candles(market, to, count).await },
        )
    }

    #[napi(js_name = "orderbookInstruments", ts_args_type = "markets: string")]
    pub fn orderbook_instruments_native<'env>(
        &self,
        env: &'env Env,
        markets: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let markets = native_json_text(markets, "markets");
        spawn_native(
            env,
            async move { this.orderbook_instruments(markets).await },
        )
    }

    #[napi(js_name = "marketEvents")]
    pub async fn market_events_native(&self) -> Value {
        self.market_events().await
    }

    #[napi(js_name = "listSubscriptions", ts_args_type = "subscription: string")]
    pub fn list_subscriptions_native<'env>(
        &self,
        env: &'env Env,
        subscription: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let subscription = native_json_text(subscription, "subscription");
        spawn_native(
            env,
            async move { this.list_subscriptions(subscription).await },
        )
    }

    #[napi(js_name = "testOrder", ts_args_type = "request: string")]
    pub fn test_order_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.test_order(request).await })
    }

    #[napi(
        js_name = "depositInfo",
        ts_args_type = "asset: string, network: string"
    )]
    pub fn deposit_info_native<'env>(
        &self,
        env: &'env Env,
        asset: NativeJsonText<'env>,
        network: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let asset = native_json_text(asset, "asset");
        let network = native_json_text(network, "network");
        spawn_native(env, async move { this.deposit_info(asset, network).await })
    }

    #[napi(js_name = "travelRuleVasps")]
    pub async fn travel_rule_vasps_native(&self) -> Value {
        self.travel_rule_vasps().await
    }

    #[napi(
        js_name = "verifyTravelRuleByUuid",
        ts_args_type = "depositUuid: string, vaspUuid: string"
    )]
    pub fn verify_travel_rule_by_uuid_native<'env>(
        &self,
        env: &'env Env,
        deposit_uuid: NativeJsonText<'env>,
        vasp_uuid: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let deposit_uuid = native_json_text(deposit_uuid, "deposit_uuid");
        let vasp_uuid = native_json_text(vasp_uuid, "vasp_uuid");
        spawn_native(env, async move {
            this.verify_travel_rule_by_uuid(deposit_uuid, vasp_uuid)
                .await
        })
    }

    #[napi(
        js_name = "verifyTravelRuleByTxid",
        ts_args_type = "txid: string, vaspUuid: string, currency: string, netType: string"
    )]
    pub fn verify_travel_rule_by_txid_native<'env>(
        &self,
        env: &'env Env,
        txid: NativeJsonText<'env>,
        vasp_uuid: NativeJsonText<'env>,
        currency: NativeJsonText<'env>,
        net_type: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let txid = native_json_text(txid, "txid");
        let vasp_uuid = native_json_text(vasp_uuid, "vasp_uuid");
        let currency = native_json_text(currency, "currency");
        let net_type = native_json_text(net_type, "net_type");
        spawn_native(env, async move {
            this.verify_travel_rule_by_txid(txid, vasp_uuid, currency, net_type)
                .await
        })
    }

    #[napi(js_name = "batchCancelOpenOrders", ts_args_type = "request: string")]
    pub fn batch_cancel_open_orders_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(
            env,
            async move { this.batch_cancel_open_orders(request).await },
        )
    }

    #[napi(js_name = "cancelAndNewOrder", ts_args_type = "request: string")]
    pub fn cancel_and_new_order_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.cancel_and_new_order(request).await })
    }

    #[napi(js_name = "depositKrw", ts_args_type = "request: string")]
    pub fn deposit_krw_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.deposit_krw(request).await })
    }

    #[napi(js_name = "withdrawKrw", ts_args_type = "request: string")]
    pub fn withdraw_krw_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.withdraw_krw(request).await })
    }

    #[napi(js_name = "apiKeys")]
    pub fn api_keys_native<'env>(&self, env: &'env Env) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.api_keys().await })
    }

    #[napi(js_name = "listPockets")]
    pub fn list_pockets_native<'env>(
        &self,
        env: &'env Env,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.list_pockets().await })
    }

    #[napi(js_name = "listPocketApiKeys", ts_args_type = "request: string")]
    pub fn list_pocket_api_keys_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.list_pocket_api_keys(request).await })
    }

    #[napi(js_name = "subPocketBalances", ts_args_type = "pocketUuid: string")]
    pub fn sub_pocket_balances_native<'env>(
        &self,
        env: &'env Env,
        pocket_uuid: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let pocket_uuid = native_json_text(pocket_uuid, "pocketUuid");
        spawn_native(
            env,
            async move { this.sub_pocket_balances(pocket_uuid).await },
        )
    }

    #[napi(js_name = "universalTransfer", ts_args_type = "request: string")]
    pub fn universal_transfer_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.universal_transfer(request).await })
    }

    #[napi(js_name = "universalTransfers", ts_args_type = "request: string")]
    pub fn universal_transfers_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.universal_transfers(request).await })
    }

    #[napi(js_name = "subPocketTransfer", ts_args_type = "request: string")]
    pub fn sub_pocket_transfer_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.sub_pocket_transfer(request).await })
    }

    #[napi(js_name = "subPocketTransfers", ts_args_type = "request: string")]
    pub fn sub_pocket_transfers_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.sub_pocket_transfers(request).await })
    }

    #[napi(js_name = "orderDetail", ts_args_type = "request: string")]
    pub fn order_detail_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.order_detail(request).await })
    }

    #[napi(js_name = "closedOrders", ts_args_type = "request: string")]
    pub fn closed_orders_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.closed_orders(request).await })
    }
}

impl Clone for NativeUpbit {
    fn clone(&self) -> Self {
        Self {
            adapter: Arc::clone(&self.adapter),
        }
    }
}

#[cfg_attr(all(not(test), not(target_arch = "wasm32")), napi)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct NativeBithumb {
    #[cfg_attr(
        test,
        allow(dead_code, reason = "네이티브 메서드는 테스트 빌드에서 제외됩니다")
    )]
    adapter: Arc<BithumbAdapter>,
}

#[cfg_attr(
    test,
    allow(dead_code, reason = "네이티브 메서드는 테스트 빌드에서 제외됩니다")
)]
impl NativeBithumb {
    fn create(options: maxt::Result<String>) -> maxt::Result<Self> {
        let options: BithumbOptions = parse_options(options)?;
        let mut adapter = BithumbAdapter::new();
        if let Some((access_key, secret_key)) = credential_pair(
            options.access_key,
            options.secret_key,
            "access_key",
            "secret_key",
        )? {
            adapter = adapter.with_credentials(access_key, secret_key);
        }
        Ok(Self {
            adapter: Arc::new(adapter),
        })
    }

    fn client(&self) -> NativeClient {
        NativeClient::from_boxed(Box::new((*self.adapter).clone()))
    }

    async fn market_warnings(&self) -> Value {
        outcome(self.adapter.market_warnings().await.and_then(|values| {
            values
                .into_iter()
                .map(|(market, warning)| Ok((market.try_into()?, warning)))
                .collect::<maxt::Result<Vec<(WireMarket, String)>>>()
        }))
    }

    async fn market_alerts(&self) -> Value {
        outcome(wire_pairs::<_, WireBithumbMarketAlert>(
            self.adapter.market_alerts().await,
        ))
    }

    async fn notices(&self, count: maxt::Result<String>) -> Value {
        match parse_text::<Option<u32>>(count, "count") {
            Ok(count) => outcome(wire_vec::<_, WireBithumbNotice>(
                self.adapter.notices(count).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn transfer_fees(&self, currency: maxt::Result<String>) -> Value {
        match parse_text::<String>(currency, "currency") {
            Ok(currency) => outcome(wire_vec::<_, WireBithumbAssetFee>(
                self.adapter.transfer_fees(&currency).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn api_keys(&self) -> Value {
        outcome(wire_vec::<_, WireBithumbApiKey>(
            self.adapter.api_keys().await,
        ))
    }

    async fn pending_orders(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbPendingOrdersRequest, WireBithumbPendingOrdersRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .pending_orders(&request)
                    .await
                    .and_then(TryInto::<WirePage<WireOrder>>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn closed_orders(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbClosedOrdersRequest, WireBithumbClosedOrdersRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .closed_orders(&request)
                    .await
                    .and_then(TryInto::<WirePage<WireBithumbClosedOrder>>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn batch_orders(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbBatchOrdersRequest, WireBithumbBatchOrdersRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .batch_orders(&request)
                    .await
                    .and_then(TryInto::<WireBithumbBatchOrdersResult>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn twap_orders(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbTwapOrdersRequest, WireBithumbTwapOrdersRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .twap_orders(&request)
                    .await
                    .and_then(TryInto::<WirePage<WireBithumbTwapOrder>>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn krw_withdrawals(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbKrwWithdrawalsRequest, WireBithumbKrwWithdrawalsRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_vec::<_, WireBithumbKrwWithdrawal>(
                self.adapter.krw_withdrawals(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn withdraw_krw(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbKrwTransferRequest, WireBithumbKrwTransferRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .withdraw_krw(&request)
                    .await
                    .and_then(TryInto::<WireBithumbKrwWithdrawal>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn krw_deposits(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbKrwDepositsRequest, WireBithumbKrwDepositsRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_vec::<_, WireBithumbKrwDeposit>(
                self.adapter.krw_deposits(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn deposit_krw(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbKrwTransferRequest, WireBithumbKrwTransferRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .deposit_krw(&request)
                    .await
                    .and_then(TryInto::<WireBithumbKrwDeposit>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn create_twap_order(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbTwapOrderRequest, WireBithumbTwapOrderRequest>(request, "request")
        {
            Ok(request) => outcome(self.adapter.create_twap_order(&request).await),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn cancel_twap_order(&self, algo_order_id: maxt::Result<String>) -> Value {
        match parse_text::<String>(algo_order_id, "algo_order_id") {
            Ok(algo_order_id) => outcome(self.adapter.cancel_twap_order(&algo_order_id).await),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn withdrawal_addresses(&self) -> Value {
        outcome(wire_vec::<_, WireBithumbWithdrawalAddress>(
            self.adapter.withdrawal_addresses().await,
        ))
    }

    async fn order_detail(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbOrderDetailRequest, WireBithumbOrderDetailRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .order_detail(&request)
                    .await
                    .and_then(TryInto::<WireBithumbOrderDetail>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn order_list(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BithumbOrderListRequest, WireBithumbOrderListRequest>(request, "request")
        {
            Ok(request) => outcome(wire_vec::<_, WireBithumbOrderListItem>(
                self.adapter.order_list(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
#[napi]
impl NativeBithumb {
    #[napi(js_name = "client")]
    pub fn client_native(&self) -> NativeClient {
        self.client()
    }

    #[napi(js_name = "marketWarnings")]
    pub async fn market_warnings_native(&self) -> Value {
        self.market_warnings().await
    }

    #[napi(js_name = "marketAlerts")]
    pub async fn market_alerts_native(&self) -> Value {
        self.market_alerts().await
    }

    #[napi(js_name = "notices", ts_args_type = "count: string")]
    pub fn notices_native<'env>(
        &self,
        env: &'env Env,
        count: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let count = native_json_text(count, "count");
        spawn_native(env, async move { this.notices(count).await })
    }

    #[napi(js_name = "transferFees", ts_args_type = "currency: string")]
    pub fn transfer_fees_native<'env>(
        &self,
        env: &'env Env,
        currency: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let currency = native_json_text(currency, "currency");
        spawn_native(env, async move { this.transfer_fees(currency).await })
    }

    #[napi(js_name = "apiKeys")]
    pub fn api_keys_native<'env>(&self, env: &'env Env) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.api_keys().await })
    }

    #[napi(js_name = "pendingOrders", ts_args_type = "request: string")]
    pub fn pending_orders_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.pending_orders(request).await })
    }

    #[napi(js_name = "closedOrders", ts_args_type = "request: string")]
    pub fn closed_orders_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.closed_orders(request).await })
    }

    #[napi(js_name = "batchOrders", ts_args_type = "request: string")]
    pub fn batch_orders_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.batch_orders(request).await })
    }

    #[napi(js_name = "twapOrders", ts_args_type = "request: string")]
    pub fn twap_orders_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.twap_orders(request).await })
    }

    #[napi(js_name = "krwWithdrawals", ts_args_type = "request: string")]
    pub fn krw_withdrawals_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.krw_withdrawals(request).await })
    }

    #[napi(js_name = "withdrawKrw", ts_args_type = "request: string")]
    pub fn withdraw_krw_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.withdraw_krw(request).await })
    }

    #[napi(js_name = "krwDeposits", ts_args_type = "request: string")]
    pub fn krw_deposits_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.krw_deposits(request).await })
    }

    #[napi(js_name = "depositKrw", ts_args_type = "request: string")]
    pub fn deposit_krw_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.deposit_krw(request).await })
    }

    #[napi(js_name = "createTwapOrder", ts_args_type = "request: string")]
    pub fn create_twap_order_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.create_twap_order(request).await })
    }

    #[napi(js_name = "cancelTwapOrder", ts_args_type = "algoOrderId: string")]
    pub fn cancel_twap_order_native<'env>(
        &self,
        env: &'env Env,
        algo_order_id: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let algo_order_id = native_json_text(algo_order_id, "algoOrderId");
        spawn_native(
            env,
            async move { this.cancel_twap_order(algo_order_id).await },
        )
    }

    #[napi(js_name = "withdrawalAddresses")]
    pub fn withdrawal_addresses_native<'env>(
        &self,
        env: &'env Env,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.withdrawal_addresses().await })
    }

    #[napi(js_name = "orderDetail", ts_args_type = "request: string")]
    pub fn order_detail_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.order_detail(request).await })
    }

    #[napi(js_name = "orderList", ts_args_type = "request: string")]
    pub fn order_list_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.order_list(request).await })
    }
}

impl Clone for NativeBithumb {
    fn clone(&self) -> Self {
        Self {
            adapter: Arc::clone(&self.adapter),
        }
    }
}

#[cfg_attr(all(not(test), not(target_arch = "wasm32")), napi)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct NativeBinance {
    adapter: Arc<BinanceAdapter>,
    listen_keys: Arc<Mutex<HashMap<String, BinanceListenKey>>>,
    next_listen_key_id: Arc<AtomicU64>,
}

#[cfg_attr(
    test,
    allow(dead_code, reason = "네이티브 메서드는 테스트 빌드에서 제외됩니다")
)]
impl NativeBinance {
    fn create(options: maxt::Result<String>) -> maxt::Result<Self> {
        let options: BinanceOptions = parse_options(options)?;
        let mut adapter = match options.venue.as_str() {
            "spot" => BinanceAdapter::spot(),
            "usd_m" => BinanceAdapter::usd_m_futures(),
            value => return Err(invalid_enum("options.venue", value)),
        };
        if let Some((api_key, secret_key)) =
            credential_pair(options.api_key, options.secret_key, "api_key", "secret_key")?
        {
            adapter = adapter.with_credentials(api_key, secret_key);
        }
        Ok(Self {
            adapter: Arc::new(adapter),
            listen_keys: Arc::new(Mutex::new(HashMap::new())),
            next_listen_key_id: Arc::new(AtomicU64::new(1)),
        })
    }

    fn client(&self) -> NativeClient {
        NativeClient::from_boxed(Box::new((*self.adapter).clone()))
    }

    fn venue(&self) -> &'static str {
        match self.adapter.venue() {
            BinanceMarket::Spot => "spot",
            BinanceMarket::UsdMFutures => "usd_m",
            _ => "unknown",
        }
    }

    async fn spot_symbol_filters(&self, market: maxt::Result<String>) -> Value {
        match parse_wire::<Market, WireMarket>(market, "market") {
            Ok(market) => outcome(
                self.adapter
                    .spot_symbol_filters(&market)
                    .await
                    .and_then(TryInto::<WireBinanceSymbolFilters>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn spot_average_price(&self, market: maxt::Result<String>) -> Value {
        match parse_wire::<Market, WireMarket>(market, "market") {
            Ok(market) => outcome(
                self.adapter
                    .spot_average_price(&market)
                    .await
                    .and_then(TryInto::<WireBinanceSpotAveragePrice>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn spot_order(
        &self,
        market: maxt::Result<String>,
        order_id: maxt::Result<String>,
    ) -> Value {
        let market = parse_wire::<Market, WireMarket>(market, "market");
        let order_id = parse_text::<String>(order_id, "order_id");
        match (market, order_id) {
            (Ok(market), Ok(order_id)) => outcome(
                self.adapter
                    .spot_order(&market, &order_id)
                    .await
                    .and_then(TryInto::<WireBinanceSpotOrderDetail>::try_into),
            ),
            (Err(error), _) | (_, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    async fn mark_price(&self, market: maxt::Result<String>) -> Value {
        match parse_wire::<Market, WireMarket>(market, "market") {
            Ok(market) => outcome(
                self.adapter
                    .mark_price(&market)
                    .await
                    .and_then(TryInto::<WireBinanceMarkPrice>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn mark_prices(&self) -> Value {
        outcome(wire_vec::<_, WireBinanceMarkPrice>(
            self.adapter.mark_prices().await,
        ))
    }

    async fn open_interest(&self, market: maxt::Result<String>) -> Value {
        match parse_wire::<Market, WireMarket>(market, "market") {
            Ok(market) => outcome(
                self.adapter
                    .open_interest(&market)
                    .await
                    .and_then(TryInto::<WireBinanceOpenInterest>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn aggregate_trades(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BinanceAggregateTradesRequest, WireBinanceAggregateTradesRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(wire_vec::<_, WireBinanceAggregateTrade>(
                self.adapter.aggregate_trades(&request).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn account_trades(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<maxt::HistoryRequest, crate::convert::WireHistoryRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .account_trades(&request)
                    .await
                    .and_then(TryInto::<WirePage<WireBinanceAccountTrade>>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn c2c_trade_history(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BinanceC2cTradeHistoryRequest, WireBinanceC2cTradeHistoryRequest>(
            request, "request",
        ) {
            Ok(request) => outcome(
                self.adapter
                    .c2c_trade_history(&request)
                    .await
                    .and_then(TryInto::<WireBinanceC2cTradeHistoryPage>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn test_order(&self, request: maxt::Result<String>) -> Value {
        match parse_wire::<BinanceTestOrderRequest, WireBinanceTestOrderRequest>(request, "request")
        {
            Ok(request) => outcome(
                self.adapter
                    .test_order(&request)
                    .await
                    .and_then(TryInto::<WireBinanceTestOrder>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn cancel_all_open_orders(&self, market: maxt::Result<String>) -> Value {
        match parse_wire::<Market, WireMarket>(market, "market") {
            Ok(market) => outcome(
                self.adapter
                    .cancel_all_open_orders(&market)
                    .await
                    .map(|()| Value::Null),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn usd_m_create_listen_key(&self) -> Value {
        let result = match self.adapter.usd_m_create_listen_key().await {
            Ok(key) => {
                let id = match self.next_listen_key_id.fetch_update(
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                    |id| id.checked_add(1),
                ) {
                    Ok(id) => id.to_string(),
                    Err(_) => {
                        return outcome::<Value>(Err(Error::adapter(
                            "native listen-key ID space is exhausted",
                        )));
                    }
                };
                let value = key.as_str().to_owned();
                self.listen_keys.lock().await.insert(id.clone(), key);
                Ok(WireBinanceListenKey { id, value })
            }
            Err(error) => Err(error),
        };
        outcome(result)
    }

    async fn usd_m_keepalive_listen_key(&self) -> Value {
        let result = self.adapter.usd_m_keepalive_listen_key().await;
        outcome(result.map(|()| Value::Null))
    }

    async fn usd_m_close_listen_key(&self) -> Value {
        let result = self.adapter.usd_m_close_listen_key().await;
        if result.is_ok() {
            self.listen_keys.lock().await.clear();
        }
        outcome(result.map(|()| Value::Null))
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
#[napi]
impl NativeBinance {
    #[napi(js_name = "client")]
    pub fn client_native(&self) -> NativeClient {
        self.client()
    }

    #[napi(js_name = "venue")]
    pub fn venue_native(&self) -> String {
        self.venue().to_owned()
    }

    #[napi(js_name = "spotSymbolFilters", ts_args_type = "market: string")]
    pub fn spot_symbol_filters_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let market = native_json_text(market, "market");
        spawn_native(env, async move { this.spot_symbol_filters(market).await })
    }

    #[napi(js_name = "spotAveragePrice", ts_args_type = "market: string")]
    pub fn spot_average_price_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let market = native_json_text(market, "market");
        spawn_native(env, async move { this.spot_average_price(market).await })
    }

    #[napi(
        js_name = "spotOrder",
        ts_args_type = "market: string, orderId: string"
    )]
    pub fn spot_order_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
        order_id: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let market = native_json_text(market, "market");
        let order_id = native_json_text(order_id, "order_id");
        spawn_native(env, async move { this.spot_order(market, order_id).await })
    }

    #[napi(js_name = "markPrice", ts_args_type = "market: string")]
    pub fn mark_price_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let market = native_json_text(market, "market");
        spawn_native(env, async move { this.mark_price(market).await })
    }

    #[napi(js_name = "markPrices")]
    pub fn mark_prices_native<'env>(
        &self,
        env: &'env Env,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.mark_prices().await })
    }

    #[napi(js_name = "openInterest", ts_args_type = "market: string")]
    pub fn open_interest_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let market = native_json_text(market, "market");
        spawn_native(env, async move { this.open_interest(market).await })
    }

    #[napi(js_name = "aggregateTrades", ts_args_type = "request: string")]
    pub fn aggregate_trades_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.aggregate_trades(request).await })
    }

    #[napi(js_name = "accountTrades", ts_args_type = "request: string")]
    pub fn account_trades_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.account_trades(request).await })
    }

    #[napi(js_name = "c2cTradeHistory", ts_args_type = "request: string")]
    pub fn c2c_trade_history_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.c2c_trade_history(request).await })
    }

    #[napi(js_name = "testOrder", ts_args_type = "request: string")]
    pub fn test_order_native<'env>(
        &self,
        env: &'env Env,
        request: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let request = native_json_text(request, "request");
        spawn_native(env, async move { this.test_order(request).await })
    }

    #[napi(js_name = "cancelAllOpenOrders", ts_args_type = "market: string")]
    pub fn cancel_all_open_orders_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let market = native_json_text(market, "market");
        spawn_native(
            env,
            async move { this.cancel_all_open_orders(market).await },
        )
    }

    #[napi(js_name = "usdMCreateListenKey")]
    pub async fn usd_m_create_listen_key_native(&self) -> Value {
        self.usd_m_create_listen_key().await
    }

    #[napi(js_name = "usdMKeepaliveListenKey")]
    pub async fn usd_m_keepalive_listen_key_native(&self) -> Value {
        self.usd_m_keepalive_listen_key().await
    }

    #[napi(js_name = "usdMCloseListenKey")]
    pub async fn usd_m_close_listen_key_native(&self) -> Value {
        self.usd_m_close_listen_key().await
    }
}

impl Clone for NativeBinance {
    fn clone(&self) -> Self {
        Self {
            adapter: Arc::clone(&self.adapter),
            listen_keys: Arc::clone(&self.listen_keys),
            next_listen_key_id: Arc::clone(&self.next_listen_key_id),
        }
    }
}

#[cfg_attr(all(not(test), not(target_arch = "wasm32")), napi)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct NativeHyperliquid {
    adapter: Arc<HyperliquidAdapter>,
}

#[cfg_attr(
    test,
    allow(dead_code, reason = "네이티브 메서드는 테스트 빌드에서 제외됩니다")
)]
impl NativeHyperliquid {
    fn create(options: maxt::Result<String>) -> maxt::Result<Self> {
        let options: HyperliquidOptions = parse_options(options)?;
        let mut adapter = if options.testnet {
            HyperliquidAdapter::testnet()
        } else {
            HyperliquidAdapter::new()
        };
        if let Some(address) = options.address {
            adapter = adapter.with_query_address(address);
        }
        if let Some(private_key) = options.private_key {
            adapter = adapter.with_signer(private_key);
        }
        Ok(Self {
            adapter: Arc::new(adapter),
        })
    }

    fn client(&self) -> NativeClient {
        NativeClient::from_boxed(Box::new((*self.adapter).clone()))
    }

    fn is_testnet(&self) -> bool {
        self.adapter.is_testnet()
    }

    async fn non_funding_ledger(
        &self,
        from: maxt::Result<String>,
        to: maxt::Result<String>,
        cursor: maxt::Result<String>,
        limit: maxt::Result<String>,
    ) -> Value {
        let from = parse_text::<Option<String>>(from, "from").and_then(|value| {
            value
                .as_deref()
                .map(|value| timestamp_from_wire(value, "from"))
                .transpose()
        });
        let to = parse_text::<Option<String>>(to, "to").and_then(|value| {
            value
                .as_deref()
                .map(|value| timestamp_from_wire(value, "to"))
                .transpose()
        });
        let cursor =
            parse_text::<Option<String>>(cursor, "cursor").map(|value| value.map(Cursor::new));
        let limit = parse_text::<Option<u32>>(limit, "limit");
        match (from, to, cursor, limit) {
            (Ok(from), Ok(to), Ok(cursor), Ok(limit)) => outcome(
                self.adapter
                    .non_funding_ledger(from, to, cursor.as_ref(), limit)
                    .await
                    .and_then(TryInto::<WirePage<WireHyperliquidLedgerEntry>>::try_into),
            ),
            (Err(error), _, _, _)
            | (_, Err(error), _, _)
            | (_, _, Err(error), _)
            | (_, _, _, Err(error)) => outcome::<Value>(Err(error)),
        }
    }

    async fn asset_context(&self, market: maxt::Result<String>) -> Value {
        match parse_wire::<Market, WireMarket>(market, "market") {
            Ok(market) => outcome(
                self.adapter
                    .asset_context(&market)
                    .await
                    .and_then(TryInto::<WireHyperliquidAssetContext>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn all_mids(&self) -> Value {
        outcome(wire_vec::<_, WireHyperliquidMidPrice>(
            self.adapter.all_mids().await,
        ))
    }

    async fn user_rate_limit(&self) -> Value {
        outcome(
            self.adapter
                .user_rate_limit()
                .await
                .and_then(TryInto::<WireHyperliquidUserRateLimit>::try_into),
        )
    }

    async fn user_role(&self) -> Value {
        outcome(
            self.adapter
                .user_role()
                .await
                .and_then(TryInto::<WireHyperliquidUserRole>::try_into),
        )
    }

    async fn referral(&self) -> Value {
        outcome(
            self.adapter
                .referral()
                .await
                .and_then(TryInto::<WireHyperliquidReferral>::try_into),
        )
    }

    async fn user_fees(&self) -> Value {
        outcome(
            self.adapter
                .user_fees()
                .await
                .and_then(TryInto::<WireHyperliquidUserFees>::try_into),
        )
    }

    async fn portfolio(&self) -> Value {
        outcome(wire_vec::<_, WireHyperliquidPortfolioPeriod>(
            self.adapter.portfolio().await,
        ))
    }

    async fn sub_accounts(&self) -> Value {
        outcome(wire_vec::<_, WireHyperliquidSubAccount>(
            self.adapter.sub_accounts().await,
        ))
    }

    async fn user_vault_equities(&self) -> Value {
        outcome(wire_vec::<_, WireHyperliquidVaultEquity>(
            self.adapter.user_vault_equities().await,
        ))
    }

    async fn user_fills(&self, aggregate_by_time: maxt::Result<String>) -> Value {
        match parse_text::<bool>(aggregate_by_time, "aggregate_by_time") {
            Ok(aggregate_by_time) => outcome(wire_vec::<_, WireHyperliquidUserFill>(
                self.adapter.user_fills(aggregate_by_time).await,
            )),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn user_fills_by_time(
        &self,
        from: maxt::Result<String>,
        to: maxt::Result<String>,
        aggregate_by_time: maxt::Result<String>,
    ) -> Value {
        let from = parse_text::<String>(from, "from")
            .and_then(|value| timestamp_from_wire(&value, "from"));
        let to = parse_text::<Option<String>>(to, "to").and_then(|value| {
            value
                .as_deref()
                .map(|value| timestamp_from_wire(value, "to"))
                .transpose()
        });
        let aggregate_by_time = parse_text::<bool>(aggregate_by_time, "aggregate_by_time");
        match (from, to, aggregate_by_time) {
            (Ok(from), Ok(to), Ok(aggregate_by_time)) => {
                outcome(wire_vec::<_, WireHyperliquidUserFill>(
                    self.adapter
                        .user_fills_by_time(from, to, aggregate_by_time)
                        .await,
                ))
            }
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
                outcome::<Value>(Err(error))
            }
        }
    }

    async fn basic_open_orders(&self) -> Value {
        outcome(wire_vec::<_, WireHyperliquidOpenOrder>(
            self.adapter.basic_open_orders().await,
        ))
    }

    async fn order_status(&self, reference: maxt::Result<String>) -> Value {
        match parse_wire::<HyperliquidOrderReference, WireHyperliquidOrderReference>(
            reference,
            "reference",
        ) {
            Ok(reference) => outcome(
                self.adapter
                    .order_status(reference)
                    .await
                    .and_then(TryInto::<WireHyperliquidOrderStatusResponse>::try_into),
            ),
            Err(error) => outcome::<Value>(Err(error)),
        }
    }

    async fn historical_orders(&self) -> Value {
        outcome(wire_vec::<_, WireHyperliquidOrderInfo>(
            self.adapter.historical_orders().await,
        ))
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
#[napi]
impl NativeHyperliquid {
    #[napi(js_name = "client")]
    pub fn client_native(&self) -> NativeClient {
        self.client()
    }

    #[napi(js_name = "isTestnet")]
    pub fn is_testnet_native(&self) -> bool {
        self.is_testnet()
    }

    #[napi(
        js_name = "nonFundingLedger",
        ts_args_type = "from: string, to: string, cursor: string, limit: string"
    )]
    pub fn non_funding_ledger_native<'env>(
        &self,
        env: &'env Env,
        from: NativeJsonText<'env>,
        to: NativeJsonText<'env>,
        cursor: NativeJsonText<'env>,
        limit: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let from = native_json_text(from, "from");
        let to = native_json_text(to, "to");
        let cursor = native_json_text(cursor, "cursor");
        let limit = native_json_text(limit, "limit");
        spawn_native(env, async move {
            this.non_funding_ledger(from, to, cursor, limit).await
        })
    }

    #[napi(js_name = "assetContext", ts_args_type = "market: string")]
    pub fn asset_context_native<'env>(
        &self,
        env: &'env Env,
        market: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let market = native_json_text(market, "market");
        spawn_native(env, async move { this.asset_context(market).await })
    }

    #[napi(js_name = "allMids")]
    pub fn all_mids_native<'env>(&self, env: &'env Env) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.all_mids().await })
    }

    #[napi(js_name = "userRateLimit")]
    pub fn user_rate_limit_native<'env>(
        &self,
        env: &'env Env,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.user_rate_limit().await })
    }

    #[napi(js_name = "userRole")]
    pub fn user_role_native<'env>(&self, env: &'env Env) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.user_role().await })
    }

    #[napi(js_name = "referral")]
    pub fn referral_native<'env>(&self, env: &'env Env) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.referral().await })
    }

    #[napi(js_name = "userFees")]
    pub fn user_fees_native<'env>(&self, env: &'env Env) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.user_fees().await })
    }

    #[napi(js_name = "portfolio")]
    pub fn portfolio_native<'env>(&self, env: &'env Env) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.portfolio().await })
    }

    #[napi(js_name = "subAccounts")]
    pub fn sub_accounts_native<'env>(
        &self,
        env: &'env Env,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.sub_accounts().await })
    }

    #[napi(js_name = "userVaultEquities")]
    pub fn user_vault_equities_native<'env>(
        &self,
        env: &'env Env,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.user_vault_equities().await })
    }

    #[napi(js_name = "userFills", ts_args_type = "aggregateByTime: string")]
    pub fn user_fills_native<'env>(
        &self,
        env: &'env Env,
        aggregate_by_time: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let aggregate_by_time = native_json_text(aggregate_by_time, "aggregateByTime");
        spawn_native(env, async move { this.user_fills(aggregate_by_time).await })
    }

    #[napi(
        js_name = "userFillsByTime",
        ts_args_type = "from: string, to: string, aggregateByTime: string"
    )]
    pub fn user_fills_by_time_native<'env>(
        &self,
        env: &'env Env,
        from: NativeJsonText<'env>,
        to: NativeJsonText<'env>,
        aggregate_by_time: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let from = native_json_text(from, "from");
        let to = native_json_text(to, "to");
        let aggregate_by_time = native_json_text(aggregate_by_time, "aggregateByTime");
        spawn_native(env, async move {
            this.user_fills_by_time(from, to, aggregate_by_time).await
        })
    }

    #[napi(js_name = "basicOpenOrders")]
    pub fn basic_open_orders_native<'env>(
        &self,
        env: &'env Env,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.basic_open_orders().await })
    }

    #[napi(js_name = "orderStatus", ts_args_type = "reference: string")]
    pub fn order_status_native<'env>(
        &self,
        env: &'env Env,
        reference: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let reference = native_json_text(reference, "reference");
        spawn_native(env, async move { this.order_status(reference).await })
    }

    #[napi(js_name = "historicalOrders")]
    pub fn historical_orders_native<'env>(
        &self,
        env: &'env Env,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        spawn_native(env, async move { this.historical_orders().await })
    }
}

impl Clone for NativeHyperliquid {
    fn clone(&self) -> Self {
        Self {
            adapter: Arc::clone(&self.adapter),
        }
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

fn invalid_enum(field: &str, value: &str) -> Error {
    Error::InvalidRequest {
        field: field.to_owned(),
        detail: format!("unknown value `{value}`"),
    }
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
fn factory_error(error: Error) -> napi::Error {
    let wire = outcome::<Value>(Err(error));
    napi::Error::from_reason(wire.to_string())
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
#[napi(js_name = "createUpbit", ts_args_type = "options: string")]
pub fn create_upbit(options: NativeJsonText<'_>) -> napi::Result<NativeUpbit> {
    NativeUpbit::create(native_json_text(options, "options")).map_err(factory_error)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
#[napi(js_name = "createBithumb", ts_args_type = "options: string")]
pub fn create_bithumb(options: NativeJsonText<'_>) -> napi::Result<NativeBithumb> {
    NativeBithumb::create(native_json_text(options, "options")).map_err(factory_error)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
#[napi(js_name = "createBinance", ts_args_type = "options: string")]
pub fn create_binance(options: NativeJsonText<'_>) -> napi::Result<NativeBinance> {
    NativeBinance::create(native_json_text(options, "options")).map_err(factory_error)
}

#[cfg(all(not(test), not(target_arch = "wasm32")))]
#[napi(js_name = "createHyperliquid", ts_args_type = "options: string")]
pub fn create_hyperliquid(options: NativeJsonText<'_>) -> napi::Result<NativeHyperliquid> {
    NativeHyperliquid::create(native_json_text(options, "options")).map_err(factory_error)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl NativeUpbit {
    #[wasm_bindgen(js_name = "client")]
    pub fn client_wasm(&self) -> NativeClient {
        self.client()
    }

    #[wasm_bindgen(js_name = "region")]
    pub fn region_wasm(&self) -> String {
        self.region().to_owned()
    }

    #[wasm_bindgen(js_name = "orderBooks")]
    pub async fn order_books_wasm(&self, markets: String, depth: String) -> JsValue {
        crate::web::value(self.order_books(Ok(markets), Ok(depth)).await)
    }

    #[wasm_bindgen(js_name = "orderBooksAtLevel")]
    pub async fn order_books_at_level_wasm(
        &self,
        markets: String,
        level: String,
        depth: String,
    ) -> JsValue {
        crate::web::value(
            self.order_books_at_level(Ok(markets), Ok(level), Ok(depth))
                .await,
        )
    }

    #[wasm_bindgen(js_name = "tickers")]
    pub async fn tickers_wasm(&self, markets: String) -> JsValue {
        crate::web::value(self.tickers(Ok(markets)).await)
    }

    #[wasm_bindgen(js_name = "tickersByQuote")]
    pub async fn tickers_by_quote_wasm(&self, quote_currencies: String) -> JsValue {
        crate::web::value(self.tickers_by_quote(Ok(quote_currencies)).await)
    }

    #[wasm_bindgen(js_name = "yearCandles")]
    pub async fn year_candles_wasm(&self, market: String, to: String, count: String) -> JsValue {
        crate::web::value(self.year_candles(Ok(market), Ok(to), Ok(count)).await)
    }

    #[wasm_bindgen(js_name = "orderbookInstruments")]
    pub async fn orderbook_instruments_wasm(&self, markets: String) -> JsValue {
        crate::web::value(self.orderbook_instruments(Ok(markets)).await)
    }

    #[wasm_bindgen(js_name = "marketEvents")]
    pub async fn market_events_wasm(&self) -> JsValue {
        crate::web::value(self.market_events().await)
    }

    #[wasm_bindgen(js_name = "listSubscriptions")]
    pub async fn list_subscriptions_wasm(&self, subscription: String) -> JsValue {
        crate::web::value(self.list_subscriptions(Ok(subscription)).await)
    }

    #[wasm_bindgen(js_name = "testOrder")]
    pub async fn test_order_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.test_order(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "depositInfo")]
    pub async fn deposit_info_wasm(&self, asset: String, network: String) -> JsValue {
        crate::web::value(self.deposit_info(Ok(asset), Ok(network)).await)
    }

    #[wasm_bindgen(js_name = "travelRuleVasps")]
    pub async fn travel_rule_vasps_wasm(&self) -> JsValue {
        crate::web::value(self.travel_rule_vasps().await)
    }

    #[wasm_bindgen(js_name = "verifyTravelRuleByUuid")]
    pub async fn verify_travel_rule_by_uuid_wasm(
        &self,
        deposit_uuid: String,
        vasp_uuid: String,
    ) -> JsValue {
        crate::web::value(
            self.verify_travel_rule_by_uuid(Ok(deposit_uuid), Ok(vasp_uuid))
                .await,
        )
    }

    #[wasm_bindgen(js_name = "verifyTravelRuleByTxid")]
    pub async fn verify_travel_rule_by_txid_wasm(
        &self,
        txid: String,
        vasp_uuid: String,
        currency: String,
        net_type: String,
    ) -> JsValue {
        crate::web::value(
            self.verify_travel_rule_by_txid(Ok(txid), Ok(vasp_uuid), Ok(currency), Ok(net_type))
                .await,
        )
    }

    #[wasm_bindgen(js_name = "batchCancelOpenOrders")]
    pub async fn batch_cancel_open_orders_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.batch_cancel_open_orders(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "cancelAndNewOrder")]
    pub async fn cancel_and_new_order_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.cancel_and_new_order(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "depositKrw")]
    pub async fn deposit_krw_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.deposit_krw(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "withdrawKrw")]
    pub async fn withdraw_krw_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.withdraw_krw(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "apiKeys")]
    pub async fn api_keys_wasm(&self) -> JsValue {
        crate::web::value(self.api_keys().await)
    }

    #[wasm_bindgen(js_name = "listPockets")]
    pub async fn list_pockets_wasm(&self) -> JsValue {
        crate::web::value(self.list_pockets().await)
    }

    #[wasm_bindgen(js_name = "listPocketApiKeys")]
    pub async fn list_pocket_api_keys_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.list_pocket_api_keys(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "subPocketBalances")]
    pub async fn sub_pocket_balances_wasm(&self, pocket_uuid: String) -> JsValue {
        crate::web::value(self.sub_pocket_balances(Ok(pocket_uuid)).await)
    }

    #[wasm_bindgen(js_name = "universalTransfer")]
    pub async fn universal_transfer_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.universal_transfer(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "universalTransfers")]
    pub async fn universal_transfers_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.universal_transfers(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "subPocketTransfer")]
    pub async fn sub_pocket_transfer_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.sub_pocket_transfer(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "subPocketTransfers")]
    pub async fn sub_pocket_transfers_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.sub_pocket_transfers(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "orderDetail")]
    pub async fn order_detail_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.order_detail(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "closedOrders")]
    pub async fn closed_orders_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.closed_orders(Ok(request)).await)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl NativeBithumb {
    #[wasm_bindgen(js_name = "client")]
    pub fn client_wasm(&self) -> NativeClient {
        self.client()
    }

    #[wasm_bindgen(js_name = "marketWarnings")]
    pub async fn market_warnings_wasm(&self) -> JsValue {
        crate::web::value(self.market_warnings().await)
    }

    #[wasm_bindgen(js_name = "marketAlerts")]
    pub async fn market_alerts_wasm(&self) -> JsValue {
        crate::web::value(self.market_alerts().await)
    }

    #[wasm_bindgen(js_name = "notices")]
    pub async fn notices_wasm(&self, count: String) -> JsValue {
        crate::web::value(self.notices(Ok(count)).await)
    }

    #[wasm_bindgen(js_name = "transferFees")]
    pub async fn transfer_fees_wasm(&self, currency: String) -> JsValue {
        crate::web::value(self.transfer_fees(Ok(currency)).await)
    }

    #[wasm_bindgen(js_name = "apiKeys")]
    pub async fn api_keys_wasm(&self) -> JsValue {
        crate::web::value(self.api_keys().await)
    }

    #[wasm_bindgen(js_name = "pendingOrders")]
    pub async fn pending_orders_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.pending_orders(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "closedOrders")]
    pub async fn closed_orders_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.closed_orders(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "batchOrders")]
    pub async fn batch_orders_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.batch_orders(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "twapOrders")]
    pub async fn twap_orders_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.twap_orders(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "krwWithdrawals")]
    pub async fn krw_withdrawals_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.krw_withdrawals(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "withdrawKrw")]
    pub async fn withdraw_krw_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.withdraw_krw(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "krwDeposits")]
    pub async fn krw_deposits_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.krw_deposits(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "depositKrw")]
    pub async fn deposit_krw_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.deposit_krw(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "createTwapOrder")]
    pub async fn create_twap_order_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.create_twap_order(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "cancelTwapOrder")]
    pub async fn cancel_twap_order_wasm(&self, algo_order_id: String) -> JsValue {
        crate::web::value(self.cancel_twap_order(Ok(algo_order_id)).await)
    }

    #[wasm_bindgen(js_name = "withdrawalAddresses")]
    pub async fn withdrawal_addresses_wasm(&self) -> JsValue {
        crate::web::value(self.withdrawal_addresses().await)
    }

    #[wasm_bindgen(js_name = "orderDetail")]
    pub async fn order_detail_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.order_detail(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "orderList")]
    pub async fn order_list_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.order_list(Ok(request)).await)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl NativeBinance {
    #[wasm_bindgen(js_name = "client")]
    pub fn client_wasm(&self) -> NativeClient {
        self.client()
    }

    #[wasm_bindgen(js_name = "venue")]
    pub fn venue_wasm(&self) -> String {
        self.venue().to_owned()
    }

    #[wasm_bindgen(js_name = "spotSymbolFilters")]
    pub async fn spot_symbol_filters_wasm(&self, market: String) -> JsValue {
        crate::web::value(self.spot_symbol_filters(Ok(market)).await)
    }

    #[wasm_bindgen(js_name = "spotAveragePrice")]
    pub async fn spot_average_price_wasm(&self, market: String) -> JsValue {
        crate::web::value(self.spot_average_price(Ok(market)).await)
    }

    #[wasm_bindgen(js_name = "spotOrder")]
    pub async fn spot_order_wasm(&self, market: String, order_id: String) -> JsValue {
        crate::web::value(self.spot_order(Ok(market), Ok(order_id)).await)
    }

    #[wasm_bindgen(js_name = "markPrice")]
    pub async fn mark_price_wasm(&self, market: String) -> JsValue {
        crate::web::value(self.mark_price(Ok(market)).await)
    }

    #[wasm_bindgen(js_name = "markPrices")]
    pub async fn mark_prices_wasm(&self) -> JsValue {
        crate::web::value(self.mark_prices().await)
    }

    #[wasm_bindgen(js_name = "openInterest")]
    pub async fn open_interest_wasm(&self, market: String) -> JsValue {
        crate::web::value(self.open_interest(Ok(market)).await)
    }

    #[wasm_bindgen(js_name = "aggregateTrades")]
    pub async fn aggregate_trades_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.aggregate_trades(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "accountTrades")]
    pub async fn account_trades_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.account_trades(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "c2cTradeHistory")]
    pub async fn c2c_trade_history_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.c2c_trade_history(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "testOrder")]
    pub async fn test_order_wasm(&self, request: String) -> JsValue {
        crate::web::value(self.test_order(Ok(request)).await)
    }

    #[wasm_bindgen(js_name = "cancelAllOpenOrders")]
    pub async fn cancel_all_open_orders_wasm(&self, market: String) -> JsValue {
        crate::web::value(self.cancel_all_open_orders(Ok(market)).await)
    }

    #[wasm_bindgen(js_name = "usdMCreateListenKey")]
    pub async fn usd_m_create_listen_key_wasm(&self) -> JsValue {
        crate::web::value(self.usd_m_create_listen_key().await)
    }

    #[wasm_bindgen(js_name = "usdMKeepaliveListenKey")]
    pub async fn usd_m_keepalive_listen_key_wasm(&self) -> JsValue {
        crate::web::value(self.usd_m_keepalive_listen_key().await)
    }

    #[wasm_bindgen(js_name = "usdMCloseListenKey")]
    pub async fn usd_m_close_listen_key_wasm(&self) -> JsValue {
        crate::web::value(self.usd_m_close_listen_key().await)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl NativeHyperliquid {
    #[wasm_bindgen(js_name = "client")]
    pub fn client_wasm(&self) -> NativeClient {
        self.client()
    }

    #[wasm_bindgen(js_name = "isTestnet")]
    pub fn is_testnet_wasm(&self) -> bool {
        self.is_testnet()
    }

    #[wasm_bindgen(js_name = "nonFundingLedger")]
    pub async fn non_funding_ledger_wasm(
        &self,
        from: String,
        to: String,
        cursor: String,
        limit: String,
    ) -> JsValue {
        crate::web::value(
            self.non_funding_ledger(Ok(from), Ok(to), Ok(cursor), Ok(limit))
                .await,
        )
    }

    #[wasm_bindgen(js_name = "assetContext")]
    pub async fn asset_context_wasm(&self, market: String) -> JsValue {
        crate::web::value(self.asset_context(Ok(market)).await)
    }

    #[wasm_bindgen(js_name = "allMids")]
    pub async fn all_mids_wasm(&self) -> JsValue {
        crate::web::value(self.all_mids().await)
    }

    #[wasm_bindgen(js_name = "userRateLimit")]
    pub async fn user_rate_limit_wasm(&self) -> JsValue {
        crate::web::value(self.user_rate_limit().await)
    }

    #[wasm_bindgen(js_name = "userRole")]
    pub async fn user_role_wasm(&self) -> JsValue {
        crate::web::value(self.user_role().await)
    }

    #[wasm_bindgen(js_name = "referral")]
    pub async fn referral_wasm(&self) -> JsValue {
        crate::web::value(self.referral().await)
    }

    #[wasm_bindgen(js_name = "userFees")]
    pub async fn user_fees_wasm(&self) -> JsValue {
        crate::web::value(self.user_fees().await)
    }

    #[wasm_bindgen(js_name = "portfolio")]
    pub async fn portfolio_wasm(&self) -> JsValue {
        crate::web::value(self.portfolio().await)
    }

    #[wasm_bindgen(js_name = "subAccounts")]
    pub async fn sub_accounts_wasm(&self) -> JsValue {
        crate::web::value(self.sub_accounts().await)
    }

    #[wasm_bindgen(js_name = "userVaultEquities")]
    pub async fn user_vault_equities_wasm(&self) -> JsValue {
        crate::web::value(self.user_vault_equities().await)
    }

    #[wasm_bindgen(js_name = "userFills")]
    pub async fn user_fills_wasm(&self, aggregate_by_time: String) -> JsValue {
        crate::web::value(self.user_fills(Ok(aggregate_by_time)).await)
    }

    #[wasm_bindgen(js_name = "userFillsByTime")]
    pub async fn user_fills_by_time_wasm(
        &self,
        from: String,
        to: String,
        aggregate_by_time: String,
    ) -> JsValue {
        crate::web::value(
            self.user_fills_by_time(Ok(from), Ok(to), Ok(aggregate_by_time))
                .await,
        )
    }

    #[wasm_bindgen(js_name = "basicOpenOrders")]
    pub async fn basic_open_orders_wasm(&self) -> JsValue {
        crate::web::value(self.basic_open_orders().await)
    }

    #[wasm_bindgen(js_name = "orderStatus")]
    pub async fn order_status_wasm(&self, reference: String) -> JsValue {
        crate::web::value(self.order_status(Ok(reference)).await)
    }

    #[wasm_bindgen(js_name = "historicalOrders")]
    pub async fn historical_orders_wasm(&self) -> JsValue {
        crate::web::value(self.historical_orders().await)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "createUpbit")]
pub fn create_upbit_wasm(options: String) -> Result<NativeUpbit, JsValue> {
    NativeUpbit::create(Ok(options)).map_err(crate::web::factory_error)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "createBithumb")]
pub fn create_bithumb_wasm(options: String) -> Result<NativeBithumb, JsValue> {
    NativeBithumb::create(Ok(options)).map_err(crate::web::factory_error)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "createBinance")]
pub fn create_binance_wasm(options: String) -> Result<NativeBinance, JsValue> {
    NativeBinance::create(Ok(options)).map_err(crate::web::factory_error)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = "createHyperliquid")]
pub fn create_hyperliquid_wasm(options: String) -> Result<NativeHyperliquid, JsValue> {
    NativeHyperliquid::create(Ok(options)).map_err(crate::web::factory_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_value(value: Value) -> Value {
        assert_eq!(value["ok"], true);
        value["value"].clone()
    }

    #[test]
    fn factories_preserve_provider_configuration() {
        let upbit = NativeUpbit::create(Ok(
            r#"{"region":"singapore","access_key":null,"secret_key":null}"#.to_owned(),
        ))
        .unwrap();
        assert_eq!(upbit.region(), "singapore");
        assert_eq!(upbit.adapter.exchange().id(), "upbit");

        let binance = NativeBinance::create(Ok(
            r#"{"venue":"usd_m","api_key":"key","secret_key":"secret"}"#.to_owned(),
        ))
        .unwrap();
        assert_eq!(binance.venue(), "usd_m");
        assert!(binance.adapter.supports(maxt::Feature::Trading));

        let hyperliquid = NativeHyperliquid::create(Ok(
            r#"{"testnet":true,"address":null,"private_key":null}"#.to_owned(),
        ))
        .unwrap();
        assert!(hyperliquid.is_testnet());

        let address_only = NativeHyperliquid::create(Ok(
            r#"{"testnet":false,"address":"0x14791697260e4c9a71f18484c9f997b308e59325","private_key":null}"#.to_owned(),
        ))
        .unwrap();
        assert!(address_only.adapter.supports(maxt::Feature::Balances));
        assert!(!address_only.adapter.supports(maxt::Feature::Trading));

        let signer_only = NativeHyperliquid::create(Ok(
            r#"{"testnet":false,"address":null,"private_key":"0x0123456789012345678901234567890123456789012345678901234567890123"}"#.to_owned(),
        ))
        .unwrap();
        assert!(!signer_only.adapter.supports(maxt::Feature::Balances));
        assert!(signer_only.adapter.supports(maxt::Feature::Trading));
    }

    #[test]
    fn factories_reject_incomplete_credentials_and_unknown_options() {
        let error =
            NativeBithumb::create(Ok(r#"{"access_key":"key","secret_key":null}"#.to_owned()))
                .err()
                .unwrap();
        assert!(matches!(error, Error::InvalidRequest { ref field, .. } if field == "credentials"));

        let error = NativeUpbit::create(Ok(
            r#"{"region":"korea","access_key":null,"secret_key":null,"extra":true}"#.to_owned(),
        ))
        .err()
        .unwrap();
        assert!(matches!(error, Error::InvalidRequest { ref field, .. } if field == "options"));
    }

    #[tokio::test]
    async fn provider_inputs_fail_as_structured_outcomes_before_network_io() {
        let upbit = NativeUpbit::create(Ok(
            r#"{"region":"korea","access_key":null,"secret_key":null}"#.to_owned(),
        ))
        .unwrap();
        let result = upbit.tickers(Ok("{}".to_owned())).await;
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"]["kind"], "invalid_request");
        assert_eq!(result["error"]["field"], "markets");

        let result = upbit.tickers_by_quote(Ok("[]".to_owned())).await;
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"]["kind"], "invalid_request");
        assert_eq!(result["error"]["field"], "quote_currencies");

        let result = upbit
            .order_books_at_level(
                Ok(r#"[{"exchange":"upbit","kind":"spot","base":"BTC","quote":"KRW"}]"#.to_owned()),
                Ok(r#""-1""#.to_owned()),
                Ok("null".to_owned()),
            )
            .await;
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"]["kind"], "invalid_request");
        assert_eq!(result["error"]["field"], "level");

        let result = upbit
            .universal_transfers(Ok(
                r#"{"from":null,"to":null,"direction":"sideways","states":[],"uuids":[],"identifiers":[],"start_time":null,"end_time":null,"currency":null,"limit":null,"order_by":null}"#.to_owned(),
            ))
            .await;
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"]["field"], "direction");

        let result = upbit
            .closed_orders(Ok(
                r#"{"market":null,"state":"future","states":[],"start_time":null,"end_time":null,"limit":null,"order_by":null}"#.to_owned(),
            ))
            .await;
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"]["field"], "state");

        let bithumb =
            NativeBithumb::create(Ok(r#"{"access_key":null,"secret_key":null}"#.to_owned()))
                .unwrap();
        let result = bithumb
            .closed_orders(Ok(
                r#"{"market":null,"state":"future","states":[],"start_time":null,"end_time":null,"limit":null,"order_by":null,"cursor":null}"#.to_owned(),
            ))
            .await;
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"]["kind"], "invalid_request");
        assert_eq!(result["error"]["field"], "state");

        let binance = NativeBinance::create(Ok(
            r#"{"venue":"spot","api_key":null,"secret_key":null}"#.to_owned(),
        ))
        .unwrap();
        let result = binance
            .c2c_trade_history(Ok(
                r#"{"trade_type":"HOLD","start_timestamp":null,"end_timestamp":null,"page":null,"rows":null,"recv_window":null}"#.to_owned(),
            ))
            .await;
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"]["field"], "trade_type");

        assert_eq!(ok_value(outcome(Ok(Value::Null))), Value::Null);
    }
}
