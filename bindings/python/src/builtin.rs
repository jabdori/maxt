use std::sync::Arc;

#[cfg(test)]
use maxt::Adapter;
use maxt::adapters::{
    BinanceAdapter, BinanceListenKey, BinanceMarket, BithumbAdapter, BithumbAlertStep,
    BithumbMarketAlert, HyperliquidAdapter, HyperliquidLedgerEntry, HyperliquidLedgerKind,
    UpbitAdapter, UpbitMarketEvent, UpbitRegion,
};
use maxt::{Cursor, Market, Page, Timestamp};
use pyo3::IntoPyObjectExt;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::client::{NativeClient, operation};
use crate::convert::{
    binance_mark_price_to_wire, binance_open_interest_to_wire, decimal_from_wire, decimal_to_wire,
    hyperliquid_mid_price_to_wire, list_to_wire, market_from_wire, market_to_wire,
    markets_from_wire, order_book_to_wire, order_to_wire, ticker_to_wire, timestamp_to_wire,
};

macro_rules! provider_dict {
    ($py:expr, $($key:literal => $value:expr),* $(,)?) => {{
        let dict = PyDict::new($py);
        $(dict.set_item($key, $value)?;)*
        Ok::<Py<PyAny>, PyErr>(dict.into_any().unbind())
    }};
}

fn upbit_region(value: &str) -> PyResult<UpbitRegion> {
    match value {
        "korea" => Ok(UpbitRegion::Korea),
        "singapore" => Ok(UpbitRegion::Singapore),
        "indonesia" => Ok(UpbitRegion::Indonesia),
        "thailand" => Ok(UpbitRegion::Thailand),
        _ => Err(PyValueError::new_err(format!(
            "invalid Upbit region: {value}"
        ))),
    }
}

fn upbit_region_name(value: UpbitRegion) -> PyResult<&'static str> {
    match value {
        UpbitRegion::Korea => Ok("korea"),
        UpbitRegion::Singapore => Ok("singapore"),
        UpbitRegion::Indonesia => Ok("indonesia"),
        UpbitRegion::Thailand => Ok("thailand"),
        _ => Err(PyValueError::new_err(
            "maxt binding contract does not map a new UpbitRegion variant",
        )),
    }
}

fn binance_venue(value: &str) -> PyResult<BinanceMarket> {
    match value {
        "spot" => Ok(BinanceMarket::Spot),
        "usd_m" => Ok(BinanceMarket::UsdMFutures),
        _ => Err(PyValueError::new_err(format!(
            "invalid Binance venue: {value}"
        ))),
    }
}

fn binance_venue_name(value: BinanceMarket) -> PyResult<&'static str> {
    match value {
        BinanceMarket::Spot => Ok("spot"),
        BinanceMarket::UsdMFutures => Ok("usd_m"),
        _ => Err(PyValueError::new_err(
            "maxt binding contract does not map a new BinanceMarket variant",
        )),
    }
}

fn credential_pair(
    first: Option<String>,
    second: Option<String>,
    names: (&str, &str),
) -> PyResult<Option<(String, String)>> {
    match (first, second) {
        (None, None) => Ok(None),
        (Some(first), Some(second)) => Ok(Some((first, second))),
        _ => Err(PyValueError::new_err(format!(
            "{} and {} must be provided together",
            names.0, names.1
        ))),
    }
}

#[pyclass(module = "maxt._native", frozen)]
pub(crate) struct NativeUpbitAdapter {
    inner: Arc<UpbitAdapter>,
    authenticated: bool,
}

#[pymethods]
impl NativeUpbitAdapter {
    #[new]
    #[pyo3(signature = (*, region="korea", access_key=None, secret_key=None))]
    fn new(region: &str, access_key: Option<String>, secret_key: Option<String>) -> PyResult<Self> {
        let credentials = credential_pair(access_key, secret_key, ("access_key", "secret_key"))?;
        let mut adapter = UpbitAdapter::with_region(upbit_region(region)?);
        if let Some((access_key, secret_key)) = credentials.as_ref() {
            adapter = adapter.with_credentials(access_key, secret_key);
        }
        Ok(Self {
            inner: Arc::new(adapter),
            authenticated: credentials.is_some_and(|(access_key, secret_key)| {
                !access_key.trim().is_empty() && !secret_key.trim().is_empty()
            }),
        })
    }

    #[getter]
    fn region(&self) -> PyResult<&'static str> {
        upbit_region_name(self.inner.region())
    }

    #[getter]
    fn authenticated(&self) -> bool {
        self.authenticated
    }

    fn client(&self) -> NativeClient {
        NativeClient::from_boxed(Box::new((*self.inner).clone()))
    }

    #[pyo3(signature = (markets, depth=None))]
    fn order_books<'py>(
        &self,
        py: Python<'py>,
        markets: &Bound<'_, PyAny>,
        depth: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let markets = markets_from_wire(markets)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.order_books(&markets, depth).await },
            |py, values| list_to_wire(py, &values, order_book_to_wire),
        )
    }

    #[pyo3(signature = (markets, level, depth=None))]
    fn order_books_at_level<'py>(
        &self,
        py: Python<'py>,
        markets: &Bound<'_, PyAny>,
        level: &Bound<'_, PyAny>,
        depth: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let markets = markets_from_wire(markets)?;
        let level = decimal_from_wire(level, "level")?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.order_books_at_level(&markets, level, depth).await },
            |py, values| list_to_wire(py, &values, order_book_to_wire),
        )
    }

    fn tickers<'py>(
        &self,
        py: Python<'py>,
        markets: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let markets = markets_from_wire(markets)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.tickers(&markets).await },
            |py, values| list_to_wire(py, &values, ticker_to_wire),
        )
    }

    fn tickers_by_quote<'py>(
        &self,
        py: Python<'py>,
        quote_currencies: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.tickers_by_quote(&quote_currencies).await },
            |py, values| list_to_wire(py, &values, ticker_to_wire),
        )
    }

    #[pyo3(signature = (market, to=None, count=None))]
    fn year_candles<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
        to: Option<i64>,
        count: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move {
                adapter
                    .year_candles(&market, to.map(Timestamp::from_nanos), count)
                    .await
            },
            |py, values| list_to_wire(py, &values, crate::convert::upbit_year_candle_to_wire),
        )
    }

    fn orderbook_instruments<'py>(
        &self,
        py: Python<'py>,
        markets: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let markets = markets_from_wire(markets)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.orderbook_instruments(&markets).await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::upbit_order_book_instrument_to_wire,
                )
            },
        )
    }

    fn market_events<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.market_events().await },
            |py, values| list_to_wire(py, &values, upbit_market_event_to_wire),
        )
    }

    fn list_subscriptions<'py>(
        &self,
        py: Python<'py>,
        subscription: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let subscription = crate::convert::subscription_from_wire(subscription)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.list_subscriptions(&subscription).await },
            |py, value| upbit_subscription_list_to_wire(py, &value),
        )
    }

    fn order_detail<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_order_detail_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.order_detail(&request).await },
            |py, value| crate::convert::upbit_order_detail_to_wire(py, &value),
        )
    }

    fn closed_orders<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_closed_orders_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.closed_orders(&request).await },
            |py, values| list_to_wire(py, &values, crate::convert::upbit_closed_order_to_wire),
        )
    }

    fn test_order<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::order_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.test_order(&request).await },
            |py, value| order_to_wire(py, &value),
        )
    }

    fn deposit_info<'py>(
        &self,
        py: Python<'py>,
        asset: String,
        network: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let network = crate::convert::network_from_wire(&network)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.deposit_info(&asset, &network).await },
            |py, value| crate::convert::upbit_deposit_info_to_wire(py, &value),
        )
    }

    fn withdrawal_addresses<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.withdrawal_addresses().await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::upbit_withdrawal_address_to_wire,
                )
            },
        )
    }

    fn travel_rule_vasps<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.travel_rule_vasps().await },
            |py, values| list_to_wire(py, &values, upbit_travel_rule_vasp_to_wire),
        )
    }

    fn verify_travel_rule_by_uuid<'py>(
        &self,
        py: Python<'py>,
        deposit_uuid: String,
        vasp_uuid: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move {
                adapter
                    .verify_travel_rule_by_uuid(&deposit_uuid, &vasp_uuid)
                    .await
            },
            |py, value| upbit_travel_rule_verification_to_wire(py, &value),
        )
    }

    fn verify_travel_rule_by_txid<'py>(
        &self,
        py: Python<'py>,
        txid: String,
        vasp_uuid: String,
        currency: String,
        net_type: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move {
                adapter
                    .verify_travel_rule_by_txid(&txid, &vasp_uuid, &currency, &net_type)
                    .await
            },
            |py, value| upbit_travel_rule_verification_to_wire(py, &value),
        )
    }

    fn batch_cancel_open_orders<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_batch_cancel_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.batch_cancel_open_orders(&request).await },
            |py, value| crate::convert::cancel_orders_result_to_wire(py, &value),
        )
    }

    fn cancel_and_new_order<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_cancel_and_new_order_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.cancel_and_new_order(&request).await },
            |py, value| crate::convert::upbit_cancel_and_new_order_result_to_wire(py, &value),
        )
    }

    fn deposit_krw<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_krw_transfer_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.deposit_krw(&request).await },
            |py, value| crate::convert::upbit_krw_deposit_to_wire(py, &value),
        )
    }

    fn withdraw_krw<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_krw_transfer_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.withdraw_krw(&request).await },
            |py, value| crate::convert::upbit_krw_withdrawal_to_wire(py, &value),
        )
    }

    fn api_keys<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(py, async move { adapter.api_keys().await }, |py, values| {
            list_to_wire(py, &values, crate::convert::upbit_api_key_to_wire)
        })
    }

    fn list_pockets<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.list_pockets().await },
            |py, values| list_to_wire(py, &values, crate::convert::upbit_pocket_to_wire),
        )
    }

    fn list_pocket_api_keys<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_pocket_api_keys_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.list_pocket_api_keys(&request).await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::upbit_pocket_api_key_group_to_wire,
                )
            },
        )
    }

    fn sub_pocket_balances<'py>(
        &self,
        py: Python<'py>,
        pocket_uuid: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.sub_pocket_balances(&pocket_uuid).await },
            |py, values| list_to_wire(py, &values, crate::convert::upbit_pocket_balance_to_wire),
        )
    }

    fn universal_transfer<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_pocket_universal_transfer_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.universal_transfer(&request).await },
            |py, value| crate::convert::upbit_pocket_transfer_to_wire(py, &value),
        )
    }

    fn universal_transfers<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_pocket_transfer_query_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.universal_transfers(&request).await },
            |py, values| list_to_wire(py, &values, crate::convert::upbit_pocket_transfer_to_wire),
        )
    }

    fn sub_pocket_transfer<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_pocket_transfer_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.sub_pocket_transfer(&request).await },
            |py, value| crate::convert::upbit_pocket_transfer_to_wire(py, &value),
        )
    }

    fn sub_pocket_transfers<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::upbit_pocket_transfer_query_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.sub_pocket_transfers(&request).await },
            |py, values| list_to_wire(py, &values, crate::convert::upbit_pocket_transfer_to_wire),
        )
    }
}

#[pyclass(module = "maxt._native", frozen)]
pub(crate) struct NativeBithumbAdapter {
    inner: Arc<BithumbAdapter>,
    authenticated: bool,
}

#[pymethods]
impl NativeBithumbAdapter {
    #[new]
    #[pyo3(signature = (*, access_key=None, secret_key=None))]
    fn new(access_key: Option<String>, secret_key: Option<String>) -> PyResult<Self> {
        let credentials = credential_pair(access_key, secret_key, ("access_key", "secret_key"))?;
        let mut adapter = BithumbAdapter::new();
        if let Some((access_key, secret_key)) = credentials.as_ref() {
            adapter = adapter.with_credentials(access_key, secret_key);
        }
        Ok(Self {
            inner: Arc::new(adapter),
            authenticated: credentials.is_some(),
        })
    }

    #[getter]
    fn authenticated(&self) -> bool {
        self.authenticated
    }

    fn client(&self) -> NativeClient {
        NativeClient::from_boxed(Box::new((*self.inner).clone()))
    }

    fn market_warnings<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.market_warnings().await },
            |py, values| list_to_wire(py, &values, bithumb_market_warning_to_wire),
        )
    }

    fn market_alerts<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.market_alerts().await },
            |py, values| list_to_wire(py, &values, bithumb_market_alert_to_wire),
        )
    }

    #[pyo3(signature = (count=None))]
    fn notices<'py>(&self, py: Python<'py>, count: Option<u32>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.notices(count).await },
            |py, values| list_to_wire(py, &values, crate::convert::bithumb_notice_to_wire),
        )
    }

    fn transfer_fees<'py>(&self, py: Python<'py>, currency: String) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.transfer_fees(&currency).await },
            |py, values| list_to_wire(py, &values, crate::convert::bithumb_asset_fee_to_wire),
        )
    }

    fn api_keys<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(py, async move { adapter.api_keys().await }, |py, values| {
            list_to_wire(py, &values, crate::convert::bithumb_api_key_to_wire)
        })
    }

    fn withdrawal_addresses<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.withdrawal_addresses().await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::bithumb_withdrawal_address_to_wire,
                )
            },
        )
    }

    fn order_detail<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_order_detail_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.order_detail(&request).await },
            |py, value| crate::convert::bithumb_order_detail_to_wire(py, &value),
        )
    }

    fn order_list<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_order_list_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.order_list(&request).await },
            |py, values| list_to_wire(py, &values, crate::convert::bithumb_order_list_item_to_wire),
        )
    }

    fn krw_withdrawals<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_krw_withdrawals_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.krw_withdrawals(&request).await },
            |py, values| list_to_wire(py, &values, bithumb_krw_withdrawal_to_wire),
        )
    }

    fn withdraw_krw<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_krw_transfer_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.withdraw_krw(&request).await },
            |py, value| bithumb_krw_withdrawal_to_wire(py, &value),
        )
    }

    fn krw_deposits<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_krw_deposits_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.krw_deposits(&request).await },
            |py, values| list_to_wire(py, &values, bithumb_krw_deposit_to_wire),
        )
    }

    fn deposit_krw<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_krw_transfer_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.deposit_krw(&request).await },
            |py, value| bithumb_krw_deposit_to_wire(py, &value),
        )
    }

    fn pending_orders<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_pending_orders_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.pending_orders(&request).await },
            |py, value| crate::convert::order_history_page_to_wire(py, &value),
        )
    }

    fn closed_orders<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_closed_orders_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.closed_orders(&request).await },
            |py, value| crate::convert::closed_orders_page_to_wire(py, &value),
        )
    }

    fn twap_orders<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_twap_orders_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.twap_orders(&request).await },
            |py, value| crate::convert::bithumb_twap_order_page_to_wire(py, &value),
        )
    }

    fn create_twap_order<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_twap_order_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.create_twap_order(&request).await },
            |py, value| value.into_py_any(py),
        )
    }

    fn cancel_twap_order<'py>(
        &self,
        py: Python<'py>,
        algo_order_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.cancel_twap_order(&algo_order_id).await },
            |py, value| value.into_py_any(py),
        )
    }

    fn batch_orders<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::bithumb_batch_orders_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.batch_orders(&request).await },
            |py, value| crate::convert::bithumb_batch_orders_result_to_wire(py, &value),
        )
    }
}

#[pyclass(module = "maxt._native", frozen)]
pub(crate) struct NativeBinanceAdapter {
    inner: Arc<BinanceAdapter>,
    authenticated: bool,
}

#[pymethods]
impl NativeBinanceAdapter {
    #[new]
    #[pyo3(signature = (*, venue="spot", api_key=None, secret_key=None))]
    fn new(venue: &str, api_key: Option<String>, secret_key: Option<String>) -> PyResult<Self> {
        let venue = binance_venue(venue)?;
        let credentials = credential_pair(api_key, secret_key, ("api_key", "secret_key"))?;
        let mut adapter = match venue {
            BinanceMarket::Spot => BinanceAdapter::spot(),
            BinanceMarket::UsdMFutures => BinanceAdapter::usd_m_futures(),
            _ => return Err(PyValueError::new_err("unsupported Binance venue")),
        };
        if let Some((api_key, secret_key)) = credentials.as_ref() {
            adapter = adapter.with_credentials(api_key, secret_key);
        }
        Ok(Self {
            inner: Arc::new(adapter),
            authenticated: credentials.is_some(),
        })
    }

    #[getter]
    fn venue(&self) -> PyResult<&'static str> {
        binance_venue_name(self.inner.venue())
    }

    #[getter]
    fn authenticated(&self) -> bool {
        self.authenticated
    }

    fn client(&self) -> NativeClient {
        NativeClient::from_boxed(Box::new((*self.inner).clone()))
    }

    fn spot_symbol_filters<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.spot_symbol_filters(&market).await },
            |py, value| binance_symbol_filters_to_wire(py, &value),
        )
    }

    fn spot_order<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
        order_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.spot_order(&market, &order_id).await },
            |py, value| binance_spot_order_to_wire(py, &value),
        )
    }

    fn spot_average_price<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.spot_average_price(&market).await },
            |py, value| binance_spot_average_price_to_wire(py, &value),
        )
    }

    fn spot_account_information<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.spot_account_information().await },
            |py, value| crate::convert::binance_spot_account_information_to_wire(py, &value),
        )
    }

    fn spot_cancel_all_open_orders<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.spot_cancel_all_open_orders(&market).await },
            |py, value| crate::convert::binance_spot_cancel_all_open_orders_to_wire(py, &value),
        )
    }

    fn spot_exchange_info<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.spot_exchange_info().await },
            |py, value| crate::convert::binance_exchange_info_to_wire(py, &value),
        )
    }

    fn usd_m_account_information<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.usd_m_account_information().await },
            |py, value| crate::convert::binance_usd_m_account_information_to_wire(py, &value),
        )
    }

    fn usd_m_exchange_info<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.usd_m_exchange_info().await },
            |py, value| crate::convert::binance_exchange_info_to_wire(py, &value),
        )
    }

    fn usd_m_position_information<'py>(
        &self,
        py: Python<'py>,
        market: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market.map(market_from_wire).transpose()?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.usd_m_position_information(market.as_ref()).await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::binance_usd_m_position_information_to_wire,
                )
            },
        )
    }

    fn all_coins_information<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.all_coins_information().await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::binance_coin_information_to_wire,
                )
            },
        )
    }

    fn api_key_permissions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.api_key_permissions().await },
            |py, value| crate::convert::binance_api_key_permissions_to_wire(py, &value),
        )
    }

    fn deposit_history<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::binance_deposit_history_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.deposit_history(&request).await },
            |py, value| crate::convert::binance_deposit_history_to_wire(py, &value),
        )
    }

    fn questionnaire_requirements<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.questionnaire_requirements().await },
            |py, value| crate::convert::binance_questionnaire_requirements_to_wire(py, &value),
        )
    }

    fn withdraw_address_list<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.withdraw_address_list().await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::binance_withdrawal_address_to_wire,
                )
            },
        )
    }

    fn withdraw_history<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::binance_withdraw_history_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.withdraw_history(&request).await },
            |py, value| crate::convert::binance_withdraw_history_to_wire(py, &value),
        )
    }

    fn mark_price<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.mark_price(&market).await },
            |py, value| binance_mark_price_to_wire(py, &value),
        )
    }

    fn mark_prices<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.mark_prices().await },
            |py, values| list_to_wire(py, &values, binance_mark_price_to_wire),
        )
    }

    fn open_interest<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.open_interest(&market).await },
            |py, value| binance_open_interest_to_wire(py, &value),
        )
    }

    fn aggregate_trades<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::binance_aggregate_trades_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.aggregate_trades(&request).await },
            |py, values| list_to_wire(py, &values, crate::convert::binance_aggregate_trade_to_wire),
        )
    }

    fn account_trades<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::history_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.account_trades(&request).await },
            |py, value| crate::convert::account_trades_page_to_wire(py, &value),
        )
    }

    fn c2c_trade_history<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::binance_c2c_trade_history_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.c2c_trade_history(&request).await },
            |py, value| crate::convert::binance_c2c_trade_history_page_to_wire(py, &value),
        )
    }

    fn test_order<'py>(
        &self,
        py: Python<'py>,
        request: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = crate::convert::binance_test_order_request_from_wire(&request)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.test_order(&request).await },
            |py, value| crate::convert::binance_test_order_to_wire(py, &value),
        )
    }

    fn cancel_all_open_orders<'py>(
        &self,
        py: Python<'py>,
        market: Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(&market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.cancel_all_open_orders(&market).await },
            |py, ()| Ok(py.None()),
        )
    }

    fn usd_m_create_listen_key<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.usd_m_create_listen_key().await },
            |py, inner| Ok(Py::new(py, NativeBinanceListenKey { inner })?.into_any()),
        )
    }

    fn usd_m_keepalive_listen_key<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.usd_m_keepalive_listen_key().await },
            |py, ()| Ok(py.None()),
        )
    }

    fn usd_m_close_listen_key<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.usd_m_close_listen_key().await },
            |py, ()| Ok(py.None()),
        )
    }
}

#[pyclass(module = "maxt._native", frozen)]
pub(crate) struct NativeBinanceListenKey {
    inner: BinanceListenKey,
}

#[pymethods]
impl NativeBinanceListenKey {
    #[getter]
    fn value(&self) -> String {
        self.inner.as_str().to_owned()
    }
}

#[pyclass(module = "maxt._native", frozen)]
pub(crate) struct NativeHyperliquidAdapter {
    inner: Arc<HyperliquidAdapter>,
    authenticated: bool,
}

#[pymethods]
impl NativeHyperliquidAdapter {
    #[new]
    #[pyo3(signature = (*, testnet=false, address=None, private_key=None))]
    fn new(testnet: bool, address: Option<String>, private_key: Option<String>) -> PyResult<Self> {
        let authenticated = private_key.is_some();
        let mut adapter = if testnet {
            HyperliquidAdapter::testnet()
        } else {
            HyperliquidAdapter::new()
        };
        if let Some(address) = address {
            adapter = adapter.with_query_address(address);
        }
        if let Some(private_key) = private_key {
            adapter = adapter.with_signer(private_key);
        }
        Ok(Self {
            inner: Arc::new(adapter),
            authenticated,
        })
    }

    #[getter]
    fn is_testnet(&self) -> bool {
        self.inner.is_testnet()
    }

    #[getter]
    fn authenticated(&self) -> bool {
        self.authenticated
    }

    fn client(&self) -> NativeClient {
        NativeClient::from_boxed(Box::new((*self.inner).clone()))
    }

    #[pyo3(signature = (from_ns=None, to_ns=None, cursor=None, limit=None))]
    fn non_funding_ledger<'py>(
        &self,
        py: Python<'py>,
        from_ns: Option<i64>,
        to_ns: Option<i64>,
        cursor: Option<String>,
        limit: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let from = from_ns.map(Timestamp::from_nanos);
        let to = to_ns.map(Timestamp::from_nanos);
        let cursor = cursor.map(Cursor::new);
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move {
                adapter
                    .non_funding_ledger(from, to, cursor.as_ref(), limit)
                    .await
            },
            |py, value| hyperliquid_ledger_page_to_wire(py, &value),
        )
    }

    fn asset_context<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.asset_context(&market).await },
            |py, value| hyperliquid_asset_context_to_wire(py, &value),
        )
    }

    #[pyo3(signature = (market, interval, from_ns, to_ns=None))]
    fn candle_snapshot<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
        interval: String,
        from_ns: i64,
        to_ns: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let from = Timestamp::from_nanos(from_ns);
        let to = to_ns.map(Timestamp::from_nanos);
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.candle_snapshot(&market, &interval, from, to).await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::hyperliquid_candle_snapshot_to_wire,
                )
            },
        )
    }

    fn l2_book<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.l2_book(&market).await },
            |py, value| crate::convert::hyperliquid_l2_book_to_wire(py, &value),
        )
    }

    fn recent_trades<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.recent_trades(&market).await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::hyperliquid_recent_trade_to_wire,
                )
            },
        )
    }

    #[pyo3(signature = (market, from_ns, to_ns=None))]
    fn funding_history<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
        from_ns: i64,
        to_ns: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let from = Timestamp::from_nanos(from_ns);
        let to = to_ns.map(Timestamp::from_nanos);
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.funding_history(&market, from, to).await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::hyperliquid_funding_history_entry_to_wire,
                )
            },
        )
    }

    #[pyo3(signature = (from_ns, to_ns=None))]
    fn user_funding<'py>(
        &self,
        py: Python<'py>,
        from_ns: i64,
        to_ns: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let from = Timestamp::from_nanos(from_ns);
        let to = to_ns.map(Timestamp::from_nanos);
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.user_funding(from, to).await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::hyperliquid_user_funding_to_wire,
                )
            },
        )
    }

    fn spot_clearinghouse_state<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.spot_clearinghouse_state().await },
            |py, value| crate::convert::hyperliquid_spot_clearinghouse_state_to_wire(py, &value),
        )
    }

    fn spot_meta<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(py, async move { adapter.spot_meta().await }, |py, value| {
            crate::convert::hyperliquid_spot_meta_to_wire(py, &value)
        })
    }

    fn spot_meta_and_asset_contexts<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.spot_meta_and_asset_contexts().await },
            |py, value| {
                crate::convert::hyperliquid_spot_meta_and_asset_contexts_to_wire(py, &value)
            },
        )
    }

    fn all_mids<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(py, async move { adapter.all_mids().await }, |py, values| {
            list_to_wire(py, &values, hyperliquid_mid_price_to_wire)
        })
    }

    fn subscribe_detailed<'py>(
        &self,
        py: Python<'py>,
        subscription: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let subscription = crate::convert::subscription_from_wire(subscription)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.subscribe_detailed(&subscription).await },
            |py, stream| crate::stream::hyperliquid_market_stream(py, stream).map(Bound::unbind),
        )
    }

    fn subscribe_detailed_with<'py>(
        &self,
        py: Python<'py>,
        subscription: &Bound<'_, PyAny>,
        config: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let subscription = crate::convert::subscription_from_wire(subscription)?;
        let config = crate::convert::stream_config_from_wire(config)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move {
                adapter
                    .subscribe_detailed_with(&subscription, &config)
                    .await
            },
            |py, stream| crate::stream::hyperliquid_market_stream(py, stream).map(Bound::unbind),
        )
    }

    fn subscribe_detailed_account<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.subscribe_detailed_account().await },
            |py, stream| crate::stream::hyperliquid_account_stream(py, stream).map(Bound::unbind),
        )
    }

    fn subscribe_detailed_account_with<'py>(
        &self,
        py: Python<'py>,
        config: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let config = crate::convert::stream_config_from_wire(config)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.subscribe_detailed_account_with(&config).await },
            |py, stream| crate::stream::hyperliquid_account_stream(py, stream).map(Bound::unbind),
        )
    }

    fn basic_open_orders<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.basic_open_orders().await },
            |py, values| list_to_wire(py, &values, crate::convert::hyperliquid_open_order_to_wire),
        )
    }

    fn order_status<'py>(
        &self,
        py: Python<'py>,
        reference: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let reference = crate::convert::hyperliquid_order_reference_from_wire(reference)?;
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.order_status(reference).await },
            |py, value| crate::convert::hyperliquid_order_status_response_to_wire(py, &value),
        )
    }

    fn historical_orders<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.historical_orders().await },
            |py, values| list_to_wire(py, &values, crate::convert::hyperliquid_order_info_to_wire),
        )
    }

    fn user_fills<'py>(
        &self,
        py: Python<'py>,
        aggregate_by_time: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.user_fills(aggregate_by_time).await },
            |py, values| list_to_wire(py, &values, crate::convert::hyperliquid_user_fill_to_wire),
        )
    }

    #[pyo3(signature = (from_ns, to_ns=None, aggregate_by_time=false))]
    fn user_fills_by_time<'py>(
        &self,
        py: Python<'py>,
        from_ns: i64,
        to_ns: Option<i64>,
        aggregate_by_time: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let from = Timestamp::from_nanos(from_ns);
        let to = to_ns.map(Timestamp::from_nanos);
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move {
                adapter
                    .user_fills_by_time(from, to, aggregate_by_time)
                    .await
            },
            |py, values| list_to_wire(py, &values, crate::convert::hyperliquid_user_fill_to_wire),
        )
    }

    fn user_rate_limit<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.user_rate_limit().await },
            |py, value| crate::convert::hyperliquid_user_rate_limit_to_wire(py, &value),
        )
    }

    fn user_role<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(py, async move { adapter.user_role().await }, |py, value| {
            crate::convert::hyperliquid_user_role_to_wire(py, &value)
        })
    }

    fn referral<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(py, async move { adapter.referral().await }, |py, value| {
            crate::convert::hyperliquid_referral_to_wire(py, &value)
        })
    }

    fn user_fees<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(py, async move { adapter.user_fees().await }, |py, value| {
            crate::convert::hyperliquid_user_fees_to_wire(py, &value)
        })
    }

    fn portfolio<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.portfolio().await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::hyperliquid_portfolio_period_to_wire,
                )
            },
        )
    }

    fn sub_accounts<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.sub_accounts().await },
            |py, values| list_to_wire(py, &values, crate::convert::hyperliquid_sub_account_to_wire),
        )
    }

    fn user_vault_equities<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let adapter = Arc::clone(&self.inner);
        operation(
            py,
            async move { adapter.user_vault_equities().await },
            |py, values| {
                list_to_wire(
                    py,
                    &values,
                    crate::convert::hyperliquid_vault_equity_to_wire,
                )
            },
        )
    }
}

include!("generated/provider_convert.rs");
include!("generated/provider_native.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_must_be_complete_pairs() {
        assert!(credential_pair(None, None, ("access_key", "secret_key")).is_ok());
        assert!(credential_pair(Some("key".into()), None, ("access_key", "secret_key")).is_err());
        assert!(
            credential_pair(None, Some("secret".into()), ("access_key", "secret_key")).is_err()
        );
        assert_eq!(
            credential_pair(
                Some("key".into()),
                Some("secret".into()),
                ("access_key", "secret_key")
            )
            .unwrap(),
            Some(("key".into(), "secret".into()))
        );
    }

    #[test]
    fn constructor_names_select_every_rust_adapter_variant() {
        assert_eq!(upbit_region("korea").unwrap(), UpbitRegion::Korea);
        assert_eq!(upbit_region("singapore").unwrap(), UpbitRegion::Singapore);
        assert_eq!(upbit_region("indonesia").unwrap(), UpbitRegion::Indonesia);
        assert_eq!(upbit_region("thailand").unwrap(), UpbitRegion::Thailand);
        assert_eq!(binance_venue("spot").unwrap(), BinanceMarket::Spot);
        assert_eq!(binance_venue("usd_m").unwrap(), BinanceMarket::UsdMFutures);
        assert!(upbit_region("global").is_err());
        assert!(binance_venue("coin_m").is_err());
    }

    #[test]
    fn constructors_keep_credentials_and_network_selection() {
        let upbit = NativeUpbitAdapter::new("singapore", Some("key".into()), Some("secret".into()))
            .unwrap();
        assert_eq!(upbit.region().unwrap(), "singapore");
        assert!(upbit.authenticated());

        assert!(
            !NativeUpbitAdapter::new("korea", Some(" ".into()), Some("secret".into()))
                .unwrap()
                .authenticated()
        );
        assert!(
            !NativeUpbitAdapter::new("korea", Some("access".into()), Some(" \t".into()))
                .unwrap()
                .authenticated()
        );

        let binance =
            NativeBinanceAdapter::new("usd_m", Some("key".into()), Some("secret".into())).unwrap();
        assert_eq!(binance.venue().unwrap(), "usd_m");
        assert!(binance.authenticated());

        let hyperliquid = NativeHyperliquidAdapter::new(true, None, None).unwrap();
        assert!(hyperliquid.is_testnet());
        assert!(!hyperliquid.authenticated());

        let address_only = NativeHyperliquidAdapter::new(
            false,
            Some("0x14791697260e4c9a71f18484c9f997b308e59325".into()),
            None,
        )
        .unwrap();
        assert!(address_only.inner.supports(maxt::Feature::Balances));
        assert!(!address_only.inner.supports(maxt::Feature::Trading));

        let signer_only = NativeHyperliquidAdapter::new(
            false,
            None,
            Some("0x0123456789012345678901234567890123456789012345678901234567890123".into()),
        )
        .unwrap();
        assert!(!signer_only.inner.supports(maxt::Feature::Balances));
        assert!(signer_only.inner.supports(maxt::Feature::Trading));
        assert!(signer_only.authenticated());
    }
}
