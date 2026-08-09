//! Official API coverage used by code generation and documentation.

use maxt::Exchange;

/// Date of the official documentation snapshot represented by this catalog.
pub const BASELINE_DATE: &str = "2026-08-10";

/// Network interface that carries an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ApiInterface {
    /// HTTP request and response.
    Http,
    /// Request and response messages over WebSocket.
    WebSocketRequest,
    /// Subscription feed over WebSocket.
    WebSocketStream,
    /// FIX session protocol.
    Fix,
    /// JSON-RPC.
    JsonRpc,
    /// On-chain contract call.
    Contract,
}

impl ApiInterface {
    /// Stable documentation identifier.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::WebSocketRequest => "websocket_request",
            Self::WebSocketStream => "websocket_stream",
            Self::Fix => "fix",
            Self::JsonRpc => "json_rpc",
            Self::Contract => "contract",
        }
    }
}

/// Payload encoding, kept separate from the network interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encoding {
    /// JSON text.
    Json,
    /// Simple Binary Encoding.
    Sbe,
    /// FIX tag-value messages.
    FixTagValue,
    /// FIX messages encoded with SBE.
    FixSbe,
    /// Provider-specific binary payload.
    Binary,
}

impl Encoding {
    /// Stable documentation identifier.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Sbe => "sbe",
            Self::FixTagValue => "fix_tag_value",
            Self::FixSbe => "fix_sbe",
            Self::Binary => "binary",
        }
    }
}

/// Authentication required by one operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Authentication {
    /// No secret is required.
    Public,
    /// API key header without a request signature.
    ApiKey,
    /// HMAC-signed request.
    Hmac,
    /// JWT-signed request.
    Jwt,
    /// RSA-signed request.
    Rsa,
    /// Ed25519-signed request.
    Ed25519,
    /// EIP-712 user signature.
    Eip712,
    /// OAuth access token.
    OAuth,
    /// Contracted partner credentials.
    Partner,
}

/// Side-effect risk of one operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationRisk {
    /// Read-only.
    Read,
    /// Reversible account configuration change.
    AccountWrite,
    /// Order, transfer, loan, subscription, or other financial write.
    FinancialWrite,
    /// Credential, KYC, deployer, or partner administration.
    AdministrativeWrite,
}

/// How an official operation is exposed publicly by maxt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMapping {
    /// Common [`maxt::Client`] operation.
    Common(&'static str),
    /// One provider operation supplies several common [`maxt::Client`] operations.
    CommonMany(&'static [&'static str]),
    /// Exchange-specific typed service operation.
    Provider(&'static str),
    /// Typed service operation unavailable on one platform.
    PlatformLimited {
        /// Service method.
        service: &'static str,
        /// Unsupported platform.
        platform: &'static str,
    },
}

/// Current implementation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Implementation {
    /// Catalogued but not implemented.
    Planned,
    /// Implemented in the Rust core.
    Implemented,
}

/// Strongest verification completed for an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Validation {
    /// Official contract recorded only.
    Documented,
    /// Request and response fixture verified.
    Fixture,
    /// Official testnet verified.
    Testnet,
    /// Real read-only request verified.
    LiveRead,
    /// Real write request verified.
    LiveWrite,
}

/// Availability restriction published by the provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    /// Generally available to eligible exchange users.
    General,
    /// Limited to one named region.
    Region(&'static str),
    /// Limited to approved partners or account tiers.
    Partner,
    /// Officially marked beta.
    Beta,
    /// Testnet only.
    Testnet,
}

/// One product in an exchange's official API catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProductCoverage {
    /// Owning exchange.
    pub exchange: Exchange,
    /// Stable exchange-local identifier.
    pub id: &'static str,
    /// Official product name.
    pub name: &'static str,
    /// Endpoint count published by the official catalog, when available.
    pub endpoint_count: Option<u16>,
    /// Official network interfaces.
    pub interfaces: &'static [ApiInterface],
    /// Official payload encodings.
    pub encodings: &'static [Encoding],
}

impl ProductCoverage {
    /// Number of operations currently recorded for this product.
    pub fn mapped_operations(self) -> usize {
        OPERATIONS
            .iter()
            .filter(|operation| operation.exchange == self.exchange && operation.product == self.id)
            .count()
    }

    /// Number of recorded operations implemented in Rust.
    pub fn implemented_operations(self) -> usize {
        OPERATIONS
            .iter()
            .filter(|operation| {
                operation.exchange == self.exchange
                    && operation.product == self.id
                    && operation.implementation == Implementation::Implemented
            })
            .count()
    }

    /// Product status derived from operation records rather than edited by hand.
    pub fn stage(self) -> CoverageStage {
        let mapped = self.mapped_operations();
        if mapped == 0 {
            return CoverageStage::Planned;
        }
        if self.endpoint_count == u16::try_from(mapped).ok()
            && self.implemented_operations() == mapped
        {
            CoverageStage::Complete
        } else {
            CoverageStage::Partial
        }
    }
}

/// Derived product implementation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverageStage {
    /// No operation is implemented yet.
    Planned,
    /// At least one operation is mapped, but the official product is incomplete.
    Partial,
    /// Every active endpoint in the pinned official count is implemented.
    Complete,
}

impl CoverageStage {
    /// Human-readable documentation label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Planned => "Planned",
            Self::Partial => "Partial",
            Self::Complete => "Complete",
        }
    }
}

/// One official endpoint, request message, stream, or protocol operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperationCoverage {
    /// Owning exchange.
    pub exchange: Exchange,
    /// Product identifier from [`PRODUCTS`].
    pub product: &'static str,
    /// Stable operation identifier.
    pub id: &'static str,
    /// HTTP verb or protocol action.
    pub method: &'static str,
    /// Official path, stream, or message name.
    pub path: &'static str,
    /// Network interface.
    pub interface: ApiInterface,
    /// Payload encoding.
    pub encoding: Encoding,
    /// Authentication mode.
    pub authentication: Authentication,
    /// Side-effect risk.
    pub risk: OperationRisk,
    /// Public maxt mapping.
    pub mapping: OperationMapping,
    /// Current implementation state.
    pub implementation: Implementation,
    /// Strongest completed verification.
    pub validation: Validation,
    /// Account, region, or rollout restriction.
    pub availability: Availability,
}

const HTTP: &[ApiInterface] = &[ApiInterface::Http];
const HTTP_STREAM: &[ApiInterface] = &[ApiInterface::Http, ApiInterface::WebSocketStream];
const HTTP_WS_STREAM: &[ApiInterface] = &[
    ApiInterface::Http,
    ApiInterface::WebSocketRequest,
    ApiInterface::WebSocketStream,
];
const SPOT_INTERFACES: &[ApiInterface] = &[
    ApiInterface::Http,
    ApiInterface::WebSocketRequest,
    ApiInterface::WebSocketStream,
    ApiInterface::Fix,
];
const HYPERLIQUID_REQUEST: &[ApiInterface] = &[ApiInterface::Http, ApiInterface::WebSocketRequest];
const HYPERLIQUID_HIP3: &[ApiInterface] = &[
    ApiInterface::Http,
    ApiInterface::WebSocketRequest,
    ApiInterface::WebSocketStream,
    ApiInterface::Contract,
];
const JSON: &[Encoding] = &[Encoding::Json];
const JSON_BINARY: &[Encoding] = &[Encoding::Json, Encoding::Binary];
const BINANCE_SPOT_ENCODINGS: &[Encoding] = &[
    Encoding::Json,
    Encoding::Sbe,
    Encoding::FixTagValue,
    Encoding::FixSbe,
];

const fn product(
    exchange: Exchange,
    id: &'static str,
    name: &'static str,
    endpoint_count: Option<u16>,
    interfaces: &'static [ApiInterface],
    encodings: &'static [Encoding],
) -> ProductCoverage {
    ProductCoverage {
        exchange,
        id,
        name,
        endpoint_count,
        interfaces,
        encodings,
    }
}

/// Products listed by official documentation at [`BASELINE_DATE`].
pub const PRODUCTS: &[ProductCoverage] = &[
    product(
        Exchange::Upbit,
        "quotation",
        "Quotation",
        None,
        HTTP_STREAM,
        JSON,
    ),
    product(
        Exchange::Upbit,
        "exchange",
        "Exchange",
        None,
        HTTP_STREAM,
        JSON,
    ),
    product(
        Exchange::Upbit,
        "wallet",
        "Deposits and withdrawals",
        None,
        HTTP,
        JSON,
    ),
    product(
        Exchange::Upbit,
        "travel_rule",
        "Travel Rule",
        None,
        HTTP,
        JSON,
    ),
    product(
        Exchange::Upbit,
        "pockets",
        "Korea pockets",
        None,
        HTTP,
        JSON,
    ),
    product(
        Exchange::Bithumb,
        "quotation",
        "Quotation",
        None,
        HTTP_STREAM,
        JSON,
    ),
    product(
        Exchange::Bithumb,
        "exchange",
        "Exchange",
        None,
        HTTP_STREAM,
        JSON,
    ),
    product(
        Exchange::Bithumb,
        "wallet",
        "Deposits and withdrawals",
        None,
        HTTP,
        JSON,
    ),
    product(Exchange::Bithumb, "twap", "TWAP", None, HTTP, JSON),
    product(
        Exchange::Bithumb,
        "krw",
        "KRW deposits and withdrawals",
        None,
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "spot",
        "Spot Trading",
        Some(118),
        SPOT_INTERFACES,
        BINANCE_SPOT_ENCODINGS,
    ),
    product(
        Exchange::Binance,
        "usd_m",
        "Futures (USDⓈ-M)",
        Some(133),
        HTTP_WS_STREAM,
        JSON,
    ),
    product(
        Exchange::Binance,
        "coin_m",
        "Futures (COIN-M)",
        Some(93),
        HTTP_WS_STREAM,
        JSON,
    ),
    product(
        Exchange::Binance,
        "options",
        "Options",
        Some(54),
        HTTP_STREAM,
        JSON,
    ),
    product(Exchange::Binance, "margin", "Margin", Some(65), HTTP, JSON),
    product(Exchange::Binance, "wallet", "Wallet", Some(50), HTTP, JSON),
    product(Exchange::Binance, "convert", "Convert", Some(9), HTTP, JSON),
    product(
        Exchange::Binance,
        "portfolio_margin",
        "Portfolio Margin",
        Some(109),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "portfolio_margin_pro",
        "Portfolio Margin Pro",
        Some(24),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "algo",
        "Algo Trading",
        Some(11),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "copy_trading",
        "Copy Trading",
        Some(2),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "institutional_loan",
        "Institutional Loan",
        Some(16),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "alpha",
        "Alpha Trading",
        Some(19),
        HTTP_STREAM,
        JSON,
    ),
    product(
        Exchange::Binance,
        "stocks",
        "Stocks Trading",
        Some(23),
        HTTP_STREAM,
        JSON,
    ),
    product(
        Exchange::Binance,
        "sub_account",
        "Sub Account",
        Some(49),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "spot_block_matching",
        "Spot Block Matching",
        Some(7),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "vip_service",
        "VIP Service",
        Some(11),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "caas",
        "Crypto-as-a-Service (CAAS)",
        Some(10),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "fund_account",
        "Fund Account",
        Some(4),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "link_plus",
        "Link Plus",
        Some(8),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "exchange_link",
        "Exchange Link",
        Some(35),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "kyc_saas",
        "KYC SaaS",
        Some(12),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "link_and_trade",
        "Link and Trade",
        Some(23),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "staking",
        "Staking",
        Some(37),
        HTTP,
        JSON,
    ),
    product(Exchange::Binance, "mining", "Mining", Some(13), HTTP, JSON),
    product(
        Exchange::Binance,
        "crypto_loan",
        "Crypto Loan",
        Some(16),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "vip_loan",
        "VIP Loan",
        Some(14),
        HTTP,
        JSON,
    ),
    product(Exchange::Binance, "c2c", "C2C", Some(1), HTTP, JSON),
    product(Exchange::Binance, "fiat", "Fiat", Some(5), HTTP, JSON),
    product(
        Exchange::Binance,
        "gift_card",
        "Gift Card",
        Some(6),
        HTTP,
        JSON,
    ),
    product(Exchange::Binance, "rebate", "Rebate", Some(1), HTTP, JSON),
    product(
        Exchange::Binance,
        "simple_earn",
        "Simple Earn",
        Some(41),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "discount_buy",
        "Discount Buy",
        Some(4),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Binance,
        "dual_investment",
        "Dual Investment",
        Some(5),
        HTTP,
        JSON,
    ),
    product(Exchange::Binance, "pay", "Pay", Some(1), HTTP, JSON),
    product(
        Exchange::Binance,
        "prediction",
        "Prediction Trading",
        Some(26),
        HTTP,
        JSON,
    ),
    product(
        Exchange::Hyperliquid,
        "info",
        "Info",
        None,
        HYPERLIQUID_REQUEST,
        JSON,
    ),
    product(
        Exchange::Hyperliquid,
        "exchange",
        "Exchange",
        None,
        HYPERLIQUID_REQUEST,
        JSON,
    ),
    product(
        Exchange::Hyperliquid,
        "subscriptions",
        "WebSocket subscriptions",
        None,
        &[ApiInterface::WebSocketStream],
        JSON_BINARY,
    ),
    product(
        Exchange::Hyperliquid,
        "hip3",
        "HIP-3 DEX",
        None,
        HYPERLIQUID_HIP3,
        JSON,
    ),
    product(
        Exchange::Hyperliquid,
        "vaults",
        "Subaccounts and vaults",
        None,
        HTTP_STREAM,
        JSON,
    ),
    product(
        Exchange::Hyperliquid,
        "staking",
        "Staking",
        None,
        HYPERLIQUID_REQUEST,
        JSON,
    ),
    product(
        Exchange::Hyperliquid,
        "outcomes",
        "Outcome markets",
        None,
        HYPERLIQUID_HIP3,
        JSON,
    ),
    product(
        Exchange::Hyperliquid,
        "deployers",
        "Deployer actions",
        None,
        HYPERLIQUID_HIP3,
        JSON,
    ),
    product(
        Exchange::Hyperliquid,
        "hyperevm",
        "HyperEVM",
        None,
        &[ApiInterface::JsonRpc, ApiInterface::Contract],
        JSON,
    ),
];

#[allow(clippy::too_many_arguments)]
const fn operation(
    exchange: Exchange,
    product: &'static str,
    id: &'static str,
    method: &'static str,
    path: &'static str,
    interface: ApiInterface,
    authentication: Authentication,
    risk: OperationRisk,
    mapping: OperationMapping,
    validation: Validation,
) -> OperationCoverage {
    OperationCoverage {
        exchange,
        product,
        id,
        method,
        path,
        interface,
        encoding: Encoding::Json,
        authentication,
        risk,
        mapping,
        implementation: Implementation::Implemented,
        validation,
        availability: Availability::General,
    }
}

/// Initial operation-level inventory. Each provider batch expands this list.
pub const OPERATIONS: &[OperationCoverage] = &[
    operation(
        Exchange::Upbit,
        "quotation",
        "markets",
        "GET",
        "/v1/market/all",
        ApiInterface::Http,
        Authentication::Public,
        OperationRisk::Read,
        OperationMapping::Common("markets"),
        Validation::LiveRead,
    ),
    operation(
        Exchange::Upbit,
        "exchange",
        "balances",
        "GET",
        "/v1/accounts",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("balances"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Upbit,
        "wallet",
        "wallet_status",
        "GET",
        "/v1/status/wallet",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("asset_networks"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Upbit,
        "wallet",
        "deposit_address",
        "GET",
        "/v1/deposits/coin_address",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("deposit_address"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Upbit,
        "wallet",
        "withdraw_chance",
        "GET",
        "/v1/withdraws/chance",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("prepare_withdrawal"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Upbit,
        "wallet",
        "withdraw_addresses",
        "GET",
        "/v1/withdraws/coin_addresses",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("prepare_withdrawal"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Upbit,
        "wallet",
        "withdraw_coin",
        "POST",
        "/v1/withdraws/coin",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::FinancialWrite,
        OperationMapping::Common("withdraw"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Upbit,
        "wallet",
        "deposits",
        "GET",
        "/v1/deposits",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("deposits"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Upbit,
        "wallet",
        "withdrawals",
        "GET",
        "/v1/withdraws",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("withdrawals"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Bithumb,
        "quotation",
        "markets",
        "GET",
        "/v1/market/all",
        ApiInterface::Http,
        Authentication::Public,
        OperationRisk::Read,
        OperationMapping::Common("markets"),
        Validation::LiveRead,
    ),
    operation(
        Exchange::Bithumb,
        "exchange",
        "balances",
        "GET",
        "/v1/accounts",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("balances"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Bithumb,
        "wallet",
        "wallet_status",
        "GET",
        "/v1/status/wallet",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("asset_networks"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Bithumb,
        "wallet",
        "deposit_address",
        "GET",
        "/v1/deposits/coin_address",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("deposit_address"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Bithumb,
        "wallet",
        "withdraw_chance",
        "GET",
        "/v1/withdraws/chance",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("prepare_withdrawal"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Bithumb,
        "wallet",
        "withdraw_addresses",
        "GET",
        "/v1/withdraws/coin_addresses",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("prepare_withdrawal"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Bithumb,
        "wallet",
        "withdraw_coin",
        "POST",
        "/v1/withdraws/coin",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::FinancialWrite,
        OperationMapping::Common("withdraw"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Bithumb,
        "wallet",
        "deposits",
        "GET",
        "/v1/deposits",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("deposits"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Bithumb,
        "wallet",
        "withdrawals",
        "GET",
        "/v1/withdraws",
        ApiInterface::Http,
        Authentication::Jwt,
        OperationRisk::Read,
        OperationMapping::Common("withdrawals"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Binance,
        "spot",
        "exchange_info",
        "GET",
        "/api/v3/exchangeInfo",
        ApiInterface::Http,
        Authentication::Public,
        OperationRisk::Read,
        OperationMapping::Common("markets"),
        Validation::LiveRead,
    ),
    operation(
        Exchange::Binance,
        "usd_m",
        "exchange_info",
        "GET",
        "/fapi/v1/exchangeInfo",
        ApiInterface::Http,
        Authentication::Public,
        OperationRisk::Read,
        OperationMapping::Common("markets"),
        Validation::LiveRead,
    ),
    operation(
        Exchange::Binance,
        "wallet",
        "all_coins_information",
        "GET",
        "/sapi/v1/capital/config/getall",
        ApiInterface::Http,
        Authentication::Hmac,
        OperationRisk::Read,
        OperationMapping::Common("asset_networks"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Binance,
        "wallet",
        "deposit_address",
        "GET",
        "/sapi/v1/capital/deposit/address",
        ApiInterface::Http,
        Authentication::Hmac,
        OperationRisk::Read,
        OperationMapping::Common("deposit_address"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Binance,
        "wallet",
        "api_key_permissions",
        "GET",
        "/sapi/v1/account/apiRestrictions",
        ApiInterface::Http,
        Authentication::Hmac,
        OperationRisk::Read,
        OperationMapping::Common("prepare_withdrawal"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Binance,
        "wallet",
        "withdraw_address_list",
        "GET",
        "/sapi/v1/capital/withdraw/address/list",
        ApiInterface::Http,
        Authentication::Hmac,
        OperationRisk::Read,
        OperationMapping::Common("prepare_withdrawal"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Binance,
        "wallet",
        "questionnaire_requirements",
        "GET",
        "/sapi/v1/localentity/questionnaire-requirements",
        ApiInterface::Http,
        Authentication::Hmac,
        OperationRisk::Read,
        OperationMapping::Common("prepare_withdrawal"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Binance,
        "wallet",
        "withdraw",
        "POST",
        "/sapi/v1/capital/withdraw/apply",
        ApiInterface::Http,
        Authentication::Hmac,
        OperationRisk::FinancialWrite,
        OperationMapping::Common("withdraw"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Binance,
        "wallet",
        "deposit_history",
        "GET",
        "/sapi/v1/capital/deposit/hisrec",
        ApiInterface::Http,
        Authentication::Hmac,
        OperationRisk::Read,
        OperationMapping::Common("deposits"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Binance,
        "wallet",
        "withdraw_history",
        "GET",
        "/sapi/v1/capital/withdraw/history",
        ApiInterface::Http,
        Authentication::Hmac,
        OperationRisk::Read,
        OperationMapping::Common("withdrawals"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Hyperliquid,
        "info",
        "meta",
        "POST",
        "/info type=meta",
        ApiInterface::Http,
        Authentication::Public,
        OperationRisk::Read,
        OperationMapping::Common("markets"),
        Validation::LiveRead,
    ),
    operation(
        Exchange::Hyperliquid,
        "info",
        "user_non_funding_ledger_updates",
        "POST",
        "/info type=userNonFundingLedgerUpdates",
        ApiInterface::Http,
        Authentication::Public,
        OperationRisk::Read,
        OperationMapping::CommonMany(&["deposits", "withdrawals"]),
        Validation::Fixture,
    ),
    operation(
        Exchange::Hyperliquid,
        "exchange",
        "order",
        "POST",
        "/exchange action=order",
        ApiInterface::Http,
        Authentication::Eip712,
        OperationRisk::FinancialWrite,
        OperationMapping::Common("place_order"),
        Validation::Fixture,
    ),
    operation(
        Exchange::Hyperliquid,
        "subscriptions",
        "trades",
        "SUBSCRIBE",
        "trades",
        ApiInterface::WebSocketStream,
        Authentication::Public,
        OperationRisk::Read,
        OperationMapping::Common("subscribe"),
        Validation::LiveRead,
    ),
];
