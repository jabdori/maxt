//! Market identity: which exchange, which instrument, which settlement asset.

use std::fmt;

/// An exchange `maxt` can talk to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Exchange {
    /// Upbit, a Korean spot exchange.
    Upbit,
    /// Bithumb, a Korean spot exchange.
    Bithumb,
    /// Binance, global spot and USD-margined perpetual futures.
    Binance,
    /// Hyperliquid, on-chain spot and perpetual futures.
    Hyperliquid,
}

impl Exchange {
    /// The lowercase identifier used in symbols, logs, and error messages.
    ///
    /// Stable across releases, so it is safe to persist and to match on.
    pub const fn id(self) -> &'static str {
        match self {
            Self::Upbit => "upbit",
            Self::Bithumb => "bithumb",
            Self::Binance => "binance",
            Self::Hyperliquid => "hyperliquid",
        }
    }

    /// The exchange's own preferred spelling, for display to people.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Upbit => "Upbit",
            Self::Bithumb => "Bithumb",
            Self::Binance => "Binance",
            Self::Hyperliquid => "Hyperliquid",
        }
    }
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

/// What kind of instrument a market is.
///
/// `BTC/USDT` spot and `BTC/USDT` perpetual are different markets with
/// different prices, so the kind is part of a market's identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum MarketKind {
    /// Spot: you hold the base asset after the trade settles.
    Spot,
    /// Perpetual futures: no expiry, funding is exchanged periodically.
    Perpetual,
}

impl MarketKind {
    /// Whether this kind carries positions, leverage, and funding.
    ///
    /// The derivatives half of [`Client`](crate::Client), covering positions,
    /// margin, and funding, is meaningful only when this is `true`.
    pub const fn is_derivative(self) -> bool {
        matches!(self, Self::Perpetual)
    }

    const fn suffix(self) -> &'static str {
        match self {
            Self::Spot => "",
            Self::Perpetual => ":perp",
        }
    }
}

/// Identifies one tradable market on one exchange.
///
/// Every market-scoped call in `maxt` takes one of these. Build it with
/// [`Market::spot`] or [`Market::perpetual`]. The adapter translates it into
/// whatever the exchange calls the same instrument: `KRW-BTC` on Upbit,
/// `BTCUSDT` on Binance, `BTC` on Hyperliquid.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Market {
    /// The exchange this market lives on.
    pub exchange: Exchange,
    /// Spot or perpetual.
    pub kind: MarketKind,
    /// The asset being traded, uppercase. For example `BTC`.
    pub base: String,
    /// The asset the market is priced and settled in, uppercase. For example
    /// `KRW` or `USDT`.
    pub quote: String,
}

impl Market {
    /// A spot market, for example `BTC` priced in `KRW`.
    pub fn spot(exchange: Exchange, base: impl AsRef<str>, quote: impl AsRef<str>) -> Self {
        Self::new(exchange, MarketKind::Spot, base, quote)
    }

    /// A perpetual futures market, for example `BTC` settled in `USDT`.
    pub fn perpetual(exchange: Exchange, base: impl AsRef<str>, quote: impl AsRef<str>) -> Self {
        Self::new(exchange, MarketKind::Perpetual, base, quote)
    }

    /// A market of an explicit kind.
    ///
    /// Prefer [`Market::spot`] or [`Market::perpetual`] unless the kind is
    /// itself a runtime value.
    pub fn new(
        exchange: Exchange,
        kind: MarketKind,
        base: impl AsRef<str>,
        quote: impl AsRef<str>,
    ) -> Self {
        Self {
            exchange,
            kind,
            base: base.as_ref().to_ascii_uppercase(),
            quote: quote.as_ref().to_ascii_uppercase(),
        }
    }
}

impl fmt::Display for Market {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}/{}{}",
            self.exchange,
            self.base,
            self.quote,
            self.kind.suffix()
        )
    }
}

/// Whether an exchange is currently accepting orders on a market.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MarketStatus {
    /// Listed and trading.
    Active,
    /// Trading is halted, but the listing still exists.
    Paused,
    /// Delisted.
    Delisted,
    /// The exchange did not say.
    Unknown,
}

/// One entry from [`Client::markets`](crate::Client::markets).
///
/// [`MarketInfo::market`] is the identity to pass back into other calls.
/// [`MarketInfo::native_symbol`] is what the exchange calls the same thing, and
/// helps when reconciling against the exchange's own UI or docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketInfo {
    /// The market, in `maxt` terms.
    pub market: Market,
    /// The exchange's own symbol for it, verbatim. For example `KRW-BTC`.
    pub native_symbol: String,
    /// Whether it is currently trading.
    pub status: MarketStatus,
    /// The asset's name in Korean, when the exchange publishes one.
    pub korean_name: Option<String>,
    /// The asset's name in English, when the exchange publishes one.
    pub english_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spot_and_perpetual_on_one_pair_are_distinct_markets() {
        let spot = Market::spot(Exchange::Binance, "BTC", "USDT");
        let perp = Market::perpetual(Exchange::Binance, "BTC", "USDT");

        assert_ne!(spot, perp);
        assert!(!spot.kind.is_derivative());
        assert!(perp.kind.is_derivative());
    }

    #[test]
    fn assets_are_normalized_to_uppercase() {
        let market = Market::spot(Exchange::Upbit, "btc", "krw");

        assert_eq!(market.base, "BTC");
        assert_eq!(market.quote, "KRW");
        assert_eq!(market, Market::spot(Exchange::Upbit, "BTC", "KRW"));
    }

    #[test]
    fn the_same_pair_on_two_exchanges_is_two_markets() {
        assert_ne!(
            Market::spot(Exchange::Upbit, "BTC", "KRW"),
            Market::spot(Exchange::Bithumb, "BTC", "KRW"),
        );
    }

    #[test]
    fn display_round_trips_every_field_of_the_identity() {
        assert_eq!(
            Market::spot(Exchange::Hyperliquid, "ETH", "USDC").to_string(),
            "hyperliquid:ETH/USDC"
        );
        assert_eq!(
            Market::perpetual(Exchange::Hyperliquid, "ETH", "USDC").to_string(),
            "hyperliquid:ETH/USDC:perp"
        );
    }
}
