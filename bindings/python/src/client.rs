use std::future::Future;
use std::sync::Arc;

use maxt::{Adapter, Client, Exchange, Feature};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

use crate::convert::{
    balance_to_wire, candle_request_from_wire, candle_to_wire, core_error, exchange_to_wire,
    funding_payments_page_to_wire, funding_rates_page_to_wire, history_request_from_wire,
    list_to_wire, margin_request_from_wire, margin_summary_to_wire, market_from_wire,
    market_info_to_wire, market_kind_from_wire, order_book_to_wire, order_request_from_wire,
    order_to_wire, position_to_wire, stream_config_from_wire, subscription_from_wire,
    ticker_to_wire, trade_to_wire,
};

#[pyclass(module = "maxt._native")]
pub(crate) struct NativeClient {
    core: Arc<Client<Box<dyn Adapter>>>,
}

impl NativeClient {
    pub(crate) fn from_boxed(adapter: Box<dyn Adapter>) -> Self {
        Self {
            core: Arc::new(Client::new(adapter)),
        }
    }

    pub(crate) fn core(&self) -> Arc<Client<Box<dyn Adapter>>> {
        Arc::clone(&self.core)
    }

    fn exchange_core(&self) -> Exchange {
        self.core.exchange()
    }

    fn supports_core(&self, feature: Feature) -> bool {
        self.core.supports(feature)
    }
}

pub(crate) fn operation<'py, T, F, C>(
    py: Python<'py>,
    future: F,
    convert: C,
) -> PyResult<Bound<'py, PyAny>>
where
    T: Send + 'static,
    F: Future<Output = maxt::Result<T>> + Send + 'static,
    C: FnOnce(Python<'_>, T) -> PyResult<Py<PyAny>> + Send + 'static,
{
    future_into_py(py, async move {
        let value = future.await.map_err(core_error)?;
        Python::attach(|py| convert(py, value))
    })
}

#[pymethods]
impl NativeClient {
    #[getter]
    fn exchange(&self) -> PyResult<&'static str> {
        exchange_to_wire(self.exchange_core())
    }

    fn supports(&self, feature: &Bound<'_, PyAny>) -> PyResult<bool> {
        Ok(self.supports_core(crate::convert::feature_from_wire(feature)?))
    }

    fn markets<'py>(
        &self,
        py: Python<'py>,
        kind: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let kind = market_kind_from_wire(kind)?;
        let core = self.core();
        operation(py, async move { core.markets(kind).await }, |py, values| {
            list_to_wire(py, &values, market_info_to_wire)
        })
    }

    #[pyo3(signature = (market, limit=None))]
    fn trades<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
        limit: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let core = self.core();
        operation(
            py,
            async move { core.trades(&market, limit).await },
            |py, values| list_to_wire(py, &values, trade_to_wire),
        )
    }

    #[pyo3(signature = (market, depth=None))]
    fn order_book<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
        depth: Option<u32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let core = self.core();
        operation(
            py,
            async move { core.order_book(&market, depth).await },
            |py, value| order_book_to_wire(py, &value),
        )
    }

    fn ticker<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let core = self.core();
        operation(
            py,
            async move { core.ticker(&market).await },
            |py, value| ticker_to_wire(py, &value),
        )
    }

    fn candles<'py>(
        &self,
        py: Python<'py>,
        request: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = candle_request_from_wire(request)?;
        let core = self.core();
        operation(
            py,
            async move { core.candles(&request).await },
            |py, values| list_to_wire(py, &values, candle_to_wire),
        )
    }

    fn subscribe<'py>(
        &self,
        py: Python<'py>,
        subscription: &Bound<'_, PyAny>,
        config: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let subscription = subscription_from_wire(subscription)?;
        let config = stream_config_from_wire(config)?;
        let core = self.core();
        operation(
            py,
            async move { core.subscribe_with(&subscription, &config).await },
            |py, value| Ok(crate::stream::market_stream(py, value)?.unbind()),
        )
    }

    fn subscribe_with<'py>(
        &self,
        py: Python<'py>,
        subscription: &Bound<'_, PyAny>,
        config: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.subscribe(py, subscription, config)
    }

    fn balances<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let core = self.core();
        operation(py, async move { core.balances().await }, |py, values| {
            list_to_wire(py, &values, balance_to_wire)
        })
    }

    fn open_orders<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let core = self.core();
        operation(py, async move { core.open_orders().await }, |py, values| {
            list_to_wire(py, &values, order_to_wire)
        })
    }

    fn open_orders_on<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let core = self.core();
        operation(
            py,
            async move { core.open_orders_on(&market).await },
            |py, values| list_to_wire(py, &values, order_to_wire),
        )
    }

    fn subscribe_account<'py>(
        &self,
        py: Python<'py>,
        config: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let config = stream_config_from_wire(config)?;
        let core = self.core();
        operation(
            py,
            async move { core.subscribe_account_with(&config).await },
            |py, value| Ok(crate::stream::account_stream(py, value)?.unbind()),
        )
    }

    fn subscribe_account_with<'py>(
        &self,
        py: Python<'py>,
        config: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.subscribe_account(py, config)
    }

    fn place_order<'py>(
        &self,
        py: Python<'py>,
        request: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = order_request_from_wire(request)?;
        let core = self.core();
        operation(
            py,
            async move { core.place_order(&request).await },
            |py, value| order_to_wire(py, &value),
        )
    }

    fn cancel_order<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
        order_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let core = self.core();
        operation(
            py,
            async move { core.cancel_order(&market, &order_id).await },
            |py, value| order_to_wire(py, &value),
        )
    }

    fn positions<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let core = self.core();
        operation(py, async move { core.positions().await }, |py, values| {
            list_to_wire(py, &values, position_to_wire)
        })
    }

    fn positions_on<'py>(
        &self,
        py: Python<'py>,
        market: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let market = market_from_wire(market)?;
        let core = self.core();
        operation(
            py,
            async move { core.positions_on(&market).await },
            |py, values| list_to_wire(py, &values, position_to_wire),
        )
    }

    fn margin_summary<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let core = self.core();
        operation(
            py,
            async move { core.margin_summary().await },
            |py, value| margin_summary_to_wire(py, &value),
        )
    }

    fn funding_rates<'py>(
        &self,
        py: Python<'py>,
        request: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = history_request_from_wire(request)?;
        let core = self.core();
        operation(
            py,
            async move { core.funding_rates(&request).await },
            |py, value| funding_rates_page_to_wire(py, &value),
        )
    }

    fn funding_payments<'py>(
        &self,
        py: Python<'py>,
        request: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = history_request_from_wire(request)?;
        let core = self.core();
        operation(
            py,
            async move { core.funding_payments(&request).await },
            |py, value| funding_payments_page_to_wire(py, &value),
        )
    }

    fn set_margin<'py>(
        &self,
        py: Python<'py>,
        request: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let request = margin_request_from_wire(request)?;
        let core = self.core();
        operation(
            py,
            async move { core.set_margin(&request).await },
            |py, ()| Ok(py.None()),
        )
    }
}

#[cfg(test)]
mod tests {
    use maxt::adapters::UpbitAdapter;

    use super::*;

    #[test]
    fn native_client_keeps_the_configured_adapter_capabilities() {
        let client = NativeClient::from_boxed(Box::new(UpbitAdapter::new()));

        assert_eq!(client.exchange_core(), Exchange::Upbit);
        assert!(client.supports_core(Feature::Ticker));
        assert!(!client.supports_core(Feature::Trading));
    }
}
