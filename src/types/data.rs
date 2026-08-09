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
/// [`Trade::taker_side`] has the same meaning across exchanges.
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
    /// The aggressor's side.
    ///
    /// A `Buy` means the taker lifted an ask.
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderBook {
    /// The market it describes.
    pub market: Market,
    /// When the exchange says the snapshot was taken, or when `maxt` read it if
    /// the exchange does not say.
    ///
    /// Staleness measured from a read-time fallback is only a lower bound.
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

/// A provider ticker summary for one market.
///
/// Fields the exchange does not publish are `None`. A market that genuinely
/// traded zero volume is different from one whose exchange does not report
/// volume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ticker {
    /// The market it summarizes.
    pub market: Market,
    /// When the exchange produced the summary, or when `maxt` read it if the
    /// exchange does not say.
    ///
    /// Staleness measured from a read-time fallback is only a lower bound.
    pub timestamp: Timestamp,
    /// When the trade behind [`Ticker::last_price`] executed, if the primary
    /// price is a trade price.
    ///
    /// Distinct from [`Ticker::timestamp`], which is when the exchange built
    /// the summary. On a quiet market the two drift apart, and the gap between
    /// them is how stale the price is.
    ///
    /// `None` when the exchange does not report it or when the provider's
    /// primary ticker price is not a trade price.
    pub last_trade_time: Option<Timestamp>,
    /// The provider summary's primary price.
    ///
    /// This is usually the most recent trade price, but providers may use
    /// another reference price. Use the trades API for execution prices.
    pub last_price: Decimal,
    /// Signed change against the provider's reference price.
    pub change: Option<Decimal>,
    /// The same change as a signed ratio.
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
/// An interval specifies a length, not an exchange-independent opening grid.
/// Providers may anchor daily, weekly, monthly, and multi-day candles to
/// different time zones or epochs. Compare [`Candle::open_time`] across
/// exchanges only after accounting for each provider's grid.
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
    /// Ten minutes.
    Min10,
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
    /// Six hours.
    Hour6,
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
    /// [`Interval::Month1`] has no fixed length and returns `None`. Use
    /// [`Interval::advance`] to move by whole intervals including months.
    pub const fn as_secs(self) -> Option<u64> {
        Some(match self {
            Self::Sec1 => 1,
            Self::Min1 => 60,
            Self::Min3 => 180,
            Self::Min5 => 300,
            Self::Min10 => 600,
            Self::Min15 => 900,
            Self::Min30 => 1_800,
            Self::Hour1 => 3_600,
            Self::Hour2 => 7_200,
            Self::Hour4 => 14_400,
            Self::Hour6 => 21_600,
            Self::Hour8 => 28_800,
            Self::Hour12 => 43_200,
            Self::Day1 => 86_400,
            Self::Day3 => 259_200,
            Self::Week1 => 604_800,
            Self::Month1 => return None,
        })
    }

    /// `at` shifted by `count` intervals.
    ///
    /// A negative count moves backwards and zero returns `at`. Fixed intervals
    /// use their exact nanosecond length. [`Interval::Month1`] uses UTC calendar
    /// months and clamps a day absent from the target month to its last day.
    /// Because of that clamp, month movement is not generally reversible.
    ///
    /// This method does not apply a provider's candle grid. It returns `None`
    /// if the result cannot be represented by [`Timestamp`].
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

/// One open-high-low-close-volume (OHLCV) candle.
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
    /// A live window may be emitted repeatedly with `false`. Within one
    /// uninterrupted connection, a settled window is emitted at most once with
    /// `true`. Providers without a native close flag infer settlement when a
    /// later window arrives. Reconnection clears held state; a new snapshot may
    /// emit an already ended window again.
    ///
    /// REST candles are classified from the window end relative to the read
    /// time, so every candle in a historical range may be closed.
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
        assert_eq!(book.mid_price().unwrap(), Decimal::new(1005, 1));
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
        assert_eq!(Interval::Min10.as_secs(), Some(600));
        assert_eq!(Interval::Hour6.as_secs(), Some(21_600));
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
