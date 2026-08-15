//! Hyperliquid-specific data exposed by [`HyperliquidAdapter`].

use rust_decimal::Decimal;
use serde_json::Value;

use crate::error::{Error, Result};
use crate::types::{
    Balance, Candle, Deposit, DepositStatus, Market, MarketKind, Order, OrderBook, Ticker,
    Timestamp, Trade, Withdrawal, WithdrawalStatus,
};

use super::parse::{
    self, Asset, RawAssetCtx, RawBook, RawCandle, RawFundingHistory, RawLedgerUpdate, RawSpotMeta,
    RawSpotState, RawTrade, RawUserFunding, Universe,
};

/// One Hyperliquid candle snapshot entry, including its official trade count.
///
/// The normalized [`Candle`](crate::Candle) intentionally omits `trade_count`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidCandleSnapshot {
    /// Hyperliquid's native market name.
    pub coin: String,
    /// Resolved default-universe market.
    pub market: Market,
    /// Hyperliquid's interval spelling.
    pub interval: String,
    /// Opening timestamp.
    pub open_time: Timestamp,
    /// Inclusive closing timestamp.
    pub close_time: Timestamp,
    /// Opening price.
    pub open: Decimal,
    /// Highest price.
    pub high: Decimal,
    /// Lowest price.
    pub low: Decimal,
    /// Closing price.
    pub close: Decimal,
    /// Base-asset volume.
    pub volume: Decimal,
    /// Number of trades Hyperliquid aggregated into the candle, when supplied.
    pub trade_count: Option<u64>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// One price level in a Hyperliquid L2 book snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperliquidBookLevel {
    /// Level price.
    pub price: Decimal,
    /// Aggregate resting size.
    pub size: Decimal,
    /// Number of orders aggregated at this level, when supplied.
    pub order_count: Option<u64>,
}

/// A Hyperliquid L2 book snapshot, retaining each level's order count.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidL2Book {
    /// Hyperliquid's native market name.
    pub coin: String,
    /// Resolved default-universe market.
    pub market: Market,
    /// Snapshot timestamp.
    pub time: Timestamp,
    /// Bids in provider response order.
    pub bids: Vec<HyperliquidBookLevel>,
    /// Asks in provider response order.
    pub asks: Vec<HyperliquidBookLevel>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// One execution from Hyperliquid's `recentTrades` Info response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidRecentTrade {
    /// Hyperliquid's native market name.
    pub coin: String,
    /// Resolved default-universe market.
    pub market: Market,
    /// Provider taker-side string, currently `B` or `A`.
    pub side: String,
    /// Execution price.
    pub price: Decimal,
    /// Executed base quantity.
    pub size: Decimal,
    /// Execution timestamp.
    pub time: Timestamp,
    /// Hyperliquid trade identifier.
    pub trade_id: String,
    /// Transaction hash, when supplied.
    pub hash: Option<String>,
    /// Accounts involved in the execution in provider order.
    pub users: Vec<String>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// One native Hyperliquid `trades` stream event.
///
/// [`Self::common`] keeps the portable trade projection while `provider`
/// preserves the native transaction and participant fields.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidTradeEvent {
    /// Portable trade projection used by the common stream API.
    pub common: Trade,
    /// Native Hyperliquid trade data.
    pub provider: HyperliquidRecentTrade,
}

/// One native Hyperliquid `l2Book` stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidOrderBookEvent {
    /// Portable order-book projection used by the common stream API.
    pub common: OrderBook,
    /// Native levels, including per-level order counts.
    pub provider: HyperliquidL2Book,
}

/// One native Hyperliquid `candle` stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidCandleEvent {
    /// Portable candle projection used by the common stream API.
    pub common: Candle,
    /// Native candle fields, including its inclusive close time and trade count.
    pub provider: HyperliquidCandleSnapshot,
}

/// One native Hyperliquid `activeAssetCtx` or `activeSpotAssetCtx` event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidAssetContextEvent {
    /// Portable ticker projection used by the common stream API.
    pub common: Ticker,
    /// Hyperliquid's native market name.
    pub coin: String,
    /// Mid price, when published.
    pub mid_price: Option<Decimal>,
    /// Mark price, when published.
    pub mark_price: Option<Decimal>,
    /// Previous-day price, when published.
    pub previous_day_price: Option<Decimal>,
    /// Day base volume, when published.
    pub day_base_volume: Option<Decimal>,
    /// Day notional volume, when published.
    pub day_notional_volume: Option<Decimal>,
    /// Oracle price for perpetual markets, when published.
    pub oracle_price: Option<Decimal>,
    /// Funding rate for perpetual markets, when published.
    pub funding_rate: Option<Decimal>,
    /// Open interest for perpetual markets, when published.
    pub open_interest: Option<Decimal>,
    /// Circulating supply for spot markets, when published.
    pub circulating_supply: Option<Decimal>,
    /// Total supply for spot markets, when published.
    pub total_supply: Option<Decimal>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// One native Hyperliquid `orderUpdates` stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidOrderUpdate {
    /// Portable order projection used by the common account stream API.
    pub common: Order,
    /// Hyperliquid's native market name.
    pub coin: String,
    /// Provider side string, currently `A` or `B`.
    pub side: String,
    /// Provider limit price.
    pub limit_price: Decimal,
    /// Remaining provider order size.
    pub remaining_size: Decimal,
    /// Original provider order size.
    pub original_size: Decimal,
    /// Provider order identifier.
    pub order_id: u64,
    /// Provider acceptance timestamp.
    pub accepted_at: Timestamp,
    /// Provider client order identifier, when supplied.
    pub client_order_id: Option<String>,
    /// Provider status string, preserved without normalization.
    pub status: String,
    /// Provider status-update timestamp, when supplied.
    pub status_at: Option<Timestamp>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// One native balance in a Hyperliquid `spotState` stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidSpotStateBalance {
    /// Portable balance projection used by the common account stream API.
    pub common: Balance,
    /// Native balance details including token and entry notional.
    pub provider: HyperliquidSpotBalance,
}

/// One native Hyperliquid `spotState` stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidSpotStateEvent {
    /// Account address carried by the provider frame.
    pub user: String,
    /// Balances in provider response order.
    pub balances: Vec<HyperliquidSpotStateBalance>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// A full-fidelity Hyperliquid market stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HyperliquidMarketEvent {
    /// A native trade update.
    Trade(HyperliquidTradeEvent),
    /// A native order-book snapshot update.
    OrderBook(HyperliquidOrderBookEvent),
    /// A native asset-context update.
    AssetContext(HyperliquidAssetContextEvent),
    /// A native candle update.
    Candle(HyperliquidCandleEvent),
    /// The connection was restored and events may have been missed.
    Reconnected,
}

/// A full-fidelity Hyperliquid account stream event.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
pub enum HyperliquidAccountEvent {
    /// A native order-status update.
    OrderUpdate(HyperliquidOrderUpdate),
    /// A native Spot account-state update.
    SpotState(HyperliquidSpotStateEvent),
    /// The connection was restored and events may have been missed.
    Reconnected,
}

/// One historical Hyperliquid funding-rate observation with its premium.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidFundingHistoryEntry {
    /// Hyperliquid's native perpetual market name.
    pub coin: String,
    /// Resolved default-universe perpetual market.
    pub market: Market,
    /// Funding rate.
    pub funding_rate: Decimal,
    /// Premium index used by Hyperliquid's funding calculation, when supplied.
    pub premium: Option<Decimal>,
    /// Observation timestamp.
    pub time: Timestamp,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// One account funding payment with the provider's position-size and sample data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidUserFunding {
    /// Provider event kind, normally `funding`.
    pub kind: Option<String>,
    /// Hyperliquid's native perpetual market name.
    pub coin: String,
    /// Resolved default-universe perpetual market.
    pub market: Market,
    /// Signed USDC amount. Negative means the account paid funding.
    pub usdc: Decimal,
    /// Funding rate applied to this payment.
    pub funding_rate: Decimal,
    /// Signed position size at the funding event, when supplied.
    pub position_size: Option<Decimal>,
    /// Number of samples used for the funding calculation, when supplied.
    pub sample_count: Option<u64>,
    /// Transaction hash.
    pub hash: String,
    /// Event timestamp.
    pub time: Timestamp,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// One balance in Hyperliquid's spot clearinghouse state.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidSpotBalance {
    /// Provider coin name.
    pub coin: String,
    /// Hyperliquid token index, when supplied.
    pub token: Option<u32>,
    /// Total balance including held funds.
    pub total: Decimal,
    /// Amount held for orders.
    pub hold: Decimal,
    /// Provider spot entry notional, when supplied.
    pub entry_notional: Option<Decimal>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// The spot clearinghouse state for one configured account.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidSpotClearinghouseState {
    /// Spot balances in provider response order.
    pub balances: Vec<HyperliquidSpotBalance>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// EVM contract metadata associated with a Hyperliquid spot token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperliquidEvmContract {
    /// EVM contract address.
    pub address: String,
    /// Extra decimal adjustment Hyperliquid supplies for the EVM representation.
    pub extra_wei_decimals: u32,
}

/// One token from Hyperliquid's spot metadata response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidSpotToken {
    /// Provider token name.
    pub name: String,
    /// Trading-size decimal precision.
    pub size_decimals: u32,
    /// EVM wei decimal precision, when supplied.
    pub wei_decimals: Option<u32>,
    /// Hyperliquid token index.
    pub index: u32,
    /// Provider token identifier, when supplied.
    pub token_id: Option<String>,
    /// Whether Hyperliquid marks the token canonical.
    pub is_canonical: Option<bool>,
    /// EVM contract details, when Hyperliquid supplies a complete contract object.
    pub evm_contract: Option<HyperliquidEvmContract>,
    /// Full token name, when supplied.
    pub full_name: Option<String>,
    /// Deployer trading fee share, when supplied.
    pub deployer_trading_fee_share: Option<Decimal>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// One pair from Hyperliquid's spot metadata response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidSpotPair {
    /// Provider pair name.
    pub name: String,
    /// Base and quote token indices, in provider order.
    pub tokens: Vec<u32>,
    /// Hyperliquid spot pair index.
    pub index: u32,
    /// Whether Hyperliquid marks the pair canonical.
    pub is_canonical: Option<bool>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// The complete provider-specific `spotMeta` response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidSpotMeta {
    /// Tokens in provider response order.
    pub tokens: Vec<HyperliquidSpotToken>,
    /// Spot pairs in provider response order.
    pub universe: Vec<HyperliquidSpotPair>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// One spot market context from `spotMetaAndAssetCtxs`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidSpotAssetContext {
    /// Provider spot market name.
    pub coin: Option<String>,
    /// Mid price, when supplied.
    pub mid_price: Option<Decimal>,
    /// Mark price, when supplied.
    pub mark_price: Option<Decimal>,
    /// Previous-day price, when supplied.
    pub previous_day_price: Option<Decimal>,
    /// Day base volume, when supplied.
    pub day_base_volume: Option<Decimal>,
    /// Day notional volume, when supplied.
    pub day_notional_volume: Option<Decimal>,
    /// Circulating supply, when supplied.
    pub circulating_supply: Option<Decimal>,
    /// Total supply, when supplied.
    pub total_supply: Option<Decimal>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// The complete provider-specific `spotMetaAndAssetCtxs` response.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidSpotMetaAndAssetContexts {
    /// Provider spot metadata.
    pub meta: HyperliquidSpotMeta,
    /// Spot contexts in provider response order.
    pub contexts: Vec<HyperliquidSpotAssetContext>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// One account-wide entry from Hyperliquid's non-funding ledger.
///
/// These entries describe deposits, withdrawals, transfers, and liquidations;
/// they are not market-scoped [`FundingPayment`](crate::FundingPayment) records.
///
/// Fields not supplied for an entry kind are `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidLedgerEntry {
    /// What kind of movement this was.
    pub kind: HyperliquidLedgerKind,
    /// When Hyperliquid recorded it.
    pub time: Timestamp,
    /// The on-chain transaction hash.
    pub hash: String,
    /// The asset that moved, uppercase. Spot transfers name their token;
    /// other amount-bearing entries use `USDC`.
    pub asset: Option<String>,
    /// How much moved, **unsigned**.
    ///
    /// Direction is represented by [`HyperliquidLedgerEntry::kind`], not by the
    /// sign of this value.
    pub amount: Option<Decimal>,
    /// The fee charged on top, when the kind has one.
    pub fee: Option<Decimal>,
    /// The other address, for the kinds that move funds between two of them.
    pub counterparty: Option<String>,
}

/// What kind of movement a [`HyperliquidLedgerEntry`] records.
///
/// Unrecognized wire values are preserved by
/// [`HyperliquidLedgerKind::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HyperliquidLedgerKind {
    /// USDC arrived from the bridge.
    Deposit,
    /// USDC left over the bridge.
    Withdraw,
    /// USDC moved to another Hyperliquid address.
    InternalTransfer,
    /// USDC moved between this account and one of its subaccounts.
    SubAccountTransfer,
    /// A spot token moved to another Hyperliquid address.
    SpotTransfer,
    /// USDC moved between the spot wallet and the perpetual wallet.
    AccountClassTransfer,
    /// USDC went into a vault.
    VaultDeposit,
    /// USDC came out of a vault.
    VaultWithdraw,
    /// A vault paid out profits.
    VaultDistribution,
    /// A position was closed by the liquidation engine.
    Liquidation,
    /// A kind this release does not name, under Hyperliquid's own spelling.
    Other(String),
}

impl HyperliquidLedgerKind {
    fn from_name(name: &str) -> Self {
        match name {
            "deposit" => Self::Deposit,
            "withdraw" => Self::Withdraw,
            "internalTransfer" => Self::InternalTransfer,
            "subAccountTransfer" => Self::SubAccountTransfer,
            "spotTransfer" => Self::SpotTransfer,
            "accountClassTransfer" => Self::AccountClassTransfer,
            "vaultDeposit" => Self::VaultDeposit,
            "vaultWithdraw" => Self::VaultWithdraw,
            "vaultDistribution" => Self::VaultDistribution,
            "liquidation" | "ledgerLiquidation" => Self::Liquidation,
            other => Self::Other(other.to_string()),
        }
    }
}

/// Hyperliquid's current context and order precision for one market.
///
/// [`HyperliquidAssetContext::funding_rate`] is the provider's current market
/// rate. [`FundingRate`](crate::FundingRate) contains historical market-rate
/// observations, while [`FundingPayment`](crate::FundingPayment) contains
/// amounts actually charged or credited to an account.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidAssetContext {
    /// The provider's mid price, or `None` when unavailable.
    pub mid_price: Option<Decimal>,
    /// The provider's mark price.
    pub mark_price: Option<Decimal>,
    /// The provider's oracle price. Perpetual markets only.
    pub oracle_price: Option<Decimal>,
    /// The current funding rate as a signed ratio.
    ///
    /// Perpetual markets only; `None` on spot, which pays no funding.
    pub funding_rate: Option<Decimal>,
    /// Open interest in the base asset. Perpetual markets only.
    pub open_interest: Option<Decimal>,
    /// Maximum decimal places accepted for order size.
    ///
    /// Finer sizes are rejected locally before signing.
    pub size_decimals: u32,
    /// Maximum decimal places accepted for a fractional order price.
    ///
    /// This is `6 - size_decimals` for perpetuals and `8 - size_decimals` for
    /// spot. Fractional prices are also limited to five significant digits;
    /// integer prices are exempt from the significant-digit limit.
    pub price_decimals: u32,
}

/// Hyperliquid's account-specific Info API request allowance.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidUserRateLimit {
    /// Cumulative trading volume under Hyperliquid's spelling.
    pub cumulative_volume: Decimal,
    /// Requests currently counted against the allowance.
    pub requests_used: u64,
    /// Total current request allowance.
    pub requests_cap: u64,
    /// Reserved request allowance not currently counted as used.
    pub requests_surplus: u64,
}

/// The role Hyperliquid associates with an account.
///
/// New provider role strings are retained as [`Self::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HyperliquidUserRole {
    /// A regular trading user.
    User,
    /// An API agent wallet associated with a user.
    Agent {
        /// Associated user address, when the provider supplies one.
        user: Option<String>,
    },
    /// A vault account.
    Vault,
    /// A subaccount associated with a master account.
    SubAccount {
        /// Associated master account address, when the provider supplies one.
        master: Option<String>,
    },
    /// An address unknown to Hyperliquid.
    Missing,
    /// A provider role this release does not name.
    Other {
        /// Provider role name.
        role: String,
        /// Provider role data encoded as JSON, including explicit `null`.
        data_json: Option<String>,
    },
}

/// Referral data published by Hyperliquid.
///
/// Nested provider-owned state stays as JSON text because its shape depends on
/// the referral stage and token. Stable top-level balances remain exact
/// decimals.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidReferral {
    /// The referring address and code, when the account was referred.
    pub referred_by: Option<HyperliquidReferrer>,
    /// Cumulative volume in Hyperliquid's USDC accounting.
    pub cumulative_volume: Decimal,
    /// Rewards not yet claimed.
    pub unclaimed_rewards: Decimal,
    /// Rewards already claimed.
    pub claimed_rewards: Decimal,
    /// Builder rewards already earned.
    pub builder_rewards: Decimal,
    /// Current referral-program state under Hyperliquid's provider schema.
    pub referrer_state_json: String,
    /// Legacy reward history under Hyperliquid's provider schema as JSON.
    pub reward_history_json: String,
    /// Per-token referral state under Hyperliquid's provider schema as JSON.
    pub token_to_state_json: String,
}

/// The account and code that referred a Hyperliquid user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperliquidReferrer {
    /// Referring account address.
    pub address: String,
    /// Hyperliquid referral code.
    pub code: String,
}

/// Hyperliquid's account-specific fee data.
///
/// The fee schedule is provider-owned and can gain tiers, products, and trial
/// data independently. It is retained as JSON text; the directly applicable
/// rates and daily volume records are exposed as exact decimals.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidUserFees {
    /// Daily volume records in provider response order.
    pub daily_volumes: Vec<HyperliquidDailyVolume>,
    /// Hyperliquid's full fee schedule.
    pub fee_schedule_json: String,
    /// Current perpetual taker rate.
    pub user_cross_rate: Decimal,
    /// Current perpetual maker rate.
    pub user_add_rate: Decimal,
    /// Current spot taker rate, when supplied by Hyperliquid.
    pub user_spot_cross_rate: Option<Decimal>,
    /// Current spot maker rate, when supplied by Hyperliquid.
    pub user_spot_add_rate: Option<Decimal>,
    /// Active referral discount, when supplied by Hyperliquid.
    pub active_referral_discount: Option<Decimal>,
    /// Fields outside the stable rates and volume records.
    pub details_json: String,
}

/// One day of Hyperliquid user volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperliquidDailyVolume {
    /// Calendar date under Hyperliquid's spelling.
    pub date: String,
    /// User taker volume.
    pub user_cross: Decimal,
    /// User maker volume.
    pub user_add: Decimal,
    /// Exchange-wide volume.
    pub exchange: Decimal,
}

/// One named Hyperliquid portfolio period.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperliquidPortfolioPeriod {
    /// Provider period label, such as `day` or `perpAllTime`.
    pub period: String,
    /// Account-value observations in provider response order.
    pub account_value_history: Vec<HyperliquidPortfolioPoint>,
    /// Profit/loss observations in provider response order.
    pub pnl_history: Vec<HyperliquidPortfolioPoint>,
    /// Volume for the period.
    pub volume: Decimal,
}

/// One timestamped portfolio value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperliquidPortfolioPoint {
    /// Provider timestamp.
    pub time: Timestamp,
    /// Account value or profit/loss at that time.
    pub value: Decimal,
}

/// One Hyperliquid subaccount with its current perpetual and spot states.
///
/// The two state payloads are provider-owned nested schemas and are retained
/// as JSON text rather than narrowed into a partial common account model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperliquidSubAccount {
    /// Human-readable subaccount name.
    pub name: String,
    /// Subaccount address.
    pub user: String,
    /// Master account address.
    pub master: String,
    /// Current perpetual clearinghouse state as Hyperliquid JSON.
    pub perpetual_state_json: String,
    /// Current spot state as Hyperliquid JSON.
    pub spot_state_json: String,
}

/// One vault equity record for the configured account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HyperliquidVaultEquity {
    /// Vault address.
    pub vault_address: String,
    /// Current account equity in the vault.
    pub equity: Decimal,
    /// Lock expiry, when Hyperliquid returns one.
    pub locked_until: Option<Timestamp>,
}

/// One execution returned by Hyperliquid's `userFills` Info API.
///
/// This provider-specific record intentionally does not map onto the common
/// [`Trade`](crate::Trade) contract: it includes account position, fee, order,
/// and provider direction data. [`HyperliquidUserFill::raw_json`] retains the
/// complete response object, including fields Hyperliquid adds later.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidUserFill {
    /// Hyperliquid market name, including its native spot spelling when used.
    pub coin: String,
    /// Execution price.
    pub price: Decimal,
    /// Executed base quantity.
    pub size: Decimal,
    /// Provider side string, currently `B` or `A`.
    ///
    /// This remains raw so a provider-added side is not discarded.
    pub side: String,
    /// Execution time.
    pub time: Timestamp,
    /// Position size immediately before this execution.
    pub start_position: Decimal,
    /// Provider direction string, such as `Open Long`.
    ///
    /// This remains raw so a provider-added direction is not discarded.
    pub direction: String,
    /// Realized profit or loss attributed to this execution.
    pub closed_pnl: Decimal,
    /// Transaction hash.
    pub hash: String,
    /// Hyperliquid order identifier.
    pub order_id: u64,
    /// Whether this execution crossed the order book.
    pub crossed: bool,
    /// Fee charged for this execution.
    pub fee: Decimal,
    /// Builder fee, when the provider includes one.
    pub builder_fee: Option<Decimal>,
    /// Hyperliquid trade identifier.
    pub trade_id: u64,
    /// Asset in which the fee was charged.
    pub fee_token: String,
    /// TWAP identifier, when this is a TWAP execution.
    pub twap_id: Option<u64>,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// An order identifier accepted by Hyperliquid's `orderStatus` Info request.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum HyperliquidOrderReference {
    /// Hyperliquid's unsigned server-assigned order identifier.
    OrderId(u64),
    /// A client order identifier encoded as a `0x`-prefixed 16-byte hex string.
    ClientOrderId(String),
}

impl HyperliquidOrderReference {
    /// Refers to a server-assigned order identifier.
    pub const fn order_id(order_id: u64) -> Self {
        Self::OrderId(order_id)
    }

    /// Refers to a client order identifier.
    ///
    /// The value is validated when
    /// [`HyperliquidAdapter::order_status`](crate::adapters::HyperliquidAdapter::order_status)
    /// builds the request.
    pub fn client_order_id(client_order_id: impl Into<String>) -> Self {
        Self::ClientOrderId(client_order_id.into())
    }
}

/// One entry from Hyperliquid's compact `openOrders` Info response.
///
/// This deliberately differs from the common open-order API, which uses
/// Hyperliquid's richer `frontendOpenOrders` response. The complete provider
/// object is retained in [`Self::raw_json`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidOpenOrder {
    /// Hyperliquid's native market name.
    pub coin: String,
    /// Limit price.
    pub limit_price: Decimal,
    /// Server-assigned order identifier.
    pub order_id: u64,
    /// Provider side string, currently `A` or `B`.
    pub side: String,
    /// Remaining size reported by `openOrders`.
    pub size: Decimal,
    /// Order creation time.
    pub timestamp: Timestamp,
    /// Complete provider response object, encoded as JSON.
    pub raw_json: String,
}

/// Hyperliquid's detailed provider order object.
///
/// Provider-owned enum strings stay as strings, and [`Self::raw_json`] keeps
/// fields introduced after this release.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidOrderDetail {
    /// Hyperliquid's native market name.
    pub coin: String,
    /// Provider side string, currently `A` or `B`.
    pub side: String,
    /// Limit price.
    pub limit_price: Decimal,
    /// Size under Hyperliquid's detailed order schema.
    pub size: Decimal,
    /// Server-assigned order identifier.
    pub order_id: u64,
    /// Order creation time.
    pub timestamp: Timestamp,
    /// Provider trigger-condition string.
    pub trigger_condition: String,
    /// Whether this is a trigger order.
    pub is_trigger: bool,
    /// Trigger price.
    pub trigger_price: Decimal,
    /// Child orders under Hyperliquid's provider schema, encoded as JSON.
    pub children_json: String,
    /// Whether this is a position take-profit or stop-loss order.
    pub is_position_tpsl: bool,
    /// Whether the order can only reduce a position.
    pub reduce_only: bool,
    /// Provider order-type string.
    pub order_type: String,
    /// Original order size.
    pub original_size: Decimal,
    /// Provider time-in-force string, when supplied.
    pub time_in_force: Option<String>,
    /// Client order identifier, when supplied.
    pub client_order_id: Option<String>,
    /// Complete provider order object, encoded as JSON.
    pub raw_json: String,
}

/// A detailed Hyperliquid order together with its provider status.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HyperliquidOrderInfo {
    /// Detailed provider order.
    pub order: HyperliquidOrderDetail,
    /// Provider status string, preserved even when this release does not know it.
    pub status: String,
    /// Time at which the provider status last changed.
    pub status_timestamp: Timestamp,
    /// Complete `{order, status, statusTimestamp}` object, encoded as JSON.
    pub raw_json: String,
}

/// Normal response variants from Hyperliquid's `orderStatus` Info request.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HyperliquidOrderStatusResponse {
    /// Hyperliquid found the order and returned its details.
    Order(Box<HyperliquidOrderInfo>),
    /// Hyperliquid returned its documented `unknownOid` response.
    UnknownOrder,
    /// A provider response status introduced after this release.
    Other {
        /// Provider top-level status string.
        status: String,
        /// Complete provider response object, encoded as JSON.
        raw_json: String,
    },
}

/// Reads a page of non-funding ledger entries.
pub(crate) fn ledger_entries(raw: &[RawLedgerUpdate]) -> Result<Vec<HyperliquidLedgerEntry>> {
    raw.iter().map(ledger_entry).collect()
}

fn ledger_entry(raw: &RawLedgerUpdate) -> Result<HyperliquidLedgerEntry> {
    let name = kind_name(raw);

    // Spot transfers use `token`/`amount`; other amount-bearing entries use
    // `USDC`/`usdc`. Liquidations carry neither amount field.
    let (asset, amount_field) = match text(&raw.delta, "token") {
        Some(token) => (token.to_ascii_uppercase(), "amount"),
        None => (parse::SETTLE_ASSET.to_string(), "usdc"),
    };
    let amount = decimal_field(&raw.delta, amount_field)?.map(|value| value.abs());

    Ok(HyperliquidLedgerEntry {
        kind: HyperliquidLedgerKind::from_name(name),
        time: parse::millis(raw.time, "time")?,
        hash: raw.hash.clone(),
        asset: amount.map(|_| asset),
        amount,
        fee: decimal_field(&raw.delta, "fee")?,
        counterparty: text(&raw.delta, "destination").map(str::to_string),
    })
}

/// Maps credited bridge deposits into the common transfer history model.
pub(crate) fn deposits(raw: &[RawLedgerUpdate]) -> Result<Vec<(Deposit, i64)>> {
    raw.iter()
        .filter(|entry| kind_name(entry) == "deposit")
        .map(|entry| {
            let hash = entry.hash.clone();
            Ok((
                Deposit {
                    id: hash.clone(),
                    asset: parse::SETTLE_ASSET.to_string(),
                    // The ledger event does not identify the source bridge.
                    network: None,
                    provider_network: None,
                    amount: required_decimal(&entry.delta, "usdc")?.abs(),
                    // Hyperliquid publishes neither a generated address nor memo.
                    address: None,
                    memo: None,
                    // `deposit` is an event kind, not an explicit lifecycle status.
                    status: DepositStatus::Unknown,
                    provider_status: "deposit".to_string(),
                    tx_id: Some(hash),
                    created_at: Some(parse::millis(entry.time, "time")?),
                },
                entry.time,
            ))
        })
        .collect()
}

/// Maps bridge withdrawals into the common transfer history model.
pub(crate) fn withdrawals(raw: &[RawLedgerUpdate]) -> Result<Vec<(Withdrawal, i64)>> {
    raw.iter()
        .filter(|entry| kind_name(entry) == "withdraw")
        .map(|entry| {
            let hash = entry.hash.clone();
            Ok((
                Withdrawal {
                    id: hash.clone(),
                    asset: parse::SETTLE_ASSET.to_string(),
                    // The ledger event omits both bridge network and destination.
                    network: None,
                    provider_network: None,
                    amount: required_decimal(&entry.delta, "usdc")?.abs(),
                    fee: decimal_field(&entry.delta, "fee")?,
                    destination: None,
                    // `withdraw` does not prove that the bridge finalized.
                    status: WithdrawalStatus::Unknown,
                    provider_status: "withdraw".to_string(),
                    tx_id: Some(hash),
                    created_at: Some(parse::millis(entry.time, "time")?),
                },
                entry.time,
            ))
        })
        .collect()
}

fn kind_name(raw: &RawLedgerUpdate) -> &str {
    raw.delta
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
}

fn text<'a>(delta: &'a Value, field: &str) -> Option<&'a str> {
    delta.get(field).and_then(Value::as_str)
}

fn required_decimal(delta: &Value, field: &'static str) -> Result<Decimal> {
    decimal_field(delta, field)?
        .ok_or_else(|| Error::decode(format!("Hyperliquid ledger delta has no `{field}`")))
}

fn decimal_field(delta: &Value, field: &'static str) -> Result<Option<Decimal>> {
    match delta.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => parse::decimal(value, field).map(Some),
        Some(Value::Number(value)) => parse::decimal(&value.to_string(), field).map(Some),
        Some(other) => Err(Error::decode(format!(
            "Hyperliquid ledger `{field}` is not a number: {other}"
        ))),
    }
}

/// Maps candle snapshots without narrowing away Hyperliquid's trade count.
pub(crate) fn candle_snapshots(
    raw: &[Value],
    universe: &Universe,
) -> Result<Vec<HyperliquidCandleSnapshot>> {
    raw.iter()
        .map(|value| candle_snapshot(&parse::value(value)?, universe, value))
        .collect()
}

fn candle_snapshot(
    raw: &RawCandle,
    universe: &Universe,
    value: &Value,
) -> Result<HyperliquidCandleSnapshot> {
    Ok(HyperliquidCandleSnapshot {
        coin: raw.coin.clone(),
        market: universe.market_from_native_symbol(&raw.coin)?.clone(),
        interval: raw.interval.clone(),
        open_time: parse::millis(raw.open_time, "t")?,
        close_time: parse::millis(raw.close_time, "T")?,
        open: parse::decimal(&raw.open, "o")?,
        high: parse::decimal(&raw.high, "h")?,
        low: parse::decimal(&raw.low, "l")?,
        close: parse::decimal(&raw.close, "c")?,
        volume: parse::decimal(&raw.volume, "v")?,
        trade_count: raw.trade_count,
        raw_json: json_text(value)?,
    })
}

/// Maps an L2 snapshot without dropping each level's provider order count.
pub(crate) fn l2_book(raw: &Value, universe: &Universe) -> Result<HyperliquidL2Book> {
    let book: RawBook = parse::value(raw)?;
    let levels = |levels: &[parse::RawLevel]| {
        levels
            .iter()
            .map(|level| {
                Ok(HyperliquidBookLevel {
                    price: parse::decimal(&level.px, "px")?,
                    size: parse::decimal(&level.sz, "sz")?,
                    order_count: level.n,
                })
            })
            .collect::<Result<Vec<_>>>()
    };

    Ok(HyperliquidL2Book {
        coin: book.coin.clone(),
        market: universe.market_from_native_symbol(&book.coin)?.clone(),
        time: parse::millis(book.time, "time")?,
        bids: levels(&book.levels[0])?,
        asks: levels(&book.levels[1])?,
        raw_json: json_text(raw)?,
    })
}

/// Maps recent trades without dropping their transaction hash or account list.
pub(crate) fn recent_trades(
    raw: &[Value],
    universe: &Universe,
) -> Result<Vec<HyperliquidRecentTrade>> {
    raw.iter()
        .map(|value| recent_trade(&parse::value(value)?, universe, value))
        .collect()
}

pub(crate) fn recent_trade(
    raw: &RawTrade,
    universe: &Universe,
    value: &Value,
) -> Result<HyperliquidRecentTrade> {
    Ok(HyperliquidRecentTrade {
        coin: raw.coin.clone(),
        market: universe.market_from_native_symbol(&raw.coin)?.clone(),
        side: raw.side.clone(),
        price: parse::decimal(&raw.px, "px")?,
        size: parse::decimal(&raw.sz, "sz")?,
        time: parse::millis(raw.time, "time")?,
        trade_id: raw.tid.to_string(),
        hash: raw.hash.clone(),
        users: raw.users.clone(),
        raw_json: json_text(value)?,
    })
}

/// Maps one native trade stream entry and its normalized projection.
pub(crate) fn stream_trade(
    raw: &RawTrade,
    universe: &Universe,
    value: &Value,
) -> Result<HyperliquidTradeEvent> {
    Ok(HyperliquidTradeEvent {
        common: parse::trade(raw, universe)?,
        provider: recent_trade(raw, universe, value)?,
    })
}

/// Maps one native L2 stream frame and its normalized projection.
pub(crate) fn stream_order_book(
    raw: &RawBook,
    universe: &Universe,
    value: &Value,
) -> Result<HyperliquidOrderBookEvent> {
    Ok(HyperliquidOrderBookEvent {
        common: parse::order_book(raw, universe)?,
        provider: l2_book(value, universe)?,
    })
}

/// Maps one native candle stream entry and its normalized projection.
pub(crate) fn stream_candle(
    raw: &RawCandle,
    universe: &Universe,
    value: &Value,
    at: Timestamp,
) -> Result<HyperliquidCandleEvent> {
    Ok(HyperliquidCandleEvent {
        common: parse::candle(raw, universe, at)?,
        provider: candle_snapshot(raw, universe, value)?,
    })
}

/// Maps one native asset-context frame and its normalized ticker projection.
pub(crate) fn stream_asset_context(
    coin: &str,
    raw: &RawAssetCtx,
    universe: &Universe,
    value: &Value,
    at: Timestamp,
) -> Result<HyperliquidAssetContextEvent> {
    let decimal =
        |value: Option<&str>, field| value.map(|value| parse::decimal(value, field)).transpose();
    let market = universe.market_from_native_symbol(coin)?.clone();

    Ok(HyperliquidAssetContextEvent {
        common: parse::ticker(raw, &market, at)?,
        coin: coin.to_owned(),
        mid_price: decimal(raw.mid_px.as_deref(), "midPx")?,
        mark_price: decimal(raw.mark_px.as_deref(), "markPx")?,
        previous_day_price: decimal(raw.prev_day_px.as_deref(), "prevDayPx")?,
        day_base_volume: decimal(raw.day_base_volume.as_deref(), "dayBaseVlm")?,
        day_notional_volume: decimal(raw.day_notional_volume.as_deref(), "dayNtlVlm")?,
        oracle_price: decimal(raw.oracle_px.as_deref(), "oraclePx")?,
        funding_rate: decimal(raw.funding.as_deref(), "funding")?,
        open_interest: decimal(raw.open_interest.as_deref(), "openInterest")?,
        circulating_supply: decimal(raw.circulating_supply.as_deref(), "circulatingSupply")?,
        total_supply: decimal(raw.total_supply.as_deref(), "totalSupply")?,
        raw_json: json_text(value)?,
    })
}

/// Maps one native order update and its normalized order projection.
pub(crate) fn stream_order_update(
    raw: &parse::RawStreamOrder,
    universe: &Universe,
    value: &Value,
) -> Result<HyperliquidOrderUpdate> {
    Ok(HyperliquidOrderUpdate {
        common: parse::stream_order(raw, universe)?,
        coin: raw.order.coin.clone(),
        side: raw.order.side.clone(),
        limit_price: parse::decimal(&raw.order.limit_px, "limitPx")?,
        remaining_size: parse::decimal(&raw.order.sz, "sz")?,
        original_size: parse::decimal(&raw.order.orig_sz, "origSz")?,
        order_id: raw.order.oid,
        accepted_at: parse::millis(raw.order.timestamp, "timestamp")?,
        client_order_id: raw.order.cloid.clone(),
        status: raw.status.clone(),
        status_at: raw
            .status_timestamp
            .map(|time| parse::millis(time, "statusTimestamp"))
            .transpose()?,
        raw_json: json_text(value)?,
    })
}

/// Maps historical funding rates without dropping Hyperliquid's premium index.
pub(crate) fn funding_history(
    raw: &[Value],
    universe: &Universe,
) -> Result<Vec<HyperliquidFundingHistoryEntry>> {
    raw.iter()
        .map(|value| funding_history_entry(&parse::value(value)?, universe, value))
        .collect()
}

fn funding_history_entry(
    raw: &RawFundingHistory,
    universe: &Universe,
    value: &Value,
) -> Result<HyperliquidFundingHistoryEntry> {
    let native = raw
        .coin
        .as_deref()
        .ok_or_else(|| Error::decode("Hyperliquid fundingHistory entry has no `coin`"))?;
    let asset = perpetual_asset(universe, native)?;
    Ok(HyperliquidFundingHistoryEntry {
        coin: native.to_owned(),
        market: asset.market.clone(),
        funding_rate: parse::decimal(&raw.funding_rate, "fundingRate")?,
        premium: raw
            .premium
            .as_deref()
            .map(|value| parse::decimal(value, "premium"))
            .transpose()?,
        time: parse::millis(raw.time, "time")?,
        raw_json: json_text(value)?,
    })
}

/// Maps user funding without dropping signed position size or sample count.
pub(crate) fn user_funding(
    raw: &[Value],
    universe: &Universe,
) -> Result<Vec<HyperliquidUserFunding>> {
    raw.iter()
        .map(|value| user_funding_entry(&parse::value(value)?, universe, value))
        .collect()
}

fn user_funding_entry(
    raw: &RawUserFunding,
    universe: &Universe,
    value: &Value,
) -> Result<HyperliquidUserFunding> {
    let asset = perpetual_asset(universe, &raw.delta.coin)?;
    Ok(HyperliquidUserFunding {
        kind: raw.delta.kind.clone(),
        coin: raw.delta.coin.clone(),
        market: asset.market.clone(),
        usdc: parse::decimal(&raw.delta.usdc, "usdc")?,
        funding_rate: parse::decimal(&raw.delta.funding_rate, "fundingRate")?,
        position_size: raw
            .delta
            .szi
            .as_deref()
            .map(|value| parse::decimal(value, "szi"))
            .transpose()?,
        sample_count: raw.delta.sample_count,
        hash: raw.hash.clone(),
        time: parse::millis(raw.time, "time")?,
        raw_json: json_text(value)?,
    })
}

fn perpetual_asset<'a>(universe: &'a Universe, native: &str) -> Result<&'a Asset> {
    let market = universe.market_from_native_symbol(native)?;
    let asset = universe.asset(market)?;
    if asset.market.kind != MarketKind::Perpetual {
        return Err(Error::decode(format!(
            "Hyperliquid funding entry names spot market `{native}`"
        )));
    }
    Ok(asset)
}

/// Maps the spot state without dropping each balance's token index or entry notional.
pub(crate) fn spot_clearinghouse_state(raw: &Value) -> Result<HyperliquidSpotClearinghouseState> {
    let state: RawSpotState = parse::value(raw)?;
    let values = raw
        .get("balances")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            Error::decode("Hyperliquid spotClearinghouseState has no `balances` array")
        })?;
    let balances = state
        .balances
        .iter()
        .zip(values)
        .map(|(balance, value)| spot_balance(balance, value))
        .collect::<Result<Vec<_>>>()?;

    Ok(HyperliquidSpotClearinghouseState {
        balances,
        raw_json: json_text(raw)?,
    })
}

/// Maps a native `spotState` stream frame without narrowing its balances.
pub(crate) fn stream_spot_state(
    raw: &parse::RawStreamSpotState,
    value: &Value,
) -> Result<HyperliquidSpotStateEvent> {
    let values = value
        .get("spotState")
        .and_then(|state| state.get("balances"))
        .and_then(Value::as_array)
        .ok_or_else(|| Error::decode("Hyperliquid spotState has no `spotState.balances` array"))?;
    let balances = raw
        .spot_state
        .balances
        .iter()
        .zip(values)
        .map(|(balance, value)| {
            Ok(HyperliquidSpotStateBalance {
                common: parse::balance(balance)?,
                provider: spot_balance(balance, value)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(HyperliquidSpotStateEvent {
        user: raw.user.clone(),
        balances,
        raw_json: json_text(value)?,
    })
}

fn spot_balance(raw: &parse::RawSpotBalance, value: &Value) -> Result<HyperliquidSpotBalance> {
    Ok(HyperliquidSpotBalance {
        coin: raw.coin.clone(),
        token: raw.token,
        total: parse::decimal(&raw.total, "total")?,
        hold: parse::decimal(&raw.hold, "hold")?,
        entry_notional: raw
            .entry_notional
            .as_deref()
            .map(|value| parse::decimal(value, "entryNtl"))
            .transpose()?,
        raw_json: json_text(value)?,
    })
}

/// Maps the complete `spotMeta` response without dropping provider token fields.
pub(crate) fn spot_meta(raw: &Value) -> Result<HyperliquidSpotMeta> {
    let meta: RawSpotMeta = parse::value(raw)?;
    spot_meta_from_raw(&meta, raw)
}

fn spot_meta_from_raw(raw: &RawSpotMeta, value: &Value) -> Result<HyperliquidSpotMeta> {
    let tokens_value = value
        .get("tokens")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::decode("Hyperliquid spotMeta has no `tokens` array"))?;
    let tokens = raw
        .tokens
        .iter()
        .zip(tokens_value)
        .map(|(token, value)| {
            Ok(HyperliquidSpotToken {
                name: token.name.clone(),
                size_decimals: token.sz_decimals,
                wei_decimals: token.wei_decimals,
                index: token.index,
                token_id: token.token_id.clone(),
                is_canonical: token.is_canonical,
                evm_contract: token.evm_contract.as_ref().and_then(|contract| {
                    Some(HyperliquidEvmContract {
                        address: contract.address.clone()?,
                        extra_wei_decimals: contract.extra_wei_decimals?,
                    })
                }),
                full_name: token.full_name.clone(),
                deployer_trading_fee_share: token
                    .deployer_trading_fee_share
                    .as_deref()
                    .map(|value| parse::decimal(value, "deployerTradingFeeShare"))
                    .transpose()?,
                raw_json: json_text(value)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let universe_value = value
        .get("universe")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::decode("Hyperliquid spotMeta has no `universe` array"))?;
    let universe = raw
        .universe
        .iter()
        .zip(universe_value)
        .map(|(pair, value)| {
            Ok(HyperliquidSpotPair {
                name: pair.name.clone(),
                tokens: pair.tokens.to_vec(),
                index: pair.index,
                is_canonical: pair.is_canonical,
                raw_json: json_text(value)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(HyperliquidSpotMeta {
        tokens,
        universe,
        raw_json: json_text(value)?,
    })
}

/// Maps `spotMetaAndAssetCtxs` without dropping supply-related spot context fields.
pub(crate) fn spot_meta_and_asset_contexts(
    raw: &Value,
) -> Result<HyperliquidSpotMetaAndAssetContexts> {
    let values = raw.as_array().ok_or_else(|| {
        Error::decode("Hyperliquid spotMetaAndAssetCtxs response is not an array")
    })?;
    let [meta, contexts] = values.as_slice() else {
        return Err(Error::decode(
            "Hyperliquid spotMetaAndAssetCtxs response is not a [meta, contexts] pair",
        ));
    };
    let meta_raw: RawSpotMeta = parse::value(meta)?;
    let contexts = contexts
        .as_array()
        .ok_or_else(|| Error::decode("Hyperliquid spot asset contexts are not an array"))?
        .iter()
        .map(|value| spot_asset_context(&parse::value(value)?, value))
        .collect::<Result<Vec<_>>>()?;

    Ok(HyperliquidSpotMetaAndAssetContexts {
        meta: spot_meta_from_raw(&meta_raw, meta)?,
        contexts,
        raw_json: json_text(raw)?,
    })
}

fn spot_asset_context(raw: &RawAssetCtx, value: &Value) -> Result<HyperliquidSpotAssetContext> {
    let decimal =
        |value: Option<&str>, field| value.map(|value| parse::decimal(value, field)).transpose();
    Ok(HyperliquidSpotAssetContext {
        coin: raw.coin.clone(),
        mid_price: decimal(raw.mid_px.as_deref(), "midPx")?,
        mark_price: decimal(raw.mark_px.as_deref(), "markPx")?,
        previous_day_price: decimal(raw.prev_day_px.as_deref(), "prevDayPx")?,
        day_base_volume: decimal(raw.day_base_volume.as_deref(), "dayBaseVlm")?,
        day_notional_volume: decimal(raw.day_notional_volume.as_deref(), "dayNtlVlm")?,
        circulating_supply: decimal(raw.circulating_supply.as_deref(), "circulatingSupply")?,
        total_supply: decimal(raw.total_supply.as_deref(), "totalSupply")?,
        raw_json: json_text(value)?,
    })
}

/// Reads the current context of one market.
pub(crate) fn asset_context(
    raw: &RawAssetCtx,
    asset: &parse::Asset,
) -> Result<HyperliquidAssetContext> {
    let optional = |value: Option<&str>, field: &'static str| {
        value.map(|value| parse::decimal(value, field)).transpose()
    };

    Ok(HyperliquidAssetContext {
        size_decimals: asset.size_decimals,
        price_decimals: asset.price_decimals(),
        mid_price: optional(raw.mid_px.as_deref(), "midPx")?,
        mark_price: optional(raw.mark_px.as_deref(), "markPx")?,
        oracle_price: optional(raw.oracle_px.as_deref(), "oraclePx")?,
        funding_rate: optional(raw.funding.as_deref(), "funding")?,
        open_interest: optional(raw.open_interest.as_deref(), "openInterest")?,
    })
}

pub(crate) fn user_rate_limit(raw: &Value) -> Result<HyperliquidUserRateLimit> {
    let raw = object(raw, "userRateLimit")?;
    Ok(HyperliquidUserRateLimit {
        cumulative_volume: decimal_value(raw, "cumVlm")?,
        requests_used: unsigned_value(raw, "nRequestsUsed")?,
        requests_cap: unsigned_value(raw, "nRequestsCap")?,
        requests_surplus: unsigned_value(raw, "nRequestsSurplus")?,
    })
}

pub(crate) fn user_role(raw: &Value) -> Result<HyperliquidUserRole> {
    let object = object(raw, "userRole")?;
    let role = text_value(object, "role")?;
    let data = object.get("data").cloned();

    Ok(match role {
        "user" => HyperliquidUserRole::User,
        "agent" => HyperliquidUserRole::Agent {
            user: role_address(data.as_ref(), "user")?,
        },
        "vault" => HyperliquidUserRole::Vault,
        "subAccount" => HyperliquidUserRole::SubAccount {
            master: role_address(data.as_ref(), "master")?,
        },
        "missing" => HyperliquidUserRole::Missing,
        other => HyperliquidUserRole::Other {
            role: other.to_string(),
            data_json: data.as_ref().map(json_text).transpose()?,
        },
    })
}

pub(crate) fn referral(raw: &Value) -> Result<HyperliquidReferral> {
    let values = object(raw, "referral")?;
    let referred_by = match values.get("referredBy") {
        None | Some(Value::Null) => None,
        Some(value) => {
            let referrer = object(value, "referral.referredBy")?;
            Some(HyperliquidReferrer {
                address: text_value(referrer, "referrer")?.to_string(),
                code: text_value(referrer, "code")?.to_string(),
            })
        }
    };

    Ok(HyperliquidReferral {
        referred_by,
        cumulative_volume: decimal_value(values, "cumVlm")?,
        unclaimed_rewards: decimal_value(values, "unclaimedRewards")?,
        claimed_rewards: decimal_value(values, "claimedRewards")?,
        builder_rewards: decimal_value(values, "builderRewards")?,
        referrer_state_json: json_text(required_value(values, "referrerState")?)?,
        reward_history_json: json_text(required_value(values, "rewardHistory")?)?,
        token_to_state_json: json_text(required_value(values, "tokenToState")?)?,
    })
}

pub(crate) fn user_fees(raw: &Value) -> Result<HyperliquidUserFees> {
    let values = object(raw, "userFees")?;
    let daily_volumes = value_array(values, "dailyUserVlm")?
        .iter()
        .map(|value| {
            let volume = object(value, "userFees.dailyUserVlm")?;
            Ok(HyperliquidDailyVolume {
                date: text_value(volume, "date")?.to_string(),
                user_cross: decimal_value(volume, "userCross")?,
                user_add: decimal_value(volume, "userAdd")?,
                exchange: decimal_value(volume, "exchange")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut details = values.clone();
    for field in [
        "dailyUserVlm",
        "feeSchedule",
        "userCrossRate",
        "userAddRate",
        "userSpotCrossRate",
        "userSpotAddRate",
        "activeReferralDiscount",
    ] {
        details.remove(field);
    }

    Ok(HyperliquidUserFees {
        daily_volumes,
        fee_schedule_json: json_text(required_value(values, "feeSchedule")?)?,
        user_cross_rate: decimal_value(values, "userCrossRate")?,
        user_add_rate: decimal_value(values, "userAddRate")?,
        user_spot_cross_rate: optional_decimal_value(values, "userSpotCrossRate")?,
        user_spot_add_rate: optional_decimal_value(values, "userSpotAddRate")?,
        active_referral_discount: optional_decimal_value(values, "activeReferralDiscount")?,
        details_json: json_text(&Value::Object(details))?,
    })
}

pub(crate) fn portfolio(raw: &Value) -> Result<Vec<HyperliquidPortfolioPeriod>> {
    let periods = raw
        .as_array()
        .ok_or_else(|| Error::decode("Hyperliquid portfolio response is not an array"))?;

    periods
        .iter()
        .map(|entry| {
            let pair = entry.as_array().ok_or_else(|| {
                Error::decode("Hyperliquid portfolio period is not a [name, data] pair")
            })?;
            let [name, data] = pair.as_slice() else {
                return Err(Error::decode(
                    "Hyperliquid portfolio period is not a [name, data] pair",
                ));
            };
            let name = name
                .as_str()
                .ok_or_else(|| Error::decode("Hyperliquid portfolio period name is not text"))?;
            let data = object(data, "portfolio period")?;

            Ok(HyperliquidPortfolioPeriod {
                period: name.to_string(),
                account_value_history: portfolio_points(
                    required_value(data, "accountValueHistory")?,
                    "accountValueHistory",
                )?,
                pnl_history: portfolio_points(required_value(data, "pnlHistory")?, "pnlHistory")?,
                volume: decimal_value(data, "vlm")?,
            })
        })
        .collect()
}

pub(crate) fn sub_accounts(raw: &Value) -> Result<Vec<HyperliquidSubAccount>> {
    match raw {
        Value::Null => Ok(Vec::new()),
        Value::Array(values) => values
            .iter()
            .map(|value| {
                let value = object(value, "subAccounts entry")?;
                Ok(HyperliquidSubAccount {
                    name: text_value(value, "name")?.to_string(),
                    user: text_value(value, "subAccountUser")?.to_string(),
                    master: text_value(value, "master")?.to_string(),
                    perpetual_state_json: json_text(required_value(value, "clearinghouseState")?)?,
                    spot_state_json: json_text(required_value(value, "spotState")?)?,
                })
            })
            .collect(),
        _ => Err(Error::decode(
            "Hyperliquid subAccounts response is not an array or null",
        )),
    }
}

pub(crate) fn vault_equities(raw: &Value) -> Result<Vec<HyperliquidVaultEquity>> {
    let values = raw
        .as_array()
        .ok_or_else(|| Error::decode("Hyperliquid userVaultEquities response is not an array"))?;

    values
        .iter()
        .map(|value| {
            let object = object(value, "userVaultEquities entry")?;
            Ok(HyperliquidVaultEquity {
                vault_address: text_value(object, "vaultAddress")?.to_string(),
                equity: decimal_value(object, "equity")?,
                locked_until: optional_timestamp_value(object, "lockedUntilTimestamp")?,
            })
        })
        .collect()
}

pub(crate) fn user_fills(raw: &Value) -> Result<Vec<HyperliquidUserFill>> {
    raw.as_array()
        .ok_or_else(|| Error::decode("Hyperliquid userFills response is not an array"))?
        .iter()
        .map(|value| {
            let fill = object(value, "userFills entry")?;
            Ok(HyperliquidUserFill {
                coin: text_value(fill, "coin")?.to_string(),
                price: decimal_value(fill, "px")?,
                size: decimal_value(fill, "sz")?,
                side: text_value(fill, "side")?.to_string(),
                time: timestamp_value(fill, "time")?,
                start_position: decimal_value(fill, "startPosition")?,
                direction: text_value(fill, "dir")?.to_string(),
                closed_pnl: decimal_value(fill, "closedPnl")?,
                hash: text_value(fill, "hash")?.to_string(),
                order_id: unsigned_json(required_value(fill, "oid")?, "oid")?,
                crossed: bool_value(fill, "crossed")?,
                fee: decimal_value(fill, "fee")?,
                builder_fee: optional_decimal_value(fill, "builderFee")?,
                trade_id: unsigned_json(required_value(fill, "tid")?, "tid")?,
                fee_token: text_value(fill, "feeToken")?.to_string(),
                twap_id: optional_unsigned_value(fill, "twapId")?,
                raw_json: json_text(value)?,
            })
        })
        .collect()
}

pub(crate) fn basic_open_orders(raw: &Value) -> Result<Vec<HyperliquidOpenOrder>> {
    raw.as_array()
        .ok_or_else(|| Error::decode("Hyperliquid openOrders response is not an array"))?
        .iter()
        .map(|value| {
            let order = object(value, "openOrders entry")?;
            Ok(HyperliquidOpenOrder {
                coin: text_value(order, "coin")?.to_string(),
                limit_price: decimal_value(order, "limitPx")?,
                order_id: unsigned_value(order, "oid")?,
                side: text_value(order, "side")?.to_string(),
                size: decimal_value(order, "sz")?,
                timestamp: timestamp_value(order, "timestamp")?,
                raw_json: json_text(value)?,
            })
        })
        .collect()
}

pub(crate) fn order_status(raw: &Value) -> Result<HyperliquidOrderStatusResponse> {
    let response = object(raw, "orderStatus response")?;
    let status = text_value(response, "status")?;

    match status {
        "unknownOid" => Ok(HyperliquidOrderStatusResponse::UnknownOrder),
        "order" => Ok(HyperliquidOrderStatusResponse::Order(Box::new(order_info(
            required_value(response, "order")?,
        )?))),
        other => Ok(HyperliquidOrderStatusResponse::Other {
            status: other.to_string(),
            raw_json: json_text(raw)?,
        }),
    }
}

pub(crate) fn historical_orders(raw: &Value) -> Result<Vec<HyperliquidOrderInfo>> {
    raw.as_array()
        .ok_or_else(|| Error::decode("Hyperliquid historicalOrders response is not an array"))?
        .iter()
        .map(order_info)
        .collect()
}

fn order_info(value: &Value) -> Result<HyperliquidOrderInfo> {
    let info = object(value, "order info")?;
    Ok(HyperliquidOrderInfo {
        order: order_detail(required_value(info, "order")?)?,
        status: text_value(info, "status")?.to_string(),
        status_timestamp: timestamp_value(info, "statusTimestamp")?,
        raw_json: json_text(value)?,
    })
}

fn order_detail(value: &Value) -> Result<HyperliquidOrderDetail> {
    let order = object(value, "order detail")?;
    Ok(HyperliquidOrderDetail {
        coin: text_value(order, "coin")?.to_string(),
        side: text_value(order, "side")?.to_string(),
        limit_price: decimal_value(order, "limitPx")?,
        size: decimal_value(order, "sz")?,
        order_id: unsigned_value(order, "oid")?,
        timestamp: timestamp_value(order, "timestamp")?,
        trigger_condition: text_value(order, "triggerCondition")?.to_string(),
        is_trigger: bool_value(order, "isTrigger")?,
        trigger_price: decimal_value(order, "triggerPx")?,
        children_json: json_text(required_value(order, "children")?)?,
        is_position_tpsl: bool_value(order, "isPositionTpsl")?,
        reduce_only: bool_value(order, "reduceOnly")?,
        order_type: text_value(order, "orderType")?.to_string(),
        original_size: decimal_value(order, "origSz")?,
        time_in_force: optional_text_value(order, "tif")?,
        client_order_id: optional_text_value(order, "cloid")?,
        raw_json: json_text(value)?,
    })
}

fn portfolio_points(raw: &Value, field: &str) -> Result<Vec<HyperliquidPortfolioPoint>> {
    let points = raw
        .as_array()
        .ok_or_else(|| Error::decode(format!("Hyperliquid portfolio `{field}` is not an array")))?;

    points
        .iter()
        .map(|point| {
            let pair = point.as_array().ok_or_else(|| {
                Error::decode(format!(
                    "Hyperliquid portfolio `{field}` point is not a pair"
                ))
            })?;
            let [time, value] = pair.as_slice() else {
                return Err(Error::decode(format!(
                    "Hyperliquid portfolio `{field}` point is not a pair"
                )));
            };
            let time = time.as_i64().ok_or_else(|| {
                Error::decode(format!(
                    "Hyperliquid portfolio `{field}` time is not an integer"
                ))
            })?;
            Ok(HyperliquidPortfolioPoint {
                time: parse::millis(time, field)?,
                value: decimal_json(value, field)?,
            })
        })
        .collect()
}

fn object<'a>(value: &'a Value, context: &str) -> Result<&'a serde_json::Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| Error::decode(format!("Hyperliquid {context} is not an object")))
}

fn required_value<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a Value> {
    object
        .get(field)
        .ok_or_else(|| Error::decode(format!("Hyperliquid response has no `{field}`")))
}

fn text_value<'a>(object: &'a serde_json::Map<String, Value>, field: &str) -> Result<&'a str> {
    required_value(object, field)?
        .as_str()
        .ok_or_else(|| Error::decode(format!("Hyperliquid `{field}` is not text")))
}

fn optional_text_value(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<Option<String>> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(Error::decode(format!("Hyperliquid `{field}` is not text"))),
    }
}

fn value_array<'a>(object: &'a serde_json::Map<String, Value>, field: &str) -> Result<&'a [Value]> {
    required_value(object, field)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| Error::decode(format!("Hyperliquid `{field}` is not an array")))
}

fn decimal_value(object: &serde_json::Map<String, Value>, field: &str) -> Result<Decimal> {
    decimal_json(required_value(object, field)?, field)
}

fn optional_decimal_value(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<Option<Decimal>> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => decimal_json(value, field).map(Some),
    }
}

fn decimal_json(value: &Value, field: &str) -> Result<Decimal> {
    match value {
        Value::String(value) => parse::decimal(value, field),
        Value::Number(value) => parse::decimal(&value.to_string(), field),
        _ => Err(Error::decode(format!(
            "Hyperliquid `{field}` is not a number"
        ))),
    }
}

fn json_text(value: &Value) -> Result<String> {
    serde_json::to_string(value)
        .map_err(|error| Error::decode(format!("Hyperliquid JSON cannot be encoded: {error}")))
}

fn unsigned_value(object: &serde_json::Map<String, Value>, field: &str) -> Result<u64> {
    unsigned_json(required_value(object, field)?, field)
}

fn optional_unsigned_value(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<Option<u64>> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => unsigned_json(value, field).map(Some),
    }
}

fn unsigned_json(value: &Value, field: &str) -> Result<u64> {
    match value {
        Value::Number(value) => value.as_u64(),
        Value::String(value) => value.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| Error::decode(format!("Hyperliquid `{field}` is not an unsigned integer")))
}

fn bool_value(object: &serde_json::Map<String, Value>, field: &str) -> Result<bool> {
    required_value(object, field)?
        .as_bool()
        .ok_or_else(|| Error::decode(format!("Hyperliquid `{field}` is not a boolean")))
}

fn timestamp_value(object: &serde_json::Map<String, Value>, field: &str) -> Result<Timestamp> {
    match required_value(object, field)? {
        Value::Number(value) => value.as_i64(),
        Value::String(value) => value.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| Error::decode(format!("Hyperliquid `{field}` is not an integer")))
    .and_then(|value| parse::millis(value, field))
}

fn optional_timestamp_value(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<Option<Timestamp>> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_i64()
            .ok_or_else(|| Error::decode(format!("Hyperliquid `{field}` is not an integer")))
            .and_then(|value| parse::millis(value, field))
            .map(Some),
    }
}

fn role_address(data: Option<&Value>, field: &str) -> Result<Option<String>> {
    match data {
        None | Some(Value::Null) => Ok(None),
        Some(data) => match object(data, "userRole.data")?.get(field) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(value)) => Ok(Some(value.clone())),
            Some(_) => Err(Error::decode(format!(
                "Hyperliquid userRole data `{field}` is not text"
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, Market, MarketStatus};

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint#retrieve-a-users-funding-history-or-non-funding-ledger-updates
    const LEDGER: &str = r#"[
      {
        "delta": {"type": "deposit", "usdc": "1000.0"},
        "hash": "0x0000000000000000000000000000000000000000000000000000000000000001",
        "time": 1681222254710
      },
      {
        "delta": {
          "type": "withdraw",
          "usdc": "250.0",
          "nonce": 1681222254711,
          "fee": "1.0"
        },
        "hash": "0x0000000000000000000000000000000000000000000000000000000000000002",
        "time": 1681222354710
      },
      {
        "delta": {
          "type": "spotTransfer",
          "token": "PURR",
          "amount": "12.5",
          "usdcValue": "3.75",
          "user": "0x14791697260e4c9a71f18484c9f997b308e59325",
          "destination": "0x0000000000000000000000000000000000000009",
          "fee": "0.0"
        },
        "hash": "0x0000000000000000000000000000000000000000000000000000000000000003",
        "time": 1681222454710
      },
      {
        "delta": {
          "type": "liquidation",
          "accountValue": "12.0",
          "leverage": 20.0,
          "liquidatedPositions": [{"coin": "ETH", "szi": "-0.5"}]
        },
        "hash": "0x0000000000000000000000000000000000000000000000000000000000000004",
        "time": 1681222554710
      },
      {
        "delta": {"type": "somethingHyperliquidAddedLater", "usdc": "1.0"},
        "hash": "0x0000000000000000000000000000000000000000000000000000000000000005",
        "time": 1681222654710
      }
    ]"#;

    fn entries() -> Vec<HyperliquidLedgerEntry> {
        let raw: Vec<RawLedgerUpdate> =
            parse::json(LEDGER).expect("official ledger updates payload");

        ledger_entries(&raw).expect("a page of entries")
    }

    #[test]
    fn each_kind_of_cash_movement_keeps_its_own_meaning() {
        let entries = entries();

        assert_eq!(entries[0].kind, HyperliquidLedgerKind::Deposit);
        assert_eq!(entries[0].amount, Some(Decimal::from(1_000)));
        assert_eq!(entries[0].asset.as_deref(), Some("USDC"));
        assert_eq!(entries[0].fee, None);
        assert_eq!(entries[0].time, Timestamp::from_millis(1_681_222_254_710));

        // Direction is encoded by the kind; amount remains a magnitude.
        assert_eq!(entries[1].kind, HyperliquidLedgerKind::Withdraw);
        assert_eq!(entries[1].amount, Some(Decimal::from(250)));
        assert_eq!(entries[1].fee, Some(Decimal::ONE));
    }

    #[test]
    fn bridge_events_map_without_inventing_network_destination_or_status() {
        let raw: Vec<RawLedgerUpdate> = parse::json(LEDGER).expect("official ledger payload");
        let deposits = deposits(&raw).expect("deposit history");
        let withdrawals = withdrawals(&raw).expect("withdrawal history");

        let deposit = &deposits[0].0;
        assert_eq!(deposit.asset, "USDC");
        assert_eq!(deposit.amount, Decimal::from(1_000));
        assert_eq!(deposit.network, None);
        assert_eq!(deposit.provider_network, None);
        assert_eq!(deposit.address, None);
        assert_eq!(deposit.status, DepositStatus::Unknown);
        assert_eq!(deposit.provider_status, "deposit");
        assert_eq!(deposit.tx_id.as_deref(), Some(deposit.id.as_str()));

        let withdrawal = &withdrawals[0].0;
        assert_eq!(withdrawal.asset, "USDC");
        assert_eq!(withdrawal.amount, Decimal::from(250));
        assert_eq!(withdrawal.fee, Some(Decimal::ONE));
        assert_eq!(withdrawal.network, None);
        assert_eq!(withdrawal.provider_network, None);
        assert_eq!(withdrawal.destination, None);
        assert_eq!(withdrawal.status, WithdrawalStatus::Unknown);
        assert_eq!(withdrawal.provider_status, "withdraw");
        assert_eq!(withdrawal.tx_id.as_deref(), Some(withdrawal.id.as_str()));
    }

    #[test]
    fn numeric_ledger_amounts_are_read_without_float_round_trips() {
        let raw: Vec<RawLedgerUpdate> = parse::json(
            r#"[{
              "delta": {"type": "deposit", "usdc": 0.125},
              "hash": "0x01",
              "time": 1681222254710
            }]"#,
        )
        .expect("numeric ledger payload");

        assert_eq!(
            deposits(&raw).expect("deposit")[0].0.amount,
            Decimal::new(125, 3)
        );
    }

    #[test]
    fn a_spot_transfer_names_its_token_rather_than_assuming_usdc() {
        let entries = entries();

        assert_eq!(entries[2].kind, HyperliquidLedgerKind::SpotTransfer);
        assert_eq!(entries[2].asset.as_deref(), Some("PURR"));
        assert_eq!(entries[2].amount, Some(Decimal::new(125, 1)));
        assert_eq!(
            entries[2].counterparty.as_deref(),
            Some("0x0000000000000000000000000000000000000009")
        );
    }

    #[test]
    fn a_liquidation_has_no_single_amount_and_does_not_invent_one() {
        let entries = entries();

        assert_eq!(entries[3].kind, HyperliquidLedgerKind::Liquidation);
        assert_eq!(entries[3].amount, None);
        assert_eq!(entries[3].asset, None);
        assert!(entries[3].hash.ends_with("04"));
    }

    #[test]
    fn a_kind_this_release_does_not_know_arrives_named_rather_than_dropped() {
        let entries = entries();

        assert_eq!(
            entries[4].kind,
            HyperliquidLedgerKind::Other("somethingHyperliquidAddedLater".to_string())
        );
        assert_eq!(entries[4].amount, Some(Decimal::ONE));
    }

    #[test]
    fn an_asset_context_carries_the_numbers_the_common_api_has_no_field_for() {
        // `activeAssetCtx` context payload.
        // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions
        let body = r#"{
          "dayNtlVlm": "1169046.29406",
          "funding": "0.0000125",
          "markPx": "14.3161",
          "midPx": "14.314",
          "openInterest": "688.11",
          "oraclePx": "14.325",
          "prevDayPx": "15.322"
        }"#;
        let raw: RawAssetCtx = parse::json(body).expect("official asset context payload");
        // Order precision comes from market metadata, not the context payload.
        let asset = parse::Asset {
            market: Market::perpetual(Exchange::Hyperliquid, "HYPE", "USDC"),
            native: "HYPE".to_string(),
            asset_id: 0,
            size_decimals: 2,
            max_leverage: Some(3),
            only_isolated: false,
            status: MarketStatus::Active,
        };
        let context = asset_context(&raw, &asset).expect("a context");

        // Perpetual price decimals are `6 - size_decimals`.
        assert_eq!(context.size_decimals, 2);
        assert_eq!(context.price_decimals, 4);
        assert_eq!(context.funding_rate, Some(Decimal::new(125, 7)));
        assert_eq!(context.open_interest, Some(Decimal::new(68_811, 2)));
        assert_eq!(context.oracle_price, Some(Decimal::new(14_325, 3)));
        assert_eq!(context.mark_price, Some(Decimal::new(143_161, 4)));
        assert_eq!(context.mid_price, Some(Decimal::new(14_314, 3)));
    }

    #[test]
    fn provider_snapshots_preserve_fields_the_common_contract_omits() {
        let universe = parse::tests::universe();

        let candles: Vec<Value> = parse::json(parse::tests::CANDLE_SNAPSHOT).expect("candles");
        let candle = candle_snapshots(&candles, &universe).expect("provider candles");
        assert_eq!(candle[0].coin, "BTC");
        assert_eq!(candle[0].trade_count, Some(189));
        assert!(candle[0].raw_json.contains("\"n\":189"));

        let book: Value = parse::json(parse::tests::L2_BOOK).expect("book");
        let book = l2_book(&book, &universe).expect("provider book");
        assert_eq!(book.coin, "BTC");
        assert_eq!(book.bids[0].order_count, Some(8));
        assert_eq!(book.asks[1].order_count, Some(3));

        let trades: Vec<Value> = parse::json(
            r#"[{"coin":"BTC","side":"B","px":"1.2","sz":"3.4","time":1681923600000,"tid":7,"hash":"0xhash","users":["0xmaker","0xtaker"]}]"#,
        )
        .expect("trades");
        let trades = recent_trades(&trades, &universe).expect("provider trades");
        assert_eq!(trades[0].coin, "BTC");
        assert_eq!(trades[0].hash.as_deref(), Some("0xhash"));
        assert_eq!(trades[0].users, ["0xmaker", "0xtaker"]);

        let funding: Vec<Value> = parse::json(parse::tests::FUNDING_HISTORY).expect("funding");
        let funding = funding_history(&funding, &universe).expect("provider funding");
        assert_eq!(funding[0].coin, "BTC");
        assert_eq!(funding[0].premium, Some(Decimal::new(-52_196, 8)));

        let user_funding_values: Vec<Value> =
            parse::json(parse::tests::USER_FUNDING).expect("user funding");
        let user_funding =
            user_funding(&user_funding_values, &universe).expect("provider user funding");
        assert_eq!(user_funding[0].coin, "ETH");
        assert_eq!(
            user_funding[0].position_size,
            Some(Decimal::new(491_477, 4))
        );
        assert_eq!(user_funding[0].sample_count, Some(24));

        let state: Value = parse::json(
            r#"{"balances":[{"coin":"PURR","token":1,"total":"2000","hold":"3","entryNtl":"1234.56"}]}"#,
        )
        .expect("spot state");
        let state = spot_clearinghouse_state(&state).expect("provider spot state");
        assert_eq!(state.balances[0].token, Some(1));
        assert_eq!(
            state.balances[0].entry_notional,
            Some(Decimal::new(123_456, 2))
        );
    }

    #[test]
    fn spot_metadata_and_contexts_keep_provider_token_and_supply_fields() {
        let meta: Value = parse::json(
            r#"{
              "tokens":[{
                "name":"PURR","szDecimals":0,"weiDecimals":5,"index":1,
                "tokenId":"0xtoken","isCanonical":true,
                "evmContract":{"address":"0xcontract","evm_extra_wei_decimals":13},
                "fullName":"Purr","deployerTradingFeeShare":"0.01"
              }],
              "universe":[{"name":"PURR/USDC","tokens":[1,0],"index":0,"isCanonical":true}]
            }"#,
        )
        .expect("spot metadata");
        let meta = spot_meta(&meta).expect("provider spot metadata");
        assert_eq!(meta.tokens[0].token_id.as_deref(), Some("0xtoken"));
        assert_eq!(
            meta.tokens[0]
                .evm_contract
                .as_ref()
                .map(|value| value.extra_wei_decimals),
            Some(13)
        );
        assert_eq!(
            meta.tokens[0].deployer_trading_fee_share,
            Some(Decimal::new(1, 2))
        );
        assert_eq!(meta.universe[0].is_canonical, Some(true));

        let contexts: Value = parse::json(
            r#"[
              {
                "tokens":[{"name":"PURR","szDecimals":0,"index":1}],
                "universe":[{"name":"PURR/USDC","tokens":[1,0],"index":0}]
              },
              [{
                "coin":"PURR/USDC","midPx":"0.061726","markPx":"0.061848",
                "prevDayPx":"0.063091","dayBaseVlm":"6100365.0","dayNtlVlm":"380143.3363749998",
                "circulatingSupply":"595169798.8345600367","totalSupply":"595169805.3573399782"
              }]
            ]"#,
        )
        .expect("spot contexts");
        let contexts = spot_meta_and_asset_contexts(&contexts).expect("provider spot contexts");
        assert_eq!(contexts.contexts[0].coin.as_deref(), Some("PURR/USDC"));
        assert_eq!(
            contexts.contexts[0].circulating_supply,
            Some(Decimal::new(5_951_697_988_345_600_367, 10))
        );
        assert_eq!(
            contexts.contexts[0].total_supply,
            Some(Decimal::new(5_951_698_053_573_399_782, 10))
        );
    }

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint.md
    const USER_RATE_LIMIT: &str = r#"{
      "cumVlm": "2854574.593578",
      "nRequestsUsed": 2890,
      "nRequestsCap": 2864574,
      "nRequestsSurplus": 0
    }"#;

    const USER_ROLE: &str = r#"{"role":"subAccount","data":{"master":"0xmaster"}}"#;

    const REFERRAL: &str = r#"{
      "referredBy": {"referrer":"0xreferrer","code":"TESTNET"},
      "cumVlm":"149428030.6628420055",
      "unclaimedRewards":"11.047361",
      "claimedRewards":"22.743781",
      "builderRewards":"0.027802",
      "referrerState":{"stage":"ready","data":{"code":"TEST"}},
      "rewardHistory":[],
      "tokenToState":[[0,{"cumVlm":"149428030.6628420055"}]]
    }"#;

    const USER_FEES: &str = r#"{
      "dailyUserVlm":[{"date":"2025-05-23","userCross":"0.0","userAdd":"0.0","exchange":"2852367.0770729999"}],
      "feeSchedule":{"cross":"0.00045","tiers":{"vip":[]}},
      "userCrossRate":"0.000315",
      "userAddRate":"0.000105",
      "userSpotCrossRate":"0.00049",
      "userSpotAddRate":"0.00028",
      "activeReferralDiscount":"0.0",
      "trial":null,
      "nextTrialAvailableTimestamp":null
    }"#;

    const PORTFOLIO: &str = r#"[["day",{
      "accountValueHistory":[[1741886630493,"1.25"]],
      "pnlHistory":[[1741886630493,"-0.25"]],
      "vlm":"0.0"
    }]]"#;

    const SUB_ACCOUNTS: &str = r#"[
      {
        "name":"Test",
        "subAccountUser":"0xsub",
        "master":"0xmaster",
        "clearinghouseState":{
          "marginSummary":{"accountValue":"29.78001","totalNtlPos":"0.0","totalRawUsd":"29.78001","totalMarginUsed":"0.0"},
          "crossMarginSummary":{"accountValue":"29.78001","totalNtlPos":"0.0","totalRawUsd":"29.78001","totalMarginUsed":"0.0"},
          "crossMaintenanceMarginUsed":"0.0",
          "withdrawable":"29.78001",
          "assetPositions":[{"type":"oneWay","position":{
            "coin":"ETH","szi":"-0.5","cumFunding":{"allTime":"1.0","sinceChange":"0.0","sinceOpen":"0.5"},
            "entryPx":"2986.3","liquidationPx":"2866.2","marginUsed":"4.9","maxLeverage":50,
            "positionValue":"100.02765","returnOnEquity":"-0.0026789","unrealizedPnl":"-0.0026789",
            "leverage":{"type":"isolated","value":20}
          }}],
          "time":1733968369395
        },
        "spotState":{"balances":[{"coin":"USDC","token":0,"total":"0.22","hold":"0.0","entryNtl":null}]}
      }
    ]"#;

    const VAULT_EQUITIES: &str = r#"[
      {"vaultAddress":"0xvault","equity":"742500.082809"},
      {"vaultAddress":"0xlocked","equity":"100.003834","lockedUntilTimestamp":1749857239300}
    ]"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint#retrieve-a-users-fills
    const USER_FILLS: &str = r#"[
      {
        "closedPnl":"0.0",
        "coin":"AVAX",
        "crossed":false,
        "dir":"Open Long",
        "hash":"0xa166e3fa63c25663024b03f2e0da011a00307e4017465df020210d3d432e7cb8",
        "oid":90542681,
        "px":"18.435",
        "side":"B",
        "startPosition":"26.86",
        "sz":"93.53",
        "time":1681222254710,
        "fee":"0.01",
        "builderFee":"0.002",
        "feeToken":"USDC",
        "tid":118906512037719,
        "twapId":null,
        "futureProviderField":{"kept":true}
      },
      {
        "closedPnl":"-0.125",
        "coin":"@107",
        "crossed":true,
        "dir":"Provider Added Direction",
        "hash":"0x0000000000000000000000000000000000000000000000000000000000000001",
        "oid":"90542682",
        "px":18.5,
        "side":"Provider Added Side",
        "startPosition":"0.0",
        "sz":"1.0",
        "time":"1681222254711",
        "fee":"0.0",
        "feeToken":"HYPE",
        "tid":"118906512037720",
        "twapId":"12"
      }
    ]"#;

    // https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint.md
    const BASIC_OPEN_ORDERS: &str = r#"[
      {
        "coin":"BTC",
        "limitPx":"29792.000000000000000001",
        "oid":18446744073709551615,
        "side":"ProviderAddedSide",
        "sz":"0.000000000000000001",
        "timestamp":1681247412573,
        "futureProviderField":{"kept":true}
      }
    ]"#;

    const ORDER_INFO: &str = r#"{
      "order":{
        "coin":"ETH",
        "side":"A",
        "limitPx":"2412.700000000000000001",
        "sz":"0.0",
        "oid":18446744073709551615,
        "timestamp":1724361546645,
        "triggerCondition":"ProviderAddedCondition",
        "isTrigger":false,
        "triggerPx":"0.0",
        "children":[{"futureChild":true}],
        "isPositionTpsl":false,
        "reduceOnly":true,
        "orderType":"ProviderAddedOrderType",
        "origSz":"0.007600000000000001",
        "tif":"ProviderAddedTif",
        "cloid":"0x0123456789abcdef0123456789abcdef",
        "futureOrderField":{"kept":true}
      },
      "status":"ProviderAddedStatus",
      "statusTimestamp":1724361546646,
      "futureInfoField":{"kept":true}
    }"#;

    #[test]
    fn account_info_fixtures_keep_exact_values_and_provider_owned_state() {
        let rate_limit =
            user_rate_limit(&parse::json(USER_RATE_LIMIT).expect("official rate limit"))
                .expect("rate limit");
        assert_eq!(
            rate_limit.cumulative_volume,
            parse::decimal("2854574.593578", "test").unwrap()
        );
        assert_eq!(rate_limit.requests_cap, 2_864_574);

        let referral =
            referral(&parse::json(REFERRAL).expect("official referral")).expect("referral");
        assert_eq!(
            referral
                .referred_by
                .as_ref()
                .map(|value| value.code.as_str()),
            Some("TESTNET")
        );
        assert_eq!(
            referral.token_to_state_json,
            r#"[[0,{"cumVlm":"149428030.6628420055"}]]"#
        );
        assert_eq!(
            referral.referrer_state_json,
            r#"{"stage":"ready","data":{"code":"TEST"}}"#
        );

        let fees = user_fees(&parse::json(USER_FEES).expect("official user fees")).expect("fees");
        assert_eq!(
            fees.daily_volumes[0].exchange,
            parse::decimal("2852367.0770729999", "test").unwrap()
        );
        assert_eq!(
            fees.user_spot_add_rate,
            Some(parse::decimal("0.00028", "test").unwrap())
        );
        let fee_details: Value = serde_json::from_str(&fees.details_json).expect("details JSON");
        assert!(fee_details["trial"].is_null());
        assert!(fee_details["nextTrialAvailableTimestamp"].is_null());

        let portfolio =
            portfolio(&parse::json(PORTFOLIO).expect("official portfolio")).expect("portfolio");
        assert_eq!(portfolio[0].period, "day");
        assert_eq!(
            portfolio[0].pnl_history[0].value,
            parse::decimal("-0.25", "test").unwrap()
        );
        assert_eq!(
            portfolio[0].account_value_history[0].time,
            Timestamp::from_millis(1_741_886_630_493)
        );
    }

    #[test]
    fn account_info_roles_null_subaccounts_and_nullable_vault_locks_are_preserved() {
        let role = user_role(&parse::json(USER_ROLE).expect("official role")).expect("role");
        assert_eq!(
            role,
            HyperliquidUserRole::SubAccount {
                master: Some("0xmaster".to_string())
            }
        );
        let unknown = user_role(&serde_json::json!({"role":"newRole","data":null}))
            .expect("unknown role stays readable");
        assert_eq!(
            unknown,
            HyperliquidUserRole::Other {
                role: "newRole".to_string(),
                data_json: Some("null".to_string()),
            }
        );

        assert!(
            sub_accounts(&Value::Null)
                .expect("null means no subaccounts")
                .is_empty()
        );
        let subaccounts = sub_accounts(&parse::json(SUB_ACCOUNTS).expect("official subaccounts"))
            .expect("subaccounts");
        assert_eq!(subaccounts[0].name, "Test");
        assert_eq!(subaccounts[0].user, "0xsub");
        assert_eq!(subaccounts[0].master, "0xmaster");
        assert_eq!(
            serde_json::from_str::<Value>(&subaccounts[0].perpetual_state_json)
                .expect("perpetual state JSON")["assetPositions"][0]["position"]["szi"],
            "-0.5"
        );
        assert!(
            serde_json::from_str::<Value>(&subaccounts[0].spot_state_json)
                .expect("spot state JSON")["balances"][0]["entryNtl"]
                .is_null()
        );

        let vaults = vault_equities(&parse::json(VAULT_EQUITIES).expect("official vault equities"))
            .expect("vault equities");
        assert_eq!(vaults[0].locked_until, None);
        assert_eq!(
            vaults[1].locked_until,
            Some(Timestamp::from_millis(1_749_857_239_300))
        );
    }

    #[test]
    fn user_fills_keep_exact_values_unknown_strings_and_the_complete_raw_object() {
        let fills =
            user_fills(&parse::json(USER_FILLS).expect("official user fills")).expect("user fills");

        assert_eq!(fills[0].coin, "AVAX");
        assert_eq!(fills[0].price, Decimal::new(18_435, 3));
        assert_eq!(fills[0].size, Decimal::new(9_353, 2));
        assert_eq!(fills[0].order_id, 90_542_681);
        assert_eq!(fills[0].trade_id, 118_906_512_037_719);
        assert_eq!(fills[0].builder_fee, Some(Decimal::new(2, 3)));
        assert_eq!(fills[0].twap_id, None);
        let raw: Value = serde_json::from_str(&fills[0].raw_json).expect("raw fill JSON");
        assert_eq!(raw["futureProviderField"]["kept"], true);

        assert_eq!(fills[1].coin, "@107");
        assert_eq!(fills[1].side, "Provider Added Side");
        assert_eq!(fills[1].direction, "Provider Added Direction");
        assert_eq!(fills[1].order_id, 90_542_682);
        assert_eq!(fills[1].trade_id, 118_906_512_037_720);
        assert_eq!(fills[1].time, Timestamp::from_millis(1_681_222_254_711));
        assert_eq!(fills[1].twap_id, Some(12));
        assert_eq!(fills[1].builder_fee, None);
    }

    #[test]
    fn compact_open_orders_keep_decimal_id_timestamp_and_future_json_boundaries() {
        let orders = basic_open_orders(
            &parse::json(BASIC_OPEN_ORDERS).expect("official openOrders response"),
        )
        .expect("basic open orders");

        assert_eq!(orders[0].order_id, u64::MAX);
        assert_eq!(
            orders[0].limit_price,
            parse::decimal("29792.000000000000000001", "test").unwrap()
        );
        assert_eq!(orders[0].size, Decimal::new(1, 18));
        assert_eq!(orders[0].side, "ProviderAddedSide");
        assert_eq!(
            orders[0].timestamp,
            Timestamp::from_millis(1_681_247_412_573)
        );
        let raw: Value = serde_json::from_str(&orders[0].raw_json).expect("raw order JSON");
        assert_eq!(raw["futureProviderField"]["kept"], true);
    }

    #[test]
    fn order_status_and_history_share_lossless_detailed_order_parsing() {
        let info_value: Value = parse::json(ORDER_INFO).expect("official order info");
        let response = order_status(&serde_json::json!({
            "status":"order",
            "order":info_value.clone()
        }))
        .expect("order status response");
        let HyperliquidOrderStatusResponse::Order(status) = response else {
            panic!("expected an order response")
        };

        assert_eq!(status.order.order_id, u64::MAX);
        assert_eq!(status.status, "ProviderAddedStatus");
        assert_eq!(status.order.order_type, "ProviderAddedOrderType");
        assert_eq!(
            status.order.time_in_force.as_deref(),
            Some("ProviderAddedTif")
        );
        assert_eq!(
            status.order.client_order_id.as_deref(),
            Some("0x0123456789abcdef0123456789abcdef")
        );
        assert_eq!(
            status.status_timestamp,
            Timestamp::from_millis(1_724_361_546_646)
        );
        assert_eq!(
            status.order.original_size,
            Decimal::new(7_600_000_000_000_001, 18)
        );
        let raw_order: Value =
            serde_json::from_str(&status.order.raw_json).expect("raw detailed order JSON");
        assert_eq!(raw_order["futureOrderField"]["kept"], true);

        let history = historical_orders(&Value::Array(vec![info_value])).expect("history");
        assert_eq!(history, vec![status.as_ref().clone()]);
    }

    #[test]
    fn unknown_and_future_order_status_envelopes_are_normal_results() {
        assert_eq!(
            order_status(&serde_json::json!({"status":"unknownOid"})).expect("unknown oid"),
            HyperliquidOrderStatusResponse::UnknownOrder
        );

        let future = order_status(&serde_json::json!({
            "status":"futureResponse",
            "data":{"kept":true}
        }))
        .expect("future response");
        let HyperliquidOrderStatusResponse::Other { status, raw_json } = future else {
            panic!("expected a future response")
        };
        assert_eq!(status, "futureResponse");
        assert_eq!(
            serde_json::from_str::<Value>(&raw_json).expect("raw future response")["data"]["kept"],
            true
        );
    }

    #[test]
    fn provider_millisecond_timestamps_reject_values_outside_maxts_range() {
        let mut order: Value = parse::json(ORDER_INFO).expect("order info");
        order["statusTimestamp"] = serde_json::json!(i64::MAX);

        assert!(matches!(
            historical_orders(&Value::Array(vec![order])),
            Err(Error::Decode { .. })
        ));
    }
}
