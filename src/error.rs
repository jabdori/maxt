//! Error and result types shared by every exchange.

use std::fmt;

use crate::Feature;

/// Result type returned by every fallible `maxt` operation.
pub type Result<T> = std::result::Result<T, Error>;

/// Anything that can go wrong while talking to an exchange.
///
/// The variants separate the four cases a caller has to tell apart: a bad
/// request from the caller, a feature the exchange does not offer, a rejection
/// from the exchange, and a connection that failed.
///
/// [`Error::Auth`] is drawn at the process boundary rather than at the
/// credential. It means `maxt` could not build a credentialed request and so
/// sent nothing. A credential the exchange itself read and refused comes back
/// as [`Error::Exchange`] under that exchange's own code, because what counts
/// as a refused credential is answered per exchange and `maxt` does not answer
/// it on their behalf:
///
/// | Exchange | A refused credential arrives as |
/// | --- | --- |
/// | Binance | HTTP 400 `-1022` for a bad signature, HTTP 401 `-2015` for a bad key |
/// | Upbit | HTTP 401 with a JWT error name, HTTP 403 `out_of_scope` |
/// | Bithumb | HTTP 401 with a JWT error name, under names of its own |
/// | Hyperliquid | HTTP 200, `status: "err"`, an English sentence, no code |
///
/// Each provider page lists its own. Nothing here flattens them, because a
/// rule that did would have to be right about four exchanges at once and would
/// go quietly wrong the first time one of them renamed a code.
///
/// [`Error::Auth`] and [`Error::Unsupported`] are the two that get confused.
/// Missing credentials are always `Auth`, on every adapter, because the
/// endpoint exists and a key would reach it. `Unsupported` means `maxt` maps no
/// endpoint there, which no credential can change.
///
/// ```
/// use maxt::{Client, Error, adapters::UpbitAdapter};
///
/// /// What a caller can actually do about an error.
/// fn advice(error: &Error) -> &'static str {
///     // Both checked before the variants: either one is answered by waiting
///     // rather than by reading what failed, and a rate limit asks for a
///     // longer pause than the other retryable failures do.
///     if error.is_rate_limited() {
///         return "back off, then retry";
///     }
///     if error.is_retryable() {
///         return "retry behind a backoff";
///     }
///     match error {
///         // Nothing was sent: no credentials, or none `maxt` could sign with.
///         Error::Auth { .. } => "supply credentials",
///         Error::Unsupported { feature, exchange, .. } => {
///             let _ = (feature, exchange); // both name what is missing, for a log
///             "ask another exchange"
///         }
///         Error::InvalidRequest { field, .. } => {
///             let _ = field;
///             "fix the request"
///         }
///         // The exchange read the request and refused it. A credential it
///         // rejected is here rather than in `Auth`, under that exchange's own
///         // code: a wrong Binance secret is `-1022`, a wrong Binance key is
///         // `-2015`. The provider page lists the codes worth branching on.
///         Error::Exchange { exchange, code, .. } => {
///             let _ = (exchange, code);
///             "read the exchange's own verdict"
///         }
///         _ => "report it",
///     }
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let public = Client::new(UpbitAdapter::new());
///
///     // Neither call reaches the network: both are decided before a request
///     // is built, so this example runs as a test.
///     let no_key = public.balances().await.err();
///     assert_eq!(no_key.as_ref().map(advice), Some("supply credentials"));
///
///     // Upbit lists no derivatives, so there is nothing to authenticate to.
///     let no_endpoint = public.positions().await.err();
///     assert_eq!(no_endpoint.as_ref().map(advice), Some("ask another exchange"));
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The request was rejected before it left the process.
    ///
    /// The request is malformed and retrying it unchanged will fail again.
    InvalidRequest {
        /// The request field that failed validation.
        field: &'static str,
        /// What was wrong with it.
        detail: String,
    },

    /// The exchange does not offer this feature at all.
    ///
    /// A permanent, structural answer. Bithumb publishes no candle stream, for
    /// example, so [`Feature::CandleStream`] is unsupported there however the
    /// request is phrased.
    Unsupported {
        /// The feature `maxt` maps no endpoint for.
        feature: Feature,
        /// The exchange it was asked of.
        exchange: &'static str,
        /// Why it is unmapped, and the closest alternative if there is one.
        detail: String,
    },

    /// `maxt` could not build a credentialed request, so none was sent.
    ///
    /// The credentials are missing, malformed, or unusable for signing, and
    /// every one of those is decided inside this process. An exchange that read
    /// a credential and refused it answers with [`Error::Exchange`] under its
    /// own code instead, which is the only place that code survives.
    Auth {
        /// What failed about the credentials.
        detail: String,
    },

    /// The exchange accepted the connection and answered with an error.
    ///
    /// Including a credential it read and refused. `code` and `message` stay
    /// the exchange's own rather than being folded into [`Error::Auth`],
    /// because no two of the four exchanges spell a refused credential alike
    /// and the code is what a caller branches on.
    Exchange {
        /// The exchange that answered.
        exchange: &'static str,
        /// The exchange's own error code, verbatim. The name of a pushed event
        /// where the exchange sent no code, as Binance's `listenKeyExpired` is.
        code: String,
        /// The exchange's own error message, verbatim where it sent one, and
        /// what happened where it sent only an event name.
        message: String,
        /// HTTP status, where the exchange gave one. Binance's WebSocket API
        /// puts one inside the frame, so this is set there too.
        status: Option<u16>,
        /// How the error classifies for retry purposes.
        kind: ExchangeErrorKind,
    },

    /// The request never completed: DNS, TLS, socket, or timeout.
    Transport {
        /// What failed about the connection.
        detail: String,
    },

    /// The exchange answered, but the payload could not be read.
    ///
    /// In practice this means the exchange changed a response shape. It is
    /// worth reporting as a bug against `maxt`.
    Decode {
        /// What could not be read.
        detail: String,
    },
}

impl Error {
    /// Whether retrying the identical request could plausibly succeed.
    ///
    /// Rate limits, exchange-side failures, and transport faults are worth
    /// retrying behind a backoff. Validation errors, unsupported features, and
    /// missing credentials will fail identically every time.
    ///
    /// The identical request is what this asks about, and a request the
    /// exchange rejected is `false` here even where building a fresh one would
    /// succeed. A refused credential is the clear case, a stale timestamp the
    /// arguable one: see [`ExchangeErrorKind::Rejected`].
    ///
    /// ```
    /// use maxt::{Client, adapters::BithumbAdapter};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let public = Client::new(BithumbAdapter::new());
    ///
    ///     // A missing key is not a blip. A retry loop that ignored this would
    ///     // spin through its whole attempt budget and report the same failure.
    ///     let error = public.open_orders().await.err();
    ///     assert_eq!(error.map(|error| error.is_retryable()), Some(false));
    /// }
    /// ```
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Exchange { kind, .. } => kind.is_retryable(),
            Self::Transport { .. } => true,
            Self::InvalidRequest { .. }
            | Self::Unsupported { .. }
            | Self::Auth { .. }
            | Self::Decode { .. } => false,
        }
    }

    /// Whether the exchange refused because the caller sent requests too fast.
    ///
    /// Worth branching on separately from [`Error::is_retryable`]: a rate limit
    /// asks for a longer pause than a transport blip does.
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use maxt::{Client, Error, adapters::UpbitAdapter};
    ///
    /// /// How long to wait before sending the identical request again.
    /// fn pause_after(error: &Error, attempt: u32) -> Option<Duration> {
    ///     if error.is_rate_limited() {
    ///         // A quota refills on the exchange's clock. An exponential step
    ///         // measured in milliseconds just spends the next window as soon
    ///         // as it opens.
    ///         Some(Duration::from_secs(30))
    ///     } else if error.is_retryable() {
    ///         Some(Duration::from_millis(100 << attempt.min(6)))
    ///     } else {
    ///         None
    ///     }
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let error = Client::new(UpbitAdapter::new()).balances().await.err();
    ///
    ///     // Not rate limited, not retryable: this one never waits at all.
    ///     assert_eq!(error.as_ref().and_then(|error| pause_after(error, 0)), None);
    /// }
    /// ```
    pub fn is_rate_limited(&self) -> bool {
        matches!(
            self,
            Self::Exchange {
                kind: ExchangeErrorKind::RateLimited,
                ..
            }
        )
    }

    pub(crate) fn invalid_request(field: &'static str, detail: impl Into<String>) -> Self {
        Self::InvalidRequest {
            field,
            detail: detail.into(),
        }
    }

    pub(crate) fn unsupported(
        feature: Feature,
        exchange: &'static str,
        detail: impl Into<String>,
    ) -> Self {
        Self::Unsupported {
            feature,
            exchange,
            detail: detail.into(),
        }
    }

    pub(crate) fn auth(detail: impl Into<String>) -> Self {
        Self::Auth {
            detail: detail.into(),
        }
    }

    pub(crate) fn transport(detail: impl Into<String>) -> Self {
        Self::Transport {
            detail: detail.into(),
        }
    }

    pub(crate) fn decode(detail: impl Into<String>) -> Self {
        Self::Decode {
            detail: detail.into(),
        }
    }

    pub(crate) fn exchange(
        exchange: &'static str,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Exchange {
            exchange,
            code: code.into(),
            message: message.into(),
            status: None,
            kind: ExchangeErrorKind::Unknown,
        }
    }

    pub(crate) fn exchange_http(
        exchange: &'static str,
        status: u16,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Exchange {
            exchange,
            code: code.into(),
            message: message.into(),
            status: Some(status),
            kind: ExchangeErrorKind::from_status(status),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest { field, detail } => {
                write!(f, "invalid request: `{field}`: {detail}")
            }
            Self::Unsupported {
                feature,
                exchange,
                detail,
            } => write!(f, "{exchange} does not support {feature}: {detail}"),
            Self::Auth { detail } => write!(f, "authentication failed: {detail}"),
            Self::Exchange {
                exchange,
                code,
                message,
                status,
                ..
            } => match status {
                Some(status) => write!(f, "{exchange} returned {status} {code}: {message}"),
                None => write!(f, "{exchange} returned {code}: {message}"),
            },
            Self::Transport { detail } => write!(f, "transport failed: {detail}"),
            Self::Decode { detail } => write!(f, "could not read exchange response: {detail}"),
        }
    }
}

impl std::error::Error for Error {}

/// How an exchange-side error classifies for retry purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ExchangeErrorKind {
    /// The request was wrong: a bad symbol, an insufficient balance, a
    /// signature or a credential the exchange would not accept.
    ///
    /// A timestamp outside the exchange's receive window is here too, and not
    /// under [`Self::Unavailable`], although clock drift is transient. The two
    /// causes want opposite things from a retry loop: a clock that is genuinely
    /// wrong fails every rebuild until someone corrects it, so a loop spends
    /// its whole budget learning that, while a request merely delayed in flight
    /// wants one request built again and sent once. Neither is a loop, and
    /// [`Error::is_retryable`] is about sending the identical request again,
    /// which for a signed timestamp can only fail further outside the window
    /// than it did the first time.
    Rejected,
    /// The caller exceeded a rate limit or is temporarily banned.
    RateLimited,
    /// The exchange failed on its own side.
    Unavailable,
    /// The exchange did not classify the failure.
    Unknown,
}

impl ExchangeErrorKind {
    /// Whether an error of this kind is worth retrying behind a backoff.
    ///
    /// ```
    /// use maxt::ExchangeErrorKind;
    ///
    /// // The exchange's own fault, or ours for being too fast: both pass.
    /// assert!(ExchangeErrorKind::Unavailable.is_retryable());
    /// assert!(ExchangeErrorKind::RateLimited.is_retryable());
    ///
    /// // A rejection is a verdict on the request itself.
    /// assert!(!ExchangeErrorKind::Rejected.is_retryable());
    /// ```
    pub fn is_retryable(self) -> bool {
        matches!(self, Self::RateLimited | Self::Unavailable)
    }

    fn from_status(status: u16) -> Self {
        match status {
            // 418 is Binance's "you ignored 429 and kept going" ban response.
            418 | 429 => Self::RateLimited,
            400..=499 => Self::Rejected,
            500..=599 => Self::Unavailable,
            _ => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_classification_follows_http_status() {
        let cases = [
            (429, ExchangeErrorKind::RateLimited, true),
            (418, ExchangeErrorKind::RateLimited, true),
            (503, ExchangeErrorKind::Unavailable, true),
            (400, ExchangeErrorKind::Rejected, false),
            (401, ExchangeErrorKind::Rejected, false),
        ];

        for (status, expected_kind, expected_retryable) in cases {
            let error = Error::exchange_http("upbit", status, "code", "message");
            let Error::Exchange { kind, .. } = error else {
                panic!("expected an exchange error for status {status}");
            };
            assert_eq!(kind, expected_kind, "status {status}");
            assert_eq!(error.is_retryable(), expected_retryable, "status {status}");
        }
    }

    #[test]
    fn caller_side_errors_are_never_retryable() {
        assert!(!Error::invalid_request("limit", "must be 1..=200").is_retryable());
        assert!(!Error::auth("missing secret key").is_retryable());
        assert!(!Error::decode("unexpected null in `price`").is_retryable());
        assert!(
            !Error::unsupported(Feature::CandleStream, "bithumb", "no public candle stream")
                .is_retryable()
        );
    }

    #[test]
    fn rate_limit_is_distinguishable_from_other_retryable_errors() {
        assert!(
            Error::exchange_http("binance", 429, "-1003", "too many requests").is_rate_limited()
        );
        assert!(!Error::exchange_http("binance", 503, "-1001", "disconnected").is_rate_limited());
        assert!(!Error::transport("connection reset").is_rate_limited());
        assert!(Error::transport("connection reset").is_retryable());
    }

    #[test]
    fn display_keeps_the_exchange_verdict_verbatim() {
        let error = Error::exchange_http("binance", 400, "-1121", "Invalid symbol.");
        assert_eq!(
            error.to_string(),
            "binance returned 400 -1121: Invalid symbol."
        );
    }
}
