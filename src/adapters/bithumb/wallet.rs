//! Bithumb's authenticated wallet and transfer endpoints.

use chrono::DateTime;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::error::{Error, Result, TransferErrorKind};
use crate::request::{DepositAddressRequest, TransferHistoryRequest, WithdrawRequest};
use crate::transport::{HttpRequest, HttpTransport};
use crate::types::{
    AssetNetwork, Cursor, Deposit, DepositAddress, DepositAddressEntry, DepositStatus, Exchange,
    Network, Page, TransferDestination, TravelRuleRequirement, Withdrawal, WithdrawalFee,
    WithdrawalQuote, WithdrawalStatus,
};

use super::parse::{self, EXCHANGE};
use super::{BithumbCredentials, private, rest};

const WALLET_STATUS_PATH: &str = "/v1/status/wallet";
const DEPOSIT_ADDRESSES_PATH: &str = "/v1/deposits/coin_addresses";
const DEPOSIT_ADDRESS_PATH: &str = "/v1/deposits/coin_address";
const CREATE_DEPOSIT_ADDRESS_PATH: &str = "/v1/deposits/generate_coin_address";
const WITHDRAW_CHANCE_PATH: &str = "/v1/withdraws/chance";
const WITHDRAWAL_ADDRESSES_PATH: &str = "/v1/withdraws/coin_addresses";
const WITHDRAW_PATH: &str = "/v1/withdraws/coin";
const DEPOSITS_PATH: &str = "/v1/deposits";
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
    withdraw_rate: Option<Value>,
    #[serde(default)]
    withdraw_fee_min: Option<Value>,
    #[serde(default)]
    withdraw_fee_max: Option<Value>,
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
    #[serde(default)]
    exchange_name: Option<String>,
    #[serde(default)]
    owner_type: Option<String>,
    #[serde(default)]
    owner_ko_name: Option<String>,
    #[serde(default)]
    owner_en_name: Option<String>,
    #[serde(default)]
    owner_corp_ko_name: Option<String>,
    #[serde(default)]
    owner_corp_en_name: Option<String>,
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
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    asset: &str,
) -> Result<Vec<AssetNetwork>> {
    let asset = asset_code(asset)?;
    let statuses = wallet_statuses(http, credentials).await?;
    let mut networks = Vec::new();

    for status in statuses.into_iter().filter(|status| status.asset == asset) {
        let chance = if status.withdrawal_enabled {
            Some(withdraw_chance(http, credentials, &asset, &status.provider_id).await?)
        } else {
            None
        };
        let withdrawal_enabled = status.withdrawal_enabled
            && chance
                .as_ref()
                .is_none_or(|chance| chance.supports_withdrawal);

        networks.push(AssetNetwork {
            exchange: Exchange::Bithumb,
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
    http: &HttpTransport,
    credentials: &BithumbCredentials,
) -> Result<Vec<DepositAddressEntry>> {
    let request = signed_get(credentials, DEPOSIT_ADDRESSES_PATH, &[])?;
    let body = rest::send(http, &request).await?;
    parse_deposit_addresses(&body)
}

pub(crate) async fn deposit_address(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &DepositAddressRequest,
) -> Result<DepositAddress> {
    reject_deposit_address_amount(request)?;
    let asset = asset_code(&request.asset)?;
    let provider_id = resolve_network(http, credentials, &asset, &request.network).await?;
    let api_request = signed_get(
        credentials,
        DEPOSIT_ADDRESS_PATH,
        &[
            ("currency", asset.clone()),
            ("net_type", provider_id.clone()),
        ],
    )?;
    let body = rest::send(http, &api_request).await?;
    parse_deposit_address(&body, &asset, &provider_id, request.network.clone())
}

pub(crate) async fn create_deposit_address(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &DepositAddressRequest,
) -> Result<DepositAddress> {
    reject_deposit_address_amount(request)?;
    let asset = asset_code(&request.asset)?;
    let provider_id = resolve_network(http, credentials, &asset, &request.network).await?;
    let api_request = create_deposit_address_request(credentials, &asset, &provider_id)?;
    let body = rest::send(http, &api_request).await?;
    parse_deposit_address(&body, &asset, &provider_id, request.network.clone())
}

pub(crate) async fn prepare_withdrawal(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &WithdrawRequest,
) -> Result<WithdrawalQuote> {
    let asset = asset_code(&request.asset)?;
    let provider_id = resolve_network(http, credentials, &asset, &request.network).await?;
    let chance = withdraw_chance(http, credentials, &asset, &provider_id).await?;
    if !chance.supports_withdrawal || !chance.can_withdraw {
        return Err(Error::exchange(
            EXCHANGE,
            "withdrawal_unavailable",
            format!("{asset} withdrawals on {provider_id} are not currently allowed"),
        ));
    }
    let addresses = withdrawal_addresses(http, credentials).await?;
    let allowed = addresses
        .iter()
        .find(|allowed| address_matches(allowed, request, &asset, &provider_id));
    let fee = chance
        .fee
        .as_ref()
        .map(|fee| fee.for_amount(request.amount));

    Ok(WithdrawalQuote {
        fee,
        expected_receive: fee.map(|fee| request.amount - fee),
        minimum_amount: chance.minimum,
        maximum_amount: chance.quote_maximum,
        address_allowed: Some(allowed.is_some()),
        travel_rule: if allowed.is_some_and(|allowed| travel_rule_data_missing(request, allowed)) {
            TravelRuleRequirement::Required { consent_url: None }
        } else {
            // Whether one-time privacy consent is still outstanding is exposed
            // only by the documented 422 response from the write endpoint.
            TravelRuleRequirement::NotRequired
        },
        expires_at: None,
    })
}

pub(crate) async fn withdraw(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &WithdrawRequest,
) -> Result<Withdrawal> {
    let asset = asset_code(&request.asset)?;
    let provider_id = resolve_network(http, credentials, &asset, &request.network).await?;
    let chance = withdraw_chance(http, credentials, &asset, &provider_id).await?;
    validate_live_withdrawal(&chance, request.amount, &asset, &provider_id)?;
    let addresses = withdrawal_addresses(http, credentials).await?;
    let allowed = addresses
        .iter()
        .find(|allowed| address_matches(allowed, request, &asset, &provider_id))
        .ok_or_else(|| {
            Error::transfer(
                TransferErrorKind::AddressNotAllowed,
                "destination is not in the Bithumb withdrawal-address list",
            )
        })?;
    if travel_rule_data_missing(request, allowed) {
        return Err(Error::transfer(
            TransferErrorKind::TravelRuleRequired,
            "Bithumb did not publish the destination exchange name required for this withdrawal",
        ));
    }

    let api_request = withdraw_request(credentials, request, &asset, &provider_id, allowed)?;
    let body = rest::send(http, &api_request).await?;
    parse_withdrawal(&decode(&body)?)
}

pub(crate) async fn deposits(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &TransferHistoryRequest,
) -> Result<Page<Deposit>> {
    let (page, limit) = history_position(request, "bithumb:deposits")?;
    let api_request = history_request(credentials, DEPOSITS_PATH, request, page, limit)?;
    let body = rest::send(http, &api_request).await?;
    let raw: Vec<RawTransfer> = decode(&body)?;
    let raw_count = raw.len();
    let mut items = raw.iter().map(parse_deposit).collect::<Result<Vec<_>>>()?;
    filter_deposits(&mut items, request.network.as_ref());

    Ok(Page {
        items,
        next: next_cursor("bithumb:deposits", page, limit, raw_count),
    })
}

pub(crate) async fn withdrawals(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    request: &TransferHistoryRequest,
) -> Result<Page<Withdrawal>> {
    let (page, limit) = history_position(request, "bithumb:withdrawals")?;
    let api_request = history_request(credentials, WITHDRAWALS_PATH, request, page, limit)?;
    let body = rest::send(http, &api_request).await?;
    let raw: Vec<RawTransfer> = decode(&body)?;
    let raw_count = raw.len();
    let mut items = raw
        .iter()
        .map(parse_withdrawal)
        .collect::<Result<Vec<_>>>()?;
    filter_withdrawals(&mut items, request.network.as_ref());

    Ok(Page {
        items,
        next: next_cursor("bithumb:withdrawals", page, limit, raw_count),
    })
}

async fn wallet_statuses(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
) -> Result<Vec<WalletStatus>> {
    let request = signed_get(credentials, WALLET_STATUS_PATH, &[])?;
    let body = rest::send(http, &request).await?;
    parse_wallet_statuses(&body)
}

async fn resolve_network(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    asset: &str,
    requested: &Network,
) -> Result<String> {
    let matches = wallet_statuses(http, credentials)
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
            format!("Bithumb publishes no {asset} network matching {requested}"),
        )),
        _ => Err(Error::invalid_request(
            "network",
            format!("more than one Bithumb {asset} network matches {requested}"),
        )),
    }
}

async fn withdraw_chance(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
    asset: &str,
    provider_id: &str,
) -> Result<Chance> {
    let request = withdraw_chance_request(credentials, asset, provider_id)?;
    let body = rest::send(http, &request).await?;
    parse_chance(&body, asset)
}

async fn withdrawal_addresses(
    http: &HttpTransport,
    credentials: &BithumbCredentials,
) -> Result<Vec<WithdrawalAddress>> {
    let request = signed_get(credentials, WITHDRAWAL_ADDRESSES_PATH, &[])?;
    decode(&rest::send(http, &request).await?)
}

fn signed_get(
    credentials: &BithumbCredentials,
    path: &str,
    params: &[(&str, String)],
) -> Result<HttpRequest> {
    let query = signed_query(params)?;
    let request = HttpRequest::get(path).header(
        "authorization",
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
                format!("`{value}` is not a safe Bithumb query value"),
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
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_')
}

fn withdraw_chance_request(
    credentials: &BithumbCredentials,
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

fn create_deposit_address_request(
    credentials: &BithumbCredentials,
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
            "could not build Bithumb deposit-address JSON: {error}"
        ))
    })?;

    Ok(HttpRequest::post(CREATE_DEPOSIT_ADDRESS_PATH)
        .json_body(body)
        .header(
            "authorization",
            private::authorization(credentials, &query)?,
        ))
}

fn reject_deposit_address_amount(request: &DepositAddressRequest) -> Result<()> {
    if request.amount.is_some() {
        return Err(Error::invalid_request(
            "amount",
            "Bithumb deposit-address requests accept only an asset and network",
        ));
    }
    Ok(())
}

fn withdraw_request(
    credentials: &BithumbCredentials,
    request: &WithdrawRequest,
    asset: &str,
    provider_id: &str,
    allowed: &WithdrawalAddress,
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
    if !matches!(
        &request.destination,
        TransferDestination::Exchange(destination) if destination.exchange == Exchange::Bithumb
    ) {
        push_optional(
            &mut params,
            "exchange_name",
            allowed.exchange_name.as_deref(),
        );
        let personal = match (
            allowed.owner_type.as_deref(),
            nonempty(allowed.owner_ko_name.as_deref()),
            nonempty(allowed.owner_en_name.as_deref()),
        ) {
            (Some("personal"), Some(korean), Some(english)) => Some((korean, english)),
            _ => None,
        };
        let corporation = match (
            allowed.owner_type.as_deref(),
            nonempty(allowed.owner_ko_name.as_deref()),
            nonempty(allowed.owner_en_name.as_deref()),
            nonempty(allowed.owner_corp_ko_name.as_deref()),
            nonempty(allowed.owner_corp_en_name.as_deref()),
        ) {
            (
                Some("corporation"),
                Some(korean),
                Some(english),
                Some(company_korean),
                Some(company_english),
            ) => Some((korean, english, company_korean, company_english)),
            _ => None,
        };
        if let Some((korean, english)) = personal {
            params.push(("receiver_type", "personal".to_string()));
            params.push(("receiver_ko_name", korean.to_string()));
            params.push(("receiver_en_name", english.to_string()));
        } else if let Some((korean, english, company_korean, company_english)) = corporation {
            params.push(("receiver_type", "corporation".to_string()));
            params.push(("receiver_ko_name", korean.to_string()));
            params.push(("receiver_en_name", english.to_string()));
            params.push(("receiver_corp_ko_name", company_korean.to_string()));
            params.push(("receiver_corp_en_name", company_english.to_string()));
        }
    }

    let query = raw_query(&params);
    let body = params
        .iter()
        .map(|(name, value)| ((*name).to_string(), Value::String(value.clone())))
        .collect::<Map<_, _>>();
    let body = serde_json::to_string(&body).map_err(|error| {
        Error::decode(format!("could not build Bithumb withdrawal JSON: {error}"))
    })?;

    Ok(HttpRequest::post(WITHDRAW_PATH).json_body(body).header(
        "authorization",
        private::authorization(credentials, &query)?,
    ))
}

fn push_optional(
    params: &mut Vec<(&'static str, String)>,
    name: &'static str,
    value: Option<&str>,
) {
    if let Some(value) = value.filter(|value| !value.is_empty()) {
        params.push((name, value.to_string()));
    }
}

fn nonempty(value: Option<&str>) -> Option<&str> {
    value.filter(|value| !value.is_empty())
}

fn history_request(
    credentials: &BithumbCredentials,
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
            format!("Bithumb serves 1 to {MAX_HISTORY_COUNT} transfer rows per page"),
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
                    "cursor was not issued by this Bithumb history endpoint",
                )
            })?,
    };
    Ok((page, limit))
}

fn next_cursor(prefix: &str, page: u32, limit: u32, count: usize) -> Option<Cursor> {
    (count >= limit as usize).then(|| Cursor::new(format!("{prefix}:{}", page + 1)))
}

fn parse_wallet_statuses(value: &Value) -> Result<Vec<WalletStatus>> {
    decode::<Vec<RawWalletStatus>>(value)?
        .into_iter()
        .map(|raw| {
            let provider_id = raw.net_type.ok_or_else(|| {
                Error::decode(format!(
                    "Bithumb wallet status for {} carries no `net_type`",
                    raw.currency
                ))
            })?;
            let (deposit_enabled, withdrawal_enabled) = match raw.wallet_state.as_str() {
                "working" => (true, true),
                "withdraw_only" => (false, true),
                "deposit_only" => (true, false),
                "paused" => (false, false),
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
    value: &Value,
    expected_asset: &str,
    expected_provider: &str,
    requested_network: Network,
) -> Result<DepositAddress> {
    let raw: RawDepositAddress = decode(value)?;
    if !raw.currency.eq_ignore_ascii_case(expected_asset) {
        return Err(Error::decode(format!(
            "Bithumb returned deposit asset {} for requested {expected_asset}",
            raw.currency
        )));
    }
    if raw
        .net_type
        .as_deref()
        .is_some_and(|provider| provider != expected_provider)
    {
        return Err(Error::decode(format!(
            "Bithumb returned a different deposit network than {expected_provider}"
        )));
    }

    Ok(DepositAddress {
        exchange: Exchange::Bithumb,
        asset: expected_asset.to_string(),
        network: requested_network,
        address: raw.deposit_address,
        memo: raw.secondary_address,
    })
}

fn parse_deposit_addresses(value: &Value) -> Result<Vec<DepositAddressEntry>> {
    decode::<Vec<RawDepositAddress>>(value)?
        .into_iter()
        .map(|raw| {
            let asset = raw.currency.trim().to_ascii_uppercase();
            if asset.is_empty() {
                return Err(Error::decode(
                    "Bithumb returned a deposit-address entry without a currency",
                ));
            }
            let provider_network = raw.net_type;
            Ok(DepositAddressEntry {
                exchange: Exchange::Bithumb,
                asset,
                network: provider_network.as_deref().map(network_from_provider),
                provider_network,
                address: raw.deposit_address,
                memo: raw.secondary_address,
            })
        })
        .collect()
}

fn parse_chance(value: &Value, expected_asset: &str) -> Result<Chance> {
    let raw: RawChance = decode(value)?;
    if !raw.currency.code.eq_ignore_ascii_case(expected_asset) {
        return Err(Error::decode(format!(
            "Bithumb returned withdrawal rules for {} instead of {expected_asset}",
            raw.currency.code
        )));
    }
    let fixed = decimal_opt(raw.currency.withdraw_fee.as_ref(), "currency.withdraw_fee")?;
    let rate = decimal_opt(
        raw.currency.withdraw_rate.as_ref(),
        "currency.withdraw_rate",
    )?;
    let fee = match (fixed, rate) {
        (Some(fixed), None) => Some(WithdrawalFee::Fixed(fixed)),
        (None, Some(rate)) => Some(WithdrawalFee::Rate {
            rate,
            minimum: decimal_opt(
                raw.currency.withdraw_fee_min.as_ref(),
                "currency.withdraw_fee_min",
            )?,
            maximum: decimal_opt(
                raw.currency.withdraw_fee_max.as_ref(),
                "currency.withdraw_fee_max",
            )?,
        }),
        (None, None) => None,
        (Some(_), Some(_)) => {
            return Err(Error::decode(
                "Bithumb returned both fixed and rate withdrawal fees",
            ));
        }
    };
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

fn decimal_opt(value: Option<&Value>, field: &'static str) -> Result<Option<Decimal>> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(raw)) => parse::decimal(raw, field).map(Some),
        Some(Value::Number(raw)) => parse::decimal(&raw.to_string(), field).map(Some),
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
            format!("{amount} is outside Bithumb's current withdrawal bounds"),
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

fn decimal_value(value: &Value, field: &'static str) -> Result<Decimal> {
    match value {
        Value::String(raw) => parse::decimal(raw, field),
        Value::Number(raw) => parse::decimal(&raw.to_string(), field),
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
        "REQUESTED_PENDING"
        | "REQUESTED_PROCESSING"
        | "DEPOSIT_PROCESSING"
        | "REFUNDING_PENDING"
        | "REFUNDING_PROCESSING"
        | "REFUNDED_PROCESSING" => DepositStatus::Pending,
        "DEPOSIT_ACCEPTED" => DepositStatus::Completed,
        "REQUESTED_SYSTEM_REJECTED"
        | "REQUESTED_ADMIN_REJECTED"
        | "DEPOSIT_CANCELLED"
        | "REFUNDING_SYSTEM_REJECTED"
        | "REFUNDING_ADMIN_REJECTED"
        | "REFUNDING_ACCEPTED"
        | "REFUNDED_ACCEPTED"
        | "REFUNDED_CANCELLED" => DepositStatus::Failed,
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

fn travel_rule_data_missing(request: &WithdrawRequest, allowed: &WithdrawalAddress) -> bool {
    matches!(
        &request.destination,
        TransferDestination::Exchange(destination)
            if destination.exchange != Exchange::Bithumb
                && allowed.exchange_name.as_deref().is_none_or(str::is_empty)
    )
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
            format!("`{raw}` is not a Bithumb asset code"),
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

fn decode<T: for<'de> Deserialize<'de>>(value: &Value) -> Result<T> {
    serde_json::from_value(value.clone())
        .map_err(|error| Error::decode(format!("invalid Bithumb wallet response: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChainDestination, ExchangeDestination, Timestamp};
    use sha2::{Digest, Sha512};

    fn parsed(raw: &str) -> Value {
        serde_json::from_str(raw).expect("fixture is JSON")
    }

    fn credentials() -> BithumbCredentials {
        BithumbCredentials {
            access_key: "test-access".to_string(),
            secret_key: "test-secret".to_string(),
        }
    }

    fn external_withdrawal() -> WithdrawRequest {
        WithdrawRequest::new(
            "XRP",
            Network::XrpLedger,
            Decimal::new(12_345, 3),
            TransferDestination::Exchange(ExchangeDestination {
                exchange: Exchange::Upbit,
                asset: "XRP".to_string(),
                network: Network::XrpLedger,
                address: "rDestination".to_string(),
                memo: Some("12345".to_string()),
            }),
        )
    }

    fn allowed_address() -> WithdrawalAddress {
        WithdrawalAddress {
            currency: "XRP".to_string(),
            net_type: "XRP".to_string(),
            withdraw_address: "rDestination".to_string(),
            secondary_address: Some("12345".to_string()),
            exchange_name: Some("Upbit".to_string()),
            owner_type: Some("personal".to_string()),
            owner_ko_name: Some("홍길동".to_string()),
            owner_en_name: Some("GILDONG HONG".to_string()),
            owner_corp_ko_name: None,
            owner_corp_en_name: None,
        }
    }

    fn claims(request: &HttpRequest) -> Value {
        use base64::Engine as _;

        let token = request
            .headers
            .iter()
            .find(|(name, _)| name == "authorization")
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
        let statuses = parse_wallet_statuses(&parsed(
            r#"[
              {"currency":"BTC","wallet_state":"working","net_type":"BTC"},
              {"currency":"ETH","wallet_state":"withdraw_only","net_type":"MYSTERY"},
              {"currency":"XRP","wallet_state":"new_state","net_type":"XRP"}
            ]"#,
        ))
        .expect("official wallet status fixture");

        assert_eq!(statuses[0].network, Network::Bitcoin);
        assert!(statuses[0].deposit_enabled && statuses[0].withdrawal_enabled);
        assert_eq!(statuses[1].network, Network::Other("MYSTERY".to_string()));
        assert!(!statuses[1].deposit_enabled && statuses[1].withdrawal_enabled);
        assert!(!statuses[2].deposit_enabled && !statuses[2].withdrawal_enabled);
    }

    #[test]
    fn rate_fee_fixture_keeps_the_ratio_and_both_clamps() {
        let chance = parse_chance(
            &parsed(
                r#"{
                  "currency":{"code":"TOKEN","withdraw_fee":null,"withdraw_rate":"0.01",
                    "withdraw_fee_min":"1","withdraw_fee_max":"100","wallet_support":["withdraw"]},
                  "account":{"balance":"10000"},
                  "withdraw_limit":{"onetime":"9000","remaining_daily":"8000","minimum":"10","can_withdraw":true}
                }"#,
            ),
            "TOKEN",
        )
        .expect("official rate fee fixture");

        assert_eq!(
            chance.fee,
            Some(WithdrawalFee::Rate {
                rate: Decimal::new(1, 2),
                minimum: Some(Decimal::ONE),
                maximum: Some(Decimal::from(100)),
            })
        );
        assert_eq!(chance.quote_maximum, Some(Decimal::from(8_000)));
    }

    #[test]
    fn pending_address_and_nullable_history_fields_are_not_invented() {
        let address = parse_deposit_address(
            &parsed(
                r#"{"currency":"BTC","net_type":null,"deposit_address":null,"secondary_address":null}"#,
            ),
            "BTC",
            "BTC",
            Network::Bitcoin,
        )
        .expect("nullable address fixture");
        let raw: RawTransfer = decode(&parsed(
            r#"{"uuid":"w-1","currency":"BTC","net_type":null,"txid":null,
               "state":"processing","created_at":null,"amount":"0.1","fee":null}"#,
        ))
        .expect("nullable withdrawal fixture");
        let withdrawal = parse_withdrawal(&raw).expect("withdrawal");

        assert_eq!(address.address, None);
        assert_eq!(withdrawal.network, None);
        assert_eq!(withdrawal.provider_network, None);
        assert_eq!(withdrawal.destination, None);
        assert_eq!(withdrawal.provider_status, "processing");
        assert_eq!(withdrawal.status, WithdrawalStatus::Processing);
    }

    #[test]
    fn deposit_address_creation_uses_bithumbs_documented_json_and_signature() {
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

        let address = parse_deposit_address(
            &parsed(
                r#"{"currency":"BTC","net_type":"BTC","deposit_address":"bc1qissued","secondary_address":null}"#,
            ),
            "BTC",
            "BTC",
            Network::Bitcoin,
        )
        .expect("official issued response");
        assert_eq!(address.address.as_deref(), Some("bc1qissued"));
    }

    #[test]
    fn deposit_address_list_preserves_pending_entries_and_provider_networks() {
        let entries = parse_deposit_addresses(&parsed(
            r#"[{"currency":"BTC","net_type":"BTC","deposit_address":"bc1qissued","secondary_address":null},{"currency":"XRP","net_type":null,"deposit_address":null,"secondary_address":null}]"#,
        ))
        .expect("official list response");
        assert_eq!(entries[0].network, Some(Network::Bitcoin));
        assert_eq!(entries[0].provider_network.as_deref(), Some("BTC"));
        assert_eq!(entries[1].network, None);
        assert_eq!(entries[1].address, None);
    }

    #[test]
    fn deposit_address_amount_is_rejected_before_signing() {
        let request = DepositAddressRequest::new("BTC", Network::Bitcoin).amount(Decimal::ONE);
        assert!(reject_deposit_address_amount(&request).is_err());
    }

    #[test]
    fn withdrawal_request_serializes_travel_rule_fields_without_sending_it() {
        let request = withdraw_request(
            &credentials(),
            &external_withdrawal(),
            "XRP",
            "XRP",
            &allowed_address(),
        )
        .expect("serializable withdrawal");
        let raw_body = request.body.as_deref().expect("JSON body");
        assert_eq!(
            raw_body,
            r#"{"currency":"XRP","net_type":"XRP","amount":"12.345","address":"rDestination","secondary_address":"12345","exchange_name":"Upbit","receiver_type":"personal","receiver_ko_name":"홍길동","receiver_en_name":"GILDONG HONG"}"#
        );
        let body: Value = serde_json::from_str(raw_body).expect("valid JSON");

        assert_eq!(request.path, WITHDRAW_PATH);
        assert_eq!(body["exchange_name"], "Upbit");
        assert_eq!(body["receiver_type"], "personal");
        assert_eq!(body["receiver_ko_name"], "홍길동");
        assert_eq!(body["receiver_en_name"], "GILDONG HONG");
        assert_eq!(body["secondary_address"], "12345");
        let signed = "currency=XRP&net_type=XRP&amount=12.345&address=rDestination&secondary_address=12345&exchange_name=Upbit&receiver_type=personal&receiver_ko_name=홍길동&receiver_en_name=GILDONG HONG";
        assert_eq!(
            claims(&request)["query_hash"],
            hex::encode(Sha512::digest(signed.as_bytes()))
        );
    }

    #[test]
    fn incomplete_owner_metadata_is_not_misrepresented_as_a_code_withdrawal() {
        let mut allowed = allowed_address();
        allowed.owner_en_name = None;
        let request = withdraw_request(
            &credentials(),
            &external_withdrawal(),
            "XRP",
            "XRP",
            &allowed,
        )
        .expect("whitelist or ID Connect request");
        let body: Value =
            serde_json::from_str(request.body.as_deref().expect("JSON body")).expect("valid JSON");

        assert_eq!(body["exchange_name"], "Upbit");
        assert!(body.get("receiver_type").is_none());
        assert!(body.get("receiver_ko_name").is_none());
        assert!(body.get("receiver_en_name").is_none());
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
    fn external_exchange_without_published_travel_rule_metadata_is_blocked() {
        let mut allowed = allowed_address();
        allowed.exchange_name = None;

        assert!(travel_rule_data_missing(&external_withdrawal(), &allowed));
        assert!(!travel_rule_data_missing(
            &WithdrawRequest::new(
                "XRP",
                Network::XrpLedger,
                Decimal::ONE,
                TransferDestination::Chain(ChainDestination {
                    asset: "XRP".to_string(),
                    network: Network::XrpLedger,
                    address: "rDestination".to_string(),
                    memo: Some("12345".to_string()),
                })
            ),
            &allowed
        ));
    }

    #[test]
    fn deposit_states_keep_the_provider_status_and_refunds_are_not_successes() {
        let raw: RawTransfer = decode(&parsed(
            r#"{"uuid":"d-1","currency":"BTC","net_type":"BTC","txid":"tx-1",
               "state":"REFUNDED_ACCEPTED","created_at":"2024-07-14T13:35:41+09:00",
               "amount":"0.1","fee":"0"}"#,
        ))
        .expect("deposit fixture");
        let deposit = parse_deposit(&raw).expect("deposit");

        assert_eq!(deposit.status, DepositStatus::Failed);
        assert_eq!(deposit.provider_status, "REFUNDED_ACCEPTED");
        assert_eq!(
            deposit.created_at,
            Some(Timestamp::from_secs(1_720_931_741))
        );
    }

    #[test]
    fn unknown_network_filters_match_only_the_exact_provider_id() {
        assert!(network_matches(
            &Network::Other("DASH".to_string()),
            Some(&Network::Other("DASH".to_string())),
            Some("DASH")
        ));
        assert!(!network_matches(
            &Network::Other("dash".to_string()),
            Some(&Network::Other("DASH".to_string())),
            Some("DASH")
        ));
    }
}
