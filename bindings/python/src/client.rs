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
}

include!("generated/client_methods.rs");

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
