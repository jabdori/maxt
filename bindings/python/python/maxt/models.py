from __future__ import annotations

from calendar import monthrange
from dataclasses import dataclass, field, fields, is_dataclass
from datetime import datetime, timedelta, timezone
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


class Exchange(str, Enum):
    UPBIT = "upbit"
    BITHUMB = "bithumb"
    BINANCE = "binance"
    HYPERLIQUID = "hyperliquid"

    def display_name(self) -> str:
        return {
            Exchange.UPBIT: "Upbit",
            Exchange.BITHUMB: "Bithumb",
            Exchange.BINANCE: "Binance",
            Exchange.HYPERLIQUID: "Hyperliquid",
        }[self]


class Feature(str, Enum):
    MARKETS = "markets"
    TRADES = "trades"
    ORDER_BOOK = "order_book"
    TICKER = "ticker"
    CANDLES = "candles"
    TRADE_STREAM = "trade_stream"
    ORDER_BOOK_STREAM = "order_book_stream"
    TICKER_STREAM = "ticker_stream"
    CANDLE_STREAM = "candle_stream"
    BALANCES = "balances"
    OPEN_ORDERS = "open_orders"
    ACCOUNT_STREAM = "account_stream"
    TRADING = "trading"
    POSITIONS = "positions"
    MARGIN = "margin"
    FUNDING_RATES = "funding_rates"
    FUNDING_PAYMENTS = "funding_payments"
    MARGIN_CONFIG = "margin_config"
    REDUCE_ONLY_ORDERS = "reduce_only_orders"

    def needs_credentials(self) -> bool:
        return self in {
            Feature.BALANCES,
            Feature.OPEN_ORDERS,
            Feature.ACCOUNT_STREAM,
            Feature.TRADING,
            Feature.POSITIONS,
            Feature.MARGIN,
            Feature.FUNDING_PAYMENTS,
            Feature.MARGIN_CONFIG,
            Feature.REDUCE_ONLY_ORDERS,
        }

    def is_derivatives_only(self) -> bool:
        return self in {
            Feature.POSITIONS,
            Feature.MARGIN,
            Feature.FUNDING_RATES,
            Feature.FUNDING_PAYMENTS,
            Feature.MARGIN_CONFIG,
            Feature.REDUCE_ONLY_ORDERS,
        }


class MarketKind(str, Enum):
    SPOT = "spot"
    PERPETUAL = "perpetual"

    def is_derivative(self) -> bool:
        return self is MarketKind.PERPETUAL


class MarketStatus(str, Enum):
    ACTIVE = "active"
    PAUSED = "paused"
    DELISTED = "delisted"
    UNKNOWN = "unknown"


class Side(str, Enum):
    BUY = "buy"
    SELL = "sell"

    def flip(self) -> Side:
        return Side.SELL if self is Side.BUY else Side.BUY


class Interval(str, Enum):
    SEC1 = "sec1"
    MIN1 = "min1"
    MIN3 = "min3"
    MIN5 = "min5"
    MIN15 = "min15"
    MIN30 = "min30"
    HOUR1 = "hour1"
    HOUR2 = "hour2"
    HOUR4 = "hour4"
    HOUR8 = "hour8"
    HOUR12 = "hour12"
    DAY1 = "day1"
    DAY3 = "day3"
    WEEK1 = "week1"
    MONTH1 = "month1"

    def as_secs(self) -> Optional[int]:
        return {
            Interval.SEC1: 1,
            Interval.MIN1: 60,
            Interval.MIN3: 180,
            Interval.MIN5: 300,
            Interval.MIN15: 900,
            Interval.MIN30: 1_800,
            Interval.HOUR1: 3_600,
            Interval.HOUR2: 7_200,
            Interval.HOUR4: 14_400,
            Interval.HOUR8: 28_800,
            Interval.HOUR12: 43_200,
            Interval.DAY1: 86_400,
            Interval.DAY3: 259_200,
            Interval.WEEK1: 604_800,
            Interval.MONTH1: None,
        }[self]

    def advance(self, timestamp_ns: Timestamp, count: int) -> Optional[Timestamp]:
        if not _is_i64(timestamp_ns) or not _is_i64(count):
            return None

        seconds = self.as_secs()
        if seconds is not None:
            span = seconds * 1_000_000_000 * count
            if not _is_i64(span):
                return None
            result = timestamp_ns + span
            return result if _is_i64(result) else None

        if abs(count) > (1 << 32) - 1:
            return None

        whole_seconds, nanoseconds = divmod(timestamp_ns, 1_000_000_000)
        current = _UNIX_EPOCH + timedelta(seconds=whole_seconds)
        month_index = current.year * 12 + current.month - 1 + count
        year, zero_based_month = divmod(month_index, 12)
        month = zero_based_month + 1
        if not 1 <= year <= 9999:
            return None

        moved = datetime(
            year,
            month,
            min(current.day, monthrange(year, month)[1]),
            current.hour,
            current.minute,
            current.second,
            tzinfo=timezone.utc,
        )
        elapsed = moved - _UNIX_EPOCH
        result = (
            (elapsed.days * 86_400 + elapsed.seconds) * 1_000_000_000
            + nanoseconds
        )
        return result if _is_i64(result) else None


class Overflow(str, Enum):
    BACKPRESSURE = "backpressure"
    DROP_NEWEST = "drop_newest"


class MarginMode(str, Enum):
    CROSS = "cross"
    ISOLATED = "isolated"


class OrderStatus(str, Enum):
    ACCEPTED = "accepted"
    OPEN = "open"
    PARTIALLY_FILLED = "partially_filled"
    FILLED = "filled"
    CANCELLED = "cancelled"
    REJECTED = "rejected"
    UNKNOWN = "unknown"

    def is_live(self) -> bool:
        return self in {
            OrderStatus.ACCEPTED,
            OrderStatus.OPEN,
            OrderStatus.PARTIALLY_FILLED,
        }


class OrderType(str, Enum):
    MARKET = "market"
    LIMIT = "limit"


class TimeInForce(str, Enum):
    GOOD_TIL_CANCELLED = "good_til_cancelled"
    IMMEDIATE_OR_CANCEL = "immediate_or_cancel"
    FILL_OR_KILL = "fill_or_kill"
    POST_ONLY = "post_only"


class SizeKind(str, Enum):
    BASE = "base"
    QUOTE = "quote"


class UpbitRegion(str, Enum):
    KOREA = "korea"
    SINGAPORE = "singapore"
    INDONESIA = "indonesia"
    THAILAND = "thailand"


class BithumbAlertStep(str, Enum):
    CAUTION = "caution"
    WARNING = "warning"
    DANGER = "danger"
    UNKNOWN = "unknown"


class BinanceMarket(str, Enum):
    SPOT = "spot"
    USD_M_FUTURES = "usd_m"


class HyperliquidLedgerKind(str, Enum):
    DEPOSIT = "deposit"
    WITHDRAW = "withdraw"
    INTERNAL_TRANSFER = "internal_transfer"
    SUB_ACCOUNT_TRANSFER = "sub_account_transfer"
    SPOT_TRANSFER = "spot_transfer"
    ACCOUNT_CLASS_TRANSFER = "account_class_transfer"
    VAULT_DEPOSIT = "vault_deposit"
    VAULT_WITHDRAW = "vault_withdraw"
    VAULT_DISTRIBUTION = "vault_distribution"
    LIQUIDATION = "liquidation"

    @classmethod
    def _missing_(cls, value: object) -> HyperliquidLedgerKind:
        if not isinstance(value, str):
            raise ValueError(value)
        member = str.__new__(cls, value)
        member._name_ = "OTHER"
        member._value_ = value
        return member


Timestamp = int
T = TypeVar("T")

_I64_MIN = -(1 << 63)
_I64_MAX = (1 << 63) - 1
_UNIX_EPOCH = datetime(1970, 1, 1, tzinfo=timezone.utc)


def _is_i64(value: int) -> bool:
    return _I64_MIN <= value <= _I64_MAX


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
        return cls("trade", trade)

    @classmethod
    def order_book(cls, order_book: OrderBook) -> MarketEvent:
        return cls("order_book", order_book)

    @classmethod
    def ticker(cls, ticker: Ticker) -> MarketEvent:
        return cls("ticker", ticker)

    @classmethod
    def candle(cls, candle: Candle) -> MarketEvent:
        return cls("candle", candle)

    @classmethod
    def reconnected(cls) -> MarketEvent:
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
    return model_type(
        **{
            item.name: _decode_value(
                hints[item.name],
                value[item.metadata.get("wire_name", item.name)],
            )
            for item in fields(model_type)
        }
    )


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
            item.metadata.get("wire_name", item.name): _model_to_wire(
                getattr(value, item.name)
            )
            for item in fields(value)
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
    model_types: dict[str, type[WireModel]] = {
        "Balance": Balance,
        "Candle": Candle,
        "CandleRequest": CandleRequest,
        "FundingPayment": FundingPayment,
        "FundingRate": FundingRate,
        "HistoryRequest": HistoryRequest,
        "MarginRequest": MarginRequest,
        "MarginSummary": MarginSummary,
        "MarketInfo": MarketInfo,
        "Market": Market,
        "Order": Order,
        "OrderBook": OrderBook,
        "OrderRequest": OrderRequest,
        "Position": Position,
        "StreamConfig": StreamConfig,
        "Subscription": Subscription,
        "Ticker": Ticker,
        "Trade": Trade,
    }
    try:
        model_type = model_types[type_name]
    except KeyError as error:
        raise ValueError(f"unknown maxt model: {type_name}") from error
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
    "_model_from_wire",
    "_model_to_wire",
]
