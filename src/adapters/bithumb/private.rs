//! Bithumb authenticated REST requests and JWT signing.
//!
//! Private requests use an HS256 JWT. Parameterized requests also include a
//! SHA-512 query hash.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256, Sha512};

use crate::error::{Error, Result};
use crate::request::{
    CancelOrdersRequest, OrderHistoryRequest, OrderIdKind, OrderLookupRequest, OrderRequest,
};
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{
    Balance, CancelOrdersResult, CancelledOrder, Cursor, Market, Order, OrderCancelFailure,
    OrderRules, OrderStatus, OrderType, Page, Side, Size, Timestamp,
};

use super::BithumbCredentials;
use super::parse::{self, EXCHANGE};
use super::rest;

/// JWT claims sent to Bithumb; query fields are omitted for parameterless calls.
#[derive(Debug, Serialize)]
struct Claims<'a> {
    access_key: &'a str,
    nonce: &'a str,
    timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_hash_alg: Option<&'static str>,
}

/// The `Authorization` value for one request.
pub(crate) fn authorization(credentials: &BithumbCredentials, query: &str) -> Result<String> {
    authorization_with(
        credentials,
        query,
        &uuid::Uuid::new_v4().to_string(),
        Timestamp::now().as_millis(),
    )
}

/// Signs with an explicit nonce and timestamp for deterministic tests.
fn authorization_with(
    credentials: &BithumbCredentials,
    query: &str,
    nonce: &str,
    timestamp_ms: i64,
) -> Result<String> {
    if credentials.access_key.trim().is_empty() || credentials.secret_key.trim().is_empty() {
        return Err(Error::auth(
            "bithumb needs both an access key and a secret key",
        ));
    }

    let query_hash = (!query.is_empty()).then(|| sha512_hex(query.as_bytes()));
    let claims = Claims {
        access_key: &credentials.access_key,
        nonce,
        timestamp: timestamp_ms,
        query_hash_alg: query_hash.as_ref().map(|_| "SHA512"),
        query_hash,
    };

    let token = encode_hs256(&claims, credentials.secret_key.as_bytes())?;

    Ok(format!("Bearer {token}"))
}

fn encode_hs256(claims: &impl Serialize, secret: &[u8]) -> Result<String> {
    let header = URL_SAFE_NO_PAD.encode(br#"{"typ":"JWT","alg":"HS256"}"#);
    let payload =
        URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(claims).map_err(|err| {
                Error::auth(format!("could not sign the Bithumb request: {err}"))
            })?);
    let message = format!("{header}.{payload}");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)
        .map_err(|err| Error::auth(format!("could not sign the Bithumb request: {err}")))?;
    mac.update(message.as_bytes());

    Ok(format!(
        "{message}.{}",
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    ))
}

fn sha512_hex(bytes: &[u8]) -> String {
    hex::encode(Sha512::digest(bytes))
}

/// Builds the exact validated query string covered by the signature.
fn signed_query(params: &[(&str, String)]) -> Result<String> {
    for (name, value) in params {
        signed_value(name, value)?;
    }

    Ok(params
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&"))
}

/// Rejects values that could alter the signed query structure.
fn signed_value(name: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(Error::invalid_request(name, format!("`{name}` is empty")));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~'))
    {
        return Err(Error::invalid_request(
            name,
            format!("`{name}` holds a character that would change the signed request: `{value}`"),
        ));
    }
    Ok(())
}

fn signed_get(
    credentials: &BithumbCredentials,
    path: &str,
    params: &[(&str, String)],
) -> Result<HttpRequest> {
    let signed = signed_query(params)?;

    Ok(HttpRequest::get(path)
        .query(encoded_query(params))
        .header("authorization", authorization(credentials, &signed)?))
}

fn encoded_query(params: &[(&str, String)]) -> String {
    params
        .iter()
        .map(|(key, value)| format!("{key}={}", encoded_value(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn encoded_value(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[(byte >> 4) as usize]));
            encoded.push(char::from(HEX[(byte & 0x0f) as usize]));
        }
    }
    encoded
}

pub(crate) fn balances_request(credentials: &BithumbCredentials) -> Result<HttpRequest> {
    signed_get(credentials, "/v1/accounts", &[])
}

pub(crate) fn order_rules_request(
    credentials: &BithumbCredentials,
    market: &Market,
) -> Result<HttpRequest> {
    signed_get(
        credentials,
        "/v1/orders/chance",
        &[("market", parse::native_symbol(market)?)],
    )
}

pub(crate) fn open_orders_request(
    credentials: &BithumbCredentials,
    market: Option<&Market>,
) -> Result<HttpRequest> {
    let mut params = Vec::new();
    if let Some(market) = market {
        params.push(("market", parse::native_symbol(market)?));
    }
    // `wait` is Bithumb's resting-order state.
    params.push(("state", "wait".to_string()));

    signed_get(credentials, "/v1/orders", &params)
}

pub(crate) fn order_request(
    credentials: &BithumbCredentials,
    market: &Market,
    order_id: &str,
) -> Result<HttpRequest> {
    order_request_by(credentials, market, "uuid", "order_id", order_id)
}

pub(crate) fn order_by_client_id_request(
    credentials: &BithumbCredentials,
    market: &Market,
    client_id: &str,
) -> Result<HttpRequest> {
    validate_client_order_id(client_id)?;
    order_request_by(
        credentials,
        market,
        "client_order_id",
        "client_id",
        client_id,
    )
}

pub(crate) fn orders_by_ids_request(
    credentials: &BithumbCredentials,
    request: &OrderLookupRequest,
) -> Result<HttpRequest> {
    crate::adapters::validate_order_lookup(request)?;
    let mut signed_params = Vec::new();
    let mut body = serde_json::Map::new();
    if let Some(market) = &request.market {
        let market = parse::native_symbol(market)?;
        signed_params.push(("market", market.clone()));
        body.insert("market".to_string(), Value::String(market));
    }
    let (body_key, signed_key) = match request.kind {
        OrderIdKind::Exchange => ("order_ids", "order_ids[]"),
        OrderIdKind::Client => {
            for id in &request.ids {
                validate_client_order_id(id)?;
            }
            ("client_order_ids", "client_order_ids[]")
        }
    };
    signed_params.extend(request.ids.iter().cloned().map(|id| (signed_key, id)));
    signed_params.push(("order_by", "desc".to_string()));
    body.insert(
        body_key.to_string(),
        Value::Array(request.ids.iter().cloned().map(Value::String).collect()),
    );
    body.insert("order_by".to_string(), Value::String("desc".to_string()));

    let query = signed_query(&signed_params)?;
    let body = serde_json::to_string(&Value::Object(body))
        .map_err(|err| Error::decode(format!("could not build the Bithumb lookup body: {err}")))?;
    Ok(HttpRequest::post("/v2/orders/search")
        .json_body(body)
        .header("authorization", authorization(credentials, &query)?))
}

fn order_request_by(
    credentials: &BithumbCredentials,
    market: &Market,
    parameter: &'static str,
    field: &'static str,
    value: &str,
) -> Result<HttpRequest> {
    parse::native_symbol(market)?;
    if value.trim().is_empty() {
        return Err(Error::invalid_request(field, "must not be empty"));
    }
    signed_get(credentials, "/v1/order", &[(parameter, value.to_string())])
}

pub(crate) fn order_history_request(
    credentials: &BithumbCredentials,
    request: &OrderHistoryRequest,
) -> Result<HttpRequest> {
    let (limit, from, to) = crate::adapters::order_history_window(request)?;
    let mut params = Vec::new();
    if let Some(market) = &request.market {
        params.push(("market", parse::native_symbol(market)?));
    }
    if let Some(state) = crate::adapters::final_order_state(&request.statuses)? {
        params.push(("state", state.to_string()));
    }
    if let Some(from) = from {
        params.push(("start_time", from.to_string()));
    }
    if let Some(to) = to {
        params.push(("end_time", to.to_string()));
    }
    params.push(("limit", limit.to_string()));
    params.push(("order_by", "desc".to_string()));
    let mut signed = signed_query(&params)?;
    if let Some(cursor) = &request.cursor {
        let cursor = cursor.as_str();
        if cursor.is_empty()
            || !cursor.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=' | b'-' | b'_')
            })
        {
            return Err(Error::invalid_request(
                "cursor",
                "not a Bithumb order-history cursor",
            ));
        }
        signed.push_str("&next_key=");
        signed.push_str(cursor);
        params.push(("next_key", cursor.to_string()));
    }

    Ok(HttpRequest::get("/v2/orders/history")
        .query(encoded_query(&params))
        .header("authorization", authorization(credentials, &signed)?))
}

pub(crate) fn cancel_order_request(
    credentials: &BithumbCredentials,
    order_id: &str,
) -> Result<HttpRequest> {
    cancel_order_request_by(credentials, "order_id", order_id)
}

pub(crate) fn cancel_order_by_client_id_request(
    credentials: &BithumbCredentials,
    client_id: &str,
) -> Result<HttpRequest> {
    validate_client_order_id(client_id)?;
    cancel_order_request_by(credentials, "client_order_id", client_id)
}

pub(crate) fn cancel_orders_request(
    credentials: &BithumbCredentials,
    request: &CancelOrdersRequest,
) -> Result<HttpRequest> {
    crate::adapters::validate_cancel_order_limit(request, 30)?;
    let (body_key, signed_key) = match request.kind {
        OrderIdKind::Exchange => ("order_ids", "order_ids[]"),
        OrderIdKind::Client => {
            for id in &request.ids {
                validate_client_order_id(id)?;
            }
            ("client_order_ids", "client_order_ids[]")
        }
    };
    let signed = request
        .ids
        .iter()
        .cloned()
        .map(|id| (signed_key, id))
        .collect::<Vec<_>>();
    let query = signed_query(&signed)?;
    let body = serde_json::to_string(&serde_json::json!({ body_key: request.ids }))
        .map_err(|err| Error::decode(format!("could not build the Bithumb cancel body: {err}")))?;

    Ok(HttpRequest::post("/v2/orders/cancel")
        .json_body(body)
        .header("authorization", authorization(credentials, &query)?))
}

fn cancel_order_request_by(
    credentials: &BithumbCredentials,
    parameter: &str,
    value: &str,
) -> Result<HttpRequest> {
    let query = signed_query(&[(parameter, value.to_string())])?;

    Ok(HttpRequest::delete("/v2/order")
        .query(query.clone())
        .header("authorization", authorization(credentials, &query)?))
}

/// Request data needed to interpret Bithumb's minimal order acknowledgement.
pub(crate) struct PlacedOrder {
    pub(crate) params: Vec<(&'static str, String)>,
    /// Remaining base quantity; zero when a quote-sized request has no base amount.
    pub(crate) remaining_quantity: Decimal,
}

/// Validates an order and maps it to `/v2/orders` parameters.
pub(crate) fn placed_order(request: &OrderRequest) -> Result<PlacedOrder> {
    if request.reduce_only {
        return Err(Error::unsupported(
            crate::feature::Feature::ReduceOnlyOrders,
            EXCHANGE,
            "bithumb lists spot markets only, which have no position to reduce",
        ));
    }
    let mut params = vec![
        ("market", parse::native_symbol(&request.market)?),
        (
            "side",
            match request.side {
                Side::Buy => "bid",
                Side::Sell => "ask",
            }
            .to_string(),
        ),
    ];

    let remaining_quantity = match (&request.order_type, &request.size, request.side) {
        (OrderType::Limit, Size::Base(quantity), _) => {
            let price = request.price.ok_or_else(|| {
                Error::invalid_request("price", "a limit order needs a limit price")
            })?;
            params.push(("order_type", "limit".to_string()));
            params.push(("price", amount("price", price)?));
            params.push(("volume", amount("volume", *quantity)?));
            *quantity
        }
        // Bithumb encodes a market-buy quote amount as `price`.
        (OrderType::Market, Size::Quote(amount_to_spend), Side::Buy) => {
            params.push(("order_type", "price".to_string()));
            params.push(("price", amount("price", *amount_to_spend)?));
            Decimal::ZERO
        }
        (OrderType::Market, Size::Base(quantity), Side::Sell) => {
            params.push(("order_type", "market".to_string()));
            params.push(("volume", amount("volume", *quantity)?));
            *quantity
        }
        (OrderType::Best, Size::Quote(amount_to_spend), Side::Buy) => {
            ensure_krw_best_order(&request.market)?;
            if request.price.is_some() {
                return Err(Error::invalid_request(
                    "price",
                    "a bithumb best buy takes its quote amount from `size`, not `price`",
                ));
            }
            params.push(("order_type", "best".to_string()));
            params.push(("price", amount("price", *amount_to_spend)?));
            Decimal::ZERO
        }
        (OrderType::Best, Size::Base(quantity), Side::Sell) => {
            ensure_krw_best_order(&request.market)?;
            if request.price.is_some() {
                return Err(Error::invalid_request(
                    "price",
                    "a bithumb best sell has no caller-selected price",
                ));
            }
            params.push(("order_type", "best".to_string()));
            params.push(("volume", amount("volume", *quantity)?));
            *quantity
        }
        (OrderType::Market, Size::Base(_), Side::Buy) => {
            return Err(Error::invalid_request(
                "size",
                "bithumb sizes a market buy in the quote asset; use `Size::Quote`",
            ));
        }
        (OrderType::Market, Size::Quote(_), Side::Sell) => {
            return Err(Error::invalid_request(
                "size",
                "bithumb sizes a market sell in the base asset; use `Size::Base`",
            ));
        }
        (OrderType::Limit, Size::Quote(_), _) => {
            return Err(Error::invalid_request(
                "size",
                "bithumb sizes a limit order in the base asset; use `Size::Base`",
            ));
        }
        (OrderType::Best, Size::Base(_), Side::Buy) => {
            return Err(Error::invalid_request(
                "size",
                "bithumb sizes a best buy in the quote asset; use `Size::Quote`",
            ));
        }
        (OrderType::Best, Size::Quote(_), Side::Sell) => {
            return Err(Error::invalid_request(
                "size",
                "bithumb sizes a best sell in the base asset; use `Size::Base`",
            ));
        }
    };

    if let Some(time_in_force) = bithumb_time_in_force(&request.order_type, request.time_in_force)?
    {
        if request.market.quote != "KRW" {
            return Err(Error::unsupported(
                crate::feature::Feature::Trading,
                EXCHANGE,
                "bithumb currently supports time-in-force only on KRW markets",
            ));
        }
        params.push(("time_in_force", time_in_force.to_string()));
    }
    if let Some(client_id) = &request.client_id {
        validate_client_order_id(client_id)?;
        params.push(("client_order_id", client_id.clone()));
    }

    Ok(PlacedOrder {
        params,
        remaining_quantity,
    })
}

fn ensure_krw_best_order(market: &Market) -> Result<()> {
    if market.quote == "KRW" {
        return Ok(());
    }
    Err(Error::unsupported(
        crate::feature::Feature::Trading,
        EXCHANGE,
        "bithumb currently supports best orders only on KRW markets",
    ))
}

fn bithumb_time_in_force(
    order_type: &OrderType,
    value: Option<crate::types::TimeInForce>,
) -> Result<Option<&'static str>> {
    use crate::types::TimeInForce;

    Ok(match (order_type, value) {
        (OrderType::Limit, None | Some(TimeInForce::GoodTilCancelled)) => None,
        (OrderType::Limit, Some(TimeInForce::ImmediateOrCancel)) => Some("ioc"),
        (OrderType::Limit, Some(TimeInForce::FillOrKill)) => Some("fok"),
        (OrderType::Limit, Some(TimeInForce::PostOnly)) => Some("post_only"),
        (OrderType::Best, Some(TimeInForce::ImmediateOrCancel)) => Some("ioc"),
        (OrderType::Best, Some(TimeInForce::FillOrKill)) => Some("fok"),
        (OrderType::Best, other) => {
            return Err(Error::invalid_request(
                "time_in_force",
                format!(
                    "a bithumb best order requires immediate-or-cancel or fill-or-kill, not {other:?}"
                ),
            ));
        }
        (OrderType::Market, None) => None,
        (OrderType::Market, Some(other)) => {
            return Err(Error::invalid_request(
                "time_in_force",
                format!("a bithumb market order has no configurable time-in-force, not {other:?}"),
            ));
        }
    })
}

fn validate_client_order_id(value: &str) -> Result<()> {
    if (1..=36).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Ok(());
    }
    Err(Error::invalid_request(
        "client_id",
        "a bithumb client order id must be 1-36 ASCII letters, digits, '-' or '_'",
    ))
}

/// Validates a positive amount and preserves its decimal spelling.
fn amount(field: &'static str, value: Decimal) -> Result<String> {
    if value <= Decimal::ZERO {
        return Err(Error::invalid_request(
            field,
            format!("must be greater than zero, not {value}"),
        ));
    }
    Ok(value.to_string())
}

pub(crate) fn place_order_request(
    credentials: &BithumbCredentials,
    placed: &PlacedOrder,
) -> Result<HttpRequest> {
    // The JWT hashes these body fields in query-string form.
    let query = signed_query(&placed.params)?;
    let body: serde_json::Map<String, Value> = placed
        .params
        .iter()
        .map(|(key, value)| ((*key).to_string(), Value::String(value.clone())))
        .collect();
    let body = serde_json::to_string(&Value::Object(body))
        .map_err(|err| Error::decode(format!("could not build the Bithumb order body: {err}")))?;

    Ok(HttpRequest::post("/v2/orders")
        .json_body(body)
        .header("authorization", authorization(credentials, &query)?))
}

pub(crate) async fn balances(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
) -> Result<Vec<Balance>> {
    parse::balances(&rest::send(http, &balances_request(credentials)?).await?)
}

pub(crate) async fn order_rules(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    market: &Market,
) -> Result<OrderRules> {
    let native_symbol = parse::native_symbol(market)?;
    let body = rest::send(http, &order_rules_request(credentials, market)?).await?;
    crate::adapters::order_rules::parse(&body, market, &native_symbol)
}

pub(crate) async fn open_orders(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    market: Option<&Market>,
) -> Result<Vec<Order>> {
    parse::orders(&rest::send(http, &open_orders_request(credentials, market)?).await?)
}

pub(crate) async fn order(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    market: &Market,
    order_id: &str,
) -> Result<Order> {
    let body = rest::send(http, &order_request(credentials, market, order_id)?).await?;
    checked_market(parse::order(&body)?, market)
}

pub(crate) async fn order_by_client_id(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    market: &Market,
    client_id: &str,
) -> Result<Order> {
    let body = rest::send(
        http,
        &order_by_client_id_request(credentials, market, client_id)?,
    )
    .await?;
    checked_market(parse::order(&body)?, market)
}

pub(crate) async fn orders_by_ids(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &OrderLookupRequest,
) -> Result<Vec<Order>> {
    let body = rest::send(http, &orders_by_ids_request(credentials, request)?).await?;
    parse::orders(&body)
}

pub(crate) async fn order_history(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &OrderHistoryRequest,
) -> Result<Page<Order>> {
    let body = rest::send(http, &order_history_request(credentials, request)?).await?;
    order_history_page(&body)
}

fn order_history_page(body: &Value) -> Result<Page<Order>> {
    let data = body
        .get("data")
        .ok_or_else(|| Error::decode("bithumb order history carries no `data`"))?;
    let next = match body.get("next_key") {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) if !value.is_empty() => Some(Cursor::new(value.clone())),
        Some(_) => {
            return Err(Error::decode(
                "bithumb order history `next_key` is not a non-empty string or null",
            ));
        }
    };
    Ok(Page {
        items: parse::orders(data)?,
        next,
    })
}

fn checked_market(order: Order, expected: &Market) -> Result<Order> {
    if order.market == *expected {
        Ok(order)
    } else {
        Err(Error::invalid_request(
            "market",
            format!(
                "the requested identifier belongs to {}, not {expected}",
                order.market
            ),
        ))
    }
}

pub(crate) async fn place_order(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &OrderRequest,
) -> Result<Order> {
    let placed = placed_order(request)?;
    let body = rest::send(http, &place_order_request(credentials, &placed)?).await?;

    parse::order_ack(
        &body,
        request.market.clone(),
        request.side,
        // The acknowledgement contains no fill state.
        OrderStatus::Accepted,
        placed.remaining_quantity,
        request.price,
    )
}

pub(crate) async fn cancel_order(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    market: &Market,
    order_id: &str,
) -> Result<()> {
    // Validate the caller's market even though Bithumb cancels by identifier.
    parse::native_symbol(market)?;
    let body = rest::send(http, &cancel_order_request(credentials, order_id)?).await?;

    cancel_ack(&body)
}

pub(crate) async fn cancel_order_by_client_id(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    market: &Market,
    client_id: &str,
) -> Result<()> {
    parse::native_symbol(market)?;
    let body = rest::send(
        http,
        &cancel_order_by_client_id_request(credentials, client_id)?,
    )
    .await?;

    cancel_ack(&body)
}

pub(crate) async fn cancel_orders(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &CancelOrdersRequest,
) -> Result<CancelOrdersResult> {
    let body = rest::send(http, &cancel_orders_request(credentials, request)?).await?;
    cancel_orders_result(&body)
}

fn cancel_orders_result(body: &Value) -> Result<CancelOrdersResult> {
    let success = batch_array(body, "success")?
        .iter()
        .map(|entry| {
            let order_id = batch_text(entry, "order_id")?.ok_or_else(|| {
                Error::decode("bithumb batch-cancel success carries no `order_id`")
            })?;
            let cancelled_at = batch_text(entry, "created_at")?
                .map(|raw| {
                    parse::offset_time(&raw).ok_or_else(|| {
                        Error::decode(format!(
                            "bithumb batch-cancel `created_at` is not RFC 3339: `{raw}`"
                        ))
                    })
                })
                .transpose()?;
            Ok(CancelledOrder {
                order_id,
                client_id: batch_text(entry, "client_order_id")?,
                market: None,
                cancelled_at,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let failed = batch_array(body, "fail")?
        .iter()
        .map(|entry| {
            let error = entry
                .get("error")
                .ok_or_else(|| Error::decode("bithumb batch-cancel failure carries no `error`"))?;
            Ok(OrderCancelFailure {
                order_id: batch_text(entry, "order_id")?,
                client_id: batch_text(entry, "client_order_id")?,
                market: None,
                code: batch_text(error, "name")?,
                message: batch_text(error, "message")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(CancelOrdersResult {
        cancelled: success,
        failed,
    })
}

fn batch_array<'a>(body: &'a Value, field: &str) -> Result<&'a [Value]> {
    body.get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| {
            Error::decode(format!(
                "bithumb batch-cancel response has no `{field}` array"
            ))
        })
}

fn batch_text(value: &Value, field: &str) -> Result<Option<String>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(Error::decode(format!(
            "bithumb batch-cancel `{field}` is not text"
        ))),
    }
}

fn cancel_ack(body: &Value) -> Result<()> {
    match body.get("order_id").and_then(Value::as_str) {
        Some(order_id) if !order_id.is_empty() => Ok(()),
        _ => Err(Error::decode(
            "bithumb cancel response carries no `order_id`",
        )),
    }
}

/// Builds a parameterless authorization header for a private WebSocket handshake.
pub(crate) fn websocket_authorization(credentials: &BithumbCredentials) -> Result<String> {
    authorization(credentials, "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, TimeInForce};

    fn credentials() -> BithumbCredentials {
        BithumbCredentials {
            access_key: "test-access".to_string(),
            secret_key: "test-secret".to_string(),
        }
    }

    fn btc_krw() -> Market {
        Market::spot(Exchange::Bithumb, "BTC", "KRW")
    }

    fn payload(token: &str) -> Value {
        let encoded = token
            .trim_start_matches("Bearer ")
            .split('.')
            .nth(1)
            .expect("a JWT has three segments");
        let bytes = base64_url(encoded);

        serde_json::from_slice(&bytes).expect("the payload is JSON")
    }

    fn base64_url(encoded: &str) -> Vec<u8> {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(encoded)
            .expect("a JWT payload is base64url")
    }

    #[test]
    fn a_parameterised_request_is_signed_over_the_sha512_of_its_query() {
        // The hash covers the exact query string.
        assert_eq!(
            sha512_hex(b"market=KRW-BTC&limit=10"),
            "d8214a07d0b7181ac91485f885d4349e9de6733bbd0806fec3102519a0ba1479\
             b9be54245055706da413a6e916a8a978c1fc1a79e8e459d54c4de8fbe2bc70cd"
        );
    }

    #[test]
    fn one_fixed_key_nonce_and_clock_always_produce_the_same_token() {
        let token = authorization_with(
            &credentials(),
            "market=KRW-BTC&limit=10",
            "nonce-2",
            1_717_000_000_456,
        )
        .expect("the credentials are complete");

        assert_eq!(
            token,
            "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.\
             eyJhY2Nlc3Nfa2V5IjoidGVzdC1hY2Nlc3MiLCJub25jZSI6Im5vbmNlLTIiLCJ0aW1lc3RhbXAiOjE3MTcwMDAwMDA0NTYsInF1ZXJ5X2hhc2giOiJkODIxNGEwN2QwYjcxODFhYzkxNDg1Zjg4NWQ0MzQ5ZTlkZTY3MzNiYmQwODA2ZmVjMzEwMjUxOWEwYmExNDc5YjliZTU0MjQ1MDU1NzA2ZGE0MTNhNmU5MTZhOGE5NzhjMWZjMWE3OWU4ZTQ1OWQ1NGM0ZGU4ZmJlMmJjNzBjZCIsInF1ZXJ5X2hhc2hfYWxnIjoiU0hBNTEyIn0.\
             WaCH4TpRIigfnZ2bK0k4YouWz4RVtPXb9oiKWcw8RCM"
        );
    }

    #[test]
    fn a_request_without_parameters_names_no_query_hash() {
        let token = authorization_with(&credentials(), "", "nonce-1", 1_717_000_000_123)
            .expect("the credentials are complete");
        let claims = payload(&token);

        assert_eq!(claims["access_key"], "test-access");
        assert_eq!(claims["nonce"], "nonce-1");
        assert_eq!(claims["timestamp"], 1_717_000_000_123_i64);
        // Parameterless tokens omit both query claims.
        assert!(claims.get("query_hash").is_none());
        assert!(claims.get("query_hash_alg").is_none());
    }

    #[test]
    fn a_parameterised_request_names_the_algorithm_alongside_the_hash() {
        let token = authorization_with(
            &credentials(),
            "market=KRW-BTC&state=wait",
            "nonce-3",
            1_717_000_000_789,
        )
        .expect("the credentials are complete");
        let claims = payload(&token);

        assert_eq!(
            claims["query_hash"],
            sha512_hex(b"market=KRW-BTC&state=wait")
        );
        assert_eq!(claims["query_hash_alg"], "SHA512");
    }

    #[test]
    fn an_empty_credential_never_signs_anything() {
        let blank = BithumbCredentials {
            access_key: "  ".to_string(),
            secret_key: "secret".to_string(),
        };

        assert!(matches!(
            authorization_with(&blank, "", "nonce", 1),
            Err(Error::Auth { .. })
        ));
    }

    #[test]
    fn private_requests_target_the_documented_paths() {
        // Shape reference: https://apidocs.bithumb.com/reference/전체-자산-조회.md
        assert_eq!(
            balances_request(&credentials()).expect("signed").target(),
            "/v1/accounts"
        );
        assert_eq!(
            order_rules_request(&credentials(), &btc_krw())
                .expect("signed")
                .target(),
            "/v1/orders/chance?market=KRW-BTC"
        );
        // Shape reference: https://apidocs.bithumb.com/reference/대기-주문-목록-조회.md
        assert_eq!(
            open_orders_request(&credentials(), Some(&btc_krw()))
                .expect("signed")
                .target(),
            "/v1/orders?market=KRW-BTC&state=wait"
        );
        assert_eq!(
            open_orders_request(&credentials(), None)
                .expect("signed")
                .target(),
            "/v1/orders?state=wait"
        );
        // Shape reference: https://apidocs.bithumb.com/reference/주문-취소-접수.md
        assert_eq!(
            cancel_order_request(&credentials(), "C0101000000001818113")
                .expect("signed")
                .target(),
            "/v2/order?order_id=C0101000000001818113"
        );
        assert_eq!(
            cancel_order_by_client_id_request(&credentials(), "client-1")
                .expect("signed")
                .target(),
            "/v2/order?client_order_id=client-1"
        );
        assert_eq!(
            order_request(&credentials(), &btc_krw(), "C0101000000001818113")
                .expect("signed")
                .target(),
            "/v1/order?uuid=C0101000000001818113"
        );
        assert_eq!(
            order_by_client_id_request(&credentials(), &btc_krw(), "client-1")
                .expect("signed")
                .target(),
            "/v1/order?client_order_id=client-1"
        );
    }

    #[test]
    fn order_rules_hash_the_exact_market_query() {
        let request = order_rules_request(&credentials(), &btc_krw()).expect("a signed request");
        let authorization = request
            .headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value)
            .expect("an authorization header");

        assert_eq!(
            payload(authorization)["query_hash"],
            sha512_hex(b"market=KRW-BTC")
        );
    }

    #[test]
    fn multiple_order_lookup_hashes_array_items_with_bracketed_keys() {
        let lookup = OrderLookupRequest::client(["client-1", "client-2"]).market(btc_krw());
        let request = orders_by_ids_request(&credentials(), &lookup).expect("a signed lookup");
        let body: Value = serde_json::from_str(request.body.as_deref().expect("a JSON body"))
            .expect("valid JSON");
        let authorization = request
            .headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value)
            .expect("an authorization header");

        assert_eq!(request.target(), "/v2/orders/search");
        assert_eq!(
            body,
            serde_json::json!({
                "market": "KRW-BTC",
                "client_order_ids": ["client-1", "client-2"],
                "order_by": "desc"
            })
        );
        assert_eq!(
            payload(authorization)["query_hash"],
            sha512_hex(
                b"market=KRW-BTC&client_order_ids[]=client-1&client_order_ids[]=client-2&order_by=desc"
            )
        );
    }

    #[test]
    fn batch_cancellation_hashes_the_same_array_sent_in_the_json_body() {
        let request = CancelOrdersRequest::client(["client-1", "client-2"]);
        let request =
            cancel_orders_request(&credentials(), &request).expect("a signed cancellation");
        let body: Value = serde_json::from_str(request.body.as_deref().expect("a JSON body"))
            .expect("valid JSON");
        let authorization = request
            .headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value)
            .expect("an authorization header");

        assert_eq!(request.target(), "/v2/orders/cancel");
        assert_eq!(
            body,
            serde_json::json!({"client_order_ids": ["client-1", "client-2"]})
        );
        assert_eq!(
            payload(authorization)["query_hash"],
            sha512_hex(b"client_order_ids[]=client-1&client_order_ids[]=client-2")
        );

        let oversized = CancelOrdersRequest::exchange((0..31).map(|index| index.to_string()));
        assert!(cancel_orders_request(&credentials(), &oversized).is_err());
    }

    #[test]
    fn batch_cancellation_keeps_each_partial_failure() {
        let result = cancel_orders_result(&serde_json::json!({
            "success": [{
                "order_id": "done-1",
                "client_order_id": "client-1",
                "created_at": "2026-02-10T13:56:38+09:00"
            }],
            "fail": [{
                "client_order_id": "missing-1",
                "error": {"name": "order_not_found", "message": "not found"}
            }]
        }))
        .expect("a batch result");

        assert_eq!(result.cancelled[0].order_id, "done-1");
        assert_eq!(
            result.cancelled[0].cancelled_at,
            Some(Timestamp::from_secs(1_770_699_398))
        );
        assert_eq!(result.failed[0].order_id, None);
        assert_eq!(result.failed[0].client_id.as_deref(), Some("missing-1"));
        assert_eq!(result.failed[0].code.as_deref(), Some("order_not_found"));
    }

    #[test]
    fn final_order_history_encodes_the_opaque_cursor_but_hashes_its_raw_value() {
        let history = OrderHistoryRequest::new()
            .market(btc_krw())
            .status(OrderStatus::Cancelled)
            .from(Timestamp::from_millis(1_700_000_000_000))
            .to(Timestamp::from_millis(1_700_001_000_000))
            .cursor(Cursor::new("page+/=="))
            .limit(25);
        let request = order_history_request(&credentials(), &history).expect("signed history");

        assert_eq!(
            request.target(),
            "/v2/orders/history?market=KRW-BTC&state=cancel&start_time=1700000000000&end_time=1700000999999&limit=25&order_by=desc&next_key=page%2B%2F%3D%3D"
        );
        let authorization = request
            .headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value)
            .expect("an authorization header");
        assert_eq!(
            payload(authorization)["query_hash"],
            sha512_hex(
                b"market=KRW-BTC&state=cancel&start_time=1700000000000&end_time=1700000999999&limit=25&order_by=desc&next_key=page+/=="
            )
        );
    }

    #[test]
    fn bithumb_history_decodes_data_and_next_key() {
        let page = order_history_page(&serde_json::json!({
            "data": [{
                "order_id": "C0101000007410714100",
                "side": "bid",
                "state": "done",
                "market": "KRW-BTC",
                "price": "50000000",
                "remaining_volume": "0",
                "executed_volume": "0.001",
                "created_at": "2026-04-09T10:00:00.123+09:00"
            }],
            "next_key": "page+/=="
        }))
        .expect("a v2.1.5 history page");

        assert_eq!(page.items[0].id, "C0101000007410714100");
        assert_eq!(page.next.expect("another page").as_str(), "page+/==");
    }

    #[test]
    fn every_private_request_carries_a_bearer_token() {
        let request = balances_request(&credentials()).expect("signed");

        assert!(
            request
                .headers
                .iter()
                .any(|(name, value)| { name == "authorization" && value.starts_with("Bearer ") })
        );
    }

    #[test]
    fn an_order_id_that_could_smuggle_a_parameter_never_gets_signed() {
        for order_id in ["", "id&market=KRW-ETH", "id=1", "id/../../v1/orders"] {
            assert!(
                matches!(
                    cancel_order_request(&credentials(), order_id),
                    Err(Error::InvalidRequest { field, .. }) if field == "order_id"
                ),
                "{order_id}"
            );
        }
    }

    #[test]
    fn each_order_shape_bithumb_accepts_maps_to_its_own_parameters() {
        let limit = placed_order(&OrderRequest::limit(
            btc_krw(),
            Side::Sell,
            Size::Base(Decimal::new(1, 2)),
            Decimal::from(100_000_000),
        ))
        .expect("a limit sell");
        let market_buy = placed_order(&OrderRequest::market(
            btc_krw(),
            Side::Buy,
            Size::Quote(Decimal::from(10_000)),
        ))
        .expect("a market buy");
        let market_sell = placed_order(&OrderRequest::market(
            btc_krw(),
            Side::Sell,
            Size::Base(Decimal::new(1, 2)),
        ))
        .expect("a market sell");
        let best_buy = placed_order(
            &OrderRequest::best(
                btc_krw(),
                Side::Buy,
                Size::Quote(Decimal::from(10_000)),
                TimeInForce::ImmediateOrCancel,
            )
            .client_id("client-1"),
        )
        .expect("a best buy");

        assert_eq!(
            signed_query(&limit.params).expect("signable"),
            "market=KRW-BTC&side=ask&order_type=limit&price=100000000&volume=0.01"
        );
        assert_eq!(
            signed_query(&market_buy.params).expect("signable"),
            "market=KRW-BTC&side=bid&order_type=price&price=10000"
        );
        assert_eq!(
            signed_query(&market_sell.params).expect("signable"),
            "market=KRW-BTC&side=ask&order_type=market&volume=0.01"
        );
        assert_eq!(
            signed_query(&best_buy.params).expect("signable"),
            "market=KRW-BTC&side=bid&order_type=best&price=10000&time_in_force=ioc&client_order_id=client-1"
        );
        // A quote-sized market buy has no known base remainder.
        assert_eq!(market_buy.remaining_quantity, Decimal::ZERO);
        assert_eq!(limit.remaining_quantity, Decimal::new(1, 2));
    }

    #[test]
    fn a_size_bithumb_cannot_express_for_that_side_is_refused() {
        let market_buy_in_base =
            OrderRequest::market(btc_krw(), Side::Buy, Size::Base(Decimal::ONE));
        let market_sell_in_quote =
            OrderRequest::market(btc_krw(), Side::Sell, Size::Quote(Decimal::ONE));
        let limit_in_quote = OrderRequest::limit(
            btc_krw(),
            Side::Buy,
            Size::Quote(Decimal::ONE),
            Decimal::ONE,
        );

        for request in [market_buy_in_base, market_sell_in_quote, limit_in_quote] {
            assert!(matches!(
                placed_order(&request),
                Err(Error::InvalidRequest { field, .. }) if field == "size"
            ));
        }
    }

    #[test]
    fn a_spot_exchange_refuses_a_reduce_only_order_rather_than_dropping_the_flag() {
        let request =
            OrderRequest::market(btc_krw(), Side::Sell, Size::Base(Decimal::ONE)).reduce_only();

        assert!(matches!(
            placed_order(&request),
            Err(Error::Unsupported {
                feature: crate::feature::Feature::ReduceOnlyOrders,
                exchange: "bithumb",
                ..
            })
        ));
    }

    #[test]
    fn time_in_force_is_sent_only_for_supported_order_shapes() {
        let limit =
            OrderRequest::limit(btc_krw(), Side::Buy, Size::Base(Decimal::ONE), Decimal::ONE)
                .time_in_force(TimeInForce::PostOnly);
        let invalid_best = OrderRequest::best(
            btc_krw(),
            Side::Buy,
            Size::Quote(Decimal::ONE),
            TimeInForce::PostOnly,
        );
        let non_krw = OrderRequest::limit(
            Market::spot(Exchange::Bithumb, "BTC", "USDT"),
            Side::Buy,
            Size::Base(Decimal::ONE),
            Decimal::ONE,
        )
        .time_in_force(TimeInForce::ImmediateOrCancel);

        assert!(
            signed_query(&placed_order(&limit).expect("post-only limit").params)
                .expect("signable")
                .ends_with("time_in_force=post_only")
        );
        assert!(matches!(
            placed_order(&invalid_best),
            Err(Error::InvalidRequest { field, .. }) if field == "time_in_force"
        ));
        assert!(matches!(
            placed_order(&non_krw),
            Err(Error::Unsupported { .. })
        ));
    }

    #[test]
    fn a_cancel_acknowledgement_never_becomes_a_synthetic_order() {
        assert_eq!(
            cancel_ack(&serde_json::json!({"order_id": "order-1"})),
            Ok(())
        );
        assert!(matches!(
            cancel_ack(&serde_json::json!({"client_order_id": "client-1"})),
            Err(Error::Decode { .. })
        ));
    }

    #[test]
    fn a_price_or_quantity_that_is_not_positive_never_reaches_the_exchange() {
        let free = OrderRequest::limit(
            btc_krw(),
            Side::Buy,
            Size::Base(Decimal::ONE),
            Decimal::ZERO,
        );
        let empty = OrderRequest::market(btc_krw(), Side::Sell, Size::Base(Decimal::ZERO));

        assert!(matches!(
            placed_order(&free),
            Err(Error::InvalidRequest { field, .. }) if field == "price"
        ));
        assert!(matches!(
            placed_order(&empty),
            Err(Error::InvalidRequest { field, .. }) if field == "volume"
        ));
    }

    #[test]
    fn the_order_body_carries_the_same_fields_the_signature_covers() {
        let placed = placed_order(&OrderRequest::limit(
            btc_krw(),
            Side::Buy,
            Size::Base(Decimal::new(1, 2)),
            Decimal::from(100_000_000),
        ))
        .expect("a limit buy");
        let request = place_order_request(&credentials(), &placed).expect("signed");

        assert_eq!(request.target(), "/v2/orders");
        let raw_body = request.body.as_deref().expect("a body");
        assert_eq!(
            raw_body,
            r#"{"market":"KRW-BTC","side":"bid","order_type":"limit","price":"100000000","volume":"0.01"}"#
        );
        let body: Value = serde_json::from_str(raw_body).expect("JSON");
        assert_eq!(body["market"], "KRW-BTC");
        assert_eq!(body["side"], "bid");
        assert_eq!(body["order_type"], "limit");
        assert_eq!(body["price"], "100000000");
        assert_eq!(body["volume"], "0.01");
    }

    #[test]
    fn a_market_from_another_exchange_never_reaches_a_private_call() {
        let elsewhere = Market::spot(Exchange::Upbit, "BTC", "KRW");

        assert!(open_orders_request(&credentials(), Some(&elsewhere)).is_err());
        assert!(
            placed_order(&OrderRequest::market(
                elsewhere,
                Side::Sell,
                Size::Base(Decimal::ONE)
            ))
            .is_err()
        );
    }
}
