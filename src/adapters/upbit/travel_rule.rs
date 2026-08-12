//! Upbit Korea and Singapore Travel Rule verification endpoints.

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::feature::Feature;
use crate::transport::{HttpRequest, HttpTransport};

use super::parse::{self, EXCHANGE};
use super::{UpbitCredentials, UpbitRegion, private, rest};

const VASPS_PATH: &str = "/v1/travel_rule/vasps";
const VERIFY_UUID_PATH: &str = "/v1/travel_rule/deposit/uuid";
const VERIFY_TXID_PATH: &str = "/v1/travel_rule/deposit/txid";

/// One VASP that Upbit supports for Travel Rule verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitTravelRuleVasp {
    /// Exchange name published by Upbit.
    pub vasp_name: String,
    /// Upbit's UUID for the counterparty exchange.
    pub vasp_uuid: String,
    /// Whether deposits from this VASP are available.
    pub depositable: bool,
    /// Whether withdrawals to this VASP are available.
    pub withdrawable: bool,
}

/// Result returned after an Upbit Travel Rule verification request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpbitTravelRuleVerification {
    /// Upbit's UUID for the deposit being verified.
    pub deposit_uuid: String,
    /// Deposit state returned by Upbit.
    ///
    /// Upbit currently documents `PROCESSING`, `ACCEPTED`, `CANCELLED`,
    /// `REJECTED`, `TRAVEL_RULE_SUSPECTED`, `REFUNDING`, and `REFUNDED`.
    /// This remains a string so a newly added provider state is not discarded.
    pub deposit_state: String,
    /// Account-owner verification result returned by Upbit.
    pub verification_result: String,
}

#[derive(Debug, Deserialize)]
struct RawVasp {
    vasp_name: String,
    vasp_uuid: String,
    depositable: bool,
    withdrawable: bool,
}

#[derive(Debug, Deserialize)]
struct RawVerification {
    deposit_uuid: String,
    deposit_state: String,
    verification_result: String,
}

pub(crate) async fn vasps(
    region: UpbitRegion,
    credentials: &UpbitCredentials,
    http: &HttpTransport,
) -> Result<Vec<UpbitTravelRuleVasp>> {
    ensure_supported_region(region)?;
    let request = signed_get(credentials, VASPS_PATH)?;
    let body = rest::send(http, &request).await?;
    parse::json::<Vec<RawVasp>>(&body)?
        .into_iter()
        .map(|raw| {
            Ok(UpbitTravelRuleVasp {
                vasp_name: raw.vasp_name,
                vasp_uuid: raw.vasp_uuid,
                depositable: raw.depositable,
                withdrawable: raw.withdrawable,
            })
        })
        .collect()
}

pub(crate) async fn verify_by_uuid(
    region: UpbitRegion,
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    deposit_uuid: &str,
    vasp_uuid: &str,
) -> Result<UpbitTravelRuleVerification> {
    ensure_supported_region(region)?;
    let params = vec![
        ("deposit_uuid", required("deposit_uuid", deposit_uuid)?),
        ("vasp_uuid", required("vasp_uuid", vasp_uuid)?),
    ];
    verify(credentials, http, VERIFY_UUID_PATH, params).await
}

pub(crate) async fn verify_by_txid(
    region: UpbitRegion,
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    txid: &str,
    vasp_uuid: &str,
    currency: &str,
    net_type: &str,
) -> Result<UpbitTravelRuleVerification> {
    ensure_supported_region(region)?;
    let params = vec![
        ("txid", required("txid", txid)?),
        ("vasp_uuid", required("vasp_uuid", vasp_uuid)?),
        ("currency", required("currency", currency)?),
        ("net_type", required("net_type", net_type)?),
    ];
    verify(credentials, http, VERIFY_TXID_PATH, params).await
}

async fn verify(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    path: &str,
    params: Vec<(&'static str, String)>,
) -> Result<UpbitTravelRuleVerification> {
    let query = private::json_body_query(&params);
    let request = HttpRequest::post(path)
        .json_body(private::json_body(&params)?)
        .header(
            private::AUTHORIZATION,
            private::authorization(credentials, &query)?,
        );
    let body = rest::send(http, &request).await?;
    let raw: RawVerification = parse::json(&body)?;
    Ok(UpbitTravelRuleVerification {
        deposit_uuid: raw.deposit_uuid,
        deposit_state: raw.deposit_state,
        verification_result: raw.verification_result,
    })
}

fn signed_get(credentials: &UpbitCredentials, path: &str) -> Result<HttpRequest> {
    Ok(HttpRequest::get(path).header(
        private::AUTHORIZATION,
        private::authorization(credentials, "")?,
    ))
}

pub(crate) fn ensure_supported_region(region: UpbitRegion) -> Result<()> {
    if matches!(region, UpbitRegion::Korea | UpbitRegion::Singapore) {
        Ok(())
    } else {
        Err(Error::unsupported(
            Feature::TravelRule,
            EXCHANGE,
            "Upbit Travel Rule APIs are available only in the Korea and Singapore regions",
        ))
    }
}

fn required(field: &'static str, value: &str) -> Result<String> {
    if value.trim().is_empty() {
        return Err(Error::invalid_request(field, "must not be empty"));
    }
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use serde_json::Value;
    use sha2::{Digest, Sha512};

    fn credentials() -> UpbitCredentials {
        UpbitCredentials {
            access_key: "test-access".to_string(),
            secret_key: "test-secret".to_string(),
        }
    }

    fn claims(request: &HttpRequest) -> Value {
        let token = request
            .headers
            .iter()
            .find(|(name, _)| name == private::AUTHORIZATION)
            .map(|(_, value)| value.trim_start_matches("Bearer "))
            .expect("request is authorized");
        let payload = token.split('.').nth(1).expect("JWT payload");
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .expect("base64url claims");
        serde_json::from_slice(&bytes).expect("JSON claims")
    }

    #[test]
    fn vasp_list_request_uses_region_independent_path_and_no_query_hash() {
        let request = signed_get(&credentials(), VASPS_PATH).expect("signed VASP request");

        assert_eq!(request.method, crate::transport::HttpMethod::Get);
        assert_eq!(request.target(), "/v1/travel_rule/vasps");
        assert_eq!(claims(&request)["query_hash"], Value::Null);
    }

    #[test]
    fn uuid_verification_body_and_signature_match_official_fixture_shape() {
        let params = vec![
            (
                "deposit_uuid",
                "5b871d34-fe38-4025-8f5c-9b22028f85d3".to_string(),
            ),
            (
                "vasp_uuid",
                "8d4fe968-82b2-42e5-822f-3840a245f802".to_string(),
            ),
        ];
        let query = private::json_body_query(&params);
        let request = HttpRequest::post(VERIFY_UUID_PATH)
            .json_body(private::json_body(&params).expect("JSON body"))
            .header(
                private::AUTHORIZATION,
                private::authorization(&credentials(), &query).expect("JWT"),
            );

        assert_eq!(request.target(), "/v1/travel_rule/deposit/uuid");
        assert_eq!(
            request.body.as_deref(),
            Some(
                r#"{"deposit_uuid":"5b871d34-fe38-4025-8f5c-9b22028f85d3","vasp_uuid":"8d4fe968-82b2-42e5-822f-3840a245f802"}"#
            )
        );
        assert_eq!(
            claims(&request)["query_hash"],
            hex::encode(Sha512::digest(query.as_bytes()))
        );
    }

    #[test]
    fn txid_verification_keeps_all_required_fields_in_signed_order() {
        let params = vec![
            ("txid", "tx-123".to_string()),
            ("vasp_uuid", "vasp-123".to_string()),
            ("currency", "ETH".to_string()),
            ("net_type", "ETH".to_string()),
        ];
        let body = private::json_body(&params).expect("JSON body");
        assert_eq!(
            body,
            r#"{"txid":"tx-123","vasp_uuid":"vasp-123","currency":"ETH","net_type":"ETH"}"#
        );
        assert_eq!(
            private::json_body_query(&params),
            "txid=tx-123&vasp_uuid=vasp-123&currency=ETH&net_type=ETH"
        );
    }

    #[test]
    fn txid_keeps_documented_reserved_characters_in_the_body_hash() {
        let params = vec![
            ("txid", "a/b==".to_string()),
            ("vasp_uuid", "vasp-123".to_string()),
            ("currency", "ETH".to_string()),
            ("net_type", "ETH".to_string()),
        ];

        assert_eq!(
            private::json_body_query(&params),
            "txid=a/b==&vasp_uuid=vasp-123&currency=ETH&net_type=ETH"
        );
        assert!(private::json_body(&params).is_ok());
    }

    #[test]
    fn official_response_shapes_preserve_provider_strings() {
        let vasps: Vec<RawVasp> = parse::json(
            r#"[{"vasp_name":"Upbit Korea","vasp_uuid":"00000000-0000-0000-0000-000000000000","depositable":true,"withdrawable":false}]"#,
        )
        .expect("official VASP fixture");
        assert_eq!(vasps[0].vasp_name, "Upbit Korea");
        assert!(vasps[0].depositable);
        assert!(!vasps[0].withdrawable);

        let result: RawVerification = parse::json(
            r#"{"deposit_uuid":"9f432943-54e0-40b7-825f-b6fec8b42b79","verification_result":"verified","deposit_state":"TRAVEL_RULE_SUSPECTED"}"#,
        )
        .expect("official verification fixture");
        assert_eq!(result.deposit_state, "TRAVEL_RULE_SUSPECTED");
        assert_eq!(result.verification_result, "verified");
    }

    #[test]
    fn required_fields_reject_empty_values_before_signing() {
        assert!(matches!(
            required("txid", "  "),
            Err(Error::InvalidRequest { field, .. }) if field == "txid"
        ));
        assert!(ensure_supported_region(UpbitRegion::Korea).is_ok());
        assert!(ensure_supported_region(UpbitRegion::Singapore).is_ok());
        assert!(ensure_supported_region(UpbitRegion::Indonesia).is_err());
    }
}
