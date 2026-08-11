use maxt::{
    Balance, CandleRequest, Cursor, Decimal, Feed, HistoryRequest, MarginRequest, Market,
    OrderRequest, OrderType, Overflow, Page, Size, StreamConfig, Subscription, Timestamp,
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

pub(crate) fn u32_from_wire(value: &Bound<'_, PyAny>, field: &str) -> PyResult<u32> {
    let value = value.extract::<u64>().map_err(|_| {
        core_error(maxt::Error::InvalidRequest {
            field: field.to_owned(),
            detail: "must be an unsigned 32-bit integer".to_owned(),
        })
    })?;
    u32::try_from(value).map_err(|_| {
        core_error(maxt::Error::InvalidRequest {
            field: field.to_owned(),
            detail: "must be an unsigned 32-bit integer".to_owned(),
        })
    })
}

fn invalid(field: &str, value: &str) -> PyErr {
    PyValueError::new_err(format!("invalid {field}: {value}"))
}

fn binding_contract(type_name: &str) -> PyErr {
    PyRuntimeError::new_err(format!(
        "maxt binding contract does not map a new {type_name} variant"
    ))
}

#[allow(dead_code, unreachable_patterns)]
mod generated {
    use super::*;

    include!("generated/convert.rs");
}

pub(crate) use generated::*;

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

pub(crate) fn only_fields(dict: &Bound<'_, PyDict>, allowed: &[&str], field: &str) -> PyResult<()> {
    for (key, _) in dict.iter() {
        let key = key.extract::<String>().map_err(|_| {
            core_error(maxt::Error::InvalidRequest {
                field: field.to_owned(),
                detail: "wire object keys must be strings".to_owned(),
            })
        })?;
        if !allowed.contains(&key.as_str()) {
            return Err(core_error(maxt::Error::InvalidRequest {
                field: field.to_owned(),
                detail: format!("does not accept `{key}`"),
            }));
        }
    }
    Ok(())
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

pub(crate) fn balance_from_wire(value: &Bound<'_, PyAny>) -> PyResult<Balance> {
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
    let time_in_force = optional(dict, "time_in_force")?
        .map(|value| time_in_force_from_wire(&value))
        .transpose()?;
    let mut request = match (order_type, price) {
        (OrderType::Market, None) => OrderRequest::market(market, side, size),
        (OrderType::Limit, Some(price)) => OrderRequest::limit(market, side, size, price),
        (OrderType::Best, None) => OrderRequest::best(
            market,
            side,
            size,
            time_in_force
                .ok_or_else(|| PyValueError::new_err("best orders require time_in_force"))?,
        ),
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
    if let Some(time_in_force) = time_in_force {
        request = request.time_in_force(time_in_force);
    }
    if optional(dict, "reduce_only")?
        .map(|value| value.extract::<bool>())
        .transpose()?
        .unwrap_or(false)
    {
        request = request.reduce_only();
    }
    if let Some(client_id) = optional(dict, "client_id")? {
        request = request.client_id(client_id.extract::<String>()?);
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

pub(crate) fn list_from_wire<T>(
    value: &Bound<'_, PyAny>,
    parse: fn(&Bound<'_, PyAny>) -> PyResult<T>,
) -> PyResult<Vec<T>> {
    value
        .try_iter()?
        .map(|item| parse(&item?))
        .collect::<PyResult<Vec<_>>>()
}

pub(crate) fn bithumb_twap_order_page_to_wire(
    py: Python<'_>,
    value: &Page<maxt::BithumbTwapOrder>,
) -> PyResult<Py<PyAny>> {
    wire_dict!(
        py,
        "items" => list_to_wire(py, &value.items, bithumb_twap_order_to_wire)?,
        "next" => value.next.as_ref().map(Cursor::as_str),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use maxt::{Exchange, Feature, Interval};
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
        Python::initialize();
        let values = [
            ("sec1", Interval::Sec1),
            ("min1", Interval::Min1),
            ("min3", Interval::Min3),
            ("min5", Interval::Min5),
            ("min10", Interval::Min10),
            ("min15", Interval::Min15),
            ("min30", Interval::Min30),
            ("hour1", Interval::Hour1),
            ("hour2", Interval::Hour2),
            ("hour4", Interval::Hour4),
            ("hour6", Interval::Hour6),
            ("hour8", Interval::Hour8),
            ("hour12", Interval::Hour12),
            ("day1", Interval::Day1),
            ("day3", Interval::Day3),
            ("week1", Interval::Week1),
            ("month1", Interval::Month1),
        ];

        Python::attach(|py| {
            for (wire, expected) in values {
                assert_eq!(
                    interval_from_wire(PyString::new(py, wire).as_any()).unwrap(),
                    expected
                );
                assert_eq!(interval_to_wire(expected).unwrap(), wire);
            }
            assert!(interval_from_wire(PyString::new(py, "minute").as_any()).is_err());
        });
    }

    #[test]
    fn feature_names_cover_the_exact_public_set() {
        Python::initialize();
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

        Python::attach(|py| {
            for (feature, wire) in values {
                assert_eq!(feature_to_wire(feature).unwrap(), wire);
                assert_eq!(
                    feature_from_wire(PyString::new(py, wire).as_any()).unwrap(),
                    feature
                );
            }
        });
    }

    #[test]
    fn exchange_names_cover_the_exact_public_set() {
        Python::initialize();
        let values = [
            (Exchange::Upbit, "upbit"),
            (Exchange::Bithumb, "bithumb"),
            (Exchange::Binance, "binance"),
            (Exchange::Hyperliquid, "hyperliquid"),
        ];

        Python::attach(|py| {
            for (exchange, wire) in values {
                assert_eq!(exchange_to_wire(exchange).unwrap(), wire);
                assert_eq!(
                    exchange_from_wire(PyString::new(py, wire).as_any()).unwrap(),
                    exchange
                );
            }
        });
    }
}
