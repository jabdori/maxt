use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(all(not(test), not(target_arch = "wasm32")))]
use std::future::Future;

#[cfg(test)]
use maxt::Adapter;
use maxt::adapters::{
    BinanceAdapter, BinanceListenKey, BinanceMarket, BithumbAdapter, HyperliquidAdapter,
    BithumbPendingOrdersRequest, UpbitAdapter, UpbitRegion,
};
use maxt::{Cursor, Error, Market};
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
    WireBinanceSpotOrderDetail, WireBinanceSymbolFilters, WireBithumbMarketAlert,
    WireBithumbApiKey, WireBithumbAssetFee, WireBithumbNotice, WireBithumbPendingOrdersRequest,
    WireHyperliquidAssetContext, WireHyperliquidLedgerEntry, WireMarket, WireOrder,
    WireOrderBook, WirePage, WireTicker,
    WireUpbitMarketEvent, WireUpbitOrderBookInstrument, WireUpbitYearCandle, decimal_from_wire,
    from_wire_text, outcome, timestamp_from_wire,
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
        spawn_native(env, async move {
            this.tickers_by_quote(quote_currencies).await
        })
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
        spawn_native(env, async move { this.year_candles(market, to, count).await })
    }

    #[napi(js_name = "orderbookInstruments", ts_args_type = "markets: string")]
    pub fn orderbook_instruments_native<'env>(
        &self,
        env: &'env Env,
        markets: NativeJsonText<'env>,
    ) -> napi::Result<PromiseRaw<'env, Value>> {
        let this = self.clone();
        let markets = native_json_text(markets, "markets");
        spawn_native(env, async move { this.orderbook_instruments(markets).await })
    }

    #[napi(js_name = "marketEvents")]
    pub async fn market_events_native(&self) -> Value {
        self.market_events().await
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
            Ok(count) => outcome(wire_vec::<_, WireBithumbNotice>(self.adapter.notices(count).await)),
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
        outcome(wire_vec::<_, WireBithumbApiKey>(self.adapter.api_keys().await))
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

    #[wasm_bindgen(js_name = "spotOrder")]
    pub async fn spot_order_wasm(&self, market: String, order_id: String) -> JsValue {
        crate::web::value(self.spot_order(Ok(market), Ok(order_id)).await)
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

        assert_eq!(ok_value(outcome(Ok(Value::Null))), Value::Null);
    }
}
