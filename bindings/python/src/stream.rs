//! Python asynchronous iterator bridge for maxt streams.

use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use futures_core::Stream;
use maxt::{
    AccountEvent, AccountStream, HyperliquidAccountEvent, HyperliquidAccountStream,
    HyperliquidMarketEvent, HyperliquidMarketStream, MarketEvent, MarketStream, Result,
};
use pyo3::exceptions::{PyRuntimeError, PyStopAsyncIteration};
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict};

use crate::convert::{
    balance_to_wire, candle_to_wire, error_to_wire, order_book_to_wire, order_to_wire,
    ticker_to_wire, trade_to_wire,
};

type NextFuture = Pin<Box<dyn Future<Output = PyResult<Py<PyAny>>> + Send + 'static>>;
type Parse<T> = for<'py> fn(&Bound<'py, PyAny>) -> Result<T>;

enum StreamPoll<T> {
    Closed,
    Item(Option<T>),
}

struct PendingNext {
    cancellation: Arc<PendingCancellation>,
    future: NextFuture,
}

struct PendingCancellation {
    requested: AtomicBool,
    task: Mutex<Option<Py<PyAny>>>,
    event_loop: Py<PyAny>,
}

impl PendingCancellation {
    fn new(event_loop: Py<PyAny>) -> Self {
        Self {
            requested: AtomicBool::new(false),
            task: Mutex::new(None),
            event_loop,
        }
    }

    fn register(&self, py: Python<'_>, task: &Py<PyAny>) -> PyResult<()> {
        *self
            .task
            .lock()
            .expect("pending Python task mutex poisoned") = Some(task.clone_ref(py));
        if self.requested.load(Ordering::Acquire) {
            task.call_method0(py, "cancel")?;
        }
        Ok(())
    }

    fn request(&self) -> PyResult<()> {
        self.requested.store(true, Ordering::Release);
        Python::try_attach(|py| {
            let task = self
                .task
                .lock()
                .expect("pending Python task mutex poisoned")
                .as_ref()
                .map(|task| task.clone_ref(py));
            let Some(task) = task else {
                return Ok(());
            };
            let cancel = task.bind(py).getattr("cancel")?;
            self.event_loop
                .bind(py)
                .call_method1("call_soon_threadsafe", (cancel,))?;
            Ok(())
        })
        .unwrap_or(Ok(()))
    }
}

#[pyclass]
struct ScheduleCall {
    call: Option<Py<PyAny>>,
    sender: Option<tokio::sync::oneshot::Sender<PyResult<Py<PyAny>>>>,
    cancellation: Arc<PendingCancellation>,
}

#[pymethods]
impl ScheduleCall {
    fn __call__(&mut self, py: Python<'_>) -> PyResult<()> {
        let result = (|| {
            let call = self
                .call
                .take()
                .ok_or_else(|| PyRuntimeError::new_err("Python call was already scheduled"))?;
            let awaitable = call.bind(py).call0()?;
            let task = py
                .import("asyncio")?
                .call_method1("ensure_future", (awaitable,))?
                .unbind();
            self.cancellation.register(py, &task)?;
            Ok(task)
        })();
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(result);
        }
        Ok(())
    }
}

fn schedule_call(
    py: Python<'_>,
    call: Py<PyAny>,
    locals: pyo3_async_runtimes::TaskLocals,
    operation: &'static str,
) -> PyResult<PendingNext> {
    let event_loop = locals.event_loop(py);
    let cancellation = Arc::new(PendingCancellation::new(event_loop.clone().unbind()));
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let callback = Py::new(
        py,
        ScheduleCall {
            call: Some(call),
            sender: Some(sender),
            cancellation: Arc::clone(&cancellation),
        },
    )?;
    let kwargs = PyDict::new(py);
    kwargs.set_item("context", locals.context(py))?;
    event_loop.call_method("call_soon_threadsafe", (callback,), Some(&kwargs))?;

    let future = Box::pin(async move {
        let task = receiver.await.map_err(|_| {
            PyRuntimeError::new_err(format!("Python event loop did not schedule {operation}"))
        })??;
        let future = Python::attach(|py| {
            pyo3_async_runtimes::into_future_with_locals(&locals, task.into_bound(py))
        })?;
        future.await
    }) as NextFuture;
    Ok(PendingNext {
        cancellation,
        future,
    })
}

pub(crate) fn market_stream_from_python(value: &Bound<'_, PyAny>) -> PyResult<MarketStream> {
    let iterator = value.call_method0("__aiter__")?.unbind();
    let locals = pyo3_async_runtimes::tokio::get_current_locals(value.py()).ok();
    let source = Arc::new(PythonSource::new(iterator, locals));
    let close = Arc::clone(&source);
    Ok(MarketStream::new_with_close(
        PythonIterator::new(source, crate::adapter::market_stream_item),
        move || close.close(),
    ))
}

pub(crate) fn account_stream_from_python(value: &Bound<'_, PyAny>) -> PyResult<AccountStream> {
    let iterator = value.call_method0("__aiter__")?.unbind();
    let locals = pyo3_async_runtimes::tokio::get_current_locals(value.py()).ok();
    let source = Arc::new(PythonSource::new(iterator, locals));
    let close = Arc::clone(&source);
    Ok(AccountStream::new_with_close(
        PythonIterator::new(source, crate::adapter::account_stream_item),
        move || close.close(),
    ))
}

struct PythonSource {
    iterator: Py<PyAny>,
    pending: Mutex<Option<PendingNext>>,
    locals: Mutex<Option<pyo3_async_runtimes::TaskLocals>>,
    cleanup_started: AtomicBool,
    completed: tokio::sync::watch::Sender<Option<Result<()>>>,
}

impl PythonSource {
    fn new(iterator: Py<PyAny>, locals: Option<pyo3_async_runtimes::TaskLocals>) -> Self {
        let (completed, _) = tokio::sync::watch::channel(None);
        Self {
            iterator,
            pending: Mutex::new(None),
            locals: Mutex::new(locals),
            cleanup_started: AtomicBool::new(false),
            completed,
        }
    }

    fn next_future(&self) -> PyResult<PendingNext> {
        Python::attach(|py| {
            let call = self.iterator.bind(py).getattr("__anext__")?.unbind();
            let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
            *self
                .locals
                .lock()
                .expect("Python task locals mutex poisoned") = Some(locals.clone());
            schedule_call(py, call, locals, "__anext__")
        })
    }

    async fn close(self: Arc<Self>) -> Result<()> {
        if self
            .cleanup_started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return self.wait_for_completion().await;
        }
        let result = self.cleanup().await;
        self.completed.send_replace(Some(result.clone()));
        result
    }

    fn close_on_drop(self: &Arc<Self>) {
        if self
            .cleanup_started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let source = Arc::clone(self);
        pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
            let result = source.cleanup().await;
            source.completed.send_replace(Some(result));
        });
    }

    async fn wait_for_completion(&self) -> Result<()> {
        let mut completed = self.completed.subscribe();
        loop {
            if let Some(result) = completed.borrow().clone() {
                return result;
            }
            if completed.changed().await.is_err() {
                return Err(maxt::Error::adapter(
                    "Python stream cleanup completion channel closed",
                ));
            }
        }
    }

    async fn cleanup(&self) -> Result<()> {
        let pending = self
            .pending
            .lock()
            .expect("pending Python future mutex poisoned")
            .take();
        if let Some(pending) = pending {
            pending
                .cancellation
                .request()
                .map_err(crate::adapter::python_error)?;
            let _ = pending.future.await;
        }

        let locals = self
            .locals
            .lock()
            .expect("Python task locals mutex poisoned")
            .clone()
            .or_else(|| {
                Python::try_attach(pyo3_async_runtimes::tokio::get_current_locals)
                    .and_then(std::result::Result::ok)
            })
            .ok_or_else(|| maxt::Error::adapter("Python stream has no active event loop"))?;
        let pending = Python::try_attach(|py| -> PyResult<Option<PendingNext>> {
            let Some(close) = self.iterator.bind(py).getattr_opt("aclose")? else {
                return Ok(None);
            };
            schedule_call(py, close.unbind(), locals.clone(), "aclose").map(Some)
        })
        .ok_or_else(|| maxt::Error::adapter("Python runtime is not available"))?
        .map_err(crate::adapter::python_error)?;
        match pending {
            Some(pending) => pending
                .future
                .await
                .map(|_| ())
                .map_err(crate::adapter::python_error),
            None => Ok(()),
        }
    }
}

struct PythonIterator<T> {
    source: Arc<PythonSource>,
    parse: Parse<T>,
    finished: bool,
}

impl<T> PythonIterator<T> {
    fn new(source: Arc<PythonSource>, parse: Parse<T>) -> Self {
        Self {
            source,
            parse,
            finished: false,
        }
    }
}

impl<T> Stream for PythonIterator<T> {
    type Item = Result<T>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.finished {
            return Poll::Ready(None);
        }
        if self
            .source
            .pending
            .lock()
            .expect("pending Python future mutex poisoned")
            .is_none()
        {
            match self.source.next_future() {
                Ok(future) => {
                    *self
                        .source
                        .pending
                        .lock()
                        .expect("pending Python future mutex poisoned") = Some(future);
                }
                Err(error) if is_stop_async_iteration(&error) => {
                    self.finished = true;
                    return Poll::Ready(None);
                }
                Err(error) => return Poll::Ready(Some(Err(crate::adapter::python_error(error)))),
            }
        }
        let result = self
            .source
            .pending
            .lock()
            .expect("pending Python future mutex poisoned")
            .as_mut()
            .expect("pending Python future must exist")
            .future
            .as_mut()
            .poll(cx);
        match result {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                self.source
                    .pending
                    .lock()
                    .expect("pending Python future mutex poisoned")
                    .take();
                match result {
                    Ok(value) => {
                        Poll::Ready(Some(Python::attach(|py| (self.parse)(value.bind(py)))))
                    }
                    Err(error) if is_stop_async_iteration(&error) => {
                        self.finished = true;
                        Poll::Ready(None)
                    }
                    Err(error) => Poll::Ready(Some(Err(crate::adapter::python_error(error)))),
                }
            }
        }
    }
}

impl<T> Drop for PythonIterator<T> {
    fn drop(&mut self) {
        self.source.close_on_drop();
    }
}

fn is_stop_async_iteration(error: &PyErr) -> bool {
    Python::attach(|py| error.is_instance_of::<PyStopAsyncIteration>(py))
}

#[pyclass(module = "maxt._native")]
pub(crate) struct NativeMarketStream {
    state: Arc<NativeStreamState<MarketStream>>,
}

impl NativeMarketStream {
    fn new(stream: MarketStream) -> Self {
        Self {
            state: Arc::new(NativeStreamState::new(stream)),
        }
    }
}

#[pymethods]
impl NativeMarketStream {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let state = Arc::clone(&self.state);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let item = {
                let mut guard = state.inner.lock().await;
                if state.closed.load(Ordering::Acquire) {
                    return Err(PyStopAsyncIteration::new_err(()));
                }
                let Some(stream) = guard.as_mut() else {
                    return Err(PyStopAsyncIteration::new_err(()));
                };
                let polled = tokio::select! {
                    biased;
                    _ = state.close.notified() => StreamPoll::Closed,
                    item = poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)) => StreamPoll::Item(item),
                };
                match polled {
                    StreamPoll::Closed => None,
                    StreamPoll::Item(_) if state.closed.load(Ordering::Acquire) => None,
                    StreamPoll::Item(Some(item)) => Some(item),
                    StreamPoll::Item(None) => {
                        state.closed.store(true, Ordering::Release);
                        None
                    }
                }
            };
            match item {
                Some(item) => Python::attach(|py| market_item_to_wire(py, item)),
                None => Err(PyStopAsyncIteration::new_err(())),
            }
        })
    }

    fn aclose<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let state = Arc::clone(&self.state);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            state.close().await.map_err(crate::convert::core_error)?;
            Ok(())
        })
    }
}

#[pyclass(module = "maxt._native")]
pub(crate) struct NativeAccountStream {
    state: Arc<NativeStreamState<AccountStream>>,
}

#[pyclass(module = "maxt._native")]
pub(crate) struct NativeHyperliquidMarketStream {
    state: Arc<NativeStreamState<HyperliquidMarketStream>>,
}

impl NativeHyperliquidMarketStream {
    fn new(stream: HyperliquidMarketStream) -> Self {
        Self {
            state: Arc::new(NativeStreamState::new(stream)),
        }
    }
}

#[pymethods]
impl NativeHyperliquidMarketStream {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let state = Arc::clone(&self.state);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let item = {
                let mut guard = state.inner.lock().await;
                if state.closed.load(Ordering::Acquire) {
                    return Err(PyStopAsyncIteration::new_err(()));
                }
                let Some(stream) = guard.as_mut() else {
                    return Err(PyStopAsyncIteration::new_err(()));
                };
                let polled = tokio::select! {
                    biased;
                    _ = state.close.notified() => StreamPoll::Closed,
                    item = poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)) => StreamPoll::Item(item),
                };
                match polled {
                    StreamPoll::Closed => None,
                    StreamPoll::Item(_) if state.closed.load(Ordering::Acquire) => None,
                    StreamPoll::Item(Some(item)) => Some(item),
                    StreamPoll::Item(None) => {
                        state.closed.store(true, Ordering::Release);
                        None
                    }
                }
            };
            match item {
                Some(item) => Python::attach(|py| hyperliquid_market_item_to_wire(py, item)),
                None => Err(PyStopAsyncIteration::new_err(())),
            }
        })
    }

    fn aclose<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let state = Arc::clone(&self.state);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            state.close().await.map_err(crate::convert::core_error)?;
            Ok(())
        })
    }
}

#[pyclass(module = "maxt._native")]
pub(crate) struct NativeHyperliquidAccountStream {
    state: Arc<NativeStreamState<HyperliquidAccountStream>>,
}

impl NativeHyperliquidAccountStream {
    fn new(stream: HyperliquidAccountStream) -> Self {
        Self {
            state: Arc::new(NativeStreamState::new(stream)),
        }
    }
}

#[pymethods]
impl NativeHyperliquidAccountStream {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let state = Arc::clone(&self.state);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let item = {
                let mut guard = state.inner.lock().await;
                if state.closed.load(Ordering::Acquire) {
                    return Err(PyStopAsyncIteration::new_err(()));
                }
                let Some(stream) = guard.as_mut() else {
                    return Err(PyStopAsyncIteration::new_err(()));
                };
                let polled = tokio::select! {
                    biased;
                    _ = state.close.notified() => StreamPoll::Closed,
                    item = poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)) => StreamPoll::Item(item),
                };
                match polled {
                    StreamPoll::Closed => None,
                    StreamPoll::Item(_) if state.closed.load(Ordering::Acquire) => None,
                    StreamPoll::Item(Some(item)) => Some(item),
                    StreamPoll::Item(None) => {
                        state.closed.store(true, Ordering::Release);
                        None
                    }
                }
            };
            match item {
                Some(item) => Python::attach(|py| hyperliquid_account_item_to_wire(py, item)),
                None => Err(PyStopAsyncIteration::new_err(())),
            }
        })
    }

    fn aclose<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let state = Arc::clone(&self.state);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            state.close().await.map_err(crate::convert::core_error)?;
            Ok(())
        })
    }
}

impl NativeAccountStream {
    fn new(stream: AccountStream) -> Self {
        Self {
            state: Arc::new(NativeStreamState::new(stream)),
        }
    }
}

#[pymethods]
impl NativeAccountStream {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let state = Arc::clone(&self.state);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let item = {
                let mut guard = state.inner.lock().await;
                if state.closed.load(Ordering::Acquire) {
                    return Err(PyStopAsyncIteration::new_err(()));
                }
                let Some(stream) = guard.as_mut() else {
                    return Err(PyStopAsyncIteration::new_err(()));
                };
                let polled = tokio::select! {
                    biased;
                    _ = state.close.notified() => StreamPoll::Closed,
                    item = poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)) => StreamPoll::Item(item),
                };
                match polled {
                    StreamPoll::Closed => None,
                    StreamPoll::Item(_) if state.closed.load(Ordering::Acquire) => None,
                    StreamPoll::Item(Some(item)) => Some(item),
                    StreamPoll::Item(None) => {
                        state.closed.store(true, Ordering::Release);
                        None
                    }
                }
            };
            match item {
                Some(item) => Python::attach(|py| account_item_to_wire(py, item)),
                None => Err(PyStopAsyncIteration::new_err(())),
            }
        })
    }

    fn aclose<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let state = Arc::clone(&self.state);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            state.close().await.map_err(crate::convert::core_error)?;
            Ok(())
        })
    }
}

struct NativeStreamState<S> {
    inner: tokio::sync::Mutex<Option<S>>,
    closed: AtomicBool,
    close: tokio::sync::Notify,
}

impl<S> NativeStreamState<S> {
    fn new(stream: S) -> Self {
        Self {
            inner: tokio::sync::Mutex::new(Some(stream)),
            closed: AtomicBool::new(false),
            close: tokio::sync::Notify::new(),
        }
    }
}

impl NativeStreamState<MarketStream> {
    async fn close(&self) -> Result<()> {
        self.closed.store(true, Ordering::Release);
        self.close.notify_one();
        let mut guard = self.inner.lock().await;
        let result = match guard.as_mut() {
            Some(stream) => stream.close().await,
            None => Ok(()),
        };
        guard.take();
        result
    }
}

impl NativeStreamState<AccountStream> {
    async fn close(&self) -> Result<()> {
        self.closed.store(true, Ordering::Release);
        self.close.notify_one();
        let mut guard = self.inner.lock().await;
        let result = match guard.as_mut() {
            Some(stream) => stream.close().await,
            None => Ok(()),
        };
        guard.take();
        result
    }
}

impl NativeStreamState<HyperliquidMarketStream> {
    async fn close(&self) -> Result<()> {
        self.closed.store(true, Ordering::Release);
        self.close.notify_one();
        let mut guard = self.inner.lock().await;
        let result = match guard.as_mut() {
            Some(stream) => stream.close().await,
            None => Ok(()),
        };
        guard.take();
        result
    }
}

impl NativeStreamState<HyperliquidAccountStream> {
    async fn close(&self) -> Result<()> {
        self.closed.store(true, Ordering::Release);
        self.close.notify_one();
        let mut guard = self.inner.lock().await;
        let result = match guard.as_mut() {
            Some(stream) => stream.close().await,
            None => Ok(()),
        };
        guard.take();
        result
    }
}

pub(crate) fn market_stream<'py>(
    py: Python<'py>,
    stream: MarketStream,
) -> PyResult<Bound<'py, PyAny>> {
    Bound::new(py, NativeMarketStream::new(stream)).map(Bound::into_any)
}

pub(crate) fn account_stream<'py>(
    py: Python<'py>,
    stream: AccountStream,
) -> PyResult<Bound<'py, PyAny>> {
    Bound::new(py, NativeAccountStream::new(stream)).map(Bound::into_any)
}

pub(crate) fn hyperliquid_market_stream<'py>(
    py: Python<'py>,
    stream: HyperliquidMarketStream,
) -> PyResult<Bound<'py, PyAny>> {
    Bound::new(py, NativeHyperliquidMarketStream::new(stream)).map(Bound::into_any)
}

pub(crate) fn hyperliquid_account_stream<'py>(
    py: Python<'py>,
    stream: HyperliquidAccountStream,
) -> PyResult<Bound<'py, PyAny>> {
    Bound::new(py, NativeHyperliquidAccountStream::new(stream)).map(Bound::into_any)
}

fn market_item_to_wire(py: Python<'_>, item: Result<MarketEvent>) -> PyResult<Py<PyAny>> {
    match item {
        Ok(event) => {
            let (kind, value) = match event {
                MarketEvent::Trade(value) => ("trade", trade_to_wire(py, &value)?),
                MarketEvent::OrderBook(value) => ("order_book", order_book_to_wire(py, &value)?),
                MarketEvent::Ticker(value) => ("ticker", ticker_to_wire(py, &value)?),
                MarketEvent::Candle(value) => ("candle", candle_to_wire(py, &value)?),
                MarketEvent::Reconnected => ("reconnected", py.None()),
                _ => return Err(binding_contract("MarketEvent")),
            };
            stream_event_wire(py, kind, value)
        }
        Err(error) => stream_error_wire(py, &error),
    }
}

fn account_item_to_wire(py: Python<'_>, item: Result<AccountEvent>) -> PyResult<Py<PyAny>> {
    match item {
        Ok(event) => {
            let (kind, value) = match event {
                AccountEvent::Balance(value) => ("balance", balance_to_wire(py, &value)?),
                AccountEvent::Order(value) => ("order", order_to_wire(py, &value)?),
                AccountEvent::Reconnected => ("reconnected", py.None()),
                _ => return Err(binding_contract("AccountEvent")),
            };
            stream_event_wire(py, kind, value)
        }
        Err(error) => stream_error_wire(py, &error),
    }
}

fn hyperliquid_market_item_to_wire(
    py: Python<'_>,
    item: Result<HyperliquidMarketEvent>,
) -> PyResult<Py<PyAny>> {
    match item {
        Ok(event) => provider_stream_event_wire(
            py,
            crate::convert::hyperliquid_market_event_to_wire(py, &event)?,
        ),
        Err(error) => stream_error_wire(py, &error),
    }
}

fn hyperliquid_account_item_to_wire(
    py: Python<'_>,
    item: Result<HyperliquidAccountEvent>,
) -> PyResult<Py<PyAny>> {
    match item {
        Ok(event) => provider_stream_event_wire(
            py,
            crate::convert::hyperliquid_account_event_to_wire(py, &event)?,
        ),
        Err(error) => stream_error_wire(py, &error),
    }
}

fn stream_event_wire(py: Python<'_>, kind: &str, value: Py<PyAny>) -> PyResult<Py<PyAny>> {
    let event = PyDict::new(py);
    event.set_item("kind", kind)?;
    event.set_item("value", value)?;
    let item = PyDict::new(py);
    item.set_item("kind", "event")?;
    item.set_item("event", event)?;
    Ok(item.into_any().unbind())
}

fn provider_stream_event_wire(py: Python<'_>, event: Py<PyAny>) -> PyResult<Py<PyAny>> {
    let item = PyDict::new(py);
    item.set_item("kind", "event")?;
    item.set_item("event", event)?;
    Ok(item.into_any().unbind())
}

fn stream_error_wire(py: Python<'_>, error: &maxt::Error) -> PyResult<Py<PyAny>> {
    let item = PyDict::new(py);
    item.set_item("kind", "error")?;
    item.set_item("error", error_to_wire(py, error)?)?;
    Ok(item.into_any().unbind())
}

fn binding_contract(type_name: &str) -> PyErr {
    PyRuntimeError::new_err(format!(
        "maxt binding contract does not map a new {type_name} variant"
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::ffi::CString;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use pyo3::types::{PyDict, PyModule};

    use super::*;

    struct Items<T>(VecDeque<T>);

    impl<T: Unpin> Stream for Items<T> {
        type Item = T;

        fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<T>> {
            Poll::Ready(self.0.pop_front())
        }
    }

    #[test]
    fn rust_market_stream_yields_event_error_then_stop() {
        Python::initialize();
        let stream = MarketStream::new(Items(VecDeque::from([
            Ok(MarketEvent::Reconnected),
            Err(maxt::Error::Decode {
                detail: "bad frame".to_string(),
            }),
        ])));
        let stream = Python::attach(|py| market_stream(py, stream).map(Bound::unbind)).unwrap();

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                let event = anext(&stream).await?;
                Python::attach(|py| -> PyResult<()> {
                    let item = event.bind(py).cast::<PyDict>()?;
                    assert_eq!(
                        item.get_item("kind")?.unwrap().extract::<String>()?,
                        "event"
                    );
                    let event = item.get_item("event")?.unwrap();
                    let event = event.cast::<PyDict>()?;
                    assert_eq!(
                        event.get_item("kind")?.unwrap().extract::<String>()?,
                        "reconnected"
                    );
                    Ok(())
                })?;

                let error = anext(&stream).await?;
                Python::attach(|py| -> PyResult<()> {
                    let item = error.bind(py).cast::<PyDict>()?;
                    assert_eq!(
                        item.get_item("kind")?.unwrap().extract::<String>()?,
                        "error"
                    );
                    let error = item.get_item("error")?.unwrap();
                    let error = error.cast::<PyDict>()?;
                    assert_eq!(
                        error.get_item("detail")?.unwrap().extract::<String>()?,
                        "bad frame"
                    );
                    Ok(())
                })?;

                let error = anext(&stream).await.unwrap_err();
                assert!(Python::attach(
                    |py| error.is_instance_of::<PyStopAsyncIteration>(py)
                ));
                Ok(())
            })
        })
        .unwrap();
    }

    #[test]
    fn native_aclose_drops_the_rust_stream() {
        Python::initialize();
        let dropped = Arc::new(AtomicBool::new(false));
        let stream = MarketStream::new(DropStream(Arc::clone(&dropped)));
        let stream = Python::attach(|py| market_stream(py, stream).map(Bound::unbind)).unwrap();

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                let close = Python::attach(|py| {
                    let awaitable = stream.call_method0(py, "aclose")?;
                    pyo3_async_runtimes::tokio::into_future(awaitable.into_bound(py))
                })?;
                close.await?;
                Ok(())
            })
        })
        .unwrap();
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn native_market_aclose_cancels_a_pending_next() {
        Python::initialize();
        let dropped = Arc::new(AtomicBool::new(false));
        let stream = MarketStream::new(DropStream(Arc::clone(&dropped)));
        let stream = Python::attach(|py| market_stream(py, stream).map(Bound::unbind)).unwrap();

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                let next = call_future(&stream, "__anext__")?;
                asyncio_tick().await?;
                let close = call_future(&stream, "aclose")?;
                tokio::time::timeout(Duration::from_millis(200), close)
                    .await
                    .expect("aclose must not wait behind a pending __anext__")?;

                let error = tokio::time::timeout(Duration::from_millis(200), next)
                    .await
                    .expect("pending __anext__ must wake after aclose")
                    .unwrap_err();
                assert!(Python::attach(
                    |py| error.is_instance_of::<PyStopAsyncIteration>(py)
                ));
                Ok(())
            })
        })
        .unwrap();
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn native_account_aclose_cancels_a_pending_next() {
        Python::initialize();
        let dropped = Arc::new(AtomicBool::new(false));
        let stream = AccountStream::new(AccountDropStream(Arc::clone(&dropped)));
        let stream = Python::attach(|py| account_stream(py, stream).map(Bound::unbind)).unwrap();

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                let next = call_future(&stream, "__anext__")?;
                asyncio_tick().await?;
                let close = call_future(&stream, "aclose")?;
                tokio::time::timeout(Duration::from_millis(200), close)
                    .await
                    .expect("account aclose must not wait behind a pending __anext__")?;

                let error = tokio::time::timeout(Duration::from_millis(200), next)
                    .await
                    .expect("pending account __anext__ must wake after aclose")
                    .unwrap_err();
                assert!(Python::attach(
                    |py| error.is_instance_of::<PyStopAsyncIteration>(py)
                ));
                Ok(())
            })
        })
        .unwrap();
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[test]
    fn dropping_a_pending_python_iterator_cancels_next_before_aclose() {
        Python::initialize();
        let (mut stream, owner) = Python::attach(|py| -> PyResult<_> {
            let module = PyModule::from_code(
                py,
                &CString::new(
                    r#"
import asyncio

class Owner:
    def __init__(self):
        self.started = False
        self.closed = False
        self.close_called = False
        self.close_error = None

class Source:
    def __init__(self, owner):
        self.owner = owner
        self.generator = self.events()

    async def events(self):
        try:
            self.owner.started = True
            await asyncio.Event().wait()
            yield None
        finally:
            await asyncio.sleep(0)
            self.owner.closed = True

    def __aiter__(self):
        return self

    def __anext__(self):
        return self.generator.__anext__()

    async def aclose(self):
        self.owner.close_called = True
        try:
            await self.generator.aclose()
        except BaseException as error:
            self.owner.close_error = str(error)
            raise

OWNER = Owner()
SOURCE = Source(OWNER)
"#,
                )
                .unwrap(),
                &CString::new("test_pending_python_iterator.py").unwrap(),
                &CString::new("test_pending_python_iterator").unwrap(),
            )?;
            Ok((
                market_stream_from_python(&module.getattr("SOURCE")?)?,
                module.getattr("OWNER")?.unbind(),
            ))
        })
        .unwrap();

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                poll_fn(|cx| {
                    assert!(Pin::new(&mut stream).poll_next(cx).is_pending());
                    Poll::Ready(())
                })
                .await;
                asyncio_sleep(0.01).await?;
                assert!(Python::attach(|py| owner
                    .getattr(py, "started")?
                    .extract(py))?);

                drop(stream);
                asyncio_sleep(0.05).await?;

                Python::attach(|py| -> PyResult<()> {
                    assert!(owner.getattr(py, "closed")?.extract::<bool>(py)?);
                    assert!(owner.getattr(py, "close_called")?.extract::<bool>(py)?);
                    assert!(owner.getattr(py, "close_error")?.is_none(py));
                    Ok(())
                })
            })
        })
        .unwrap();
    }

    async fn anext(stream: &Py<PyAny>) -> PyResult<Py<PyAny>> {
        call_future(stream, "__anext__")?.await
    }

    fn call_future(stream: &Py<PyAny>, method: &str) -> PyResult<NextFuture> {
        Python::attach(|py| {
            let awaitable = stream.call_method0(py, method)?;
            pyo3_async_runtimes::tokio::into_future(awaitable.into_bound(py))
                .map(|future| Box::pin(future) as NextFuture)
        })
    }

    async fn asyncio_tick() -> PyResult<()> {
        asyncio_sleep(0.0).await
    }

    async fn asyncio_sleep(seconds: f64) -> PyResult<()> {
        let future = Python::attach(|py| {
            let awaitable = py.import("asyncio")?.call_method1("sleep", (seconds,))?;
            pyo3_async_runtimes::tokio::into_future(awaitable)
        })?;
        future.await.map(|_| ())
    }

    struct DropStream(Arc<AtomicBool>);

    impl Stream for DropStream {
        type Item = Result<MarketEvent>;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Pending
        }
    }

    impl Drop for DropStream {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    struct AccountDropStream(Arc<AtomicBool>);

    impl Stream for AccountDropStream {
        type Item = Result<AccountEvent>;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Pending
        }
    }

    impl Drop for AccountDropStream {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
}
