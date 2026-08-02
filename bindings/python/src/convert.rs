use maxt::{
    Balance, Candle, CandleRequest, Cursor, Decimal, Exchange, ExchangeErrorKind, Feature, Feed,
    FundingPayment, FundingRate, HistoryRequest, Interval, Level, MarginMode, MarginRequest,
    MarginSummary, Market, MarketInfo, MarketKind, MarketStatus, Order, OrderBook, OrderRequest,
    OrderStatus, OrderType, Overflow, Page, Position, Side, Size, StreamConfig, Subscription,
    Ticker, TimeInForce, Timestamp, Trade,
};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

macro_rules! wire_dict {
    ($py:expr, $($key:literal => $value:expr),* $(,)?) => {{
        let dict = PyDict::new($py);
        $(dict.set_item($key, $value)?;)*
        Ok::<Py<PyAny>, PyErr>(dict.into_any().unbind())
    }};
}

pub(crate) fn core_error(error: maxt::Error) -> PyErr {
    let exception = crate::MaxtError::new_err(error.to_string());
    let metadata = Python::attach(|py| {
        let wire = error_to_wire(py, &error)?;
        let value = exception.value(py);
        value.setattr("kind", error_kind(&error))?;
        value.setattr("wire", wire)?;
        Ok::<(), PyErr>(())
    });
    match metadata {
        Ok(()) => exception,
        Err(error) => error,
    }
}

pub(crate) fn decimal_to_wire(value: Decimal) -> String {
    value.to_string()
}

pub(crate) fn timestamp_to_wire(value: Timestamp) -> i64 {
    value.as_nanos()
}

fn invalid(field: &str, value: &str) -> PyErr {
    PyValueError::new_err(format!("invalid {field}: {value}"))
}

fn binding_contract(type_name: &str) -> PyErr {
    PyRuntimeError::new_err(format!(
        "maxt binding contract does not map a new {type_name} variant"
    ))
}

fn error_kind(error: &maxt::Error) -> &'static str {
    match error {
        maxt::Error::InvalidRequest { .. } => "invalid_request",
        maxt::Error::Unsupported { .. } => "unsupported",
        maxt::Error::Adapter { .. } => "adapter",
        maxt::Error::Auth { .. } => "auth",
        maxt::Error::Exchange { .. } => "exchange",
        maxt::Error::Transport { .. } => "transport",
        maxt::Error::Decode { .. } => "decode",
        _ => "binding_contract",
    }
}

fn exchange_error_kind_to_wire(value: ExchangeErrorKind) -> PyResult<&'static str> {
    match value {
        ExchangeErrorKind::Rejected => Ok("rejected"),
        ExchangeErrorKind::RateLimited => Ok("rate_limited"),
        ExchangeErrorKind::Unavailable => Ok("unavailable"),
        ExchangeErrorKind::Unknown => Ok("unknown"),
        _ => Err(binding_contract("ExchangeErrorKind")),
    }
}

pub(crate) fn error_to_wire(py: Python<'_>, error: &maxt::Error) -> PyResult<Py<PyAny>> {
    let message = error.to_string();
    let retryable = error.is_retryable();
    match error {
        maxt::Error::InvalidRequest { field, detail } => wire_dict!(
            py,
            "kind" => "invalid_request",
            "message" => &message,
            "retryable" => retryable,
            "field" => field,
            "detail" => detail,
        ),
        maxt::Error::Unsupported {
            feature,
            exchange,
            detail,
        } => wire_dict!(
            py,
            "kind" => "unsupported",
            "message" => &message,
            "retryable" => retryable,
            "feature" => feature_to_wire(*feature)?,
            "exchange" => exchange,
            "detail" => detail,
        ),
        maxt::Error::Adapter { detail } => wire_dict!(
            py,
            "kind" => "adapter",
            "message" => &message,
            "retryable" => retryable,
            "detail" => detail,
        ),
        maxt::Error::Auth { detail } => wire_dict!(
            py,
            "kind" => "auth",
            "message" => &message,
            "retryable" => retryable,
            "detail" => detail,
        ),
        maxt::Error::Exchange {
            exchange,
            code,
            message: provider_message,
            status,
            kind,
        } => wire_dict!(
            py,
            "kind" => "exchange",
            "message" => &message,
            "retryable" => retryable,
            "exchange" => exchange,
            "code" => code,
            "provider_message" => provider_message,
            "status" => status,
            "exchange_kind" => exchange_error_kind_to_wire(*kind)?,
        ),
        maxt::Error::Transport { detail } => wire_dict!(
            py,
            "kind" => "transport",
            "message" => &message,
            "retryable" => retryable,
            "detail" => detail,
        ),
        maxt::Error::Decode { detail } => wire_dict!(
            py,
            "kind" => "decode",
            "message" => &message,
            "retryable" => retryable,
            "detail" => detail,
        ),
        _ => wire_dict!(
            py,
            "kind" => "binding_contract",
            "message" => "maxt binding contract does not map a new Error variant",
            "retryable" => false,
        ),
    }
}

pub(crate) fn wire_object<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    if value.cast::<PyDict>().is_ok() {
        Ok(value.clone())
    } else {
        value.call_method0("to_wire")
    }
}

pub(crate) fn required<'py>(dict: &Bound<'py, PyDict>, name: &str) -> PyResult<Bound<'py, PyAny>> {
    dict.get_item(name)?
        .ok_or_else(|| PyValueError::new_err(format!("missing wire field: {name}")))
}

pub(crate) fn optional<'py>(
    dict: &Bound<'py, PyDict>,
    name: &str,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    Ok(dict.get_item(name)?.filter(|value| !value.is_none()))
}

pub(crate) fn text(value: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(value) = value.extract::<String>() {
        return Ok(value);
    }
    value.getattr("value")?.extract()
}

pub(crate) fn decimal_from_wire(value: &Bound<'_, PyAny>, field: &str) -> PyResult<Decimal> {
    let value = text(value)?;
    maxt::parse_decimal_exact(&value).map_err(|_| invalid(field, &value))
}

pub(crate) fn exchange_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Exchange> {
    let value = text(value)?;
    exchange_name(&value).ok_or_else(|| invalid("exchange", &value))
}

fn exchange_name(value: &str) -> Option<Exchange> {
    Some(match value {
        "upbit" => Exchange::Upbit,
        "bithumb" => Exchange::Bithumb,
        "binance" => Exchange::Binance,
        "hyperliquid" => Exchange::Hyperliquid,
        _ => return None,
    })
}

pub(crate) fn feature_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Feature> {
    let value = text(value)?;
    feature_name(&value).ok_or_else(|| invalid("feature", &value))
}

fn feature_name(value: &str) -> Option<Feature> {
    Some(match value {
        "markets" => Feature::Markets,
        "trades" => Feature::Trades,
        "order_book" => Feature::OrderBook,
        "ticker" => Feature::Ticker,
        "candles" => Feature::Candles,
        "trade_stream" => Feature::TradeStream,
        "order_book_stream" => Feature::OrderBookStream,
        "ticker_stream" => Feature::TickerStream,
        "candle_stream" => Feature::CandleStream,
        "balances" => Feature::Balances,
        "open_orders" => Feature::OpenOrders,
        "account_stream" => Feature::AccountStream,
        "trading" => Feature::Trading,
        "positions" => Feature::Positions,
        "margin" => Feature::Margin,
        "funding_rates" => Feature::FundingRates,
        "funding_payments" => Feature::FundingPayments,
        "margin_config" => Feature::MarginConfig,
        "reduce_only_orders" => Feature::ReduceOnlyOrders,
        _ => return None,
    })
}

pub(crate) fn feature_to_wire(value: Feature) -> PyResult<&'static str> {
    match value {
        Feature::Markets => Ok("markets"),
        Feature::Trades => Ok("trades"),
        Feature::OrderBook => Ok("order_book"),
        Feature::Ticker => Ok("ticker"),
        Feature::Candles => Ok("candles"),
        Feature::TradeStream => Ok("trade_stream"),
        Feature::OrderBookStream => Ok("order_book_stream"),
        Feature::TickerStream => Ok("ticker_stream"),
        Feature::CandleStream => Ok("candle_stream"),
        Feature::Balances => Ok("balances"),
        Feature::OpenOrders => Ok("open_orders"),
        Feature::AccountStream => Ok("account_stream"),
        Feature::Trading => Ok("trading"),
        Feature::Positions => Ok("positions"),
        Feature::Margin => Ok("margin"),
        Feature::FundingRates => Ok("funding_rates"),
        Feature::FundingPayments => Ok("funding_payments"),
        Feature::MarginConfig => Ok("margin_config"),
        Feature::ReduceOnlyOrders => Ok("reduce_only_orders"),
        _ => Err(binding_contract("Feature")),
    }
}

pub(crate) fn market_kind_from_wire(value: &Bound<'_, PyAny>) -> PyResult<MarketKind> {
    let value = text(value)?;
    market_kind_name(&value).ok_or_else(|| invalid("market kind", &value))
}

fn market_kind_name(value: &str) -> Option<MarketKind> {
    Some(match value {
        "spot" => MarketKind::Spot,
        "perpetual" => MarketKind::Perpetual,
        _ => return None,
    })
}

fn interval_name(value: &str) -> Option<Interval> {
    Some(match value {
        "sec1" => Interval::Sec1,
        "min1" => Interval::Min1,
        "min3" => Interval::Min3,
        "min5" => Interval::Min5,
        "min15" => Interval::Min15,
        "min30" => Interval::Min30,
        "hour1" => Interval::Hour1,
        "hour2" => Interval::Hour2,
        "hour4" => Interval::Hour4,
        "hour8" => Interval::Hour8,
        "hour12" => Interval::Hour12,
        "day1" => Interval::Day1,
        "day3" => Interval::Day3,
        "week1" => Interval::Week1,
        "month1" => Interval::Month1,
        _ => return None,
    })
}

pub(crate) fn interval_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Interval> {
    let value = text(value)?;
    interval_name(&value).ok_or_else(|| invalid("interval", &value))
}

pub(crate) fn side_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Side> {
    let value = text(value)?;
    match value.as_str() {
        "buy" => Ok(Side::Buy),
        "sell" => Ok(Side::Sell),
        _ => Err(invalid("side", &value)),
    }
}

fn order_type_from_wire(value: &Bound<'_, PyAny>) -> PyResult<OrderType> {
    let value = text(value)?;
    match value.as_str() {
        "market" => Ok(OrderType::Market),
        "limit" => Ok(OrderType::Limit),
        _ => Err(invalid("order type", &value)),
    }
}

fn time_in_force_from_wire(value: &Bound<'_, PyAny>) -> PyResult<TimeInForce> {
    let value = text(value)?;
    match value.as_str() {
        "good_til_cancelled" => Ok(TimeInForce::GoodTilCancelled),
        "immediate_or_cancel" => Ok(TimeInForce::ImmediateOrCancel),
        "fill_or_kill" => Ok(TimeInForce::FillOrKill),
        "post_only" => Ok(TimeInForce::PostOnly),
        _ => Err(invalid("time in force", &value)),
    }
}

pub(crate) fn margin_mode_from_wire(value: &Bound<'_, PyAny>) -> PyResult<MarginMode> {
    let value = text(value)?;
    match value.as_str() {
        "cross" => Ok(MarginMode::Cross),
        "isolated" => Ok(MarginMode::Isolated),
        _ => Err(invalid("margin mode", &value)),
    }
}

pub(crate) fn market_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Market> {
    let value = wire_object(value)?;
    let dict = value.cast::<PyDict>()?;
    Ok(Market::new(
        exchange_from_wire(&required(dict, "exchange")?)?,
        market_kind_from_wire(&required(dict, "kind")?)?,
        required(dict, "base")?.extract::<String>()?,
        required(dict, "quote")?.extract::<String>()?,
    ))
}

pub(crate) fn markets_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Vec<Market>> {
    value
        .try_iter()?
        .map(|item| market_from_wire(&item?))
        .collect()
}

fn feed_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Feed> {
    let value = wire_object(value)?;
    let dict = value.cast::<PyDict>()?;
    let kind = text(&required(dict, "kind")?)?;
    match kind.as_str() {
        "trades" => Ok(Feed::Trades),
        "order_book" => Ok(Feed::OrderBook),
        "ticker" => Ok(Feed::Ticker),
        "candles" => Ok(Feed::Candles(interval_from_wire(&required(
            dict, "interval",
        )?)?)),
        _ => Err(invalid("feed", &kind)),
    }
}

pub(crate) fn subscription_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Subscription> {
    let value = wire_object(value)?;
    let dict = value.cast::<PyDict>()?;
    let markets = markets_from_wire(&required(dict, "markets")?)?;
    let feeds = required(dict, "feeds")?
        .try_iter()?
        .map(|item| feed_from_wire(&item?))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(feeds.into_iter().fold(
        Subscription::new().markets_iter(markets),
        Subscription::feed,
    ))
}

pub(crate) fn stream_config_from_wire(value: &Bound<'_, PyAny>) -> PyResult<StreamConfig> {
    let value = wire_object(value)?;
    let dict = value.cast::<PyDict>()?;
    let mut config = StreamConfig::default();
    if let Some(item) = optional(dict, "max_reconnect_attempts")? {
        config.max_reconnect_attempts = Some(item.extract()?);
    } else if dict.contains("max_reconnect_attempts")? {
        config.max_reconnect_attempts = None;
    }
    if let Some(item) = optional(dict, "initial_reconnect_delay_ms")? {
        config.initial_reconnect_delay_ms = item.extract()?;
    }
    if let Some(item) = optional(dict, "max_reconnect_delay_ms")? {
        config.max_reconnect_delay_ms = item.extract()?;
    }
    if let Some(item) = optional(dict, "idle_timeout_ms")? {
        config.idle_timeout_ms = item.extract()?;
    }
    if let Some(item) = optional(dict, "buffer_size")? {
        config.buffer_size = item.extract()?;
    }
    if let Some(item) = optional(dict, "overflow")? {
        let overflow = text(&item)?;
        config.overflow = match overflow.as_str() {
            "backpressure" => Overflow::Backpressure,
            "drop_newest" => Overflow::DropNewest,
            _ => return Err(invalid("overflow", &overflow)),
        };
    }
    Ok(config)
}

pub(crate) fn candle_request_from_wire(value: &Bound<'_, PyAny>) -> PyResult<CandleRequest> {
    let value = wire_object(value)?;
    let dict = value.cast::<PyDict>()?;
    let mut request = CandleRequest::new(
        market_from_wire(&required(dict, "market")?)?,
        interval_from_wire(&required(dict, "interval")?)?,
    );
    if let Some(from) = optional(dict, "from")? {
        request = request.from(Timestamp::from_nanos(from.extract()?));
    }
    if let Some(to) = optional(dict, "to")? {
        request = request.to(Timestamp::from_nanos(to.extract()?));
    }
    if let Some(limit) = optional(dict, "limit")? {
        request = request.limit(limit.extract()?);
    }
    Ok(request)
}

fn size_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Size> {
    let value = wire_object(value)?;
    let dict = value.cast::<PyDict>()?;
    let kind = text(&required(dict, "kind")?)?;
    let amount = decimal_from_wire(&required(dict, "value")?, "size")?;
    match kind.as_str() {
        "base" => Ok(Size::Base(amount)),
        "quote" => Ok(Size::Quote(amount)),
        _ => Err(invalid("size kind", &kind)),
    }
}

pub(crate) fn order_request_from_wire(value: &Bound<'_, PyAny>) -> PyResult<OrderRequest> {
    let value = wire_object(value)?;
    let dict = value.cast::<PyDict>()?;
    let market = market_from_wire(&required(dict, "market")?)?;
    let side = side_from_wire(&required(dict, "side")?)?;
    let size = size_from_wire(&required(dict, "size")?)?;
    let order_type = order_type_from_wire(&required(dict, "order_type")?)?;
    let price = optional(dict, "price")?
        .map(|value| decimal_from_wire(&value, "price"))
        .transpose()?;
    let mut request = match (order_type, price) {
        (OrderType::Market, None) => OrderRequest::market(market, side, size),
        (OrderType::Limit, Some(price)) => OrderRequest::limit(market, side, size, price),
        (OrderType::Market, Some(_)) => {
            return Err(PyValueError::new_err(
                "market orders must not include price",
            ));
        }
        (OrderType::Limit, None) => {
            return Err(PyValueError::new_err("limit orders require price"));
        }
        _ => return Err(PyValueError::new_err("unsupported order type")),
    };
    if let Some(time_in_force) = optional(dict, "time_in_force")? {
        request = request.time_in_force(time_in_force_from_wire(&time_in_force)?);
    }
    if optional(dict, "reduce_only")?
        .map(|value| value.extract::<bool>())
        .transpose()?
        .unwrap_or(false)
    {
        request = request.reduce_only();
    }
    Ok(request)
}

pub(crate) fn history_request_from_wire(value: &Bound<'_, PyAny>) -> PyResult<HistoryRequest> {
    let value = wire_object(value)?;
    let dict = value.cast::<PyDict>()?;
    let mut request = HistoryRequest::new(market_from_wire(&required(dict, "market")?)?);
    if let Some(from) = optional(dict, "from")? {
        request = request.from(Timestamp::from_nanos(from.extract()?));
    }
    if let Some(to) = optional(dict, "to")? {
        request = request.to(Timestamp::from_nanos(to.extract()?));
    }
    if let Some(cursor) = optional(dict, "cursor")? {
        request = request.cursor(Cursor::new(cursor.extract::<String>()?));
    }
    if let Some(limit) = optional(dict, "limit")? {
        request = request.limit(limit.extract()?);
    }
    Ok(request)
}

pub(crate) fn margin_request_from_wire(value: &Bound<'_, PyAny>) -> PyResult<MarginRequest> {
    let value = wire_object(value)?;
    let dict = value.cast::<PyDict>()?;
    let mut request = MarginRequest::new(market_from_wire(&required(dict, "market")?)?);
    if let Some(leverage) = optional(dict, "leverage")? {
        request = request.leverage(decimal_from_wire(&leverage, "leverage")?);
    }
    if let Some(mode) = optional(dict, "margin_mode")? {
        request = request.margin_mode(margin_mode_from_wire(&mode)?);
    }
    Ok(request)
}

pub(crate) fn exchange_to_wire(value: Exchange) -> PyResult<&'static str> {
    match value {
        Exchange::Upbit => Ok("upbit"),
        Exchange::Bithumb => Ok("bithumb"),
        Exchange::Binance => Ok("binance"),
        Exchange::Hyperliquid => Ok("hyperliquid"),
        _ => Err(binding_contract("Exchange")),
    }
}

fn market_kind_to_wire(value: MarketKind) -> PyResult<&'static str> {
    match value {
        MarketKind::Spot => Ok("spot"),
        MarketKind::Perpetual => Ok("perpetual"),
        _ => Err(binding_contract("MarketKind")),
    }
}

fn side_to_wire(value: Side) -> &'static str {
    match value {
        Side::Buy => "buy",
        Side::Sell => "sell",
    }
}

fn interval_to_wire(value: Interval) -> PyResult<&'static str> {
    match value {
        Interval::Sec1 => Ok("sec1"),
        Interval::Min1 => Ok("min1"),
        Interval::Min3 => Ok("min3"),
        Interval::Min5 => Ok("min5"),
        Interval::Min15 => Ok("min15"),
        Interval::Min30 => Ok("min30"),
        Interval::Hour1 => Ok("hour1"),
        Interval::Hour2 => Ok("hour2"),
        Interval::Hour4 => Ok("hour4"),
        Interval::Hour8 => Ok("hour8"),
        Interval::Hour12 => Ok("hour12"),
        Interval::Day1 => Ok("day1"),
        Interval::Day3 => Ok("day3"),
        Interval::Week1 => Ok("week1"),
        Interval::Month1 => Ok("month1"),
        _ => Err(binding_contract("Interval")),
    }
}

fn market_status_to_wire(value: MarketStatus) -> PyResult<&'static str> {
    match value {
        MarketStatus::Active => Ok("active"),
        MarketStatus::Paused => Ok("paused"),
        MarketStatus::Delisted => Ok("delisted"),
        MarketStatus::Unknown => Ok("unknown"),
        _ => Err(binding_contract("MarketStatus")),
    }
}

fn order_status_to_wire(value: OrderStatus) -> PyResult<&'static str> {
    match value {
        OrderStatus::Accepted => Ok("accepted"),
        OrderStatus::Open => Ok("open"),
        OrderStatus::PartiallyFilled => Ok("partially_filled"),
        OrderStatus::Filled => Ok("filled"),
        OrderStatus::Cancelled => Ok("cancelled"),
        OrderStatus::Rejected => Ok("rejected"),
        OrderStatus::Unknown => Ok("unknown"),
        _ => Err(binding_contract("OrderStatus")),
    }
}

fn margin_mode_to_wire(value: MarginMode) -> PyResult<&'static str> {
    match value {
        MarginMode::Cross => Ok("cross"),
        MarginMode::Isolated => Ok("isolated"),
        _ => Err(binding_contract("MarginMode")),
    }
}

pub(crate) fn market_to_wire(py: Python<'_>, value: &Market) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "exchange" => exchange_to_wire(value.exchange)?,
        "kind" => market_kind_to_wire(value.kind)?,
        "base" => &value.base,
        "quote" => &value.quote,
    )
}

pub(crate) fn market_info_to_wire(py: Python<'_>, value: &MarketInfo) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "native_symbol" => &value.native_symbol,
        "status" => market_status_to_wire(value.status)?,
        "korean_name" => &value.korean_name,
        "english_name" => &value.english_name,
    )
}

pub(crate) fn trade_to_wire(py: Python<'_>, value: &Trade) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "timestamp" => timestamp_to_wire(value.timestamp),
        "price" => decimal_to_wire(value.price),
        "quantity" => decimal_to_wire(value.quantity),
        "taker_side" => side_to_wire(value.taker_side),
        "id" => &value.id,
    )
}

fn level_to_wire(py: Python<'_>, value: &Level) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "price" => decimal_to_wire(value.price),
        "quantity" => decimal_to_wire(value.quantity),
    )
}

pub(crate) fn order_book_to_wire(py: Python<'_>, value: &OrderBook) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "timestamp" => timestamp_to_wire(value.timestamp),
        "bids" => list_to_wire(py, &value.bids, level_to_wire)?,
        "asks" => list_to_wire(py, &value.asks, level_to_wire)?,
    )
}

pub(crate) fn ticker_to_wire(py: Python<'_>, value: &Ticker) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "timestamp" => timestamp_to_wire(value.timestamp),
        "last_trade_time" => value.last_trade_time.map(timestamp_to_wire),
        "last_price" => decimal_to_wire(value.last_price),
        "change" => value.change.map(decimal_to_wire),
        "change_rate" => value.change_rate.map(decimal_to_wire),
        "high" => value.high.map(decimal_to_wire),
        "low" => value.low.map(decimal_to_wire),
        "volume" => value.volume.map(decimal_to_wire),
        "quote_volume" => value.quote_volume.map(decimal_to_wire),
    )
}

pub(crate) fn candle_to_wire(py: Python<'_>, value: &Candle) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "interval" => interval_to_wire(value.interval)?,
        "open_time" => timestamp_to_wire(value.open_time),
        "open" => decimal_to_wire(value.open),
        "high" => decimal_to_wire(value.high),
        "low" => decimal_to_wire(value.low),
        "close" => decimal_to_wire(value.close),
        "volume" => decimal_to_wire(value.volume),
        "quote_volume" => value.quote_volume.map(decimal_to_wire),
        "closed" => value.closed,
    )
}

pub(crate) fn balance_to_wire(py: Python<'_>, value: &Balance) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "asset" => &value.asset,
        "available" => decimal_to_wire(value.available),
        "locked" => decimal_to_wire(value.locked),
    )
}

pub(crate) fn order_to_wire(py: Python<'_>, value: &Order) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "id" => &value.id,
        "market" => market_to_wire(py, &value.market)?,
        "side" => side_to_wire(value.side),
        "status" => order_status_to_wire(value.status)?,
        "filled_quantity" => decimal_to_wire(value.filled_quantity),
        "remaining_quantity" => decimal_to_wire(value.remaining_quantity),
        "price" => value.price.map(decimal_to_wire),
        "created_at" => value.created_at.map(timestamp_to_wire),
    )
}

pub(crate) fn position_to_wire(py: Python<'_>, value: &Position) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "side" => value.side.map(side_to_wire),
        "quantity" => decimal_to_wire(value.quantity),
        "entry_price" => value.entry_price.map(decimal_to_wire),
        "mark_price" => value.mark_price.map(decimal_to_wire),
        "notional" => value.notional.map(decimal_to_wire),
        "unrealized_pnl" => value.unrealized_pnl.map(decimal_to_wire),
        "leverage" => value.leverage.map(decimal_to_wire),
        "margin_mode" => value.margin_mode.map(margin_mode_to_wire).transpose()?,
    )
}

pub(crate) fn margin_summary_to_wire(py: Python<'_>, value: &MarginSummary) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "asset" => &value.asset,
        "equity" => value.equity.map(decimal_to_wire),
        "margin_balance" => value.margin_balance.map(decimal_to_wire),
        "available_balance" => value.available_balance.map(decimal_to_wire),
    )
}

pub(crate) fn funding_rate_to_wire(py: Python<'_>, value: &FundingRate) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "timestamp" => timestamp_to_wire(value.timestamp),
        "rate" => decimal_to_wire(value.rate),
        "mark_price" => value.mark_price.map(decimal_to_wire),
    )
}

pub(crate) fn funding_payment_to_wire(
    py: Python<'_>,
    value: &FundingPayment,
) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "market" => market_to_wire(py, &value.market)?,
        "timestamp" => timestamp_to_wire(value.timestamp),
        "amount" => decimal_to_wire(value.amount),
        "rate" => value.rate.map(decimal_to_wire),
        "id" => &value.id,
    )
}

pub(crate) fn funding_rates_page_to_wire(
    py: Python<'_>,
    value: &Page<FundingRate>,
) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "items" => list_to_wire(py, &value.items, funding_rate_to_wire)?,
        "next" => value.next.as_ref().map(Cursor::as_str),
    )
}

pub(crate) fn funding_payments_page_to_wire(
    py: Python<'_>,
    value: &Page<FundingPayment>,
) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "items" => list_to_wire(py, &value.items, funding_payment_to_wire)?,
        "next" => value.next.as_ref().map(Cursor::as_str),
    )
}

pub(crate) fn list_to_wire<T>(
    py: Python<'_>,
    values: &[T],
    convert: fn(Python<'_>, &T) -> PyResult<Py<PyAny>>,
) -> PyResult<Py<PyAny>> {
    let values = values
        .iter()
        .map(|value| convert(py, value))
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyList::new(py, values)?.into_any().unbind())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyString;

    #[test]
    fn decimal_input은_표현할_수_없는_값을_반올림하지_않는다() {
        Python::initialize();
        Python::attach(|py| {
            let rejected = [
                "2.5e-28",
                "0.00000000000000000000000000001",
                "79228162514264337593543950335.4",
            ]
            .map(|text| {
                let value = PyString::new(py, text).into_any();
                decimal_from_wire(&value, "price").is_err()
            });

            assert_eq!(rejected, [true, true, true]);
            let value = PyString::new(py, "8.428e-05").into_any();
            assert_eq!(
                decimal_from_wire(&value, "price").unwrap().to_string(),
                "0.00008428",
            );
        });
    }

    #[test]
    fn decimal_wire_values_preserve_every_digit() {
        let value = Decimal::from_str_exact("12345678901234567890.12345678").unwrap();

        assert_eq!(decimal_to_wire(value), "12345678901234567890.12345678");
    }

    #[test]
    fn decimal_wire_values_preserve_scale() {
        assert_eq!(decimal_to_wire(Decimal::new(12_300, 4)), "1.2300");
    }

    #[test]
    fn timestamp_wire_values_are_epoch_nanoseconds() {
        let value = Timestamp::from_nanos(1_700_000_000_123_456_789);

        assert_eq!(timestamp_to_wire(value), 1_700_000_000_123_456_789);
    }

    #[test]
    fn native_errors_keep_their_variant_metadata() {
        Python::initialize();
        let error = core_error(maxt::Error::adapter("dispatcher returned the wrong reply"));

        Python::attach(|py| {
            let value = error.value(py);
            assert_eq!(
                value.getattr("kind").unwrap().extract::<String>().unwrap(),
                "adapter"
            );
            let wire = value.getattr("wire").unwrap();
            let wire = wire.cast::<PyDict>().unwrap();
            assert_eq!(
                required(wire, "detail")
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "dispatcher returned the wrong reply"
            );
            assert!(
                !required(wire, "retryable")
                    .unwrap()
                    .extract::<bool>()
                    .unwrap()
            );
        });
    }

    #[test]
    fn interval_names_cover_every_core_variant() {
        let values = [
            ("sec1", Interval::Sec1),
            ("min1", Interval::Min1),
            ("min3", Interval::Min3),
            ("min5", Interval::Min5),
            ("min15", Interval::Min15),
            ("min30", Interval::Min30),
            ("hour1", Interval::Hour1),
            ("hour2", Interval::Hour2),
            ("hour4", Interval::Hour4),
            ("hour8", Interval::Hour8),
            ("hour12", Interval::Hour12),
            ("day1", Interval::Day1),
            ("day3", Interval::Day3),
            ("week1", Interval::Week1),
            ("month1", Interval::Month1),
        ];

        for (wire, expected) in values {
            assert_eq!(interval_name(wire), Some(expected));
            assert_eq!(interval_to_wire(expected).unwrap(), wire);
        }
        assert_eq!(interval_name("minute"), None);
    }

    #[test]
    fn feature_names_cover_the_exact_public_set() {
        let values = [
            (Feature::Markets, "markets"),
            (Feature::Trades, "trades"),
            (Feature::OrderBook, "order_book"),
            (Feature::Ticker, "ticker"),
            (Feature::Candles, "candles"),
            (Feature::TradeStream, "trade_stream"),
            (Feature::OrderBookStream, "order_book_stream"),
            (Feature::TickerStream, "ticker_stream"),
            (Feature::CandleStream, "candle_stream"),
            (Feature::Balances, "balances"),
            (Feature::OpenOrders, "open_orders"),
            (Feature::AccountStream, "account_stream"),
            (Feature::Trading, "trading"),
            (Feature::Positions, "positions"),
            (Feature::Margin, "margin"),
            (Feature::FundingRates, "funding_rates"),
            (Feature::FundingPayments, "funding_payments"),
            (Feature::MarginConfig, "margin_config"),
            (Feature::ReduceOnlyOrders, "reduce_only_orders"),
        ];

        for (feature, wire) in values {
            assert_eq!(feature_to_wire(feature).unwrap(), wire);
            assert_eq!(feature_name(wire), Some(feature));
        }
    }

    #[test]
    fn exchange_names_cover_the_exact_public_set() {
        let values = [
            (Exchange::Upbit, "upbit"),
            (Exchange::Bithumb, "bithumb"),
            (Exchange::Binance, "binance"),
            (Exchange::Hyperliquid, "hyperliquid"),
        ];

        for (exchange, wire) in values {
            assert_eq!(exchange_to_wire(exchange).unwrap(), wire);
            assert_eq!(exchange_name(wire), Some(exchange));
        }
    }
}
