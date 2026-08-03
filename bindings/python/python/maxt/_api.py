from __future__ import annotations

import asyncio
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from importlib import import_module
from inspect import isawaitable
from typing import Any, AsyncIterator, Generic, Literal, Optional, TypeVar, Union

from ._generated_identifiers import ExchangeErrorKind
from ._generated_wire import ERROR_FIELDS
from .models import (
    AccountEvent,
    Balance,
    Candle,
    CandleRequest,
    Exchange,
    Feature,
    FundingPayment,
    FundingRate,
    HistoryRequest,
    MarginRequest,
    MarginSummary,
    Market,
    MarketEvent,
    MarketInfo,
    MarketKind,
    Order,
    OrderBook,
    OrderRequest,
    Page,
    Position,
    StreamConfig,
    Subscription,
    Ticker,
    Trade,
    _model_to_wire,
)


A = TypeVar("A", bound="Adapter")


class MaxtError(Exception):
    """Base class for maxt operation failures."""

    kind = "maxt"

    def to_wire(self) -> dict[str, Any]:
        return {"kind": self.kind, **_model_to_wire(self.__dict__)}

    def is_retryable(self) -> bool:
        return False

    def is_rate_limited(self) -> bool:
        return False


class InvalidRequestError(MaxtError):
    kind = "invalid_request"

    def __init__(self, field: str, detail: str) -> None:
        self.field = field
        self.detail = detail
        super().__init__(f"invalid request: `{field}`: {detail}")


class DecodeError(MaxtError):
    kind = "decode"

    def __init__(self, detail: str) -> None:
        self.detail = detail
        super().__init__(detail)


class AuthError(MaxtError):
    kind = "auth"

    def __init__(self, detail: str) -> None:
        self.detail = detail
        super().__init__(detail)


class TransportError(MaxtError):
    kind = "transport"

    def __init__(self, detail: str) -> None:
        self.detail = detail
        super().__init__(detail)

    def is_retryable(self) -> bool:
        return True


class AdapterError(MaxtError):
    kind = "adapter"

    def __init__(self, detail: str) -> None:
        self.detail = detail
        super().__init__(detail)


class ExchangeError(MaxtError):
    kind = "exchange"

    def __init__(
        self,
        exchange: Exchange,
        code: str,
        message: str,
        status: Optional[int],
        exchange_kind: ExchangeErrorKind,
    ) -> None:
        self.exchange = exchange
        self.code = code
        self.message = message
        self.status = status
        self.exchange_kind = exchange_kind
        status_text = f" {status}" if status is not None else ""
        super().__init__(f"{exchange.value} returned{status_text} {code}: {message}")

    def is_retryable(self) -> bool:
        return self.exchange_kind.is_retryable()

    def is_rate_limited(self) -> bool:
        return self.exchange_kind is ExchangeErrorKind.RATE_LIMITED


def _load_native() -> Any:
    return import_module("maxt._native")


def _error_from_wire(value: dict[str, Any]) -> MaxtError:
    kind_value = value.get("kind")
    kind = kind_value if isinstance(kind_value, str) else None
    expected_fields = ERROR_FIELDS.get(kind) if kind is not None else None
    if expected_fields is not None:
        present = set(value)
        if kind == "exchange" and "provider_message" in present:
            present.add("message")
        required = {
            name
            for name, field_type in expected_fields.items()
            if not field_type.startswith("optional:")
        }
        missing = required - present
        if missing:
            return AdapterError(
                f"invalid {kind} error: missing {', '.join(sorted(missing))}"
            )
    if kind == "invalid_request":
        return InvalidRequestError(value["field"], value["detail"])
    if kind == "unsupported":
        return UnsupportedError(
            Feature(value["feature"]),
            Exchange(value["exchange"]),
            value["detail"],
        )
    if kind == "auth":
        return AuthError(value["detail"])
    if kind == "adapter":
        return AdapterError(value["detail"])
    if kind == "exchange":
        return ExchangeError(
            Exchange(value["exchange"]),
            value["code"],
            value["provider_message"]
            if "provider_message" in value
            else value["message"],
            value.get("status"),
            ExchangeErrorKind(value["exchange_kind"]),
        )
    if kind == "transport":
        return TransportError(value["detail"])
    if kind == "decode":
        return DecodeError(value["detail"])
    return AdapterError(value.get("message") or value.get("detail") or str(value))


class UnsupportedError(MaxtError):
    kind = "unsupported"

    def __init__(
        self,
        feature: Feature,
        exchange: Exchange,
        detail: Optional[str] = None,
    ) -> None:
        self.feature = feature
        self.exchange = exchange
        self.detail = detail or f"{exchange.value} has no endpoint for {feature.value}"
        super().__init__(self.detail)


T = TypeVar("T")


@dataclass(frozen=True)
class StreamEvent(Generic[T]):
    event: T
    kind: Literal["event"] = field(default="event", init=False)


@dataclass(frozen=True)
class StreamError:
    error: MaxtError
    kind: Literal["error"] = field(default="error", init=False)


class AsyncStream(Generic[T]):
    """Async iterator with deterministic close semantics."""

    def __init__(self, source: AsyncIterator[T]) -> None:
        self._source = source.__aiter__()
        self._closed = False
        self._close_complete = False
        self._close_task: Optional[asyncio.Future[Any]] = None

    def __aiter__(self) -> AsyncStream[T]:
        return self

    async def __anext__(self) -> T:
        if self._closed:
            raise StopAsyncIteration
        try:
            item = await self._source.__anext__()
        except StopAsyncIteration:
            self._closed = True
            raise
        return item

    async def aclose(self) -> None:
        if self._close_complete:
            return
        self._closed = True
        if self._close_task is None:
            close = getattr(self._source, "aclose", None)
            if close is None:
                self._close_complete = True
                return
            result = close()
            if not isawaitable(result):
                self._close_complete = True
                return
            self._close_task = asyncio.ensure_future(result)
        await asyncio.shield(self._close_task)
        self._close_complete = True

    async def __aenter__(self) -> AsyncStream[T]:
        return self

    async def __aexit__(self, *exc_info: object) -> None:
        await self.aclose()


class MarketStream(AsyncStream[T]):
    pass


class AccountStream(AsyncStream[T]):
    pass


from ._generated_api import _GeneratedAdapterApi, _GeneratedClientApi


class Adapter(_GeneratedAdapterApi, ABC):
    """Interface implemented by built-in and custom exchange adapters."""

    @property
    @abstractmethod
    def exchange(self) -> Exchange:
        raise NotImplementedError

    @property
    @abstractmethod
    def features(self) -> frozenset[Feature]:
        raise NotImplementedError

    def supports(self, feature: Feature) -> bool:
        return feature in self.features

    def _unsupported(self, feature: Feature) -> UnsupportedError:
        return UnsupportedError(feature, self.exchange)

    async def subscribe(
        self,
        subscription: Subscription,
        config: StreamConfig,
    ) -> MarketStream[Union[StreamEvent[MarketEvent], StreamError]]:
        """Open a stream; `StreamError` items do not terminate iteration."""
        if not subscription.markets:
            raise InvalidRequestError(
                "markets",
                "a subscription needs at least one market",
            )
        if not subscription.feeds:
            raise InvalidRequestError(
                "feeds",
                "a subscription needs at least one feed",
            )
        feature = {
            "order_book": Feature.ORDER_BOOK_STREAM,
            "ticker": Feature.TICKER_STREAM,
            "candles": Feature.CANDLE_STREAM,
        }.get(subscription.feeds[0].kind, Feature.TRADE_STREAM)
        raise self._unsupported(feature)


class Client(_GeneratedClientApi, Generic[A]):
    """Consistent public API over one adapter."""

    def __init__(self, adapter: A) -> None:
        self._adapter = adapter
        self._delegate: Adapter = import_module("maxt.adapters")._client_delegate(adapter)

    @property
    def adapter(self) -> A:
        return self._adapter

    def into_adapter(self) -> A:
        return self._adapter

    def exchange(self) -> Exchange:
        return self._delegate.exchange

    def supports(self, feature: Feature) -> bool:
        return self._delegate.supports(feature)

__all__ = [
    "AccountStream",
    "Adapter",
    "AdapterError",
    "AsyncStream",
    "AuthError",
    "Client",
    "DecodeError",
    "ExchangeError",
    "ExchangeErrorKind",
    "InvalidRequestError",
    "MarketStream",
    "MaxtError",
    "StreamError",
    "StreamEvent",
    "TransportError",
    "UnsupportedError",
]
