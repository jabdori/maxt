from __future__ import annotations

from dataclasses import dataclass, field, fields, is_dataclass
from decimal import Decimal
from enum import Enum
from typing import (
    Any,
    ClassVar,
    Generic,
    Optional,
    Tuple,
    TypeVar,
    Union,
    get_args,
    get_origin,
    get_type_hints,
)

from ._generated_identifiers import (
    BinanceMarket,
    BithumbAlertStep,
    Exchange,
    Feature,
    HyperliquidLedgerKind,
    Interval,
    MarginMode,
    MarketKind,
    MarketStatus,
    OrderStatus,
    OrderType,
    Overflow,
    Side,
    SizeKind,
    TimeInForce,
    UpbitRegion,
)
from ._generated_wire import RECORD_FIELDS


Timestamp = int
T = TypeVar("T")

def _ascii_upper(value: str) -> str:
    return "".join(
        chr(ord(character) - 32) if "a" <= character <= "z" else character
        for character in value
    )


class WireModel:
    def to_wire(self) -> dict[str, Any]:
        return _model_to_wire(self)

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> Any:
        return _decode_dataclass(cls, value)


@dataclass(frozen=True)
class Market(WireModel):
    exchange: Exchange
    kind: MarketKind
    base: str
    quote: str

    def __post_init__(self) -> None:
        object.__setattr__(self, "base", _ascii_upper(self.base))
        object.__setattr__(self, "quote", _ascii_upper(self.quote))

    @classmethod
    def spot(cls, exchange: Exchange, base: str, quote: str) -> Market:
        return cls(exchange, MarketKind.SPOT, base, quote)

    @classmethod
    def perpetual(cls, exchange: Exchange, base: str, quote: str) -> Market:
        return cls(exchange, MarketKind.PERPETUAL, base, quote)

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> Market:
        return cls(
            exchange=Exchange(value["exchange"]),
            kind=MarketKind(value["kind"]),
            base=value["base"],
            quote=value["quote"],
        )


@dataclass(frozen=True)
class Trade(WireModel):
    market: Market
    timestamp: Timestamp
    price: Decimal
    quantity: Decimal
    taker_side: Side
    id: Optional[str]

    @classmethod
    def from_wire(cls, value: dict[str, Any]) -> Trade:
        return cls(
            market=Market.from_wire(value["market"]),
            timestamp=value["timestamp"],
            price=Decimal(value["price"]),
            quantity=Decimal(value["quantity"]),
            taker_side=Side(value["taker_side"]),
            id=value.get("id"),
        )


@dataclass(frozen=True)
class MarketInfo(WireModel):
    market: Market
    native_symbol: str
    status: MarketStatus
    korean_name: Optional[str]
    english_name: Optional[str]


@dataclass(frozen=True)
class Level(WireModel):
    price: Decimal
    quantity: Decimal


@dataclass(frozen=True)
class OrderBook(WireModel):
    market: Market
    timestamp: Timestamp
    bids: list[Level]
    asks: list[Level]

    def best_bid(self) -> Optional[Level]:
        return self.bids[0] if self.bids else None

    def best_ask(self) -> Optional[Level]:
        return self.asks[0] if self.asks else None

    def spread(self) -> Optional[Decimal]:
        if not self.bids or not self.asks:
            return None
        return self.asks[0].price - self.bids[0].price

    def mid_price(self) -> Optional[Decimal]:
        if not self.bids or not self.asks:
            return None
        return (self.asks[0].price + self.bids[0].price) / 2


@dataclass(frozen=True)
class Ticker(WireModel):
    market: Market
    timestamp: Timestamp
    last_trade_time: Optional[Timestamp]
    last_price: Decimal
    change: Optional[Decimal]
    change_rate: Optional[Decimal]
    high: Optional[Decimal]
    low: Optional[Decimal]
    volume: Optional[Decimal]
    quote_volume: Optional[Decimal]


@dataclass(frozen=True)
class Candle(WireModel):
    market: Market
    interval: Interval
    open_time: Timestamp
    open: Decimal
    high: Decimal
    low: Decimal
    close: Decimal
    volume: Decimal
    quote_volume: Optional[Decimal]
    closed: bool


@dataclass(frozen=True)
class CandleRequest(WireModel):
    """Select candles by open time.

    `from_` is inclusive and `to` is exclusive; results are oldest-first.
    With `from_`, `limit` selects the oldest matches; otherwise the newest.
    """

    market: Market
    interval: Interval
    from_: Optional[Timestamp] = field(default=None, metadata={"wire_name": "from"})
    to: Optional[Timestamp] = None
    limit: Optional[int] = None


@dataclass(frozen=True)
class Size(WireModel):
    kind: SizeKind
    value: Decimal

    @classmethod
    def base(cls, value: Decimal) -> Size:
        return cls(SizeKind.BASE, value)

    @classmethod
    def quote(cls, value: Decimal) -> Size:
        return cls(SizeKind.QUOTE, value)


@dataclass(frozen=True)
class OrderRequest(WireModel):
    market: Market
    side: Side
    order_type: OrderType
    size: Size
    price: Optional[Decimal] = None
    time_in_force: Optional[TimeInForce] = None
    reduce_only: bool = False

    def __post_init__(self) -> None:
        if self.order_type is OrderType.MARKET and self.price is not None:
            raise ValueError("market orders must not include price")
        if self.order_type is OrderType.LIMIT and self.price is None:
            raise ValueError("limit orders require price")

    @classmethod
    def market_order(
        cls,
        market: Market,
        side: Side,
        size: Size,
        *,
        time_in_force: Optional[TimeInForce] = None,
        reduce_only: bool = False,
    ) -> OrderRequest:
        return cls(
            market,
            side,
            OrderType.MARKET,
            size,
            time_in_force=time_in_force,
            reduce_only=reduce_only,
        )

    @classmethod
    def limit_order(
        cls,
        market: Market,
        side: Side,
        size: Size,
        price: Decimal,
        *,
        time_in_force: Optional[TimeInForce] = None,
        reduce_only: bool = False,
    ) -> OrderRequest:
        return cls(
            market,
            side,
            OrderType.LIMIT,
            size,
            price,
            time_in_force,
            reduce_only,
        )


@dataclass(frozen=True)
class StreamConfig(WireModel):
    max_reconnect_attempts: Optional[int] = None
    initial_reconnect_delay_ms: int = 1_000
    max_reconnect_delay_ms: int = 30_000
    idle_timeout_ms: int = 30_000
    buffer_size: int = 4_096
    overflow: Overflow = Overflow.BACKPRESSURE

    def __post_init__(self) -> None:
        values = (
            ("max_reconnect_attempts", self.max_reconnect_attempts),
            ("initial_reconnect_delay_ms", self.initial_reconnect_delay_ms),
            ("max_reconnect_delay_ms", self.max_reconnect_delay_ms),
            ("idle_timeout_ms", self.idle_timeout_ms),
            ("buffer_size", self.buffer_size),
        )
        for field_name, value in values:
            if value is not None and value < 0:
                raise ValueError(f"{field_name} must be non-negative")


@dataclass(frozen=True)
class Feed(WireModel):
    kind: str
    interval: Optional[Interval] = None

    TRADES: ClassVar[Feed]
    ORDER_BOOK: ClassVar[Feed]
    TICKER: ClassVar[Feed]

    @classmethod
    def candles(cls, interval: Interval) -> Feed:
        return cls("candles", interval)


Feed.TRADES = Feed("trades")
Feed.ORDER_BOOK = Feed("order_book")
Feed.TICKER = Feed("ticker")


@dataclass(frozen=True)
class Subscription(WireModel):
    markets: Tuple[Market, ...]
    feeds: Tuple[Feed, ...]

    def __post_init__(self) -> None:
        object.__setattr__(self, "markets", tuple(dict.fromkeys(self.markets)))
        object.__setattr__(self, "feeds", tuple(dict.fromkeys(self.feeds)))


@dataclass(frozen=True)
class Balance(WireModel):
    asset: str
    available: Decimal
    locked: Decimal

    def __post_init__(self) -> None:
        object.__setattr__(self, "asset", _ascii_upper(self.asset))

    def total(self) -> Decimal:
        """Return the available balance plus the locked balance."""
        return self.available + self.locked


@dataclass(frozen=True)
class Order(WireModel):
    id: str
    market: Market
    side: Side
    status: OrderStatus
    filled_quantity: Decimal
    remaining_quantity: Decimal
    price: Optional[Decimal]
    created_at: Optional[Timestamp]


@dataclass(frozen=True)
class MarketEvent(WireModel):
    kind: str
    value: Any = None

    @classmethod
    def trade(cls, trade: Trade) -> MarketEvent:
        """Wrap a trade as a market event."""
        return cls("trade", trade)

    @classmethod
    def order_book(cls, order_book: OrderBook) -> MarketEvent:
        """Wrap an order book as a market event."""
        return cls("order_book", order_book)

    @classmethod
    def ticker(cls, ticker: Ticker) -> MarketEvent:
        """Wrap a ticker as a market event."""
        return cls("ticker", ticker)

    @classmethod
    def candle(cls, candle: Candle) -> MarketEvent:
        """Wrap a candle as a market event."""
        return cls("candle", candle)

    @classmethod
    def reconnected(cls) -> MarketEvent:
        """Report a reconnect gap in market events."""
        return cls("reconnected")


@dataclass(frozen=True)
class AccountEvent(WireModel):
    kind: str
    value: Any = None

    @classmethod
    def balance(cls, balance: Balance) -> AccountEvent:
        return cls("balance", balance)

    @classmethod
    def order(cls, order: Order) -> AccountEvent:
        return cls("order", order)

    @classmethod
    def reconnected(cls) -> AccountEvent:
        return cls("reconnected")


@dataclass(frozen=True)
class UpbitMarketEvent(WireModel):
    warning: bool
    cautions: list[str]


@dataclass(frozen=True)
class BithumbMarketAlert(WireModel):
    kind: str
    step: BithumbAlertStep
    ends_at: Timestamp


@dataclass(frozen=True)
class BinanceSymbolFilters(WireModel):
    symbol: str
    tick_size: Optional[Decimal]
    min_price: Optional[Decimal]
    max_price: Optional[Decimal]
    step_size: Optional[Decimal]
    min_quantity: Optional[Decimal]
    max_quantity: Optional[Decimal]
    min_notional: Optional[Decimal]


@dataclass(frozen=True)
class BinanceSpotOrderDetail(WireModel):
    order: Order
    client_order_id: str
    order_type: str
    time_in_force: str
    filled_quote_quantity: Decimal
    updated_at: Optional[Timestamp]


@dataclass(frozen=True)
class HyperliquidLedgerEntry(WireModel):
    kind: HyperliquidLedgerKind
    time: Timestamp
    hash: str
    asset: Optional[str]
    amount: Optional[Decimal]
    fee: Optional[Decimal]
    counterparty: Optional[str]


@dataclass(frozen=True)
class HyperliquidAssetContext(WireModel):
    mid_price: Optional[Decimal]
    mark_price: Optional[Decimal]
    oracle_price: Optional[Decimal]
    funding_rate: Optional[Decimal]
    open_interest: Optional[Decimal]
    size_decimals: int
    price_decimals: int


@dataclass(frozen=True)
class Position(WireModel):
    market: Market
    side: Optional[Side]
    quantity: Decimal
    entry_price: Optional[Decimal] = None
    mark_price: Optional[Decimal] = None
    notional: Optional[Decimal] = None
    unrealized_pnl: Optional[Decimal] = None
    leverage: Optional[Decimal] = None
    margin_mode: Optional[MarginMode] = None

    def is_flat(self) -> bool:
        return self.quantity == 0


@dataclass(frozen=True)
class MarginSummary(WireModel):
    asset: str
    equity: Optional[Decimal]
    margin_balance: Optional[Decimal]
    available_balance: Optional[Decimal]

    def __post_init__(self) -> None:
        object.__setattr__(self, "asset", _ascii_upper(self.asset))


@dataclass(frozen=True)
class FundingRate(WireModel):
    market: Market
    timestamp: Timestamp
    rate: Decimal
    mark_price: Optional[Decimal]


@dataclass(frozen=True)
class FundingPayment(WireModel):
    market: Market
    timestamp: Timestamp
    amount: Decimal
    rate: Optional[Decimal]
    id: Optional[str]


class Cursor(str):
    def as_str(self) -> str:
        return str(self)

    def to_wire(self) -> str:
        return str(self)


@dataclass(frozen=True)
class Page(WireModel, Generic[T]):
    items: list[T]
    next: Optional[Cursor]

    def has_more(self) -> bool:
        return self.next is not None


@dataclass(frozen=True)
class HistoryRequest(WireModel):
    """Select one page of timestamped history.

    `from_` is inclusive and `to` is exclusive. `cursor` overrides `from_`.
    `limit` is a target, not a hard maximum; timestamp groups remain whole.
    """

    market: Market
    from_: Optional[Timestamp] = field(default=None, metadata={"wire_name": "from"})
    to: Optional[Timestamp] = None
    cursor: Optional[Cursor] = None
    limit: Optional[int] = None


@dataclass(frozen=True)
class MarginRequest(WireModel):
    market: Market
    leverage: Optional[Decimal] = None
    margin_mode: Optional[MarginMode] = None


def _decimal_to_wire(value: Decimal) -> str:
    if not value.is_finite():
        raise ValueError("maxt decimal values must be finite")
    return format(value, "f")


def _decode_dataclass(model_type: Any, value: dict[str, Any]) -> Any:
    hints = get_type_hints(model_type)
    model_fields = _schema_fields(model_type)
    return model_type(
        **{
            item.name: _decode_value(
                hints[item.name],
                value[wire_name],
            )
            for item, wire_name in model_fields
        }
    )


def _schema_fields(model_type: Any) -> list[tuple[Any, str]]:
    model_fields = {
        item.metadata.get("wire_name", item.name): item for item in fields(model_type)
    }
    schema_fields = RECORD_FIELDS.get(model_type.__name__)
    if schema_fields is None:
        return [(item, wire_name) for wire_name, item in model_fields.items()]
    if set(model_fields) != set(schema_fields):
        raise TypeError(
            f"{model_type.__name__} fields do not match the generated binding schema"
        )
    return [(model_fields[wire_name], wire_name) for wire_name in schema_fields]


def _decode_value(expected: Any, value: Any) -> Any:
    if value is None or expected is Any:
        return value
    origin = get_origin(expected)
    arguments = get_args(expected)
    if origin is Union:
        actual = next(item for item in arguments if item is not type(None))
        return _decode_value(actual, value)
    if origin is list:
        return [_decode_value(arguments[0], item) for item in value]
    if origin is tuple:
        if len(arguments) == 2 and arguments[1] is Ellipsis:
            return tuple(_decode_value(arguments[0], item) for item in value)
        return tuple(
            _decode_value(item_type, item)
            for item_type, item in zip(arguments, value)
        )
    if expected is Decimal:
        return Decimal(value)
    if isinstance(expected, type) and issubclass(expected, Enum):
        return expected(value)
    if isinstance(expected, type) and issubclass(expected, WireModel):
        return expected.from_wire(value)
    return value


def _model_to_wire(value: Any) -> Any:
    if isinstance(value, Cursor):
        return str(value)
    if isinstance(value, Decimal):
        return _decimal_to_wire(value)
    if isinstance(value, Enum):
        return value.value
    if is_dataclass(value):
        return {
            wire_name: _model_to_wire(getattr(value, item.name))
            for item, wire_name in _schema_fields(type(value))
        }
    if isinstance(value, (list, tuple)):
        return [_model_to_wire(item) for item in value]
    if isinstance(value, (set, frozenset)):
        return [_model_to_wire(item) for item in value]
    if isinstance(value, dict):
        return {key: _model_to_wire(item) for key, item in value.items()}
    return value


def _model_from_wire(type_name: str, value: dict[str, Any]) -> Any:
    page_types: dict[str, type[WireModel]] = {
        "FundingRatePage": FundingRate,
        "FundingPaymentPage": FundingPayment,
    }
    if type_name in page_types:
        item_type = page_types[type_name]
        return Page(
            [item_type.from_wire(item) for item in value["items"]],
            Cursor(value["next"]) if value.get("next") is not None else None,
        )
    model_type = globals().get(type_name)
    if type_name not in RECORD_FIELDS or not isinstance(model_type, type):
        raise ValueError(f"unknown maxt model: {type_name}")
    if not issubclass(model_type, WireModel):
        raise ValueError(f"unknown maxt model: {type_name}")
    return model_type.from_wire(value)


__all__ = [
    "AccountEvent",
    "BinanceMarket",
    "BinanceSpotOrderDetail",
    "BinanceSymbolFilters",
    "BithumbAlertStep",
    "BithumbMarketAlert",
    "Candle",
    "CandleRequest",
    "Balance",
    "Cursor",
    "Decimal",
    "Exchange",
    "Feed",
    "Feature",
    "FundingPayment",
    "FundingRate",
    "HyperliquidAssetContext",
    "HyperliquidLedgerEntry",
    "HyperliquidLedgerKind",
    "HistoryRequest",
    "Interval",
    "Level",
    "Market",
    "MarketEvent",
    "MarketInfo",
    "MarketKind",
    "MarketStatus",
    "MarginMode",
    "MarginRequest",
    "MarginSummary",
    "Overflow",
    "Order",
    "OrderBook",
    "OrderRequest",
    "OrderStatus",
    "OrderType",
    "Page",
    "Position",
    "Side",
    "Size",
    "SizeKind",
    "StreamConfig",
    "Subscription",
    "Timestamp",
    "Ticker",
    "TimeInForce",
    "Trade",
    "UpbitMarketEvent",
    "UpbitRegion",
]
