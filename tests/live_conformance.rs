//! Checks advertised public feeds against live exchange responses.
//!
//! Ignored by default because it opens public network connections. Run it with:
//!
//! ```text
//! cargo test --test live_conformance -- --ignored --nocapture
//! ```
//!
//! It uses no credentials and checks, for each advertised feed:
//!
//! * at least one event of the requested type and no stream errors
//! * at least one settled candle on every candle feed
//! * a non-empty book no deeper than the provider page states
//! * venue-supplied timestamps near the local wall clock
//!
//! Payloads without a venue clock use `Timestamp::now()` and are listed in
//! [`UNSTAMPED`] so the report does not validate a local fallback against itself.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::time::Duration;

use futures_util::StreamExt;
use futures_util::future::join_all;
use maxt::adapters::{BinanceAdapter, BithumbAdapter, HyperliquidAdapter, UpbitAdapter};
use maxt::{
    Adapter, Client, Exchange, Feature, Feed, Interval, Market, MarketEvent, StreamConfig,
    Subscription, Timestamp,
};

/// How long each subscription is held open.
///
/// Crosses at least two one-minute boundaries so a candle feed has time to emit
/// a settled window.
const WINDOW: Duration = Duration::from_secs(150);

/// The candle interval every candle feed is opened at.
///
/// The shortest interval shared by every venue that advertises a candle stream.
/// Bithumb advertises no candle stream and is therefore not part of this set.
const CANDLE_INTERVAL: Interval = Interval::Min1;

/// How far behind the host system clock a timestamp may sit, in
/// milliseconds.
///
/// Five minutes permits a settled candle's opening time while still detecting a
/// unit or time-zone error.
const OLDEST_ALLOWED_MS: i64 = 300_000;

/// How far ahead of the host system clock a timestamp may sit, in
/// milliseconds.
///
/// No valid event should be materially ahead of the host clock.
const NEWEST_ALLOWED_MS: i64 = 30_000;

/// The venue and payload pairs the exchange sends no clock on.
///
/// Their timestamps are local fallbacks, so clock skew is reported as unchecked
/// instead of comparing the process clock with itself.
const UNSTAMPED: [(&str, &str); 4] = [
    ("hyperliquid", "Ticker"),
    ("hyperliquid", "ticker.timestamp"),
    ("binance-spot", "OrderBook"),
    ("binance-spot", "order_book.timestamp"),
];

/// The four streaming features, each paired with the [`Feed`] a
/// [`Subscription`] carries it as.
///
/// [`Client::supports`] selects which configured venues advertise each pair.
const STREAM_FEEDS: [(Feature, Feed); 4] = [
    (Feature::TradeStream, Feed::Trades),
    (Feature::OrderBookStream, Feed::OrderBook),
    (Feature::TickerStream, Feed::Ticker),
    (Feature::CandleStream, Feed::Candles(CANDLE_INTERVAL)),
];

/// One exchange configuration, and the market its feeds are opened on.
struct Venue {
    /// How the report names it. Binance ships two configurations under one
    /// [`Exchange`], so this is not simply the exchange's name.
    label: &'static str,
    /// Public and unauthenticated, which is all this check is allowed to be.
    client: Client<Box<dyn Adapter>>,
    /// The representative BTC market used for this venue's checks. A zero event
    /// count fails the feed without making a volume-rank claim.
    market: Market,
    /// The file under `docs/providers` whose book depth row is the claim being
    /// checked.
    page: &'static str,
}

/// Production venue configurations covered by this live check.
///
/// Hyperliquid testnet is an alternate host and is outside this test's scope.
fn venues() -> Vec<Venue> {
    vec![
        Venue {
            label: "upbit",
            client: Client::new(Box::new(UpbitAdapter::new()) as _),
            market: Market::spot(Exchange::Upbit, "BTC", "KRW"),
            page: "upbit.md",
        },
        Venue {
            label: "bithumb",
            client: Client::new(Box::new(BithumbAdapter::new()) as _),
            market: Market::spot(Exchange::Bithumb, "BTC", "KRW"),
            page: "bithumb.md",
        },
        Venue {
            label: "binance-spot",
            client: Client::new(Box::new(BinanceAdapter::spot()) as _),
            market: Market::spot(Exchange::Binance, "BTC", "USDT"),
            page: "binance.md",
        },
        Venue {
            label: "binance-usdm",
            client: Client::new(Box::new(BinanceAdapter::usd_m_futures()) as _),
            market: Market::perpetual(Exchange::Binance, "BTC", "USDT"),
            page: "binance.md",
        },
        Venue {
            label: "hyperliquid",
            client: Client::new(Box::new(HyperliquidAdapter::new()) as _),
            market: Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC"),
            page: "hyperliquid.md",
        },
    ]
}

/// The maximum levels per side stated for `Feed::OrderBook`.
///
/// Exactly one depth must be present so the live assertion uses the documented
/// value instead of a duplicated test fixture.
fn documented_book_depth(page: &str) -> Result<usize, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/providers")
        .join(page);
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("could not read docs/providers/{page}: {error}"))?;

    let stated: Vec<usize> = text
        .lines()
        .filter(|line| line.starts_with("| `Feed::OrderBook`"))
        .filter_map(level_count)
        .collect();

    match stated.as_slice() {
        [depth] => Ok(*depth),
        [] => Err(format!(
            "no \"N levels\" in the `Feed::OrderBook` row of docs/providers/{page}"
        )),
        many => Err(format!(
            "docs/providers/{page} states {many:?} levels a side across {} \
             `Feed::OrderBook` rows; this check cannot tell which venue is meant",
            many.len()
        )),
    }
}

/// The number written immediately before the word `levels` on one line.
///
/// Accepts punctuation around counts such as `**30 levels**` and `20 levels,`.
fn level_count(line: &str) -> Option<usize> {
    let words: Vec<&str> = line.split_whitespace().collect();

    words.windows(2).find_map(|pair| {
        pair[1]
            .starts_with("levels")
            .then(|| {
                pair[0]
                    .trim_matches(|c: char| !c.is_ascii_digit())
                    .parse()
                    .ok()
            })
            .flatten()
    })
}

/// What one subscription produced over [`WINDOW`].
#[derive(Default)]
struct Tally {
    /// Events of the requested feed kind.
    events: usize,
    /// Other event kinds on a single-feed subscription.
    stray: usize,
    /// `Err` items.
    errors: usize,
    /// The first error's text.
    first_error: Option<String>,
    /// Candle events with `closed` set.
    settled: usize,
    /// Every distinct `(bids, asks)` pair a book event carried.
    depths: BTreeSet<(usize, usize)>,
    /// The event timestamp furthest from the wall clock, in milliseconds,
    /// positive when the event is behind.
    worst_skew_ms: i64,
    /// Reconnects, reported to explain low event counts.
    reconnects: usize,
}

impl Tally {
    /// Folds one stream item in, measuring its clock against this instant.
    fn record(&mut self, feed: Feed, item: maxt::Result<MarketEvent>) {
        let event = match item {
            Ok(event) => event,
            Err(error) => {
                self.errors += 1;
                self.first_error.get_or_insert_with(|| error.to_string());
                return;
            }
        };

        let stamp = match (&event, feed) {
            (MarketEvent::Trade(trade), Feed::Trades) => trade.timestamp,
            (MarketEvent::OrderBook(book), Feed::OrderBook) => {
                self.depths.insert((book.bids.len(), book.asks.len()));
                book.timestamp
            }
            (MarketEvent::Ticker(ticker), Feed::Ticker) => ticker.timestamp,
            (MarketEvent::Candle(candle), Feed::Candles(interval))
                if candle.interval == interval =>
            {
                if candle.closed {
                    self.settled += 1;
                }
                candle.open_time
            }
            (MarketEvent::Reconnected, _) => {
                self.reconnects += 1;
                return;
            }
            _ => {
                self.stray += 1;
                return;
            }
        };

        self.events += 1;
        let skew = Timestamp::now().as_millis() - stamp.as_millis();
        if skew.abs() > self.worst_skew_ms.abs() {
            self.worst_skew_ms = skew;
        }
    }
}

/// Whether a timestamp sat too far from the host system clock.
///
/// The skew is measured as now minus the event, so a positive value is an
/// event behind the clock and a negative one is an event stamped in the future.
fn clock_is_wrong(skew_ms: i64) -> bool {
    !(-NEWEST_ALLOWED_MS..=OLDEST_ALLOWED_MS).contains(&skew_ms)
}

/// Records clock skew unless [`UNSTAMPED`] names a local fallback.
fn judge_clock(
    label: &str,
    field: &str,
    skew_ms: i64,
    notes: &mut Vec<String>,
    faults: &mut Vec<String>,
) {
    if UNSTAMPED.contains(&(label, field)) {
        notes.push(format!("{field} clock unchecked: the exchange sends none"));
        return;
    }

    notes.push(format!("{field} clock off by {skew_ms}ms"));
    if clock_is_wrong(skew_ms) {
        faults.push(format!(
            "{field} sat {skew_ms}ms from the wall clock, outside -{NEWEST_ALLOWED_MS}ms to +{OLDEST_ALLOWED_MS}ms"
        ));
    }
}

/// One line of the report.
struct Row {
    /// Venue and feed, as the report names the pair.
    subject: String,
    /// Whether every claim held.
    ok: bool,
    /// The numbers, and on a failure what was wrong with them.
    detail: String,
}

impl Row {
    /// Renders as one aligned line, verdict in the middle.
    fn line(&self) -> String {
        let verdict = if self.ok { "ok  " } else { "FAIL" };
        format!("{:<32} {verdict}  {}", self.subject, self.detail)
    }
}

/// Holds one subscription open for [`WINDOW`] and counts what arrives.
async fn observe(venue: &Venue, feed: Feed) -> Tally {
    let subscription = Subscription::new().market(venue.market.clone()).feed(feed);
    // One feed per subscription attributes every event and error to that feed.
    // It deliberately does not cover Binance USD-M's two-socket mixed-feed path.
    let config = StreamConfig {
        max_reconnect_attempts: Some(3),
        ..StreamConfig::default()
    };
    let mut tally = Tally::default();

    let mut stream = match venue.client.subscribe_with(&subscription, &config).await {
        Ok(stream) => stream,
        Err(error) => {
            tally.errors = 1;
            tally.first_error = Some(format!("subscribe refused: {error}"));
            return tally;
        }
    };

    let deadline = tokio::time::Instant::now() + WINDOW;
    while let Ok(Some(item)) = tokio::time::timeout_at(deadline, stream.next()).await {
        tally.record(feed, item);
    }

    tally
}

/// Runs one `Feed` × exchange pair and judges it.
async fn stream_row(venue: &Venue, feed: Feed) -> Row {
    let tally = observe(venue, feed).await;
    let mut faults: Vec<String> = Vec::new();
    let mut notes = vec![
        format!("{} events", tally.events),
        format!("{} errors", tally.errors),
    ];

    if tally.events == 0 {
        faults.push("no event of this feed's own kind arrived".to_string());
    }
    if tally.errors > 0 {
        faults.push(format!(
            "first error: {}",
            tally.first_error.as_deref().unwrap_or("(none recorded)")
        ));
    }
    if tally.stray > 0 {
        faults.push(format!(
            "{} events of another kind on a single-feed subscription",
            tally.stray
        ));
    }
    if tally.reconnects > 0 {
        notes.push(format!("{} reconnects", tally.reconnects));
    }

    if let Feed::Candles(interval) = feed {
        notes.push(format!("{} settled", tally.settled));
        if tally.events > 0 && tally.settled == 0 {
            faults.push(format!(
                "no {interval:?} candle closed across {}s and at least two window boundaries",
                WINDOW.as_secs()
            ));
        }
    }

    if feed == Feed::OrderBook {
        notes.push(format!("levels {}", render_depths(&tally.depths)));
        match documented_book_depth(venue.page) {
            Err(reason) => faults.push(reason),
            Ok(claimed) => {
                let wrong: Vec<&(usize, usize)> = tally
                    .depths
                    .iter()
                    .filter(|(bids, asks)| {
                        *bids == 0 || *asks == 0 || *bids > claimed || *asks > claimed
                    })
                    .collect();
                if !wrong.is_empty() {
                    faults.push(format!(
                        "docs/providers/{} states up to {claimed} levels a side, saw {}",
                        venue.page,
                        render_depths(&wrong.into_iter().copied().collect())
                    ));
                }
            }
        }
    }

    if tally.events > 0 {
        // Judge the worst event skew seen during the window.
        judge_clock(
            venue.label,
            &format!("{feed:?}"),
            tally.worst_skew_ms,
            &mut notes,
            &mut faults,
        );
    }

    finish(format!("{} {feed:?}", venue.label), notes, faults)
}

/// Runs the ticker, order-book, and recent-trade REST reads, then judges only
/// timestamps supplied by the venue.
async fn rest_row(venue: &Venue) -> Row {
    let mut faults: Vec<String> = Vec::new();
    let mut stamps: Vec<(&str, Timestamp)> = Vec::new();

    match venue.client.ticker(&venue.market).await {
        Err(error) => faults.push(format!("ticker: {error}")),
        Ok(ticker) => {
            stamps.push(("ticker.timestamp", ticker.timestamp));
            if let Some(at) = ticker.last_trade_time {
                stamps.push(("ticker.last_trade_time", at));
            }
        }
    }
    match venue.client.order_book(&venue.market, None).await {
        Err(error) => faults.push(format!("order_book: {error}")),
        Ok(book) => stamps.push(("order_book.timestamp", book.timestamp)),
    }
    match venue.client.trades(&venue.market, None).await {
        Err(error) => faults.push(format!("trades: {error}")),
        Ok(trades) => match trades.first() {
            None => faults.push("trades: an empty list from the configured market".to_string()),
            Some(trade) => stamps.push(("trades[0].timestamp", trade.timestamp)),
        },
    }

    let now = Timestamp::now().as_millis();
    let mut notes = vec![format!("{} clocks read", stamps.len())];
    for (field, stamp) in &stamps {
        judge_clock(
            venue.label,
            field,
            now - stamp.as_millis(),
            &mut notes,
            &mut faults,
        );
    }

    finish(format!("{} REST clocks", venue.label), notes, faults)
}

/// Turns numbers and complaints into a row, so both phases report alike.
fn finish(subject: String, notes: Vec<String>, faults: Vec<String>) -> Row {
    let ok = faults.is_empty();
    let detail = if ok {
        notes.join(", ")
    } else {
        format!("{}; {}", notes.join(", "), faults.join("; "))
    };

    Row {
        subject,
        ok,
        detail,
    }
}

/// Book shapes as `15x15`, or every distinct one seen when they differed.
fn render_depths(depths: &BTreeSet<(usize, usize)>) -> String {
    if depths.is_empty() {
        return "none".to_string();
    }

    depths
        .iter()
        .map(|(bids, asks)| format!("{bids}x{asks}"))
        .collect::<Vec<_>>()
        .join(" and ")
}

/// Every provider page states exactly one book depth this test can assert live.
#[test]
fn every_provider_page_states_a_book_depth() {
    for venue in venues() {
        let depth = documented_book_depth(venue.page);

        assert!(
            matches!(depth, Ok(levels) if levels > 0),
            "{}: {depth:?}",
            venue.label
        );
    }
}

/// Verifies local fallbacks stay unchecked while venue timestamps are judged.
#[test]
fn a_payload_the_exchange_does_not_stamp_gets_no_clock_reading() {
    let impossible = OLDEST_ALLOWED_MS * 100;

    for (label, field) in UNSTAMPED {
        let mut notes = Vec::new();
        let mut faults = Vec::new();
        judge_clock(label, field, impossible, &mut notes, &mut faults);

        assert!(
            faults.is_empty(),
            "{label} {field}: judged a stamp this harness wrote itself: {faults:?}"
        );
        assert!(
            !notes.iter().any(|note| note.contains("off by")),
            "{label} {field}: printed a figure nothing asserted on: {notes:?}"
        );
        assert!(
            notes.iter().any(|note| note.contains("unchecked")),
            "{label} {field}: said nothing about the claim it skipped: {notes:?}"
        );
    }

    // A venue timestamp remains checked, so the skip is explicit.
    let mut notes = Vec::new();
    let mut faults = Vec::new();
    judge_clock(
        "binance-usdm",
        "OrderBook",
        impossible,
        &mut notes,
        &mut faults,
    );

    assert_eq!(faults.len(), 1, "{faults:?}");
    assert!(
        notes.iter().any(|note| note.contains("off by")),
        "{notes:?}"
    );
}

/// Opens every advertised feed on each configured venue and checks its contract.
///
/// Ignored because it touches the network and waits through [`WINDOW`]. Each
/// feed is tested separately; mixed-feed socket topology is outside this test.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "opens a socket to every exchange; run with --ignored"]
async fn every_carried_feed_delivers_what_the_docs_promise() {
    let venues = venues();
    let pairs: Vec<(&Venue, Feed)> = venues
        .iter()
        .flat_map(|venue| {
            STREAM_FEEDS
                .iter()
                .filter(|(feature, _)| venue.client.supports(*feature))
                .map(move |(_, feed)| (venue, *feed))
        })
        .collect();

    println!("maxt live conformance");
    println!(
        "{} pairs, from Client::supports on {} exchange configurations:",
        pairs.len(),
        venues.len()
    );
    for venue in &venues {
        let feeds: Vec<String> = pairs
            .iter()
            .filter(|(at, _)| at.label == venue.label)
            .map(|(_, feed)| format!("{feed:?}"))
            .collect();
        println!("  {:<14} {}", venue.label, feeds.join(", "));
    }
    println!("\nreading public REST clocks");

    let mut rows: Vec<Row> = join_all(venues.iter().map(rest_row)).await;
    for row in &rows {
        println!("{}", row.line());
    }

    println!(
        "\nholding {} subscriptions open for {}s",
        pairs.len(),
        WINDOW.as_secs()
    );
    let stream_rows = join_all(pairs.iter().map(|(venue, feed)| stream_row(venue, *feed))).await;
    for row in &stream_rows {
        println!("{}", row.line());
    }
    rows.extend(stream_rows);

    let failed: Vec<String> = rows
        .iter()
        .filter(|row| !row.ok)
        .map(|row| format!("  {}: {}", row.subject, row.detail))
        .collect();

    println!(
        "\n{} of {} checks passed",
        rows.len() - failed.len(),
        rows.len()
    );
    assert!(
        failed.is_empty(),
        "{} of {} live conformance checks failed:\n{}",
        failed.len(),
        rows.len(),
        failed.join("\n")
    );
}
