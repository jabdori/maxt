//! Hyperliquid-specific data exposed by [`HyperliquidAdapter`].

use rust_decimal::Decimal;
use serde_json::Value;

use crate::error::Result;
use crate::types::Timestamp;

use super::parse::{self, RawAssetCtx, RawLedgerUpdate};

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

/// Reads a page of non-funding ledger entries.
pub(crate) fn ledger_entries(raw: &[RawLedgerUpdate]) -> Result<Vec<HyperliquidLedgerEntry>> {
    raw.iter().map(ledger_entry).collect()
}

fn ledger_entry(raw: &RawLedgerUpdate) -> Result<HyperliquidLedgerEntry> {
    let name = raw
        .delta
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");

    // Spot transfers use `token`/`amount`; other amount-bearing entries use
    // `USDC`/`usdc`. Liquidations carry neither amount field.
    let (asset, amount) = match text(&raw.delta, "token") {
        Some(token) => (token.to_ascii_uppercase(), text(&raw.delta, "amount")),
        None => (parse::SETTLE_ASSET.to_string(), text(&raw.delta, "usdc")),
    };

    Ok(HyperliquidLedgerEntry {
        kind: HyperliquidLedgerKind::from_name(name),
        time: parse::millis(raw.time, "time")?,
        hash: raw.hash.clone(),
        asset: amount.map(|_| asset),
        amount: amount
            .map(|amount| parse::decimal(amount, "amount"))
            .transpose()?
            .map(|amount| amount.abs()),
        fee: text(&raw.delta, "fee")
            .map(|fee| parse::decimal(fee, "fee"))
            .transpose()?,
        counterparty: text(&raw.delta, "destination").map(str::to_string),
    })
}

fn text<'a>(delta: &'a Value, field: &str) -> Option<&'a str> {
    delta.get(field).and_then(Value::as_str)
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
}
