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
use crate::request::OrderRequest;
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{Balance, Market, Order, OrderStatus, OrderType, Side, Size, Timestamp};

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
        return Err(Error::invalid_request(
            "order_id",
            format!("`{name}` is empty"),
        ));
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_'))
    {
        return Err(Error::invalid_request(
            "order_id",
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
    let query = signed_query(params)?;

    Ok(HttpRequest::get(path)
        .query(query.clone())
        .header("authorization", authorization(credentials, &query)?))
}

pub(crate) fn balances_request(credentials: &BithumbCredentials) -> Result<HttpRequest> {
    signed_get(credentials, "/v1/accounts", &[])
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

pub(crate) fn cancel_order_request(
    credentials: &BithumbCredentials,
    order_id: &str,
) -> Result<HttpRequest> {
    let query = signed_query(&[("order_id", order_id.to_string())])?;

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
    if let Some(time_in_force) = request.time_in_force {
        return Err(Error::invalid_request(
            "time_in_force",
            format!(
                "bithumb's order endpoint takes no time-in-force, so {time_in_force:?} cannot be honoured"
            ),
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
    };

    Ok(PlacedOrder {
        params,
        remaining_quantity,
    })
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

pub(crate) async fn open_orders(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    market: Option<&Market>,
) -> Result<Vec<Order>> {
    parse::orders(&rest::send(http, &open_orders_request(credentials, market)?).await?)
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
) -> Result<Order> {
    // Validate the caller's market even though Bithumb cancels by identifier.
    parse::native_symbol(market)?;
    let body = rest::send(http, &cancel_order_request(credentials, order_id)?).await?;

    parse::order_ack(
        &body,
        market.clone(),
        // The acknowledgement may omit the side.
        side_of(&body),
        OrderStatus::Cancelled,
        Decimal::ZERO,
        None,
    )
}

fn side_of(body: &Value) -> Side {
    match body.get("side").and_then(Value::as_str) {
        Some("ask" | "sell" | "ASK") => Side::Sell,
        _ => Side::Buy,
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
    fn a_time_in_force_bithumb_cannot_honour_is_refused_rather_than_dropped() {
        let request =
            OrderRequest::limit(btc_krw(), Side::Buy, Size::Base(Decimal::ONE), Decimal::ONE)
                .time_in_force(TimeInForce::FillOrKill);

        assert!(matches!(
            placed_order(&request),
            Err(Error::InvalidRequest { field, .. }) if field == "time_in_force"
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
