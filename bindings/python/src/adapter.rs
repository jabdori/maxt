use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use maxt::{
    AccountEvent, Adapter, Balance, BoxFuture, CancelOrdersRequest, Candle, CandleRequest, Cursor,
    Decimal, Error, Exchange, ExchangeErrorKind, Feature, Feed, FundingPayment, FundingRate,
    HistoryRequest, Interval, Level, MarginMode, MarginRequest, MarginSummary, Market, MarketEvent,
    MarketInfo, MarketKind, MarketStatus, Order, OrderBook, OrderHistoryRequest, OrderRequest,
    OrderStatus, OrderType, Overflow, Page, Position, Result, Side, Size, StreamConfig,
    Subscription, Ticker, TimeInForce, Timestamp, Trade,
};
use maxt_bindings_common::{AdapterCall, AdapterReply, ForeignAdapter, ForeignDispatcher};
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyDict, PyList, PyTracebackMethods, PyTuple};

use crate::convert::{
    decimal_from_wire, decimal_to_wire, exchange_from_wire, feature_from_wire, interval_from_wire,
    list_from_wire, margin_mode_from_wire, market_from_wire, market_to_wire, optional, required,
    side_from_wire, text, timestamp_to_wire, transfer_error_kind_from_wire, wire_object,
};

macro_rules! wire_dict {
    ($py:expr, $($key:literal => $value:expr),* $(,)?) => {{
        let dict = PyDict::new($py);
        $(dict.set_item($key, $value)?;)*
        Ok::<Py<PyAny>, PyErr>(dict.into_any().unbind())
    }};
}

type PythonFuture = Pin<Box<dyn Future<Output = PyResult<Py<PyAny>>> + Send + 'static>>;

pub(crate) fn boxed_adapter(object: Py<PyAny>) -> PyResult<Box<dyn Adapter>> {
    let (exchange, features) = Python::attach(|py| read_metadata(object.bind(py)))?;
    let dispatcher = Arc::new(PythonDispatcher {
        object: Arc::new(object),
    });
    Ok(Box::new(ForeignAdapter::new(
        exchange, features, dispatcher,
    )))
}

#[pymethods]
impl crate::client::NativeClient {
    #[staticmethod]
    fn from_adapter(object: Py<PyAny>) -> PyResult<Self> {
        boxed_adapter(object).map(Self::from_boxed)
    }
}

struct PythonDispatcher {
    object: Arc<Py<PyAny>>,
}

impl ForeignDispatcher for PythonDispatcher {
    fn dispatch(&self, call: AdapterCall) -> BoxFuture<'_, Result<AdapterReply>> {
        let object = Arc::clone(&self.object);
        Box::pin(async move {
            let (reply, future) =
                Python::attach(|py| prepare_call(py, &object, call)).map_err(python_error)?;
            let value = future.await.map_err(python_error)?;
            Python::attach(|py| decode_reply(py, reply, value.bind(py))).map_err(python_error)
        })
    }
}

include!("generated/adapter_dispatch.rs");

fn enum_object(py: Python<'_>, class: &str, value: &str) -> PyResult<Py<PyAny>> {
    py.import("maxt")?
        .getattr(class)?
        .call1((value,))
        .map(Bound::unbind)
}

fn model_object(py: Python<'_>, class: &str, wire: Py<PyAny>) -> PyResult<Py<PyAny>> {
    py.import("maxt")?
        .getattr(class)?
        .call_method1("from_wire", (wire,))
        .map(Bound::unbind)
}

fn market_object(py: Python<'_>, value: &Market) -> PyResult<Py<PyAny>> {
    model_object(py, "Market", market_to_wire(py, value)?)
}

fn optional_market_object(py: Python<'_>, value: Option<&Market>) -> PyResult<Py<PyAny>> {
    value.map_or_else(|| Ok(py.None()), |market| market_object(py, market))
}

fn candle_request_object(py: Python<'_>, value: &CandleRequest) -> PyResult<Py<PyAny>> {
    let wire = wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "interval" => interval_to_wire(value.interval)?,
        "from" => value.from.map(timestamp_to_wire),
        "to" => value.to.map(timestamp_to_wire),
        "limit" => value.limit,
    )?;
    model_object(py, "CandleRequest", wire)
}

fn subscription_object(py: Python<'_>, value: &Subscription) -> PyResult<Py<PyAny>> {
    let markets = value
        .markets()
        .iter()
        .map(|market| market_to_wire(py, market))
        .collect::<PyResult<Vec<_>>>()?;
    let feeds = value
        .feeds()
        .iter()
        .map(|feed| feed_to_wire(py, *feed))
        .collect::<PyResult<Vec<_>>>()?;
    let wire = wire_dict!(
        py,
        "markets" => PyList::new(py, markets)?,
        "feeds" => PyList::new(py, feeds)?,
    )?;
    model_object(py, "Subscription", wire)
}

fn feed_to_wire(py: Python<'_>, value: Feed) -> PyResult<Py<PyAny>> {
    let (kind, interval) = match value {
        Feed::Trades => ("trades", None),
        Feed::OrderBook => ("order_book", None),
        Feed::Ticker => ("ticker", None),
        Feed::Candles(interval) => ("candles", Some(interval_to_wire(interval)?)),
        _ => return Err(binding_contract("Feed")),
    };
    wire_dict!(py, "kind" => kind, "interval" => interval)
}

fn stream_config_object(py: Python<'_>, value: &StreamConfig) -> PyResult<Py<PyAny>> {
    let overflow = match value.overflow {
        Overflow::Backpressure => "backpressure",
        Overflow::DropNewest => "drop_newest",
        _ => return Err(binding_contract("Overflow")),
    };
    let wire = wire_dict!(
        py,
        "max_reconnect_attempts" => value.max_reconnect_attempts,
        "initial_reconnect_delay_ms" => value.initial_reconnect_delay_ms,
        "max_reconnect_delay_ms" => value.max_reconnect_delay_ms,
        "idle_timeout_ms" => value.idle_timeout_ms,
        "buffer_size" => value.buffer_size,
        "overflow" => overflow,
    )?;
    model_object(py, "StreamConfig", wire)
}

fn order_request_object(py: Python<'_>, value: &OrderRequest) -> PyResult<Py<PyAny>> {
    let order_type = match value.order_type {
        OrderType::Market => "market",
        OrderType::Limit => "limit",
        OrderType::Best => "best",
        _ => return Err(binding_contract("OrderType")),
    };
    let (size_kind, size_value) = match value.size {
        Size::Base(amount) => ("base", amount),
        Size::Quote(amount) => ("quote", amount),
        _ => return Err(binding_contract("Size")),
    };
    let time_in_force = value.time_in_force.map(time_in_force_to_wire).transpose()?;
    let size = wire_dict!(
        py,
        "kind" => size_kind,
        "value" => decimal_to_wire(size_value),
    )?;
    let wire = wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "side" => side_to_wire(value.side),
        "order_type" => order_type,
        "size" => size,
        "price" => value.price.map(decimal_to_wire),
        "time_in_force" => time_in_force,
        "reduce_only" => value.reduce_only,
        "client_id" => &value.client_id,
    )?;
    model_object(py, "OrderRequest", wire)
}

fn order_history_request_object(
    py: Python<'_>,
    value: &OrderHistoryRequest,
) -> PyResult<Py<PyAny>> {
    let wire = crate::convert::order_history_request_to_wire(py, value)?;
    model_object(py, "OrderHistoryRequest", wire)
}

fn cancel_orders_request_object(
    py: Python<'_>,
    value: &CancelOrdersRequest,
) -> PyResult<Py<PyAny>> {
    let wire = crate::convert::cancel_orders_request_to_wire(py, value)?;
    model_object(py, "CancelOrdersRequest", wire)
}

fn history_request_object(py: Python<'_>, value: &HistoryRequest) -> PyResult<Py<PyAny>> {
    let wire = wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "from" => value.from.map(timestamp_to_wire),
        "to" => value.to.map(timestamp_to_wire),
        "cursor" => value.cursor.as_ref().map(Cursor::as_str),
        "limit" => value.limit,
    )?;
    model_object(py, "HistoryRequest", wire)
}

fn margin_request_object(py: Python<'_>, value: &MarginRequest) -> PyResult<Py<PyAny>> {
    let margin_mode = value.margin_mode.map(margin_mode_to_wire).transpose()?;
    let wire = wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "leverage" => value.leverage.map(decimal_to_wire),
        "margin_mode" => margin_mode,
    )?;
    model_object(py, "MarginRequest", wire)
}

fn market_kind_to_wire(value: MarketKind) -> PyResult<&'static str> {
    match value {
        MarketKind::Spot => Ok("spot"),
        MarketKind::Perpetual => Ok("perpetual"),
        _ => Err(binding_contract("MarketKind")),
    }
}

fn interval_to_wire(value: Interval) -> PyResult<&'static str> {
    match value {
        Interval::Sec1 => Ok("sec1"),
        Interval::Min1 => Ok("min1"),
        Interval::Min3 => Ok("min3"),
        Interval::Min5 => Ok("min5"),
        Interval::Min10 => Ok("min10"),
        Interval::Min15 => Ok("min15"),
        Interval::Min30 => Ok("min30"),
        Interval::Hour1 => Ok("hour1"),
        Interval::Hour2 => Ok("hour2"),
        Interval::Hour4 => Ok("hour4"),
        Interval::Hour6 => Ok("hour6"),
        Interval::Hour8 => Ok("hour8"),
        Interval::Hour12 => Ok("hour12"),
        Interval::Day1 => Ok("day1"),
        Interval::Day3 => Ok("day3"),
        Interval::Week1 => Ok("week1"),
        Interval::Month1 => Ok("month1"),
        _ => Err(binding_contract("Interval")),
    }
}

fn side_to_wire(value: Side) -> &'static str {
    match value {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}

fn time_in_force_to_wire(value: TimeInForce) -> PyResult<&'static str> {
    match value {
        TimeInForce::GoodTilCancelled => Ok("good_til_cancelled"),
        TimeInForce::ImmediateOrCancel => Ok("immediate_or_cancel"),
        TimeInForce::FillOrKill => Ok("fill_or_kill"),
        TimeInForce::PostOnly => Ok("post_only"),
        _ => Err(binding_contract("TimeInForce")),
    }
}

fn margin_mode_to_wire(value: MarginMode) -> PyResult<&'static str> {
    match value {
        MarginMode::Cross => Ok("cross"),
        MarginMode::Isolated => Ok("isolated"),
        _ => Err(binding_contract("MarginMode")),
    }
}

fn binding_contract(type_name: &str) -> PyErr {
    pyo3::exceptions::PyRuntimeError::new_err(format!(
        "maxt binding contract does not map a new {type_name} variant"
    ))
}

fn decode_reply(
    _py: Python<'_>,
    reply: ReplyKind,
    value: &Bound<'_, PyAny>,
) -> PyResult<AdapterReply> {
    if let Some(result) = decode_generated_reply(reply, value) {
        return result;
    }
    match reply {
        ReplyKind::Markets => {
            list_from_wire(value, market_info_from_wire).map(AdapterReply::Markets)
        }
        ReplyKind::Trades => list_from_wire(value, trade_from_wire).map(AdapterReply::Trades),
        ReplyKind::OrderBook => order_book_from_wire(value).map(AdapterReply::OrderBook),
        ReplyKind::Ticker => ticker_from_wire(value).map(AdapterReply::Ticker),
        ReplyKind::Candles => list_from_wire(value, candle_from_wire).map(AdapterReply::Candles),
        ReplyKind::MarketStream => {
            crate::stream::market_stream_from_python(value).map(AdapterReply::MarketStream)
        }
        ReplyKind::Balances => list_from_wire(value, balance_from_wire).map(AdapterReply::Balances),
        ReplyKind::OpenOrders => {
            list_from_wire(value, order_from_wire).map(AdapterReply::OpenOrders)
        }
        ReplyKind::Order | ReplyKind::OrderByClientId => {
            order_from_wire(value).map(AdapterReply::Order)
        }
        ReplyKind::OrdersByIds => {
            list_from_wire(value, order_from_wire).map(AdapterReply::OrdersByIds)
        }
        ReplyKind::OrderHistory => {
            page_from_wire(value, order_from_wire).map(AdapterReply::OrderHistory)
        }
        ReplyKind::AccountStream => {
            crate::stream::account_stream_from_python(value).map(AdapterReply::AccountStream)
        }
        ReplyKind::PlaceOrder => order_from_wire(value).map(AdapterReply::PlaceOrder),
        ReplyKind::Positions => {
            list_from_wire(value, position_from_wire).map(AdapterReply::Positions)
        }
        ReplyKind::MarginSummary => {
            margin_summary_from_wire(value).map(AdapterReply::MarginSummary)
        }
        ReplyKind::FundingRates => {
            funding_rates_page_from_wire(value).map(AdapterReply::FundingRates)
        }
        ReplyKind::FundingPayments => {
            funding_payments_page_from_wire(value).map(AdapterReply::FundingPayments)
        }
        ReplyKind::Unit if value.is_none() => Ok(AdapterReply::Unit),
        ReplyKind::Unit => Err(pyo3::exceptions::PyTypeError::new_err(
            "unit adapter methods must return None",
        )),
        _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
            "generated adapter reply decoder returned no result",
        )),
    }
}

fn page_from_wire<T>(
    value: &Bound<'_, PyAny>,
    parse: fn(&Bound<'_, PyAny>) -> PyResult<T>,
) -> PyResult<Page<T>> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(Page {
        items: list_from_wire(&required(value, "items")?, parse)?,
        next: optional(value, "next")?
            .map(|value| value.extract::<String>().map(Cursor::new))
            .transpose()?,
    })
}

fn market_info_from_wire(value: &Bound<'_, PyAny>) -> PyResult<MarketInfo> {
    let value = wire_object(value)?;
    let value = value.cast::<pyo3::types::PyDict>()?;
    let status = text(&required(value, "status")?)?;
    Ok(MarketInfo {
        market: market_from_wire(&required(value, "market")?)?,
        native_symbol: required(value, "native_symbol")?.extract()?,
        status: match status.as_str() {
            "active" => MarketStatus::Active,
            "paused" => MarketStatus::Paused,
            "delisted" => MarketStatus::Delisted,
            "unknown" => MarketStatus::Unknown,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "invalid market status: {status}"
                )));
            }
        },
        korean_name: required(value, "korean_name")?.extract()?,
        english_name: required(value, "english_name")?.extract()?,
    })
}

fn trade_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Trade> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(Trade {
        market: market_from_wire(&required(value, "market")?)?,
        timestamp: timestamp_from_wire(&required(value, "timestamp")?)?,
        price: decimal_from_wire(&required(value, "price")?, "price")?,
        quantity: decimal_from_wire(&required(value, "quantity")?, "quantity")?,
        taker_side: side_from_wire(&required(value, "taker_side")?)?,
        id: optional(value, "id")?
            .map(|value| value.extract())
            .transpose()?,
    })
}

fn level_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Level> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(Level {
        price: decimal_from_wire(&required(value, "price")?, "price")?,
        quantity: decimal_from_wire(&required(value, "quantity")?, "quantity")?,
    })
}

fn order_book_from_wire(value: &Bound<'_, PyAny>) -> PyResult<OrderBook> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(OrderBook {
        market: market_from_wire(&required(value, "market")?)?,
        timestamp: timestamp_from_wire(&required(value, "timestamp")?)?,
        bids: list_from_wire(&required(value, "bids")?, level_from_wire)?,
        asks: list_from_wire(&required(value, "asks")?, level_from_wire)?,
    })
}

fn ticker_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Ticker> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(Ticker {
        market: market_from_wire(&required(value, "market")?)?,
        timestamp: timestamp_from_wire(&required(value, "timestamp")?)?,
        last_trade_time: optional_timestamp(value, "last_trade_time")?,
        last_price: decimal_from_wire(&required(value, "last_price")?, "last_price")?,
        change: optional_decimal(value, "change")?,
        change_rate: optional_decimal(value, "change_rate")?,
        high: optional_decimal(value, "high")?,
        low: optional_decimal(value, "low")?,
        volume: optional_decimal(value, "volume")?,
        quote_volume: optional_decimal(value, "quote_volume")?,
    })
}

fn candle_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Candle> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(Candle {
        market: market_from_wire(&required(value, "market")?)?,
        interval: interval_from_wire(&required(value, "interval")?)?,
        open_time: timestamp_from_wire(&required(value, "open_time")?)?,
        open: decimal_from_wire(&required(value, "open")?, "open")?,
        high: decimal_from_wire(&required(value, "high")?, "high")?,
        low: decimal_from_wire(&required(value, "low")?, "low")?,
        close: decimal_from_wire(&required(value, "close")?, "close")?,
        volume: decimal_from_wire(&required(value, "volume")?, "volume")?,
        quote_volume: optional_decimal(value, "quote_volume")?,
        closed: required(value, "closed")?.extract()?,
    })
}

fn balance_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Balance> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(Balance {
        asset: required(value, "asset")?
            .extract::<String>()?
            .to_ascii_uppercase(),
        available: decimal_from_wire(&required(value, "available")?, "available")?,
        locked: decimal_from_wire(&required(value, "locked")?, "locked")?,
    })
}

fn order_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Order> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(Order {
        id: required(value, "id")?.extract()?,
        market: market_from_wire(&required(value, "market")?)?,
        side: side_from_wire(&required(value, "side")?)?,
        status: order_status_from_wire(&required(value, "status")?)?,
        filled_quantity: decimal_from_wire(
            &required(value, "filled_quantity")?,
            "filled_quantity",
        )?,
        remaining_quantity: decimal_from_wire(
            &required(value, "remaining_quantity")?,
            "remaining_quantity",
        )?,
        price: optional_decimal(value, "price")?,
        created_at: optional_timestamp(value, "created_at")?,
    })
}

fn position_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Position> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(Position {
        market: market_from_wire(&required(value, "market")?)?,
        side: optional(value, "side")?
            .map(|value| side_from_wire(&value))
            .transpose()?,
        quantity: decimal_from_wire(&required(value, "quantity")?, "quantity")?,
        entry_price: optional_decimal(value, "entry_price")?,
        mark_price: optional_decimal(value, "mark_price")?,
        notional: optional_decimal(value, "notional")?,
        unrealized_pnl: optional_decimal(value, "unrealized_pnl")?,
        leverage: optional_decimal(value, "leverage")?,
        margin_mode: optional(value, "margin_mode")?
            .map(|value| margin_mode_from_wire(&value))
            .transpose()?,
    })
}

fn margin_summary_from_wire(value: &Bound<'_, PyAny>) -> PyResult<MarginSummary> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(MarginSummary {
        asset: required(value, "asset")?
            .extract::<String>()?
            .to_ascii_uppercase(),
        equity: optional_decimal(value, "equity")?,
        margin_balance: optional_decimal(value, "margin_balance")?,
        available_balance: optional_decimal(value, "available_balance")?,
    })
}

fn funding_rate_from_wire(value: &Bound<'_, PyAny>) -> PyResult<FundingRate> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(FundingRate {
        market: market_from_wire(&required(value, "market")?)?,
        timestamp: timestamp_from_wire(&required(value, "timestamp")?)?,
        rate: decimal_from_wire(&required(value, "rate")?, "rate")?,
        mark_price: optional_decimal(value, "mark_price")?,
    })
}

fn funding_payment_from_wire(value: &Bound<'_, PyAny>) -> PyResult<FundingPayment> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    Ok(FundingPayment {
        market: market_from_wire(&required(value, "market")?)?,
        timestamp: timestamp_from_wire(&required(value, "timestamp")?)?,
        amount: decimal_from_wire(&required(value, "amount")?, "amount")?,
        rate: optional_decimal(value, "rate")?,
        id: optional(value, "id")?
            .map(|value| value.extract())
            .transpose()?,
    })
}

fn funding_rates_page_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Page<FundingRate>> {
    page_from_wire(value, funding_rate_from_wire)
}

fn funding_payments_page_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Page<FundingPayment>> {
    page_from_wire(value, funding_payment_from_wire)
}

fn timestamp_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Timestamp> {
    value.extract::<i64>().map(Timestamp::from_nanos)
}

fn optional_timestamp(dict: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<Timestamp>> {
    optional(dict, name)?
        .map(|value| timestamp_from_wire(&value))
        .transpose()
}

fn optional_decimal(dict: &Bound<'_, PyDict>, name: &str) -> PyResult<Option<Decimal>> {
    optional(dict, name)?
        .map(|value| decimal_from_wire(&value, name))
        .transpose()
}

fn order_status_from_wire(value: &Bound<'_, PyAny>) -> PyResult<OrderStatus> {
    let value = text(value)?;
    match value.as_str() {
        "accepted" => Ok(OrderStatus::Accepted),
        "open" => Ok(OrderStatus::Open),
        "partially_filled" => Ok(OrderStatus::PartiallyFilled),
        "filled" => Ok(OrderStatus::Filled),
        "cancelled" => Ok(OrderStatus::Cancelled),
        "rejected" => Ok(OrderStatus::Rejected),
        "unknown" => Ok(OrderStatus::Unknown),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "invalid order status: {value}"
        ))),
    }
}

pub(crate) fn market_stream_item(value: &Bound<'_, PyAny>) -> Result<MarketEvent> {
    match stream_item_kind(value)? {
        StreamItemKind::Event(event) => {
            market_event_from_wire(event.bind(value.py())).map_err(adapter_contract_error)
        }
        StreamItemKind::Error(error) => Err(error),
    }
}

pub(crate) fn account_stream_item(value: &Bound<'_, PyAny>) -> Result<AccountEvent> {
    match stream_item_kind(value)? {
        StreamItemKind::Event(event) => {
            account_event_from_wire(event.bind(value.py())).map_err(adapter_contract_error)
        }
        StreamItemKind::Error(error) => Err(error),
    }
}

enum StreamItemKind {
    Event(Py<PyAny>),
    Error(Error),
}

fn stream_item_kind(value: &Bound<'_, PyAny>) -> Result<StreamItemKind> {
    let kind = value
        .getattr("kind")
        .and_then(|kind| kind.extract::<String>())
        .map_err(adapter_contract_error)?;
    match kind.as_str() {
        "event" => value
            .getattr("event")
            .map(Bound::unbind)
            .map(StreamItemKind::Event)
            .map_err(adapter_contract_error),
        "error" => {
            let error = value.getattr("error").map_err(adapter_contract_error)?;
            let error = known_error_value(value.py(), &error)
                .map_err(adapter_contract_error)?
                .ok_or_else(|| Error::adapter("StreamError.error must be a maxt.MaxtError"))?;
            Ok(StreamItemKind::Error(error))
        }
        _ => Err(Error::adapter(format!(
            "Python stream item kind must be event or error, got {kind}"
        ))),
    }
}

fn market_event_from_wire(value: &Bound<'_, PyAny>) -> PyResult<MarketEvent> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    let kind = text(&required(value, "kind")?)?;
    match kind.as_str() {
        "trade" => trade_from_wire(&required(value, "value")?).map(MarketEvent::Trade),
        "order_book" => {
            order_book_from_wire(&required(value, "value")?).map(MarketEvent::OrderBook)
        }
        "ticker" => ticker_from_wire(&required(value, "value")?).map(MarketEvent::Ticker),
        "candle" => candle_from_wire(&required(value, "value")?).map(MarketEvent::Candle),
        "reconnected" => Ok(MarketEvent::Reconnected),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "invalid market event kind: {kind}"
        ))),
    }
}

fn account_event_from_wire(value: &Bound<'_, PyAny>) -> PyResult<AccountEvent> {
    let value = wire_object(value)?;
    let value = value.cast::<PyDict>()?;
    let kind = text(&required(value, "kind")?)?;
    match kind.as_str() {
        "balance" => balance_from_wire(&required(value, "value")?).map(AccountEvent::Balance),
        "order" => order_from_wire(&required(value, "value")?).map(AccountEvent::Order),
        "reconnected" => Ok(AccountEvent::Reconnected),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "invalid account event kind: {kind}"
        ))),
    }
}

fn adapter_contract_error(error: PyErr) -> Error {
    Error::adapter(error.to_string())
}

pub(crate) fn python_error(error: PyErr) -> Error {
    Python::attach(|py| {
        known_error_value(py, error.value(py).as_any())
            .ok()
            .flatten()
            .unwrap_or_else(|| Error::adapter(traceback_detail(py, &error)))
    })
}

fn known_error_value(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Option<Error>> {
    let maxt_error = py.import("maxt")?.getattr("MaxtError")?;
    if !value.is_instance(&maxt_error)? {
        return Ok(None);
    }
    let wire = value.call_method0("to_wire")?;
    let wire = wire.cast::<PyDict>()?;
    let kind = text(&required(wire, "kind")?)?;
    let error = match kind.as_str() {
        "invalid_request" => {
            let field = required(wire, "field")?.extract::<String>()?;
            Error::InvalidRequest {
                field,
                detail: required(wire, "detail")?.extract()?,
            }
        }
        "transfer" => Error::Transfer {
            kind: transfer_error_kind_from_wire(&required(wire, "transfer_kind")?)?,
            detail: required(wire, "detail")?.extract()?,
        },
        "unsupported" => Error::Unsupported {
            feature: feature_from_wire(&required(wire, "feature")?)?,
            exchange: exchange_from_wire(&required(wire, "exchange")?)?.id(),
            detail: required(wire, "detail")?.extract()?,
        },
        "adapter" => Error::Adapter {
            detail: required(wire, "detail")?.extract()?,
        },
        "auth" => Error::Auth {
            detail: required(wire, "detail")?.extract()?,
        },
        "exchange" => Error::Exchange {
            exchange: exchange_from_wire(&required(wire, "exchange")?)?.id(),
            code: required(wire, "code")?.extract()?,
            message: required(wire, "message")?.extract()?,
            status: optional(wire, "status")?
                .map(|value| value.extract())
                .transpose()?,
            kind: exchange_error_kind_from_wire(&required(wire, "exchange_kind")?)?,
        },
        "transport" => Error::Transport {
            detail: required(wire, "detail")?.extract()?,
        },
        "decode" => Error::Decode {
            detail: required(wire, "detail")?.extract()?,
        },
        _ => return Ok(None),
    };
    Ok(Some(error))
}

fn exchange_error_kind_from_wire(value: &Bound<'_, PyAny>) -> PyResult<ExchangeErrorKind> {
    let value = text(value)?;
    match value.as_str() {
        "rejected" => Ok(ExchangeErrorKind::Rejected),
        "rate_limited" => Ok(ExchangeErrorKind::RateLimited),
        "unavailable" => Ok(ExchangeErrorKind::Unavailable),
        "unknown" => Ok(ExchangeErrorKind::Unknown),
        _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
            "invalid exchange error kind: {value}"
        ))),
    }
}

fn traceback_detail(py: Python<'_>, error: &PyErr) -> String {
    match error.traceback(py) {
        Some(traceback) => traceback
            .format()
            .map(|traceback| format!("{traceback}{error}"))
            .unwrap_or_else(|_| error.to_string()),
        None => error.to_string(),
    }
}

fn read_metadata(object: &Bound<'_, PyAny>) -> PyResult<(Exchange, Vec<Feature>)> {
    let exchange = exchange_from_wire(&object.getattr("exchange")?)?;

    let mut features = Vec::new();
    for value in object.getattr("features")?.try_iter()? {
        let feature = feature_from_wire(&value?)?;
        if !features.contains(&feature) {
            features.push(feature);
        }
    }
    Ok((exchange, features))
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::future::poll_fn;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use futures_core::Stream;
    use maxt::{
        CandleRequest, HistoryRequest, Interval, MarginMode, MarginRequest, Market, MarketKind,
        OrderRequest, Side, Size, StreamConfig, Subscription,
    };
    use pyo3::exceptions::PyRuntimeError;
    use pyo3::types::PyModule;

    static FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

    fn fixture(code: &str) -> PyResult<Py<PyAny>> {
        Python::initialize();
        Python::attach(|py| {
            py.import("sys")?.getattr("path")?.call_method1(
                "insert",
                (0, concat!(env!("CARGO_MANIFEST_DIR"), "/python")),
            )?;
            let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            let code = CString::new(code).unwrap();
            let file = CString::new(format!("test_adapter_{id}.py")).unwrap();
            let name = CString::new(format!("test_adapter_{id}")).unwrap();
            PyModule::from_code(py, &code, &file, &name)?
                .getattr("Fixture")?
                .call0()
                .map(Bound::unbind)
        })
    }

    #[test]
    fn registration_validates_and_caches_exchange_and_features() {
        let object = fixture(
            r#"
from enum import Enum

class Exchange(Enum):
    UPBIT = "upbit"
    BINANCE = "binance"

class Feature(Enum):
    MARKETS = "markets"
    TICKER = "ticker"

class Fixture:
    def __init__(self):
        self._exchange = Exchange.UPBIT
        self._features = frozenset({Feature.MARKETS, Feature.TICKER})

    @property
    def exchange(self):
        return self._exchange

    @property
    def features(self):
        return self._features
"#,
        )
        .unwrap();
        let probe = Python::attach(|py| object.clone_ref(py));

        let adapter = boxed_adapter(object).unwrap();
        Python::attach(|py| -> PyResult<()> {
            probe.setattr(py, "_exchange", "changed-after-registration")?;
            probe.setattr(py, "_features", ())?;
            Ok(())
        })
        .unwrap();

        assert_eq!(adapter.exchange(), Exchange::Upbit);
        assert!(adapter.supports(Feature::Markets));
        assert!(adapter.supports(Feature::Ticker));
        assert!(!adapter.supports(Feature::Trading));
    }

    #[test]
    fn async_methods_round_trip_through_the_python_adapter() {
        let object = fixture(
            r#"
from enum import Enum

class Exchange(Enum):
    UPBIT = "upbit"

class Feature(Enum):
    MARKETS = "markets"

class Fixture:
    exchange = Exchange.UPBIT
    features = frozenset({Feature.MARKETS})

    async def markets(self, kind):
        assert kind.value == "spot"
        return []
"#,
        )
        .unwrap();
        let adapter = boxed_adapter(object).unwrap();

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                let markets = adapter
                    .markets(MarketKind::Spot)
                    .await
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
                assert!(markets.is_empty());
                Ok(())
            })
        })
        .unwrap();
    }

    #[test]
    fn public_rest_values_round_trip_without_losing_decimals_or_timestamps() {
        let object = fixture(
            r#"
from decimal import Decimal
from maxt import (
    Candle, Exchange, Feature, Interval, Level, Market, OrderBook, Side,
    Ticker, Trade,
)

MARKET = Market.spot(Exchange.UPBIT, "BTC", "KRW")

class Fixture:
    exchange = Exchange.UPBIT
    features = frozenset({
        Feature.TRADES, Feature.ORDER_BOOK, Feature.TICKER, Feature.CANDLES,
    })

    async def trades(self, market, limit):
        assert market == MARKET and limit == 1
        return [Trade(MARKET, 1700000000123456789, Decimal("50000000.01"), Decimal("0.001"), Side.BUY, "t-1")]

    async def order_book(self, market, depth):
        assert market == MARKET and depth == 1
        return OrderBook(MARKET, 1700000000123456790, [Level(Decimal("1"), Decimal("2"))], [Level(Decimal("3"), Decimal("4"))])

    async def ticker(self, market):
        assert market == MARKET
        return Ticker(MARKET, 1700000000123456791, None, Decimal("5"), None, None, None, None, None, None)

    async def candles(self, request):
        assert request.market == MARKET and request.interval is Interval.MIN1
        return [Candle(MARKET, Interval.MIN1, 1700000000123456792, Decimal("1"), Decimal("2"), Decimal("0.5"), Decimal("1.5"), Decimal("10"), None, True)]
"#,
        )
        .unwrap();
        let adapter = boxed_adapter(object).unwrap();
        let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                let trades = adapter
                    .trades(&market, Some(1))
                    .await
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
                assert_eq!(trades[0].timestamp.as_nanos(), 1_700_000_000_123_456_789);
                assert_eq!(trades[0].price.to_string(), "50000000.01");

                let book = adapter
                    .order_book(&market, Some(1))
                    .await
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
                assert_eq!(book.bids[0].quantity, Decimal::from(2));

                let ticker = adapter
                    .ticker(&market)
                    .await
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
                assert_eq!(ticker.last_price, Decimal::from(5));

                let candles = adapter
                    .candles(&CandleRequest::new(market, Interval::Min1))
                    .await
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
                assert!(candles[0].closed);
                Ok(())
            })
        })
        .unwrap();
    }

    #[test]
    fn private_order_margin_and_history_methods_round_trip() {
        let object = fixture(
            r#"
from decimal import Decimal
from maxt import (
    Balance, Cursor, Exchange, Feature, FundingPayment, FundingRate,
    MarginMode, MarginSummary, Market, Order, OrderStatus, Page, Position, Side,
)

MARKET = Market.perpetual(Exchange.HYPERLIQUID, "BTC", "USDC")
ORDER = Order("order-1", MARKET, Side.BUY, OrderStatus.OPEN, Decimal("0"), Decimal("1"), Decimal("30000"), 1700000000000000000)

class Fixture:
    exchange = Exchange.HYPERLIQUID
    features = frozenset({
        Feature.BALANCES, Feature.OPEN_ORDERS, Feature.TRADING, Feature.POSITIONS,
        Feature.MARGIN, Feature.FUNDING_RATES, Feature.FUNDING_PAYMENTS,
        Feature.MARGIN_CONFIG,
    })

    async def balances(self):
        return [Balance("USDC", Decimal("100"), Decimal("5"))]

    async def open_orders(self, market):
        assert market is None
        return [ORDER]

    async def place_order(self, request):
        assert request.market == MARKET and request.size.value == Decimal("1")
        return ORDER

    async def cancel_order(self, market, order_id):
        assert market == MARKET and order_id == "order-1"
        return None

    async def positions(self, market):
        assert market == MARKET
        return [Position(MARKET, Side.BUY, Decimal("1"), leverage=Decimal("3"), margin_mode=MarginMode.CROSS)]

    async def margin_summary(self):
        return MarginSummary("USDC", Decimal("105"), Decimal("20"), Decimal("85"))

    async def funding_rates(self, request):
        assert request.market == MARKET
        return Page([FundingRate(MARKET, 1700000000000000001, Decimal("0.0001"), Decimal("30000"))], Cursor("next-rate"))

    async def funding_payments(self, request):
        assert request.market == MARKET
        return Page([FundingPayment(MARKET, 1700000000000000002, Decimal("-0.25"), Decimal("0.0001"), "payment-1")], None)

    async def set_margin(self, request):
        assert request.market == MARKET and request.leverage == Decimal("3") and request.margin_mode is MarginMode.CROSS
        return None
"#,
        )
        .unwrap();
        let adapter = boxed_adapter(object).unwrap();
        let market = Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC");

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                assert_eq!(
                    adapter.balances().await.map_err(to_runtime)?[0].asset,
                    "USDC"
                );
                assert_eq!(
                    adapter.open_orders(None).await.map_err(to_runtime)?[0].id,
                    "order-1"
                );

                let request =
                    OrderRequest::market(market.clone(), Side::Buy, Size::Base(Decimal::ONE));
                assert_eq!(
                    adapter.place_order(&request).await.map_err(to_runtime)?.id,
                    "order-1"
                );
                adapter
                    .cancel_order(&market, "order-1")
                    .await
                    .map_err(to_runtime)?;

                let positions = adapter.positions(Some(&market)).await.map_err(to_runtime)?;
                assert_eq!(positions[0].leverage, Some(Decimal::from(3)));
                assert_eq!(
                    adapter.margin_summary().await.map_err(to_runtime)?.asset,
                    "USDC"
                );

                let history = HistoryRequest::new(market.clone());
                assert!(
                    adapter
                        .funding_rates(&history)
                        .await
                        .map_err(to_runtime)?
                        .has_more()
                );
                assert_eq!(
                    adapter
                        .funding_payments(&history)
                        .await
                        .map_err(to_runtime)?
                        .items[0]
                        .id
                        .as_deref(),
                    Some("payment-1")
                );

                adapter
                    .set_margin(
                        &MarginRequest::new(market)
                            .leverage(Decimal::from(3))
                            .margin_mode(MarginMode::Cross),
                    )
                    .await
                    .map_err(to_runtime)?;
                Ok(())
            })
        })
        .unwrap();
    }

    #[test]
    fn known_errors_stay_structured_and_unexpected_tracebacks_are_preserved() {
        let object = fixture(
            r#"
from maxt import (
    DecodeError, Exchange, ExchangeError, ExchangeErrorKind, Feature,
    InvalidRequestError,
)

class Fixture:
    exchange = Exchange.UPBIT
    features = frozenset({Feature.MARKETS, Feature.BALANCES, Feature.TRADES, Feature.TICKER})

    async def markets(self, kind):
        raise InvalidRequestError("custom_field", "bad value")

    async def balances(self):
        raise DecodeError("bad frame")

    async def trades(self, market, limit):
        raise ExchangeError(Exchange.UPBIT, "429", "slow down", 429, ExchangeErrorKind.RATE_LIMITED)

    async def ticker(self, market):
        raise RuntimeError("unexpected adapter failure")
"#,
        )
        .unwrap();
        let adapter = boxed_adapter(object).unwrap();
        let market = Market::spot(Exchange::Upbit, "BTC", "KRW");

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                assert!(matches!(
                    adapter.markets(MarketKind::Spot).await.unwrap_err(),
                    Error::InvalidRequest { field, detail }
                        if field == "custom_field" && detail == "bad value"
                ));
                assert!(matches!(
                    adapter.balances().await.unwrap_err(),
                    Error::Decode { detail } if detail == "bad frame"
                ));
                assert!(matches!(
                    adapter.trades(&market, None).await.unwrap_err(),
                    Error::Exchange {
                        status: Some(429),
                        kind: ExchangeErrorKind::RateLimited,
                        ..
                    }
                ));

                let Error::Adapter { detail } = adapter.ticker(&market).await.unwrap_err() else {
                    panic!("unexpected Python exception must become an adapter error");
                };
                assert!(detail.contains("Traceback (most recent call last)"));
                assert!(detail.contains("unexpected adapter failure"));
                Ok(())
            })
        })
        .unwrap();
    }

    #[test]
    fn python_stream_errors_are_items_and_drop_calls_aclose() {
        let object = fixture(
            r#"
from decimal import Decimal
from maxt import (
    DecodeError, Exchange, Feature, Market, MarketEvent, Side, StreamError,
    StreamEvent, Trade,
)

MARKET = Market.spot(Exchange.UPBIT, "BTC", "KRW")
TRADE = Trade(MARKET, 1700000000000000000, Decimal("1"), Decimal("2"), Side.BUY, "trade-1")

class Source:
    def __init__(self, owner):
        self.owner = owner
        self.items = iter([
            StreamEvent(MarketEvent.trade(TRADE)),
            StreamError(DecodeError("bad stream item")),
            StreamEvent(MarketEvent.reconnected()),
        ])

    def __aiter__(self):
        return self

    async def __anext__(self):
        try:
            return next(self.items)
        except StopIteration:
            raise StopAsyncIteration

    async def aclose(self):
        self.owner.closed = True

class Fixture:
    exchange = Exchange.UPBIT
    features = frozenset({Feature.TRADE_STREAM})

    def __init__(self):
        self.closed = False

    async def subscribe(self, subscription, config):
        assert subscription.markets == (MARKET,)
        return Source(self)
"#,
        )
        .unwrap();
        let probe = Python::attach(|py| object.clone_ref(py));
        let adapter = boxed_adapter(object).unwrap();
        let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
        let subscription = Subscription::new().market(market).feed(Feed::Trades);

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                let mut stream = adapter
                    .subscribe(&subscription, &StreamConfig::default())
                    .await
                    .map_err(to_runtime)?;
                assert!(matches!(next_market(&mut stream).await, Some(Ok(MarketEvent::Trade(_)))));
                assert!(matches!(next_market(&mut stream).await, Some(Err(Error::Decode { detail })) if detail == "bad stream item"));
                assert!(matches!(next_market(&mut stream).await, Some(Ok(MarketEvent::Reconnected))));
                assert!(next_market(&mut stream).await.is_none());
                drop(stream);

                let tick = Python::attach(|py| {
                    let sleep = py.import("asyncio")?.call_method1("sleep", (0.01,))?;
                    pyo3_async_runtimes::tokio::into_future(sleep)
                })?;
                tick.await?;
                assert!(Python::attach(|py| {
                    probe.getattr(py, "closed")?.extract::<bool>(py)
                })?);
                Ok(())
            })
        })
        .unwrap();
    }

    #[test]
    fn python_account_stream_values_round_trip() {
        let object = fixture(
            r#"
from decimal import Decimal
from maxt import AccountEvent, AccountStream, Balance, Exchange, Feature, StreamEvent

class Fixture:
    exchange = Exchange.UPBIT
    features = frozenset({Feature.ACCOUNT_STREAM})

    async def subscribe_account(self, config):
        async def events():
            yield StreamEvent(AccountEvent.balance(Balance("KRW", Decimal("10"), Decimal("2"))))
            yield StreamEvent(AccountEvent.reconnected())
        return AccountStream(events())
"#,
        )
        .unwrap();
        let adapter = boxed_adapter(object).unwrap();

        Python::attach(|py| {
            pyo3_async_runtimes::tokio::run(py, async move {
                let mut stream = adapter
                    .subscribe_account(&StreamConfig::default())
                    .await
                    .map_err(to_runtime)?;
                assert!(matches!(
                    next_account(&mut stream).await,
                    Some(Ok(AccountEvent::Balance(balance))) if balance.total() == Decimal::from(12)
                ));
                assert!(matches!(
                    next_account(&mut stream).await,
                    Some(Ok(AccountEvent::Reconnected))
                ));
                assert!(next_account(&mut stream).await.is_none());
                Ok(())
            })
        })
        .unwrap();
    }

    async fn next_market(stream: &mut maxt::MarketStream) -> Option<maxt::Result<MarketEvent>> {
        poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)).await
    }

    async fn next_account(stream: &mut maxt::AccountStream) -> Option<maxt::Result<AccountEvent>> {
        poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)).await
    }

    fn to_runtime(error: Error) -> PyErr {
        PyRuntimeError::new_err(error.to_string())
    }
}
