//! Binding-facing inventories and adapter error classification.

use std::collections::HashSet;

use maxt::{Error, Exchange, Feature};

#[test]
fn binding_inventories_are_complete_stable_and_duplicate_free() {
    assert_eq!(
        Exchange::ALL,
        [
            Exchange::Upbit,
            Exchange::Bithumb,
            Exchange::Binance,
            Exchange::Hyperliquid,
        ]
    );
    assert_eq!(
        Feature::ALL,
        [
            Feature::Markets,
            Feature::Trades,
            Feature::OrderBook,
            Feature::Ticker,
            Feature::Candles,
            Feature::TradeStream,
            Feature::OrderBookStream,
            Feature::TickerStream,
            Feature::CandleStream,
            Feature::Balances,
            Feature::AssetNetworks,
            Feature::DepositAddresses,
            Feature::DepositHistory,
            Feature::WithdrawalQuotes,
            Feature::Withdrawals,
            Feature::WithdrawalHistory,
            Feature::OpenOrders,
            Feature::OrderHistory,
            Feature::AccountStream,
            Feature::Trading,
            Feature::Positions,
            Feature::Margin,
            Feature::FundingRates,
            Feature::FundingPayments,
            Feature::MarginConfig,
            Feature::ReduceOnlyOrders,
        ]
    );

    let exchange_ids: HashSet<_> = Exchange::ALL.map(Exchange::id).into_iter().collect();
    let feature_ids: HashSet<_> = Feature::ALL.map(Feature::id).into_iter().collect();
    assert_eq!(exchange_ids.len(), Exchange::ALL.len());
    assert_eq!(feature_ids.len(), Feature::ALL.len());
    assert_eq!(
        Feature::ALL.map(Feature::id),
        [
            "markets",
            "trades",
            "order_book",
            "ticker",
            "candles",
            "trade_stream",
            "order_book_stream",
            "ticker_stream",
            "candle_stream",
            "balances",
            "asset_networks",
            "deposit_addresses",
            "deposit_history",
            "withdrawal_quotes",
            "withdrawals",
            "withdrawal_history",
            "open_orders",
            "order_history",
            "account_stream",
            "trading",
            "positions",
            "margin",
            "funding_rates",
            "funding_payments",
            "margin_config",
            "reduce_only_orders",
        ]
    );
}

#[test]
fn adapter_contract_errors_are_public_non_retryable_and_readable() {
    let error = Error::adapter("foreign dispatcher returned Ticker for trades");

    assert_eq!(
        error,
        Error::Adapter {
            detail: "foreign dispatcher returned Ticker for trades".to_string(),
        }
    );
    assert_eq!(
        error.to_string(),
        "adapter failed: foreign dispatcher returned Ticker for trades"
    );
    assert!(!error.is_retryable());
    assert!(!error.is_rate_limited());
}
