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

use super::parse::{self, EXCHANGE};
use super::rest;
use super::{
    BithumbApiKey, BithumbBatchOrder, BithumbBatchOrderFailure, BithumbBatchOrderOutcome,
    BithumbBatchOrdersRequest, BithumbBatchOrdersResult, BithumbCancelOrderResponse,
    BithumbCancelOrdersResponse, BithumbClosedOrder, BithumbClosedOrdersRequest,
    BithumbCredentials, BithumbOrderDetail, BithumbOrderDetailRequest, BithumbOrderDetailTrade,
    BithumbOrderDirection, BithumbOrderListItem, BithumbOrderListRequest, BithumbOrderListState,
    BithumbOrderResponse, BithumbOrdersResponse, BithumbPendingOrderState,
    BithumbPendingOrdersRequest, BithumbTwapOrder, BithumbTwapOrderDirection,
    BithumbTwapOrderRequest, BithumbTwapOrdersRequest, BithumbTwapState,
};

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

fn signed_get_with_cursor(
    credentials: &BithumbCredentials,
    path: &str,
    mut params: Vec<(&str, String)>,
    cursor: Option<&Cursor>,
) -> Result<HttpRequest> {
    let mut signed = signed_query(&params)?;
    if let Some(cursor) = cursor {
        let cursor = checked_cursor(cursor)?;
        if !signed.is_empty() {
            signed.push('&');
        }
        signed.push_str("next_key=");
        signed.push_str(cursor);
        params.push(("next_key", cursor.to_owned()));
    }

    Ok(HttpRequest::get(path)
        .query(encoded_query(&params))
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

pub(crate) fn api_keys_request(credentials: &BithumbCredentials) -> Result<HttpRequest> {
    signed_get(credentials, "/v1/api_keys", &[])
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

/// Builds the legacy `/v1/orders` list request without reducing its filters.
pub(crate) fn order_list_request(
    credentials: &BithumbCredentials,
    request: &BithumbOrderListRequest,
) -> Result<HttpRequest> {
    if request.state.is_some() && !request.states.is_empty() {
        return Err(Error::invalid_request(
            "states",
            "Bithumb accepts either `state` or `states`, not both",
        ));
    }
    if request.states.contains(&BithumbOrderListState::Watch)
        && request
            .states
            .iter()
            .any(|state| *state != BithumbOrderListState::Watch)
    {
        return Err(Error::invalid_request(
            "states",
            "Bithumb does not combine `watch` with ordinary order states",
        ));
    }
    if request.uuids.len() > 100 {
        return Err(Error::invalid_request(
            "uuids",
            format!(
                "Bithumb accepts at most 100 order UUIDs, not {}",
                request.uuids.len()
            ),
        ));
    }
    let use_uuid_filters = !request.uuids.is_empty();
    if !use_uuid_filters && request.client_order_ids.len() > 100 {
        return Err(Error::invalid_request(
            "client_order_ids",
            format!(
                "Bithumb accepts at most 100 client order ids, not {}",
                request.client_order_ids.len()
            ),
        ));
    }

    let page = request.page.unwrap_or(1);
    if page == 0 {
        return Err(Error::invalid_request(
            "page",
            "Bithumb order-list pages start at 1",
        ));
    }
    let limit = request.limit.unwrap_or(100);
    if !(1..=100).contains(&limit) {
        return Err(Error::invalid_request(
            "limit",
            format!("Bithumb serves 1 to 100 orders per page, not {limit}"),
        ));
    }

    let mut params = Vec::new();
    if let Some(market) = &request.market {
        params.push(("market", parse::native_symbol(market)?));
    }
    if let Some(state) = request.state {
        params.push(("state", state.wire_name().to_string()));
    }
    for state in &request.states {
        params.push(("states[]", state.wire_name().to_string()));
    }
    for uuid in &request.uuids {
        params.push(("uuids[]", checked_order_uuid(uuid)?));
    }
    if !use_uuid_filters {
        for client_order_id in &request.client_order_ids {
            validate_client_order_id(client_order_id)?;
            params.push(("client_order_ids[]", client_order_id.clone()));
        }
    }
    params.push(("page", page.to_string()));
    params.push(("limit", limit.to_string()));
    params.push((
        "order_by",
        match request
            .order_by
            .unwrap_or(BithumbOrderDirection::Descending)
        {
            BithumbOrderDirection::Ascending => "asc",
            BithumbOrderDirection::Descending => "desc",
        }
        .to_string(),
    ));

    signed_get(credentials, "/v1/orders", &params)
}

pub(crate) fn pending_orders_request(
    credentials: &BithumbCredentials,
    request: &BithumbPendingOrdersRequest,
) -> Result<HttpRequest> {
    let mut params = Vec::new();
    if let Some(market) = &request.market {
        params.push(("market", parse::native_symbol(market)?));
    }
    if let Some(state) = request.state {
        params.push((
            "state",
            match state {
                BithumbPendingOrderState::Wait => "wait",
                BithumbPendingOrderState::Watch => "watch",
            }
            .to_owned(),
        ));
    }
    if let Some(limit) = request.limit {
        if !(1..=100).contains(&limit) {
            return Err(Error::invalid_request(
                "limit",
                format!("bithumb serves 1 to 100 pending orders per page, not {limit}"),
            ));
        }
        params.push(("limit", limit.to_string()));
    }
    if let Some(order_by) = request.order_by {
        params.push((
            "order_by",
            match order_by {
                BithumbOrderDirection::Ascending => "asc",
                BithumbOrderDirection::Descending => "desc",
            }
            .to_owned(),
        ));
    }
    signed_get_with_cursor(
        credentials,
        "/v2/orders/pending",
        params,
        request.cursor.as_ref(),
    )
}

pub(crate) fn twap_orders_request(
    credentials: &BithumbCredentials,
    request: &BithumbTwapOrdersRequest,
) -> Result<HttpRequest> {
    let mut params = Vec::new();
    if let Some(market) = &request.market {
        params.push(("market", parse::native_symbol(market)?));
    }
    for uuid in &request.uuids {
        params.push(("uuids[]", uuid.clone()));
    }
    if let Some(state) = request.state {
        params.push((
            "state",
            match state {
                BithumbTwapState::Progress => "progress",
                BithumbTwapState::Done => "done",
                BithumbTwapState::Cancel => "cancel",
            }
            .to_owned(),
        ));
    }
    if let Some(limit) = request.limit {
        if !(1..=100).contains(&limit) {
            return Err(Error::invalid_request(
                "limit",
                format!("bithumb serves 1 to 100 TWAP orders per page, not {limit}"),
            ));
        }
        params.push(("limit", limit.to_string()));
    }
    if let Some(order_by) = request.order_by {
        params.push((
            "order_by",
            match order_by {
                BithumbTwapOrderDirection::Ascending => "asc",
                BithumbTwapOrderDirection::Descending => "desc",
            }
            .to_owned(),
        ));
    }
    signed_get_with_cursor(credentials, "/v1/twap", params, request.cursor.as_ref())
}

pub(crate) fn create_twap_order_request(
    credentials: &BithumbCredentials,
    request: &BithumbTwapOrderRequest,
) -> Result<HttpRequest> {
    let market = parse::native_symbol(&request.market)?;
    if request.market.quote != "KRW" {
        return Err(Error::invalid_request(
            "market.quote",
            "Bithumb TWAP supports KRW markets only",
        ));
    }
    let side = match request.side {
        Side::Buy => "bid",
        Side::Sell => "ask",
    };
    let required_amount = match request.side {
        Side::Buy => ("price", request.price),
        Side::Sell => ("volume", request.volume),
    };
    if required_amount.1.is_none() {
        return Err(Error::invalid_request(
            required_amount.0,
            format!("a TWAP {side} needs `{}`", required_amount.0),
        ));
    }
    if !(300..=43_200).contains(&request.duration) {
        return Err(Error::invalid_request(
            "duration",
            format!(
                "bithumb TWAP duration is 300..=43200 seconds, not {}",
                request.duration
            ),
        ));
    }
    if !matches!(request.frequency, 15 | 20 | 30 | 60 | 120) {
        return Err(Error::invalid_request(
            "frequency",
            format!(
                "bithumb TWAP frequency is 15, 20, 30, 60 or 120 seconds, not {}",
                request.frequency
            ),
        ));
    }

    let mut params = vec![("market", market), ("side", side.to_owned())];
    if let Some(volume) = request.volume {
        params.push(("volume", amount("volume", volume)?));
    }
    if let Some(price) = request.price {
        params.push(("price", amount("price", price)?));
    }
    params.push(("duration", request.duration.to_string()));
    params.push(("frequency", request.frequency.to_string()));

    let query = signed_query(&params)?;
    let body = params
        .iter()
        .map(|(key, value)| ((*key).to_owned(), Value::String(value.clone())))
        .collect::<serde_json::Map<_, _>>();
    let body = serde_json::to_string(&Value::Object(body))
        .map_err(|err| Error::decode(format!("could not build the Bithumb TWAP body: {err}")))?;

    Ok(HttpRequest::post("/v1/twap")
        .json_body(body)
        .header("authorization", authorization(credentials, &query)?))
}

/// Builds Bithumb's non-atomic batch-order request.
pub(crate) fn batch_orders_request(
    credentials: &BithumbCredentials,
    request: &BithumbBatchOrdersRequest,
) -> Result<HttpRequest> {
    if !(1..=20).contains(&request.orders.len()) {
        return Err(Error::invalid_request(
            "orders",
            format!(
                "bithumb accepts 1 to 20 batch orders, not {}",
                request.orders.len()
            ),
        ));
    }

    let mut body_orders = Vec::with_capacity(request.orders.len());
    let mut signed_params = Vec::new();
    for (index, order) in request.orders.iter().enumerate() {
        let placed = placed_order(order)?;
        let mut body = serde_json::Map::new();
        for (key, value) in placed.params {
            signed_params.push((format!("batch_orders[{index}][{key}]"), value.clone()));
            body.insert(key.to_string(), Value::String(value));
        }
        body_orders.push(Value::Object(body));
    }

    let signed_params = signed_params
        .iter()
        .map(|(key, value)| (key.as_str(), value.clone()))
        .collect::<Vec<_>>();
    let query = signed_query(&signed_params)?;
    let body = serde_json::to_string(&serde_json::json!({
        "batch_orders": body_orders
    }))
    .map_err(|err| Error::decode(format!("could not build the Bithumb batch body: {err}")))?;

    Ok(HttpRequest::post("/v2/orders/batch")
        .json_body(body)
        .header("authorization", authorization(credentials, &query)?))
}

pub(crate) fn cancel_twap_order_request(
    credentials: &BithumbCredentials,
    algo_order_id: &str,
) -> Result<HttpRequest> {
    let params = [("algo_order_id", algo_order_id.to_owned())];
    let query = signed_query(&params)?;
    Ok(HttpRequest::delete("/v1/twap")
        .query(encoded_query(&params))
        .header("authorization", authorization(credentials, &query)?))
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
        "client_order_id",
        client_id,
    )
}

/// Builds the documented `/v1/order` lookup, including both identifiers when
/// present. Bithumb gives `uuid` precedence in that case.
pub(crate) fn order_detail_request(
    credentials: &BithumbCredentials,
    request: &BithumbOrderDetailRequest,
) -> Result<HttpRequest> {
    parse::native_symbol(&request.market)?;

    let mut params = Vec::with_capacity(2);
    if let Some(uuid) = &request.uuid {
        params.push(("uuid", checked_order_uuid(uuid)?));
    }
    if let Some(client_order_id) = &request.client_order_id {
        validate_client_order_id(client_order_id)?;
        params.push(("client_order_id", client_order_id.clone()));
    }
    if params.is_empty() {
        return Err(Error::invalid_request(
            "identifier",
            "set a Bithumb UUID or client order id",
        ));
    }

    signed_get(credentials, "/v1/order", &params)
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

fn checked_order_uuid(value: &str) -> Result<String> {
    if value.trim().is_empty() {
        return Err(Error::invalid_request("uuid", "must not be empty"));
    }
    signed_value("uuid", value)?;
    Ok(value.to_owned())
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
    signed_get_with_cursor(
        credentials,
        "/v2/orders/history",
        params,
        request.cursor.as_ref(),
    )
}

pub(crate) fn closed_orders_request(
    credentials: &BithumbCredentials,
    request: &BithumbClosedOrdersRequest,
) -> Result<HttpRequest> {
    const SEVEN_DAYS_NANOS: i64 = 7 * 24 * 60 * 60 * 1_000_000_000;

    if request.state.is_some() && !request.states.is_empty() {
        return Err(Error::invalid_request(
            "states",
            "Bithumb accepts either `state` or `states`, not both",
        ));
    }
    if let Some(limit) = request.limit
        && !(1..=1_000).contains(&limit)
    {
        return Err(Error::invalid_request(
            "limit",
            format!("Bithumb serves 1 to 1000 closed orders per page, not {limit}"),
        ));
    }

    if let (Some(start_time), Some(end_time)) = (request.start_time, request.end_time) {
        let width = end_time
            .as_nanos()
            .checked_sub(start_time.as_nanos())
            .ok_or_else(|| Error::invalid_request("end_time", "must be at least `start_time`"))?;
        if width < 0 {
            return Err(Error::invalid_request(
                "end_time",
                "must be at least `start_time`",
            ));
        }
        if width > SEVEN_DAYS_NANOS {
            return Err(Error::invalid_request(
                "end_time",
                "Bithumb closed-order windows cannot exceed seven days",
            ));
        }
    }

    let start_time = request.start_time.map(Timestamp::as_millis);
    let end_time = request.end_time.map(Timestamp::as_millis);

    let mut params = Vec::new();
    if let Some(market) = &request.market {
        params.push(("market", parse::native_symbol(market)?));
    }
    if let Some(state) = request.state {
        params.push(("state", state.wire_name().to_owned()));
    }
    for state in &request.states {
        params.push(("states[]", state.wire_name().to_owned()));
    }
    if let Some(start_time) = start_time {
        params.push(("start_time", start_time.to_string()));
    }
    if let Some(end_time) = end_time {
        params.push(("end_time", end_time.to_string()));
    }
    if let Some(limit) = request.limit {
        params.push(("limit", limit.to_string()));
    }
    if let Some(order_by) = request.order_by {
        params.push((
            "order_by",
            match order_by {
                BithumbOrderDirection::Ascending => "asc",
                BithumbOrderDirection::Descending => "desc",
            }
            .to_owned(),
        ));
    }

    signed_get_with_cursor(
        credentials,
        "/v2/orders/history",
        params,
        request.cursor.as_ref(),
    )
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
        "client_order_id",
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

pub(crate) async fn pending_orders(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &BithumbPendingOrdersRequest,
) -> Result<Page<Order>> {
    let body = rest::send(http, &pending_orders_request(credentials, request)?).await?;
    order_page(&body)
}

pub(crate) async fn twap_orders(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &BithumbTwapOrdersRequest,
) -> Result<Page<BithumbTwapOrder>> {
    let body = rest::send(http, &twap_orders_request(credentials, request)?).await?;
    twap_order_page(&body)
}

pub(crate) async fn create_twap_order(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &BithumbTwapOrderRequest,
) -> Result<String> {
    let body = rest::send(http, &create_twap_order_request(credentials, request)?).await?;
    algo_order_id(&body)
}

pub(crate) async fn cancel_twap_order(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    algo_order_id: &str,
) -> Result<String> {
    let body = rest::send(
        http,
        &cancel_twap_order_request(credentials, algo_order_id)?,
    )
    .await?;
    algo_order_id_from_body(&body)
}

fn twap_order_page(body: &Value) -> Result<Page<BithumbTwapOrder>> {
    let has_next = body
        .get("has_next")
        .and_then(Value::as_bool)
        .ok_or_else(|| Error::decode("bithumb TWAP page `has_next` is not a boolean"))?;
    let next = match body.get("next_key") {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) if !value.is_empty() => Some(Cursor::new(value.clone())),
        Some(_) => {
            return Err(Error::decode(
                "bithumb TWAP page `next_key` is not a non-empty string or null",
            ));
        }
    };
    if has_next != next.is_some() {
        return Err(Error::decode(
            "bithumb TWAP page disagrees about `has_next` and `next_key`",
        ));
    }
    let orders = body
        .get("orders")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::decode("bithumb TWAP page carries no `orders` array"))?;
    Ok(Page {
        items: orders
            .iter()
            .map(parse_twap_order)
            .collect::<Result<Vec<_>>>()?,
        next,
    })
}

fn parse_twap_order(value: &Value) -> Result<BithumbTwapOrder> {
    let id = twap_text(value, "uuid")?;
    if id.is_empty() {
        return Err(Error::decode("bithumb TWAP `uuid` is empty"));
    }
    let market = twap_market(twap_text(value, "market")?)?;
    let created_at = twap_time(value, "created_at")?;
    Ok(BithumbTwapOrder {
        id: id.to_owned(),
        side: twap_side(value, "side")?,
        price: parse::dec(value, "price")?,
        state: twap_state(value, "state")?,
        market,
        created_at,
        volume: parse::dec(value, "volume")?,
        finished_at: twap_optional_time(value, "finished_at")?,
        total_order_count: twap_count(value, "total_order_count")?,
        total_trades_count: twap_count(value, "total_trades_count")?,
        progress_count: twap_count(value, "progress_count")?,
        total_executed_amount: parse::dec(value, "total_executed_amount")?,
        total_executed_volume: parse::dec(value, "total_executed_volume")?,
        avg_trade_price: parse::dec(value, "avg_trade_price")?,
        wallet_id: twap_optional_text(value, "wallet_id")?,
        canceled_at: twap_optional_time(value, "canceled_at")?,
        cancel_type: twap_optional_text(value, "cancel_type")?,
    })
}

fn algo_order_id(body: &Value) -> Result<String> {
    algo_order_id_from_body(body)
}

fn algo_order_id_from_body(body: &Value) -> Result<String> {
    let id = twap_text(body, "algo_order_id")?;
    if id.is_empty() {
        return Err(Error::decode(
            "bithumb TWAP response `algo_order_id` is empty",
        ));
    }
    Ok(id.to_owned())
}

fn twap_text<'a>(value: &'a Value, field: &'static str) -> Result<&'a str> {
    value
        .get(field)
        .filter(|value| !value.is_null())
        .and_then(Value::as_str)
        .ok_or_else(|| Error::decode(format!("bithumb TWAP `{field}` is not a string")))
}

fn twap_optional_text(value: &Value, field: &'static str) -> Result<Option<String>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(Error::decode(format!(
            "bithumb TWAP `{field}` is not a string"
        ))),
    }
}

fn twap_side(value: &Value, field: &'static str) -> Result<Side> {
    match twap_text(value, field)? {
        "bid" => Ok(Side::Buy),
        "ask" => Ok(Side::Sell),
        other => Err(Error::decode(format!(
            "bithumb TWAP `{field}` is not bid or ask: `{other}`"
        ))),
    }
}

fn twap_state(value: &Value, field: &'static str) -> Result<BithumbTwapState> {
    match twap_text(value, field)? {
        "progress" => Ok(BithumbTwapState::Progress),
        "done" => Ok(BithumbTwapState::Done),
        "cancel" => Ok(BithumbTwapState::Cancel),
        other => Err(Error::decode(format!(
            "bithumb TWAP `{field}` is unknown: `{other}`"
        ))),
    }
}

fn twap_market(raw: &str) -> Result<Market> {
    let (quote, base) = raw
        .split_once('-')
        .ok_or_else(|| Error::decode(format!("bithumb TWAP market is invalid: `{raw}`")))?;
    let market = Market::spot(crate::types::Exchange::Bithumb, base, quote);
    if parse::native_symbol(&market)? != raw {
        return Err(Error::decode(format!(
            "bithumb TWAP market is invalid: `{raw}`"
        )));
    }
    Ok(market)
}

fn twap_time(value: &Value, field: &'static str) -> Result<Timestamp> {
    let raw = twap_text(value, field)?;
    parse::offset_time(raw)
        .ok_or_else(|| Error::decode(format!("bithumb TWAP `{field}` is not RFC 3339: `{raw}`")))
}

fn twap_optional_time(value: &Value, field: &'static str) -> Result<Option<Timestamp>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => twap_time(value, field).map(Some),
    }
}

fn twap_count(value: &Value, field: &'static str) -> Result<u32> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| Error::decode(format!("bithumb TWAP `{field}` is not a u32")))
}

pub(crate) async fn api_keys(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
) -> Result<Vec<BithumbApiKey>> {
    parse::api_keys(&rest::send(http, &api_keys_request(credentials)?).await?)
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

/// Reads a single Bithumb order without collapsing provider-specific fields.
pub(crate) async fn order_detail(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &BithumbOrderDetailRequest,
) -> Result<BithumbOrderDetail> {
    let body = rest::send(http, &order_detail_request(credentials, request)?).await?;
    let detail = parse_order_detail(&body)?;
    if detail.market == request.market {
        Ok(detail)
    } else {
        Err(Error::invalid_request(
            "market",
            format!(
                "the requested identifier belongs to {}, not {}",
                detail.market, request.market
            ),
        ))
    }
}

/// Reads the legacy Bithumb order list without collapsing provider fields.
pub(crate) async fn order_list(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &BithumbOrderListRequest,
) -> Result<Vec<BithumbOrderListItem>> {
    let body = rest::send(http, &order_list_request(credentials, request)?).await?;
    parse_order_list(&body)
}

pub(crate) async fn orders_by_ids(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &OrderLookupRequest,
) -> Result<Vec<Order>> {
    let body = rest::send(http, &orders_by_ids_request(credentials, request)?).await?;
    parse::orders(&body)
}

/// Reads matching orders while preserving Bithumb's provider response body.
pub(crate) async fn orders_by_ids_detail(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &OrderLookupRequest,
) -> Result<BithumbOrdersResponse> {
    let body = rest::send(http, &orders_by_ids_request(credentials, request)?).await?;
    Ok(BithumbOrdersResponse {
        common: parse::orders(&body)?,
        raw_json: response_json(&body, "order lookup")?,
    })
}

pub(crate) async fn order_history(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &OrderHistoryRequest,
) -> Result<Page<Order>> {
    let body = rest::send(http, &order_history_request(credentials, request)?).await?;
    order_page(&body)
}

pub(crate) async fn closed_orders(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &BithumbClosedOrdersRequest,
) -> Result<Page<BithumbClosedOrder>> {
    let body = rest::send(http, &closed_orders_request(credentials, request)?).await?;
    closed_order_page(&body)
}

fn closed_order_page(body: &Value) -> Result<Page<BithumbClosedOrder>> {
    let data = body
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::decode("bithumb closed-order page carries no `data` array"))?;
    let has_next = body
        .get("has_next")
        .and_then(Value::as_bool)
        .ok_or_else(|| Error::decode("bithumb closed-order page `has_next` is not a boolean"))?;
    let next = match body.get("next_key") {
        Some(Value::Null) => None,
        Some(Value::String(value)) if !value.is_empty() => Some(Cursor::new(value.clone())),
        Some(_) => {
            return Err(Error::decode(
                "bithumb closed-order page `next_key` is not a non-empty string or null",
            ));
        }
        None => {
            return Err(Error::decode(
                "bithumb closed-order page carries no `next_key`",
            ));
        }
    };
    if has_next != next.is_some() {
        return Err(Error::decode(
            "bithumb closed-order page disagrees about `has_next` and `next_key`",
        ));
    }

    Ok(Page {
        items: data
            .iter()
            .map(parse_closed_order)
            .collect::<Result<Vec<_>>>()?,
        next,
    })
}

fn parse_closed_order(value: &Value) -> Result<BithumbClosedOrder> {
    Ok(BithumbClosedOrder {
        order_id: detail_text(value, "order_id")?,
        side: detail_text(value, "side")?,
        order_type: detail_text(value, "order_type")?,
        price: closed_order_optional_decimal(value, "price")?,
        state: detail_text(value, "state")?,
        market: parse::market_field(value, "market")?,
        created_at: closed_order_optional_time(value, "created_at")?,
        volume: parse::dec(value, "volume")?,
        remaining_volume: parse::dec(value, "remaining_volume")?,
        reserved_fee: parse::dec(value, "reserved_fee")?,
        remaining_fee: parse::dec(value, "remaining_fee")?,
        paid_fee: parse::dec(value, "paid_fee")?,
        locked: parse::dec(value, "locked")?,
        executed_volume: parse::dec(value, "executed_volume")?,
        executed_funds: parse::dec(value, "executed_funds")?,
        trades_count: detail_u32(value, "trades_count")?,
        client_order_id: detail_optional_text(value, "client_order_id")?,
        stp_type: detail_optional_text(value, "stp_type")?,
        time_in_force: detail_optional_text(value, "time_in_force")?,
        cancel_type: detail_optional_text(value, "cancel_type")?,
        canceling_order_id: detail_optional_text(value, "canceling_order_id")?,
    })
}

fn closed_order_optional_decimal(value: &Value, field: &'static str) -> Result<Option<Decimal>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => parse::dec(value, field).map(Some),
    }
}

fn closed_order_optional_time(value: &Value, field: &'static str) -> Result<Option<Timestamp>> {
    detail_optional_text(value, field)?
        .map(|raw| {
            parse::offset_time(&raw).ok_or_else(|| {
                Error::decode(format!(
                    "bithumb closed order `{field}` is not an RFC 3339 timestamp with an offset"
                ))
            })
        })
        .transpose()
}

fn order_page(body: &Value) -> Result<Page<Order>> {
    let data = body
        .get("data")
        .ok_or_else(|| Error::decode("bithumb order page carries no `data`"))?;
    let has_next = body
        .get("has_next")
        .and_then(Value::as_bool)
        .ok_or_else(|| Error::decode("bithumb order page `has_next` is not a boolean"))?;
    let next = match body.get("next_key") {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) if !value.is_empty() => Some(Cursor::new(value.clone())),
        Some(_) => {
            return Err(Error::decode(
                "bithumb order page `next_key` is not a non-empty string or null",
            ));
        }
    };
    if has_next != next.is_some() {
        return Err(Error::decode(
            "bithumb order page disagrees about `has_next` and `next_key`",
        ));
    }
    Ok(Page {
        items: parse::orders(data)?,
        next,
    })
}

fn checked_cursor(cursor: &Cursor) -> Result<&str> {
    let cursor = cursor.as_str();
    if cursor.is_empty() {
        return Err(Error::invalid_request("cursor", "must not be empty"));
    }
    Ok(cursor)
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

fn parse_order_detail(body: &Value) -> Result<BithumbOrderDetail> {
    let trades = body
        .get("trades")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::decode("bithumb order detail has no `trades` array"))?
        .iter()
        .map(parse_order_detail_trade)
        .collect::<Result<Vec<_>>>()?;
    let trades_count = detail_u32(body, "trades_count")?;
    if trades.len() != usize::try_from(trades_count).unwrap_or(usize::MAX) {
        return Err(Error::decode(
            "bithumb order detail disagrees about `trades_count` and the `trades` array",
        ));
    }

    Ok(BithumbOrderDetail {
        uuid: detail_text(body, "uuid")?,
        client_order_id: detail_optional_text(body, "client_order_id")?,
        side: detail_text(body, "side")?,
        order_type: detail_text(body, "ord_type")?,
        price: parse::dec(body, "price")?,
        state: detail_text(body, "state")?,
        market: parse::market_field(body, "market")?,
        created_at: detail_time(body, "created_at")?,
        volume: parse::dec(body, "volume")?,
        remaining_volume: parse::dec(body, "remaining_volume")?,
        reserved_fee: parse::dec(body, "reserved_fee")?,
        remaining_fee: parse::dec(body, "remaining_fee")?,
        paid_fee: parse::dec(body, "paid_fee")?,
        locked: parse::dec(body, "locked")?,
        executed_volume: parse::dec(body, "executed_volume")?,
        executed_funds: parse::dec(body, "executed_funds")?,
        trades_count,
        trades,
        stp_type: detail_optional_text(body, "stp_type")?,
        cancel_type: detail_optional_text(body, "cancel_type")?,
        canceling_uuid: detail_optional_text(body, "canceling_uuid")?,
        time_in_force: detail_optional_text(body, "time_in_force")?,
    })
}

fn parse_order_detail_trade(value: &Value) -> Result<BithumbOrderDetailTrade> {
    Ok(BithumbOrderDetailTrade {
        market: parse::market_field(value, "market")?,
        uuid: detail_text(value, "uuid")?,
        price: parse::dec(value, "price")?,
        volume: parse::dec(value, "volume")?,
        funds: parse::dec(value, "funds")?,
        side: detail_text(value, "side")?,
        created_at: detail_time(value, "created_at")?,
    })
}

fn parse_order_list(body: &Value) -> Result<Vec<BithumbOrderListItem>> {
    body.as_array()
        .ok_or_else(|| Error::decode("bithumb order list is not an array"))?
        .iter()
        .map(parse_order_list_item)
        .collect()
}

fn parse_order_list_item(value: &Value) -> Result<BithumbOrderListItem> {
    Ok(BithumbOrderListItem {
        uuid: detail_text(value, "uuid")?,
        client_order_id: detail_optional_text(value, "client_order_id")?,
        side: detail_text(value, "side")?,
        order_type: detail_text(value, "ord_type")?,
        price: parse::dec(value, "price")?,
        state: detail_text(value, "state")?,
        market: parse::market_field(value, "market")?,
        created_at: detail_time(value, "created_at")?,
        volume: parse::dec(value, "volume")?,
        remaining_volume: parse::dec(value, "remaining_volume")?,
        reserved_fee: parse::dec(value, "reserved_fee")?,
        remaining_fee: parse::dec(value, "remaining_fee")?,
        paid_fee: parse::dec(value, "paid_fee")?,
        locked: parse::dec(value, "locked")?,
        executed_volume: parse::dec(value, "executed_volume")?,
        executed_funds: parse::dec(value, "executed_funds")?,
        trades_count: detail_u32(value, "trades_count")?,
        stp_type: detail_optional_text(value, "stp_type")?,
        time_in_force: detail_optional_text(value, "time_in_force")?,
        raw_json: response_json(value, "order-list item")?,
    })
}

fn detail_text(value: &Value, field: &'static str) -> Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            Error::decode(format!(
                "bithumb order detail `{field}` is missing or empty"
            ))
        })
}

fn detail_optional_text(value: &Value, field: &'static str) -> Result<Option<String>> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if !value.is_empty() => Ok(Some(value.clone())),
        Some(Value::String(_)) => Err(Error::decode(format!(
            "bithumb order detail `{field}` must not be empty when present"
        ))),
        Some(_) => Err(Error::decode(format!(
            "bithumb order detail `{field}` is not text"
        ))),
    }
}

fn detail_time(value: &Value, field: &'static str) -> Result<Timestamp> {
    let raw = detail_text(value, field)?;
    parse::offset_time(&raw).ok_or_else(|| {
        Error::decode(format!(
            "bithumb order detail `{field}` is not an RFC 3339 timestamp with an offset"
        ))
    })
}

fn detail_u32(value: &Value, field: &'static str) -> Result<u32> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| Error::decode(format!("bithumb order detail `{field}` is not a u32")))
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

/// Places one order while preserving Bithumb's provider acknowledgement.
pub(crate) async fn place_order_detail(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &OrderRequest,
) -> Result<BithumbOrderResponse> {
    let placed = placed_order(request)?;
    let body = rest::send(http, &place_order_request(credentials, &placed)?).await?;
    Ok(BithumbOrderResponse {
        common: parse::order_ack(
            &body,
            request.market.clone(),
            request.side,
            OrderStatus::Accepted,
            placed.remaining_quantity,
            request.price,
        )?,
        raw_json: response_json(&body, "order placement")?,
    })
}

pub(crate) async fn batch_orders(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &BithumbBatchOrdersRequest,
) -> Result<BithumbBatchOrdersResult> {
    let body = rest::send(http, &batch_orders_request(credentials, request)?).await?;
    batch_orders_result(&body)
}

fn batch_orders_result(body: &Value) -> Result<BithumbBatchOrdersResult> {
    let outcomes = body
        .get("batch_orders_response")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            Error::decode("bithumb batch-order response has no `batch_orders_response` array")
        })?
        .iter()
        .map(batch_order_outcome)
        .collect::<Result<Vec<_>>>()?;
    Ok(BithumbBatchOrdersResult {
        outcomes,
        raw_json: response_json(body, "batch order")?,
    })
}

fn batch_order_outcome(entry: &Value) -> Result<BithumbBatchOrderOutcome> {
    let client_order_id = batch_text(entry, "client_order_id")?;
    if let Some(order_id) = batch_text(entry, "order_id")? {
        if order_id.is_empty() {
            return Err(Error::decode(
                "bithumb batch-order success carries an empty `order_id`",
            ));
        }
        let market = parse::market_field(entry, "market")?;
        let side = parse::side(entry, "side")?;
        let order_type = match batch_text(entry, "order_type")?
            .ok_or_else(|| Error::decode("bithumb batch-order success carries no `order_type`"))?
            .as_str()
        {
            "limit" => crate::types::OrderType::Limit,
            "price" | "market" => crate::types::OrderType::Market,
            "best" => crate::types::OrderType::Best,
            other => {
                return Err(Error::decode(format!(
                    "bithumb batch-order has unknown `order_type`: `{other}`"
                )));
            }
        };
        let created_at = batch_text(entry, "created_at")?
            .map(|raw| {
                parse::offset_time(&raw).ok_or_else(|| {
                    Error::decode(format!(
                        "bithumb batch-order `created_at` is not RFC 3339: `{raw}`"
                    ))
                })
            })
            .transpose()?;
        let time_in_force = batch_text(entry, "time_in_force")?;
        let stp_type = batch_text(entry, "stp_type")?;
        return Ok(BithumbBatchOrderOutcome::Accepted(BithumbBatchOrder {
            order_id,
            client_order_id,
            market,
            side,
            order_type,
            time_in_force,
            stp_type,
            created_at,
        }));
    }

    let time_in_force = batch_text(entry, "time_in_force")?;
    let code = batch_text(entry, "name")?
        .ok_or_else(|| Error::decode("bithumb batch-order failure carries no `name`"))?;
    let message = batch_text(entry, "message")?
        .ok_or_else(|| Error::decode("bithumb batch-order failure carries no `message`"))?;
    Ok(BithumbBatchOrderOutcome::Rejected(
        BithumbBatchOrderFailure {
            client_order_id,
            time_in_force,
            code,
            message,
        },
    ))
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

/// Cancels one exchange-ID order while preserving Bithumb's response body.
pub(crate) async fn cancel_order_detail(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    market: &Market,
    order_id: &str,
) -> Result<BithumbCancelOrderResponse> {
    parse::native_symbol(market)?;
    let body = rest::send(http, &cancel_order_request(credentials, order_id)?).await?;
    cancel_order_response(&body)
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

/// Cancels one client-ID order while preserving Bithumb's response body.
pub(crate) async fn cancel_order_by_client_id_detail(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    market: &Market,
    client_id: &str,
) -> Result<BithumbCancelOrderResponse> {
    parse::native_symbol(market)?;
    let body = rest::send(
        http,
        &cancel_order_by_client_id_request(credentials, client_id)?,
    )
    .await?;
    cancel_order_response(&body)
}

pub(crate) async fn cancel_orders(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &CancelOrdersRequest,
) -> Result<CancelOrdersResult> {
    let body = rest::send(http, &cancel_orders_request(credentials, request)?).await?;
    cancel_orders_result(&body)
}

/// Cancels an order batch while preserving Bithumb's provider response body.
pub(crate) async fn cancel_orders_detail(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &CancelOrdersRequest,
) -> Result<BithumbCancelOrdersResponse> {
    let body = rest::send(http, &cancel_orders_request(credentials, request)?).await?;
    Ok(BithumbCancelOrdersResponse {
        common: cancel_orders_result(&body)?,
        raw_json: response_json(&body, "batch cancellation")?,
    })
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
    cancel_order_response(body).map(|_| ())
}

fn cancel_order_response(body: &Value) -> Result<BithumbCancelOrderResponse> {
    match body.get("order_id").and_then(Value::as_str) {
        Some(order_id) if !order_id.is_empty() => Ok(BithumbCancelOrderResponse {
            order_id: order_id.to_owned(),
            raw_json: response_json(body, "order cancellation")?,
        }),
        _ => Err(Error::decode(
            "bithumb cancel response carries no `order_id`",
        )),
    }
}

fn response_json(body: &Value, operation: &str) -> Result<String> {
    serde_json::to_string(body)
        .map_err(|error| Error::decode(format!("could not preserve Bithumb {operation}: {error}")))
}

/// Builds a parameterless authorization header for a private WebSocket handshake.
pub(crate) fn websocket_authorization(credentials: &BithumbCredentials) -> Result<String> {
    authorization(credentials, "")
}

#[cfg(test)]
mod tests {
    use super::super::BithumbClosedOrderState;
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
        // Shape reference: https://apidocs.bithumb.com/reference/api-키-리스트-조회.md
        assert_eq!(
            api_keys_request(&credentials()).expect("signed").target(),
            "/v1/api_keys"
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
        let pending = BithumbPendingOrdersRequest::new()
            .market(btc_krw())
            .state(BithumbPendingOrderState::Watch)
            .limit(25)
            .order_by(BithumbOrderDirection::Ascending)
            .cursor(Cursor::new("page+/=="));
        let request = pending_orders_request(&credentials(), &pending).expect("signed");
        assert_eq!(
            request.target(),
            "/v2/orders/pending?market=KRW-BTC&state=watch&limit=25&order_by=asc&next_key=page%2B%2F%3D%3D"
        );
        let authorization = request
            .headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value)
            .expect("an authorization header");
        assert_eq!(
            payload(authorization)["query_hash"],
            sha512_hex(b"market=KRW-BTC&state=watch&limit=25&order_by=asc&next_key=page+/==")
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
        let detail =
            BithumbOrderDetailRequest::by_uuid(btc_krw(), "order-1").client_order_id("client-1");
        assert_eq!(
            order_detail_request(&credentials(), &detail)
                .expect("signed")
                .target(),
            "/v1/order?uuid=order-1&client_order_id=client-1"
        );
        let order_list = BithumbOrderListRequest::new()
            .market(btc_krw())
            .states(vec![
                BithumbOrderListState::Done,
                BithumbOrderListState::Cancel,
            ])
            .uuids(vec!["order-1".to_string(), "order-2".to_string()])
            .client_order_ids(vec!["client-1".to_string(), "client-2".to_string()])
            .page(2)
            .limit(25)
            .order_by(BithumbOrderDirection::Ascending);
        let request = order_list_request(&credentials(), &order_list).expect("signed");
        assert_eq!(
            request.target(),
            "/v1/orders?market=KRW-BTC&states[]=done&states[]=cancel&uuids[]=order-1&uuids[]=order-2&page=2&limit=25&order_by=asc"
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
                b"market=KRW-BTC&states[]=done&states[]=cancel&uuids[]=order-1&uuids[]=order-2&page=2&limit=25&order_by=asc"
            )
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
    fn detailed_cancel_response_keeps_the_provider_body() {
        let response = cancel_order_response(&serde_json::json!({
            "order_id": "order-1",
            "provider_only": { "reason": "user_cancel" }
        }))
        .expect("a cancellation response");

        assert_eq!(response.order_id, "order-1");
        assert!(response.raw_json.contains("provider_only"));
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
    fn batch_order_hash_matches_the_flattened_body_and_preserves_duplicate_ids() {
        let first = OrderRequest::limit(
            btc_krw(),
            Side::Buy,
            Size::Base(Decimal::new(1, 3)),
            Decimal::from(100_000_000),
        )
        .client_id("duplicate");
        let second = OrderRequest::limit(
            btc_krw(),
            Side::Sell,
            Size::Base(Decimal::new(2, 3)),
            Decimal::from(101_000_000),
        )
        .client_id("duplicate");
        let request = batch_orders_request(
            &credentials(),
            &BithumbBatchOrdersRequest::new(vec![first, second]),
        )
        .expect("a signed batch request");
        let body: Value =
            serde_json::from_str(request.body.as_deref().expect("a body")).expect("valid JSON");
        let authorization = request
            .headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value)
            .expect("an authorization header");

        assert_eq!(request.target(), "/v2/orders/batch");
        assert_eq!(
            body,
            serde_json::json!({
                "batch_orders": [
                    {
                        "market": "KRW-BTC",
                        "side": "bid",
                        "order_type": "limit",
                        "price": "100000000",
                        "volume": "0.001",
                        "client_order_id": "duplicate"
                    },
                    {
                        "market": "KRW-BTC",
                        "side": "ask",
                        "order_type": "limit",
                        "price": "101000000",
                        "volume": "0.002",
                        "client_order_id": "duplicate"
                    }
                ]
            })
        );
        assert_eq!(
            payload(authorization)["query_hash"],
            sha512_hex(
                b"batch_orders[0][market]=KRW-BTC&batch_orders[0][side]=bid&batch_orders[0][order_type]=limit&batch_orders[0][price]=100000000&batch_orders[0][volume]=0.001&batch_orders[0][client_order_id]=duplicate&batch_orders[1][market]=KRW-BTC&batch_orders[1][side]=ask&batch_orders[1][order_type]=limit&batch_orders[1][price]=101000000&batch_orders[1][volume]=0.002&batch_orders[1][client_order_id]=duplicate"
            )
        );
    }

    #[test]
    fn batch_order_limit_and_market_validation_happen_before_signing() {
        let order = OrderRequest::market(btc_krw(), Side::Buy, Size::Quote(Decimal::ONE));
        let twenty = BithumbBatchOrdersRequest::new(vec![order.clone(); 20]);
        assert!(batch_orders_request(&credentials(), &twenty).is_ok());
        assert!(matches!(
            batch_orders_request(
                &credentials(),
                &BithumbBatchOrdersRequest::new(vec![order.clone(); 21]),
            ),
            Err(Error::InvalidRequest { field, .. }) if field == "orders"
        ));

        let elsewhere = OrderRequest::market(
            Market::spot(Exchange::Upbit, "BTC", "KRW"),
            Side::Buy,
            Size::Quote(Decimal::ONE),
        );
        assert!(matches!(
            batch_orders_request(
                &credentials(),
                &BithumbBatchOrdersRequest::new(vec![elsewhere]),
            ),
            Err(Error::InvalidRequest { field, .. }) if field == "market.exchange"
        ));
    }

    #[test]
    fn batch_order_response_keeps_partial_success_and_failure() {
        let result = batch_orders_result(&serde_json::json!({
            "batch_orders_response": [
                {
                    "client_order_id": "first",
                    "order_id": "C0101",
                    "market": "KRW-BTC",
                    "side": "bid",
                    "order_type": "limit",
                    "time_in_force": "post_only",
                    "stp_type": "cancel_maker",
                    "created_at": "2026-02-10T13:56:38+09:00"
                },
                {
                    "client_order_id": "second",
                    "time_in_force": "ioc",
                    "name": "cross_trading",
                    "message": "rejected"
                }
            ]
        }))
        .expect("a partial result");

        assert!(matches!(
            &result.outcomes[0],
            BithumbBatchOrderOutcome::Accepted(order)
                if order.order_id == "C0101"
                    && order.market == btc_krw()
                    && order.side == Side::Buy
                    && order.time_in_force.as_deref() == Some("post_only")
                    && order.stp_type.as_deref() == Some("cancel_maker")
        ));
        assert!(matches!(
            &result.outcomes[1],
            BithumbBatchOrderOutcome::Rejected(failure)
                if failure.client_order_id.as_deref() == Some("second")
                    && failure.time_in_force.as_deref() == Some("ioc")
                    && failure.code == "cross_trading"
        ));
    }

    #[test]
    fn final_order_history_encodes_an_opaque_cursor_but_hashes_its_raw_value() {
        let history = OrderHistoryRequest::new()
            .market(btc_krw())
            .status(OrderStatus::Cancelled)
            .from(Timestamp::from_millis(1_700_000_000_000))
            .to(Timestamp::from_millis(1_700_001_000_000))
            .cursor(Cursor::new("opaque:cursor%2F"))
            .limit(25);
        let request = order_history_request(&credentials(), &history).expect("signed history");

        assert_eq!(
            request.target(),
            "/v2/orders/history?market=KRW-BTC&state=cancel&start_time=1700000000000&end_time=1700000999999&limit=25&order_by=desc&next_key=opaque%3Acursor%252F"
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
                b"market=KRW-BTC&state=cancel&start_time=1700000000000&end_time=1700000999999&limit=25&order_by=desc&next_key=opaque:cursor%2F"
            )
        );
    }

    #[test]
    fn closed_orders_hash_raw_values_but_url_encode_the_opaque_cursor() {
        let request = BithumbClosedOrdersRequest::new()
            .market(btc_krw())
            .states(vec![
                BithumbClosedOrderState::Done,
                BithumbClosedOrderState::Cancel,
            ])
            .start_time(Timestamp::from_nanos(1_700_000_000_000_999_999))
            .end_time(Timestamp::from_nanos(1_700_001_000_000_999_999))
            .limit(1_000)
            .order_by(BithumbOrderDirection::Ascending)
            .cursor(Cursor::new("opaque+/=="));
        let built = closed_orders_request(&credentials(), &request).expect("signed history");

        assert_eq!(
            built.target(),
            "/v2/orders/history?market=KRW-BTC&states[]=done&states[]=cancel&start_time=1700000000000&end_time=1700001000000&limit=1000&order_by=asc&next_key=opaque%2B%2F%3D%3D"
        );
        let authorization = built
            .headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value)
            .expect("authorization");
        assert_eq!(
            payload(authorization)["query_hash"],
            sha512_hex(
                b"market=KRW-BTC&states[]=done&states[]=cancel&start_time=1700000000000&end_time=1700001000000&limit=1000&order_by=asc&next_key=opaque+/=="
            )
        );
    }

    #[test]
    fn closed_orders_reject_invalid_filters_before_signing() {
        let conflicting = BithumbClosedOrdersRequest::new()
            .state(BithumbClosedOrderState::Done)
            .states(vec![BithumbClosedOrderState::Cancel]);
        let backwards = BithumbClosedOrdersRequest::new()
            .start_time(Timestamp::from_millis(2))
            .end_time(Timestamp::from_millis(1));
        let too_wide = BithumbClosedOrdersRequest::new()
            .start_time(Timestamp::from_millis(0))
            .end_time(Timestamp::from_millis(7 * 24 * 60 * 60 * 1_000 + 1));

        for request in [conflicting, backwards, too_wide] {
            assert!(matches!(
                closed_orders_request(&credentials(), &request),
                Err(Error::InvalidRequest { .. })
            ));
        }
        for limit in [0, 1_001] {
            assert!(matches!(
                closed_orders_request(
                    &credentials(),
                    &BithumbClosedOrdersRequest::new().limit(limit),
                ),
                Err(Error::InvalidRequest { field, .. }) if field == "limit"
            ));
        }
    }

    #[test]
    fn closed_order_page_preserves_all_provider_fields_and_null_cursor() {
        let page = closed_order_page(&serde_json::json!({
            "data": [{
                "order_id": "C0101000007410714029",
                "side": "provider_future_side",
                "order_type": "provider_future_type",
                "price": "50000000.0",
                "state": "provider_future_state",
                "market": "KRW-BTC",
                "created_at": "2026-04-06T12:00:00.000+09:00",
                "volume": "0.001",
                "remaining_volume": "0.0",
                "reserved_fee": "25.0",
                "remaining_fee": "0.0",
                "paid_fee": "25.0",
                "locked": "0.0",
                "executed_volume": "0.001",
                "executed_funds": "50000.0",
                "trades_count": 1,
                "client_order_id": "my-order-001",
                "stp_type": "provider_future_stp",
                "time_in_force": "provider_future_tif",
                "cancel_type": "provider_future_cancel",
                "canceling_order_id": "C0101000007410714000"
            }, {
                "order_id": "C0101000007410714030",
                "side": "ask",
                "order_type": "market",
                "state": "cancel",
                "market": "KRW-BTC",
                "volume": "1",
                "remaining_volume": "1",
                "reserved_fee": "0",
                "remaining_fee": "0",
                "paid_fee": "0",
                "locked": "0",
                "executed_volume": "0",
                "executed_funds": "0",
                "trades_count": 0
            }],
            "has_next": false,
            "next_key": null
        }))
        .expect("official-shaped page");

        assert_eq!(page.next, None);
        assert_eq!(page.items.len(), 2);
        let order = &page.items[0];
        assert_eq!(order.order_id, "C0101000007410714029");
        assert_eq!(order.side, "provider_future_side");
        assert_eq!(order.order_type, "provider_future_type");
        assert_eq!(order.state, "provider_future_state");
        assert_eq!(order.price, Some(Decimal::from(50_000_000)));
        assert_eq!(
            order.created_at,
            Some(Timestamp::from_millis(1_775_444_400_000))
        );
        assert_eq!(order.volume, Decimal::new(1, 3));
        assert_eq!(order.remaining_volume, Decimal::ZERO);
        assert_eq!(order.reserved_fee, Decimal::from(25));
        assert_eq!(order.remaining_fee, Decimal::ZERO);
        assert_eq!(order.paid_fee, Decimal::from(25));
        assert_eq!(order.locked, Decimal::ZERO);
        assert_eq!(order.executed_volume, Decimal::new(1, 3));
        assert_eq!(order.executed_funds, Decimal::from(50_000));
        assert_eq!(order.trades_count, 1);
        assert_eq!(order.client_order_id.as_deref(), Some("my-order-001"));
        assert_eq!(order.stp_type.as_deref(), Some("provider_future_stp"));
        assert_eq!(order.time_in_force.as_deref(), Some("provider_future_tif"));
        assert_eq!(order.cancel_type.as_deref(), Some("provider_future_cancel"));
        assert_eq!(
            order.canceling_order_id.as_deref(),
            Some("C0101000007410714000")
        );
        assert_eq!(page.items[1].price, None);
        assert_eq!(page.items[1].created_at, None);
    }

    #[test]
    fn closed_order_page_requires_consistent_cursor_metadata() {
        for body in [
            serde_json::json!({"data": [], "has_next": false}),
            serde_json::json!({"data": [], "has_next": true, "next_key": null}),
            serde_json::json!({"data": [], "has_next": false, "next_key": "cursor"}),
        ] {
            assert!(matches!(
                closed_order_page(&body),
                Err(Error::Decode { .. })
            ));
        }
    }

    #[test]
    fn pending_orders_reject_an_empty_cursor() {
        assert!(matches!(
            pending_orders_request(
                &credentials(),
                &BithumbPendingOrdersRequest::new().cursor(Cursor::new("")),
            ),
            Err(Error::InvalidRequest { field, .. }) if field == "cursor"
        ));
    }

    #[test]
    fn pending_orders_refuse_a_page_size_outside_bithumbs_limit() {
        for limit in [0, 101] {
            assert!(matches!(
                pending_orders_request(
                    &credentials(),
                    &BithumbPendingOrdersRequest::new().limit(limit),
                ),
                Err(Error::InvalidRequest { field, .. }) if field == "limit"
            ));
        }
    }

    #[test]
    fn bithumb_history_decodes_data_and_next_key() {
        let page = order_page(&serde_json::json!({
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
            "has_next": true,
            "next_key": "page+/=="
        }))
        .expect("a v2.1.5 history page");

        assert_eq!(page.items[0].id, "C0101000007410714100");
        assert_eq!(page.next.expect("another page").as_str(), "page+/==");
    }

    #[test]
    fn pending_orders_preserve_watch_orders_and_page_cursors() {
        let page = order_page(&serde_json::json!({
            "data": [{
                "order_id": "watch-1",
                "side": "bid",
                "state": "watch",
                "market": "KRW-BTC",
                "remaining_volume": "0.001",
                "executed_volume": "0",
                "created_at": "2026-04-09T10:00:00.123+09:00"
            }],
            "has_next": false,
            "next_key": null
        }))
        .expect("a pending-order page");

        assert_eq!(page.items[0].status, OrderStatus::Open);
        assert!(page.next.is_none());
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

    #[test]
    fn order_detail_requires_a_safe_identifier_and_a_bithumb_market() {
        let market = btc_krw();
        for request in [
            BithumbOrderDetailRequest::new(market.clone()),
            BithumbOrderDetailRequest::by_uuid(market.clone(), ""),
            BithumbOrderDetailRequest::by_uuid(market.clone(), "id&market=KRW-ETH"),
            BithumbOrderDetailRequest::by_client_order_id(market.clone(), "not valid"),
            BithumbOrderDetailRequest::by_uuid(
                Market::spot(Exchange::Upbit, "BTC", "KRW"),
                "order-1",
            ),
        ] {
            assert!(order_detail_request(&credentials(), &request).is_err());
        }

        assert!(matches!(
            order_detail_request(
                &credentials(),
                &BithumbOrderDetailRequest::by_client_order_id(market, "not valid"),
            ),
            Err(Error::InvalidRequest { field, .. }) if field == "client_order_id"
        ));
    }

    #[test]
    fn order_list_rejects_conflicting_or_invalid_filters_before_signing() {
        let conflicting_states = BithumbOrderListRequest::new()
            .state(BithumbOrderListState::Wait)
            .states(vec![BithumbOrderListState::Done]);
        let mixed_watch = BithumbOrderListRequest::new().states(vec![
            BithumbOrderListState::Watch,
            BithumbOrderListState::Done,
        ]);
        let zero_page = BithumbOrderListRequest::new().page(0);
        let large_limit = BithumbOrderListRequest::new().limit(101);
        let too_many_uuids = BithumbOrderListRequest::new()
            .uuids((0..101).map(|index| index.to_string()).collect::<Vec<_>>());
        let unsafe_uuid = BithumbOrderListRequest::new().uuids(vec!["uuid&state=done".to_string()]);
        let unsafe_client_id =
            BithumbOrderListRequest::new().client_order_ids(vec!["not valid".to_string()]);
        let too_many_client_ids = BithumbOrderListRequest::new().client_order_ids(
            (0..101)
                .map(|index| format!("client-{index}"))
                .collect::<Vec<_>>(),
        );

        for request in [
            conflicting_states,
            mixed_watch,
            zero_page,
            large_limit,
            too_many_uuids,
            unsafe_uuid,
            unsafe_client_id,
            too_many_client_ids,
        ] {
            assert!(order_list_request(&credentials(), &request).is_err());
        }
    }

    #[test]
    fn order_list_ignores_client_order_ids_when_uuids_are_present() {
        let request = BithumbOrderListRequest::new()
            .uuids(vec!["order-1".to_string()])
            .client_order_ids(vec!["not valid".to_string(); 101]);

        assert_eq!(
            order_list_request(&credentials(), &request)
                .expect("UUID filters take precedence")
                .target(),
            "/v1/orders?uuids[]=order-1&page=1&limit=100&order_by=desc"
        );
    }

    #[test]
    fn order_list_fixture_preserves_every_documented_provider_field() {
        // Shape reference: https://apidocs.bithumb.com/reference/%EC%A3%BC%EB%AC%B8-%EB%A6%AC%EC%8A%A4%ED%8A%B8-%EC%A1%B0%ED%9A%8C
        let orders = parse_order_list(&serde_json::json!([
            {
                "uuid": "C0101000000001799625",
                "client_order_id": "strategy-42",
                "side": "ask",
                "ord_type": "best",
                "price": "84001000",
                "state": "wait",
                "market": "KRW-BTC",
                "created_at": "2024-07-12T16:30:01+09:00",
                "volume": "0.2",
                "remaining_volume": "0.2",
                "reserved_fee": "0",
                "remaining_fee": "0",
                "paid_fee": "0",
                "locked": "0.2",
                "executed_volume": "0",
                "executed_funds": "0",
                "trades_count": 0,
                "stp_type": "cancel_taker",
                "time_in_force": "ioc"
            },
            {
                "uuid": "C0661000000000760010",
                "side": "ask",
                "ord_type": "limit",
                "price": "1055",
                "state": "done",
                "market": "KRW-GMT",
                "created_at": "2024-07-10T20:00:02+09:00",
                "volume": "16",
                "remaining_volume": "11",
                "reserved_fee": "0",
                "remaining_fee": "0",
                "paid_fee": "0.52",
                "locked": "11",
                "executed_volume": "5",
                "executed_funds": "5275",
                "trades_count": 1
            }
        ]))
        .expect("official-shaped fixture");

        assert_eq!(orders.len(), 2);
        assert_eq!(orders[0].uuid, "C0101000000001799625");
        assert_eq!(orders[0].client_order_id.as_deref(), Some("strategy-42"));
        assert_eq!(orders[0].order_type, "best");
        assert_eq!(orders[0].executed_funds, Decimal::ZERO);
        assert_eq!(orders[0].stp_type.as_deref(), Some("cancel_taker"));
        assert_eq!(orders[0].time_in_force.as_deref(), Some("ioc"));
        assert_eq!(orders[1].client_order_id, None);
        assert_eq!(orders[1].paid_fee, Decimal::new(52, 2));
        assert_eq!(orders[1].executed_funds, Decimal::from(5_275));
        assert_eq!(orders[1].trades_count, 1);
    }

    #[test]
    fn order_detail_fixture_preserves_provider_only_fields_and_every_trade() {
        // Shape reference: https://apidocs.bithumb.com/reference/%EA%B0%9C%EB%B3%84-%EC%A3%BC%EB%AC%B8-%EC%A1%B0%ED%9A%8C.md
        let detail = parse_order_detail(&serde_json::json!({
            "uuid": "C0101000000001799231",
            "client_order_id": "strategy-42",
            "side": "bid",
            "ord_type": "best",
            "price": "83000000",
            "state": "cancel",
            "market": "KRW-BTC",
            "created_at": "2024-07-09T16:32:23+09:00",
            "volume": "1",
            "remaining_volume": "0",
            "reserved_fee": "207500",
            "remaining_fee": "0",
            "paid_fee": "207500",
            "locked": "0",
            "executed_volume": "1",
            "executed_funds": "83000000",
            "trades_count": 1,
            "stp_type": "provider_future_stp",
            "cancel_type": "tif_cancel",
            "canceling_uuid": "C0101000000001713005",
            "time_in_force": "ioc",
            "trades": [{
                "market": "KRW-BTC",
                "uuid": "C0101000000001713006",
                "price": "83000000",
                "volume": "1",
                "funds": "83000000",
                "side": "bid",
                "created_at": "2024-07-09T16:32:23+09:00"
            }]
        }))
        .expect("official-shaped fixture");

        assert_eq!(detail.uuid, "C0101000000001799231");
        assert_eq!(detail.client_order_id.as_deref(), Some("strategy-42"));
        assert_eq!(detail.side, "bid");
        assert_eq!(detail.order_type, "best");
        assert_eq!(detail.state, "cancel");
        assert_eq!(detail.price, Decimal::from(83_000_000));
        assert_eq!(detail.executed_funds, Decimal::from(83_000_000));
        assert_eq!(detail.reserved_fee, Decimal::from(207_500));
        assert_eq!(detail.trades_count, 1);
        assert_eq!(detail.trades[0].uuid, "C0101000000001713006");
        assert_eq!(detail.trades[0].funds, Decimal::from(83_000_000));
        assert_eq!(detail.stp_type.as_deref(), Some("provider_future_stp"));
        assert_eq!(detail.cancel_type.as_deref(), Some("tif_cancel"));
        assert_eq!(
            detail.canceling_uuid.as_deref(),
            Some("C0101000000001713005")
        );
        assert_eq!(detail.time_in_force.as_deref(), Some("ioc"));
    }

    #[test]
    fn order_detail_rejects_a_trade_count_that_does_not_match_its_trades() {
        let error = parse_order_detail(&serde_json::json!({
            "uuid": "order-1", "side": "bid", "ord_type": "limit", "price": "1",
            "state": "done", "market": "KRW-BTC", "created_at": "2024-01-01T00:00:00+09:00",
            "volume": "1", "remaining_volume": "0", "reserved_fee": "0", "remaining_fee": "0",
            "paid_fee": "0", "locked": "0", "executed_volume": "1", "executed_funds": "1",
            "trades_count": 1, "trades": []
        }))
        .expect_err("inconsistent fixture");

        assert!(matches!(error, Error::Decode { .. }));
    }

    #[test]
    fn twap_requests_match_the_documented_paths_and_signatures() {
        let request = BithumbTwapOrdersRequest::new()
            .market(btc_krw())
            .uuids(vec!["TWAP-1".to_string(), "TWAP-2".to_string()])
            .state(BithumbTwapState::Progress)
            .limit(50)
            .order_by(BithumbTwapOrderDirection::Descending)
            .cursor(Cursor::new("opaque+/=="));
        let get = twap_orders_request(&credentials(), &request).expect("signed GET");
        assert_eq!(
            get.target(),
            "/v1/twap?market=KRW-BTC&uuids[]=TWAP-1&uuids[]=TWAP-2&state=progress&limit=50&order_by=desc&next_key=opaque%2B%2F%3D%3D"
        );
        let authorization = get
            .headers
            .iter()
            .find(|(name, _)| name == "authorization")
            .map(|(_, value)| value)
            .expect("authorization");
        assert_eq!(
            payload(authorization)["query_hash"],
            sha512_hex(
                b"market=KRW-BTC&uuids[]=TWAP-1&uuids[]=TWAP-2&state=progress&limit=50&order_by=desc&next_key=opaque+/=="
            )
        );

        let create = BithumbTwapOrderRequest {
            market: btc_krw(),
            side: Side::Buy,
            volume: None,
            price: Some(Decimal::from(100_000_000)),
            duration: 3_600,
            frequency: 60,
        };
        let post = create_twap_order_request(&credentials(), &create).expect("signed POST");
        assert_eq!(post.target(), "/v1/twap");
        assert_eq!(
            post.body.as_deref(),
            Some(
                r#"{"market":"KRW-BTC","side":"bid","price":"100000000","duration":"3600","frequency":"60"}"#
            )
        );
        let delete = cancel_twap_order_request(&credentials(), "TWAP-1").expect("signed DELETE");
        assert_eq!(delete.target(), "/v1/twap?algo_order_id=TWAP-1");
    }

    #[test]
    fn twap_validation_rejects_invalid_duration_frequency_and_amount() {
        let base = BithumbTwapOrderRequest {
            market: btc_krw(),
            side: Side::Sell,
            volume: Some(Decimal::ONE),
            price: None,
            duration: 300,
            frequency: 60,
        };
        for (field, request) in [
            (
                "duration",
                BithumbTwapOrderRequest {
                    duration: 299,
                    ..base.clone()
                },
            ),
            (
                "frequency",
                BithumbTwapOrderRequest {
                    frequency: 10,
                    ..base.clone()
                },
            ),
            (
                "volume",
                BithumbTwapOrderRequest {
                    volume: Some(Decimal::ZERO),
                    ..base.clone()
                },
            ),
        ] {
            assert!(
                matches!(create_twap_order_request(&credentials(), &request), Err(Error::InvalidRequest { field: found, .. }) if found == field),
                "{field}"
            );
        }

        let non_krw = BithumbTwapOrderRequest {
            market: Market::spot(Exchange::Bithumb, "BTC", "USDT"),
            ..base
        };
        assert!(matches!(
            create_twap_order_request(&credentials(), &non_krw),
            Err(Error::InvalidRequest { field, .. }) if field == "market.quote"
        ));
    }

    #[test]
    fn twap_fixture_preserves_optional_fields_and_page_cursor() {
        let page = twap_order_page(&serde_json::json!({
            "has_next": true,
            "next_key": "opaque+/==",
            "orders": [{
                "uuid": "TWAP-1",
                "side": "ask",
                "price": "5000",
                "state": "cancel",
                "market": "KRW-XRP",
                "created_at": "2025-12-03T09:00:00+09:00",
                "volume": "1000",
                "total_order_count": 120,
                "total_trades_count": 5,
                "progress_count": 15,
                "total_executed_amount": "25000000",
                "total_executed_volume": "5000",
                "avg_trade_price": "5000.0",
                "canceled_at": "2025-12-03T09:15:00+09:00",
                "cancel_type": "user"
            }]
        }))
        .expect("valid TWAP fixture");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].state, BithumbTwapState::Cancel);
        assert_eq!(page.items[0].wallet_id, None);
        assert_eq!(page.items[0].cancel_type.as_deref(), Some("user"));
        assert_eq!(page.next.expect("cursor").as_str(), "opaque+/==");

        assert!(matches!(
            twap_order_page(
                &serde_json::json!({"has_next": false, "next_key": null, "orders": [{"uuid": "bad"}]})
            ),
            Err(Error::Decode { .. })
        ));
    }
}
