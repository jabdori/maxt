//! `supports` and the call behind it must agree, in both directions.
//!
//! `Client::supports` is what documentation, capability checks, and routing
//! logic all read. If it disagrees with what the call does, every one of them
//! is wrong. A `true` that then fails is worse than a `false`, because callers
//! branch on it.
//!
//! Both directions are checked here, over the whole `Feature` × adapter
//! configuration cross product.
//!
//! * `supports(f) == false` → the call refuses, as `Unsupported`, naming `f`.
//! * `supports(f) == true` → the call does not answer `Unsupported`.
//!
//! These assertions never touch the network. An adapter that does not offer a
//! feature answers before it would open a connection, which is itself part of
//! what is checked here. A test that hung would mean an adapter is reaching for
//! the wire to report something it already knows.
//!
//! The second direction is harder to check offline, because a feature an
//! adapter *does* offer is usually answered by the exchange. Where a probe
//! resolves before the wire, such as a market belonging to a different exchange
//! or a malformed wallet address, it is used. What it asserts is narrow:
//! whatever else comes back, it must not be `Unsupported`. Where no such probe
//! exists, [`offline_probe`] returns `None` and says why.

use maxt::adapters::{BinanceAdapter, BithumbAdapter, HyperliquidAdapter, UpbitAdapter};
use maxt::{
    Adapter, CandleRequest, Client, Error, Exchange, Feature, Feed, HistoryRequest, Interval,
    MarginRequest, Market, MarketKind, OrderRequest, Side, Size, Subscription, Timestamp,
};

/// Every feature, so the cross product below is a cross product.
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

/// The intervals all four exchanges publish an endpoint for.
///
/// This list is read off the exchanges' own documentation, not off `maxt`. A
/// baseline copied from whatever the adapters happen to implement would assert
/// the code against itself and pass no matter how much an adapter left out,
/// which is exactly how three intervals stayed missing while the errors called
/// them absent from the exchange.
///
/// * Upbit serves `/v1/candles/minutes/{unit}` for units 1, 3, 5, 10, 15, 30,
///   60 and 240, plus `days`, `weeks`, `months`, `years` and a one-second
///   `seconds` endpoint.
/// * Bithumb serves the same minute units, plus `days`, `weeks` and `months`.
/// * Binance and Hyperliquid both take an interval parameter spanning `1m`
///   through `1M`.
///
/// The intersection, restricted to what [`Interval`] can name, is below.
/// Ten-minute candles and Upbit's years are served by exchanges and unnameable
/// here; two, eight and twelve hours, three days and the one-second candle are
/// nameable and served by only some. Both kinds are outside the baseline, so a
/// caller who needs one asks the adapter rather than the feature.
/// The intervals every exchange that streams candles at all publishes a feed for.
///
/// A stream is not the REST endpoint, and the previous list cannot stand in for
/// it: Upbit serves daily, weekly and monthly candles over REST and streams
/// none of them. Read off the exchanges' own documentation, same as above.
///
/// * Upbit streams `candle.{1s,1m,3m,5m,10m,15m,30m,60m,240m}`.
/// * Binance streams every interval its klines endpoint serves, `1s` through
///   `1M`, one-second candles on spot only.
/// * Hyperliquid's `candle` subscription takes the same names its
///   `candleSnapshot` does, `1m` through `1M`.
/// * Bithumb publishes no candle stream at all, so it constrains nothing here
///   and declines [`Feature::CandleStream`] outright, which the refusing
///   direction above already covers.
///
/// The intersection, restricted to what [`Interval`] can name, is below. One
/// second is out because Hyperliquid does not aggregate it; a day and longer are
/// out because Upbit's stream stops at four hours.
const BASELINE_STREAM_INTERVALS: [Interval; 7] = [
    Interval::Min1,
    Interval::Min3,
    Interval::Min5,
    Interval::Min15,
    Interval::Min30,
    Interval::Hour1,
    Interval::Hour4,
];

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
        // An API key is any string until an exchange refuses it, so nothing
        // about one is wrong before the wire.
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
            // Deliberately malformed. `with_wallet` takes it without complaint,
            // so the address is only found to be wrong at the call, which is
            // what makes it a probe that answers offline.
            boxed(HyperliquidAdapter::new().with_wallet("0xabc", "0xdef"))
        } else {
            boxed(HyperliquidAdapter::new())
        },
        market: Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC"),
        elsewhere: Market::spot(Exchange::Upbit, "BTC", "KRW"),
        // Every public call builds the symbol table first, and that is a
        // request; there is nothing Hyperliquid rejects about a market before
        // it knows which markets exist.
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

/// A call for a feature the adapter claims, arranged to answer without a
/// network, or `None` when this adapter cannot answer that one offline.
///
/// What comes back is only ever inspected for one thing: that it is not
/// `Unsupported`. A rejected market, a malformed wallet, and an empty listing
/// are all fine answers; "this exchange does not do that" is not, because
/// `supports` just said it does.
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
        // Everything else is a question only the exchange can answer, and this
        // file does not ask it. The refusing direction still covers every
        // feature on every configuration, and each adapter's own request tests
        // cover what it accepts without a network.
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

            // A `false` here means one of two things, and they are not
            // interchangeable. The exchange may not have the feature at all,
            // which is `Unsupported` and tells a caller to stop asking. Or this
            // client may simply not have authenticated for it, which is `Auth`
            // and tells them to supply a key. Both are honest refusals; what
            // would not be is refusing for one reason and reporting the other,
            // or naming a feature the caller did not ask about.
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

    // Every probe returning `None` would make the loop above pass while
    // asserting nothing. The count is a floor, not a fixture: the public
    // features of the three exchanges that reject a market offline are eight
    // apiece across eight configurations, before the private ones.
    assert!(
        probed >= 48,
        "only {probed} claimed features were actually exercised"
    );
}

#[tokio::test]
async fn every_baseline_interval_is_mapped_on_the_exchanges_that_can_be_asked_offline() {
    // The one place `supports(Feature::Candles)` could be true and the call
    // still refuse: an interval the adapter maps to no endpoint.
    //
    // Not every exchange, despite what the baseline covers. The probe is a
    // market from somewhere else, and only an adapter that rejects one before
    // opening a connection can answer it offline. Hyperliquid builds its symbol
    // table first, so it is skipped here and its interval map is asserted in
    // its own unit tests instead. A name claiming all four would be the same
    // kind of overstatement this file exists to catch.
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
    // The same hole as the test above, on the streaming side: `supports` says
    // `CandleStream` and the subscription then refuses one interval of it.
    //
    // Nothing here asserts that a refused interval is wrong to refuse. Upbit
    // genuinely streams no daily candle, and refusing that is the honest answer.
    // What it asserts is that the intervals every candle-streaming exchange
    // publishes a feed for are all reachable, so `supports(CandleStream) == true`
    // means the same thing on each of them.
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
    // Both refusals are `Unsupported`, and they are not the same sentence: an
    // interval `maxt` has not mapped is an issue to open here, and one the
    // exchange does not publish is not. A caller who cannot tell them apart files
    // against the wrong project.
    //
    // Upbit's candle stream stops at four hours while its REST endpoints go to a
    // month, so these three are the clearest case of the second kind anywhere in
    // the crate.
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
        // And the same interval over REST is served, which is the thing a caller
        // reading the refusal most needs to know.
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
    // Two of the four exchanges have no start-time parameter at all. Refusing
    // `from` there would make `CandleRequest::from` mean something different
    // per exchange, which is exactly what a common API exists to prevent. The
    // count is past every per-response cap on purpose: the cap is on one
    // response, and `CandleRequest::limit` documents that `maxt` pages behind
    // the scenes rather than refusing.
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
    // An exchange that has an endpoint the caller has not authenticated for is
    // an auth failure, not a missing feature. `Unsupported` would tell a caller
    // to stop asking when the fix is to supply a key, and a caller that
    // branches on the error variant would then behave differently per exchange,
    // which is exactly what a common API exists to prevent.
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

    // `recentTrades` is not on Hyperliquid's info reference page, only on its
    // rate-limit list. Absent from the documentation is not absent from the API,
    // and this crate claimed the opposite for a while.
    // What the endpoint does *not* offer is a count. Ten is the whole window, and
    // a wider `limit` is `InvalidRequest` on `limit` rather than a quiet cut,
    // which `rest`'s own unit tests assert without a network.
    assert!(client.supports(Feature::Trades));
    assert!(client.supports(Feature::TradeStream));
}
