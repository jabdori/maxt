//! Checks `Client::supports` against the call behind each feature for every
//! adapter configuration.
//!
//! * A structural `false` returns `Unsupported` naming the requested feature.
//! * A credential-gated `false` on an unauthenticated adapter returns `Auth`.
//! * A `true` never returns `Unsupported` in the offline probes available here.
//!
//! No probe intentionally reaches the network. Claimed features are checked only
//! where an invalid market or credential shape produces an offline result;
//! [`offline_probe`] returns `None` otherwise.

use maxt::adapters::{BinanceAdapter, BithumbAdapter, HyperliquidAdapter, UpbitAdapter};
use maxt::{
    Adapter, CandleRequest, Client, Error, Exchange, Feature, Feed, HistoryRequest, Interval,
    MarginRequest, Market, MarketKind, OrderRequest, Side, Size, Subscription, Timestamp,
};

/// All features covered by the adapter-configuration cross product.
///
/// Written out rather than iterated because [`Feature`] is `#[non_exhaustive]`
/// and has no iterator; the length in the type is what fails when a variant is
/// added and not listed.
const ALL_FEATURES: [Feature; 19] = [
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
    Feature::OpenOrders,
    Feature::AccountStream,
    Feature::Trading,
    Feature::Positions,
    Feature::Margin,
    Feature::FundingRates,
    Feature::FundingPayments,
    Feature::MarginConfig,
    Feature::ReduceOnlyOrders,
];

/// Candle intervals common to every venue that advertises [`Feature::CandleStream`].
///
/// This is the documented intersection for Upbit, Binance, and Hyperliquid.
/// Bithumb advertises no candle stream; Upbit's stream excludes daily and longer
/// intervals, and Hyperliquid excludes one second.
const BASELINE_STREAM_INTERVALS: [Interval; 7] = [
    Interval::Min1,
    Interval::Min3,
    Interval::Min5,
    Interval::Min15,
    Interval::Min30,
    Interval::Hour1,
    Interval::Hour4,
];

/// The [`Interval`] values common to all four REST candle APIs.
///
/// This documented baseline is independent of the adapter mappings so the test
/// does not certify an implementation from its own interval table.
const BASELINE_INTERVALS: [Interval; 10] = [
    Interval::Min1,
    Interval::Min3,
    Interval::Min5,
    Interval::Min15,
    Interval::Min30,
    Interval::Hour1,
    Interval::Hour4,
    Interval::Day1,
    Interval::Week1,
    Interval::Month1,
];

/// One adapter, configured one way, with what a probe of it needs.
struct Case {
    name: &'static str,
    client: Client<Box<dyn Adapter>>,
    /// A market this exchange actually lists.
    market: Market,
    /// A market it does not, for probes that must stop before the wire.
    elsewhere: Market,
    /// Whether this adapter rejects a market it does not list without first
    /// opening a connection.
    checks_markets_offline: bool,
    /// Whether its private calls reject a malformed credential without first
    /// opening a connection.
    checks_credentials_offline: bool,
    /// Whether this configuration was given credentials at all.
    credentialed: bool,
}

fn boxed(adapter: impl Adapter + 'static) -> Client<Box<dyn Adapter>> {
    Client::new(Box::new(adapter) as Box<dyn Adapter>)
}

fn upbit(credentials: bool) -> Case {
    Case {
        name: if credentials { "upbit+keys" } else { "upbit" },
        client: if credentials {
            boxed(UpbitAdapter::new().with_credentials("key", "secret"))
        } else {
            boxed(UpbitAdapter::new())
        },
        market: Market::spot(Exchange::Upbit, "BTC", "KRW"),
        elsewhere: Market::spot(Exchange::Binance, "BTC", "USDT"),
        checks_markets_offline: true,
        // API keys are validated only by the exchange.
        checks_credentials_offline: false,
        credentialed: credentials,
    }
}

fn bithumb(credentials: bool) -> Case {
    Case {
        name: if credentials {
            "bithumb+keys"
        } else {
            "bithumb"
        },
        client: if credentials {
            boxed(BithumbAdapter::new().with_credentials("key", "secret"))
        } else {
            boxed(BithumbAdapter::new())
        },
        market: Market::spot(Exchange::Bithumb, "BTC", "KRW"),
        elsewhere: Market::spot(Exchange::Upbit, "BTC", "KRW"),
        checks_markets_offline: true,
        checks_credentials_offline: false,
        credentialed: credentials,
    }
}

fn binance(spot: bool, credentials: bool) -> Case {
    let adapter = if spot {
        BinanceAdapter::spot()
    } else {
        BinanceAdapter::usd_m_futures()
    };
    Case {
        name: match (spot, credentials) {
            (true, true) => "binance-spot+keys",
            (true, false) => "binance-spot",
            (false, true) => "binance-usdm+keys",
            (false, false) => "binance-usdm",
        },
        client: if credentials {
            boxed(adapter.with_credentials("key", "secret"))
        } else {
            boxed(adapter)
        },
        market: if spot {
            Market::spot(Exchange::Binance, "BTC", "USDT")
        } else {
            Market::perpetual(Exchange::Binance, "BTC", "USDT")
        },
        elsewhere: Market::spot(Exchange::Upbit, "BTC", "KRW"),
        checks_markets_offline: true,
        checks_credentials_offline: false,
        credentialed: credentials,
    }
}

fn hyperliquid(wallet: bool) -> Case {
    Case {
        name: if wallet {
            "hyperliquid+wallet"
        } else {
            "hyperliquid"
        },
        client: if wallet {
            // The malformed wallet is rejected at the call before network I/O.
            boxed(HyperliquidAdapter::new().with_wallet("0xabc", "0xdef"))
        } else {
            boxed(HyperliquidAdapter::new())
        },
        market: Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC"),
        elsewhere: Market::spot(Exchange::Upbit, "BTC", "KRW"),
        // Public probes require Hyperliquid's network-built symbol table.
        checks_markets_offline: false,
        checks_credentials_offline: wallet,
        credentialed: wallet,
    }
}

fn every_configuration() -> Vec<Case> {
    vec![
        upbit(false),
        upbit(true),
        bithumb(false),
        bithumb(true),
        binance(true, false),
        binance(true, true),
        binance(false, false),
        binance(false, true),
        hyperliquid(false),
        hyperliquid(true),
    ]
}

fn order(market: Market) -> OrderRequest {
    OrderRequest::limit(
        market,
        Side::Buy,
        Size::Base(maxt::Decimal::ONE),
        maxt::Decimal::from(1),
    )
}

async fn subscribe(case: &Case, market: Market, feed: Feed) -> Option<Error> {
    case.client
        .subscribe(&Subscription::new().market(market).feed(feed))
        .await
        .err()
}

/// Makes the call behind a feature.
async fn call(case: &Case, feature: Feature, market: &Market) -> Option<Error> {
    let client = &case.client;
    let market = market.clone();

    match feature {
        // A venue asked for the kind it does not list answers "none" from
        // memory; that is a listing, not a refusal.
        Feature::Markets => client.markets(MarketKind::Perpetual).await.err(),
        Feature::Trades => client.trades(&market, Some(1)).await.err(),
        Feature::OrderBook => client.order_book(&market, None).await.err(),
        Feature::Ticker => client.ticker(&market).await.err(),
        Feature::Candles => client
            .candles(&CandleRequest::new(market, Interval::Min1))
            .await
            .err(),
        Feature::TradeStream => subscribe(case, market, Feed::Trades).await,
        Feature::OrderBookStream => subscribe(case, market, Feed::OrderBook).await,
        Feature::TickerStream => subscribe(case, market, Feed::Ticker).await,
        Feature::CandleStream => subscribe(case, market, Feed::Candles(Interval::Min1)).await,
        Feature::Balances => client.balances().await.err(),
        Feature::OpenOrders => client.open_orders_on(&market).await.err(),
        Feature::AccountStream => client.subscribe_account().await.err(),
        Feature::Trading => client.place_order(&order(market)).await.err(),
        Feature::Positions => client.positions_on(&market).await.err(),
        Feature::Margin => client.margin_summary().await.err(),
        Feature::FundingRates => client
            .funding_rates(&HistoryRequest::new(market))
            .await
            .err(),
        Feature::FundingPayments => client
            .funding_payments(&HistoryRequest::new(market))
            .await
            .err(),
        Feature::MarginConfig => client
            .set_margin(&MarginRequest::new(market).leverage(maxt::Decimal::from(2)))
            .await
            .err(),
        Feature::ReduceOnlyOrders => client.place_order(&order(market).reduce_only()).await.err(),
        other => panic!("no call wired up for {other:?}"),
    }
}

/// Probes a claimed feature offline, or returns `None` when network I/O is required.
/// Any offline result except `Unsupported` satisfies the claim.
async fn offline_probe(case: &Case, feature: Feature) -> Option<Error> {
    match feature {
        // Answerable from memory everywhere except Hyperliquid, which builds
        // its symbol table before it can say which markets exist.
        Feature::Markets if case.checks_markets_offline => call(case, feature, &case.market).await,
        // A market on another exchange never reaches the wire.
        Feature::Trades
        | Feature::OrderBook
        | Feature::Ticker
        | Feature::Candles
        | Feature::FundingRates
        | Feature::TradeStream
        | Feature::OrderBookStream
        | Feature::TickerStream
        | Feature::CandleStream
            if case.checks_markets_offline =>
        {
            call(case, feature, &case.elsewhere).await
        }
        // A credential that is malformed rather than merely unaccepted is
        // rejected before a connection is opened. Only Hyperliquid has a
        // credential with a shape to be wrong about.
        Feature::Balances
        | Feature::OpenOrders
        | Feature::Positions
        | Feature::Margin
        | Feature::FundingPayments
        | Feature::MarginConfig
        | Feature::Trading
        | Feature::ReduceOnlyOrders
        | Feature::AccountStream
            if case.checks_credentials_offline =>
        {
            call(case, feature, &case.market).await
        }
        // Other claimed features require the network; adapter unit tests cover them.
        _ => None,
    }
}

#[tokio::test]
async fn a_feature_an_adapter_declines_is_declined_by_the_call_behind_it() {
    for case in every_configuration() {
        for feature in ALL_FEATURES {
            if case.client.supports(feature) {
                continue;
            }

            let Some(error) = call(&case, feature, &case.market).await else {
                panic!("{} answered {feature:?} instead of declining it", case.name);
            };

            // Structural gaps are `Unsupported`; missing credentials are `Auth`.
            // An `Unsupported` result must name the feature that was requested.
            match error {
                Error::Unsupported {
                    feature: reported, ..
                } => assert_eq!(
                    reported, feature,
                    "{} declined {feature:?} but blamed {reported:?}",
                    case.name
                ),
                Error::Auth { .. } if feature.needs_credentials() && !case.credentialed => {}
                other => panic!(
                    "{} declined {feature:?} as {other:?}; an absent feature is `Unsupported` \
                     and a missing key is `Auth`, never a transport or request error",
                    case.name
                ),
            }
        }
    }
}

#[tokio::test]
async fn a_feature_an_adapter_claims_is_never_answered_with_unsupported() {
    let mut probed = 0;

    for case in every_configuration() {
        for feature in ALL_FEATURES {
            if !case.client.supports(feature) {
                continue;
            }

            let Some(error) = offline_probe(&case, feature).await else {
                continue;
            };
            probed += 1;

            assert!(
                !matches!(error, Error::Unsupported { .. }),
                "{} claims {feature:?} and then answers {error:?}",
                case.name
            );
        }
    }

    // Prevent every claimed-feature probe from being skipped.
    assert!(
        probed >= 48,
        "only {probed} claimed features were actually exercised"
    );
}

#[tokio::test]
async fn every_baseline_interval_is_mapped_on_the_exchanges_that_can_be_asked_offline() {
    // An unsupported result here would expose a missing REST interval mapping.
    // Hyperliquid needs a network-built symbol table, so its mapping is covered
    // by adapter unit tests rather than this offline probe.
    let mut asserted = 0;

    for case in every_configuration() {
        if !case.client.supports(Feature::Candles) || !case.checks_markets_offline {
            continue;
        }

        for interval in BASELINE_INTERVALS {
            let error = case
                .client
                .candles(&CandleRequest::new(case.elsewhere.clone(), interval))
                .await
                .expect_err("a market from another exchange is not a valid request");

            assert!(
                !matches!(error, Error::Unsupported { .. }),
                "{} serves candles but not at {interval:?}: {error:?}",
                case.name
            );
            asserted += 1;
        }
    }

    // Upbit, Bithumb, and both Binance venues, credentialled and not.
    assert_eq!(asserted, 8 * BASELINE_INTERVALS.len());
}

#[tokio::test]
async fn every_baseline_stream_interval_is_mapped_on_the_exchanges_that_can_be_asked_offline() {
    // An unsupported result here would expose a missing stream interval mapping.
    // The baseline contains only intervals shared by candle-streaming venues;
    // venue-specific gaps such as Upbit daily candles remain valid refusals.
    let mut asserted = 0;

    for case in every_configuration() {
        if !case.client.supports(Feature::CandleStream) || !case.checks_markets_offline {
            continue;
        }

        for interval in BASELINE_STREAM_INTERVALS {
            let error = subscribe(&case, case.elsewhere.clone(), Feed::Candles(interval))
                .await
                .expect("a market from another exchange is not a valid subscription");

            assert!(
                !matches!(error, Error::Unsupported { .. }),
                "{} streams candles but not at {interval:?}: {error:?}",
                case.name
            );
            asserted += 1;
        }
    }

    // Upbit and both Binance venues, credentialled and not. Bithumb streams no
    // candles and Hyperliquid answers nothing offline.
    assert_eq!(asserted, 6 * BASELINE_STREAM_INTERVALS.len());
}

#[tokio::test]
async fn an_interval_an_exchange_does_not_stream_is_refused_as_the_exchanges_gap() {
    // Distinguish an unmapped maxt interval from a venue's stream gap.
    // Upbit serves day, week, and month candles only over REST.
    let upbit = upbit(false);

    for interval in [Interval::Day1, Interval::Week1, Interval::Month1] {
        let Some(Error::Unsupported { detail, .. }) =
            subscribe(&upbit, upbit.market.clone(), Feed::Candles(interval)).await
        else {
            panic!("upbit streams no {interval:?} candles and should say so");
        };

        assert!(
            detail.contains("upbit streams candles at"),
            "{interval:?} was refused without saying what upbit streams: {detail}"
        );
        assert!(
            !detail.contains("no stream mapped"),
            "{interval:?} was refused as a gap in `maxt`: {detail}"
        );
        // REST serves the same interval.
        assert!(
            upbit
                .client
                .candles(&CandleRequest::new(upbit.elsewhere.clone(), interval))
                .await
                .is_err_and(|error| !matches!(error, Error::Unsupported { .. })),
            "{interval:?} is refused over REST too"
        );
    }
}

#[tokio::test]
async fn a_start_time_and_an_over_cap_count_are_things_every_exchange_accepts() {
    // `from` and a paged `limit` remain common inputs across adapters.
    for case in every_configuration() {
        if !case.checks_markets_offline {
            continue;
        }

        let request = CandleRequest::new(case.elsewhere.clone(), Interval::Min1)
            .from(Timestamp::from_secs(1_499_040_000))
            .limit(10_000);

        let error = case
            .client
            .candles(&request)
            .await
            .expect_err("a market from another exchange is not a valid request");

        assert!(
            !matches!(
                error,
                Error::Unsupported { .. } | Error::InvalidRequest { field: "limit", .. }
            ),
            "{} refused a start time or a paged count: {error:?}",
            case.name
        );
    }
}

#[tokio::test]
async fn a_spot_exchange_reports_no_perpetuals_rather_than_refusing_the_question() {
    // "Which perpetuals do you list?" is answerable by a spot exchange, and the
    // answer is "none". That is different from a feature it does not have.
    for client in [
        boxed(UpbitAdapter::new()),
        boxed(BithumbAdapter::new()),
        boxed(BinanceAdapter::spot()),
    ] {
        assert!(client.supports(Feature::Markets));
        let markets = client
            .markets(MarketKind::Perpetual)
            .await
            .expect("listing perpetuals is a question a spot exchange can answer");
        assert!(markets.is_empty());
    }
}

#[tokio::test]
async fn missing_credentials_read_the_same_way_on_every_exchange() {
    // Missing credentials are `Auth`; `Unsupported` is structural.
    let anonymous: [(&str, Box<dyn Adapter>); 4] = [
        ("upbit", Box::new(UpbitAdapter::new())),
        ("bithumb", Box::new(BithumbAdapter::new())),
        ("binance", Box::new(BinanceAdapter::spot())),
        ("hyperliquid", Box::new(HyperliquidAdapter::new())),
    ];

    for (name, adapter) in anonymous {
        let client = Client::new(adapter);

        match client.balances().await {
            Err(Error::Auth { .. }) => {}
            Err(other) => panic!("{name} reported missing credentials as {other:?}"),
            Ok(_) => panic!("{name} served balances with no credentials"),
        }
    }
}

#[tokio::test]
async fn every_private_feature_is_closed_until_credentials_are_supplied() {
    for case in every_configuration() {
        if case.credentialed {
            continue;
        }

        for feature in ALL_FEATURES {
            if feature.needs_credentials() {
                assert!(
                    !case.client.supports(feature),
                    "{} offers {feature:?} without credentials",
                    case.name
                );
            }
        }
        // Public market data stays open to everyone.
        for feature in [Feature::Markets, Feature::OrderBook, Feature::Candles] {
            assert!(
                case.client.supports(feature),
                "{} withholds public {feature:?}",
                case.name
            );
        }
    }
}

#[tokio::test]
async fn hyperliquid_serves_recent_trades_over_rest_as_well_as_live() {
    let client = Client::new(HyperliquidAdapter::new());

    // `recentTrades` has a fixed ten-item window; wider limits are invalid.
    assert!(client.supports(Feature::Trades));
    assert!(client.supports(Feature::TradeStream));
}
