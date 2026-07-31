//! Error and result types shared by every exchange.

use std::fmt;

use crate::Feature;

/// Result type returned by every fallible `maxt` operation.
pub type Result<T> = std::result::Result<T, Error>;

/// Local request, adapter, authentication, exchange, transport, and decoding failures.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// The request was rejected before it left the process.
    ///
    /// The request is malformed and retrying it unchanged will fail again.
    InvalidRequest {
        /// The request field that failed validation.
        field: String,
        /// What was wrong with it.
        detail: String,
    },

    /// The adapter does not map this feature or request shape.
    ///
    /// Retrying the same request or adding credentials cannot change this
    /// result. The exchange may still expose a native API outside `maxt`.
    Unsupported {
        /// The unmapped feature.
        feature: Feature,
        /// The configured exchange.
        exchange: &'static str,
        /// Mapping details and any available alternative.
        detail: String,
    },

    /// An adapter or foreign dispatcher violated the [`Adapter`](crate::Adapter)
    /// contract.
    Adapter {
        /// What contract boundary failed.
        detail: String,
    },

    /// A credentialed request could not be built locally, so none was sent.
    Auth {
        /// Credential or signing failure.
        detail: String,
    },

    /// Error response from the exchange, including rejected credentials.
    Exchange {
        /// The exchange that answered.
        exchange: &'static str,
        /// Provider error code or event name.
        code: String,
        /// Provider error message.
        message: String,
        /// HTTP status when available.
        status: Option<u16>,
        /// How the error classifies for retry purposes.
        kind: ExchangeErrorKind,
    },

    /// The request never completed: DNS, TLS, socket, or timeout.
    Transport {
        /// What failed about the connection.
        detail: String,
    },

    /// The response payload could not be decoded.
    Decode {
        /// Decode failure.
        detail: String,
    },
}

impl Error {
    /// Whether retrying the identical request could plausibly succeed.
    ///
    /// Returns `true` for rate limits, exchange unavailability, and transport
    /// failures. A rejected request is `false`, even when rebuilding a request
    /// with a fresh timestamp could succeed.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Exchange { kind, .. } => kind.is_retryable(),
            Self::Transport { .. } => true,
            Self::InvalidRequest { .. }
            | Self::Unsupported { .. }
            | Self::Adapter { .. }
            | Self::Auth { .. }
            | Self::Decode { .. } => false,
        }
    }

    /// Whether the exchange refused because the caller sent requests too fast.
    ///
    /// Worth branching on separately from [`Error::is_retryable`]: a rate limit
    /// asks for a longer pause than a transport blip does.
    pub fn is_rate_limited(&self) -> bool {
        matches!(
            self,
            Self::Exchange {
                kind: ExchangeErrorKind::RateLimited,
                ..
            }
        )
    }

    pub(crate) fn invalid_request(field: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::InvalidRequest {
            field: field.into(),
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

    /// Builds an adapter contract error.
    pub fn adapter(detail: impl Into<String>) -> Self {
        Self::Adapter {
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
            } => write!(f, "{exchange} adapter does not support {feature}: {detail}"),
            Self::Adapter { detail } => write!(f, "adapter failed: {detail}"),
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
    /// This includes timestamps outside the exchange's receive window. Retrying
    /// the identical signed request cannot refresh its timestamp.
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
    fn invalid_request_keeps_a_runtime_defined_field_name() {
        let field = format!("custom_{}", "field");
        let error = Error::invalid_request(field.clone(), "bad value");

        assert!(matches!(
            error,
            Error::InvalidRequest { field: actual, .. } if actual == field
        ));
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
