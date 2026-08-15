//! Binance Wallet SAPI transfer rules, addresses, withdrawals, and histories.

use std::borrow::Cow;

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::error::{Error, Result, TransferErrorKind};
use crate::feature::Feature;
use crate::request::{DepositAddressRequest, TransferHistoryRequest, WithdrawRequest};
use crate::transport::{HttpMethod, HttpRequest};
use crate::types::{
    AssetNetwork, Cursor, Deposit, DepositAddress, DepositStatus, Exchange, Network, Page,
    Timestamp, TransferDestination, TravelRuleRequirement, Withdrawal, WithdrawalFee,
    WithdrawalQuote, WithdrawalStatus,
};

use super::private::signed;
use super::{
    BinanceAdapter, BinanceDepositHistoryRequest, BinanceMarket, BinanceWithdrawHistoryRequest,
    EXCHANGE, check_asset, now_millis, parse,
};

const MAX_HISTORY_LIMIT: u32 = 1_000;
const DEFAULT_HISTORY_LIMIT: u32 = 100;
// Binance requires the requested interval to be less than 90 days.
const HISTORY_WINDOW_MS: i64 = 90 * 24 * 60 * 60 * 1_000 - 1;
const WITHDRAW_ORDER_ID_HISTORY_WINDOW_MS: i64 = 7 * 24 * 60 * 60 * 1_000 - 1;

/// One Wallet SAPI coin configuration entry.
///
/// The normalized asset-network API selects only transferable networks. This
/// provider contract preserves Binance's coin-wide flags and each network's
/// raw configuration for callers that need Wallet-specific decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceCoinInformation {
    /// Binance asset code.
    pub coin: String,
    /// Whether Binance enables deposits for the coin overall.
    pub deposit_all_enabled: bool,
    /// Whether Binance enables withdrawals for the coin overall.
    pub withdraw_all_enabled: bool,
    /// Provider display name when Binance provides it.
    pub name: Option<String>,
    /// Available Wallet balance when Binance provides it.
    pub free: Option<Decimal>,
    /// Wallet balance locked by Binance when Binance provides it.
    pub locked: Option<Decimal>,
    /// Wallet balance frozen by Binance when Binance provides it.
    pub freeze: Option<Decimal>,
    /// Wallet balance currently withdrawing when Binance provides it.
    pub withdrawing: Option<Decimal>,
    /// Whether Binance regards the asset as legal money.
    pub is_legal_money: Option<bool>,
    /// Whether Binance allows trading this asset.
    pub trading: Option<bool>,
    /// Per-network deposit and withdrawal configuration.
    pub networks: Vec<BinanceCoinNetworkInformation>,
    /// Complete compact JSON entry for forward-compatible provider fields.
    pub raw_json: String,
}

/// One network configuration within a Wallet SAPI coin entry.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceCoinNetworkInformation {
    /// Binance's network identifier.
    pub network: String,
    /// Whether Binance enables deposits on this network.
    pub deposit_enabled: bool,
    /// Whether Binance enables withdrawals on this network.
    pub withdraw_enabled: bool,
    /// Whether the network is currently unavailable for Wallet transfers.
    pub busy: bool,
    /// Required whole multiple for withdrawal amounts, when Binance provides it.
    pub withdrawal_integer_multiple: Option<Decimal>,
    /// Fixed withdrawal fee, when Binance provides it.
    pub withdrawal_fee: Option<Decimal>,
    /// Minimum withdrawal amount, when Binance provides it.
    pub minimum_withdrawal: Option<Decimal>,
    /// Maximum withdrawal amount, when Binance provides it.
    pub maximum_withdrawal: Option<Decimal>,
    /// Whether this network requires an address tag or memo.
    pub withdrawal_tag: Option<bool>,
    /// Whether this is Binance's default network for the asset.
    pub is_default: Option<bool>,
    /// Minimum on-chain confirmations Binance requires for deposit credit.
    pub minimum_confirmations: Option<u64>,
    /// Confirmations Binance requires before funds are unlocked.
    pub unlock_confirmations: Option<u64>,
    /// Provider contract address when Binance provides it.
    pub contract_address: Option<String>,
    /// Complete compact JSON entry for forward-compatible provider fields.
    pub raw_json: String,
}

/// Wallet SAPI API-key permissions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceApiKeyPermissions {
    /// Whether Binance restricts the key to configured IP addresses.
    pub ip_restrict: bool,
    /// Binance's key creation timestamp when it provides one.
    pub create_time: Option<Timestamp>,
    /// Whether the key may read account data.
    pub enable_reading: bool,
    /// Whether the key may withdraw assets.
    pub enable_withdrawals: bool,
    /// Whether the key may perform internal transfers.
    pub enable_internal_transfer: bool,
    /// Whether the key may use margin operations.
    pub enable_margin: bool,
    /// Whether the key may trade Spot and margin products.
    pub enable_spot_and_margin_trading: bool,
    /// Whether the key may trade USD-M/COIN-M futures.
    pub enable_futures: bool,
    /// Whether the key may use universal transfers.
    pub permits_universal_transfer: bool,
    /// Whether the key may trade vanilla options.
    pub enable_vanilla_options: bool,
    /// Whether the key may submit FIX API trades.
    pub enable_fix_api_trade: bool,
    /// Whether the key may use FIX API read-only access.
    pub enable_fix_read_only: bool,
    /// Whether the key may use portfolio-margin trading.
    pub enable_portfolio_margin_trading: bool,
    /// Complete compact JSON response for forward-compatible permission flags.
    pub raw_json: String,
}

/// Wallet SAPI deposit-history response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceDepositHistory {
    /// Deposit records returned in this provider page.
    pub entries: Vec<BinanceDepositHistoryEntry>,
    /// Complete compact JSON response array.
    pub raw_json: String,
}

/// One Wallet SAPI deposit-history record.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceDepositHistoryEntry {
    /// Binance deposit identifier.
    pub id: String,
    /// Deposited amount.
    pub amount: Decimal,
    /// Asset code.
    pub coin: String,
    /// Binance network identifier.
    pub network: String,
    /// Provider-specific deposit status code.
    pub status: u32,
    /// Destination address when Binance provides it.
    pub address: Option<String>,
    /// Destination address tag when Binance provides it.
    pub address_tag: Option<String>,
    /// On-chain transaction identifier when Binance provides it.
    pub tx_id: Option<String>,
    /// Binance deposit insertion time.
    pub insert_time: Timestamp,
    /// Binance completion time when it provides one.
    pub complete_time: Option<Timestamp>,
    /// Provider transfer type when it provides one.
    pub transfer_type: Option<u32>,
    /// Source address when requested and supplied by Binance.
    pub source_address: Option<String>,
    /// Complete compact JSON entry for forward-compatible provider fields.
    pub raw_json: String,
}

/// Wallet SAPI Travel Rule questionnaire requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceQuestionnaireRequirements {
    /// Country code Binance uses to determine the questionnaire requirement.
    pub questionnaire_country_code: String,
    /// Complete compact JSON response for forward-compatible provider fields.
    pub raw_json: String,
}

/// A registered Wallet SAPI withdrawal address.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceWithdrawalAddress {
    /// Registered destination address.
    pub address: String,
    /// Optional registered address tag or memo.
    pub address_tag: Option<String>,
    /// Asset code.
    pub coin: String,
    /// Binance network identifier.
    pub network: String,
    /// Whether Binance marked this address as whitelisted.
    pub white_status: bool,
    /// User-facing address-book name when Binance provides it.
    pub name: Option<String>,
    /// Provider address origin when Binance provides it.
    pub origin: Option<String>,
    /// Provider address origin type when Binance provides it.
    pub origin_type: Option<String>,
    /// Complete compact JSON entry for forward-compatible provider fields.
    pub raw_json: String,
}

/// Wallet SAPI withdrawal-history response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceWithdrawHistory {
    /// Withdrawal records returned in this provider page.
    pub entries: Vec<BinanceWithdrawHistoryEntry>,
    /// Complete compact JSON response array.
    pub raw_json: String,
}

/// One Wallet SAPI withdrawal-history record.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BinanceWithdrawHistoryEntry {
    /// Binance withdrawal identifier.
    pub id: String,
    /// Withdrawn amount.
    pub amount: Decimal,
    /// Binance withdrawal fee.
    pub transaction_fee: Decimal,
    /// Asset code.
    pub coin: String,
    /// Provider-specific withdrawal status code.
    pub status: u32,
    /// Destination address when Binance provides it.
    pub address: Option<String>,
    /// On-chain transaction identifier when Binance provides it.
    pub tx_id: Option<String>,
    /// Exact provider apply-time text when Binance provides it.
    pub apply_time: Option<String>,
    /// Binance network identifier when it provides one.
    pub network: Option<String>,
    /// Caller-supplied withdrawal order identifier when Binance provides it.
    pub withdraw_order_id: Option<String>,
    /// Provider status detail when Binance provides it.
    pub info: Option<String>,
    /// Provider transfer type when Binance provides it.
    pub transfer_type: Option<u32>,
    /// Binance confirmation count when it provides one.
    pub confirm_no: Option<u32>,
    /// Provider wallet type when Binance provides it.
    pub wallet_type: Option<u32>,
    /// Provider transaction key when Binance provides it.
    pub tx_key: Option<String>,
    /// Exact provider complete-time text when Binance provides it.
    pub complete_time: Option<String>,
    /// Complete compact JSON entry for forward-compatible provider fields.
    pub raw_json: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCoinConfig {
    coin: String,
    #[serde(default)]
    deposit_all_enable: bool,
    #[serde(default)]
    withdraw_all_enable: bool,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    free: Option<String>,
    #[serde(default)]
    locked: Option<String>,
    #[serde(default)]
    freeze: Option<String>,
    #[serde(default)]
    withdrawing: Option<String>,
    #[serde(default)]
    is_legal_money: Option<bool>,
    #[serde(default)]
    trading: Option<bool>,
    #[serde(default)]
    network_list: Vec<RawNetworkConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawNetworkConfig {
    network: String,
    #[serde(default)]
    deposit_enable: bool,
    #[serde(default)]
    withdraw_enable: bool,
    withdraw_integer_multiple: String,
    withdraw_fee: String,
    withdraw_min: String,
    withdraw_max: String,
    #[serde(default)]
    same_address: Option<bool>,
    #[serde(default)]
    withdraw_tag: Option<bool>,
    #[serde(default)]
    is_default: Option<bool>,
    #[serde(default)]
    min_confirm: Option<u64>,
    #[serde(default)]
    un_lock_confirm: Option<u64>,
    #[serde(default)]
    contract_address: Option<String>,
    #[serde(default)]
    busy: bool,
}

impl RawNetworkConfig {
    fn memo_required(&self) -> bool {
        self.withdraw_tag.or(self.same_address).unwrap_or(false)
    }
}

#[derive(Debug, Deserialize)]
struct RawDepositAddress {
    address: String,
    coin: String,
    #[serde(default)]
    tag: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawApiPermissions {
    #[serde(default)]
    ip_restrict: bool,
    #[serde(default)]
    create_time: Option<i64>,
    #[serde(default)]
    enable_reading: bool,
    enable_withdrawals: bool,
    #[serde(default)]
    enable_internal_transfer: bool,
    #[serde(default)]
    enable_margin: bool,
    #[serde(default)]
    enable_spot_and_margin_trading: bool,
    #[serde(default)]
    enable_futures: bool,
    #[serde(default)]
    permits_universal_transfer: bool,
    #[serde(default)]
    enable_vanilla_options: bool,
    #[serde(default)]
    enable_fix_api_trade: bool,
    #[serde(default)]
    enable_fix_read_only: bool,
    #[serde(default)]
    enable_portfolio_margin_trading: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawWithdrawAddress {
    address: String,
    #[serde(default)]
    address_tag: String,
    coin: String,
    network: String,
    white_status: bool,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    origin: Option<String>,
    #[serde(default)]
    origin_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawQuestionnaireRequirements {
    questionnaire_country_code: String,
}

#[derive(Debug, Deserialize)]
struct RawWithdrawAck {
    id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawDeposit {
    id: String,
    amount: String,
    coin: String,
    network: String,
    status: i64,
    #[serde(default)]
    address: String,
    #[serde(default)]
    address_tag: String,
    #[serde(default)]
    tx_id: String,
    insert_time: i64,
    #[serde(default)]
    complete_time: Option<i64>,
    #[serde(default)]
    transfer_type: Option<i64>,
    #[serde(default)]
    source_address: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawWithdrawal {
    id: String,
    amount: String,
    transaction_fee: String,
    coin: String,
    status: i64,
    #[serde(default)]
    address: String,
    #[serde(default)]
    tx_id: String,
    #[serde(default)]
    apply_time: Option<String>,
    #[serde(default)]
    network: Option<String>,
    #[serde(default)]
    withdraw_order_id: Option<String>,
    #[serde(default)]
    info: Option<String>,
    #[serde(default)]
    transfer_type: Option<i64>,
    #[serde(default)]
    confirm_no: Option<i64>,
    #[serde(default)]
    wallet_type: Option<i64>,
    #[serde(default)]
    tx_key: Option<String>,
    #[serde(default)]
    complete_time: Option<String>,
}

#[derive(Debug, Clone)]
struct CheckedNetwork {
    provider_id: String,
    rules: RawNetworkConfig,
    deposit_enabled: bool,
    withdrawal_enabled: bool,
}

#[derive(Debug)]
struct CheckedWithdrawal {
    provider_network: String,
    fee: Decimal,
    quote: WithdrawalQuote,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HistoryKind {
    Deposits,
    Withdrawals,
}

impl HistoryKind {
    const fn cursor_tag(self) -> &'static str {
        match self {
            Self::Deposits => "bd",
            Self::Withdrawals => "bw",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HistoryWindow {
    start_ms: i64,
    end_ms: i64,
    offset: u64,
    limit: u32,
}

pub(super) const fn is_wallet_feature(feature: Feature) -> bool {
    matches!(
        feature,
        Feature::AssetNetworks
            | Feature::DepositAddresses
            | Feature::DepositHistory
            | Feature::WithdrawalQuotes
            | Feature::Withdrawals
            | Feature::WithdrawalHistory
    )
}

fn check_wallet_venue(adapter: &BinanceAdapter, feature: Feature) -> Result<()> {
    if adapter.venue() == BinanceMarket::Spot {
        return Ok(());
    }
    Err(Error::unsupported(
        feature,
        EXCHANGE,
        "Wallet SAPI operates on the Spot/Funding wallets; build a spot adapter",
    ))
}

fn all_coins_request(adapter: &BinanceAdapter) -> Result<HttpRequest> {
    signed(
        adapter,
        HttpMethod::Get,
        "/sapi/v1/capital/config/getall",
        Vec::new(),
    )
}

async fn coin_config(adapter: &BinanceAdapter, asset: &str) -> Result<RawCoinConfig> {
    check_asset("asset", asset)?;
    let body = adapter.send_wallet(all_coins_request(adapter)?).await?;
    let coins: Vec<RawCoinConfig> = parse::json(&body, "capital/config/getall")?;

    coins
        .into_iter()
        .find(|coin| coin.coin.eq_ignore_ascii_case(asset))
        .ok_or_else(|| {
            Error::transfer(
                TransferErrorKind::NetworkUnavailable,
                format!("Binance publishes no deposit or withdrawal networks for {asset}"),
            )
        })
}

fn checked_network(coin: &RawCoinConfig, requested: &Network) -> Result<CheckedNetwork> {
    let provider_id = provider_network(requested)?;
    let rules = coin
        .network_list
        .iter()
        .find(|network| network.network.eq_ignore_ascii_case(&provider_id))
        .cloned()
        .ok_or_else(|| {
            Error::transfer(
                TransferErrorKind::NetworkUnavailable,
                format!(
                    "Binance does not list {} on network {}",
                    coin.coin, provider_id
                ),
            )
        })?;

    Ok(CheckedNetwork {
        provider_id: rules.network.clone(),
        deposit_enabled: coin.deposit_all_enable && rules.deposit_enable,
        withdrawal_enabled: coin.withdraw_all_enable && rules.withdraw_enable && !rules.busy,
        rules,
    })
}

fn asset_networks_of(coin: &RawCoinConfig) -> Result<Vec<AssetNetwork>> {
    coin.network_list
        .iter()
        .map(|network| {
            Ok(AssetNetwork {
                exchange: Exchange::Binance,
                asset: coin.coin.to_ascii_uppercase(),
                network: canonical_network(&network.network),
                provider_id: network.network.clone(),
                deposit_enabled: coin.deposit_all_enable && network.deposit_enable,
                withdrawal_enabled: coin.withdraw_all_enable
                    && network.withdraw_enable
                    && !network.busy,
                withdrawal_fee: Some(WithdrawalFee::Fixed(parse::decimal(
                    &network.withdraw_fee,
                    "withdrawFee",
                )?)),
                minimum_withdrawal: Some(parse::decimal(&network.withdraw_min, "withdrawMin")?),
                maximum_withdrawal: Some(parse::decimal(&network.withdraw_max, "withdrawMax")?),
                memo_required: network.memo_required(),
            })
        })
        .collect()
}

pub(super) async fn asset_networks(
    adapter: &BinanceAdapter,
    asset: &str,
) -> Result<Vec<AssetNetwork>> {
    check_wallet_venue(adapter, Feature::AssetNetworks)?;
    let coin = coin_config(adapter, asset).await?;
    asset_networks_of(&coin)
}

/// Reads all Wallet SAPI coin configuration entries without selecting one
/// asset or discarding network fields that the common transfer API cannot use.
pub(super) async fn all_coins_information(
    adapter: &BinanceAdapter,
) -> Result<Vec<BinanceCoinInformation>> {
    check_wallet_venue(adapter, Feature::AssetNetworks)?;
    let body = adapter.send_wallet(all_coins_request(adapter)?).await?;
    let response: serde_json::Value = parse::json(&body, "capital/config/getall")?;
    let coins = response
        .as_array()
        .ok_or_else(|| Error::decode("Binance capital/config/getall response is not an array"))?;
    coins.iter().map(coin_information_from_value).collect()
}

fn coin_information_from_value(value: &serde_json::Value) -> Result<BinanceCoinInformation> {
    let raw: RawCoinConfig = serde_json::from_value(value.clone())
        .map_err(|error| Error::decode(format!("unreadable coin information: {error}")))?;
    let network_values = value
        .get("networkList")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| Error::decode("Binance coin information has no networkList array"))?;
    if raw.network_list.len() != network_values.len() {
        return Err(Error::decode(
            "Binance coin networks could not be paired with raw entries",
        ));
    }

    let networks = raw
        .network_list
        .iter()
        .zip(network_values)
        .map(|(network, raw_json)| {
            Ok(BinanceCoinNetworkInformation {
                network: network.network.clone(),
                deposit_enabled: network.deposit_enable,
                withdraw_enabled: network.withdraw_enable,
                busy: network.busy,
                withdrawal_integer_multiple: decimal_nonempty(
                    &network.withdraw_integer_multiple,
                    "withdrawIntegerMultiple",
                )?,
                withdrawal_fee: decimal_nonempty(&network.withdraw_fee, "withdrawFee")?,
                minimum_withdrawal: decimal_nonempty(&network.withdraw_min, "withdrawMin")?,
                maximum_withdrawal: decimal_nonempty(&network.withdraw_max, "withdrawMax")?,
                withdrawal_tag: network.withdraw_tag,
                is_default: network.is_default,
                minimum_confirmations: network.min_confirm,
                unlock_confirmations: network.un_lock_confirm,
                contract_address: network.contract_address.clone(),
                raw_json: parse::canonical_json(raw_json, "coin network information")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BinanceCoinInformation {
        coin: raw.coin,
        deposit_all_enabled: raw.deposit_all_enable,
        withdraw_all_enabled: raw.withdraw_all_enable,
        name: raw.name,
        free: decimal_option(raw.free.as_deref(), "free")?,
        locked: decimal_option(raw.locked.as_deref(), "locked")?,
        freeze: decimal_option(raw.freeze.as_deref(), "freeze")?,
        withdrawing: decimal_option(raw.withdrawing.as_deref(), "withdrawing")?,
        is_legal_money: raw.is_legal_money,
        trading: raw.trading,
        networks,
        raw_json: parse::canonical_json(value, "coin information")?,
    })
}

fn decimal_nonempty(value: &str, field: &'static str) -> Result<Option<Decimal>> {
    (!value.is_empty())
        .then(|| parse::decimal(value, field))
        .transpose()
}

fn decimal_option(value: Option<&str>, field: &'static str) -> Result<Option<Decimal>> {
    value.map(|value| parse::decimal(value, field)).transpose()
}

fn deposit_address_request(
    adapter: &BinanceAdapter,
    request: &DepositAddressRequest,
    provider_network: &str,
) -> Result<HttpRequest> {
    if provider_network.eq_ignore_ascii_case("LIGHTNING") && request.amount.is_none() {
        return Err(Error::invalid_request(
            "amount",
            "Binance requires an amount when requesting a Lightning invoice",
        ));
    }
    if request.amount.is_some_and(|amount| amount <= Decimal::ZERO) {
        return Err(Error::invalid_request(
            "amount",
            "deposit-address amount must be greater than zero",
        ));
    }

    let mut params = vec![
        ("coin", request.asset.clone()),
        ("network", provider_network.to_string()),
    ];
    if let Some(amount) = request.amount {
        params.push(("amount", amount.to_string()));
    }

    signed(
        adapter,
        HttpMethod::Get,
        "/sapi/v1/capital/deposit/address",
        params,
    )
}

pub(super) async fn deposit_address(
    adapter: &BinanceAdapter,
    request: &DepositAddressRequest,
) -> Result<DepositAddress> {
    check_wallet_venue(adapter, Feature::DepositAddresses)?;
    check_asset("asset", &request.asset)?;
    let coin = coin_config(adapter, &request.asset).await?;
    let network = checked_network(&coin, &request.network)?;
    if !network.deposit_enabled {
        return Err(Error::transfer(
            TransferErrorKind::NetworkUnavailable,
            format!(
                "Binance {} deposits are currently unavailable on {}",
                request.asset, network.provider_id
            ),
        ));
    }

    let body = adapter
        .send_wallet(deposit_address_request(
            adapter,
            request,
            &network.provider_id,
        )?)
        .await?;
    let raw: RawDepositAddress = parse::json(&body, "capital/deposit/address")?;
    if !raw.coin.eq_ignore_ascii_case(&request.asset) {
        return Err(Error::decode(format!(
            "capital/deposit/address returned coin {} for {}",
            raw.coin, request.asset
        )));
    }

    Ok(DepositAddress {
        exchange: Exchange::Binance,
        asset: raw.coin.to_ascii_uppercase(),
        network: request.network.clone(),
        address: nonempty(raw.address),
        memo: nonempty(raw.tag),
    })
}

fn validate_withdrawal_rules(
    request: &WithdrawRequest,
    network: &CheckedNetwork,
) -> Result<(Decimal, Decimal, Decimal)> {
    if !network.withdrawal_enabled {
        return Err(Error::transfer(
            TransferErrorKind::NetworkUnavailable,
            format!(
                "Binance {} withdrawals are currently unavailable on {}",
                request.asset, network.provider_id
            ),
        ));
    }

    let minimum = parse::decimal(&network.rules.withdraw_min, "withdrawMin")?;
    let maximum = parse::decimal(&network.rules.withdraw_max, "withdrawMax")?;
    let multiple = parse::decimal(
        &network.rules.withdraw_integer_multiple,
        "withdrawIntegerMultiple",
    )?;
    let fee = parse::decimal(&network.rules.withdraw_fee, "withdrawFee")?;
    if request.amount < minimum || request.amount > maximum {
        return Err(Error::transfer(
            TransferErrorKind::AmountOutOfRange,
            format!(
                "Binance {} withdrawal amount {} must be between {} and {} on {}",
                request.asset, request.amount, minimum, maximum, network.provider_id
            ),
        ));
    }
    if multiple > Decimal::ZERO && request.amount % multiple != Decimal::ZERO {
        return Err(Error::transfer(
            TransferErrorKind::AmountOutOfRange,
            format!(
                "Binance {} withdrawal amount {} must be a multiple of {} on {}",
                request.asset, request.amount, multiple, network.provider_id
            ),
        ));
    }

    let memo = request
        .destination
        .memo()
        .map(str::trim)
        .filter(|memo| !memo.is_empty());
    if network.rules.memo_required() && memo.is_none() {
        return Err(Error::transfer(
            TransferErrorKind::MemoRequired,
            format!(
                "Binance {} withdrawals on {} require a memo or tag",
                request.asset, network.provider_id
            ),
        ));
    }
    if !network.rules.memo_required() && memo.is_some() {
        return Err(Error::invalid_request(
            "destination.memo",
            format!(
                "Binance network {} does not support a memo or tag; omit it",
                network.provider_id
            ),
        ));
    }

    Ok((fee, minimum, maximum))
}

fn api_permissions_request(adapter: &BinanceAdapter) -> Result<HttpRequest> {
    signed(
        adapter,
        HttpMethod::Get,
        "/sapi/v1/account/apiRestrictions",
        Vec::new(),
    )
}

fn withdraw_addresses_request(adapter: &BinanceAdapter) -> Result<HttpRequest> {
    signed(
        adapter,
        HttpMethod::Get,
        "/sapi/v1/capital/withdraw/address/list",
        Vec::new(),
    )
}

fn questionnaire_request(adapter: &BinanceAdapter) -> Result<HttpRequest> {
    signed(
        adapter,
        HttpMethod::Get,
        "/sapi/v1/localentity/questionnaire-requirements",
        Vec::new(),
    )
}

/// Reads the complete Wallet SAPI API-key permission response.
pub(super) async fn api_key_permissions(
    adapter: &BinanceAdapter,
) -> Result<BinanceApiKeyPermissions> {
    check_wallet_venue(adapter, Feature::WithdrawalQuotes)?;
    let body = adapter
        .send_wallet(api_permissions_request(adapter)?)
        .await?;
    let response: serde_json::Value = parse::json(&body, "account/apiRestrictions")?;
    if !response.is_object() {
        return Err(Error::decode(
            "Binance account/apiRestrictions response is not an object",
        ));
    }
    let raw: RawApiPermissions = serde_json::from_value(response.clone())
        .map_err(|error| Error::decode(format!("unreadable API key permissions: {error}")))?;

    Ok(BinanceApiKeyPermissions {
        ip_restrict: raw.ip_restrict,
        create_time: raw.create_time.map(parse::millis),
        enable_reading: raw.enable_reading,
        enable_withdrawals: raw.enable_withdrawals,
        enable_internal_transfer: raw.enable_internal_transfer,
        enable_margin: raw.enable_margin,
        enable_spot_and_margin_trading: raw.enable_spot_and_margin_trading,
        enable_futures: raw.enable_futures,
        permits_universal_transfer: raw.permits_universal_transfer,
        enable_vanilla_options: raw.enable_vanilla_options,
        enable_fix_api_trade: raw.enable_fix_api_trade,
        enable_fix_read_only: raw.enable_fix_read_only,
        enable_portfolio_margin_trading: raw.enable_portfolio_margin_trading,
        raw_json: parse::canonical_json(&response, "API key permissions")?,
    })
}

/// Reads registered withdrawal addresses without reducing them to a boolean
/// preflight check.
pub(super) async fn withdraw_address_list(
    adapter: &BinanceAdapter,
) -> Result<Vec<BinanceWithdrawalAddress>> {
    check_wallet_venue(adapter, Feature::WithdrawalQuotes)?;
    let body = adapter
        .send_wallet(withdraw_addresses_request(adapter)?)
        .await?;
    withdraw_address_list_from_body(&body)
}

fn withdraw_address_list_from_body(body: &str) -> Result<Vec<BinanceWithdrawalAddress>> {
    let response: serde_json::Value = parse::json(body, "capital/withdraw/address/list")?;
    let entries = response.as_array().ok_or_else(|| {
        Error::decode("Binance capital/withdraw/address/list response is not an array")
    })?;

    entries
        .iter()
        .map(|entry| {
            let raw: RawWithdrawAddress =
                serde_json::from_value(entry.clone()).map_err(|error| {
                    Error::decode(format!("unreadable withdrawal address entry: {error}"))
                })?;
            Ok(BinanceWithdrawalAddress {
                address: raw.address,
                address_tag: nonempty(raw.address_tag),
                coin: raw.coin,
                network: raw.network,
                white_status: raw.white_status,
                name: raw.name,
                origin: raw.origin,
                origin_type: raw.origin_type,
                raw_json: parse::canonical_json(entry, "withdrawal address entry")?,
            })
        })
        .collect()
}

/// Reads the Travel Rule questionnaire requirement without reducing its country
/// code to the common required/not-required preflight enum.
pub(super) async fn questionnaire_requirements(
    adapter: &BinanceAdapter,
) -> Result<BinanceQuestionnaireRequirements> {
    check_wallet_venue(adapter, Feature::WithdrawalQuotes)?;
    let body = adapter.send_wallet(questionnaire_request(adapter)?).await?;
    questionnaire_requirements_from_body(&body)
}

fn questionnaire_requirements_from_body(body: &str) -> Result<BinanceQuestionnaireRequirements> {
    let response: serde_json::Value = parse::json(body, "localentity/questionnaire-requirements")?;
    if !response.is_object() {
        return Err(Error::decode(
            "Binance questionnaire-requirements response is not an object",
        ));
    }
    let raw: RawQuestionnaireRequirements =
        serde_json::from_value(response.clone()).map_err(|error| {
            Error::decode(format!("unreadable questionnaire requirements: {error}"))
        })?;

    Ok(BinanceQuestionnaireRequirements {
        questionnaire_country_code: raw.questionnaire_country_code,
        raw_json: parse::canonical_json(&response, "questionnaire requirements")?,
    })
}

fn allowlist_status(
    entries: &[RawWithdrawAddress],
    request: &WithdrawRequest,
    provider_network: &str,
) -> bool {
    let memo = request.destination.memo().unwrap_or_default();
    entries
        .iter()
        .find(|entry| {
            entry.coin.eq_ignore_ascii_case(&request.asset)
                && entry.network.eq_ignore_ascii_case(provider_network)
                && entry.address == request.destination.address()
                && entry.address_tag == memo
        })
        .is_some_and(|entry| entry.white_status)
}

fn travel_rule(requirements: &RawQuestionnaireRequirements) -> TravelRuleRequirement {
    if requirements
        .questionnaire_country_code
        .eq_ignore_ascii_case("NIL")
    {
        TravelRuleRequirement::NotRequired
    } else {
        TravelRuleRequirement::Required { consent_url: None }
    }
}

async fn checked_withdrawal(
    adapter: &BinanceAdapter,
    request: &WithdrawRequest,
) -> Result<CheckedWithdrawal> {
    check_asset("asset", &request.asset)?;
    if request.amount <= Decimal::ZERO {
        return Err(Error::invalid_request(
            "amount",
            "withdrawal amount must be greater than zero",
        ));
    }
    if !request
        .asset
        .eq_ignore_ascii_case(request.destination.asset())
    {
        return Err(Error::transfer(
            TransferErrorKind::AssetMismatch,
            "withdrawal asset differs from destination asset",
        ));
    }
    if !requested_networks_match(&request.network, request.destination.network())? {
        return Err(Error::transfer(
            TransferErrorKind::NetworkMismatch,
            "withdrawal network differs from destination network",
        ));
    }
    let address = request.destination.address();
    if address.trim().is_empty() {
        return Err(Error::transfer(
            TransferErrorKind::DestinationUnavailable,
            "withdrawal destination address must not be empty",
        ));
    }
    if address != address.trim() {
        return Err(Error::invalid_request(
            "destination.address",
            "withdrawal destination address must not contain surrounding whitespace",
        ));
    }

    let coin = coin_config(adapter, &request.asset).await?;
    let network = checked_network(&coin, &request.network)?;
    let (fee, minimum, maximum) = validate_withdrawal_rules(request, &network)?;

    let permissions_body = adapter
        .send_wallet(api_permissions_request(adapter)?)
        .await?;
    let permissions: RawApiPermissions = parse::json(&permissions_body, "account/apiRestrictions")?;
    if !permissions.enable_withdrawals {
        return Err(Error::auth(
            "the configured Binance API key does not have withdrawal permission",
        ));
    }

    let address_body = adapter
        .send_wallet(withdraw_addresses_request(adapter)?)
        .await?;
    let addresses: Vec<RawWithdrawAddress> =
        parse::json(&address_body, "capital/withdraw/address/list")?;
    let address_allowed = allowlist_status(&addresses, request, &network.provider_id);

    let questionnaire_body = adapter.send_wallet(questionnaire_request(adapter)?).await?;
    let requirements: RawQuestionnaireRequirements = parse::json(
        &questionnaire_body,
        "localentity/questionnaire-requirements",
    )?;

    Ok(CheckedWithdrawal {
        provider_network: network.provider_id,
        fee,
        quote: WithdrawalQuote {
            fee: Some(fee),
            expected_receive: (request.amount >= fee).then_some(request.amount - fee),
            minimum_amount: Some(minimum),
            maximum_amount: Some(maximum),
            address_allowed: Some(address_allowed),
            travel_rule: travel_rule(&requirements),
            expires_at: None,
        },
    })
}

pub(super) async fn prepare_withdrawal(
    adapter: &BinanceAdapter,
    request: &WithdrawRequest,
) -> Result<WithdrawalQuote> {
    check_wallet_venue(adapter, Feature::WithdrawalQuotes)?;
    Ok(checked_withdrawal(adapter, request).await?.quote)
}

fn withdraw_request(
    adapter: &BinanceAdapter,
    request: &WithdrawRequest,
    provider_network: &str,
) -> Result<HttpRequest> {
    let mut params = vec![
        ("coin", request.asset.clone()),
        ("address", request.destination.address().to_string()),
        ("amount", request.amount.to_string()),
        ("network", provider_network.to_string()),
        // This adapter's Wallet contract names the Spot wallet explicitly;
        // Binance's omitted default follows a mutable UI account setting.
        ("walletType", "0".to_string()),
    ];
    if let Some(client_id) = request.client_id.as_ref() {
        params.push(("withdrawOrderId", client_id.clone()));
    }
    if let Some(memo) = request
        .destination
        .memo()
        .map(str::trim)
        .filter(|memo| !memo.is_empty())
    {
        params.push(("addressTag", memo.to_string()));
    }

    signed(
        adapter,
        HttpMethod::Post,
        "/sapi/v1/capital/withdraw/apply",
        params,
    )
}

pub(super) async fn withdraw(
    adapter: &BinanceAdapter,
    request: &WithdrawRequest,
) -> Result<Withdrawal> {
    check_wallet_venue(adapter, Feature::Withdrawals)?;
    // Re-check live rules immediately before the one and only submission. A
    // caller can invoke `withdraw` without first calling `prepare_withdrawal`.
    let checked = checked_withdrawal(adapter, request).await?;
    validate_common_submission(&checked.quote)?;

    // `send_wallet` sends exactly once. An ambiguous transport failure is
    // returned to the caller and this financial write is never replayed here.
    let body = adapter
        .send_wallet(withdraw_request(
            adapter,
            request,
            &checked.provider_network,
        )?)
        .await?;
    let raw: RawWithdrawAck = parse::json(&body, "capital/withdraw/apply")?;

    Ok(Withdrawal {
        id: raw.id,
        asset: request.asset.to_ascii_uppercase(),
        network: Some(request.network.clone()),
        provider_network: Some(checked.provider_network),
        amount: request.amount,
        fee: Some(checked.fee),
        destination: Some(request.destination.clone()),
        status: WithdrawalStatus::Pending,
        provider_status: "accepted".to_string(),
        tx_id: None,
        created_at: Some(Timestamp::now()),
    })
}

fn validate_common_submission(quote: &WithdrawalQuote) -> Result<()> {
    match quote.address_allowed {
        Some(true) => {}
        Some(false) => {
            return Err(Error::transfer(
                TransferErrorKind::AddressNotAllowed,
                "the destination is present in Binance's address book but is not allowed",
            ));
        }
        None => {
            return Err(Error::transfer(
                TransferErrorKind::AddressNotAllowed,
                "the destination is not registered in Binance's withdrawal address book",
            ));
        }
    }
    if matches!(quote.travel_rule, TravelRuleRequirement::Required { .. }) {
        return Err(Error::transfer(
            TransferErrorKind::TravelRuleRequired,
            "Binance requires a local-entity questionnaire; use the provider-specific Travel Rule service",
        ));
    }
    Ok(())
}

fn history_window(request: &TransferHistoryRequest, kind: HistoryKind) -> Result<HistoryWindow> {
    let limit = request.limit.unwrap_or(DEFAULT_HISTORY_LIMIT);
    if !(1..=MAX_HISTORY_LIMIT).contains(&limit) {
        return Err(Error::invalid_request(
            "limit",
            format!("Binance serves 1 to {MAX_HISTORY_LIMIT} transfer records per page"),
        ));
    }

    if let Some(cursor) = &request.cursor {
        return decode_history_cursor(cursor, kind, limit);
    }
    let end_ms = now_millis();
    Ok(HistoryWindow {
        start_ms: end_ms.saturating_sub(HISTORY_WINDOW_MS),
        end_ms,
        offset: 0,
        limit,
    })
}

fn encode_history_cursor(kind: HistoryKind, window: HistoryWindow, offset: u64) -> Cursor {
    Cursor::new(format!(
        "{}:{}:{}:{}",
        kind.cursor_tag(),
        window.start_ms,
        window.end_ms,
        offset
    ))
}

fn decode_history_cursor(cursor: &Cursor, kind: HistoryKind, limit: u32) -> Result<HistoryWindow> {
    let mut fields = cursor.as_str().split(':');
    let parsed = match (
        fields.next(),
        fields.next().and_then(|value| value.parse::<i64>().ok()),
        fields.next().and_then(|value| value.parse::<i64>().ok()),
        fields.next().and_then(|value| value.parse::<u64>().ok()),
        fields.next(),
    ) {
        (Some(tag), Some(start_ms), Some(end_ms), Some(offset), None)
            if tag == kind.cursor_tag()
                && start_ms <= end_ms
                && end_ms.saturating_sub(start_ms) <= HISTORY_WINDOW_MS =>
        {
            Some(HistoryWindow {
                start_ms,
                end_ms,
                offset,
                limit,
            })
        }
        _ => None,
    };
    parsed.ok_or_else(|| {
        Error::invalid_request(
            "cursor",
            "not a Binance transfer-history cursor for this operation",
        )
    })
}

fn history_request(
    adapter: &BinanceAdapter,
    request: &TransferHistoryRequest,
    kind: HistoryKind,
    window: HistoryWindow,
) -> Result<HttpRequest> {
    if let Some(asset) = request.asset.as_deref() {
        check_asset("asset", asset)?;
    }
    let mut params = Vec::new();
    if let Some(asset) = request.asset.as_ref() {
        params.push(("coin", asset.clone()));
    }
    params.extend([
        ("startTime", window.start_ms.to_string()),
        ("endTime", window.end_ms.to_string()),
        ("offset", window.offset.to_string()),
        ("limit", window.limit.to_string()),
    ]);
    let path = match kind {
        HistoryKind::Deposits => "/sapi/v1/capital/deposit/hisrec",
        HistoryKind::Withdrawals => "/sapi/v1/capital/withdraw/history",
    };
    signed(adapter, HttpMethod::Get, path, params)
}

fn next_history_cursor(kind: HistoryKind, window: HistoryWindow, raw_len: usize) -> Option<Cursor> {
    (raw_len >= window.limit as usize)
        .then(|| encode_history_cursor(kind, window, window.offset.saturating_add(raw_len as u64)))
}

/// Builds the provider-specific deposit-history request without manufacturing a
/// common cursor or silently replacing Binance's status/time filters.
fn provider_deposit_history_request(
    adapter: &BinanceAdapter,
    request: &BinanceDepositHistoryRequest,
) -> Result<HttpRequest> {
    let mut params = provider_history_params(
        request.coin.as_deref(),
        request.start_time,
        request.end_time,
        request.offset,
        request.limit,
    )?;
    if let Some(status) = request.status {
        params.push(("status", status.to_string()));
    }
    if let Some(tx_id) = request.tx_id.as_ref() {
        if tx_id.is_empty() {
            return Err(Error::invalid_request(
                "tx_id",
                "must not be empty when provided",
            ));
        }
        params.push(("txId", tx_id.clone()));
    }
    if request.include_source {
        params.push(("includeSource", "true".to_string()));
    }
    signed(
        adapter,
        HttpMethod::Get,
        "/sapi/v1/capital/deposit/hisrec",
        params,
    )
}

/// Builds the provider-specific withdrawal-history request without
/// manufacturing a common cursor or silently replacing Binance's filters.
fn provider_withdraw_history_request(
    adapter: &BinanceAdapter,
    request: &BinanceWithdrawHistoryRequest,
) -> Result<HttpRequest> {
    if request.id_list.len() > 45 {
        return Err(Error::invalid_request(
            "id_list",
            "Binance accepts at most 45 withdrawal identifiers per request",
        ));
    }
    if request.id_list.iter().any(String::is_empty) {
        return Err(Error::invalid_request(
            "id_list",
            "must not contain an empty withdrawal identifier",
        ));
    }
    let mut params = provider_history_params(
        request.coin.as_deref(),
        request.start_time,
        request.end_time,
        request.offset,
        request.limit,
    )?;
    if let Some(withdraw_order_id) = request.withdraw_order_id.as_ref() {
        if withdraw_order_id.is_empty() {
            return Err(Error::invalid_request(
                "withdraw_order_id",
                "must not be empty when provided",
            ));
        }
        if let (Some(start), Some(end)) = (request.start_time, request.end_time) {
            let width = end.as_millis().saturating_sub(start.as_millis());
            if width > WITHDRAW_ORDER_ID_HISTORY_WINDOW_MS {
                return Err(Error::invalid_request(
                    "end_time",
                    "Binance withdrawal history is limited to fewer than 7 days when withdraw_order_id is set",
                ));
            }
        }
        params.push(("withdrawOrderId", withdraw_order_id.clone()));
    }
    if let Some(status) = request.status {
        params.push(("status", status.to_string()));
    }
    if !request.id_list.is_empty() {
        params.push(("idList", request.id_list.join(",")));
    }
    signed(
        adapter,
        HttpMethod::Get,
        "/sapi/v1/capital/withdraw/history",
        params,
    )
}

fn provider_history_params(
    coin: Option<&str>,
    start_time: Option<Timestamp>,
    end_time: Option<Timestamp>,
    offset: Option<u64>,
    limit: Option<u32>,
) -> Result<Vec<(&'static str, String)>> {
    if let Some(coin) = coin {
        check_asset("coin", coin)?;
    }
    if let Some(limit) = limit {
        if !(1..=MAX_HISTORY_LIMIT).contains(&limit) {
            return Err(Error::invalid_request(
                "limit",
                format!(
                    "Binance serves 1 to {MAX_HISTORY_LIMIT} history entries per call, not {limit}"
                ),
            ));
        }
    }
    let start = start_time.map(Timestamp::as_millis);
    let end = end_time.map(Timestamp::as_millis);
    if let (Some(start), Some(end)) = (start, end) {
        if end < start {
            return Err(Error::invalid_request(
                "end_time",
                "must not precede start_time at Binance millisecond precision",
            ));
        }
        if end.saturating_sub(start) > HISTORY_WINDOW_MS {
            return Err(Error::invalid_request(
                "end_time",
                "Binance Wallet history time windows must span fewer than 90 days",
            ));
        }
    }

    let mut params = Vec::new();
    if let Some(coin) = coin {
        params.push(("coin", coin.to_string()));
    }
    if let Some(start) = start {
        params.push(("startTime", start.to_string()));
    }
    if let Some(end) = end {
        params.push(("endTime", end.to_string()));
    }
    if let Some(offset) = offset {
        params.push(("offset", offset.to_string()));
    }
    if let Some(limit) = limit {
        params.push(("limit", limit.to_string()));
    }
    Ok(params)
}

/// Reads Wallet SAPI deposit history with Binance's native response fields.
pub(super) async fn deposit_history(
    adapter: &BinanceAdapter,
    request: &BinanceDepositHistoryRequest,
) -> Result<BinanceDepositHistory> {
    check_wallet_venue(adapter, Feature::DepositHistory)?;
    let body = adapter
        .send_wallet(provider_deposit_history_request(adapter, request)?)
        .await?;
    deposit_history_from_body(&body)
}

fn deposit_history_from_body(body: &str) -> Result<BinanceDepositHistory> {
    let response: serde_json::Value = parse::json(body, "capital/deposit/hisrec")?;
    let entries = response
        .as_array()
        .ok_or_else(|| Error::decode("Binance deposit-history response is not an array"))?;
    let entries = entries
        .iter()
        .map(|entry| {
            let raw: RawDeposit = serde_json::from_value(entry.clone()).map_err(|error| {
                Error::decode(format!("unreadable deposit history entry: {error}"))
            })?;
            Ok(BinanceDepositHistoryEntry {
                id: raw.id,
                amount: parse::decimal(&raw.amount, "amount")?,
                coin: raw.coin,
                network: raw.network,
                status: u32::try_from(raw.status).map_err(|_| {
                    Error::decode(format!(
                        "Binance deposit status must be non-negative, got {}",
                        raw.status
                    ))
                })?,
                address: nonempty(raw.address),
                address_tag: nonempty(raw.address_tag),
                tx_id: nonempty(raw.tx_id),
                insert_time: parse::millis(raw.insert_time),
                complete_time: raw.complete_time.map(parse::millis),
                transfer_type: raw
                    .transfer_type
                    .map(|value| {
                        u32::try_from(value).map_err(|_| {
                            Error::decode(format!(
                                "Binance deposit transfer type must be non-negative, got {value}"
                            ))
                        })
                    })
                    .transpose()?,
                source_address: raw.source_address.and_then(nonempty),
                raw_json: parse::canonical_json(entry, "deposit history entry")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(BinanceDepositHistory {
        entries,
        raw_json: parse::canonical_json(&response, "deposit history")?,
    })
}

/// Reads Wallet SAPI withdrawal history with Binance's native response fields.
pub(super) async fn withdraw_history(
    adapter: &BinanceAdapter,
    request: &BinanceWithdrawHistoryRequest,
) -> Result<BinanceWithdrawHistory> {
    check_wallet_venue(adapter, Feature::WithdrawalHistory)?;
    let body = adapter
        .send_wallet(provider_withdraw_history_request(adapter, request)?)
        .await?;
    withdraw_history_from_body(&body)
}

fn withdraw_history_from_body(body: &str) -> Result<BinanceWithdrawHistory> {
    let response: serde_json::Value = parse::json(body, "capital/withdraw/history")?;
    let entries = response
        .as_array()
        .ok_or_else(|| Error::decode("Binance withdrawal-history response is not an array"))?;
    let entries = entries
        .iter()
        .map(|entry| {
            let raw: RawWithdrawal = serde_json::from_value(entry.clone()).map_err(|error| {
                Error::decode(format!("unreadable withdrawal history entry: {error}"))
            })?;
            Ok(BinanceWithdrawHistoryEntry {
                id: raw.id,
                amount: parse::decimal(&raw.amount, "amount")?,
                transaction_fee: parse::decimal(&raw.transaction_fee, "transactionFee")?,
                coin: raw.coin,
                status: u32::try_from(raw.status).map_err(|_| {
                    Error::decode(format!("Binance withdrawal status must be non-negative, got {}", raw.status))
                })?,
                address: nonempty(raw.address),
                tx_id: nonempty(raw.tx_id),
                apply_time: raw.apply_time,
                network: raw.network.and_then(nonempty),
                withdraw_order_id: raw.withdraw_order_id,
                info: raw.info,
                transfer_type: raw
                    .transfer_type
                    .map(|value| u32::try_from(value).map_err(|_| Error::decode(format!("Binance withdrawal transfer type must be non-negative, got {value}"))))
                    .transpose()?,
                confirm_no: raw
                    .confirm_no
                    .map(|value| u32::try_from(value).map_err(|_| Error::decode(format!("Binance withdrawal confirmation count must be non-negative, got {value}"))))
                    .transpose()?,
                wallet_type: raw
                    .wallet_type
                    .map(|value| u32::try_from(value).map_err(|_| Error::decode(format!("Binance wallet type must be non-negative, got {value}"))))
                    .transpose()?,
                tx_key: raw.tx_key,
                complete_time: raw.complete_time,
                raw_json: parse::canonical_json(entry, "withdrawal history entry")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(BinanceWithdrawHistory {
        entries,
        raw_json: parse::canonical_json(&response, "withdrawal history")?,
    })
}

pub(super) async fn deposits(
    adapter: &BinanceAdapter,
    request: &TransferHistoryRequest,
) -> Result<Page<Deposit>> {
    check_wallet_venue(adapter, Feature::DepositHistory)?;
    let window = history_window(request, HistoryKind::Deposits)?;
    let body = adapter
        .send_wallet(history_request(
            adapter,
            request,
            HistoryKind::Deposits,
            window,
        )?)
        .await?;
    let raw: Vec<RawDeposit> = parse::json(&body, "capital/deposit/hisrec")?;
    let next = next_history_cursor(HistoryKind::Deposits, window, raw.len());
    let mut items = raw.iter().map(deposit_of).collect::<Result<Vec<_>>>()?;
    filter_deposits(&mut items, request);

    Ok(Page { items, next })
}

pub(super) async fn withdrawals(
    adapter: &BinanceAdapter,
    request: &TransferHistoryRequest,
) -> Result<Page<Withdrawal>> {
    check_wallet_venue(adapter, Feature::WithdrawalHistory)?;
    let window = history_window(request, HistoryKind::Withdrawals)?;
    let body = adapter
        .send_wallet(history_request(
            adapter,
            request,
            HistoryKind::Withdrawals,
            window,
        )?)
        .await?;
    let raw: Vec<RawWithdrawal> = parse::json(&body, "capital/withdraw/history")?;
    let next = next_history_cursor(HistoryKind::Withdrawals, window, raw.len());
    let mut items = raw.iter().map(withdrawal_of).collect::<Result<Vec<_>>>()?;
    filter_withdrawals(&mut items, request);

    Ok(Page { items, next })
}

fn deposit_of(raw: &RawDeposit) -> Result<Deposit> {
    let provider_network = nonempty(raw.network.clone());
    Ok(Deposit {
        id: raw.id.clone(),
        asset: raw.coin.to_ascii_uppercase(),
        network: provider_network.as_deref().map(canonical_network),
        provider_network,
        amount: parse::decimal(&raw.amount, "amount")?,
        address: nonempty(raw.address.clone()),
        memo: nonempty(raw.address_tag.clone()),
        status: deposit_status(raw.status),
        provider_status: raw.status.to_string(),
        tx_id: nonempty(raw.tx_id.clone()),
        created_at: Some(parse::millis(raw.insert_time)),
    })
}

fn withdrawal_of(raw: &RawWithdrawal) -> Result<Withdrawal> {
    let provider_network = raw.network.clone().and_then(nonempty);
    let network = provider_network.as_deref().map(canonical_network);
    let destination = match (network.clone(), nonempty(raw.address.clone())) {
        (Some(network), Some(address)) => {
            Some(TransferDestination::Chain(crate::types::ChainDestination {
                asset: raw.coin.to_ascii_uppercase(),
                network,
                address,
                memo: None,
            }))
        }
        _ => None,
    };

    Ok(Withdrawal {
        id: raw.id.clone(),
        asset: raw.coin.to_ascii_uppercase(),
        network,
        provider_network,
        amount: parse::decimal(&raw.amount, "amount")?,
        fee: Some(parse::decimal(&raw.transaction_fee, "transactionFee")?),
        destination,
        status: withdrawal_status(raw.status),
        provider_status: raw.status.to_string(),
        tx_id: nonempty(raw.tx_id.clone()),
        created_at: raw
            .apply_time
            .as_deref()
            .map(parse_apply_time)
            .transpose()?,
    })
}

fn parse_apply_time(value: &str) -> Result<Timestamp> {
    let time = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S").map_err(|error| {
        Error::decode(format!("unreadable Binance applyTime `{value}`: {error}"))
    })?;
    Ok(Timestamp::from_millis(time.and_utc().timestamp_millis()))
}

const fn deposit_status(status: i64) -> DepositStatus {
    match status {
        0 | 8 => DepositStatus::Pending,
        1 | 6 => DepositStatus::Completed,
        2 | 7 => DepositStatus::Failed,
        _ => DepositStatus::Unknown,
    }
}

const fn withdrawal_status(status: i64) -> WithdrawalStatus {
    match status {
        0 | 2 => WithdrawalStatus::Pending,
        1 => WithdrawalStatus::Cancelled,
        4 => WithdrawalStatus::Processing,
        6 => WithdrawalStatus::Completed,
        3 | 5 => WithdrawalStatus::Failed,
        _ => WithdrawalStatus::Unknown,
    }
}

fn filter_deposits(items: &mut Vec<Deposit>, request: &TransferHistoryRequest) {
    items.retain(|item| {
        request
            .asset
            .as_ref()
            .is_none_or(|asset| item.asset.eq_ignore_ascii_case(asset))
            && request.network.as_ref().is_none_or(|network| {
                item.provider_network
                    .as_deref()
                    .is_some_and(|provider| provider_matches(network, provider))
            })
    });
}

fn filter_withdrawals(items: &mut Vec<Withdrawal>, request: &TransferHistoryRequest) {
    items.retain(|item| {
        request
            .asset
            .as_ref()
            .is_none_or(|asset| item.asset.eq_ignore_ascii_case(asset))
            && request.network.as_ref().is_none_or(|network| {
                item.provider_network
                    .as_deref()
                    .is_some_and(|provider| provider_matches(network, provider))
            })
    });
}

fn provider_matches(network: &Network, provider: &str) -> bool {
    provider_network(network).is_ok_and(|requested| requested.eq_ignore_ascii_case(provider))
}

fn requested_networks_match(left: &Network, right: &Network) -> Result<bool> {
    Ok(provider_network(left)?.eq_ignore_ascii_case(&provider_network(right)?))
}

fn provider_network(network: &Network) -> Result<Cow<'_, str>> {
    let id = match network {
        Network::Bitcoin => Cow::Borrowed("BTC"),
        Network::Ethereum => Cow::Borrowed("ETH"),
        Network::Arbitrum => Cow::Borrowed("ARBITRUM"),
        Network::BnbSmartChain => Cow::Borrowed("BSC"),
        Network::Tron => Cow::Borrowed("TRX"),
        Network::Solana => Cow::Borrowed("SOL"),
        Network::Polygon => Cow::Borrowed("MATIC"),
        Network::Base => Cow::Borrowed("BASE"),
        Network::Optimism => Cow::Borrowed("OPTIMISM"),
        Network::AvalancheC => Cow::Borrowed("AVAXC"),
        Network::XrpLedger => Cow::Borrowed("XRP"),
        Network::Stellar => Cow::Borrowed("XLM"),
        Network::Cosmos => Cow::Borrowed("ATOM"),
        Network::Aptos => Cow::Borrowed("APT"),
        Network::Sui => Cow::Borrowed("SUI"),
        Network::Ton => Cow::Borrowed("TON"),
        Network::Near => Cow::Borrowed("NEAR"),
        Network::Polkadot => Cow::Borrowed("DOT"),
        Network::Other(id) => {
            if id.is_empty()
                || !id
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || "_-".contains(character))
            {
                return Err(Error::invalid_request(
                    "network",
                    "a Binance provider network id must contain only ASCII letters, digits, `_`, or `-`",
                ));
            }
            Cow::Borrowed(id.as_str())
        }
    };
    Ok(id)
}

fn canonical_network(provider: &str) -> Network {
    match provider.to_ascii_uppercase().as_str() {
        "BTC" => Network::Bitcoin,
        "ETH" => Network::Ethereum,
        "ARBITRUM" => Network::Arbitrum,
        "BSC" => Network::BnbSmartChain,
        "TRX" => Network::Tron,
        "SOL" => Network::Solana,
        "MATIC" => Network::Polygon,
        "BASE" => Network::Base,
        "OPTIMISM" => Network::Optimism,
        "AVAXC" => Network::AvalancheC,
        "XRP" => Network::XrpLedger,
        "XLM" => Network::Stellar,
        "ATOM" => Network::Cosmos,
        "APT" => Network::Aptos,
        "SUI" => Network::Sui,
        "TON" => Network::Ton,
        "NEAR" => Network::Near,
        "DOT" => Network::Polkadot,
        // Lightning and every newly added provider route stay distinct rather
        // than being guessed to be equivalent to a settlement chain.
        _ => Network::Other(provider.to_string()),
    }
}

fn nonempty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChainDestination;

    const COINS: &str = r#"[
      {
        "coin": "BTC",
        "depositAllEnable": true,
        "withdrawAllEnable": true,
        "networkList": [
          {
            "network": "BTC",
            "depositEnable": true,
            "withdrawEnable": true,
            "withdrawIntegerMultiple": "0.00000001",
            "withdrawFee": "0.0001",
            "withdrawMin": "0.001",
            "withdrawMax": "10",
            "sameAddress": false,
            "withdrawTag": false,
            "busy": false
          },
          {
            "network": "LIGHTNING",
            "depositEnable": true,
            "withdrawEnable": true,
            "withdrawIntegerMultiple": "0.00000001",
            "withdrawFee": "0.000001",
            "withdrawMin": "0.00001",
            "withdrawMax": "1",
            "sameAddress": false,
            "withdrawTag": false,
            "busy": false
          }
        ]
      },
      {
        "coin": "ETH",
        "depositAllEnable": true,
        "withdrawAllEnable": true,
        "networkList": [
          {
            "network": "ETH",
            "depositEnable": true,
            "withdrawEnable": true,
            "withdrawIntegerMultiple": "0.0001",
            "withdrawFee": "0.01",
            "withdrawMin": "0.1",
            "withdrawMax": "100",
            "sameAddress": false,
            "withdrawTag": false,
            "busy": false
          }
        ]
      },
      {
        "coin": "XRP",
        "depositAllEnable": true,
        "withdrawAllEnable": true,
        "networkList": [
          {
            "network": "XRP",
            "depositEnable": true,
            "withdrawEnable": true,
            "withdrawIntegerMultiple": "0.1",
            "withdrawFee": "0.2",
            "withdrawMin": "10",
            "withdrawMax": "100000",
            "sameAddress": true,
            "withdrawTag": true,
            "busy": false
          }
        ]
      },
      {
        "coin": "NEW",
        "depositAllEnable": true,
        "withdrawAllEnable": true,
        "networkList": [
          {
            "network": "NEWNET",
            "depositEnable": true,
            "withdrawEnable": false,
            "withdrawIntegerMultiple": "1",
            "withdrawFee": "2",
            "withdrawMin": "10",
            "withdrawMax": "1000",
            "sameAddress": false,
            "withdrawTag": false,
            "busy": true
          }
        ]
      }
    ]"#;

    fn spot() -> BinanceAdapter {
        BinanceAdapter::spot().with_credentials("key", "secret")
    }

    fn coin(asset: &str) -> RawCoinConfig {
        parse::json::<Vec<RawCoinConfig>>(COINS, "test coins")
            .expect("valid official-shaped fixture")
            .into_iter()
            .find(|coin| coin.coin == asset)
            .expect("fixture coin")
    }

    fn destination(asset: &str, network: Network, memo: Option<&str>) -> TransferDestination {
        TransferDestination::Chain(ChainDestination {
            asset: asset.to_string(),
            network,
            address: "0x1111111111111111111111111111111111111111".to_string(),
            memo: memo.map(str::to_string),
        })
    }

    fn eth_withdrawal() -> WithdrawRequest {
        WithdrawRequest::new(
            "ETH",
            Network::Ethereum,
            Decimal::from(20),
            destination("ETH", Network::Ethereum, None),
        )
        .client_id("withdraw-once")
    }

    /// Query parameters before the clock-generated timestamp and signature.
    fn signed_params(request: &HttpRequest) -> String {
        let target = request.target();
        let (target, _) = target
            .split_once("&signature=")
            .expect("signed request has a signature");
        let (target, _) = target
            .rsplit_once("&timestamp=")
            .or_else(|| target.rsplit_once("?timestamp="))
            .expect("signed request has a timestamp");
        target.to_string()
    }

    #[test]
    fn every_wallet_operation_uses_the_documented_sapi_path() {
        let adapter = spot();
        assert_eq!(
            super::super::WALLET_REST_BASE_URL,
            "https://api.binance.com"
        );
        assert_ne!(
            super::super::WALLET_REST_BASE_URL,
            super::super::USD_M_REST_BASE_URL
        );
        assert_eq!(
            signed_params(&all_coins_request(&adapter).expect("all coins request")),
            "/sapi/v1/capital/config/getall"
        );
        assert_eq!(
            signed_params(&api_permissions_request(&adapter).expect("permissions request")),
            "/sapi/v1/account/apiRestrictions"
        );
        assert_eq!(
            signed_params(&withdraw_addresses_request(&adapter).expect("address-list request")),
            "/sapi/v1/capital/withdraw/address/list"
        );
        assert_eq!(
            signed_params(&questionnaire_request(&adapter).expect("questionnaire request")),
            "/sapi/v1/localentity/questionnaire-requirements"
        );

        let history = TransferHistoryRequest::new().asset("ETH").limit(25);
        let window = HistoryWindow {
            start_ms: 1_700_000_000_000,
            end_ms: 1_700_001_000_000,
            offset: 50,
            limit: 25,
        };
        assert_eq!(
            signed_params(
                &history_request(&adapter, &history, HistoryKind::Deposits, window)
                    .expect("deposit history request")
            ),
            "/sapi/v1/capital/deposit/hisrec?coin=ETH&startTime=1700000000000&endTime=1700001000000&offset=50&limit=25"
        );
        assert_eq!(
            signed_params(
                &history_request(&adapter, &history, HistoryKind::Withdrawals, window)
                    .expect("withdrawal history request")
            ),
            "/sapi/v1/capital/withdraw/history?coin=ETH&startTime=1700000000000&endTime=1700001000000&offset=50&limit=25"
        );

        let deposit = BinanceDepositHistoryRequest::new()
            .coin("ETH")
            .status(1)
            .start_time(Timestamp::from_millis(1_700_000_000_000))
            .end_time(Timestamp::from_millis(1_700_001_000_000))
            .offset(50)
            .limit(25)
            .tx_id("tx-1")
            .include_source();
        assert_eq!(
            signed_params(
                &provider_deposit_history_request(&adapter, &deposit)
                    .expect("provider deposit history request")
            ),
            "/sapi/v1/capital/deposit/hisrec?coin=ETH&startTime=1700000000000&endTime=1700001000000&offset=50&limit=25&status=1&txId=tx-1&includeSource=true"
        );

        let withdrawal = BinanceWithdrawHistoryRequest::new()
            .coin("ETH")
            .withdraw_order_id("withdraw-1")
            .status(6)
            .id_list(["id-1", "id-2"])
            .start_time(Timestamp::from_millis(1_700_000_000_000))
            .end_time(Timestamp::from_millis(1_700_001_000_000));
        assert_eq!(
            signed_params(
                &provider_withdraw_history_request(&adapter, &withdrawal)
                    .expect("provider withdrawal history request")
            ),
            "/sapi/v1/capital/withdraw/history?coin=ETH&startTime=1700000000000&endTime=1700001000000&withdrawOrderId=withdraw-1&status=6&idList=id-1%2Cid-2"
        );
    }

    #[test]
    fn provider_wallet_contracts_preserve_typed_fields_and_raw_json() {
        let coins: serde_json::Value = parse::json(
            r#"[{
              "coin":"ETH","depositAllEnable":true,"withdrawAllEnable":true,"name":"Ethereum",
              "free":"1.2","locked":"0.3","freeze":"0","withdrawing":"0","isLegalMoney":false,"trading":true,
              "networkList":[{"network":"ETH","depositEnable":true,"withdrawEnable":true,"withdrawIntegerMultiple":"0.0001","withdrawFee":"0.001","withdrawMin":"0.01","withdrawMax":"10","withdrawTag":false,"isDefault":true,"minConfirm":12,"unLockConfirm":64,"contractAddress":"0xabc","busy":false,"futureField":"kept"}],"futureField":true
            }]"#,
            "coin information fixture",
        )
        .expect("valid coin information fixture");
        let coin = coin_information_from_value(&coins[0]).expect("typed coin information");
        assert_eq!(coin.free, Some(Decimal::new(12, 1)));
        assert_eq!(coin.networks[0].minimum_confirmations, Some(12));
        assert!(coin.raw_json.contains("futureField"));

        let permissions = {
            let response: serde_json::Value = parse::json(
                r#"{"ipRestrict":true,"createTime":1700000000000,"enableReading":true,"enableWithdrawals":false,"enableInternalTransfer":true,"enableMargin":true,"enableFutures":true,"permitsUniversalTransfer":true,"enableVanillaOptions":true,"enableFixApiTrade":true,"enableFixReadOnly":true,"enableSpotAndMarginTrading":true,"enablePortfolioMarginTrading":true,"futureField":true}"#,
                "permission fixture",
            )
            .expect("valid permission fixture");
            let raw: RawApiPermissions =
                serde_json::from_value(response.clone()).expect("typed permission fixture");
            BinanceApiKeyPermissions {
                ip_restrict: raw.ip_restrict,
                create_time: raw.create_time.map(parse::millis),
                enable_reading: raw.enable_reading,
                enable_withdrawals: raw.enable_withdrawals,
                enable_internal_transfer: raw.enable_internal_transfer,
                enable_margin: raw.enable_margin,
                enable_spot_and_margin_trading: raw.enable_spot_and_margin_trading,
                enable_futures: raw.enable_futures,
                permits_universal_transfer: raw.permits_universal_transfer,
                enable_vanilla_options: raw.enable_vanilla_options,
                enable_fix_api_trade: raw.enable_fix_api_trade,
                enable_fix_read_only: raw.enable_fix_read_only,
                enable_portfolio_margin_trading: raw.enable_portfolio_margin_trading,
                raw_json: parse::canonical_json(&response, "permission fixture")
                    .expect("raw permissions"),
            }
        };
        assert!(permissions.enable_fix_read_only);
        assert!(permissions.raw_json.contains("futureField"));

        let deposits = deposit_history_from_body(
            r#"[{"id":"dep-1","amount":"1.2","coin":"ETH","network":"ETH","status":1,"address":"address","addressTag":"memo","txId":"tx","insertTime":1700000000000,"completeTime":1700000100000,"transferType":0,"sourceAddress":"source","futureField":true}]"#,
        )
        .expect("typed deposit history");
        assert_eq!(deposits.entries[0].transfer_type, Some(0));
        assert!(deposits.entries[0].raw_json.contains("futureField"));

        let withdrawals = withdraw_history_from_body(
            r#"[{"id":"wd-1","amount":"1.2","transactionFee":"0.01","coin":"ETH","status":6,"address":"address","txId":"tx","applyTime":"2026-08-10 12:34:56","network":"ETH","withdrawOrderId":"client","info":"ok","transferType":0,"confirmNo":12,"walletType":1,"txKey":"key","completeTime":"2026-08-10 12:35:56","futureField":true}]"#,
        )
        .expect("typed withdrawal history");
        assert_eq!(withdrawals.entries[0].confirm_no, Some(12));
        assert!(withdrawals.entries[0].raw_json.contains("futureField"));

        let address = withdraw_address_list_from_body(
            r#"[{"address":"address","addressTag":"memo","coin":"ETH","network":"ETH","whiteStatus":true,"name":"cold","origin":"user","originType":"others","futureField":true}]"#,
        )
        .expect("typed withdrawal address list");
        assert_eq!(address[0].name.as_deref(), Some("cold"));
        assert!(address[0].raw_json.contains("futureField"));

        let questionnaire = questionnaire_requirements_from_body(
            r#"{"questionnaireCountryCode":"KR","futureField":true}"#,
        )
        .expect("typed questionnaire requirement");
        assert_eq!(questionnaire.questionnaire_country_code, "KR");
        assert!(questionnaire.raw_json.contains("futureField"));
    }

    #[test]
    fn asset_networks_keep_provider_ids_and_unknown_routes_open() {
        let networks = asset_networks_of(&coin("NEW")).expect("valid network rules");

        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].provider_id, "NEWNET");
        assert_eq!(networks[0].network, Network::Other("NEWNET".to_string()));
        assert!(networks[0].deposit_enabled);
        assert!(!networks[0].withdrawal_enabled);
    }

    #[test]
    fn lightning_requires_and_signs_the_payment_amount() {
        let request = DepositAddressRequest::new("BTC", Network::Other("LIGHTNING".to_string()));
        assert!(matches!(
            deposit_address_request(&spot(), &request, "LIGHTNING"),
            Err(Error::InvalidRequest { field, .. }) if field == "amount"
        ));

        let request = request.amount(Decimal::new(125, 5));
        let built = deposit_address_request(&spot(), &request, "LIGHTNING")
            .expect("a Lightning invoice request");
        assert_eq!(
            signed_params(&built),
            "/sapi/v1/capital/deposit/address?coin=BTC&network=LIGHTNING&amount=0.00125"
        );
    }

    #[test]
    fn withdraw_tag_is_required_or_forbidden_exactly_as_published() {
        let xrp = checked_network(&coin("XRP"), &Network::XrpLedger).expect("XRP rules");
        let missing = WithdrawRequest::new(
            "XRP",
            Network::XrpLedger,
            Decimal::from(20),
            destination("XRP", Network::XrpLedger, None),
        );
        assert!(matches!(
            validate_withdrawal_rules(&missing, &xrp),
            Err(Error::Transfer {
                kind: TransferErrorKind::MemoRequired,
                ..
            })
        ));

        let tagged = WithdrawRequest::new(
            "XRP",
            Network::XrpLedger,
            Decimal::from(20),
            destination("XRP", Network::XrpLedger, Some("12345")),
        );
        validate_withdrawal_rules(&tagged, &xrp).expect("required tag is present");
        assert!(
            signed_params(&withdraw_request(&spot(), &tagged, "XRP").expect("tagged request"))
                .contains("&addressTag=12345")
        );

        let eth = checked_network(&coin("ETH"), &Network::Ethereum).expect("ETH rules");
        let forbidden = WithdrawRequest::new(
            "ETH",
            Network::Ethereum,
            Decimal::from(20),
            destination("ETH", Network::Ethereum, Some("must-not-be-sent")),
        );
        assert!(matches!(
            validate_withdrawal_rules(&forbidden, &eth),
            Err(Error::InvalidRequest { field, .. }) if field == "destination.memo"
        ));
        assert!(
            !signed_params(
                &withdraw_request(&spot(), &eth_withdrawal(), "ETH").expect("untagged request")
            )
            .contains("addressTag")
        );
    }

    #[test]
    fn deposit_and_withdrawal_histories_preserve_raw_status_and_network() {
        let deposit: RawDeposit = parse::json(
            r#"{
              "id":"dep-1","amount":"12.5","coin":"NEW","network":"NEWNET",
              "status":99,"address":"new-address","addressTag":"memo","txId":"tx-1",
              "insertTime":1700000000000
            }"#,
            "deposit fixture",
        )
        .expect("deposit fixture");
        let deposit = deposit_of(&deposit).expect("mapped deposit");
        assert_eq!(deposit.status, DepositStatus::Unknown);
        assert_eq!(deposit.provider_status, "99");
        assert_eq!(deposit.provider_network.as_deref(), Some("NEWNET"));
        assert_eq!(deposit.network, Some(Network::Other("NEWNET".to_string())));

        let withdrawal: RawWithdrawal = parse::json(
            r#"{
              "id":"wd-1","amount":"2","transactionFee":"0.1","coin":"ETH",
              "status":42,"address":"0x1111111111111111111111111111111111111111",
              "txId":"","applyTime":"2026-08-10 12:34:56"
            }"#,
            "withdrawal fixture",
        )
        .expect("withdrawal fixture");
        let withdrawal = withdrawal_of(&withdrawal).expect("mapped withdrawal");
        assert_eq!(withdrawal.status, WithdrawalStatus::Unknown);
        assert_eq!(withdrawal.provider_status, "42");
        assert_eq!(withdrawal.provider_network, None);
        assert_eq!(withdrawal.network, None);
        assert_eq!(withdrawal.destination, None);
    }

    #[test]
    fn questionnaire_country_code_controls_the_travel_rule_requirement() {
        assert_eq!(
            travel_rule(&RawQuestionnaireRequirements {
                questionnaire_country_code: "NIL".to_string(),
            }),
            TravelRuleRequirement::NotRequired
        );
        assert!(matches!(
            travel_rule(&RawQuestionnaireRequirements {
                questionnaire_country_code: "AE".to_string(),
            }),
            TravelRuleRequirement::Required { consent_url: None }
        ));

        let quote = WithdrawalQuote {
            fee: Some(Decimal::ONE),
            expected_receive: Some(Decimal::from(9)),
            minimum_amount: Some(Decimal::ONE),
            maximum_amount: Some(Decimal::from(100)),
            address_allowed: Some(true),
            travel_rule: TravelRuleRequirement::Required { consent_url: None },
            expires_at: None,
        };
        assert!(matches!(
            validate_common_submission(&quote),
            Err(Error::Transfer {
                kind: TransferErrorKind::TravelRuleRequired,
                ..
            })
        ));
    }

    #[test]
    fn withdrawal_submission_requires_a_confirmed_active_address_book_entry() {
        let quote = |address_allowed| WithdrawalQuote {
            fee: Some(Decimal::ONE),
            expected_receive: Some(Decimal::from(9)),
            minimum_amount: Some(Decimal::ONE),
            maximum_amount: Some(Decimal::from(100)),
            address_allowed,
            travel_rule: TravelRuleRequirement::NotRequired,
            expires_at: None,
        };

        for address_allowed in [None, Some(false)] {
            assert!(matches!(
                validate_common_submission(&quote(address_allowed)),
                Err(Error::Transfer {
                    kind: TransferErrorKind::AddressNotAllowed,
                    ..
                })
            ));
        }
        validate_common_submission(&quote(Some(true)))
            .expect("an active registered address is safe to submit");
    }

    #[test]
    fn an_unregistered_withdrawal_address_is_not_allowlisted() {
        let request = eth_withdrawal();
        let registered = RawWithdrawAddress {
            coin: "ETH".to_string(),
            network: "ETH".to_string(),
            address: "0x2222222222222222222222222222222222222222".to_string(),
            address_tag: String::new(),
            white_status: true,
            name: None,
            origin: None,
            origin_type: None,
        };

        assert!(!allowlist_status(&[registered], &request, "ETH"));
    }

    #[test]
    fn transfer_history_cursor_fixes_the_window_and_uses_raw_offset() {
        let request = TransferHistoryRequest::new().limit(2);
        let window = history_window(&request, HistoryKind::Deposits).expect("first page");
        let cursor = next_history_cursor(HistoryKind::Deposits, window, 2)
            .expect("a full raw page has a continuation");
        let resumed = decode_history_cursor(&cursor, HistoryKind::Deposits, 2)
            .expect("the same operation accepts its cursor");

        assert_eq!(resumed.start_ms, window.start_ms);
        assert_eq!(resumed.end_ms, window.end_ms);
        assert_eq!(resumed.offset, 2);
        assert!(decode_history_cursor(&cursor, HistoryKind::Withdrawals, 2).is_err());
    }

    #[test]
    fn usd_m_does_not_claim_account_wide_wallet_features() {
        let adapter = BinanceAdapter::usd_m_futures().with_credentials("key", "secret");
        for feature in [
            Feature::AssetNetworks,
            Feature::DepositAddresses,
            Feature::DepositHistory,
            Feature::WithdrawalQuotes,
            Feature::Withdrawals,
            Feature::WithdrawalHistory,
        ] {
            assert!(!crate::Adapter::supports(&adapter, feature), "{feature:?}");
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn an_ambiguous_withdraw_transport_failure_is_never_retried() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        };
        use std::thread;
        use std::time::Duration;

        let listener = TcpListener::bind("127.0.0.1:0").expect("local fixture server");
        listener
            .set_nonblocking(true)
            .expect("nonblocking fixture server");
        let address = listener.local_addr().expect("fixture address");
        let done = Arc::new(AtomicBool::new(false));
        let posts = Arc::new(AtomicUsize::new(0));
        let server_done = Arc::clone(&done);
        let server_posts = Arc::clone(&posts);
        let server = thread::spawn(move || {
            while !server_done.load(Ordering::SeqCst) {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(error) => panic!("fixture accept failed: {error}"),
                };
                stream
                    .set_nonblocking(false)
                    .expect("blocking fixture connection");
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("fixture timeout");
                let mut bytes = Vec::new();
                let mut buffer = [0_u8; 1_024];
                while !bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    let read = stream.read(&mut buffer).expect("fixture request");
                    if read == 0 {
                        break;
                    }
                    bytes.extend_from_slice(&buffer[..read]);
                }
                let request = String::from_utf8_lossy(&bytes);
                let line = request.lines().next().expect("request line");
                let mut words = line.split_whitespace();
                let method = words.next().expect("method");
                let target = words.next().expect("target");
                let path = target.split('?').next().expect("path");

                if method == "POST" && path == "/sapi/v1/capital/withdraw/apply" {
                    server_posts.fetch_add(1, Ordering::SeqCst);
                    // Closing without a response makes the submission outcome
                    // ambiguous. The adapter must return it, never replay it.
                    continue;
                }

                let body = match path {
                    "/sapi/v1/capital/config/getall" => COINS,
                    "/sapi/v1/account/apiRestrictions" => r#"{"enableWithdrawals":true}"#,
                    "/sapi/v1/capital/withdraw/address/list" => {
                        r#"[{"address":"0x1111111111111111111111111111111111111111","addressTag":"","coin":"ETH","network":"ETH","whiteStatus":true}]"#
                    }
                    "/sapi/v1/localentity/questionnaire-requirements" => {
                        r#"{"questionnaireCountryCode":"NIL"}"#
                    }
                    other => panic!("unexpected fixture path {other}"),
                };
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
                .expect("fixture response");
            }
        });

        let adapter = spot();
        adapter
            .wallet_http
            .set(
                crate::transport::HttpTransport::new(format!("http://{address}"))
                    .expect("fixture transport"),
            )
            .expect("unset wallet transport");
        let error = withdraw(&adapter, &eth_withdrawal())
            .await
            .expect_err("the fixture drops the write response");
        done.store(true, Ordering::SeqCst);
        server.join().expect("fixture server exits");

        assert!(matches!(error, Error::Transport { .. }));
        assert_eq!(posts.load(Ordering::SeqCst), 1);
    }
}
