use std::future::Future;
use std::sync::Arc;

use maxt::{Adapter, Client, Exchange, Feature};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;

use crate::convert::{core_error, exchange_to_wire};

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

    fn prepare_transfer_to<'py>(
        &self,
        py: Python<'py>,
        destination: PyRef<'_, NativeClient>,
        request: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let source = self.core();
        let destination = destination.core();
        let request = crate::convert::exchange_transfer_request_from_wire(request)?;
        operation(
            py,
            async move {
                maxt::prepare_exchange_transfer(
                    source.adapter().as_ref(),
                    destination.adapter().as_ref(),
                    &request,
                )
                .await
            },
            |py, value| crate::convert::transfer_plan_to_wire(py, &value),
        )
    }

    fn prepare_transfer_to_chain<'py>(
        &self,
        py: Python<'py>,
        request: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let source = self.core();
        let request = crate::convert::chain_transfer_request_from_wire(request)?;
        operation(
            py,
            async move {
                maxt::prepare_chain_transfer(source.adapter().as_ref(), &request).await
            },
            |py, value| crate::convert::transfer_plan_to_wire(py, &value),
        )
    }

    fn execute_transfer<'py>(
        &self,
        py: Python<'py>,
        plan: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let source = self.core();
        let plan = crate::convert::transfer_plan_from_wire(plan)?;
        operation(
            py,
            async move { maxt::execute_transfer_plan(source.adapter().as_ref(), &plan).await },
            |py, value| crate::convert::withdrawal_to_wire(py, &value),
        )
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
