//! What an exchange can and cannot do.

use std::fmt;

/// A capability an exchange may or may not offer.
///
/// `maxt` reports a missing capability as
/// [`Error::Unsupported`](crate::Error::Unsupported) at the call, naming the
/// feature. Ask ahead of time with
/// [`Client::supports`](crate::Client::supports) when the answer should change
/// what your program does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Feature {
    /// Listing the exchange's markets.
    Markets,
    /// Reading recent trades over REST.
    Trades,
    /// Reading an order book snapshot over REST.
    OrderBook,
    /// Reading a ticker over REST.
    Ticker,
    /// Reading historical candles over REST.
    ///
    /// `true` guarantees these intervals: one, three, five, fifteen and thirty
    /// minutes; one and four hours; one day; one week; and one month. Other
    /// intervals are provider-specific.
    ///
    /// An interval outside the baseline is
    /// [`Error::Unsupported`](crate::Error::Unsupported), which says `maxt`
    /// sends that interval nowhere on that exchange. It is not a claim that the
    /// exchange never aggregates it; [`Interval`] does not represent every
    /// interval exposed by every provider.
    ///
    /// [`Interval`]: crate::Interval
    Candles,
    /// Streaming trades.
    TradeStream,
    /// Streaming order book updates.
    OrderBookStream,
    /// Streaming tickers.
    TickerStream,
    /// Streaming candles.
    CandleStream,
    /// Reading account balances.
    Balances,
    /// Reading open orders.
    OpenOrders,
    /// Streaming account balance and order updates.
    AccountStream,
    /// Placing and cancelling orders.
    Trading,
    /// Reading open positions.
    Positions,
    /// Reading account margin state.
    Margin,
    /// Reading a market's funding rate history.
    FundingRates,
    /// Reading an account's funding payment history.
    FundingPayments,
    /// Setting leverage and margin mode.
    MarginConfig,
    /// Placing orders that can only reduce a position.
    ReduceOnlyOrders,
}

impl Feature {
    /// Whether the feature requires API credentials.
    ///
    /// This classifies the operation, not provider availability. For a provider
    /// that supports the operation, missing credentials produce
    /// [`Error::Auth`](crate::Error::Auth). Use
    /// [`Client::supports`](crate::Client::supports) for the configured adapter.
    pub const fn needs_credentials(self) -> bool {
        match self {
            Self::Markets
            | Self::Trades
            | Self::OrderBook
            | Self::Ticker
            | Self::Candles
            | Self::TradeStream
            | Self::OrderBookStream
            | Self::TickerStream
            | Self::CandleStream
            | Self::FundingRates => false,
            Self::Balances
            | Self::OpenOrders
            | Self::AccountStream
            | Self::Trading
            | Self::Positions
            | Self::Margin
            | Self::FundingPayments
            | Self::MarginConfig
            | Self::ReduceOnlyOrders => true,
        }
    }

    /// Whether the feature only exists on derivatives markets.
    pub const fn is_derivatives_only(self) -> bool {
        matches!(
            self,
            Self::Positions
                | Self::Margin
                | Self::FundingRates
                | Self::FundingPayments
                | Self::MarginConfig
                | Self::ReduceOnlyOrders
        )
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Markets => "listing markets",
            Self::Trades => "reading recent trades",
            Self::OrderBook => "reading order books",
            Self::Ticker => "reading tickers",
            Self::Candles => "reading candles",
            Self::TradeStream => "streaming trades",
            Self::OrderBookStream => "streaming order books",
            Self::TickerStream => "streaming tickers",
            Self::CandleStream => "streaming candles",
            Self::Balances => "reading balances",
            Self::OpenOrders => "reading open orders",
            Self::AccountStream => "streaming account updates",
            Self::Trading => "placing and cancelling orders",
            Self::Positions => "reading positions",
            Self::Margin => "reading margin state",
            Self::FundingRates => "reading funding rates",
            Self::FundingPayments => "reading funding payments",
            Self::MarginConfig => "setting leverage and margin mode",
            Self::ReduceOnlyOrders => "placing reduce-only orders",
        }
    }
}

impl fmt::Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_market_data_never_needs_credentials() {
        for feature in [
            Feature::Markets,
            Feature::Trades,
            Feature::OrderBook,
            Feature::Ticker,
            Feature::Candles,
            Feature::TradeStream,
            Feature::CandleStream,
            Feature::FundingRates,
        ] {
            assert!(!feature.needs_credentials(), "{feature:?}");
        }
    }

    #[test]
    fn everything_touching_an_account_needs_credentials() {
        for feature in [
            Feature::Balances,
            Feature::OpenOrders,
            Feature::AccountStream,
            Feature::Trading,
            Feature::Positions,
            Feature::Margin,
            Feature::FundingPayments,
            Feature::MarginConfig,
            Feature::ReduceOnlyOrders,
        ] {
            assert!(feature.needs_credentials(), "{feature:?}");
        }
    }

    #[test]
    fn funding_rates_are_derivatives_only_but_still_public() {
        assert!(Feature::FundingRates.is_derivatives_only());
        assert!(!Feature::FundingRates.needs_credentials());
    }

    #[test]
    fn spot_features_are_not_marked_derivatives_only() {
        for feature in [Feature::Trades, Feature::Balances, Feature::Trading] {
            assert!(!feature.is_derivatives_only(), "{feature:?}");
        }
    }

    #[test]
    fn display_reads_as_a_sentence_fragment_inside_an_error() {
        let message = format!("bithumb does not support {}", Feature::CandleStream);
        assert_eq!(message, "bithumb does not support streaming candles");
    }
}
