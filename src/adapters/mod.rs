//! One adapter per exchange.
//!
//! An adapter is what [`Client`](crate::Client) wraps. Build the one you want,
//! hand it to a client, and use the common API. When you need something only
//! that exchange offers, reach back to the adapter with
//! [`Client::adapter`](crate::Client::adapter) and call its own typed methods.
//!
//! Adapters built with no credentials serve public market data only. Every
//! adapter takes credentials in the form its own exchange uses. Upbit and
//! Bithumb issue an access key and a secret key, Binance an API key and a
//! secret, Hyperliquid a wallet address and a signing key. `maxt` keeps those
//! three shapes distinct.
//!
//! `docs/providers.md` covers which to pick. `docs/providers/` has one page
//! per exchange.

mod binance;
mod bithumb;
mod hyperliquid;
mod upbit;

pub(crate) mod candles;
pub(crate) mod decimal;

pub use binance::{
    BinanceAdapter, BinanceListenKey, BinanceMarket, BinanceSpotOrderDetail, BinanceSymbolFilters,
};
pub use bithumb::{BithumbAdapter, BithumbAlertStep, BithumbMarketAlert};
pub use hyperliquid::{
    HyperliquidAdapter, HyperliquidAssetContext, HyperliquidLedgerEntry, HyperliquidLedgerKind,
};
pub use upbit::{UpbitAdapter, UpbitMarketEvent, UpbitRegion};
