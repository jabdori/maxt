from __future__ import annotations

from decimal import Decimal
from importlib import import_module
from typing import Any, Awaitable, Callable, Optional, TypeVar, Union

from ._api import AccountStream, Adapter, MarketStream, StreamError, StreamEvent
from ._generated_delegate import _GeneratedNativeClientDelegateApi
from .models import (
    AccountEvent,
    Balance,
    BinanceMarket,
    BinanceSpotOrderDetail,
    BinanceSymbolFilters,
    BithumbApiKey,
    BithumbAssetFee,
    BithumbMarketAlert,
    BithumbNotice,
    BithumbPendingOrdersRequest,
    Candle,
    CandleRequest,
    Exchange,
    Feature,
    FundingPayment,
    FundingRate,
    HyperliquidAssetContext,
    HyperliquidLedgerEntry,
    HistoryRequest,
    Cursor,
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
    Network,
    UpbitDepositInfo,
    Trade,
    UpbitMarketEvent,
    UpbitOrderBookInstrument,
    UpbitRegion,
    UpbitYearCandle,
    _decimal_to_wire,
    _model_from_wire,
)


T = TypeVar("T")


def _api_module() -> Any:
    return import_module("maxt._api")


def _attribute(value: Any, name: str) -> Any:
    result = getattr(value, name)
    return result() if callable(result) else result


def _decode_market_event(value: dict[str, Any]) -> MarketEvent:
    kind = value["kind"]
    model = {
        "trade": "Trade",
        "order_book": "OrderBook",
        "ticker": "Ticker",
        "candle": "Candle",
    }.get(kind)
    return MarketEvent(
        kind,
        _model_from_wire(model, value["value"]) if model is not None else None,
    )


def _decode_account_event(value: dict[str, Any]) -> AccountEvent:
    kind = value["kind"]
    model = {"balance": "Balance", "order": "Order"}.get(kind)
    return AccountEvent(
        kind,
        _model_from_wire(model, value["value"]) if model is not None else None,
    )


def _decode_stream_item(value: dict[str, Any], *, account: bool) -> Any:
    if value["kind"] == "error":
        return StreamError(_api_module()._error_from_wire(value["error"]))
    event = value["event"]
    return StreamEvent(
        _decode_account_event(event) if account else _decode_market_event(event)
    )


def _public_native_error(native: Any, error: Exception) -> Optional[Exception]:
    native_error = getattr(native, "MaxtError", None)
    if native_error is None or not isinstance(error, native_error):
        return None
    wire = getattr(error, "wire", None)
    if isinstance(wire, dict):
        return _api_module()._error_from_wire(wire)
    return _api_module().AdapterError(str(error))


class _DecodedStream:
    def __init__(self, source: Any, *, account: bool, native: Any) -> None:
        self._source = source.__aiter__()
        self._account = account
        self._native_module = native

    def __aiter__(self) -> _DecodedStream:
        return self

    async def __anext__(self) -> Any:
        value = await self._source.__anext__()
        return _decode_stream_item(value, account=self._account)

    async def aclose(self) -> None:
        close = getattr(self._source, "aclose", None)
        if close is not None:
            try:
                await close()
            except Exception as error:
                public_error = _public_native_error(self._native_module, error)
                if public_error is None:
                    raise
                raise public_error from None


class _NativeClientDelegate(_GeneratedNativeClientDelegateApi, Adapter):
    def __init__(self, client: Any, native: Any) -> None:
        self._native_module = native
        self._client = client

    @property
    def exchange(self) -> Exchange:
        return Exchange(_attribute(self._client, "exchange"))

    @property
    def features(self) -> frozenset[Feature]:
        return frozenset(feature for feature in Feature if self._client.supports(feature))

    async def _call(self, method: Callable[..., Awaitable[T]], *args: Any) -> T:
        try:
            return await method(*args)
        except Exception as error:
            public_error = _public_native_error(self._native_module, error)
            if public_error is None:
                raise
            raise public_error from None

    def _market_stream(
        self, source: Any
    ) -> MarketStream[Union[StreamEvent[MarketEvent], StreamError]]:
        return MarketStream(
            _DecodedStream(source, account=False, native=self._native_module)
        )

    def _account_stream(
        self, source: Any
    ) -> AccountStream[Union[StreamEvent[AccountEvent], StreamError]]:
        return AccountStream(
            _DecodedStream(source, account=True, native=self._native_module)
        )


class _NativeAdapter(_NativeClientDelegate):
    def __init__(self, handle: Any, native: Any) -> None:
        self._handle = handle
        super().__init__(handle.client(), native)

    @property
    def authenticated(self) -> bool:
        return bool(_attribute(self._handle, "authenticated"))


def _client_delegate(adapter: Adapter) -> Adapter:
    if isinstance(adapter, _NativeAdapter):
        return adapter
    native = _api_module()._load_native()
    return _NativeClientDelegate(native.NativeClient.from_adapter(adapter), native)


class UpbitAdapter(_NativeAdapter):
    def __init__(
        self,
        *,
        region: UpbitRegion = UpbitRegion.KOREA,
        access_key: Optional[str] = None,
        secret_key: Optional[str] = None,
    ) -> None:
        native = _api_module()._load_native()
        handle = native.NativeUpbitAdapter(
            region=region.value,
            access_key=access_key,
            secret_key=secret_key,
        )
        super().__init__(handle, native)

    @property
    def region(self) -> UpbitRegion:
        return UpbitRegion(_attribute(self._handle, "region"))

    async def order_books(
        self,
        markets: list[Market],
        depth: Optional[int] = None,
    ) -> list[OrderBook]:
        values = await self._call(self._handle.order_books, markets, depth)
        return [_model_from_wire("OrderBook", value) for value in values]

    async def order_books_at_level(
        self,
        markets: list[Market],
        level: Decimal,
        depth: Optional[int] = None,
    ) -> list[OrderBook]:
        values = await self._call(
            self._handle.order_books_at_level,
            markets,
            _decimal_to_wire(level),
            depth,
        )
        return [_model_from_wire("OrderBook", value) for value in values]

    async def tickers(self, markets: list[Market]) -> list[Ticker]:
        values = await self._call(self._handle.tickers, markets)
        return [_model_from_wire("Ticker", value) for value in values]

    async def tickers_by_quote(self, quote_currencies: list[str]) -> list[Ticker]:
        values = await self._call(self._handle.tickers_by_quote, quote_currencies)
        return [_model_from_wire("Ticker", value) for value in values]

    async def year_candles(
        self,
        market: Market,
        to: Optional[int] = None,
        count: Optional[int] = None,
    ) -> list[UpbitYearCandle]:
        values = await self._call(self._handle.year_candles, market, to, count)
        return [_model_from_wire("UpbitYearCandle", value) for value in values]

    async def orderbook_instruments(
        self,
        markets: list[Market],
    ) -> list[UpbitOrderBookInstrument]:
        values = await self._call(self._handle.orderbook_instruments, markets)
        return [
            _model_from_wire("UpbitOrderBookInstrument", value) for value in values
        ]

    async def market_events(self) -> list[tuple[Market, UpbitMarketEvent]]:
        values = await self._call(self._handle.market_events)
        return [
            (Market.from_wire(value["market"]), UpbitMarketEvent.from_wire(value))
            for value in values
        ]

    async def test_order(self, request: OrderRequest) -> Order:
        """Validate an Upbit order without creating it.

        The returned order is a dry-run result. Its ID cannot be queried or
        cancelled, and its status does not represent a live order.
        """
        value = await self._call(self._handle.test_order, request.to_wire())
        return _model_from_wire("Order", value)

    async def deposit_info(
        self,
        asset: str,
        network: Network,
    ) -> UpbitDepositInfo:
        """Return Upbit's non-real-time deposit availability metadata."""
        value = await self._call(self._handle.deposit_info, asset, network)
        return _model_from_wire("UpbitDepositInfo", value)


class BithumbAdapter(_NativeAdapter):
    def __init__(
        self,
        *,
        access_key: Optional[str] = None,
        secret_key: Optional[str] = None,
    ) -> None:
        native = _api_module()._load_native()
        handle = native.NativeBithumbAdapter(
            access_key=access_key,
            secret_key=secret_key,
        )
        super().__init__(handle, native)

    async def market_warnings(self) -> list[tuple[Market, str]]:
        values = await self._call(self._handle.market_warnings)
        return [
            (Market.from_wire(value["market"]), value["warning"])
            for value in values
        ]

    async def market_alerts(self) -> list[tuple[Market, BithumbMarketAlert]]:
        values = await self._call(self._handle.market_alerts)
        return [
            (Market.from_wire(value["market"]), BithumbMarketAlert.from_wire(value))
            for value in values
        ]

    async def notices(self, count: Optional[int] = None) -> list[BithumbNotice]:
        values = await self._call(self._handle.notices, count)
        return [_model_from_wire("BithumbNotice", value) for value in values]

    async def transfer_fees(self, currency: str) -> list[BithumbAssetFee]:
        values = await self._call(self._handle.transfer_fees, currency)
        return [_model_from_wire("BithumbAssetFee", value) for value in values]

    async def api_keys(self) -> list[BithumbApiKey]:
        values = await self._call(self._handle.api_keys)
        return [_model_from_wire("BithumbApiKey", value) for value in values]

    async def pending_orders(
        self, request: BithumbPendingOrdersRequest
    ) -> Page[Order]:
        value = await self._call(self._handle.pending_orders, request.to_wire())
        return Page(
            [_model_from_wire("Order", item) for item in value["items"]],
            Cursor(value["next"]) if value.get("next") is not None else None,
        )


class BinanceListenKey:
    _handle: Any

    def __init__(self) -> None:
        self._handle = None
        raise TypeError(
            "BinanceListenKey values come from "
            "BinanceAdapter.usd_m_create_listen_key"
        )

    @classmethod
    def _from_native(cls, handle: Any) -> BinanceListenKey:
        value = object.__new__(cls)
        value._handle = handle
        return value

    @property
    def value(self) -> str:
        return str(_attribute(self._handle, "value"))

    def __repr__(self) -> str:
        return "BinanceListenKey(<redacted>)"


class BinanceAdapter(_NativeAdapter):
    def __init__(
        self,
        *,
        venue: BinanceMarket = BinanceMarket.SPOT,
        api_key: Optional[str] = None,
        secret_key: Optional[str] = None,
    ) -> None:
        native = _api_module()._load_native()
        handle = native.NativeBinanceAdapter(
            venue=venue.value,
            api_key=api_key,
            secret_key=secret_key,
        )
        super().__init__(handle, native)

    @classmethod
    def spot(
        cls,
        *,
        api_key: Optional[str] = None,
        secret_key: Optional[str] = None,
    ) -> BinanceAdapter:
        return cls(
            venue=BinanceMarket.SPOT,
            api_key=api_key,
            secret_key=secret_key,
        )

    @classmethod
    def usd_m_futures(
        cls,
        *,
        api_key: Optional[str] = None,
        secret_key: Optional[str] = None,
    ) -> BinanceAdapter:
        return cls(
            venue=BinanceMarket.USD_M_FUTURES,
            api_key=api_key,
            secret_key=secret_key,
        )

    @property
    def venue(self) -> BinanceMarket:
        return BinanceMarket(_attribute(self._handle, "venue"))

    async def spot_symbol_filters(self, market: Market) -> BinanceSymbolFilters:
        value = await self._call(self._handle.spot_symbol_filters, market)
        return BinanceSymbolFilters.from_wire(value)

    async def spot_order(
        self,
        market: Market,
        order_id: str,
    ) -> BinanceSpotOrderDetail:
        value = await self._call(self._handle.spot_order, market, order_id)
        return BinanceSpotOrderDetail.from_wire(value)

    async def usd_m_create_listen_key(self) -> BinanceListenKey:
        handle = await self._call(self._handle.usd_m_create_listen_key)
        return BinanceListenKey._from_native(handle)

    async def usd_m_keepalive_listen_key(self) -> None:
        await self._call(self._handle.usd_m_keepalive_listen_key)

    async def usd_m_close_listen_key(self) -> None:
        await self._call(self._handle.usd_m_close_listen_key)


class HyperliquidAdapter(_NativeAdapter):
    def __init__(
        self,
        *,
        testnet: bool = False,
        address: Optional[str] = None,
        private_key: Optional[str] = None,
    ) -> None:
        native = _api_module()._load_native()
        handle = native.NativeHyperliquidAdapter(
            testnet=testnet,
            address=address,
            private_key=private_key,
        )
        super().__init__(handle, native)

    @classmethod
    def testnet(
        cls,
        *,
        address: Optional[str] = None,
        private_key: Optional[str] = None,
    ) -> HyperliquidAdapter:
        return cls(testnet=True, address=address, private_key=private_key)

    @property
    def is_testnet(self) -> bool:
        return bool(_attribute(self._handle, "is_testnet"))

    async def non_funding_ledger(
        self,
        from_: Optional[int] = None,
        to: Optional[int] = None,
        cursor: Optional[Cursor] = None,
        limit: Optional[int] = None,
    ) -> Page[HyperliquidLedgerEntry]:
        value = await self._call(
            self._handle.non_funding_ledger,
            from_,
            to,
            str(cursor) if cursor is not None else None,
            limit,
        )
        return Page(
            [HyperliquidLedgerEntry.from_wire(item) for item in value["items"]],
            Cursor(value["next"]) if value.get("next") is not None else None,
        )

    async def asset_context(self, market: Market) -> HyperliquidAssetContext:
        value = await self._call(self._handle.asset_context, market)
        return HyperliquidAssetContext.from_wire(value)


__all__ = [
    "BinanceAdapter",
    "BinanceListenKey",
    "BithumbAdapter",
    "HyperliquidAdapter",
    "UpbitAdapter",
]
