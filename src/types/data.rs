//! Public market data: trades, order books, tickers, candles.

use rust_decimal::Decimal;

use crate::types::{Market, Timestamp};

/// Which side of the book a trade or an order sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// Buying the base asset.
    Buy,
    /// Selling the base asset.
    Sell,
}

impl Side {
    /// The opposite side.
    pub const fn flip(self) -> Self {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

/// A single executed trade.
///
/// [`Trade::taker_side`] is the aggressor's side on every supported exchange,
/// so a run of trades from two of them is comparable.
///
/// ```
/// use maxt::{Exchange, Market, Side, Timestamp, Trade};
/// use rust_decimal::Decimal;
///
/// /// Buy volume minus sell volume: positive means buyers lifted more.
/// fn signed_volume(trades: &[Trade]) -> Decimal {
///     trades
///         .iter()
///         .map(|trade| match trade.taker_side {
///             Side::Buy => trade.quantity,
///             Side::Sell => -trade.quantity,
///         })
///         .sum()
/// }
///
/// # let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
/// // Two trades as `Client::trades` would have reported them.
/// # let trade = |taker_side, quantity| Trade {
/// #     market: market.clone(),
/// #     timestamp: Timestamp::from_millis(1_700_000_000_000),
/// #     price: Decimal::from(100_000_000),
/// #     quantity: Decimal::from(quantity),
/// #     taker_side,
/// #     id: None,
/// # };
/// let trades = [trade(Side::Buy, 3), trade(Side::Sell, 1)];
///
/// assert_eq!(signed_volume(&trades), Decimal::from(2));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trade {
    /// The market it executed on.
    pub market: Market,
    /// When the exchange says it executed.
    pub timestamp: Timestamp,
    /// Execution price, in the quote asset.
    pub price: Decimal,
    /// Executed quantity, in the base asset.
    pub quantity: Decimal,
    /// Which side the *taker* was on.
    ///
    /// A `Buy` means the taker lifted an ask. This is the convention every
    /// supported exchange uses, so it is comparable across all of them.
    pub taker_side: Side,
    /// The exchange's own trade identifier, when it publishes one.
    pub id: Option<String>,
}

/// One price level in an order book.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level {
    /// The price of the level, in the quote asset.
    pub price: Decimal,
    /// Total resting quantity at that price, in the base asset.
    pub quantity: Decimal,
}

/// An order book snapshot.
///
/// [`OrderBook::bids`] is sorted best-first (descending price) and
/// [`OrderBook::asks`] is sorted best-first (ascending price), on every
/// exchange. Adapters re-sort where the exchange does not.
///
/// ```
/// use maxt::{Exchange, Level, Market, OrderBook, Timestamp};
/// use rust_decimal::Decimal;
///
/// let level = |price: i64, quantity: i64| Level {
///     price: Decimal::from(price),
///     quantity: Decimal::from(quantity),
/// };
/// let book = OrderBook {
///     market: Market::spot(Exchange::Upbit, "BTC", "KRW"),
///     timestamp: Timestamp::from_millis(1_700_000_000_000),
///     bids: vec![level(100, 2), level(99, 5)],
///     asks: vec![level(102, 1), level(103, 4)],
/// };
///
/// // Best-first on both sides, so the top of book is the front of each.
/// assert_eq!(book.best_bid().map(|level| level.price), Some(Decimal::from(100)));
/// assert_eq!(book.spread(), Some(Decimal::from(2)));
/// assert_eq!(book.mid_price(), Some(Decimal::from(101)));
///
/// // How much of the ask side a market buy would eat, in quote terms.
/// let cost: Decimal = book.asks.iter().take(2).map(|level| level.price * level.quantity).sum();
/// assert_eq!(cost, Decimal::from(514));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBook {
    /// The market it describes.
    pub market: Market,
    /// When the exchange says the snapshot was taken, or when `maxt` read it if
    /// the exchange does not say.
    ///
    /// Binance publishes no clock on a spot depth response, over REST or on
    /// the stream, so a spot book from there carries the read time. Treat this
    /// as an upper bound on the snapshot's age. Measuring staleness against it
    /// will under-report on those books.
    pub timestamp: Timestamp,
    /// Buy side, best (highest) price first.
    pub bids: Vec<Level>,
    /// Sell side, best (lowest) price first.
    pub asks: Vec<Level>,
}

impl OrderBook {
    /// The highest bid, or `None` if nobody is bidding.
    pub fn best_bid(&self) -> Option<&Level> {
        self.bids.first()
    }

    /// The lowest ask, or `None` if nobody is offering.
    pub fn best_ask(&self) -> Option<&Level> {
        self.asks.first()
    }

    /// Best ask minus best bid, or `None` if either side is empty.
    ///
    /// A crossed book yields a negative spread. `maxt` reports what the
    /// exchange sent.
    pub fn spread(&self) -> Option<Decimal> {
        Some(self.best_ask()?.price - self.best_bid()?.price)
    }

    /// The midpoint between best bid and best ask.
    pub fn mid_price(&self) -> Option<Decimal> {
        let bid = self.best_bid()?.price;
        let ask = self.best_ask()?.price;
        Some((bid + ask) / Decimal::TWO)
    }
}

/// A rolling 24-hour summary of one market.
///
/// Fields the exchange does not publish are `None`. A market that genuinely
/// traded zero volume is different from one whose exchange does not report
/// volume.
///
/// ```
/// use maxt::{Ticker, Timestamp};
///
/// /// How long ago the price traded, when the exchange says at all.
/// ///
/// /// `timestamp` is not a substitute: on the exchanges that publish no clock
/// /// it is `maxt`'s read time, which would report a week-old price as fresh.
/// fn price_age_nanos(ticker: &Ticker, now: Timestamp) -> Option<i64> {
///     Some(now.as_nanos() - ticker.last_trade_time?.as_nanos())
/// }
///
/// // `timed` reports when its price traded; `blank` comes from an exchange
/// // that never says, and leaves the field `None`.
/// # use maxt::{Exchange, Market};
/// # use rust_decimal::Decimal;
/// # let blank = Ticker {
/// #     market: Market::perpetual(Exchange::Hyperliquid, "BTC", "USDC"),
/// #     timestamp: Timestamp::from_secs(1_700_000_060),
/// #     last_trade_time: None,
/// #     last_price: Decimal::from(60_000),
/// #     change: None,
/// #     change_rate: None,
/// #     high: None,
/// #     low: None,
/// #     volume: None,
/// #     quote_volume: None,
/// # };
/// # let timed = Ticker { last_trade_time: Some(Timestamp::from_secs(1_700_000_000)), ..blank.clone() };
/// let now = Timestamp::from_secs(1_700_000_060);
///
/// assert_eq!(price_age_nanos(&timed, now), Some(60_000_000_000));
/// // Unknown, and reported as unknown.
/// assert_eq!(price_age_nanos(&blank, now), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticker {
    /// The market it summarizes.
    pub market: Market,
    /// When the exchange produced the summary, or when `maxt` read it if the
    /// exchange does not say.
    ///
    /// Hyperliquid publishes no clock with its asset contexts, so a ticker
    /// from there carries the read time. Treat this as an upper bound on the
    /// summary's age.
    pub timestamp: Timestamp,
    /// When the trade behind [`Ticker::last_price`] executed.
    ///
    /// Distinct from [`Ticker::timestamp`], which is when the exchange built
    /// the summary. On a quiet market the two drift apart, and the gap between
    /// them is how stale the price is.
    ///
    /// `None` when the exchange does not report it. Binance and Hyperliquid
    /// both publish a last price without saying when it traded. `maxt` leaves
    /// the field empty, because filling it from the read time would claim a
    /// freshness the exchange never stated.
    pub last_trade_time: Option<Timestamp>,
    /// The most recent trade price.
    pub last_price: Decimal,
    /// Change against the previous session's close, signed.
    pub change: Option<Decimal>,
    /// Change against the previous session's close, as a signed ratio.
    ///
    /// `0.05` means five percent up.
    pub change_rate: Option<Decimal>,
    /// Highest trade price over the window.
    pub high: Option<Decimal>,
    /// Lowest trade price over the window.
    pub low: Option<Decimal>,
    /// Traded volume over the window, in the base asset.
    pub volume: Option<Decimal>,
    /// Traded value over the window, in the quote asset.
    pub quote_volume: Option<Decimal>,
}

/// A candle interval.
///
/// Not every exchange serves every interval; an unsupported one is reported as
/// [`Error::Unsupported`](crate::Error::Unsupported) rather than silently
/// rounded to a neighbour.
///
/// # An interval names a length, not a grid
///
/// Where a candle of that length opens is the exchange's own decision, and the
/// four do not agree. Read live on 2026-07-30 through
/// [`Client::candles`](crate::Client::candles) for BTC on each, at every
/// interval each one serves:
///
/// | Interval | Upbit | Bithumb | Binance | Hyperliquid |
/// | --- | --- | --- | --- | --- |
/// | `Sec1` | whole UTC second | not served | whole UTC second | not served |
/// | `Min1` to `Min30` | whole UTC minute, on the interval's own grid from the hour | same | same | same |
/// | `Hour1` | whole UTC hour | whole UTC hour | whole UTC hour | whole UTC hour |
/// | `Hour2` | not served | not served | 00:00, 02:00, ... UTC | 00:00, 02:00, ... UTC |
/// | `Hour4` | 00:00, 04:00, ... UTC | **03:00, 07:00, 11:00, 15:00, 19:00, 23:00 UTC** | 00:00, 04:00, ... UTC | 00:00, 04:00, ... UTC |
/// | `Hour8` | not served | not served | 00:00, 08:00, 16:00 UTC | 00:00, 08:00, 16:00 UTC |
/// | `Hour12` | not served | not served | 00:00, 12:00 UTC | 00:00, 12:00 UTC |
/// | `Day1` | 00:00 UTC | **15:00 UTC** | 00:00 UTC | 00:00 UTC |
/// | `Day3` | not served | not served | **a 3-day grid at 00:00 UTC, one day ahead of Hyperliquid's** | **a 3-day grid at 00:00 UTC, measured from the epoch** |
/// | `Week1` | Monday 00:00 UTC | **Sunday 15:00 UTC** | Monday 00:00 UTC | **Thursday 00:00 UTC** |
/// | `Month1` | the 1st, 00:00 UTC | **the last day of the previous UTC month, 15:00 UTC** | the 1st, 00:00 UTC | **a 30-day grid measured from the epoch** |
///
/// Two things explain every bold cell:
///
/// | Exchange | Why its grid differs |
/// | --- | --- |
/// | Bithumb | it cuts every window in Korean time, nine hours ahead of UTC, so it matches the other three at every interval that divides nine hours and misses at every interval that does not |
/// | Hyperliquid | it measures `Day3`, `Week1` and `Month1` from the Unix epoch rather than from the calendar, and 1 January 1970 was a Thursday |
///
/// Binance's `Day3` is a 3-day grid too, one day ahead of Hyperliquid's:
/// counting whole days from the epoch, Binance opens where that count leaves
/// remainder 1 on division by three and Hyperliquid where it leaves 0. Read
/// live from 2025-01-01: Binance answers 2025-01-01, 01-04, 01-07 and
/// Hyperliquid 2024-12-31, 01-03, 01-06.
///
/// So a [`Candle::open_time`] is comparable across two exchanges only where the
/// row above says the same thing about both. Joining Upbit and Bithumb daily
/// candles on `open_time` matches nothing at all, and joining them by position
/// lines up windows nine hours apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Interval {
    /// One second.
    Sec1,
    /// One minute.
    Min1,
    /// Three minutes.
    Min3,
    /// Five minutes.
    Min5,
    /// Fifteen minutes.
    Min15,
    /// Thirty minutes.
    Min30,
    /// One hour.
    Hour1,
    /// Two hours.
    Hour2,
    /// Four hours.
    Hour4,
    /// Eight hours.
    Hour8,
    /// Twelve hours.
    Hour12,
    /// One day.
    Day1,
    /// Three days.
    Day3,
    /// One week.
    Week1,
    /// One month.
    Month1,
}

impl Interval {
    /// The interval's length in seconds.
    ///
    /// [`Interval::Month1`] has no fixed length and returns `None`. Moving a
    /// candle's open time by whole intervals is [`Interval::advance`], which
    /// answers for every interval including that one; a caller that reaches for
    /// this and then invents a length for a month has to guess, and every guess
    /// disagrees with the calendar somewhere.
    pub const fn as_secs(self) -> Option<u64> {
        Some(match self {
            Self::Sec1 => 1,
            Self::Min1 => 60,
            Self::Min3 => 180,
            Self::Min5 => 300,
            Self::Min15 => 900,
            Self::Min30 => 1_800,
            Self::Hour1 => 3_600,
            Self::Hour2 => 7_200,
            Self::Hour4 => 14_400,
            Self::Hour8 => 28_800,
            Self::Hour12 => 43_200,
            Self::Day1 => 86_400,
            Self::Day3 => 259_200,
            Self::Week1 => 604_800,
            Self::Month1 => return None,
        })
    }

    /// The open time `count` intervals along from the candle opening at `at`.
    ///
    /// A negative `count` walks backwards, and `0` gives `at` back. This is
    /// where the calendar lives: [`Interval::Month1`] steps whole months, so one
    /// step from 1 February is 1 March whether that is 28 days later or 29, and
    /// every other interval steps its own fixed length. Everything a caller
    /// might want a month's length *for*, where a candle's window ends, how
    /// wide a `count`-candle window is, where a backwards walk should start, is
    /// this one question, asked with a different `count`.
    ///
    /// The month it steps is a **UTC** calendar month, and a step that would
    /// land on a day the target month does not have is pulled back to its last
    /// day: one month on from 31 January is 28 February.
    ///
    /// This steps the length; it does not know any exchange's grid. Which
    /// exchange's own candle opens it lands on was read off their candles
    /// rather than assumed, and [`Interval`] carries the whole table. At
    /// [`Interval::Month1`], the interval where the step is not a fixed length:
    ///
    /// | Exchange | Where a monthly candle opens | UTC month steps between them |
    /// | --- | --- | --- |
    /// | Binance | midnight UTC on the 1st | yes |
    /// | Upbit | midnight UTC on the 1st | yes |
    /// | Bithumb | midnight KST on the 1st, which is 15:00 UTC on the last day of the previous UTC month | no |
    /// | Hyperliquid | a 30-day bucket measured from the epoch, not a calendar month at all | no |
    ///
    /// So a caller holding a Bithumb or Hyperliquid open time cannot reach the
    /// next one from here, and neither can `maxt`. Bithumb shifts into Korean
    /// time before stepping, because a UTC step from 28 February lands on 28
    /// March and its next candle opens on 31 March; Hyperliquid's own adapter
    /// measures a monthly window in 30-day buckets instead of calling this.
    /// [`Candle::closed`] on Bithumb is read off a clock and on Hyperliquid off
    /// the frame that opens the next window, so neither needs this either.
    ///
    /// `None` when the result leaves the range [`Timestamp`] can hold, roughly
    /// 1677 to 2262. Nothing here wraps or saturates, so a caller never gets a
    /// silently wrong instant back.
    ///
    /// ```
    /// use maxt::{Interval, Timestamp};
    ///
    /// // 1 February 2024, in a leap year.
    /// let february = Timestamp::from_secs(1_706_745_600);
    ///
    /// // A month is as long as the month actually is: 29 days, not 30 or 31.
    /// let march = Interval::Month1.advance(february, 1).expect("a month later");
    /// assert_eq!(march.as_secs() - february.as_secs(), 29 * 86_400);
    ///
    /// // And it is reversible, which is what a backwards page walk needs.
    /// assert_eq!(Interval::Month1.advance(march, -1), Some(february));
    ///
    /// // The clamp, and why an open time in a zone ahead of UTC needs its own
    /// // arithmetic: 2026-02-28T15:00Z is the first of March in Korea, and a
    /// // UTC month on lands three days before the first of April there.
    /// let korean_march = Timestamp::from_secs(1_772_290_800);
    /// assert_eq!(
    ///     Interval::Month1.advance(korean_march, 1),
    ///     Some(Timestamp::from_secs(1_774_710_000)) // 2026-03-28T15:00Z
    /// );
    ///
    /// // Fixed intervals are the same question with a known answer.
    /// assert_eq!(
    ///     Interval::Hour1.advance(february, 3),
    ///     Some(Timestamp::from_secs(1_706_745_600 + 3 * 3_600))
    /// );
    /// ```
    pub fn advance(self, at: Timestamp, count: i64) -> Option<Timestamp> {
        if let Some(secs) = self.as_secs() {
            let span = i64::try_from(secs)
                .ok()?
                .checked_mul(1_000_000_000)?
                .checked_mul(count)?;
            return at.as_nanos().checked_add(span).map(Timestamp::from_nanos);
        }

        // Only `Month1` reaches here, and only a calendar knows how long the
        // month starting at `at` is.
        let months = chrono::Months::new(u32::try_from(count.unsigned_abs()).ok()?);
        let at = chrono::DateTime::from_timestamp_nanos(at.as_nanos());
        let moved = if count < 0 {
            at.checked_sub_months(months)?
        } else {
            at.checked_add_months(months)?
        };

        moved.timestamp_nanos_opt().map(Timestamp::from_nanos)
    }
}

/// One OHLCV candle.
///
/// ```
/// use maxt::{Candle, Interval, Timestamp};
/// use rust_decimal::Decimal;
///
/// /// The close of the last *finished* interval.
/// ///
/// /// An open candle is republished every time a trade moves it, so an
/// /// indicator fed the unfiltered run would recompute itself on a price that
/// /// is still changing.
/// fn last_settled_close(candles: &[Candle]) -> Option<Decimal> {
///     candles.iter().rev().find(|candle| candle.closed).map(|candle| candle.close)
/// }
///
/// // A closed minute followed by the one still forming.
/// # use maxt::{Exchange, Market};
/// # let market = Market::spot(Exchange::Upbit, "BTC", "KRW");
/// # let candle = |minute: i64, close: i64, closed| Candle {
/// #     market: market.clone(),
/// #     interval: Interval::Min1,
/// #     open_time: Timestamp::from_secs(minute * 60),
/// #     open: Decimal::from(close),
/// #     high: Decimal::from(close),
/// #     low: Decimal::from(close),
/// #     close: Decimal::from(close),
/// #     volume: Decimal::ONE,
/// #     quote_volume: None,
/// #     closed,
/// # };
/// // Oldest first, which is the order every `candles` call returns.
/// let run = [candle(28_333_333, 100, true), candle(28_333_334, 101, false)];
///
/// assert_eq!(last_settled_close(&run), Some(Decimal::from(100)));
/// // `open_time` is the start of the interval, so this candle covers the
/// // minute that begins here, not the one that ends here.
/// assert_eq!(run[0].open_time.as_secs() % 60, 0);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candle {
    /// The market it aggregates.
    pub market: Market,
    /// The interval it covers.
    pub interval: Interval,
    /// The start of the interval, not its end.
    ///
    /// A one-minute candle for 12:34 opens at 12:34:00.000000000.
    pub open_time: Timestamp,
    /// First trade price in the interval.
    pub open: Decimal,
    /// Highest trade price in the interval.
    pub high: Decimal,
    /// Lowest trade price in the interval.
    pub low: Decimal,
    /// Last trade price in the interval.
    pub close: Decimal,
    /// Traded volume over the interval, in the base asset.
    pub volume: Decimal,
    /// Traded value over the interval, in the quote asset.
    pub quote_volume: Option<Decimal>,
    /// Whether the interval has closed.
    ///
    /// A streamed window arrives repeatedly while it is still open, each frame
    /// superseding the last, and gets **exactly one** emission with this set,
    /// carrying that window's own final figures. Nothing after it belongs to
    /// that window. A read over REST answers for the window as it stood when the
    /// exchange was asked, so every candle but the newest is closed and the
    /// newest one usually is not.
    ///
    /// What each exchange's stream is asked, since only one of them states it
    /// outright:
    ///
    /// | Exchange | How a live window is known to have closed |
    /// | --- | --- |
    /// | Binance | its own `x` flag on the frame |
    /// | Upbit | the arrival of a frame opening a later window. No frame of a window is ever stamped at or past that window's end |
    /// | Hyperliquid | the arrival of a frame opening a later window. It stops publishing a window a couple of seconds before that window's close time |
    /// | Bithumb | there is no candle stream |
    ///
    /// So on Upbit and Hyperliquid the settled emission arrives *after* its
    /// window ended, one frame late, and a window that never sees a successor
    /// never settles. A reconnect drops whatever was being held rather than
    /// settling it across a gap of unknown length, so the window a
    /// [`MarketEvent::Reconnected`](crate::MarketEvent::Reconnected) interrupts
    /// gets no settled emission at all.
    pub closed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Exchange;

    fn level(price: i64, quantity: i64) -> Level {
        Level {
            price: Decimal::from(price),
            quantity: Decimal::from(quantity),
        }
    }

    fn book(bids: Vec<Level>, asks: Vec<Level>) -> OrderBook {
        OrderBook {
            market: Market::spot(Exchange::Upbit, "BTC", "KRW"),
            timestamp: Timestamp::from_millis(1_700_000_000_000),
            bids,
            asks,
        }
    }

    #[test]
    fn best_prices_come_off_the_front_of_each_side() {
        let book = book(
            vec![level(100, 1), level(99, 2)],
            vec![level(101, 1), level(102, 2)],
        );

        assert_eq!(book.best_bid().unwrap().price, Decimal::from(100));
        assert_eq!(book.best_ask().unwrap().price, Decimal::from(101));
        assert_eq!(book.spread().unwrap(), Decimal::from(1));
        assert_eq!(
            book.mid_price().unwrap(),
            // Written as unscaled digits and a scale rather than 100.5: this
            // crate's whole claim about numbers is that a price never passes
            // through a binary float, and a test that reaches for one to state
            // its expectation undercuts the claim it is checking.
            Decimal::new(1005, 1)
        );
    }

    #[test]
    fn a_one_sided_book_has_no_spread_and_no_mid() {
        let bids_only = book(vec![level(100, 1)], vec![]);
        let asks_only = book(vec![], vec![level(101, 1)]);
        let empty = book(vec![], vec![]);

        for book in [&bids_only, &asks_only, &empty] {
            assert!(book.spread().is_none());
            assert!(book.mid_price().is_none());
        }
    }

    #[test]
    fn a_crossed_book_is_reported_rather_than_hidden() {
        let crossed = book(vec![level(102, 1)], vec![level(101, 1)]);

        assert_eq!(crossed.spread().unwrap(), Decimal::from(-1));
    }

    #[test]
    fn month_is_the_only_interval_without_a_fixed_length() {
        assert_eq!(Interval::Min1.as_secs(), Some(60));
        assert_eq!(Interval::Week1.as_secs(), Some(604_800));
        assert_eq!(Interval::Month1.as_secs(), None);
    }

    /// Midnight UTC on the first of a month, from its year and month number.
    fn month_start(year: i32, month: u32) -> Timestamp {
        let date = chrono::NaiveDate::from_ymd_opt(year, month, 1).expect("a first of the month");

        Timestamp::from_secs(date.and_time(chrono::NaiveTime::MIN).and_utc().timestamp())
    }

    #[test]
    fn a_month_advances_by_the_calendar_and_not_by_a_fixed_length() {
        // Every month of 2024, walked one step at a time. A fixed length gets
        // February wrong in one direction and the 31-day months in the other,
        // and after twelve steps it has drifted off the year entirely.
        let mut at = month_start(2024, 1);
        for month in 2..=12 {
            at = Interval::Month1.advance(at, 1).expect("the next month");
            assert_eq!(at, month_start(2024, month), "month {month}");
        }
        assert_eq!(
            Interval::Month1.advance(at, 1),
            Some(month_start(2025, 1)),
            "the twelfth step should cross the year"
        );

        // February 2024 is 29 days and February 2023 is 28, which is the whole
        // reason this cannot be a constant.
        for (year, days) in [(2024, 29), (2023, 28)] {
            let february = month_start(year, 2);
            let march = Interval::Month1.advance(february, 1).expect("March");
            assert_eq!(march.as_secs() - february.as_secs(), days * 86_400);
        }
    }

    #[test]
    fn a_negative_count_walks_back_the_same_way_it_walked_forward() {
        let january = month_start(2024, 1);

        // Fourteen months forward from January 2024 is March 2025, and the same
        // step back lands where it started. A backwards page walk depends on it.
        let later = Interval::Month1.advance(january, 14).expect("14 months on");
        assert_eq!(later, month_start(2025, 3));
        assert_eq!(Interval::Month1.advance(later, -14), Some(january));
        assert_eq!(Interval::Month1.advance(january, 0), Some(january));
    }

    #[test]
    fn a_fixed_interval_advances_without_losing_sub_second_precision() {
        let at = Timestamp::from_nanos(1_700_000_000_123_456_789);

        assert_eq!(
            Interval::Min1.advance(at, 2),
            Some(Timestamp::from_nanos(1_700_000_120_123_456_789))
        );
        assert_eq!(
            Interval::Week1.advance(at, -1),
            Some(Timestamp::from_nanos(
                1_700_000_000_123_456_789 - 604_800_000_000_000
            ))
        );
    }

    #[test]
    fn a_step_past_the_representable_range_is_reported_rather_than_wrapped() {
        // The year 2262 is where nanoseconds since the epoch run out. Both
        // branches answer `None` instead of handing back an earlier instant.
        let late = Timestamp::from_secs(9_220_000_000);
        assert_eq!(
            late.as_secs(),
            9_220_000_000,
            "the starting point itself fits"
        );

        assert_eq!(Interval::Month1.advance(late, 12), None);
        assert_eq!(Interval::Week1.advance(late, i64::MAX), None);
    }

    #[test]
    fn sides_flip_symmetrically() {
        assert_eq!(Side::Buy.flip(), Side::Sell);
        assert_eq!(Side::Buy.flip().flip(), Side::Buy);
    }
}
