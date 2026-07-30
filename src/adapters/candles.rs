//! One answer to "candles since T", for every exchange.
//!
//! [`CandleRequest`] means the same thing everywhere, and none of the four
//! exchanges can express all of it in one call. Each caps a response, and two
//! have no start-time parameter at all. The shape they all share is "the newest
//! `count` candles opening before this instant". That is the only thing an
//! adapter has to supply, and the walk that turns it into the requested window
//! happens once, here.
//!
//! The contract this holds up, on all four:
//!
//! * The answer is oldest-first.
//! * [`CandleRequest::from`] is honoured, as far as [`MAX_CALLS`] pages reach.
//!   A window wider than that is refused before the first call rather than
//!   walked, because the walk is sequential and an open-ended one never ends.
//! * [`CandleRequest::limit`] is honoured past what one call can carry, by
//!   paging, up to the same ceiling. It is refused only when reaching it would
//!   cost more calls than that.
//! * `from` and `limit` together mean the *oldest* `limit` candles at or after
//!   `from`, which is what a backfill loop asks for, at every interval including
//!   [`Interval::Month1`](crate::Interval::Month1). `limit` alone means the
//!   newest that many.
//!
//! Nothing here decides for itself how long an interval is.
//! [`Interval::advance`](crate::Interval::advance) answers that, once, for the
//! window a count describes and the window a `from` and a `to` describe alike.
//! That answer bounds the walk; it never bounds the answer. An exchange whose
//! buckets are not the length `advance` names, as Hyperliquid's 30-day `1M` is
//! not a calendar month, holds a different number of candles in the same
//! window, so a read anchored at a `from` stops when a page reaches that start
//! time rather than when it has counted out an estimate.

use std::future::Future;

use crate::error::{Error, Result};
use crate::request::CandleRequest;
use crate::types::{Candle, Timestamp};

/// The most calls one [`read`] is allowed to make.
///
/// Every exchange here pages backwards one response at a time, so a walk costs
/// one sequential round trip per page and nothing about a wide window makes it
/// faster. Without a ceiling, `from` alone describes a walk with no end: at
/// Upbit's two hundred candles a call, one-minute candles since the epoch is
/// about 145,000 calls, which no exchange would serve and no caller would
/// outlive. A request that would need more pages than this is refused before
/// the first one, naming the field that made it too wide.
///
/// [`read`] counts its calls against this as well, because the pre-flight
/// arithmetic bounds the pages only while every page carries a full response of
/// candles the walk has not already seen.
pub(crate) const MAX_CALLS: u32 = 100;

/// Reads the candles a request asks for, oldest first.
///
/// `fetch(end, count)` must answer with the newest `count` candles opening
/// before `end`, or before the present when `end` is `None`. An exchange whose
/// end of range is inclusive, as Hyperliquid is by taking a window rather than
/// a count, answers with the candle at `end` as well; the walk drops it, so a
/// page may carry one more than it asked for. It is called more than once only
/// when one call cannot hold the answer; a request that fits in one page costs
/// exactly one call, and no request costs more than [`MAX_CALLS`], which is
/// counted rather than assumed: a walk that has spent them all and still has
/// candles to collect says so instead of returning a short answer.
///
/// `max_per_call` is the exchange's own cap on a single response, so
/// `max_per_call * MAX_CALLS` is the widest answer this will assemble.
/// `exchange` names it in the one error this raises on its own, a cursor that
/// stops moving, which would otherwise be an endless loop.
pub(crate) async fn read<F, Fut>(
    request: &CandleRequest,
    exchange: &'static str,
    max_per_call: u32,
    fetch: F,
) -> Result<Vec<Candle>>
where
    F: Fn(Option<Timestamp>, u32) -> Fut,
    Fut: Future<Output = Result<Vec<Candle>>>,
{
    if request.limit == Some(0) {
        return Err(Error::invalid_request(
            "limit",
            "asking for no candles is not a request; leave `limit` unset for a default page",
        ));
    }
    if let (Some(from), Some(to)) = (request.from, request.to)
        && from >= to
    {
        return Err(Error::invalid_request("from", "must be earlier than `to`"));
    }

    let (target, field) = match (request.limit, request.from) {
        (Some(limit), _) => (limit as usize, "limit"),
        // With a start time and no cap, the window itself is the cap, and a
        // window is as wide as the caller cares to make it, so it is the thing
        // that has to be measured before anything is fetched.
        (None, Some(from)) => (window_candles(request, from), "from"),
        // With neither, one page of the most recent candles is the answer.
        (None, None) => (max_per_call as usize, "limit"),
    };
    let ceiling = max_per_call as usize * MAX_CALLS as usize;
    if target > ceiling {
        return Err(Error::invalid_request(
            field,
            format!(
                "that window is about {target} {:?} candles, and {exchange} serves {max_per_call} \
                 per call; `maxt` walks at most {MAX_CALLS} calls, so {ceiling} is the most one \
                 request can read. Ask for a narrower window and page it yourself.",
                request.interval
            ),
        ));
    }

    let mut cursor = window_end(request)?;
    let mut collected: Vec<Candle> = Vec::new();
    let mut previous_oldest: Option<Timestamp> = None;
    let mut calls: u32 = 0;

    loop {
        // How many one page asks for. Without a `from`, `target` is the
        // answer's own length and every page needs only what is still missing.
        // With one, the answer is anchored at the oldest end: the candles in
        // hand are the newest of the window and the ones still to read are
        // older than all of them, so nothing gathered so far shortens the next
        // page.
        let wanted = if request.from.is_some() {
            target
        } else {
            target.saturating_sub(collected.len())
        };
        let count = u32::try_from(wanted)
            .unwrap_or(max_per_call)
            .min(max_per_call);
        if count == 0 {
            break;
        }
        // The ceiling this file promises, counted. Bounding `target` bounds the
        // calls only if every page yields a full `count` of candles nobody has
        // seen yet, and an exchange that re-serves the candle the cursor names
        // yields one fewer. Returning what was gathered would be a short answer
        // presented as a complete one.
        if calls >= MAX_CALLS {
            return Err(Error::exchange(
                exchange,
                "candle_pagination_ceiling",
                format!(
                    "walked {MAX_CALLS} pages of {exchange} candles and gathered {} of the \
                     {target} asked for; each page carried fewer candles the walk had not \
                     already seen than it asked for",
                    collected.len()
                ),
            ));
        }
        calls += 1;

        let mut page = fetch(cursor, count).await?;
        let page_len = page.len();
        let Some(oldest) = page.iter().map(|candle| candle.open_time).min() else {
            break;
        };

        // Two different things look alike here, and only one of them is a
        // fault. An exchange that ignores the cursor answers with the same full
        // page forever, which is the endless loop. An exchange that reads the
        // end of its range inclusively, as Hyperliquid does, answers the last
        // page of a market's history with just the candle the cursor names,
        // and that is the history ending, not a stall.
        let prior = previous_oldest;
        let progressed = prior.is_none_or(|previous| oldest < previous);
        if !progressed {
            if page_len >= count as usize {
                return Err(Error::exchange(
                    exchange,
                    "candle_pagination_stalled",
                    "the candle cursor did not move backwards between pages",
                ));
            }
            break;
        }
        previous_oldest = Some(oldest);

        page.retain(|candle| {
            // Whatever the previous page already held. Dropping it here keeps
            // `collected.len()` an honest count without re-sorting the whole
            // run once per page.
            prior.is_none_or(|prior| candle.open_time < prior)
                && request.from.is_none_or(|from| candle.open_time >= from)
                && request.to.is_none_or(|to| candle.open_time < to)
        });
        collected.extend(page);

        let short_page = page_len < count as usize;
        let reached_start = request.from.is_some_and(|from| oldest <= from);
        // `target` ends the walk only when the caller named no `from`, because
        // only then is it the answer's own length. With a `from` it is worked
        // out from the interval's nominal length, and an exchange whose buckets
        // are shorter than that holds more of them in the same window; stopping
        // there would end the walk before it reached the start time, cap a
        // `from`-only read at the estimate, and leave a `from` and `limit` read
        // holding the newest `limit` of the window where the oldest were asked
        // for. What ends it instead is reaching the start time, which is the
        // window's real bound.
        let counted_out = request.from.is_none() && collected.len() >= target;
        if counted_out || short_page || reached_start {
            break;
        }
        cursor = Some(oldest);
    }

    // Once, at the end. Pages arrive newest window first and each one is
    // already in order, so this is a near-sorted run rather than the whole
    // history shuffled; doing it per page instead would be quadratic in the
    // number of candles read.
    collected.sort_by_key(|candle| candle.open_time);
    collected.dedup_by_key(|candle| candle.open_time);
    // Which end the overshoot is at depends on which end the caller named. A
    // `from` fixes the front of the window, so anything spare is at the newest
    // end and `limit` counts forwards from `from`; without a `from` the answer
    // is the newest `target` and the spare is at the oldest end. A page can
    // carry one more than it asked for on an exchange that reads its window
    // inclusively, so both ends really do overshoot.
    //
    // With a `from` and no `limit` nothing is cut at all: `target` there is a
    // page budget worked out from the interval's nominal length, and an
    // exchange whose buckets do not match that length would lose candles the
    // window genuinely holds. The `retain` above is the window's real bound,
    // and the walk above stops on the start time rather than on that budget, so
    // there is nothing here for it to cut back to.
    if request.from.is_some() {
        if let Some(limit) = request.limit {
            collected.truncate(limit as usize);
        }
    } else {
        collected.drain(..collected.len().saturating_sub(target));
    }
    Ok(collected)
}

/// How many candles a start time asks for when no `limit` bounds it.
///
/// The window runs from `from` to `to`, or to the present when the caller named
/// no end, and the answer is how many candle open times fall inside it *at the
/// interval's nominal length*. [`Interval::advance`](crate::Interval::advance)
/// is what counts them, rather than a division by that length: a month's length
/// depends on which month it is, and dividing needs one number for all of them.
///
/// An estimate, not the answer's length. An exchange on a shorter grid than
/// `advance` names holds more candles in the same window, so [`read`] uses this
/// to refuse a window too wide to walk and to size its pages, and stops the
/// walk on the start time itself.
///
/// Found by doubling and then halving, because the whole point of this number is
/// to refuse windows of millions of candles before anything is fetched, and
/// stepping to such a window one candle at a time would cost more than the walk
/// it is protecting against.
fn window_candles(request: &CandleRequest, from: Timestamp) -> usize {
    let end = request.to.unwrap_or_else(Timestamp::now);
    let inside = |count: i64| {
        request
            .interval
            .advance(from, count)
            .is_some_and(|at| at < end)
    };

    if !inside(0) {
        // A window that starts at or after its end holds no candles. `read`
        // rejects `from >= to`; this is `from` in the future with no `to`.
        return 0;
    }

    let mut outside: i64 = 1;
    while inside(outside) {
        let Some(doubled) = outside.checked_mul(2) else {
            return usize::MAX;
        };
        outside = doubled;
    }

    // `outside` is now past the end of the window and `outside / 2` is inside it.
    let mut last_inside = outside / 2;
    while outside - last_inside > 1 {
        let midpoint = last_inside + (outside - last_inside) / 2;
        if inside(midpoint) {
            last_inside = midpoint;
        } else {
            outside = midpoint;
        }
    }

    // Plus the candle at `from` itself, which `inside(0)` just confirmed.
    usize::try_from(last_inside + 1).unwrap_or(usize::MAX)
}

/// Where the backwards walk starts.
///
/// `to` when the caller named one. A start time and a count together describe a
/// window that many intervals wide, and the walk begins at its far end. Asking
/// an exchange for "the newest 100" and then paging back to a `from` six months
/// ago would read six months of candles to return a hundred.
///
/// "That many intervals wide" is
/// [`Interval::advance`](crate::Interval::advance)'s answer, so a monthly window
/// is that many calendar months and not that many multiples of some length a
/// month does not have. That makes this an opening bid for where to start
/// reading rather than the exact far end of the answer: on an exchange whose
/// buckets are shorter, the window holds more than `limit` of them, and [`read`]
/// walks back to `from` and keeps the oldest `limit`.
///
/// `Ok(None)` means "start at the present", which is what a request naming
/// neither end asks for. A `from` and a `limit` whose far end falls outside the
/// range [`Timestamp`] can hold is `Error::InvalidRequest` instead: the walk
/// would otherwise begin at the present, which is a different window from the
/// one asked for, and answer it without saying so.
fn window_end(request: &CandleRequest) -> Result<Option<Timestamp>> {
    let Some((from, limit)) = request.from.zip(request.limit) else {
        return Ok(request.to);
    };
    let derived = request
        .interval
        .advance(from, i64::from(limit))
        .ok_or_else(|| {
            Error::invalid_request(
                "from",
                format!(
                    "{limit} {:?} candles from {from} runs past the last instant a `Timestamp` \
                     can hold, around the year 2262",
                    request.interval
                ),
            )
        })?;

    Ok(Some(request.to.map_or(derived, |to| to.min(derived))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, Interval, Market};
    use std::cell::RefCell;

    const EXCHANGE: &str = "test";
    const MINUTE_MS: i64 = 60_000;

    fn market() -> Market {
        Market::spot(Exchange::Upbit, "BTC", "KRW")
    }

    fn candle(open_ms: i64) -> Candle {
        Candle {
            market: market(),
            interval: Interval::Min1,
            open_time: Timestamp::from_millis(open_ms),
            open: 1.into(),
            high: 1.into(),
            low: 1.into(),
            close: 1.into(),
            volume: 1.into(),
            quote_volume: None,
            closed: true,
        }
    }

    /// A minute-candle exchange holding a fixed history, answering the way a
    /// real one does: newest-first, capped, and reading `end` exclusively.
    struct Fake {
        /// Open times, in minutes since the epoch.
        history: Vec<i64>,
        /// Whether this exchange pages the way Hyperliquid does. Hyperliquid
        /// takes a window instead of a count and reads both of its ends
        /// inclusively, so a full page carries the `count` candles asked for
        /// *and* the one at `end`. Upbit and Bithumb take a count and read `to`
        /// exclusively.
        inclusive_end: bool,
        calls: RefCell<Vec<(Option<i64>, u32)>>,
    }

    impl Fake {
        fn new(minutes: std::ops::Range<i64>) -> Self {
            Self {
                history: minutes.collect(),
                inclusive_end: false,
                calls: RefCell::new(Vec::new()),
            }
        }

        fn inclusive(minutes: std::ops::Range<i64>) -> Self {
            Self {
                inclusive_end: true,
                ..Self::new(minutes)
            }
        }

        fn page(&self, end: Option<Timestamp>, count: u32) -> Result<Vec<Candle>> {
            self.calls
                .borrow_mut()
                .push((end.map(Timestamp::as_millis), count));

            let end_ms = end.map_or(i64::MAX, Timestamp::as_millis);
            let mut opens: Vec<i64> = self
                .history
                .iter()
                .map(|minute| minute * MINUTE_MS)
                .filter(|open| {
                    if self.inclusive_end {
                        *open <= end_ms
                    } else {
                        *open < end_ms
                    }
                })
                .collect();
            opens.reverse();
            // One more than asked for on an inclusive-end exchange: the
            // boundary at `end` plus the `count` intervals before it.
            opens.truncate(count as usize + usize::from(self.inclusive_end));

            Ok(opens.into_iter().map(candle).collect())
        }
    }

    async fn read_from(fake: &Fake, request: &CandleRequest, cap: u32) -> Result<Vec<i64>> {
        let candles = read(request, EXCHANGE, cap, |end, count| async move {
            fake.page(end, count)
        })
        .await?;

        Ok(candles
            .iter()
            .map(|candle| candle.open_time.as_millis() / MINUTE_MS)
            .collect())
    }

    fn request() -> CandleRequest {
        CandleRequest::new(market(), Interval::Min1)
    }

    fn minute(minute: i64) -> Timestamp {
        Timestamp::from_millis(minute * MINUTE_MS)
    }

    /// Midnight UTC on the first of a month.
    ///
    /// Built from the calendar rather than from
    /// [`Interval::advance`](crate::Interval::advance), so a monthly expectation
    /// below is not the code under test restating itself.
    fn month(year: i32, month: u32) -> Timestamp {
        let first = chrono::NaiveDate::from_ymd_opt(year, month, 1).expect("a first of the month");

        Timestamp::from_secs(first.and_time(chrono::NaiveTime::MIN).and_utc().timestamp())
    }

    /// A monthly-candle exchange, answering the way Upbit's `months` endpoint
    /// does: newest-first, capped, `to` read exclusively.
    struct Monthly {
        /// Month starts, oldest first: every month of 2000 through 2024.
        history: Vec<Timestamp>,
        calls: RefCell<usize>,
    }

    impl Monthly {
        fn new() -> Self {
            Self {
                history: (0..300)
                    .map(|n: i32| month(2000 + n / 12, u32::try_from(n % 12 + 1).expect("1..=12")))
                    .collect(),
                calls: RefCell::new(0),
            }
        }

        fn page(&self, end: Option<Timestamp>, count: u32) -> Result<Vec<Candle>> {
            *self.calls.borrow_mut() += 1;

            let end = end.unwrap_or(Timestamp::from_nanos(i64::MAX));
            let mut opens: Vec<Timestamp> = self
                .history
                .iter()
                .copied()
                .filter(|at| *at < end)
                .collect();
            opens.reverse();
            opens.truncate(count as usize);

            Ok(opens
                .into_iter()
                .map(|open_time| Candle {
                    interval: Interval::Month1,
                    open_time,
                    ..candle(0)
                })
                .collect())
        }
    }

    async fn read_months(fake: &Monthly, request: &CandleRequest) -> Result<Vec<Timestamp>> {
        let candles = read(request, EXCHANGE, 10, |end, count| async move {
            fake.page(end, count)
        })
        .await?;

        Ok(candles.iter().map(|candle| candle.open_time).collect())
    }

    /// A monthly-candle exchange whose months are 30 days long.
    ///
    /// Hyperliquid's `1M` grid, read live on 2026-07-30 from `candleSnapshot`
    /// for BTC over 2020-01-01 to 2025-01-01: 61 buckets, every one spanning
    /// exactly 30 days and opening at a whole multiple of 30 days after the
    /// Unix epoch. `2019-12-10T00:00:00Z` is bucket 608 and
    /// `2024-12-13T00:00:00Z` is bucket 669. No calendar month has 30 days
    /// twelve times a year, so this grid and
    /// [`Interval::advance`](crate::Interval::advance) count a window
    /// differently, and the walk has to answer for the grid rather than for the
    /// count.
    ///
    /// Answers the way an exchange does: newest-first, capped, and reading its
    /// end of range inclusively, as Hyperliquid does.
    struct ThirtyDay {
        buckets: Vec<i64>,
        calls: RefCell<usize>,
    }

    const THIRTY_DAYS_MS: i64 = 30 * 24 * 60 * 60 * 1_000;

    /// The open time of the `index`th 30-day bucket after the epoch.
    fn bucket(index: i64) -> Timestamp {
        Timestamp::from_millis(index * THIRTY_DAYS_MS)
    }

    impl ThirtyDay {
        fn new(indexes: std::ops::Range<i64>) -> Self {
            Self {
                buckets: indexes.map(|index| index * THIRTY_DAYS_MS).collect(),
                calls: RefCell::new(0),
            }
        }

        fn page(&self, end: Option<Timestamp>, count: u32) -> Result<Vec<Candle>> {
            *self.calls.borrow_mut() += 1;

            let end_ms = end.map_or(i64::MAX, Timestamp::as_millis);
            let mut opens: Vec<i64> = self
                .buckets
                .iter()
                .copied()
                .filter(|open| *open <= end_ms)
                .collect();
            opens.reverse();
            opens.truncate(count as usize + 1);

            Ok(opens
                .into_iter()
                .map(|open_ms| Candle {
                    interval: Interval::Month1,
                    open_time: Timestamp::from_millis(open_ms),
                    ..candle(0)
                })
                .collect())
        }
    }

    async fn read_thirty_day(fake: &ThirtyDay, request: &CandleRequest) -> Result<Vec<Timestamp>> {
        let candles = read(request, EXCHANGE, 10, |end, count| async move {
            fake.page(end, count)
        })
        .await?;

        Ok(candles.iter().map(|candle| candle.open_time).collect())
    }

    #[tokio::test]
    async fn a_window_on_a_grid_shorter_than_the_interval_answers_with_every_candle_it_holds() {
        // Eighty 30-day buckets is 2400 days, and 2400 days is 78 whole
        // calendar months and part of a 79th, so the page budget worked out
        // from the interval's nominal length says 79. The window holds 80, and
        // a `from` with no `limit` is the window, not the budget.
        let fake = ThirtyDay::new(0..200);
        let from = bucket(100);
        let request = CandleRequest::new(market(), Interval::Month1)
            .from(from)
            .to(bucket(180));

        let months = read_thirty_day(&fake, &request)
            .await
            .expect("a window inside the ceiling");

        // Stated from the grid rather than from the walk: buckets 100 through
        // 179, since `to` is exclusive.
        assert_eq!(
            months.len(),
            80,
            "the window holds 80 buckets and the answer dropped some"
        );
        assert_eq!(months.first().copied(), Some(from));
        assert_eq!(months.last().copied(), Some(bucket(179)));
    }

    #[tokio::test]
    async fn a_start_time_and_a_count_on_a_shorter_grid_answer_the_oldest_not_the_newest() {
        // Twelve calendar months on from a bucket open is 365 days, which spans
        // thirteen 30-day buckets. Walking back from there and stopping as soon
        // as twelve are in hand keeps buckets 101 to 112, the *newest* twelve of
        // that window, where `from` with `limit` promises the oldest twelve.
        let fake = ThirtyDay::new(0..200);
        let request = CandleRequest::new(market(), Interval::Month1)
            .from(bucket(100))
            .limit(12);

        let months = read_thirty_day(&fake, &request).await.expect("ok");

        assert_eq!(months.len(), 12);
        assert_eq!(
            months,
            (100..112).map(bucket).collect::<Vec<_>>(),
            "the oldest twelve at or after `from`, not the newest twelve before the cursor"
        );
    }

    #[tokio::test]
    async fn a_limit_within_one_page_costs_one_call_and_returns_the_newest_candles() {
        let fake = Fake::new(0..100);

        assert_eq!(
            read_from(&fake, &request().limit(3), 10).await.expect("ok"),
            vec![97, 98, 99]
        );
        assert_eq!(fake.calls.borrow().len(), 1);
    }

    #[tokio::test]
    async fn a_limit_above_the_cap_is_paged_rather_than_refused() {
        // Twenty-five candles from an exchange that serves ten at a time is
        // three calls, and the answer is still one unbroken run.
        let fake = Fake::new(0..100);

        assert_eq!(
            read_from(&fake, &request().limit(25), 10)
                .await
                .expect("ok"),
            (75..100).collect::<Vec<_>>()
        );
        assert_eq!(fake.calls.borrow().len(), 3);
    }

    #[tokio::test]
    async fn a_start_time_is_honoured_by_walking_back_to_it() {
        // The last forty minutes, from an exchange whose only cursor points
        // backwards. Nothing before that may appear. The history sits at the
        // present because `from` with no `to` means "up to now", and how far
        // back that reaches is what the walk is bounded by.
        let latest = Timestamp::now().as_secs() / 60;
        let fake = Fake::new(latest - 100..latest);

        assert_eq!(
            read_from(&fake, &request().from(minute(latest - 40)), 10)
                .await
                .expect("ok"),
            (latest - 40..latest).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn a_start_time_with_a_count_answers_from_the_start_not_from_the_present() {
        // "Five candles from minute 40" is 40..45, the beginning of a backfill,
        // not the tail of the history.
        let fake = Fake::new(0..100);

        assert_eq!(
            read_from(&fake, &request().from(minute(40)).limit(5), 10)
                .await
                .expect("ok"),
            vec![40, 41, 42, 43, 44]
        );
        // And it costs one call, because the window is known before asking.
        assert_eq!(fake.calls.borrow().len(), 1);
    }

    #[tokio::test]
    async fn an_end_time_is_exclusive_and_a_start_time_inclusive() {
        let fake = Fake::new(0..100);

        assert_eq!(
            read_from(&fake, &request().from(minute(40)).to(minute(43)), 10)
                .await
                .expect("ok"),
            vec![40, 41, 42]
        );
    }

    #[tokio::test]
    async fn a_history_shorter_than_the_request_ends_the_walk() {
        let fake = Fake::new(95..100);

        assert_eq!(
            read_from(&fake, &request().limit(50), 10)
                .await
                .expect("ok"),
            (95..100).collect::<Vec<_>>()
        );
        // Two calls: one full page of five is not full at ten, so the walk
        // stops rather than asking again for nothing.
        assert_eq!(fake.calls.borrow().len(), 1);
    }

    #[tokio::test]
    async fn a_cursor_that_stops_moving_is_reported_instead_of_looping() {
        // An exchange that ignores `end` repeats its newest page forever.
        let calls = RefCell::new(0u32);
        let stuck = read(&request().limit(30), EXCHANGE, 10, |_, _| {
            *calls.borrow_mut() += 1;
            async { Ok((0..10).map(|minute| candle(minute * MINUTE_MS)).collect()) }
        })
        .await;

        assert!(matches!(stuck, Err(Error::Exchange { .. })));
        assert!(*calls.borrow() < 5, "the walk did not stop");
    }

    #[tokio::test]
    async fn a_start_time_with_no_count_is_bounded_before_the_first_call() {
        // "Every one-minute candle since 1970" is twenty-nine million candles
        // and, at ten a call, three million sequential requests. The walk has
        // no way to shorten that, so it is refused where the caller can still
        // read the reason, rather than begun and abandoned by a rate limit.
        let fake = Fake::new(0..100);

        assert!(matches!(
            read_from(&fake, &request().from(Timestamp::from_secs(0)), 10).await,
            Err(Error::InvalidRequest { field: "from", .. })
        ));
        assert!(
            fake.calls.borrow().is_empty(),
            "a window that cannot be walked should not be started"
        );
    }

    #[tokio::test]
    async fn a_count_past_what_the_walk_can_reach_is_refused_with_the_number() {
        let fake = Fake::new(0..100);
        let ceiling = 10 * MAX_CALLS as usize;

        assert!(
            read_from(&fake, &request().limit(ceiling as u32), 10)
                .await
                .is_ok()
        );
        let Err(Error::InvalidRequest { field, detail }) =
            read_from(&fake, &request().limit(ceiling as u32 + 1), 10).await
        else {
            panic!("a count past the ceiling should be refused");
        };
        assert_eq!(field, "limit");
        assert!(
            detail.contains(&ceiling.to_string()),
            "the refusal should say how much can be read: {detail}"
        );
    }

    #[tokio::test]
    async fn the_first_candle_a_market_ever_had_ends_the_walk_rather_than_failing_it() {
        // An exchange whose end-of-range is inclusive answers the last page of
        // a history with only the candle the cursor names. Nothing moved, but
        // nothing is wrong either: that is where the history begins.
        let fake = Fake::inclusive(0..10);

        assert_eq!(
            read_from(&fake, &request().limit(25), 10)
                .await
                .expect("the end of a history is not an error"),
            (0..10).collect::<Vec<_>>()
        );
        // Two calls, so the second one is the page that stood still: the walk
        // asked again and read the answer as the history ending, rather than
        // stopping at the first page and never reaching the question.
        assert_eq!(fake.calls.borrow().len(), 2);
    }

    #[tokio::test]
    async fn a_start_time_with_a_count_answers_from_the_start_at_monthly_candles_too() {
        // The documented backfill shape, at the one interval with no fixed
        // length: five months from May 2000 is May through September, not the
        // newest five months the exchange holds. Deriving the walk's far end
        // from a length leaves it underived here, which puts the cursor at the
        // present and answers the tail of the history.
        let fake = Monthly::new();
        let request = CandleRequest::new(market(), Interval::Month1)
            .from(month(2000, 5))
            .limit(5);

        assert_eq!(
            read_months(&fake, &request).await.expect("ok"),
            vec![
                month(2000, 5),
                month(2000, 6),
                month(2000, 7),
                month(2000, 8),
                month(2000, 9)
            ]
        );
        assert_eq!(*fake.calls.borrow(), 1, "the window is known before asking");
    }

    #[tokio::test]
    async fn a_monthly_window_at_the_ceiling_is_measured_in_real_months() {
        // A thousand candles is exactly what ten a call over a hundred calls
        // reaches, and January 1950 to May 2033 is a thousand months. Measured as
        // a run of 28-day months it counts 1,088 and the request is refused for
        // being too wide, which it is not.
        let fake = Monthly::new();
        let request = CandleRequest::new(market(), Interval::Month1)
            .from(month(1950, 1))
            .to(month(2033, 5));

        let months = read_months(&fake, &request)
            .await
            .expect("a thousand monthly candles is inside the ceiling");
        // The window reaches further back than the exchange's history, so the
        // answer is the history: every month of 2000 through 2024.
        assert_eq!(months.len(), 300);
        assert_eq!(months.first().copied(), Some(month(2000, 1)));
        assert_eq!(months.last().copied(), Some(month(2024, 12)));
    }

    #[tokio::test]
    async fn a_page_that_re_serves_the_candle_at_the_cursor_is_not_a_stall() {
        // Twenty-five candles from the middle of a deep history on the exchange
        // whose window includes the boundary at `end`. Every page after the
        // first opens with the candle the cursor names, and that is the exchange
        // honouring the cursor rather than ignoring it.
        let fake = Fake::inclusive(0..100_000);

        assert_eq!(
            read_from(&fake, &request().limit(25), 10)
                .await
                .expect("a mid-history walk is not a stalled cursor"),
            (99_975..100_000).collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn no_request_costs_more_calls_than_the_ceiling() {
        // An exchange that answers with the candle the cursor names and `count`
        // minus one older ones yields one fewer new candle per page than it was
        // asked for, so a count the ceiling check allows takes 111 pages to
        // reach at a cap of ten. The promise on `read` is that nothing costs
        // more than `MAX_CALLS`, and a short answer presented as a whole one
        // would be the other way of keeping it.
        let calls = RefCell::new(0u32);
        let walk = read(
            &request().limit(10 * MAX_CALLS),
            EXCHANGE,
            10,
            |end, count| {
                *calls.borrow_mut() += 1;
                let newest = end.map_or(1_000_000, |end| end.as_millis() / MINUTE_MS);
                async move {
                    Ok((0..i64::from(count))
                        .map(|back| candle((newest - back) * MINUTE_MS))
                        .collect())
                }
            },
        )
        .await;

        assert!(matches!(walk, Err(Error::Exchange { .. })), "{walk:?}");
        assert_eq!(*calls.borrow(), MAX_CALLS);
    }

    #[tokio::test]
    async fn a_count_alone_answers_the_newest_even_when_a_page_carries_one_extra() {
        // `limit` alone is the newest that many. On an exchange that reads its
        // window inclusively the page carries `count + 1`, and cutting the run
        // from the front keeps the wrong end of it.
        //
        // Captured from Hyperliquid's `candleSnapshot` on 2026-07-30, a window
        // exactly one minute wide, `startTime` 2026-07-30T06:54:00Z and
        // `endTime` 2026-07-30T06:55:00Z. Two candles came back, opening at
        // 06:54:00Z and 06:55:00Z, which is the shape `Fake::inclusive` copies.
        let fake = Fake::inclusive(0..100);

        assert_eq!(
            read_from(&fake, &request().limit(1), 10).await.expect("ok"),
            vec![99],
            "one candle asked for with no start time is the newest one"
        );
    }

    #[tokio::test]
    async fn a_window_running_past_the_end_of_time_is_refused_not_answered_from_the_present() {
        // A `from` near the end of what a `Timestamp` holds puts the far end of
        // the window outside it. That end is where the walk starts, and having
        // no start means starting at the present, which answers a different
        // window than the one asked for and says nothing about the swap.
        let fake = Fake::new(0..100);
        let request = request().from(Timestamp::from_nanos(i64::MAX - 1)).limit(2);

        assert!(
            matches!(
                read_from(&fake, &request, 10).await,
                Err(Error::InvalidRequest { field: "from", .. })
            ),
            "a window that cannot be expressed should not be answered"
        );
        assert!(fake.calls.borrow().is_empty());
    }

    #[tokio::test]
    async fn a_request_for_no_candles_is_a_caller_mistake() {
        let fake = Fake::new(0..100);

        assert!(matches!(
            read_from(&fake, &request().limit(0), 10).await,
            Err(Error::InvalidRequest { field: "limit", .. })
        ));
        assert!(fake.calls.borrow().is_empty());
    }

    #[tokio::test]
    async fn a_window_that_ends_before_it_starts_never_reaches_the_wire() {
        let fake = Fake::new(0..100);

        assert!(matches!(
            read_from(&fake, &request().from(minute(50)).to(minute(50)), 10).await,
            Err(Error::InvalidRequest { field: "from", .. })
        ));
        assert!(fake.calls.borrow().is_empty());
    }

    #[tokio::test]
    async fn an_over_cap_count_of_monthly_candles_with_no_start_time_pages_like_any_other() {
        // Once refused, on the stated grounds that a month has no fixed length
        // and so names no window to page through. `Interval::advance` steps whole
        // calendar months, so the walk reaches back from the present here exactly
        // as it does at every other interval, and the refusal had no arithmetic
        // left behind it.
        let fake = Monthly::new();
        let over_cap = CandleRequest::new(market(), Interval::Month1).limit(25);

        let months = read_months(&fake, &over_cap)
            .await
            .expect("an unanchored monthly count pages");

        // Ten a page, so 25 is three calls, walking backwards from the present
        // into the newest history the exchange holds.
        assert_eq!(months.len(), 25);
        assert_eq!(months.last().copied(), Some(month(2024, 12)));
        assert_eq!(months.first().copied(), Some(month(2022, 12)));
        assert_eq!(*fake.calls.borrow(), 3);
    }
}
