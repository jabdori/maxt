//! Timestamps.

use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A point in time, as nanoseconds since the Unix epoch, UTC.
///
/// Exchanges publish timestamps in seconds, milliseconds, microseconds, and
/// nanoseconds. `maxt` converts all of them to nanoseconds, so events from two
/// exchanges can be ordered against each other without knowing which exchange
/// they came from.
///
/// This is not a `chrono` or `time` type, so no date-time library is forced on
/// callers. Convert at the edge with [`Timestamp::as_nanos`] or
/// [`Timestamp::into_system_time`].
///
/// ```
/// use maxt::Timestamp;
///
/// let ts = Timestamp::from_millis(1_700_000_000_000);
/// assert_eq!(ts.as_nanos(), 1_700_000_000_000_000_000);
/// assert_eq!(ts.as_millis(), 1_700_000_000_000);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Timestamp(i64);

impl Timestamp {
    /// From nanoseconds since the Unix epoch.
    pub const fn from_nanos(nanos: i64) -> Self {
        Self(nanos)
    }

    /// From microseconds since the Unix epoch.
    ///
    /// Saturates on overflow; see [`Timestamp::from_secs`].
    pub const fn from_micros(micros: i64) -> Self {
        Self(micros.saturating_mul(1_000))
    }

    /// From milliseconds since the Unix epoch.
    ///
    /// Saturates on overflow; see [`Timestamp::from_secs`].
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis.saturating_mul(1_000_000))
    }

    /// From whole seconds since the Unix epoch.
    ///
    /// A value too large to express in nanoseconds saturates at the end of the
    /// representable range, around the year 2262. Nothing here panics.
    ///
    /// Saturation is a last resort. An adapter reading a timestamp off an
    /// exchange checks the arithmetic itself and reports
    /// [`Error::Decode`](crate::Error::Decode), because a mis-scaled field
    /// means a changed response shape.
    pub const fn from_secs(secs: i64) -> Self {
        Self(secs.saturating_mul(1_000_000_000))
    }

    /// Nanoseconds since the Unix epoch.
    pub const fn as_nanos(self) -> i64 {
        self.0
    }

    /// Milliseconds since the Unix epoch, truncated toward the epoch.
    pub const fn as_millis(self) -> i64 {
        self.0 / 1_000_000
    }

    /// Seconds since the Unix epoch, truncated toward the epoch.
    pub const fn as_secs(self) -> i64 {
        self.0 / 1_000_000_000
    }

    /// The current time, read from the system clock.
    ///
    /// Used for signing request timestamps. It is also the read time an
    /// adapter falls back to where an exchange publishes a payload with no
    /// clock of its own. Every type that can carry such a fallback says so on
    /// the field.
    ///
    /// A clock set before the epoch reads as the epoch, and one set past the
    /// year 2262 saturates like [`Timestamp::from_secs`]. Nothing here panics
    /// and nothing wraps, so a machine with a wrong clock gives a wrong
    /// timestamp rather than a timestamp from the far side of the range.
    pub fn now() -> Self {
        Self::since_epoch(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO),
        )
    }

    /// A duration measured from the Unix epoch, as an instant.
    ///
    /// Saturates rather than truncating: a `Duration` counts nanoseconds in 128
    /// bits and this type in 64, so the conversion is narrowing and an `as`
    /// cast would wrap a clock past the year 2262 into a plausible instant in
    /// the past.
    fn since_epoch(elapsed: Duration) -> Self {
        Self(i64::try_from(elapsed.as_nanos()).unwrap_or(i64::MAX))
    }

    /// Converts to a [`SystemTime`], for handing off to a date-time library.
    ///
    /// Returns `None` for timestamps before the Unix epoch, which no exchange
    /// produces in practice.
    ///
    /// ```
    /// use std::time::{Duration, SystemTime};
    ///
    /// use maxt::Timestamp;
    ///
    /// let ts = Timestamp::from_millis(1_700_000_000_000);
    /// let at = ts.into_system_time();
    ///
    /// // The hand-off point to whichever date-time library the caller prefers.
    /// assert_eq!(at, Some(SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)));
    ///
    /// // A pre-epoch instant has no representation here, so the conversion
    /// // reports `None`.
    /// assert_eq!(Timestamp::from_secs(-1).into_system_time(), None);
    /// ```
    pub fn into_system_time(self) -> Option<SystemTime> {
        u64::try_from(self.0)
            .ok()
            .map(|nanos| UNIX_EPOCH + Duration::from_nanos(nanos))
    }
}

/// Renders as an RFC 3339 instant in UTC, to millisecond precision.
///
/// The sub-millisecond part is not shown, so this form does not round-trip.
/// Use [`Timestamp::as_nanos`] whenever the exact value matters.
///
/// ```
/// use maxt::Timestamp;
///
/// let ts = Timestamp::from_millis(1_700_000_000_000);
///
/// assert_eq!(ts.to_string(), "2023-11-14T22:13:20.000Z");
/// // The printed form has dropped precision the value still holds.
/// assert_eq!(ts.as_nanos(), 1_700_000_000_000_000_000);
/// ```
impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let instant = chrono::DateTime::from_timestamp_nanos(self.0);
        write!(f, "{}", instant.format("%Y-%m-%dT%H:%M:%S%.3fZ"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_resolution_lands_on_the_same_instant() {
        let secs = Timestamp::from_secs(1_700_000_000);

        assert_eq!(secs, Timestamp::from_millis(1_700_000_000_000));
        assert_eq!(secs, Timestamp::from_micros(1_700_000_000_000_000));
        assert_eq!(secs, Timestamp::from_nanos(1_700_000_000_000_000_000));
    }

    #[test]
    fn downscaling_truncates_instead_of_rounding() {
        let ts = Timestamp::from_nanos(1_999_999_999);

        assert_eq!(ts.as_secs(), 1);
        assert_eq!(ts.as_millis(), 1_999);
    }

    #[test]
    fn ordering_compares_instants_not_source_exchanges() {
        let earlier = Timestamp::from_millis(1_700_000_000_000);
        let later = Timestamp::from_nanos(1_700_000_000_000_000_001);

        assert!(earlier < later);
    }

    #[test]
    fn a_clock_past_the_representable_range_saturates_rather_than_wrapping() {
        // A `Duration` counts nanoseconds in 128 bits and this type in 64, so
        // reading a clock is a narrowing conversion. An `as` cast turns a
        // machine whose clock is set past the year 2262 into a plausible
        // instant somewhere else, which no caller can tell from a real one.
        let ordinary = Duration::from_secs(1_700_000_000);
        assert_eq!(
            Timestamp::since_epoch(ordinary),
            Timestamp::from_secs(1_700_000_000)
        );

        for beyond in [
            Duration::from_nanos(u64::MAX),
            Duration::from_secs(u64::MAX),
            Duration::MAX,
        ] {
            assert_eq!(
                Timestamp::since_epoch(beyond),
                Timestamp::from_nanos(i64::MAX),
                "{beyond:?}"
            );
        }
    }

    #[test]
    fn pre_epoch_timestamps_do_not_panic_on_conversion() {
        assert!(Timestamp::from_secs(-1).into_system_time().is_none());
        assert!(Timestamp::from_secs(0).into_system_time().is_some());
    }
}
