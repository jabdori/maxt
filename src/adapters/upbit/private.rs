//! Upbit's authenticated REST and WebSocket API.
//!
//! Authentication uses an HS256 JWT. Parameterized calls include the SHA-512
//! hash of the exact query string in the token claims.

use std::future::Future;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use rust_decimal::Decimal;
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256, Sha512};

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::request::{OrderHistoryRequest, OrderIdKind, OrderLookupRequest, OrderRequest};
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{
    AccountEvent, Balance, Market, Order, OrderType, Page, Side, Size, TimeInForce,
};

use super::parse::{self, EXCHANGE};
use super::{UpbitCredentials, rest, stream};

/// Authentication header used by REST and private WebSocket handshakes.
pub(crate) const AUTHORIZATION: &str = "Authorization";

const BALANCES_PATH: &str = "/v1/accounts";
const OPEN_ORDERS_PATH: &str = "/v1/orders/open";
const ORDERS_BY_IDS_PATH: &str = "/v1/orders/uuids";
const ORDER_HISTORY_PATH: &str = "/v1/orders/closed";
const PLACE_ORDER_PATH: &str = "/v1/orders";
const ORDER_PATH: &str = "/v1/order";

/// Query-hash algorithm declared in the JWT claims.
const QUERY_HASH_ALG: &str = "SHA512";

/// Upbit serves at most this many open orders per call.
const MAX_OPEN_ORDER_COUNT: u32 = 100;

/// Maximum pages read before pagination is treated as stalled.
const MAX_OPEN_ORDER_PAGES: u32 = 100;

// ---------------------------------------------------------------------------
// Signing
// ---------------------------------------------------------------------------

/// What Upbit reads out of the token.
#[derive(Debug, Serialize)]
struct Claims<'a> {
    access_key: &'a str,
    nonce: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_hash_alg: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_hash: Option<String>,
}

/// Signs one request with a caller-provided nonce.
///
/// `query` is the exact `a=1&b=2` parameter string. An empty query omits both
/// query-hash claims.
fn token(credentials: &UpbitCredentials, nonce: &str, query: &str) -> Result<String> {
    if nonce.trim().is_empty() {
        return Err(Error::auth("an Upbit token needs a nonce"));
    }

    let query_hash = (!query.is_empty()).then(|| hex::encode(Sha512::digest(query.as_bytes())));
    let claims = Claims {
        access_key: &credentials.access_key,
        nonce,
        query_hash_alg: query_hash.is_some().then_some(QUERY_HASH_ALG),
        query_hash,
    };

    encode_hs256(&claims, credentials.secret_key.as_bytes())
}

fn encode_hs256(claims: &impl Serialize, secret: &[u8]) -> Result<String> {
    let header = URL_SAFE_NO_PAD.encode(br#"{"typ":"JWT","alg":"HS256"}"#);
    let payload = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(claims)
            .map_err(|err| Error::auth(format!("could not sign the Upbit request: {err}")))?,
    );
    let message = format!("{header}.{payload}");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)
        .map_err(|err| Error::auth(format!("could not sign the Upbit request: {err}")))?;
    mac.update(message.as_bytes());

    Ok(format!(
        "{message}.{}",
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    ))
}

/// The `Authorization` value for one call, with a fresh nonce.
pub(crate) fn authorization(credentials: &UpbitCredentials, query: &str) -> Result<String> {
    let nonce = uuid::Uuid::new_v4().to_string();
    Ok(format!("Bearer {}", token(credentials, &nonce, query)?))
}

// ---------------------------------------------------------------------------
// Parameters
// ---------------------------------------------------------------------------

/// Builds the unencoded parameter string used for signing.
///
/// Values must contain only RFC 3986 unreserved bytes so the request target and
/// signed string have identical meaning.
fn query(params: &[(&'static str, String)]) -> Result<String> {
    for (name, value) in params {
        if !value.bytes().all(is_url_safe) {
            return Err(Error::invalid_request(
                *name,
                format!("`{value}` is not safe to send to Upbit unencoded"),
            ));
        }
    }

    Ok(params
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("&"))
}

/// Returns whether a byte is in the RFC 3986 unreserved set.
fn is_url_safe(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

/// Builds the string-valued JSON body used by the order endpoint.
///
/// Decimal parameters remain strings to preserve their exact representation.
fn json_body(params: &[(&'static str, String)]) -> Result<String> {
    let object = params
        .iter()
        .map(|(name, value)| ((*name).to_string(), Value::String(value.clone())))
        .collect::<Map<_, _>>();

    serde_json::to_string(&object)
        .map_err(|err| Error::decode(format!("could not build the Upbit order body: {err}")))
}

/// Formats a positive amount without rounding.
fn amount(value: &Decimal, field: &'static str) -> Result<String> {
    if value.is_zero() || value.is_sign_negative() {
        return Err(Error::invalid_request(
            field,
            format!("must be greater than zero, not {value}"),
        ));
    }

    Ok(value.to_string())
}

// ---------------------------------------------------------------------------
// Requests
// ---------------------------------------------------------------------------

pub(crate) fn balances_request(credentials: &UpbitCredentials) -> Result<HttpRequest> {
    Ok(HttpRequest::get(BALANCES_PATH).header(AUTHORIZATION, authorization(credentials, "")?))
}

/// One page of open orders. Pages are numbered from one.
pub(crate) fn open_orders_request(
    credentials: &UpbitCredentials,
    market: Option<&Market>,
    page: u32,
) -> Result<HttpRequest> {
    let mut params = Vec::new();
    if let Some(market) = market {
        params.push(("market", parse::native_symbol(market)?));
    }
    // Include resting (`wait`) and trigger-pending (`watch`) orders.
    params.push(("states[]", "wait".to_string()));
    params.push(("states[]", "watch".to_string()));
    params.push(("page", page.to_string()));
    params.push(("limit", MAX_OPEN_ORDER_COUNT.to_string()));

    let query = query(&params)?;
    Ok(HttpRequest::get(OPEN_ORDERS_PATH)
        .query(query.clone())
        .header(AUTHORIZATION, authorization(credentials, &query)?))
}

pub(crate) fn order_request(
    credentials: &UpbitCredentials,
    market: &Market,
    order_id: &str,
) -> Result<HttpRequest> {
    order_request_by(credentials, market, "uuid", "order_id", order_id)
}

pub(crate) fn order_by_client_id_request(
    credentials: &UpbitCredentials,
    market: &Market,
    client_id: &str,
) -> Result<HttpRequest> {
    validate_client_order_id(client_id)?;
    order_request_by(credentials, market, "identifier", "client_id", client_id)
}

pub(crate) fn orders_by_ids_request(
    credentials: &UpbitCredentials,
    request: &OrderLookupRequest,
) -> Result<HttpRequest> {
    crate::adapters::validate_order_lookup(request)?;
    let mut params = Vec::new();
    if let Some(market) = &request.market {
        params.push(("market", parse::native_symbol(market)?));
    }
    let key = match request.kind {
        OrderIdKind::Exchange => "uuids[]",
        OrderIdKind::Client => "identifiers[]",
    };
    params.extend(request.ids.iter().cloned().map(|id| (key, id)));
    params.push(("order_by", "desc".to_string()));

    let query = query(&params)?;
    Ok(HttpRequest::get(ORDERS_BY_IDS_PATH)
        .query(query.clone())
        .header(AUTHORIZATION, authorization(credentials, &query)?))
}

fn order_request_by(
    credentials: &UpbitCredentials,
    market: &Market,
    parameter: &'static str,
    field: &'static str,
    value: &str,
) -> Result<HttpRequest> {
    let query = order_identifier_query(market, parameter, field, value)?;
    Ok(HttpRequest::get(ORDER_PATH)
        .query(query.clone())
        .header(AUTHORIZATION, authorization(credentials, &query)?))
}

pub(crate) fn order_history_request(
    credentials: &UpbitCredentials,
    request: &OrderHistoryRequest,
) -> Result<HttpRequest> {
    if request.cursor.is_some() {
        return Err(Error::invalid_request(
            "cursor",
            "upbit final-order history does not publish a cursor",
        ));
    }

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

    let query = query(&params)?;
    Ok(HttpRequest::get(ORDER_HISTORY_PATH)
        .query(query.clone())
        .header(AUTHORIZATION, authorization(credentials, &query)?))
}

pub(crate) fn place_order_request(
    credentials: &UpbitCredentials,
    request: &OrderRequest,
) -> Result<HttpRequest> {
    let params = order_params(request)?;
    // The JWT hashes the query representation of the JSON parameters.
    let query = query(&params)?;

    Ok(HttpRequest::post(PLACE_ORDER_PATH)
        .json_body(json_body(&params)?)
        .header(AUTHORIZATION, authorization(credentials, &query)?))
}

pub(crate) fn cancel_order_request(
    credentials: &UpbitCredentials,
    market: &Market,
    order_id: &str,
) -> Result<HttpRequest> {
    cancel_order_request_by(credentials, market, "uuid", "order_id", order_id)
}

pub(crate) fn cancel_order_by_client_id_request(
    credentials: &UpbitCredentials,
    market: &Market,
    client_id: &str,
) -> Result<HttpRequest> {
    validate_client_order_id(client_id)?;
    cancel_order_request_by(credentials, market, "identifier", "client_id", client_id)
}

fn cancel_order_request_by(
    credentials: &UpbitCredentials,
    market: &Market,
    parameter: &'static str,
    field: &'static str,
    value: &str,
) -> Result<HttpRequest> {
    let query = order_identifier_query(market, parameter, field, value)?;
    Ok(HttpRequest::delete(ORDER_PATH)
        .query(query.clone())
        .header(AUTHORIZATION, authorization(credentials, &query)?))
}

fn order_identifier_query(
    market: &Market,
    parameter: &'static str,
    field: &'static str,
    value: &str,
) -> Result<String> {
    // Validate the caller's market even when the provider identifies the order globally.
    parse::native_symbol(market)?;
    if value.trim().is_empty() {
        return Err(Error::invalid_request(field, "must not be empty"));
    }
    query(&[(parameter, value.to_string())])
}

/// Maps an order request to Upbit parameters.
///
/// Limit orders use base size and quote price. Market buys use quote size;
/// market sells use base size. Other pairings return `InvalidRequest`.
fn order_params(request: &OrderRequest) -> Result<Vec<(&'static str, String)>> {
    if request.reduce_only {
        return Err(Error::unsupported(
            Feature::ReduceOnlyOrders,
            EXCHANGE,
            "upbit lists spot markets only, and a spot order has no position to reduce",
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

    match (&request.order_type, &request.size, request.side) {
        (OrderType::Limit, Size::Base(volume), _) => {
            let Some(price) = request.price.as_ref() else {
                return Err(Error::invalid_request(
                    "price",
                    "a limit order needs a price; build it with `OrderRequest::limit`",
                ));
            };
            params.push(("volume", amount(volume, "size")?));
            params.push(("price", amount(price, "price")?));
            params.push(("ord_type", "limit".to_string()));
        }
        (OrderType::Market, Size::Quote(funds), Side::Buy) => {
            params.push(("price", amount(funds, "size")?));
            params.push(("ord_type", "price".to_string()));
        }
        (OrderType::Market, Size::Base(volume), Side::Sell) => {
            params.push(("volume", amount(volume, "size")?));
            params.push(("ord_type", "market".to_string()));
        }
        (OrderType::Best, Size::Quote(funds), Side::Buy) => {
            if request.price.is_some() {
                return Err(Error::invalid_request(
                    "price",
                    "an upbit best buy takes its quote amount from `size`, not `price`",
                ));
            }
            params.push(("price", amount(funds, "size")?));
            params.push(("ord_type", "best".to_string()));
        }
        (OrderType::Best, Size::Base(volume), Side::Sell) => {
            if request.price.is_some() {
                return Err(Error::invalid_request(
                    "price",
                    "an upbit best sell has no caller-selected price",
                ));
            }
            params.push(("volume", amount(volume, "size")?));
            params.push(("ord_type", "best".to_string()));
        }
        (OrderType::Limit, Size::Quote(_), _) => {
            return Err(Error::invalid_request(
                "size",
                "upbit sizes a limit order in the base asset; use `Size::Base`",
            ));
        }
        (OrderType::Market, Size::Base(_), Side::Buy) => {
            return Err(Error::invalid_request(
                "size",
                "upbit sizes a market buy by the quote amount it spends; use `Size::Quote`",
            ));
        }
        (OrderType::Market, Size::Quote(_), Side::Sell) => {
            return Err(Error::invalid_request(
                "size",
                "upbit sizes a market sell by the base quantity it offers; use `Size::Base`",
            ));
        }
        (OrderType::Best, Size::Base(_), Side::Buy) => {
            return Err(Error::invalid_request(
                "size",
                "upbit sizes a best buy by the quote amount it spends; use `Size::Quote`",
            ));
        }
        (OrderType::Best, Size::Quote(_), Side::Sell) => {
            return Err(Error::invalid_request(
                "size",
                "upbit sizes a best sell by the base quantity it offers; use `Size::Base`",
            ));
        }
    }

    match request.time_in_force {
        Some(requested) => {
            if let Some(value) = time_in_force(&request.order_type, requested)? {
                params.push(("time_in_force", value.to_string()));
            }
        }
        None if matches!(&request.order_type, OrderType::Best) => {
            return Err(Error::invalid_request(
                "time_in_force",
                "an upbit best order requires immediate-or-cancel or fill-or-kill",
            ));
        }
        None => {}
    }
    if let Some(client_id) = &request.client_id {
        validate_client_order_id(client_id)?;
        params.push(("identifier", client_id.clone()));
    }

    Ok(params)
}

fn validate_client_order_id(value: &str) -> Result<()> {
    if (1..=64).contains(&value.len()) && value.bytes().all(is_url_safe) {
        return Ok(());
    }
    Err(Error::invalid_request(
        "client_id",
        "an Upbit client order id must contain 1-64 RFC 3986 unreserved ASCII bytes",
    ))
}

/// Maps time in force, omitting values implicit in the order type.
fn time_in_force(order_type: &OrderType, requested: TimeInForce) -> Result<Option<&'static str>> {
    Ok(match (order_type, requested) {
        (OrderType::Limit, TimeInForce::GoodTilCancelled) => None,
        (OrderType::Limit, TimeInForce::ImmediateOrCancel) => Some("ioc"),
        (OrderType::Limit, TimeInForce::FillOrKill) => Some("fok"),
        (OrderType::Limit, TimeInForce::PostOnly) => Some("post_only"),
        (OrderType::Best, TimeInForce::ImmediateOrCancel) => Some("ioc"),
        (OrderType::Best, TimeInForce::FillOrKill) => Some("fok"),
        (OrderType::Best, other) => {
            return Err(Error::invalid_request(
                "time_in_force",
                format!(
                    "an upbit best order requires immediate-or-cancel or fill-or-kill, not {other:?}"
                ),
            ));
        }
        // Immediate-or-cancel is implicit for Upbit market orders.
        (OrderType::Market, TimeInForce::ImmediateOrCancel) => None,
        (OrderType::Market, other) => {
            return Err(Error::invalid_request(
                "time_in_force",
                format!("an upbit market order is immediate-or-cancel and cannot be {other:?}"),
            ));
        }
    })
}

// ---------------------------------------------------------------------------
// Calls
// ---------------------------------------------------------------------------

pub(crate) async fn balances(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
) -> Result<Vec<Balance>> {
    let body = rest::send(http, &balances_request(credentials)?).await?;
    parse::json::<Vec<parse::RawBalance>>(&body)?
        .iter()
        .map(parse::balance)
        .collect()
}

/// Reads open orders until a short page is returned.
///
/// Each page contains at most [`MAX_OPEN_ORDER_COUNT`] orders. Reaching
/// [`MAX_OPEN_ORDER_PAGES`] full pages returns an exchange error.
pub(crate) async fn open_orders(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    market: Option<&Market>,
) -> Result<Vec<Order>> {
    walk_open_orders(|page| async move {
        let request = open_orders_request(credentials, market, page)?;
        let body = rest::send(http, &request).await?;
        parse::json::<Vec<parse::RawOrder>>(&body)?
            .iter()
            .map(parse::order)
            .collect()
    })
    .await
}

pub(crate) async fn order(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    market: &Market,
    order_id: &str,
) -> Result<Order> {
    let body = rest::send(http, &order_request(credentials, market, order_id)?).await?;
    checked_market(
        parse::order(&parse::json::<parse::RawOrder>(&body)?)?,
        market,
    )
}

pub(crate) async fn order_by_client_id(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    market: &Market,
    client_id: &str,
) -> Result<Order> {
    let body = rest::send(
        http,
        &order_by_client_id_request(credentials, market, client_id)?,
    )
    .await?;
    checked_market(
        parse::order(&parse::json::<parse::RawOrder>(&body)?)?,
        market,
    )
}

pub(crate) async fn orders_by_ids(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &OrderLookupRequest,
) -> Result<Vec<Order>> {
    let body = rest::send(http, &orders_by_ids_request(credentials, request)?).await?;
    parse::json::<Vec<parse::RawOrder>>(&body)?
        .iter()
        .map(parse::order)
        .collect()
}

pub(crate) async fn order_history(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &OrderHistoryRequest,
) -> Result<Page<Order>> {
    let body = rest::send(http, &order_history_request(credentials, request)?).await?;
    let items = parse::json::<Vec<parse::RawOrder>>(&body)?
        .iter()
        .map(parse::order)
        .collect::<Result<_>>()?;
    Ok(Page { items, next: None })
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

/// Implements the bounded open-order page walk.
async fn walk_open_orders<F, Fut>(read: F) -> Result<Vec<Order>>
where
    F: Fn(u32) -> Fut,
    Fut: Future<Output = Result<Vec<Order>>>,
{
    let mut orders = Vec::new();

    for page in 1..=MAX_OPEN_ORDER_PAGES {
        let mut read = read(page).await?;
        let full_page = read.len() as u32 >= MAX_OPEN_ORDER_COUNT;

        orders.append(&mut read);
        if !full_page {
            return Ok(orders);
        }
    }

    Err(Error::exchange(
        EXCHANGE,
        "open_order_pagination_stalled",
        format!(
            "upbit answered {MAX_OPEN_ORDER_PAGES} full pages of open orders without a short one, \
             so the end of the walk is nowhere in sight"
        ),
    ))
}

pub(crate) async fn place_order(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &OrderRequest,
) -> Result<Order> {
    let body = rest::send(http, &place_order_request(credentials, request)?).await?;
    parse::order(&parse::json::<parse::RawOrder>(&body)?)
}

pub(crate) async fn cancel_order(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    market: &Market,
    order_id: &str,
) -> Result<()> {
    let body = rest::send(http, &cancel_order_request(credentials, market, order_id)?).await?;
    parse::order(&parse::json::<parse::RawOrder>(&body)?).map(drop)
}

pub(crate) async fn cancel_order_by_client_id(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    market: &Market,
    client_id: &str,
) -> Result<()> {
    let body = rest::send(
        http,
        &cancel_order_by_client_id_request(credentials, market, client_id)?,
    )
    .await?;
    parse::order(&parse::json::<parse::RawOrder>(&body)?).map(drop)
}

// ---------------------------------------------------------------------------
// Private WebSocket
// ---------------------------------------------------------------------------

/// Builds an account-wide `myOrder` and `myAsset` subscription frame.
///
/// Omitting `codes` requests events across all markets.
pub(crate) fn subscribe_frame(ticket: &str) -> Result<String> {
    let payload = serde_json::json!([
        { "ticket": ticket },
        { "type": "myOrder" },
        { "type": "myAsset" },
        { "format": "DEFAULT" },
    ]);

    serde_json::to_string(&payload).map_err(|err| {
        Error::decode(format!(
            "could not build the Upbit account subscribe frame: {err}"
        ))
    })
}

/// Decodes one private WebSocket frame.
///
/// Keepalive responses produce no events. A `myAsset` frame produces one
/// balance event per included asset.
pub(crate) fn account_events(frame: &str) -> Result<Vec<AccountEvent>> {
    let object = stream::frame_object(frame)?;

    if let Some(error) = object.get("error") {
        return Err(stream::frame_error(error));
    }
    if object.get("status").and_then(Value::as_str) == Some("UP") {
        return Ok(Vec::new());
    }

    let Some(frame_type) = object.get("type").and_then(Value::as_str) else {
        return Err(Error::decode(
            "Upbit private frame carries no `type`".to_string(),
        ));
    };
    let frame_type = frame_type.to_string();
    let body = stream::reserialize(&object)?;

    match frame_type.as_str() {
        "myOrder" => Ok(vec![AccountEvent::Order(parse::stream_order(
            &parse::json(&body)?,
        )?)]),
        "myAsset" => parse::json::<parse::RawStreamAssets>(&body)?
            .assets
            .iter()
            .map(|asset| parse::stream_balance(asset).map(AccountEvent::Balance))
            .collect(),
        other => Err(Error::decode(format!(
            "unexpected Upbit private frame type `{other}`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, OrderStatus, Timestamp};
    use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
    use serde::Deserialize;

    const ACCESS_KEY: &str = "access-key-for-tests";
    const SECRET_KEY: &str = "secret-key-for-tests";

    // sha512("market=KRW-BTC&states[]=wait&states[]=watch")
    const OPEN_ORDERS_HASH: &str = "c01bbcb80094d2225c90eda65128baf7ef800471fbdeb76579856d1532cd2630\
                                    60e41ede9c52bfc926a0b46c4b7797a61e4327cda59d236f829cde4c875dfe77";
    // sha512("uuid=ac2dc2a3-fce9-40a2-a4f6-5987c25c438f")
    const CANCEL_HASH: &str = "141805aad375ff03e34208f2e55ff40d387e0dd386786537ed97f8af2dbeaab4\
                               f06c46e5f5a9aed35c7e142dfd056070adbd6fe2f220688ee29dfda2fa9fab0e";
    // sha512("market=KRW-BTC&side=bid&volume=0.01&price=100000000&ord_type=limit")
    const LIMIT_ORDER_HASH: &str = "04f10e7f849051645e088a4217a3e1f938268054df0e99b93ac74627b11f6931\
                                    e501657d777720f9d19fc2a6649e3afc4a6e4e83ccf3d0569be6bde4633750dc";

    const ORDER_ID: &str = "ac2dc2a3-fce9-40a2-a4f6-5987c25c438f";

    // Representative private order frame used only by offline parsing tests.
    const MY_ORDER: &str = r#"{
      "type": "myOrder",
      "code": "KRW-BTC",
      "uuid": "ac2dc2a3-fce9-40a2-a4f6-5987c25c438f",
      "ask_bid": "BID",
      "order_type": "limit",
      "state": "trade",
      "trade_uuid": "00000000-0000-0000-0000-000000000002",
      "price": 100000000.0,
      "avg_price": 100000000.0,
      "volume": 0.01,
      "remaining_volume": 0.0,
      "executed_volume": 0.01,
      "trades_count": 1,
      "reserved_fee": 0.0,
      "remaining_fee": 0.0,
      "paid_fee": 500.0,
      "locked": 0.0,
      "executed_funds": 1000000.0,
      "time_in_force": null,
      "trade_fee": 500.0,
      "is_maker": false,
      "trade_timestamp": 1781917323000,
      "order_timestamp": 1781917322000,
      "timestamp": 1781917323001
    }"#;

    // Representative private asset frame used only by offline parsing tests.
    const MY_ASSET: &str = r#"{
      "type": "myAsset",
      "asset_uuid": "00000000-0000-0000-0000-000000000003",
      "assets": [
        {
          "currency": "KRW",
          "balance": 1386929.37231066771348207123,
          "locked": 10329.670127489597585685
        },
        {
          "currency": "BTC",
          "balance": 0.01,
          "locked": 0.0
        }
      ],
      "asset_timestamp": 1781917323000,
      "timestamp": 1781917323001
    }"#;

    /// Decoded test representation of the JWT claims.
    #[derive(Debug, Deserialize)]
    struct DecodedClaims {
        access_key: String,
        nonce: String,
        query_hash: Option<String>,
        query_hash_alg: Option<String>,
    }

    fn credentials() -> UpbitCredentials {
        UpbitCredentials {
            access_key: ACCESS_KEY.to_string(),
            secret_key: SECRET_KEY.to_string(),
        }
    }

    fn btc_krw() -> Market {
        Market::spot(Exchange::Upbit, "BTC", "KRW")
    }

    /// Decodes a token after verifying its signature.
    fn verified_claims(token: &str, secret: &str) -> DecodedClaims {
        let mut validation = Validation::new(Algorithm::HS256);
        // `Claims` has no expiry field.
        validation.required_spec_claims.clear();
        validation.validate_exp = false;

        decode::<DecodedClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .expect("the token verifies under the key that signed it")
        .claims
    }

    /// Decodes claims from a generated authorization header.
    fn claims_of(authorization: &str) -> DecodedClaims {
        let token = authorization
            .strip_prefix("Bearer ")
            .expect("upbit takes a bearer token");
        let payload = token.split('.').nth(1).expect("a JWT has three parts");
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .expect("the claims are base64url");

        serde_json::from_slice(&bytes).expect("the claims are JSON")
    }

    fn authorization_of(request: &HttpRequest) -> String {
        request
            .headers
            .iter()
            .find(|(name, _)| name == AUTHORIZATION)
            .map(|(_, value)| value.clone())
            .expect("every private request is signed")
    }

    #[test]
    fn a_token_is_signed_with_the_secret_key_and_names_the_access_key() {
        let token = token(&credentials(), "nonce-1", "").expect("a signable request");
        let claims = verified_claims(&token, SECRET_KEY);

        assert_eq!(claims.access_key, ACCESS_KEY);
        assert_eq!(claims.nonce, "nonce-1");
    }

    #[test]
    fn a_token_signed_with_one_secret_does_not_verify_under_another() {
        let token = token(&credentials(), "nonce-1", "").expect("a signable request");
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims.clear();
        validation.validate_exp = false;

        assert!(
            decode::<DecodedClaims>(
                &token,
                &DecodingKey::from_secret(b"a-different-secret"),
                &validation
            )
            .is_err()
        );
    }

    #[test]
    fn a_parameterised_request_hashes_its_parameters_and_says_which_hash() {
        let query = "market=KRW-BTC&states[]=wait&states[]=watch";
        let token = token(&credentials(), "nonce-2", query).expect("a signable request");
        let claims = verified_claims(&token, SECRET_KEY);

        assert_eq!(claims.query_hash.as_deref(), Some(OPEN_ORDERS_HASH));
        assert_eq!(claims.query_hash_alg.as_deref(), Some("SHA512"));
    }

    #[test]
    fn a_request_without_parameters_claims_no_hash_at_all() {
        let token = token(&credentials(), "nonce-3", "").expect("a signable request");
        let claims = verified_claims(&token, SECRET_KEY);

        assert_eq!(claims.query_hash, None);
        assert_eq!(claims.query_hash_alg, None);
    }

    #[test]
    fn changing_one_character_of_the_query_changes_the_hash() {
        let signed = token(
            &credentials(),
            "nonce-4",
            "uuid=ac2dc2a3-fce9-40a2-a4f6-5987c25c438f",
        )
        .expect("a signable request");
        let tampered = token(
            &credentials(),
            "nonce-4",
            "uuid=ac2dc2a3-fce9-40a2-a4f6-5987c25c438e",
        )
        .expect("a signable request");

        assert_eq!(
            verified_claims(&signed, SECRET_KEY).query_hash.as_deref(),
            Some(CANCEL_HASH)
        );
        assert_ne!(
            verified_claims(&tampered, SECRET_KEY).query_hash.as_deref(),
            Some(CANCEL_HASH)
        );
    }

    #[test]
    fn a_token_without_a_nonce_is_never_minted() {
        assert!(matches!(
            token(&credentials(), "   ", ""),
            Err(Error::Auth { .. })
        ));
    }

    #[test]
    fn the_balances_request_is_signed_but_carries_nothing_to_hash() {
        let request = balances_request(&credentials()).expect("a signable request");

        assert_eq!(request.target(), "/v1/accounts");
        assert_eq!(claims_of(&authorization_of(&request)).query_hash, None);
    }

    #[test]
    fn open_orders_asks_for_both_live_states_and_hashes_what_it_sends() {
        let request =
            open_orders_request(&credentials(), Some(&btc_krw()), 1).expect("a signable request");

        assert_eq!(
            request.target(),
            "/v1/orders/open?market=KRW-BTC&states[]=wait&states[]=watch&page=1&limit=100"
        );
        assert_eq!(
            claims_of(&authorization_of(&request)).query_hash.as_deref(),
            Some(hex::encode(Sha512::digest(request.query.as_bytes())).as_str())
        );
    }

    #[test]
    fn open_orders_across_every_market_names_no_market() {
        let request = open_orders_request(&credentials(), None, 1).expect("a signable request");

        assert_eq!(
            request.target(),
            "/v1/orders/open?states[]=wait&states[]=watch&page=1&limit=100"
        );
    }

    #[test]
    fn each_page_of_open_orders_is_signed_over_its_own_page_number() {
        let first = open_orders_request(&credentials(), None, 1).expect("a signable request");
        let second = open_orders_request(&credentials(), None, 2).expect("a signable request");

        assert!(second.target().ends_with("&page=2&limit=100"));
        assert_ne!(
            claims_of(&authorization_of(&first)).query_hash,
            claims_of(&authorization_of(&second)).query_hash
        );
    }

    #[test]
    fn one_order_can_be_read_by_exchange_or_client_identifier() {
        let by_exchange = order_request(&credentials(), &btc_krw(), ORDER_ID)
            .expect("a signable exchange-id query");
        let by_client = order_by_client_id_request(&credentials(), &btc_krw(), "client-1")
            .expect("a signable client-id query");

        assert_eq!(by_exchange.target(), format!("/v1/order?uuid={ORDER_ID}"));
        assert_eq!(by_client.target(), "/v1/order?identifier=client-1");
    }

    #[test]
    fn multiple_orders_use_one_documented_identifier_namespace() {
        let request = OrderLookupRequest::exchange([ORDER_ID, "second-order"]).market(btc_krw());
        let request =
            orders_by_ids_request(&credentials(), &request).expect("a signable lookup request");

        assert_eq!(
            request.target(),
            format!(
                "/v1/orders/uuids?market=KRW-BTC&uuids[]={ORDER_ID}&uuids[]=second-order&order_by=desc"
            )
        );
        assert_eq!(
            claims_of(&authorization_of(&request)).query_hash.as_deref(),
            Some(hex::encode(Sha512::digest(request.query.as_bytes())).as_str())
        );

        let client = OrderLookupRequest::client(["client-1"]);
        assert_eq!(
            orders_by_ids_request(&credentials(), &client)
                .expect("a client-id lookup")
                .target(),
            "/v1/orders/uuids?identifiers[]=client-1&order_by=desc"
        );
    }

    #[test]
    fn multiple_order_lookup_rejects_an_empty_or_oversized_id_list() {
        let empty = OrderLookupRequest::exchange(Vec::<String>::new());
        let oversized = OrderLookupRequest::exchange((0..101).map(|index| index.to_string()));

        assert!(orders_by_ids_request(&credentials(), &empty).is_err());
        assert!(orders_by_ids_request(&credentials(), &oversized).is_err());
    }

    #[test]
    fn final_order_history_uses_the_documented_seven_day_endpoint() {
        let history = OrderHistoryRequest::new()
            .market(btc_krw())
            .status(OrderStatus::Filled)
            .from(Timestamp::from_millis(1_700_000_000_000))
            .to(Timestamp::from_millis(1_700_001_000_000))
            .limit(25);
        let request = order_history_request(&credentials(), &history).expect("a history request");

        assert_eq!(
            request.target(),
            "/v1/orders/closed?market=KRW-BTC&state=done&start_time=1700000000000&end_time=1700000999999&limit=25&order_by=desc"
        );
        assert_eq!(
            claims_of(&authorization_of(&request)).query_hash.as_deref(),
            Some(hex::encode(Sha512::digest(request.query.as_bytes())).as_str())
        );
    }

    #[test]
    fn upbit_history_refuses_a_cursor_or_non_final_status() {
        let cursor = OrderHistoryRequest::new().cursor(crate::Cursor::new("not-supported"));
        let open = OrderHistoryRequest::new().status(OrderStatus::Open);

        assert!(matches!(
            order_history_request(&credentials(), &cursor),
            Err(Error::InvalidRequest { field, .. }) if field == "cursor"
        ));
        assert!(matches!(
            order_history_request(&credentials(), &open),
            Err(Error::InvalidRequest { field, .. }) if field == "statuses"
        ));
    }

    #[test]
    fn a_limit_order_is_sized_in_base_and_priced_in_quote() {
        let request = place_order_request(
            &credentials(),
            &OrderRequest::limit(
                btc_krw(),
                Side::Buy,
                Size::Base(Decimal::new(1, 2)),
                Decimal::from(100_000_000),
            ),
        )
        .expect("a signable request");

        assert_eq!(request.target(), "/v1/orders");
        assert_eq!(
            request.body.as_deref(),
            Some(
                r#"{"market":"KRW-BTC","side":"bid","volume":"0.01","price":"100000000","ord_type":"limit"}"#
            )
        );
        assert_eq!(
            claims_of(&authorization_of(&request)).query_hash.as_deref(),
            Some(LIMIT_ORDER_HASH)
        );
    }

    #[test]
    fn a_market_order_is_named_after_how_upbit_sizes_it() {
        let buy = place_order_request(
            &credentials(),
            &OrderRequest::market(btc_krw(), Side::Buy, Size::Quote(Decimal::from(10_000))),
        )
        .expect("a signable request");
        let sell = place_order_request(
            &credentials(),
            &OrderRequest::market(btc_krw(), Side::Sell, Size::Base(Decimal::new(1, 2))),
        )
        .expect("a signable request");

        assert_eq!(
            buy.body.as_deref(),
            Some(r#"{"market":"KRW-BTC","side":"bid","price":"10000","ord_type":"price"}"#)
        );
        assert_eq!(
            sell.body.as_deref(),
            Some(r#"{"market":"KRW-BTC","side":"ask","volume":"0.01","ord_type":"market"}"#)
        );
    }

    #[test]
    fn best_orders_and_client_ids_reach_the_official_fields() {
        let buy = OrderRequest::best(
            btc_krw(),
            Side::Buy,
            Size::Quote(Decimal::from(10_000)),
            TimeInForce::ImmediateOrCancel,
        )
        .client_id("client-1");
        let sell = OrderRequest::best(
            btc_krw(),
            Side::Sell,
            Size::Base(Decimal::new(1, 2)),
            TimeInForce::FillOrKill,
        );

        assert_eq!(
            place_order_request(&credentials(), &buy)
                .expect("a signable best buy")
                .body
                .as_deref(),
            Some(
                r#"{"market":"KRW-BTC","side":"bid","price":"10000","ord_type":"best","time_in_force":"ioc","identifier":"client-1"}"#
            )
        );
        assert_eq!(
            place_order_request(&credentials(), &sell)
                .expect("a signable best sell")
                .body
                .as_deref(),
            Some(
                r#"{"market":"KRW-BTC","side":"ask","volume":"0.01","ord_type":"best","time_in_force":"fok"}"#
            )
        );
    }

    #[test]
    fn best_orders_require_a_policy_and_client_ids_fit_upbits_limit() {
        let mut missing_policy = OrderRequest::best(
            btc_krw(),
            Side::Sell,
            Size::Base(Decimal::ONE),
            TimeInForce::ImmediateOrCancel,
        );
        missing_policy.time_in_force = None;
        let too_long = OrderRequest::market(btc_krw(), Side::Sell, Size::Base(Decimal::ONE))
            .client_id("x".repeat(65));

        assert!(matches!(
            place_order_request(&credentials(), &missing_policy),
            Err(Error::InvalidRequest { field, .. }) if field == "time_in_force"
        ));
        assert!(matches!(
            place_order_request(&credentials(), &too_long),
            Err(Error::InvalidRequest { field, .. }) if field == "client_id"
        ));
    }

    #[test]
    fn a_size_upbit_cannot_express_is_refused_rather_than_reinterpreted() {
        let cases = [
            OrderRequest::limit(
                btc_krw(),
                Side::Buy,
                Size::Quote(Decimal::from(10_000)),
                Decimal::from(100_000_000),
            ),
            OrderRequest::market(btc_krw(), Side::Buy, Size::Base(Decimal::ONE)),
            OrderRequest::market(btc_krw(), Side::Sell, Size::Quote(Decimal::from(10_000))),
        ];

        for request in cases {
            assert!(
                matches!(
                    place_order_request(&credentials(), &request),
                    Err(Error::InvalidRequest { field, .. }) if field == "size"
                ),
                "{request:?}"
            );
        }
    }

    #[test]
    fn an_order_for_nothing_never_reaches_the_wire() {
        for size in [Size::Base(Decimal::ZERO), Size::Base(Decimal::from(-1))] {
            let request = OrderRequest::market(btc_krw(), Side::Sell, size);
            assert!(matches!(
                place_order_request(&credentials(), &request),
                Err(Error::InvalidRequest { field, .. }) if field == "size"
            ));
        }

        let free = OrderRequest::limit(
            btc_krw(),
            Side::Buy,
            Size::Base(Decimal::ONE),
            Decimal::ZERO,
        );
        assert!(matches!(
            place_order_request(&credentials(), &free),
            Err(Error::InvalidRequest { field, .. }) if field == "price"
        ));
    }

    #[test]
    fn a_reduce_only_order_is_refused_as_a_missing_feature_not_a_bad_field() {
        let request =
            OrderRequest::market(btc_krw(), Side::Sell, Size::Base(Decimal::ONE)).reduce_only();

        assert!(matches!(
            place_order_request(&credentials(), &request),
            Err(Error::Unsupported {
                feature: Feature::ReduceOnlyOrders,
                exchange: "upbit",
                ..
            })
        ));
    }

    #[test]
    fn each_time_in_force_upbit_has_reaches_the_body_under_its_own_name() {
        let cases = [
            (TimeInForce::ImmediateOrCancel, Some("ioc")),
            (TimeInForce::FillOrKill, Some("fok")),
            (TimeInForce::PostOnly, Some("post_only")),
            (TimeInForce::GoodTilCancelled, None),
        ];

        for (requested, expected) in cases {
            assert_eq!(
                time_in_force(&OrderType::Limit, requested).expect("a limit order takes it"),
                expected,
                "{requested:?}"
            );
        }
    }

    #[test]
    fn a_market_order_takes_no_time_in_force_but_the_one_it_already_is() {
        assert_eq!(
            time_in_force(&OrderType::Market, TimeInForce::ImmediateOrCancel)
                .expect("what a market order already does"),
            None
        );

        for requested in [
            TimeInForce::GoodTilCancelled,
            TimeInForce::FillOrKill,
            TimeInForce::PostOnly,
        ] {
            assert!(
                matches!(
                    time_in_force(&OrderType::Market, requested),
                    Err(Error::InvalidRequest { field, .. }) if field == "time_in_force"
                ),
                "{requested:?}"
            );
        }
    }

    #[test]
    fn a_cancel_names_the_order_and_nothing_else() {
        let request =
            cancel_order_request(&credentials(), &btc_krw(), ORDER_ID).expect("a signable request");
        let by_client_id =
            cancel_order_by_client_id_request(&credentials(), &btc_krw(), "client-1")
                .expect("a signable request");

        assert_eq!(
            request.target(),
            "/v1/order?uuid=ac2dc2a3-fce9-40a2-a4f6-5987c25c438f"
        );
        assert_eq!(
            claims_of(&authorization_of(&request)).query_hash.as_deref(),
            Some(CANCEL_HASH)
        );
        assert_eq!(by_client_id.target(), "/v1/order?identifier=client-1");
    }

    #[test]
    fn a_cancel_without_an_order_is_a_caller_mistake() {
        assert!(matches!(
            cancel_order_request(&credentials(), &btc_krw(), "  "),
            Err(Error::InvalidRequest { field, .. }) if field == "order_id"
        ));
    }

    #[test]
    fn another_exchanges_market_never_reaches_an_upbit_account_call() {
        let elsewhere = Market::spot(Exchange::Binance, "BTC", "USDT");

        assert!(open_orders_request(&credentials(), Some(&elsewhere), 1).is_err());
        assert!(cancel_order_request(&credentials(), &elsewhere, ORDER_ID).is_err());
        assert!(
            place_order_request(
                &credentials(),
                &OrderRequest::market(elsewhere, Side::Sell, Size::Base(Decimal::ONE)),
            )
            .is_err()
        );
    }

    #[test]
    fn a_value_that_would_change_the_query_it_signs_is_refused() {
        // `&` would change the signed parameter list.
        assert!(matches!(
            query(&[("uuid", "one&market=KRW-DOGE".to_string())]),
            Err(Error::InvalidRequest { field, .. }) if field == "uuid"
        ));
        assert!(query(&[("uuid", ORDER_ID.to_string())]).is_ok());
    }

    #[test]
    fn an_authentication_failure_comes_back_as_upbits_own_verdict() {
        // Representative authentication error envelopes.
        for (name, message) in [
            ("invalid_access_key", "Invalid access key"),
            ("jwt_verification", "Failed to verify JWT token"),
            ("expired_access_key", "Expired access key"),
        ] {
            let body = format!(r#"{{"error":{{"name":"{name}","message":"{message}"}}}}"#);
            let error = parse::exchange_error(401, &body);

            assert!(
                matches!(
                    &error,
                    Error::Exchange { exchange: "upbit", code, status: Some(401), .. }
                        if code == name
                ),
                "{name}"
            );
            assert!(!error.is_retryable());
        }
    }

    #[test]
    fn the_account_subscribe_frame_asks_for_orders_and_wallet_across_every_market() {
        let frame = subscribe_frame("ticket-1").expect("a frame");
        let value: Value = serde_json::from_str(&frame).expect("valid JSON");

        assert_eq!(value[0]["ticket"], "ticket-1");
        assert_eq!(value[1]["type"], "myOrder");
        assert!(value[1].get("codes").is_none());
        assert_eq!(value[2]["type"], "myAsset");
        assert_eq!(value[3]["format"], "DEFAULT");
    }

    #[test]
    fn an_order_frame_becomes_one_order_event() {
        let events = account_events(MY_ORDER).expect("a data frame");

        let [AccountEvent::Order(order)] = events.as_slice() else {
            panic!("expected exactly one order event");
        };
        assert_eq!(order.id, ORDER_ID);
        assert_eq!(order.market, btc_krw());
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(
            order.created_at,
            Some(Timestamp::from_millis(1_781_917_322_000))
        );
    }

    #[test]
    fn a_wallet_frame_becomes_one_event_per_asset() {
        let events = account_events(MY_ASSET).expect("a data frame");

        let [AccountEvent::Balance(krw), AccountEvent::Balance(btc)] = events.as_slice() else {
            panic!("expected one event per asset");
        };
        assert_eq!(krw.asset, "KRW");
        assert_eq!(krw.available.to_string(), "1386929.37231066771348207123");
        assert_eq!(btc.asset, "BTC");
        assert_eq!(btc.available, Decimal::new(1, 2));
    }

    #[test]
    fn a_keepalive_answer_produces_no_account_events() {
        assert!(
            account_events(r#"{"status":"UP"}"#)
                .expect("a control frame")
                .is_empty()
        );
    }

    #[test]
    fn a_private_frame_maxt_cannot_place_is_reported_rather_than_dropped() {
        let error = account_events(r#"{"error":{"name":"WRONG_FORMAT","message":"Wrong Format"}}"#)
            .expect_err("an error frame");
        assert!(matches!(
            &error,
            Error::Exchange { exchange: "upbit", code, .. } if code == "WRONG_FORMAT"
        ));

        assert!(matches!(
            account_events(r#"{"type":"myTrade","code":"KRW-BTC"}"#),
            Err(Error::Decode { .. })
        ));
        assert!(matches!(
            account_events(r#"{"code":"KRW-BTC"}"#),
            Err(Error::Decode { .. })
        ));
    }

    /// Builds one placeholder order for pagination tests.
    fn resting_order() -> Order {
        Order {
            id: ORDER_ID.to_string(),
            market: Market::spot(Exchange::Upbit, "BTC", "KRW"),
            side: Side::Buy,
            status: OrderStatus::Open,
            filled_quantity: Decimal::ZERO,
            remaining_quantity: Decimal::ONE,
            price: Some(Decimal::from(100_000_000)),
            created_at: None,
        }
    }

    #[tokio::test]
    async fn a_full_page_asks_for_the_next_one_and_a_short_page_ends_the_walk() {
        let asked = std::sync::atomic::AtomicU32::new(0);

        let orders = walk_open_orders(|page| {
            asked.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            async move {
                Ok(match page {
                    1 => vec![resting_order(); MAX_OPEN_ORDER_COUNT as usize],
                    _ => vec![resting_order()],
                })
            }
        })
        .await
        .expect("a walk that reached a short page");

        assert_eq!(asked.load(std::sync::atomic::Ordering::Relaxed), 2);
        assert_eq!(orders.len(), MAX_OPEN_ORDER_COUNT as usize + 1);
    }

    #[tokio::test]
    async fn a_walk_that_never_reaches_a_short_page_gives_up_instead_of_spinning() {
        let asked = std::sync::atomic::AtomicU32::new(0);

        // A full page forever represents stalled pagination.
        let error = walk_open_orders(|_| {
            asked.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            async { Ok(vec![resting_order(); MAX_OPEN_ORDER_COUNT as usize]) }
        })
        .await
        .expect_err("a stalled walk");

        assert_eq!(
            asked.load(std::sync::atomic::Ordering::Relaxed),
            MAX_OPEN_ORDER_PAGES
        );
        assert!(
            matches!(
                &error,
                Error::Exchange { exchange: "upbit", code, .. }
                    if code == "open_order_pagination_stalled"
            ),
            "{error:?}"
        );
    }
}
