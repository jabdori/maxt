//! Every `Feed` the documentation claims an exchange carries, opened against
//! that exchange and counted.
//!
//! Ignored by default, so `cargo test` and CI never reach the network. Run it
//! by name:
//!
//! ```text
//! cargo test --test live_conformance -- --ignored --nocapture
//! ```
//!
//! Public read-only endpoints only. No credentials are read, no order is
//! placed, and no private feed is opened, so the signed half of every adapter
//! is outside what this can say anything about.
//!
//! The offline suite checks one cell at a time: this frame decodes, that
//! interval maps to this path. Two defects walked through it anyway. Bithumb's
//! `Feed::OrderBook` stamps its frames in microseconds where every other
//! Bithumb payload is milliseconds, and a hand-written millisecond fixture kept
//! the suite green over a feed that produced nothing but rejected timestamps.
//! Hyperliquid's candle stream never set `closed`, across five window
//! transitions and 135 events. Neither is visible without subscribing and
//! counting, which is what happens below.
//!
//! What is asserted per pair:
//!
//! * a nonzero count of the feed's own event type, since a successful
//!   subscribe says only that a socket opened
//! * zero errors, since a feed that yields nothing but `Err` still reads as
//!   supported everywhere else
//! * one settled candle at least, on every candle feed
//! * the level count the provider page states, on every book feed
//! * every timestamp the exchange itself sent near the reading machine's wall
//!   clock, which is what a mis-scaled clock and a time-zone-shifted one both
//!   break. Payloads the exchange sends no clock on carry the adapter's own
//!   `Timestamp::now()` instead, so measuring them would compare this machine's
//!   clock to itself. Those are named in [`UNSTAMPED`] and reported as unchecked
//!   rather than as a number

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
/// Long enough to contain a whole `Min1` window wherever in a minute it
/// starts, plus the transition that settles it. Two boundaries are crossed, so
/// a candle feed that settles nothing has had two chances, not one.
const WINDOW: Duration = Duration::from_secs(150);

/// The candle interval every candle feed is opened at.
///
/// The shortest one all four exchanges publish, which is what makes a window
/// boundary observable inside [`WINDOW`] rather than in an hour.
const CANDLE_INTERVAL: Interval = Interval::Min1;

/// How far behind the reading machine's clock a timestamp may sit, in
/// milliseconds.
///
/// A settled `Min1` candle is announced at the end of its window and carries
/// the window's own opening instant, so a minute of lag is correct rather than
/// suspect. Five minutes leaves room for that and for a slow socket, and still
/// rejects the nine-hour offset a wall-clock-in-a-UTC-field bug produces.
const OLDEST_ALLOWED_MS: i64 = 300_000;

/// How far ahead of the reading machine's clock a timestamp may sit, in
/// milliseconds.
///
/// Tighter than the other direction because nothing legitimate is stamped in
/// the future. A microsecond field read as milliseconds lands tens of thousands
/// of years out and is caught here.
const NEWEST_ALLOWED_MS: i64 = 30_000;

/// The venue and payload pairs the exchange sends no clock on.
///
/// The adapter fills the gap with the reading machine's own `Timestamp::now()`,
/// which is the only thing it can do and is right for a caller who wants a
/// `Timestamp` on every event. It leaves this file with nothing to check: `now
/// minus stamp` on these is this machine measured against itself, so the figure
/// is local buffering latency and the claim can never fail. Naming them is what
/// keeps a check that cannot fail out of the report.
///
/// Named by payload rather than by transport, because the gap is in what the
/// venue sends and the stream reader and the REST reader both land on it.
/// Confirmed live: Hyperliquid's `activeAssetCtx` carries no time field on the
/// socket or in the REST body, and Binance spot depth carries no `E` on the
/// stream while `/api/v3/depth` returns only `lastUpdateId`. Binance USD-M
/// stamps its depth on both, so it is not here.
///
/// The cost is that this list is a claim about the venues, checked by hand. A
/// venue that starts sending a clock stays unchecked here until someone
/// notices, which is the trade for not printing a number that means nothing.
const UNSTAMPED: [(&str, &str); 4] = [
    ("hyperliquid", "Ticker"),
    ("hyperliquid", "ticker.timestamp"),
    ("binance-spot", "OrderBook"),
    ("binance-spot", "order_book.timestamp"),
];

/// The four streaming features, each paired with the [`Feed`] a
/// [`Subscription`] carries it as.
///
/// This mapping is the whole derivation. Which pairs run is decided by asking
/// each adapter [`Client::supports`], so an exchange that starts claiming a
/// feed is checked on the commit that claims it, with no list to remember to
/// edit.
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
    /// The venue's most heavily traded market, so that a count of zero means a
    /// dead feed rather than a quiet hour.
    market: Market,
    /// The file under `docs/providers` whose book depth row is the claim being
    /// checked.
    page: &'static str,
}

/// Every exchange configuration `maxt` ships a public constructor for.
///
/// Hyperliquid's testnet is left out on purpose: it is the same adapter aimed
/// at another host, and its books and candles are thin enough that a zero count
/// would say nothing.
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

/// The levels a side a provider page states for `Feed::OrderBook`.
///
/// Read out of the page rather than restated here, so the number this asserts
/// against the exchange is the number a reader was promised. A page whose row
/// no longer carries one fails the pair it belongs to instead of quietly
/// checking nothing.
///
/// Two rows carrying a figure is an error rather than a first match. Binance's
/// one row covers both its venues today, and splitting it per venue is the
/// obvious edit the day they differ. Taking the first would then assert the
/// spot depth against USD-M and still print a level count, which is the shape
/// of silence this file exists to end.
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
/// Surrounding punctuation is stripped, because the pages write the figure as
/// prose: `**30 levels a side**` and `20 levels,` both answer.
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
    /// Events of the feed's own kind, which is the only count that answers
    /// "is this feed carried".
    events: usize,
    /// Events of some other kind on a single-feed subscription, which would
    /// mean the adapter subscribed to something it was not asked for.
    stray: usize,
    /// `Err` items. A feed that only errors is still a feed that delivers
    /// nothing.
    errors: usize,
    /// The first error's text, because the count alone never says what broke.
    first_error: Option<String>,
    /// Candle events with `closed` set.
    settled: usize,
    /// Every distinct `(bids, asks)` pair a book event carried.
    depths: BTreeSet<(usize, usize)>,
    /// The event timestamp furthest from the wall clock, in milliseconds,
    /// positive when the event is behind.
    worst_skew_ms: i64,
    /// Reconnects seen. Not a failure on its own, and worth printing, since a
    /// stream that spent the window reconnecting explains a low count.
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

/// Whether a timestamp sat too far from the reading machine's wall clock.
///
/// The skew is measured as now minus the event, so a positive value is an
/// event behind the clock and a negative one is an event stamped in the future.
fn clock_is_wrong(skew_ms: i64) -> bool {
    !(-NEWEST_ALLOWED_MS..=OLDEST_ALLOWED_MS).contains(&skew_ms)
}

/// Records what one payload's timestamps said about the exchange's clock, or
/// why nothing was said.
///
/// The one place either half of the report decides that, so a payload listed in
/// [`UNSTAMPED`] is skipped whether it arrived over a socket or over REST. What
/// it never does is emit a figure it did not judge: a number in the report that
/// no assertion stands behind reads as evidence the clock was checked.
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
    // One feed per socket, so that an error belongs to a feed. A subscription
    // carrying all of them at once would report Bithumb's 109 rejected book
    // frames as errors of no particular feed.
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
                    .filter(|(bids, asks)| *bids != claimed || *asks != claimed)
                    .collect();
                if !wrong.is_empty() {
                    faults.push(format!(
                        "docs/providers/{} states {claimed} levels a side, saw {}",
                        venue.page,
                        render_depths(&wrong.into_iter().copied().collect())
                    ));
                }
            }
        }
    }

    if tally.events > 0 {
        // The feed's name is the payload here, and the skew is the furthest any
        // one event of it sat from the clock over the whole window.
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

/// Reads the three public REST endpoints that carry a clock, and judges those
/// clocks the same way.
///
/// The stream and the REST response are separate payloads with separate
/// fields, and Bithumb's `/v1/ticker` was nine hours out while its socket was
/// correct. Checking one says nothing about the other.
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
            None => faults.push("trades: an empty list from the busiest market".to_string()),
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

/// Every provider page still states a book depth this file can read.
///
/// Offline, and deliberately not ignored. [`documented_book_depth`] is what
/// turns a documented promise into an assertion, and a row reworded past it
/// would leave the live check asserting nothing while still reporting a level
/// count. The numbers themselves are not restated here: a copy of a claim
/// sitting in a test is the habit this whole file exists to break.
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

/// A payload the exchange sends no clock on gets no clock reading in the report.
///
/// Offline, and deliberately not ignored. The skew handed in is one no real
/// exchange could produce, so any pair that still judged it would fail and any
/// pair that still printed it would be printing a figure about this machine's
/// own clock. Both are what a check that cannot fail looks like from the
/// outside, and both are ruled out here without a network.
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

    // And a payload the exchange does stamp is still judged, so the skip is a
    // named exception rather than the clock check quietly going away.
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

/// Opens every carried feed on every exchange and checks what the docs promise.
///
/// Ignored, because it is the one thing in this repository that touches the
/// network. Takes a little over [`WINDOW`], since every pair runs at once and
/// the slowest claim to check is a candle window ending.
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
