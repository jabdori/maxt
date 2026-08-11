//! Upbit's authenticated wallet and transfer endpoints.

use chrono::DateTime;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::error::{Error, Result, TransferErrorKind};
use crate::request::{
    DepositAddressRequest, TransferHistoryRequest, TransferLookupRequest, WithdrawRequest,
};
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{
    AssetNetwork, Cursor, Deposit, DepositAddress, DepositAddressEntry, DepositStatus, Exchange,
    Network, Page, TransferDestination, TravelRuleRequirement, Withdrawal, WithdrawalFee,
    WithdrawalQuote, WithdrawalStatus,
};

use super::parse::{self, EXCHANGE};
use super::{UpbitCredentials, UpbitDepositInfo, private, rest};

const WALLET_STATUS_PATH: &str = "/v1/status/wallet";
const DEPOSIT_CHANCE_PATH: &str = "/v1/deposits/chance/coin";
const DEPOSIT_ADDRESSES_PATH: &str = "/v1/deposits/coin_addresses";
const DEPOSIT_ADDRESS_PATH: &str = "/v1/deposits/coin_address";
const CREATE_DEPOSIT_ADDRESS_PATH: &str = "/v1/deposits/generate_coin_address";
const WITHDRAW_CHANCE_PATH: &str = "/v1/withdraws/chance";
const WITHDRAWAL_ADDRESSES_PATH: &str = "/v1/withdraws/coin_addresses";
const WITHDRAW_PATH: &str = "/v1/withdraws/coin";
const WITHDRAWAL_PATH: &str = "/v1/withdraw";
const CANCEL_WITHDRAWAL_PATH: &str = "/v1/withdraws/coin";
const DEPOSITS_PATH: &str = "/v1/deposits";
const DEPOSIT_PATH: &str = "/v1/deposit";
const WITHDRAWALS_PATH: &str = "/v1/withdraws";
const MAX_HISTORY_COUNT: u32 = 100;

#[derive(Debug, Deserialize)]
struct RawWalletStatus {
    currency: String,
    wallet_state: String,
    #[serde(default)]
    net_type: Option<String>,
}

#[derive(Debug, Clone)]
struct WalletStatus {
    asset: String,
    provider_id: String,
    network: Network,
    deposit_enabled: bool,
    withdrawal_enabled: bool,
}

#[derive(Debug, Deserialize)]
struct RawDepositAddress {
    currency: String,
    #[serde(default)]
    net_type: Option<String>,
    #[serde(default)]
    deposit_address: Option<String>,
    #[serde(default)]
    secondary_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawDepositChance {
    currency: String,
    #[serde(default)]
    net_type: Option<String>,
    is_deposit_possible: bool,
    #[serde(default)]
    deposit_impossible_reason: Option<String>,
    minimum_deposit_amount: Value,
    minimum_deposit_confirmations: u64,
    decimal_precision: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawDepositAddressCreation {
    Address(RawDepositAddress),
    Pending { success: bool, message: String },
}

#[derive(Debug, Deserialize)]
struct RawChance {
    currency: RawChanceCurrency,
    account: RawChanceAccount,
    withdraw_limit: RawWithdrawLimit,
}

#[derive(Debug, Deserialize)]
struct RawChanceCurrency {
    code: String,
    #[serde(default)]
    withdraw_fee: Option<Value>,
    #[serde(default)]
    wallet_support: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawChanceAccount {
    #[serde(default)]
    balance: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct RawWithdrawLimit {
    #[serde(default)]
    onetime: Option<Value>,
    #[serde(default)]
    remaining_daily: Option<Value>,
    #[serde(default)]
    minimum: Option<Value>,
    can_withdraw: bool,
}

#[derive(Debug, Clone)]
struct Chance {
    fee: Option<WithdrawalFee>,
    minimum: Option<Decimal>,
    onetime_maximum: Option<Decimal>,
    quote_maximum: Option<Decimal>,
    can_withdraw: bool,
    supports_withdrawal: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct WithdrawalAddress {
    currency: String,
    net_type: String,
    withdraw_address: String,
    #[serde(default)]
    secondary_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawTransfer {
    uuid: String,
    currency: String,
    #[serde(default)]
    net_type: Option<String>,
    #[serde(default)]
    txid: Option<String>,
    state: String,
    #[serde(default)]
    created_at: Option<String>,
    amount: Value,
    #[serde(default)]
    fee: Option<Value>,
}

pub(crate) async fn asset_networks(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    asset: &str,
) -> Result<Vec<AssetNetwork>> {
    let asset = asset_code(asset)?;
    let statuses = wallet_statuses(credentials, http).await?;
    let mut networks = Vec::new();

    for status in statuses.into_iter().filter(|status| status.asset == asset) {
        let chance = if status.withdrawal_enabled {
            Some(withdraw_chance(credentials, http, &asset, &status.provider_id).await?)
        } else {
            None
        };
        let withdrawal_enabled = status.withdrawal_enabled
            && chance
                .as_ref()
                .is_none_or(|chance| chance.supports_withdrawal);

        networks.push(AssetNetwork {
            exchange: Exchange::Upbit,
            asset: asset.clone(),
            network: status.network.clone(),
            provider_id: status.provider_id,
            deposit_enabled: status.deposit_enabled,
            withdrawal_enabled,
            withdrawal_fee: chance.as_ref().and_then(|value| value.fee.clone()),
            minimum_withdrawal: chance.as_ref().and_then(|value| value.minimum),
            maximum_withdrawal: chance.as_ref().and_then(|value| value.onetime_maximum),
            memo_required: memo_required(&status.network),
        });
    }

    Ok(networks)
}

pub(crate) async fn deposit_addresses(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
) -> Result<Vec<DepositAddressEntry>> {
    let request = signed_get(credentials, DEPOSIT_ADDRESSES_PATH, &[])?;
    let body = rest::send(http, &request).await?;
    parse_deposit_addresses(&body)
}

pub(crate) async fn deposit_info(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    asset: &str,
    network: &Network,
) -> Result<UpbitDepositInfo> {
    let asset = asset_code(asset)?;
    let provider_id = resolve_network(credentials, http, &asset, network).await?;
    let request = deposit_chance_request(credentials, &asset, &provider_id)?;
    let body = rest::send(http, &request).await?;
    parse_deposit_info(&body, &asset, &provider_id)
}

pub(crate) async fn deposit_address(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &DepositAddressRequest,
) -> Result<DepositAddress> {
    reject_deposit_address_amount(request)?;
    let asset = asset_code(&request.asset)?;
    let provider_id = resolve_network(credentials, http, &asset, &request.network).await?;
    let response = signed_get(
        credentials,
        DEPOSIT_ADDRESS_PATH,
        &[
            ("currency", asset.clone()),
            ("net_type", provider_id.clone()),
        ],
    )?;
    let body = rest::send(http, &response).await?;
    parse_deposit_address(&body, &asset, &provider_id, request.network.clone())
}

pub(crate) async fn create_deposit_address(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &DepositAddressRequest,
) -> Result<DepositAddress> {
    reject_deposit_address_amount(request)?;
    let asset = asset_code(&request.asset)?;
    let provider_id = resolve_network(credentials, http, &asset, &request.network).await?;
    let response = create_deposit_address_request(credentials, &asset, &provider_id)?;
    let body = rest::send(http, &response).await?;
    parse_created_deposit_address(&body, &asset, &provider_id, request.network.clone())
}

pub(crate) async fn prepare_withdrawal(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &WithdrawRequest,
) -> Result<WithdrawalQuote> {
    let asset = asset_code(&request.asset)?;
    let provider_id = resolve_network(credentials, http, &asset, &request.network).await?;
    let chance = withdraw_chance(credentials, http, &asset, &provider_id).await?;
    if !chance.supports_withdrawal || !chance.can_withdraw {
        return Err(Error::exchange(
            EXCHANGE,
            "withdrawal_unavailable",
            format!("{asset} withdrawals on {provider_id} are not currently allowed"),
        ));
    }
    let addresses = withdrawal_addresses(credentials, http).await?;
    let address_allowed = addresses
        .iter()
        .any(|allowed| address_matches(allowed, request, &asset, &provider_id));
    let fee = chance
        .fee
        .as_ref()
        .map(|fee| fee.for_amount(request.amount));

    Ok(WithdrawalQuote {
        fee,
        expected_receive: fee.map(|fee| request.amount - fee),
        minimum_amount: chance.minimum,
        maximum_amount: chance.quote_maximum,
        address_allowed: Some(address_allowed),
        // Upbit accepts only pre-verified withdrawal addresses. It exposes no
        // separate consent step in this operation once the address is listed.
        travel_rule: TravelRuleRequirement::NotRequired,
        expires_at: None,
    })
}

pub(crate) async fn withdraw(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &WithdrawRequest,
) -> Result<Withdrawal> {
    let asset = asset_code(&request.asset)?;
    let provider_id = resolve_network(credentials, http, &asset, &request.network).await?;
    let chance = withdraw_chance(credentials, http, &asset, &provider_id).await?;
    validate_live_withdrawal(&chance, request.amount, &asset, &provider_id)?;
    let addresses = withdrawal_addresses(credentials, http).await?;
    if !addresses
        .iter()
        .any(|allowed| address_matches(allowed, request, &asset, &provider_id))
    {
        return Err(Error::transfer(
            TransferErrorKind::AddressNotAllowed,
            "destination is not in the Upbit withdrawal-address list",
        ));
    }

    let response = withdraw_request(credentials, request, &asset, &provider_id)?;
    let body = rest::send(http, &response).await?;
    parse_withdrawal(&parse::json(&body)?)
}

pub(crate) async fn deposit(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &TransferLookupRequest,
) -> Result<Deposit> {
    let response = lookup_request(credentials, DEPOSIT_PATH, request)?;
    let body = rest::send(http, &response).await?;
    parse_deposit(&parse::json(&body)?)
}

pub(crate) async fn withdrawal(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &TransferLookupRequest,
) -> Result<Withdrawal> {
    let response = lookup_request(credentials, WITHDRAWAL_PATH, request)?;
    let body = rest::send(http, &response).await?;
    parse_withdrawal(&parse::json(&body)?)
}

pub(crate) async fn cancel_withdrawal(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    withdrawal_id: &str,
) -> Result<()> {
    let response = cancel_withdrawal_request(credentials, withdrawal_id)?;
    rest::send(http, &response).await?;
    Ok(())
}

pub(crate) async fn deposits(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &TransferHistoryRequest,
) -> Result<Page<Deposit>> {
    let (page, limit) = history_position(request, "upbit:deposits")?;
    let response = history_request(credentials, DEPOSITS_PATH, request, page, limit)?;
    let body = rest::send(http, &response).await?;
    let raw: Vec<RawTransfer> = parse::json(&body)?;
    let raw_count = raw.len();
    let mut items = raw.iter().map(parse_deposit).collect::<Result<Vec<_>>>()?;
    filter_deposits(&mut items, request.network.as_ref());

    Ok(Page {
        items,
        next: next_cursor("upbit:deposits", page, limit, raw_count),
    })
}

pub(crate) async fn withdrawals(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    request: &TransferHistoryRequest,
) -> Result<Page<Withdrawal>> {
    let (page, limit) = history_position(request, "upbit:withdrawals")?;
    let response = history_request(credentials, WITHDRAWALS_PATH, request, page, limit)?;
    let body = rest::send(http, &response).await?;
    let raw: Vec<RawTransfer> = parse::json(&body)?;
    let raw_count = raw.len();
    let mut items = raw
        .iter()
        .map(parse_withdrawal)
        .collect::<Result<Vec<_>>>()?;
    filter_withdrawals(&mut items, request.network.as_ref());

    Ok(Page {
        items,
        next: next_cursor("upbit:withdrawals", page, limit, raw_count),
    })
}

async fn wallet_statuses(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
) -> Result<Vec<WalletStatus>> {
    let request = signed_get(credentials, WALLET_STATUS_PATH, &[])?;
    let body = rest::send(http, &request).await?;
    parse_wallet_statuses(&body)
}

async fn resolve_network(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    asset: &str,
    requested: &Network,
) -> Result<String> {
    let matches = wallet_statuses(credentials, http)
        .await?
        .into_iter()
        .filter(|status| {
            status.asset == asset
                && match requested {
                    Network::Other(raw) => status.provider_id == *raw,
                    known => status.network.same_chain(known),
                }
        })
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [found] => Ok(found.provider_id.clone()),
        [] => Err(Error::invalid_request(
            "network",
            format!("Upbit publishes no {asset} network matching {requested}"),
        )),
        _ => Err(Error::invalid_request(
            "network",
            format!("more than one Upbit {asset} network matches {requested}"),
        )),
    }
}

async fn withdraw_chance(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
    asset: &str,
    provider_id: &str,
) -> Result<Chance> {
    let request = withdraw_chance_request(credentials, asset, provider_id)?;
    let body = rest::send(http, &request).await?;
    parse_chance(&body, asset)
}

async fn withdrawal_addresses(
    credentials: &UpbitCredentials,
    http: &HttpTransport,
) -> Result<Vec<WithdrawalAddress>> {
    let request = signed_get(credentials, WITHDRAWAL_ADDRESSES_PATH, &[])?;
    let body = rest::send(http, &request).await?;
    parse::json(&body)
}

fn signed_get(
    credentials: &UpbitCredentials,
    path: &str,
    params: &[(&str, String)],
) -> Result<HttpRequest> {
    let query = signed_query(params)?;
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

fn signed_delete(
    credentials: &UpbitCredentials,
    path: &str,
    params: &[(&str, String)],
) -> Result<HttpRequest> {
    let query = signed_query(params)?;
    let request = HttpRequest::delete(path).header(
        private::AUTHORIZATION,
        private::authorization(credentials, &query)?,
    );
    Ok(if query.is_empty() {
        request
    } else {
        request.query(query)
    })
}

fn signed_query(params: &[(&str, String)]) -> Result<String> {
    for (name, value) in params {
        if value.is_empty() || !value.bytes().all(is_unreserved) {
            return Err(Error::invalid_request(
                *name,
                format!("`{value}` is not a safe Upbit query value"),
            ));
        }
    }
    Ok(raw_query(params))
}

fn raw_query(params: &[(&str, String)]) -> String {
    params
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

fn withdraw_chance_request(
    credentials: &UpbitCredentials,
    asset: &str,
    provider_id: &str,
) -> Result<HttpRequest> {
    signed_get(
        credentials,
        WITHDRAW_CHANCE_PATH,
        &[
            ("currency", asset.to_string()),
            ("net_type", provider_id.to_string()),
        ],
    )
}

fn deposit_chance_request(
    credentials: &UpbitCredentials,
    asset: &str,
    provider_id: &str,
) -> Result<HttpRequest> {
    signed_get(
        credentials,
        DEPOSIT_CHANCE_PATH,
        &[
            ("currency", asset.to_string()),
            ("net_type", provider_id.to_string()),
        ],
    )
}

fn lookup_request(
    credentials: &UpbitCredentials,
    path: &str,
    request: &TransferLookupRequest,
) -> Result<HttpRequest> {
    let asset = asset_code(&request.asset)?;
    let (reference_name, reference) = request.reference()?;
    signed_get(
        credentials,
        path,
        &[("currency", asset), (reference_name, reference.to_string())],
    )
}

fn cancel_withdrawal_request(
    credentials: &UpbitCredentials,
    withdrawal_id: &str,
) -> Result<HttpRequest> {
    signed_delete(
        credentials,
        CANCEL_WITHDRAWAL_PATH,
        &[("uuid", withdrawal_id.to_string())],
    )
}

fn create_deposit_address_request(
    credentials: &UpbitCredentials,
    asset: &str,
    provider_id: &str,
) -> Result<HttpRequest> {
    let params = [
        ("currency", asset.to_string()),
        ("net_type", provider_id.to_string()),
    ];
    let query = raw_query(&params);
    let body = params
        .iter()
        .map(|(name, value)| ((*name).to_string(), Value::String(value.clone())))
        .collect::<Map<_, _>>();
    let body = serde_json::to_string(&body).map_err(|error| {
        Error::decode(format!(
            "could not build Upbit deposit-address JSON: {error}"
        ))
    })?;

    Ok(HttpRequest::post(CREATE_DEPOSIT_ADDRESS_PATH)
        .json_body(body)
        .header(
            private::AUTHORIZATION,
            private::authorization(credentials, &query)?,
        ))
}

fn reject_deposit_address_amount(request: &DepositAddressRequest) -> Result<()> {
    if request.amount.is_some() {
        return Err(Error::invalid_request(
            "amount",
            "Upbit deposit-address requests accept only an asset and network",
        ));
    }
    Ok(())
}

fn withdraw_request(
    credentials: &UpbitCredentials,
    request: &WithdrawRequest,
    asset: &str,
    provider_id: &str,
) -> Result<HttpRequest> {
    let mut params = vec![
        ("currency", asset.to_string()),
        ("net_type", provider_id.to_string()),
        ("amount", request.amount.to_string()),
        ("address", request.destination.address().to_string()),
    ];
    if let Some(memo) = request.destination.memo() {
        params.push(("secondary_address", memo.to_string()));
    }
    let transaction_type = match &request.destination {
        TransferDestination::Exchange(destination) if destination.exchange == Exchange::Upbit => {
            "internal"
        }
        _ => "default",
    };
    params.push(("transaction_type", transaction_type.to_string()));

    let query = raw_query(&params);
    let body = params
        .iter()
        .map(|(name, value)| ((*name).to_string(), Value::String(value.clone())))
        .collect::<Map<_, _>>();
    let body = serde_json::to_string(&body).map_err(|error| {
        Error::decode(format!("could not build Upbit withdrawal JSON: {error}"))
    })?;

    Ok(HttpRequest::post(WITHDRAW_PATH).json_body(body).header(
        private::AUTHORIZATION,
        private::authorization(credentials, &query)?,
    ))
}

fn history_request(
    credentials: &UpbitCredentials,
    path: &str,
    request: &TransferHistoryRequest,
    page: u32,
    limit: u32,
) -> Result<HttpRequest> {
    let mut params = Vec::new();
    if let Some(asset) = &request.asset {
        params.push(("currency", asset_code(asset)?));
    }
    params.extend([
        ("page", page.to_string()),
        ("limit", limit.to_string()),
        ("order_by", "desc".to_string()),
    ]);
    signed_get(credentials, path, &params)
}

fn history_position(request: &TransferHistoryRequest, prefix: &str) -> Result<(u32, u32)> {
    let limit = request.limit.unwrap_or(MAX_HISTORY_COUNT);
    if !(1..=MAX_HISTORY_COUNT).contains(&limit) {
        return Err(Error::invalid_request(
            "limit",
            format!("Upbit serves 1 to {MAX_HISTORY_COUNT} transfer rows per page"),
        ));
    }
    let page = match &request.cursor {
        None => 1,
        Some(cursor) => cursor
            .as_str()
            .strip_prefix(&format!("{prefix}:"))
            .and_then(|page| page.parse::<u32>().ok())
            .filter(|page| *page > 0)
            .ok_or_else(|| {
                Error::invalid_request(
                    "cursor",
                    "cursor was not issued by this Upbit history endpoint",
                )
            })?,
    };
    Ok((page, limit))
}

fn next_cursor(prefix: &str, page: u32, limit: u32, count: usize) -> Option<Cursor> {
    (count >= limit as usize).then(|| Cursor::new(format!("{prefix}:{}", page + 1)))
}

fn parse_wallet_statuses(body: &str) -> Result<Vec<WalletStatus>> {
    parse::json::<Vec<RawWalletStatus>>(body)?
        .into_iter()
        .map(|raw| {
            let provider_id = raw.net_type.ok_or_else(|| {
                Error::decode(format!(
                    "Upbit wallet status for {} carries no `net_type`",
                    raw.currency
                ))
            })?;
            let (deposit_enabled, withdrawal_enabled) = match raw.wallet_state.as_str() {
                "working" => (true, true),
                "withdraw_only" => (false, true),
                "deposit_only" => (true, false),
                "paused" | "unsupported" => (false, false),
                _ => (false, false),
            };
            Ok(WalletStatus {
                asset: raw.currency.to_ascii_uppercase(),
                network: network_from_provider(&provider_id),
                provider_id,
                deposit_enabled,
                withdrawal_enabled,
            })
        })
        .collect()
}

fn parse_deposit_address(
    body: &str,
    expected_asset: &str,
    expected_provider: &str,
    requested_network: Network,
) -> Result<DepositAddress> {
    let raw: RawDepositAddress = parse::json(body)?;
    normalize_deposit_address(raw, expected_asset, expected_provider, requested_network)
}

fn parse_deposit_addresses(body: &str) -> Result<Vec<DepositAddressEntry>> {
    parse::json::<Vec<RawDepositAddress>>(body)?
        .into_iter()
        .map(|raw| {
            let asset = raw.currency.trim().to_ascii_uppercase();
            if asset.is_empty() {
                return Err(Error::decode(
                    "Upbit returned a deposit-address entry without a currency",
                ));
            }
            let provider_network = raw.net_type;
            Ok(DepositAddressEntry {
                exchange: Exchange::Upbit,
                asset,
                network: provider_network.as_deref().map(network_from_provider),
                provider_network,
                address: raw.deposit_address,
                memo: raw.secondary_address,
            })
        })
        .collect()
}

fn parse_deposit_info(
    body: &str,
    expected_asset: &str,
    expected_provider: &str,
) -> Result<UpbitDepositInfo> {
    let raw: RawDepositChance = parse::json(body)?;
    if !raw.currency.eq_ignore_ascii_case(expected_asset) {
        return Err(Error::decode(format!(
            "Upbit returned deposit information for {} instead of {expected_asset}",
            raw.currency
        )));
    }
    if raw
        .net_type
        .as_deref()
        .is_some_and(|provider| provider != expected_provider)
    {
        return Err(Error::decode(format!(
            "Upbit returned deposit information for a different network than {expected_provider}"
        )));
    }

    let provider_network = raw.net_type;
    Ok(UpbitDepositInfo {
        asset: expected_asset.to_string(),
        network: provider_network.as_deref().map(network_from_provider),
        provider_network,
        is_deposit_possible: raw.is_deposit_possible,
        deposit_impossible_reason: raw.deposit_impossible_reason,
        minimum_deposit_amount: decimal_value(
            &raw.minimum_deposit_amount,
            "minimum_deposit_amount",
        )?,
        minimum_deposit_confirmations: raw.minimum_deposit_confirmations,
        decimal_precision: raw.decimal_precision,
    })
}

fn parse_created_deposit_address(
    body: &str,
    expected_asset: &str,
    expected_provider: &str,
    requested_network: Network,
) -> Result<DepositAddress> {
    match parse::json::<RawDepositAddressCreation>(body)? {
        RawDepositAddressCreation::Address(raw) => {
            normalize_deposit_address(raw, expected_asset, expected_provider, requested_network)
        }
        RawDepositAddressCreation::Pending { success: true, .. } => Ok(DepositAddress {
            exchange: Exchange::Upbit,
            asset: expected_asset.to_string(),
            network: requested_network,
            address: None,
            memo: None,
        }),
        RawDepositAddressCreation::Pending {
            success: false,
            message,
        } => Err(Error::exchange(
            EXCHANGE,
            "deposit_address_creation_rejected",
            message,
        )),
    }
}

fn normalize_deposit_address(
    raw: RawDepositAddress,
    expected_asset: &str,
    expected_provider: &str,
    requested_network: Network,
) -> Result<DepositAddress> {
    if !raw.currency.eq_ignore_ascii_case(expected_asset) {
        return Err(Error::decode(format!(
            "Upbit returned deposit asset {} for requested {expected_asset}",
            raw.currency
        )));
    }
    if raw
        .net_type
        .as_deref()
        .is_some_and(|provider| provider != expected_provider)
    {
        return Err(Error::decode(format!(
            "Upbit returned a different deposit network than {expected_provider}"
        )));
    }

    Ok(DepositAddress {
        exchange: Exchange::Upbit,
        asset: expected_asset.to_string(),
        network: requested_network,
        address: raw.deposit_address,
        memo: raw.secondary_address,
    })
}

fn parse_chance(body: &str, expected_asset: &str) -> Result<Chance> {
    let raw: RawChance = parse::json(body)?;
    if !raw.currency.code.eq_ignore_ascii_case(expected_asset) {
        return Err(Error::decode(format!(
            "Upbit returned withdrawal rules for {} instead of {expected_asset}",
            raw.currency.code
        )));
    }
    let fee = decimal_opt(raw.currency.withdraw_fee.as_ref(), "currency.withdraw_fee")?
        .map(WithdrawalFee::Fixed);
    let minimum = decimal_opt(
        raw.withdraw_limit.minimum.as_ref(),
        "withdraw_limit.minimum",
    )?;
    let onetime = decimal_opt(
        raw.withdraw_limit.onetime.as_ref(),
        "withdraw_limit.onetime",
    )?;
    let remaining = decimal_opt(
        raw.withdraw_limit.remaining_daily.as_ref(),
        "withdraw_limit.remaining_daily",
    )?;
    let balance = decimal_opt(raw.account.balance.as_ref(), "account.balance")?;

    Ok(Chance {
        fee,
        minimum,
        onetime_maximum: onetime,
        quote_maximum: minimum_of([onetime, remaining, balance]),
        can_withdraw: raw.withdraw_limit.can_withdraw,
        supports_withdrawal: raw
            .currency
            .wallet_support
            .iter()
            .any(|value| value == "withdraw"),
    })
}

fn decimal_opt(value: Option<&Value>, field: &str) -> Result<Option<Decimal>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(raw)) => parse::decimal_text(raw, field).map(Some),
        Some(Value::Number(raw)) => parse::decimal(raw, field).map(Some),
        Some(_) => Err(Error::decode(format!("`{field}` is not a decimal"))),
    }
}

fn minimum_of(values: [Option<Decimal>; 3]) -> Option<Decimal> {
    values.into_iter().flatten().min()
}

fn validate_live_withdrawal(
    chance: &Chance,
    amount: Decimal,
    asset: &str,
    provider_id: &str,
) -> Result<()> {
    if !chance.supports_withdrawal || !chance.can_withdraw {
        return Err(Error::exchange(
            EXCHANGE,
            "withdrawal_unavailable",
            format!("{asset} withdrawals on {provider_id} are not currently allowed"),
        ));
    }
    if chance.minimum.is_some_and(|minimum| amount < minimum)
        || chance.quote_maximum.is_some_and(|maximum| amount > maximum)
    {
        return Err(Error::transfer(
            TransferErrorKind::AmountOutOfRange,
            format!("{amount} is outside Upbit's current withdrawal bounds"),
        ));
    }
    if chance
        .fee
        .as_ref()
        .is_some_and(|fee| fee.for_amount(amount) >= amount)
    {
        return Err(Error::transfer(
            TransferErrorKind::AmountOutOfRange,
            "withdrawal fee must be smaller than the amount",
        ));
    }
    Ok(())
}

fn parse_deposit(raw: &RawTransfer) -> Result<Deposit> {
    let provider_network = raw.net_type.clone();
    Ok(Deposit {
        id: raw.uuid.clone(),
        asset: raw.currency.to_ascii_uppercase(),
        network: provider_network.as_deref().map(network_from_provider),
        provider_network,
        amount: decimal_value(&raw.amount, "amount")?,
        address: None,
        memo: None,
        status: deposit_status(&raw.state),
        provider_status: raw.state.clone(),
        tx_id: raw.txid.clone(),
        created_at: timestamp_opt(raw.created_at.as_deref())?,
    })
}

fn parse_withdrawal(raw: &RawTransfer) -> Result<Withdrawal> {
    let provider_network = raw.net_type.clone();
    Ok(Withdrawal {
        id: raw.uuid.clone(),
        asset: raw.currency.to_ascii_uppercase(),
        network: provider_network.as_deref().map(network_from_provider),
        provider_network,
        amount: decimal_value(&raw.amount, "amount")?,
        fee: decimal_opt(raw.fee.as_ref(), "fee")?,
        destination: None,
        status: withdrawal_status(&raw.state),
        provider_status: raw.state.clone(),
        tx_id: raw.txid.clone(),
        created_at: timestamp_opt(raw.created_at.as_deref())?,
    })
}

fn decimal_value(value: &Value, field: &str) -> Result<Decimal> {
    match value {
        Value::String(raw) => parse::decimal_text(raw, field),
        Value::Number(raw) => parse::decimal(raw, field),
        _ => Err(Error::decode(format!("`{field}` is not a decimal"))),
    }
}

fn timestamp_opt(raw: Option<&str>) -> Result<Option<crate::types::Timestamp>> {
    raw.map(|raw| {
        DateTime::parse_from_rfc3339(raw)
            .ok()
            .and_then(|parsed| parsed.timestamp_nanos_opt())
            .map(crate::types::Timestamp::from_nanos)
            .ok_or_else(|| Error::decode(format!("`created_at` is not RFC 3339: {raw}")))
    })
    .transpose()
}

fn deposit_status(raw: &str) -> DepositStatus {
    match raw.to_ascii_uppercase().as_str() {
        "PROCESSING" | "TRAVEL_RULE_SUSPECTED" | "REFUNDING" => DepositStatus::Pending,
        "ACCEPTED" => DepositStatus::Completed,
        "CANCELLED" | "REJECTED" | "REFUNDED" => DepositStatus::Failed,
        _ => DepositStatus::Unknown,
    }
}

fn withdrawal_status(raw: &str) -> WithdrawalStatus {
    match raw.to_ascii_uppercase().as_str() {
        "WAITING" => WithdrawalStatus::Pending,
        "PROCESSING" => WithdrawalStatus::Processing,
        "DONE" => WithdrawalStatus::Completed,
        "CANCELLED" => WithdrawalStatus::Cancelled,
        "FAILED" | "REJECTED" => WithdrawalStatus::Failed,
        _ => WithdrawalStatus::Unknown,
    }
}

fn filter_deposits(items: &mut Vec<Deposit>, requested: Option<&Network>) {
    if let Some(requested) = requested {
        items.retain(|item| {
            network_matches(
                requested,
                item.network.as_ref(),
                item.provider_network.as_deref(),
            )
        });
    }
}

fn filter_withdrawals(items: &mut Vec<Withdrawal>, requested: Option<&Network>) {
    if let Some(requested) = requested {
        items.retain(|item| {
            network_matches(
                requested,
                item.network.as_ref(),
                item.provider_network.as_deref(),
            )
        });
    }
}

fn network_matches(requested: &Network, parsed: Option<&Network>, provider: Option<&str>) -> bool {
    match requested {
        Network::Other(raw) => provider == Some(raw.as_str()),
        known => parsed.is_some_and(|parsed| parsed.same_chain(known)),
    }
}

fn address_matches(
    allowed: &WithdrawalAddress,
    request: &WithdrawRequest,
    asset: &str,
    provider_id: &str,
) -> bool {
    allowed.currency.eq_ignore_ascii_case(asset)
        && allowed.net_type == provider_id
        && allowed.withdraw_address == request.destination.address()
        && allowed.secondary_address.as_deref() == request.destination.memo()
}

fn asset_code(raw: &str) -> Result<String> {
    let asset = raw.trim().to_ascii_uppercase();
    if asset.is_empty()
        || !asset
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    {
        return Err(Error::invalid_request(
            "asset",
            format!("`{raw}` is not an Upbit asset code"),
        ));
    }
    Ok(asset)
}

fn network_from_provider(raw: &str) -> Network {
    match raw {
        "BTC" => Network::Bitcoin,
        "ETH" => Network::Ethereum,
        "ARB" | "ARBITRUM" => Network::Arbitrum,
        "BSC" | "BEP20" => Network::BnbSmartChain,
        "TRX" | "TRON" => Network::Tron,
        "SOL" => Network::Solana,
        "MATIC" | "POLYGON" => Network::Polygon,
        "BASE" => Network::Base,
        "OP" | "OPTIMISM" => Network::Optimism,
        "AVAXC" | "AVAX-C" => Network::AvalancheC,
        "XRP" => Network::XrpLedger,
        "XLM" => Network::Stellar,
        "ATOM" => Network::Cosmos,
        "APT" => Network::Aptos,
        "SUI" => Network::Sui,
        "TON" => Network::Ton,
        "NEAR" => Network::Near,
        "DOT" => Network::Polkadot,
        other => Network::Other(other.to_string()),
    }
}

fn memo_required(network: &Network) -> bool {
    matches!(
        network,
        Network::XrpLedger | Network::Stellar | Network::Cosmos | Network::Ton
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChainDestination, Timestamp};
    use sha2::{Digest, Sha512};

    fn credentials() -> UpbitCredentials {
        UpbitCredentials {
            access_key: "test-access".to_string(),
            secret_key: "test-secret".to_string(),
        }
    }

    fn withdrawal() -> WithdrawRequest {
        WithdrawRequest::new(
            "XRP",
            Network::XrpLedger,
            Decimal::new(12_345, 3),
            TransferDestination::Chain(ChainDestination {
                asset: "XRP".to_string(),
                network: Network::XrpLedger,
                address: "rDestination".to_string(),
                memo: Some("12345".to_string()),
            }),
        )
    }

    fn claims(request: &HttpRequest) -> Value {
        use base64::Engine as _;

        let token = request
            .headers
            .iter()
            .find(|(name, _)| name == private::AUTHORIZATION)
            .map(|(_, value)| value.trim_start_matches("Bearer "))
            .expect("wallet request is authorized");
        let payload = token.split('.').nth(1).expect("JWT payload");
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .expect("base64url claims");
        serde_json::from_slice(&bytes).expect("JSON claims")
    }

    #[test]
    fn official_wallet_states_are_fail_closed_and_unknown_networks_stay_provider_scoped() {
        let statuses = parse_wallet_statuses(
            r#"[
              {"currency":"BTC","wallet_state":"working","net_type":"BTC"},
              {"currency":"ETH","wallet_state":"withdraw_only","net_type":"MYSTERY"},
              {"currency":"XRP","wallet_state":"new_state","net_type":"XRP"}
            ]"#,
        )
        .expect("official wallet status fixture");

        assert_eq!(statuses[0].network, Network::Bitcoin);
        assert!(statuses[0].deposit_enabled && statuses[0].withdrawal_enabled);
        assert_eq!(statuses[1].network, Network::Other("MYSTERY".to_string()));
        assert!(!statuses[1].deposit_enabled && statuses[1].withdrawal_enabled);
        assert!(!statuses[2].deposit_enabled && !statuses[2].withdrawal_enabled);
    }

    #[test]
    fn a_pending_deposit_address_preserves_null_instead_of_inventing_an_address() {
        let address = parse_deposit_address(
            r#"{"currency":"BTC","net_type":null,"deposit_address":null,"secondary_address":null}"#,
            "BTC",
            "BTC",
            Network::Bitcoin,
        )
        .expect("official pending address fixture");

        assert_eq!(address.address, None);
        assert_eq!(address.memo, None);
        assert_eq!(address.network, Network::Bitcoin);
    }

    #[test]
    fn deposit_address_creation_preserves_upbits_pending_or_issued_response() {
        let request = create_deposit_address_request(&credentials(), "BTC", "BTC")
            .expect("serializable creation request");
        assert_eq!(request.path, CREATE_DEPOSIT_ADDRESS_PATH);
        assert_eq!(
            request.body.as_deref(),
            Some(r#"{"currency":"BTC","net_type":"BTC"}"#)
        );
        assert_eq!(
            claims(&request)["query_hash"],
            hex::encode(Sha512::digest(b"currency=BTC&net_type=BTC"))
        );

        let pending = parse_created_deposit_address(
            r#"{"success":true,"message":"Generating a BTC deposit address."}"#,
            "BTC",
            "BTC",
            Network::Bitcoin,
        )
        .expect("official pending response");
        assert_eq!(pending.address, None);

        let issued = parse_created_deposit_address(
            r#"{"currency":"BTC","net_type":"BTC","deposit_address":"bc1qissued","secondary_address":null}"#,
            "BTC",
            "BTC",
            Network::Bitcoin,
        )
        .expect("official issued response");
        assert_eq!(issued.address.as_deref(), Some("bc1qissued"));

        let error = parse_created_deposit_address(
            r#"{"success":false,"message":"address creation rejected"}"#,
            "BTC",
            "BTC",
            Network::Bitcoin,
        )
        .expect_err("provider rejection");
        let Error::Exchange { code, message, .. } = error else {
            panic!("expected exchange error");
        };
        assert_eq!(code, "deposit_address_creation_rejected");
        assert_eq!(message, "address creation rejected");
    }

    #[test]
    fn deposit_address_list_preserves_pending_entries_and_provider_networks() {
        let entries = parse_deposit_addresses(
            r#"[{"currency":"BTC","net_type":"BTC","deposit_address":"bc1qissued","secondary_address":null},{"currency":"XRP","net_type":null,"deposit_address":null,"secondary_address":null}]"#,
        )
        .expect("official list response");
        assert_eq!(entries[0].network, Some(Network::Bitcoin));
        assert_eq!(entries[0].provider_network.as_deref(), Some("BTC"));
        assert_eq!(entries[1].network, None);
        assert_eq!(entries[1].address, None);
    }

    #[test]
    fn deposit_info_preserves_nullable_network_and_provider_policy() {
        let info = parse_deposit_info(
            r#"{"currency":"BTC","net_type":"BTC","is_deposit_possible":true,"deposit_impossible_reason":null,"minimum_deposit_amount":"0.0005","minimum_deposit_confirmations":18446744073709551615,"decimal_precision":18446744073709551615}"#,
            "BTC",
            "BTC",
        )
        .expect("official deposit information response");
        assert_eq!(info.network, Some(Network::Bitcoin));
        assert_eq!(info.provider_network.as_deref(), Some("BTC"));
        assert!(info.is_deposit_possible);
        assert_eq!(info.minimum_deposit_amount, Decimal::new(5, 4));
        assert_eq!(info.minimum_deposit_confirmations, u64::MAX);
        assert_eq!(info.decimal_precision, u64::MAX);

        let unavailable = parse_deposit_info(
            r#"{"currency":"BTC","net_type":null,"is_deposit_possible":false,"deposit_impossible_reason":"Network upgrade in progress","minimum_deposit_amount":"0","minimum_deposit_confirmations":0,"decimal_precision":8}"#,
            "BTC",
            "BTC",
        )
        .expect("nullable network response");
        assert_eq!(unavailable.network, None);
        assert_eq!(unavailable.provider_network, None);
        assert_eq!(
            unavailable.deposit_impossible_reason.as_deref(),
            Some("Network upgrade in progress")
        );
    }

    #[test]
    fn deposit_address_amount_is_rejected_before_signing() {
        let request = DepositAddressRequest::new("BTC", Network::Bitcoin).amount(Decimal::ONE);
        assert!(reject_deposit_address_amount(&request).is_err());
    }

    #[test]
    fn chance_fixture_keeps_fee_and_live_account_bounds_exact() {
        let chance = parse_chance(
            r#"{
              "currency":{"code":"BTC","withdraw_fee":"0.0008","wallet_support":["deposit","withdraw"]},
              "account":{"balance":"1.25"},
              "withdraw_limit":{"onetime":"1.0","remaining_daily":"0.75","minimum":"0.001","can_withdraw":true}
            }"#,
            "BTC",
        )
        .expect("official chance shape");

        assert_eq!(chance.fee, Some(WithdrawalFee::Fixed(Decimal::new(8, 4))));
        assert_eq!(chance.minimum, Some(Decimal::new(1, 3)));
        assert_eq!(chance.quote_maximum, Some(Decimal::new(75, 2)));
    }

    #[test]
    fn withdrawal_request_serializes_the_documented_body_without_sending_it() {
        let request = withdraw_request(&credentials(), &withdrawal(), "XRP", "XRP")
            .expect("serializable withdrawal");
        let raw_body = request.body.as_deref().expect("JSON body");
        assert_eq!(
            raw_body,
            r#"{"currency":"XRP","net_type":"XRP","amount":"12.345","address":"rDestination","secondary_address":"12345","transaction_type":"default"}"#
        );
        let body: Value = serde_json::from_str(raw_body).expect("valid JSON");

        assert_eq!(request.path, WITHDRAW_PATH);
        assert_eq!(body["currency"], "XRP");
        assert_eq!(body["net_type"], "XRP");
        assert_eq!(body["amount"], "12.345");
        assert_eq!(body["address"], "rDestination");
        assert_eq!(body["secondary_address"], "12345");
        assert_eq!(body["transaction_type"], "default");
        let signed = "currency=XRP&net_type=XRP&amount=12.345&address=rDestination&secondary_address=12345&transaction_type=default";
        assert_eq!(
            claims(&request)["query_hash"],
            hex::encode(Sha512::digest(signed.as_bytes()))
        );
    }

    #[test]
    fn every_wallet_read_uses_the_documented_path_and_history_parameters() {
        let credentials = credentials();
        let status = signed_get(&credentials, WALLET_STATUS_PATH, &[]).expect("status request");
        let addresses =
            signed_get(&credentials, DEPOSIT_ADDRESSES_PATH, &[]).expect("addresses request");
        let address = signed_get(
            &credentials,
            DEPOSIT_ADDRESS_PATH,
            &[
                ("currency", "BTC".to_string()),
                ("net_type", "BTC".to_string()),
            ],
        )
        .expect("address request");
        let deposit_chance =
            deposit_chance_request(&credentials, "BTC", "BTC").expect("deposit chance request");
        let chance = withdraw_chance_request(&credentials, "BTC", "BTC").expect("chance request");
        let allowlist =
            signed_get(&credentials, WITHDRAWAL_ADDRESSES_PATH, &[]).expect("allowlist request");
        let history = TransferHistoryRequest::new().asset("BTC").limit(25);
        let deposits = history_request(&credentials, DEPOSITS_PATH, &history, 2, 25)
            .expect("deposit history request");
        let withdrawals = history_request(&credentials, WITHDRAWALS_PATH, &history, 3, 25)
            .expect("withdrawal history request");

        assert_eq!(status.target(), WALLET_STATUS_PATH);
        assert_eq!(addresses.target(), DEPOSIT_ADDRESSES_PATH);
        assert_eq!(
            address.target(),
            "/v1/deposits/coin_address?currency=BTC&net_type=BTC"
        );
        assert_eq!(
            deposit_chance.target(),
            "/v1/deposits/chance/coin?currency=BTC&net_type=BTC"
        );
        assert_eq!(
            claims(&deposit_chance)["query_hash"],
            hex::encode(Sha512::digest(b"currency=BTC&net_type=BTC"))
        );
        assert_eq!(
            chance.target(),
            "/v1/withdraws/chance?currency=BTC&net_type=BTC"
        );
        assert_eq!(allowlist.target(), WITHDRAWAL_ADDRESSES_PATH);
        assert_eq!(
            deposits.target(),
            "/v1/deposits?currency=BTC&page=2&limit=25&order_by=desc"
        );
        assert_eq!(
            withdrawals.target(),
            "/v1/withdraws?currency=BTC&page=3&limit=25&order_by=desc"
        );
    }

    #[test]
    fn transfer_lookup_and_cancellation_use_documented_parameters_and_signatures() {
        let credentials = credentials();
        let deposit = lookup_request(
            &credentials,
            DEPOSIT_PATH,
            &TransferLookupRequest::by_id("BTC", "deposit-1"),
        )
        .expect("deposit lookup request");
        let withdrawal = lookup_request(
            &credentials,
            WITHDRAWAL_PATH,
            &TransferLookupRequest::by_tx_id("BTC", "tx-1"),
        )
        .expect("withdrawal lookup request");
        let cancellation = cancel_withdrawal_request(&credentials, "withdrawal-1")
            .expect("withdrawal cancellation request");

        assert_eq!(deposit.target(), "/v1/deposit?currency=BTC&uuid=deposit-1");
        assert_eq!(withdrawal.target(), "/v1/withdraw?currency=BTC&txid=tx-1");
        assert_eq!(
            cancellation.target(),
            "/v1/withdraws/coin?uuid=withdrawal-1"
        );
        assert_eq!(
            claims(&deposit)["query_hash"],
            hex::encode(Sha512::digest(b"currency=BTC&uuid=deposit-1"))
        );
        assert_eq!(
            claims(&withdrawal)["query_hash"],
            hex::encode(Sha512::digest(b"currency=BTC&txid=tx-1"))
        );
        assert_eq!(
            claims(&cancellation)["query_hash"],
            hex::encode(Sha512::digest(b"uuid=withdrawal-1"))
        );
    }

    #[test]
    fn transfer_fixtures_preserve_nullable_network_destination_and_provider_status() {
        let raw: Vec<RawTransfer> = parse::json(
            r#"[
              {"type":"deposit","uuid":"d-1","currency":"BTC","net_type":null,
               "txid":null,"state":"TRAVEL_RULE_SUSPECTED","created_at":"2024-07-14T13:35:41+09:00",
               "amount":"0.01","fee":"0"},
              {"type":"withdraw","uuid":"w-1","currency":"BTC","net_type":"BTC",
               "txid":"tx-1","state":"DONE","created_at":null,"amount":"0.02","fee":"0.0008"}
            ]"#,
        )
        .expect("official transfer list shape");
        let deposit = parse_deposit(&raw[0]).expect("deposit");
        let withdrawal = parse_withdrawal(&raw[1]).expect("withdrawal");

        assert_eq!(deposit.network, None);
        assert_eq!(deposit.provider_network, None);
        assert_eq!(deposit.provider_status, "TRAVEL_RULE_SUSPECTED");
        assert_eq!(deposit.status, DepositStatus::Pending);
        assert_eq!(
            deposit.created_at,
            Some(Timestamp::from_secs(1_720_931_741))
        );
        assert_eq!(withdrawal.destination, None);
        assert_eq!(withdrawal.network, Some(Network::Bitcoin));
        assert_eq!(withdrawal.provider_status, "DONE");
    }

    #[test]
    fn unknown_network_filters_match_only_the_exact_provider_id() {
        assert!(network_matches(
            &Network::Other("MYSTERY".to_string()),
            Some(&Network::Other("MYSTERY".to_string())),
            Some("MYSTERY")
        ));
        assert!(!network_matches(
            &Network::Other("mystery".to_string()),
            Some(&Network::Other("MYSTERY".to_string())),
            Some("MYSTERY")
        ));
    }
}
