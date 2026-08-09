//! Asset networks, deposit destinations, and transfer records.

use std::fmt;

use rust_decimal::Decimal;

use crate::types::{Exchange, Timestamp};

/// A canonical blockchain network used to compare provider-specific names.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Network {
    /// Bitcoin.
    Bitcoin,
    /// Ethereum mainnet.
    Ethereum,
    /// Arbitrum One.
    Arbitrum,
    /// BNB Smart Chain.
    BnbSmartChain,
    /// Tron.
    Tron,
    /// Solana.
    Solana,
    /// Polygon PoS.
    Polygon,
    /// Base.
    Base,
    /// Optimism.
    Optimism,
    /// Avalanche C-Chain.
    AvalancheC,
    /// XRP Ledger.
    XrpLedger,
    /// Stellar.
    Stellar,
    /// Cosmos Hub.
    Cosmos,
    /// Aptos.
    Aptos,
    /// Sui.
    Sui,
    /// The Open Network.
    Ton,
    /// NEAR.
    Near,
    /// Polkadot.
    Polkadot,
    /// A network added by an exchange after this SDK version.
    Other(String),
}

impl Network {
    /// Stable canonical identifier.
    pub fn id(&self) -> &str {
        match self {
            Self::Bitcoin => "bitcoin",
            Self::Ethereum => "ethereum",
            Self::Arbitrum => "arbitrum",
            Self::BnbSmartChain => "bnb_smart_chain",
            Self::Tron => "tron",
            Self::Solana => "solana",
            Self::Polygon => "polygon",
            Self::Base => "base",
            Self::Optimism => "optimism",
            Self::AvalancheC => "avalanche_c",
            Self::XrpLedger => "xrp_ledger",
            Self::Stellar => "stellar",
            Self::Cosmos => "cosmos",
            Self::Aptos => "aptos",
            Self::Sui => "sui",
            Self::Ton => "ton",
            Self::Near => "near",
            Self::Polkadot => "polkadot",
            Self::Other(id) => id,
        }
    }

    /// Whether two provider mappings identify the same chain.
    pub fn same_chain(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Other(left), Self::Other(right)) => left == right,
            (Self::Other(_), _) | (_, Self::Other(_)) => false,
            _ => self.id() == other.id(),
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

/// One asset's transfer rules on one exchange network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetNetwork {
    /// Exchange that published the rules.
    pub exchange: Exchange,
    /// Asset symbol, uppercase.
    pub asset: String,
    /// Canonical network.
    pub network: Network,
    /// Provider's exact network identifier.
    pub provider_id: String,
    /// Whether the exchange currently accepts deposits.
    pub deposit_enabled: bool,
    /// Whether the exchange currently accepts withdrawals.
    pub withdrawal_enabled: bool,
    /// Withdrawal fee, when published.
    pub withdrawal_fee: Option<WithdrawalFee>,
    /// Minimum withdrawal amount, when published.
    pub minimum_withdrawal: Option<Decimal>,
    /// Maximum withdrawal amount, when published.
    pub maximum_withdrawal: Option<Decimal>,
    /// Whether the destination requires a memo, tag, or secondary address.
    pub memo_required: bool,
}

/// How an exchange calculates a withdrawal fee.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WithdrawalFee {
    /// One fixed amount in the withdrawn asset.
    Fixed(Decimal),
    /// A ratio of the amount, optionally clamped to provider bounds.
    Rate {
        /// Fee ratio. `0.001` means 0.1%.
        rate: Decimal,
        /// Minimum fee.
        minimum: Option<Decimal>,
        /// Maximum fee.
        maximum: Option<Decimal>,
    },
}

impl WithdrawalFee {
    /// Calculates the fee for an amount without rounding it.
    pub fn for_amount(&self, amount: Decimal) -> Decimal {
        match self {
            Self::Fixed(fee) => *fee,
            Self::Rate {
                rate,
                minimum,
                maximum,
            } => {
                let mut fee = amount * *rate;
                if let Some(minimum) = minimum {
                    fee = fee.max(*minimum);
                }
                if let Some(maximum) = maximum {
                    fee = fee.min(*maximum);
                }
                fee
            }
        }
    }
}

/// A deposit address issued by a centralized exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepositAddress {
    /// Destination exchange.
    pub exchange: Exchange,
    /// Asset symbol, uppercase.
    pub asset: String,
    /// Canonical network.
    pub network: Network,
    /// On-chain address, or `None` while asynchronous generation is pending.
    pub address: Option<String>,
    /// Memo, tag, or secondary address when required.
    pub memo: Option<String>,
}

/// An exchange-issued destination ready to receive a withdrawal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeDestination {
    /// Destination exchange.
    pub exchange: Exchange,
    /// Asset symbol, uppercase.
    pub asset: String,
    /// Canonical network.
    pub network: Network,
    /// On-chain address.
    pub address: String,
    /// Memo, tag, or secondary address when required.
    pub memo: Option<String>,
}

/// A direct on-chain destination that was not issued by an exchange API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainDestination {
    /// Asset symbol, uppercase.
    pub asset: String,
    /// Canonical network.
    pub network: Network,
    /// On-chain address.
    pub address: String,
    /// Memo or tag when the chain requires one.
    pub memo: Option<String>,
}

/// Inputs for a checked transfer between two exchange clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeTransferRequest {
    /// Asset symbol, uppercase.
    pub asset: String,
    /// Source network, or `None` to require one unambiguous shared network.
    pub source_network: Option<Network>,
    /// Destination network, or `None` to require one unambiguous shared network.
    pub destination_network: Option<Network>,
    /// Amount submitted to the source exchange before fees.
    pub amount: Decimal,
}

impl ExchangeTransferRequest {
    /// Creates a request that selects one unambiguous shared network.
    pub fn new(asset: impl Into<String>, amount: Decimal) -> Self {
        Self {
            asset: asset.into().to_ascii_uppercase(),
            source_network: None,
            destination_network: None,
            amount,
        }
    }

    /// Pins the source exchange network.
    pub fn source_network(mut self, network: Network) -> Self {
        self.source_network = Some(network);
        self
    }

    /// Pins the destination exchange network.
    pub fn destination_network(mut self, network: Network) -> Self {
        self.destination_network = Some(network);
        self
    }
}

/// Inputs for a checked transfer to an explicit on-chain destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainTransferRequest {
    /// Source asset symbol, uppercase.
    pub asset: String,
    /// Source network, or `None` to use the destination network.
    pub source_network: Option<Network>,
    /// Explicit destination address and chain.
    pub destination: ChainDestination,
    /// Amount submitted to the source exchange before fees.
    pub amount: Decimal,
}

impl ChainTransferRequest {
    /// Creates a request that uses the destination chain on the source.
    pub fn new(asset: impl Into<String>, destination: ChainDestination, amount: Decimal) -> Self {
        Self {
            asset: asset.into().to_ascii_uppercase(),
            source_network: None,
            destination,
            amount,
        }
    }

    /// Pins the source exchange network.
    pub fn source_network(mut self, network: Network) -> Self {
        self.source_network = Some(network);
        self
    }
}

/// Where a withdrawal should arrive.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransferDestination {
    /// Address issued by another exchange.
    Exchange(ExchangeDestination),
    /// Explicit on-chain address.
    Chain(ChainDestination),
}

impl TransferDestination {
    /// Destination asset.
    pub fn asset(&self) -> &str {
        match self {
            Self::Exchange(value) => &value.asset,
            Self::Chain(value) => &value.asset,
        }
    }

    /// Destination network.
    pub fn network(&self) -> &Network {
        match self {
            Self::Exchange(value) => &value.network,
            Self::Chain(value) => &value.network,
        }
    }

    /// Destination address.
    pub fn address(&self) -> &str {
        match self {
            Self::Exchange(value) => &value.address,
            Self::Chain(value) => &value.address,
        }
    }

    /// Destination memo or tag.
    pub fn memo(&self) -> Option<&str> {
        match self {
            Self::Exchange(value) => value.memo.as_deref(),
            Self::Chain(value) => value.memo.as_deref(),
        }
    }
}

/// Current state of a withdrawal request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WithdrawalStatus {
    /// Accepted but not yet broadcast.
    Pending,
    /// Broadcast to the network.
    Processing,
    /// Confirmed as complete by the source exchange.
    Completed,
    /// Cancelled before completion.
    Cancelled,
    /// Rejected or failed.
    Failed,
    /// Provider status not mapped by this SDK version.
    Unknown,
}

/// Current state of a deposit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DepositStatus {
    /// Seen but not sufficiently confirmed.
    Pending,
    /// Credited to the destination account.
    Completed,
    /// Rejected, returned, or otherwise failed.
    Failed,
    /// Provider status not mapped by this SDK version.
    Unknown,
}

/// A withdrawal reported by the source exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Withdrawal {
    /// Exchange withdrawal identifier.
    pub id: String,
    /// Asset symbol, uppercase.
    pub asset: String,
    /// Canonical network, when the history response identifies it.
    pub network: Option<Network>,
    /// Provider's raw network identifier, when published.
    pub provider_network: Option<String>,
    /// Requested amount before fees.
    pub amount: Decimal,
    /// Charged fee, when known.
    pub fee: Option<Decimal>,
    /// Destination, when the provider returns it.
    pub destination: Option<TransferDestination>,
    /// Current state.
    pub status: WithdrawalStatus,
    /// Provider status before normalization.
    pub provider_status: String,
    /// On-chain transaction identifier, when available.
    pub tx_id: Option<String>,
    /// Provider creation time, when available.
    pub created_at: Option<Timestamp>,
}

/// A deposit reported by the destination exchange.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deposit {
    /// Exchange deposit identifier.
    pub id: String,
    /// Asset symbol, uppercase.
    pub asset: String,
    /// Canonical network, when the history response identifies it.
    pub network: Option<Network>,
    /// Provider's raw network identifier, when published.
    pub provider_network: Option<String>,
    /// Credited or pending amount.
    pub amount: Decimal,
    /// Destination address, when published.
    pub address: Option<String>,
    /// Memo or tag, when published.
    pub memo: Option<String>,
    /// Current state.
    pub status: DepositStatus,
    /// Provider status before normalization.
    pub provider_status: String,
    /// On-chain transaction identifier, when available.
    pub tx_id: Option<String>,
    /// Provider creation time, when available.
    pub created_at: Option<Timestamp>,
}

/// Live source-exchange checks performed before a withdrawal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithdrawalQuote {
    /// Fee the source exchange expects to charge.
    pub fee: Option<Decimal>,
    /// Expected destination amount after the known fee.
    pub expected_receive: Option<Decimal>,
    /// Current minimum amount, when published.
    pub minimum_amount: Option<Decimal>,
    /// Current maximum amount, when published.
    pub maximum_amount: Option<Decimal>,
    /// Whether the address is on the account's allowlist, when knowable.
    pub address_allowed: Option<bool>,
    /// Provider-specific Travel Rule requirement.
    pub travel_rule: TravelRuleRequirement,
    /// Provider expiry for this quote, when one exists.
    pub expires_at: Option<Timestamp>,
}

/// Whether a withdrawal needs a provider-specific Travel Rule step.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TravelRuleRequirement {
    /// No additional step is currently required.
    NotRequired,
    /// Extra beneficiary data or explicit consent is required.
    Required {
        /// Provider URL where consent must be completed, when supplied.
        consent_url: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::Network;

    #[test]
    fn unknown_networks_match_only_the_same_provider_identifier() {
        assert!(
            Network::Other("LIGHTNING".to_string())
                .same_chain(&Network::Other("LIGHTNING".to_string()))
        );
        assert!(
            !Network::Other("LIGHTNING".to_string())
                .same_chain(&Network::Other("lightning".to_string()))
        );
        assert!(!Network::Other("bitcoin".to_string()).same_chain(&Network::Bitcoin));
    }
}
