//! Upbit Korea pocket-management endpoints.

use chrono::DateTime;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::adapters::inclusive_millis_at_or_after;
use crate::error::{Error, Result};
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::Timestamp;

use super::parse;
use super::{
    UpbitCredentials, UpbitPocket, UpbitPocketApiKey, UpbitPocketApiKeyGroup,
    UpbitPocketApiKeysRequest, UpbitPocketBalance, UpbitPocketTransfer, UpbitPocketTransferQuery,
    UpbitPocketTransferRequest, UpbitPocketUniversalTransferRequest, private, rest,
};

const POCKETS_PATH: &str = "/v1/pockets";
const POCKET_API_KEYS_PATH: &str = "/v1/pockets/api_keys";
const POCKET_BALANCES_PATH: &str = "/v1/pockets/assets";
const UNIVERSAL_TRANSFERS_PATH: &str = "/v1/pockets/universal_transfers";
const SUB_POCKET_TRANSFERS_PATH: &str = "/v1/pockets/transfers";
const MAX_FILTER_VALUES: usize = 20;
const MAX_LIMIT: u32 = 100;
const SEVEN_DAYS_MILLIS: i64 = 7 * 24 * 60 * 60 * 1_000;
const SEVEN_DAYS_NANOS: i64 = SEVEN_DAYS_MILLIS * 1_000_000;

#[derive(Debug, Deserialize)]
struct RawPocket {
    uuid: String,
    name: String,
    #[serde(rename = "type")]
    kind: String,
}

#[derive(Debug, Deserialize)]
struct RawPocketApiKey {
    access_key: String,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    allowed_ips: Vec<String>,
    created_at: String,
    expired_at: String,
}

#[derive(Debug, Deserialize)]
struct RawPocketApiKeyGroup {
    uuid: String,
    #[serde(default)]
    keys: Vec<RawPocketApiKey>,
}

#[derive(Debug, Deserialize)]
struct RawPocketBalance {
    currency: String,
    balance: String,
    locked: String,
    avg_buy_price: String,
    avg_buy_price_modified: bool,
    unit_currency: String,
}

#[derive(Debug, Deserialize)]
struct RawPocketTransfer {
    uuid: String,
    #[serde(default)]
    identifier: Option<String>,
    #[serde(rename = "from")]
    from: String,
    to: String,
    state: String,
    currency: String,
    amount: String,
    created_at: String,
}

pub(crate) async fn list(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
) -> Result<Vec<UpbitPocket>> {
    let request = signed_get(credentials, POCKETS_PATH, &[])?;
    let body = rest::send(http, &request).await?;
    parse_pockets(&body)
}

pub(crate) async fn list_api_keys(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    filters: &UpbitPocketApiKeysRequest,
) -> Result<Vec<UpbitPocketApiKeyGroup>> {
    let request = api_keys_request(credentials, filters)?;
    let body = rest::send(http, &request).await?;
    parse_api_keys(&body)
}

pub(crate) async fn balances(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    uuid: &str,
) -> Result<Vec<UpbitPocketBalance>> {
    let pocket_uuid = pocket_uuid("uuid", uuid)?;
    let request = signed_get(credentials, POCKET_BALANCES_PATH, &[("uuid", pocket_uuid)])?;
    let body = rest::send(http, &request).await?;
    parse_balances(&body)
}

pub(crate) async fn universal_transfer(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    transfer: &UpbitPocketUniversalTransferRequest,
) -> Result<UpbitPocketTransfer> {
    let request = universal_transfer_request(credentials, transfer)?;
    let body = rest::send(http, &request).await?;
    parse_transfer(&body)
}

pub(crate) async fn sub_pocket_transfer(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    transfer: &UpbitPocketTransferRequest,
) -> Result<UpbitPocketTransfer> {
    let request = sub_pocket_transfer_request(credentials, transfer)?;
    let body = rest::send(http, &request).await?;
    parse_transfer(&body)
}

pub(crate) async fn universal_transfers(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    filters: &UpbitPocketTransferQuery,
) -> Result<Vec<UpbitPocketTransfer>> {
    let request = transfer_history_request(credentials, UNIVERSAL_TRANSFERS_PATH, filters, true)?;
    let body = rest::send(http, &request).await?;
    parse_transfers(&body)
}

pub(crate) async fn sub_pocket_transfers(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    filters: &UpbitPocketTransferQuery,
) -> Result<Vec<UpbitPocketTransfer>> {
    let request = transfer_history_request(credentials, SUB_POCKET_TRANSFERS_PATH, filters, false)?;
    let body = rest::send(http, &request).await?;
    parse_transfers(&body)
}

fn api_keys_request(
    credentials: &UpbitCredentials,
    filters: &UpbitPocketApiKeysRequest,
) -> Result<HttpRequest> {
    let mut params = Vec::with_capacity(filters.uuids.len() + 1);
    for value in &filters.uuids {
        params.push(("uuids[]", pocket_uuid("uuids", value)?));
    }
    params.push(("include_expired", filters.include_expired.to_string()));
    signed_get(credentials, POCKET_API_KEYS_PATH, &params)
}

fn universal_transfer_request(
    credentials: &UpbitCredentials,
    transfer: &UpbitPocketUniversalTransferRequest,
) -> Result<HttpRequest> {
    let to = pocket_uuid("to", &transfer.to)?;
    let mut params = Vec::with_capacity(5);
    if let Some(from) = transfer.from.as_deref() {
        let from = pocket_uuid("from", from)?;
        if from == to {
            return Err(Error::invalid_request(
                "to",
                "must differ from the source pocket",
            ));
        }
        params.push(("from", from));
    }
    params.push(("to", to));
    params.push(("currency", currency(&transfer.currency)?));
    params.push(("amount", positive_amount(&transfer.amount)?));
    if let Some(value) = transfer.identifier.as_deref() {
        params.push(("identifier", identifier(value)?));
    }
    signed_json_post(credentials, UNIVERSAL_TRANSFERS_PATH, &params)
}

fn sub_pocket_transfer_request(
    credentials: &UpbitCredentials,
    transfer: &UpbitPocketTransferRequest,
) -> Result<HttpRequest> {
    let mut params = vec![
        ("to", pocket_uuid("to", &transfer.to)?),
        ("currency", currency(&transfer.currency)?),
        ("amount", positive_amount(&transfer.amount)?),
    ];
    if let Some(value) = transfer.identifier.as_deref() {
        params.push(("identifier", identifier(value)?));
    }
    signed_json_post(credentials, SUB_POCKET_TRANSFERS_PATH, &params)
}

fn transfer_history_request(
    credentials: &UpbitCredentials,
    path: &'static str,
    filters: &UpbitPocketTransferQuery,
    universal: bool,
) -> Result<HttpRequest> {
    let mut params = Vec::new();
    if universal {
        if filters.direction.is_some() {
            return Err(Error::invalid_request(
                "direction",
                "is only accepted by the sub-pocket transfer history endpoint",
            ));
        }
        if let Some(from) = filters.from.as_deref() {
            params.push(("from", pocket_uuid("from", from)?));
        }
        if let Some(to) = filters.to.as_deref() {
            params.push(("to", pocket_uuid("to", to)?));
        }
    } else {
        if filters.from.is_some() {
            return Err(Error::invalid_request(
                "from",
                "is only accepted by the universal transfer history endpoint",
            ));
        }
        if filters.to.is_some() {
            return Err(Error::invalid_request(
                "to",
                "is only accepted by the universal transfer history endpoint",
            ));
        }
        if let Some(direction) = filters.direction {
            params.push(("direction", direction.wire_name().to_string()));
        }
    }

    for state in &filters.states {
        params.push(("states[]", state.wire_name().to_string()));
    }
    validate_filter_count("uuids", &filters.uuids)?;
    for value in &filters.uuids {
        params.push(("uuids[]", pocket_uuid("uuids", value)?));
    }
    validate_filter_count("identifiers", &filters.identifiers)?;
    for value in &filters.identifiers {
        params.push(("identifiers[]", identifier(value)?));
    }

    if let (Some(start), Some(end)) = (filters.start_time, filters.end_time) {
        let width = end
            .as_nanos()
            .checked_sub(start.as_nanos())
            .ok_or_else(|| {
                Error::invalid_request("end_time", "must not be earlier than start_time")
            })?;
        if width < 0 {
            return Err(Error::invalid_request(
                "end_time",
                "must not be earlier than start_time",
            ));
        }
        if width > SEVEN_DAYS_NANOS {
            return Err(Error::invalid_request(
                "end_time",
                "pocket transfer history windows cannot exceed seven days",
            ));
        }
    }
    let start = filters.start_time.map(inclusive_millis_at_or_after);
    let end = filters.end_time.map(inclusive_millis_at_or_before);
    if let (Some(start), Some(end)) = (start, end)
        && end < start
    {
        return Err(Error::invalid_request(
            "end_time",
            "does not include a whole millisecond after start_time",
        ));
    }
    if let Some(start) = start {
        params.push(("start_time", start.to_string()));
    }
    if let Some(end) = end {
        params.push(("end_time", end.to_string()));
    }
    if let Some(value) = filters.currency.as_deref() {
        params.push(("currency", currency(value)?));
    }
    if let Some(limit) = filters.limit {
        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::invalid_request(
                "limit",
                format!("must be from 1 through {MAX_LIMIT}, not {limit}"),
            ));
        }
        params.push(("limit", limit.to_string()));
    }
    if let Some(order_by) = filters.order_by {
        params.push(("order_by", order_by.wire_name().to_string()));
    }
    signed_get(credentials, path, &params)
}

fn signed_get(
    credentials: &UpbitCredentials,
    path: &'static str,
    params: &[(&'static str, String)],
) -> Result<HttpRequest> {
    let query = private::query(params)?;
    let request = HttpRequest::get(path).header(
        private::AUTHORIZATION,
        private::authorization(credentials, &query)?,
    );
    Ok(if query.is_empty() {
        request
    } else {
        request.query(query)
    })
}

fn signed_json_post(
    credentials: &UpbitCredentials,
    path: &'static str,
    params: &[(&'static str, String)],
) -> Result<HttpRequest> {
    let query = private::json_body_query(params);
    Ok(HttpRequest::post(path)
        .json_body(private::json_body(params)?)
        .header(
            private::AUTHORIZATION,
            private::authorization(credentials, &query)?,
        ))
}

fn parse_pockets(body: &str) -> Result<Vec<UpbitPocket>> {
    parse::json::<Vec<RawPocket>>(body)?
        .into_iter()
        .map(|raw| {
            Ok(UpbitPocket {
                uuid: required_text("uuid", raw.uuid)?,
                name: required_text("name", raw.name)?,
                kind: required_text("type", raw.kind)?,
            })
        })
        .collect()
}

fn parse_api_keys(body: &str) -> Result<Vec<UpbitPocketApiKeyGroup>> {
    parse::json::<Vec<RawPocketApiKeyGroup>>(body)?
        .into_iter()
        .map(|raw| {
            let keys = raw
                .keys
                .into_iter()
                .map(|key| {
                    Ok(UpbitPocketApiKey {
                        access_key: required_text("access_key", key.access_key)?,
                        permissions: key.permissions,
                        allowed_ips: key.allowed_ips,
                        created_at: timestamp("created_at", key.created_at)?,
                        expired_at: timestamp("expired_at", key.expired_at)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(UpbitPocketApiKeyGroup {
                uuid: required_text("uuid", raw.uuid)?,
                keys,
            })
        })
        .collect()
}

fn parse_balances(body: &str) -> Result<Vec<UpbitPocketBalance>> {
    parse::json::<Vec<RawPocketBalance>>(body)?
        .into_iter()
        .map(|raw| {
            Ok(UpbitPocketBalance {
                currency: currency(&raw.currency).map_err(|_| {
                    Error::decode(format!(
                        "`currency` is not an Upbit asset code: {}",
                        raw.currency
                    ))
                })?,
                balance: parse::decimal_text(&raw.balance, "balance")?,
                locked: parse::decimal_text(&raw.locked, "locked")?,
                avg_buy_price: parse::decimal_text(&raw.avg_buy_price, "avg_buy_price")?,
                avg_buy_price_modified: raw.avg_buy_price_modified,
                unit_currency: currency(&raw.unit_currency).map_err(|_| {
                    Error::decode(format!(
                        "`unit_currency` is not an Upbit asset code: {}",
                        raw.unit_currency
                    ))
                })?,
            })
        })
        .collect()
}

fn parse_transfers(body: &str) -> Result<Vec<UpbitPocketTransfer>> {
    parse::json::<Vec<RawPocketTransfer>>(body)?
        .into_iter()
        .map(raw_transfer)
        .collect()
}

fn parse_transfer(body: &str) -> Result<UpbitPocketTransfer> {
    raw_transfer(parse::json::<RawPocketTransfer>(body)?)
}

fn raw_transfer(raw: RawPocketTransfer) -> Result<UpbitPocketTransfer> {
    Ok(UpbitPocketTransfer {
        uuid: required_text("uuid", raw.uuid)?,
        identifier: raw.identifier,
        from: required_text("from", raw.from)?,
        to: required_text("to", raw.to)?,
        state: required_text("state", raw.state)?,
        currency: currency(&raw.currency).map_err(|_| {
            Error::decode(format!(
                "`currency` is not an Upbit asset code: {}",
                raw.currency
            ))
        })?,
        amount: parse::decimal_text(&raw.amount, "amount")?,
        created_at: timestamp("created_at", raw.created_at)?,
    })
}

fn pocket_uuid(field: &'static str, value: &str) -> Result<String> {
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~'))
    {
        return Err(Error::invalid_request(
            field,
            "must be a non-empty URL-safe Upbit pocket UUID",
        ));
    }
    Ok(value.to_string())
}

fn identifier(value: &str) -> Result<String> {
    if !(1..=64).contains(&value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_'))
    {
        return Err(Error::invalid_request(
            "identifier",
            "must contain 1 through 64 ASCII letters, digits, hyphens, underscores, or dots",
        ));
    }
    Ok(value.to_string())
}

fn currency(value: &str) -> Result<String> {
    let value = value.trim().to_ascii_uppercase();
    if value.is_empty()
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    {
        return Err(Error::invalid_request(
            "currency",
            "must be a non-empty Upbit asset code",
        ));
    }
    Ok(value)
}

fn positive_amount(value: &Decimal) -> Result<String> {
    if *value <= Decimal::ZERO {
        return Err(Error::invalid_request(
            "amount",
            "must be greater than zero",
        ));
    }
    Ok(value.to_string())
}

fn validate_filter_count(field: &'static str, values: &[String]) -> Result<()> {
    if values.len() > MAX_FILTER_VALUES {
        return Err(Error::invalid_request(
            field,
            format!("accepts at most {MAX_FILTER_VALUES} values"),
        ));
    }
    Ok(())
}

/// Largest whole millisecond that does not exceed an inclusive timestamp.
fn inclusive_millis_at_or_before(end: Timestamp) -> i64 {
    end.as_nanos().div_euclid(1_000_000)
}

fn timestamp(field: &'static str, value: String) -> Result<Timestamp> {
    DateTime::parse_from_rfc3339(&value)
        .ok()
        .and_then(|value| value.timestamp_nanos_opt())
        .map(Timestamp::from_nanos)
        .ok_or_else(|| Error::decode(format!("`{field}` is not RFC 3339: {value}")))
}

fn required_text(field: &'static str, value: String) -> Result<String> {
    if value.trim().is_empty() {
        Err(Error::decode(format!("`{field}` is missing or empty")))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use base64::Engine as _;
    use serde_json::Value;
    use sha2::{Digest, Sha512};

    use super::*;
    use crate::adapters::upbit::{
        UpbitPocketTransferDirection, UpbitPocketTransferOrder, UpbitPocketTransferState,
    };

    fn credentials() -> UpbitCredentials {
        UpbitCredentials {
            access_key: "access".to_string(),
            secret_key: "secret".to_string(),
        }
    }

    fn claims(request: &HttpRequest) -> Value {
        let token = request
            .headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(private::AUTHORIZATION))
            .expect("authorization header")
            .1
            .strip_prefix("Bearer ")
            .expect("bearer token");
        let payload = token.split('.').nth(1).expect("JWT payload");
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .expect("base64 payload");
        serde_json::from_slice(&bytes).expect("JSON payload")
    }

    #[test]
    fn request_builders_preserve_array_order_and_body_signature() {
        let credentials = credentials();
        let keys = api_keys_request(
            &credentials,
            &UpbitPocketApiKeysRequest::new()
                .uuids(vec!["pocket-1".to_string(), "pocket-2".to_string()])
                .include_expired(),
        )
        .expect("API-key request");
        assert_eq!(
            keys.target(),
            "/v1/pockets/api_keys?uuids[]=pocket-1&uuids[]=pocket-2&include_expired=true"
        );
        let expected_key_query = "uuids[]=pocket-1&uuids[]=pocket-2&include_expired=true";
        assert_eq!(
            claims(&keys)["query_hash"],
            hex::encode(Sha512::digest(expected_key_query.as_bytes()))
        );

        let query = UpbitPocketTransferQuery::new()
            .from("pocket-1")
            .to("pocket-2")
            .states(vec![
                UpbitPocketTransferState::Done,
                UpbitPocketTransferState::Failed,
            ])
            .uuids(vec!["transfer-1".to_string(), "transfer-2".to_string()])
            .identifiers(vec!["transfer.1".to_string(), "transfer_2".to_string()])
            .start_time(Timestamp::from_millis(1_000))
            .end_time(Timestamp::from_millis(1_000 + SEVEN_DAYS_MILLIS))
            .currency("xrp")
            .limit(20)
            .order_by(UpbitPocketTransferOrder::Ascending);
        let history =
            transfer_history_request(&credentials, UNIVERSAL_TRANSFERS_PATH, &query, true)
                .expect("history request");
        let expected_history = "from=pocket-1&to=pocket-2&states[]=done&states[]=failed&uuids[]=transfer-1&uuids[]=transfer-2&identifiers[]=transfer.1&identifiers[]=transfer_2&start_time=1000&end_time=604801000&currency=XRP&limit=20&order_by=asc";
        assert_eq!(
            history.target(),
            format!("{UNIVERSAL_TRANSFERS_PATH}?{expected_history}")
        );
        assert_eq!(
            claims(&history)["query_hash"],
            hex::encode(Sha512::digest(expected_history.as_bytes()))
        );

        let universal = universal_transfer_request(
            &credentials,
            &UpbitPocketUniversalTransferRequest::new("pocket-2", "xrp", Decimal::new(125, 2))
                .from("pocket-1")
                .identifier("transfer_1"),
        )
        .expect("universal transfer request");
        let expected_body = "{\"from\":\"pocket-1\",\"to\":\"pocket-2\",\"currency\":\"XRP\",\"amount\":\"1.25\",\"identifier\":\"transfer_1\"}";
        assert_eq!(universal.body.as_deref(), Some(expected_body));
        let expected_body_query =
            "from=pocket-1&to=pocket-2&currency=XRP&amount=1.25&identifier=transfer_1";
        assert_eq!(
            claims(&universal)["query_hash"],
            hex::encode(Sha512::digest(expected_body_query.as_bytes()))
        );

        let sub = sub_pocket_transfer_request(
            &credentials,
            &UpbitPocketTransferRequest::new("pocket-2", "xrp", Decimal::new(125, 2))
                .identifier("transfer_1"),
        )
        .expect("sub-pocket transfer request");
        assert_eq!(
            sub.body.as_deref(),
            Some(
                "{\"to\":\"pocket-2\",\"currency\":\"XRP\",\"amount\":\"1.25\",\"identifier\":\"transfer_1\"}"
            )
        );
    }

    #[test]
    fn parses_pocket_fixtures() {
        let pockets = parse_pockets(
            r#"[{"uuid":"main-1","name":"Main","type":"main"},{"uuid":"sub-1","name":"Trading","type":"user_spot_trading"}]"#,
        )
        .expect("pocket fixture");
        assert_eq!(pockets[1].kind, "user_spot_trading");

        let keys = parse_api_keys(
            r#"[{"uuid":"sub-1","keys":[{"access_key":"access-1","permissions":["transfer"],"allowed_ips":["127.0.0.1"],"created_at":"2026-08-01T00:00:00+00:00","expired_at":"2027-08-01T00:00:00+00:00"}]}]"#,
        )
        .expect("API-key fixture");
        assert_eq!(keys[0].keys[0].permissions, ["transfer"]);

        let balances = parse_balances(
            r#"[{"currency":"XRP","balance":"12.3","locked":"0","avg_buy_price":"2.4","avg_buy_price_modified":false,"unit_currency":"KRW"}]"#,
        )
        .expect("balance fixture");
        assert_eq!(balances[0].balance, Decimal::new(123, 1));

        let transfer = parse_transfer(
            r#"{"uuid":"transfer-1","identifier":null,"from":"main-1","to":"sub-1","state":"done","currency":"XRP","amount":"1.25","created_at":"2026-08-01T00:00:00+00:00"}"#,
        )
        .expect("transfer fixture");
        assert_eq!(transfer.identifier, None);
        assert_eq!(transfer.amount, Decimal::new(125, 2));
    }

    #[test]
    fn validates_documented_pocket_constraints_before_signing() {
        let credentials = credentials();
        let error = universal_transfer_request(
            &credentials,
            &UpbitPocketUniversalTransferRequest::new("pocket-2", "XRP", Decimal::ZERO),
        )
        .expect_err("zero amount");
        assert!(matches!(error, Error::InvalidRequest { field, .. } if field == "amount"));

        let error = universal_transfer_request(
            &credentials,
            &UpbitPocketUniversalTransferRequest::new("pocket-1", "XRP", Decimal::ONE)
                .from("pocket-1"),
        )
        .expect_err("same source and destination");
        assert!(matches!(error, Error::InvalidRequest { field, .. } if field == "to"));

        let error = sub_pocket_transfer_request(
            &credentials,
            &UpbitPocketTransferRequest::new("", "XRP", Decimal::ONE),
        )
        .expect_err("OpenAPI-required destination");
        assert!(matches!(error, Error::InvalidRequest { field, .. } if field == "to"));

        let error = sub_pocket_transfer_request(
            &credentials,
            &UpbitPocketTransferRequest::new("pocket-2", "XRP", Decimal::ONE)
                .identifier("not~allowed"),
        )
        .expect_err("identifier alphabet");
        assert!(matches!(error, Error::InvalidRequest { field, .. } if field == "identifier"));

        let error = transfer_history_request(
            &credentials,
            UNIVERSAL_TRANSFERS_PATH,
            &UpbitPocketTransferQuery::new().uuids(vec!["id".to_string(); 21]),
            true,
        )
        .expect_err("UUID count");
        assert!(matches!(error, Error::InvalidRequest { field, .. } if field == "uuids"));

        let error = transfer_history_request(
            &credentials,
            UNIVERSAL_TRANSFERS_PATH,
            &UpbitPocketTransferQuery::new()
                .start_time(Timestamp::from_millis(1_000))
                .end_time(Timestamp::from_millis(1_000 + SEVEN_DAYS_MILLIS + 1)),
            true,
        )
        .expect_err("seven-day window");
        assert!(matches!(error, Error::InvalidRequest { field, .. } if field == "end_time"));

        let error = transfer_history_request(
            &credentials,
            UNIVERSAL_TRANSFERS_PATH,
            &UpbitPocketTransferQuery::new().direction(UpbitPocketTransferDirection::All),
            true,
        )
        .expect_err("universal direction");
        assert!(matches!(error, Error::InvalidRequest { field, .. } if field == "direction"));

        let error = transfer_history_request(
            &credentials,
            SUB_POCKET_TRANSFERS_PATH,
            &UpbitPocketTransferQuery::new().from("pocket-1"),
            false,
        )
        .expect_err("sub-pocket source filter");
        assert!(matches!(error, Error::InvalidRequest { field, .. } if field == "from"));
    }
}
