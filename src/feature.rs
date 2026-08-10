//! What an exchange can and cannot do.

use std::fmt;

macro_rules! define_features {
    (
        $(
            $(#[$meta:meta])*
            $variant:ident => ($id:literal, $label:literal),
        )+
    ) => {
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
            $(
                $(#[$meta])*
                $variant,
            )+
        }

        impl Feature {
            /// Every feature variant in stable binding order.
            pub const ALL: [Self; define_features!(@count $($variant),+)] = [
                $(Self::$variant,)+
            ];

            /// The stable lowercase identifier used by language bindings.
            pub const fn id(self) -> &'static str {
                match self {
                    $(Self::$variant => $id,)+
                }
            }

            const fn label(self) -> &'static str {
                match self {
                    $(Self::$variant => $label,)+
                }
            }
        }
    };
    (@count $($variant:ident),+) => {
        <[()]>::len(&[$(define_features!(@unit $variant)),+])
    };
    (@unit $variant:ident) => { () };
}

define_features! {
    /// Listing the exchange's markets.
    Markets => ("markets", "listing markets"),
    /// Reading recent trades over REST.
    Trades => ("trades", "reading recent trades"),
    /// Reading an order book snapshot over REST.
    OrderBook => ("order_book", "reading order books"),
    /// Reading a ticker over REST.
    Ticker => ("ticker", "reading tickers"),
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
    Candles => ("candles", "reading candles"),
    /// Streaming trades.
    TradeStream => ("trade_stream", "streaming trades"),
    /// Streaming order book updates.
    OrderBookStream => ("order_book_stream", "streaming order books"),
    /// Streaming tickers.
    TickerStream => ("ticker_stream", "streaming tickers"),
    /// Streaming candles.
    CandleStream => ("candle_stream", "streaming candles"),
    /// Reading account balances.
    Balances => ("balances", "reading balances"),
    /// Reading live asset and network transfer rules.
    AssetNetworks => ("asset_networks", "reading asset networks"),
    /// Reading exchange-issued deposit addresses.
    DepositAddresses => ("deposit_addresses", "reading deposit addresses"),
    /// Reading deposit history.
    DepositHistory => ("deposit_history", "reading deposit history"),
    /// Checking a withdrawal before submitting it.
    WithdrawalQuotes => ("withdrawal_quotes", "checking withdrawals"),
    /// Submitting withdrawals.
    Withdrawals => ("withdrawals", "submitting withdrawals"),
    /// Reading withdrawal history.
    WithdrawalHistory => ("withdrawal_history", "reading withdrawal history"),
    /// Reading open orders.
    OpenOrders => ("open_orders", "reading open orders"),
    /// Reading one order or final-order history.
    OrderHistory => ("order_history", "reading order history"),
    /// Streaming account balance and order updates.
    AccountStream => ("account_stream", "streaming account updates"),
    /// Placing and cancelling orders.
    Trading => ("trading", "placing and cancelling orders"),
    /// Reading open positions.
    Positions => ("positions", "reading positions"),
    /// Reading account margin state.
    Margin => ("margin", "reading margin state"),
    /// Reading a market's funding rate history.
    FundingRates => ("funding_rates", "reading funding rates"),
    /// Reading an account's funding payment history.
    FundingPayments => ("funding_payments", "reading funding payments"),
    /// Setting leverage and margin mode.
    MarginConfig => ("margin_config", "setting leverage and margin mode"),
    /// Placing orders that can only reduce a position.
    ReduceOnlyOrders => ("reduce_only_orders", "placing reduce-only orders"),
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
            | Self::AssetNetworks
            | Self::DepositAddresses
            | Self::DepositHistory
            | Self::WithdrawalQuotes
            | Self::Withdrawals
            | Self::WithdrawalHistory
            | Self::OpenOrders
            | Self::OrderHistory
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
            Feature::OrderHistory,
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
