//! Shared backwards pagination for candle history.
//!
//! Provider pages are normalized to oldest-first results. `from` is inclusive,
//! `to` is exclusive, and `from` with `limit` returns the oldest matching
//! candles. Provider-specific calendar grids may override interval advancement.

use std::future::Future;

use crate::error::{Error, Result};
use crate::request::CandleRequest;
use crate::types::{Candle, Timestamp};

/// Maximum provider calls made by one candle request.
///
/// Requests that exceed this sequential-page budget fail instead of returning
/// a partial result.
pub(crate) const MAX_CALLS: u32 = 100;

/// Reads candles oldest first through a backwards-page callback.
///
/// `fetch(end, count)` returns the newest candles before `end`, or before now
/// when `end` is `None`. Inclusive provider boundaries are accepted and
/// deduplicated. The walk fails if its cursor stalls or reaches [`MAX_CALLS`].
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
    let interval = request.interval;
    read_on_grid(
        request,
        exchange,
        max_per_call,
        move |at, count| interval.advance(at, count),
        fetch,
    )
    .await
}

/// Reads candles using a provider-specific calendar grid.
pub(crate) async fn read_on_grid<F, Fut, G>(
    request: &CandleRequest,
    exchange: &'static str,
    max_per_call: u32,
    advance: G,
    fetch: F,
) -> Result<Vec<Candle>>
where
    F: Fn(Option<Timestamp>, u32) -> Fut,
    Fut: Future<Output = Result<Vec<Candle>>>,
    G: Fn(Timestamp, i64) -> Option<Timestamp> + Copy,
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
        // Preflight the unbounded window before issuing provider calls.
        (None, Some(from)) => (window_candles(request, from, advance), "from"),
        // With no bounds, return one provider page.
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

    if let (Some(from), Some(limit)) = (request.from, request.limit) {
        if request.to.is_none() {
            advance(from, i64::from(limit)).ok_or_else(|| {
                Error::invalid_request(
                    "from",
                    format!(
                        "{limit} {:?} candles from {from} runs past the last instant a \
                         `Timestamp` can hold, around the year 2262",
                        request.interval
                    ),
                )
            })?;
        }
        return read_from_limit(
            exchange,
            max_per_call,
            from,
            request.to.unwrap_or_else(Timestamp::now),
            limit,
            advance,
            &fetch,
        )
        .await;
    }

    let mut cursor = request.to;
    let mut collected: Vec<Candle> = Vec::new();
    let mut previous_oldest: Option<Timestamp> = None;
    let mut calls: u32 = 0;

    loop {
        // A `from`-anchored walk must keep paging to the oldest boundary.
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
        // Inclusive page boundaries can reduce the number of new candles per call.
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

        // A repeated full page is a stalled cursor; a short page ends history.
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
            // Remove an inclusive boundary already returned by the prior page.
            prior.is_none_or(|prior| candle.open_time < prior)
                && request.from.is_none_or(|from| candle.open_time >= from)
                && request.to.is_none_or(|to| candle.open_time < to)
        });
        collected.extend(page);

        let short_page = page_len < count as usize;
        let reached_start = request.from.is_some_and(|from| oldest <= from);
        // For `from`, the estimated target is a page budget; the actual boundary
        // is the requested start time because provider grids may differ.
        let counted_out = request.from.is_none() && collected.len() >= target;
        if counted_out || short_page || reached_start {
            break;
        }
        cursor = Some(oldest);
    }

    // Normalize once after all backwards pages have been collected.
    collected.sort_by_key(|candle| candle.open_time);
    collected.dedup_by_key(|candle| candle.open_time);
    // A `from`-only target is an estimate and must not truncate the result.
    if request.from.is_none() {
        collected.drain(..collected.len().saturating_sub(target));
    }
    Ok(collected)
}

/// Reads the oldest `limit` candles at or after `from`.
///
/// Providers may omit empty windows. Each probe therefore expands its end
/// until it contains the next page of actual candles instead of assuming one
/// candle exists at every grid point.
async fn read_from_limit<F, Fut, G>(
    exchange: &'static str,
    max_per_call: u32,
    from: Timestamp,
    upper: Timestamp,
    limit: u32,
    advance: G,
    fetch: &F,
) -> Result<Vec<Candle>>
where
    F: Fn(Option<Timestamp>, u32) -> Fut,
    Fut: Future<Output = Result<Vec<Candle>>>,
    G: Fn(Timestamp, i64) -> Option<Timestamp> + Copy,
{
    if from >= upper {
        return Ok(Vec::new());
    }

    let target = limit as usize;
    let mut lower = from;
    let mut calls = 0;
    let mut collected = Vec::with_capacity(target);

    while lower < upper && collected.len() < target {
        let wanted = (target - collected.len()).min(max_per_call as usize) as u32;
        let mut probe = CandleProbe {
            exchange,
            lower,
            upper,
            target,
            collected: collected.len(),
            fetch,
            calls: &mut calls,
        };
        let chunk = oldest_chunk(&mut probe, wanted, max_per_call, advance).await?;
        let complete = chunk.len() == wanted as usize;
        let Some(last) = chunk.last().map(|candle| candle.open_time) else {
            break;
        };

        collected.extend(chunk);
        if !complete {
            break;
        }
        let Some(next) = last.as_nanos().checked_add(1) else {
            break;
        };
        lower = Timestamp::from_nanos(next);
    }

    collected.sort_by_key(|candle| candle.open_time);
    collected.dedup_by_key(|candle| candle.open_time);
    collected.truncate(target);
    Ok(collected)
}

struct CandleProbe<'a, F> {
    exchange: &'static str,
    lower: Timestamp,
    upper: Timestamp,
    target: usize,
    collected: usize,
    fetch: &'a F,
    calls: &'a mut u32,
}

/// Finds the earliest `wanted` candles in `[lower, upper)`.
async fn oldest_chunk<F, Fut, G>(
    probe: &mut CandleProbe<'_, F>,
    wanted: u32,
    max_per_call: u32,
    advance: G,
) -> Result<Vec<Candle>>
where
    F: Fn(Option<Timestamp>, u32) -> Fut,
    Fut: Future<Output = Result<Vec<Candle>>>,
    G: Fn(Timestamp, i64) -> Option<Timestamp> + Copy,
{
    let lower = probe.lower;
    let upper = probe.upper;
    let wanted_len = wanted as usize;
    let mut low_steps = 0_i64;
    let mut high_steps = i64::from(if wanted == max_per_call && wanted > 1 {
        wanted - 1
    } else {
        wanted
    });
    let mut high_page;

    if max_per_call == 1 {
        let page = probe.run(lower, 1).await?;
        if !page.is_empty() {
            return Ok(page);
        }
    }

    let request_count = wanted.saturating_add(1).min(max_per_call);

    loop {
        let end = bounded_advance(lower, high_steps, upper, advance)?;
        high_page = probe.run(end, request_count).await?;

        if high_page.len() >= wanted_len {
            if low_steps == 0 {
                high_page.truncate(wanted_len);
                return Ok(high_page);
            }
            break;
        }
        if end == upper {
            return Ok(high_page);
        }

        low_steps = high_steps;
        high_steps = high_steps.checked_mul(2).unwrap_or(i64::MAX);
    }

    while high_steps - low_steps > 1 {
        let middle = low_steps + (high_steps - low_steps) / 2;
        let end = bounded_advance(lower, middle, upper, advance)?;
        let page = probe.run(end, request_count).await?;

        if page.len() >= wanted_len {
            high_steps = middle;
            high_page = page;
        } else {
            low_steps = middle;
        }
    }

    high_page.truncate(wanted_len);
    Ok(high_page)
}

fn bounded_advance<G>(
    lower: Timestamp,
    steps: i64,
    upper: Timestamp,
    advance: G,
) -> Result<Timestamp>
where
    G: Fn(Timestamp, i64) -> Option<Timestamp>,
{
    let end = advance(lower, steps).map_or(upper, |end| end.min(upper));
    if end <= lower {
        return Err(Error::invalid_request(
            "interval",
            "the provider candle grid did not advance",
        ));
    }
    Ok(end)
}

impl<F> CandleProbe<'_, F> {
    async fn run<Fut>(&mut self, end: Timestamp, count: u32) -> Result<Vec<Candle>>
    where
        F: Fn(Option<Timestamp>, u32) -> Fut,
        Fut: Future<Output = Result<Vec<Candle>>>,
    {
        if *self.calls >= MAX_CALLS {
            return Err(Error::exchange(
                self.exchange,
                "candle_pagination_ceiling",
                format!(
                    "walked {MAX_CALLS} pages of {} candles and gathered {} of the {} asked for",
                    self.exchange, self.collected, self.target
                ),
            ));
        }
        *self.calls += 1;

        let mut page = (self.fetch)(Some(end), count).await?;
        page.retain(|candle| {
            candle.open_time >= self.lower
                && if end < self.upper {
                    candle.open_time <= end
                } else {
                    candle.open_time < end
                }
        });
        page.sort_by_key(|candle| candle.open_time);
        page.dedup_by_key(|candle| candle.open_time);
        Ok(page)
    }
}

/// Estimates the number of grid points in an unbounded `from` window.
///
/// The estimate is found by exponential search followed by binary search. It
/// limits provider calls but does not truncate results on a different grid.
fn window_candles<G>(request: &CandleRequest, from: Timestamp, advance: G) -> usize
where
    G: Fn(Timestamp, i64) -> Option<Timestamp>,
{
    let end = request.to.unwrap_or_else(Timestamp::now);
    let inside = |count: i64| advance(from, count).is_some_and(|at| at < end);

    if !inside(0) {
        // This covers a future `from` when `to` is omitted.
        return 0;
    }

    let mut outside: i64 = 1;
    while inside(outside) {
        let Some(doubled) = outside.checked_mul(2) else {
            return usize::MAX;
        };
        outside = doubled;
    }

    // `outside` is past the window and `outside / 2` remains inside it.
    let mut last_inside = outside / 2;
    while outside - last_inside > 1 {
        let midpoint = last_inside + (outside - last_inside) / 2;
        if inside(midpoint) {
            last_inside = midpoint;
        } else {
            outside = midpoint;
        }
    }

    // Include the candle at `from`.
    usize::try_from(last_inside + 1).unwrap_or(usize::MAX)
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

    /// Fixed newest-first candle history used by pagination tests.
    struct Fake {
        /// Open times in minutes since the epoch.
        history: Vec<i64>,
        /// Whether a page includes its end cursor.
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
            // Inclusive pages also return the candle at `end`.
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

    /// Midnight UTC on the first of a month, independent of interval logic.
    fn month(year: i32, month: u32) -> Timestamp {
        let first = chrono::NaiveDate::from_ymd_opt(year, month, 1).expect("a first of the month");

        Timestamp::from_secs(first.and_time(chrono::NaiveTime::MIN).and_utc().timestamp())
    }

    fn kst_month(year: i32, month_number: u32) -> Timestamp {
        Timestamp::from_secs(month(year, month_number).as_secs() - 9 * 3_600)
    }

    fn advance_kst_month(at: Timestamp, count: i64) -> Option<Timestamp> {
        let shifted = Timestamp::from_secs(at.as_secs().checked_add(9 * 3_600)?);
        Interval::Month1
            .advance(shifted, count)?
            .as_secs()
            .checked_sub(9 * 3_600)
            .map(Timestamp::from_secs)
    }

    /// Newest-first calendar-month history with an exclusive end cursor.
    struct Monthly {
        /// Month starts in ascending order.
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

    /// Newest-first 30-day buckets with an inclusive end cursor.
    struct ThirtyDay {
        buckets: Vec<i64>,
        calls: RefCell<usize>,
    }

    const THIRTY_DAYS_MS: i64 = 30 * 24 * 60 * 60 * 1_000;

    /// Opens the indexed 30-day bucket after the epoch.
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
        // The provider grid contains 80 buckets although the nominal estimate is 79.
        let fake = ThirtyDay::new(0..200);
        let from = bucket(100);
        let request = CandleRequest::new(market(), Interval::Month1)
            .from(from)
            .to(bucket(180));

        let months = read_thirty_day(&fake, &request)
            .await
            .expect("a window inside the ceiling");

        // Expected boundaries come directly from the provider grid.
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
        // The nominal 12-month window spans 13 provider buckets.
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
        let fake = Fake::new(0..100);

        assert_eq!(
            read_from(&fake, &request().from(minute(40)).limit(5), 10)
                .await
                .expect("ok"),
            vec![40, 41, 42, 43, 44]
        );
        assert_eq!(fake.calls.borrow().len(), 1);
    }

    #[tokio::test]
    async fn a_start_time_with_a_count_skips_missing_candle_windows() {
        let fake = Fake {
            history: vec![0, 2],
            inclusive_end: false,
            calls: RefCell::new(Vec::new()),
        };

        assert_eq!(
            read_from(&fake, &request().from(minute(0)).limit(2), 10)
                .await
                .expect("two candles across an empty window"),
            vec![0, 2]
        );
    }

    #[tokio::test]
    async fn an_inclusive_cursor_can_use_one_of_the_requested_slots() {
        let request = request().from(minute(0)).to(minute(2)).limit(2);
        let candles = read(&request, EXCHANGE, 2, |end, count| async move {
            let end = end.expect("a bounded probe").as_millis() / MINUTE_MS;
            let mut opens = [0, 1, 2]
                .into_iter()
                .filter(|open| *open <= end)
                .collect::<Vec<_>>();
            opens.reverse();
            opens.truncate(count as usize);
            Ok(opens
                .into_iter()
                .map(|open| candle(open * MINUTE_MS))
                .collect())
        })
        .await
        .expect("an inclusive provider page");

        assert_eq!(
            candles
                .into_iter()
                .map(|candle| candle.open_time.as_millis() / MINUTE_MS)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
    }

    #[tokio::test]
    async fn a_long_empty_span_is_searched_without_one_call_per_window() {
        let fake = Fake {
            history: vec![0, 10_000, 10_001],
            inclusive_end: false,
            calls: RefCell::new(Vec::new()),
        };

        assert_eq!(
            read_from(&fake, &request().from(minute(0)).limit(3), 10)
                .await
                .expect("three candles across a long empty span"),
            vec![0, 10_000, 10_001]
        );
        assert!(
            fake.calls.borrow().len() < 40,
            "the search should expand and bisect the empty span"
        );
    }

    #[tokio::test]
    async fn an_end_time_can_leave_a_from_limit_request_short() {
        let fake = Fake {
            history: vec![0, 2, 4],
            inclusive_end: false,
            calls: RefCell::new(Vec::new()),
        };

        assert_eq!(
            read_from(&fake, &request().from(minute(0)).to(minute(3)).limit(3), 10,)
                .await
                .expect("the bounded history"),
            vec![0, 2]
        );
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
        assert_eq!(fake.calls.borrow().len(), 1);
    }

    #[tokio::test]
    async fn a_cursor_that_stops_moving_is_reported_instead_of_looping() {
        // Ignoring the cursor would otherwise repeat the same page forever.
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
        // Reject an oversized sequential walk before the first provider call.
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
        // A short inclusive page at the cursor marks the start of history.
        let fake = Fake::inclusive(0..10);

        assert_eq!(
            read_from(&fake, &request().limit(25), 10)
                .await
                .expect("the end of a history is not an error"),
            (0..10).collect::<Vec<_>>()
        );
        assert_eq!(fake.calls.borrow().len(), 2);
    }

    #[tokio::test]
    async fn a_start_time_with_a_count_answers_from_the_start_at_monthly_candles_too() {
        // Calendar-month backfills start at `from`, just like fixed intervals.
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
    async fn a_provider_calendar_grid_sets_the_from_limit_window() {
        let history = [kst_month(2026, 3), kst_month(2026, 4)];
        let request = CandleRequest::new(market(), Interval::Month1)
            .from(Timestamp::from_secs(history[0].as_secs() + 1))
            .limit(1);

        let candles = read_on_grid(
            &request,
            EXCHANGE,
            10,
            advance_kst_month,
            |end: Option<Timestamp>, count: u32| {
                let history = &history;
                async move {
                    let end = end.unwrap_or(Timestamp::from_nanos(i64::MAX));
                    let mut opens: Vec<_> =
                        history.iter().copied().filter(|open| *open < end).collect();
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
            },
        )
        .await
        .expect("a KST monthly window");

        assert_eq!(
            candles
                .into_iter()
                .map(|candle| candle.open_time)
                .collect::<Vec<_>>(),
            vec![history[1]]
        );
    }

    #[tokio::test]
    async fn a_monthly_window_at_the_ceiling_is_measured_in_real_months() {
        // Calendar-month counting keeps this window at the exact call ceiling.
        let fake = Monthly::new();
        let request = CandleRequest::new(market(), Interval::Month1)
            .from(month(1950, 1))
            .to(month(2033, 5));

        let months = read_months(&fake, &request)
            .await
            .expect("a thousand monthly candles is inside the ceiling");
        assert_eq!(months.len(), 300);
        assert_eq!(months.first().copied(), Some(month(2000, 1)));
        assert_eq!(months.last().copied(), Some(month(2024, 12)));
    }

    #[tokio::test]
    async fn a_page_that_re_serves_the_candle_at_the_cursor_is_not_a_stall() {
        // Repeated inclusive boundaries are deduplicated rather than treated as stalls.
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
        // Inclusive boundaries can exhaust the runtime call budget before the estimate.
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
        // `limit` alone trims an inclusive page from its oldest end.
        let fake = Fake::inclusive(0..100);

        assert_eq!(
            read_from(&fake, &request().limit(1), 10).await.expect("ok"),
            vec![99],
            "one candle asked for with no start time is the newest one"
        );
    }

    #[tokio::test]
    async fn a_window_running_past_the_end_of_time_is_refused_not_answered_from_the_present() {
        // Overflow while deriving the end cursor must not fall back to now.
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
    async fn an_explicit_end_can_bound_a_limit_that_would_otherwise_overflow() {
        let fake = Fake::new(0..100);
        let from = Timestamp::from_nanos(i64::MAX - 120_000_000_000);
        let to = Timestamp::from_nanos(i64::MAX - 60_000_000_000);
        let request = request().from(from).to(to).limit(10);

        assert!(
            read_from(&fake, &request, 10).await.is_ok(),
            "the explicit end keeps the requested window representable"
        );
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
        let fake = Monthly::new();
        let over_cap = CandleRequest::new(market(), Interval::Month1).limit(25);

        let months = read_months(&fake, &over_cap)
            .await
            .expect("an unanchored monthly count pages");

        assert_eq!(months.len(), 25);
        assert_eq!(months.last().copied(), Some(month(2024, 12)));
        assert_eq!(months.first().copied(), Some(month(2022, 12)));
        assert_eq!(*fake.calls.borrow(), 3);
    }
}
