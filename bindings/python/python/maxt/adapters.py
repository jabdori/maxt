from __future__ import annotations

from decimal import Decimal
from importlib import import_module
from typing import Any, Awaitable, Callable, Optional, TypeVar, Union

from ._api import (
    AccountStream,
    Adapter,
    HyperliquidAccountStream,
    HyperliquidMarketStream,
    MarketStream,
    StreamError,
    StreamEvent,
)
from ._generated_delegate import _GeneratedNativeClientDelegateApi
from .models import (
    AccountEvent,
    Balance,
    CancelOrdersResult,
    BinanceAccountTrade,
    BinanceMarket,
    BinanceAggregateTrade,
    BinanceAggregateTradesRequest,
    BinanceC2cTradeHistoryPage,
    BinanceC2cTradeHistoryRequest,
    BinanceMarkPrice,
    BinanceOpenInterest,
    BinanceSpotAveragePrice,
    BinanceDepositHistory,
    BinanceDepositHistoryRequest,
    BinanceWithdrawHistory,
    BinanceWithdrawHistoryRequest,
    BinanceSpotAccountInformation,
    BinanceSpotCancelAllOpenOrders,
    BinanceExchangeInfo,
    BinanceUsdMAccountInformation,
    BinanceUsdMPositionInformation,
    BinanceCoinInformation,
    BinanceApiKeyPermissions,
    BinanceQuestionnaireRequirements,
    BinanceWithdrawalAddress,
    BinanceSpotOrderDetail,
    BinanceSymbolFilters,
    BinanceTestOrder,
    BinanceTestOrderRequest,
    BithumbApiKey,
    BithumbAssetFee,
    BithumbBatchOrdersRequest,
    BithumbBatchOrdersResult,
    BithumbClosedOrder,
    BithumbClosedOrdersRequest,
    BithumbOrderDetail,
    BithumbOrderDetailRequest,
    BithumbOrderListItem,
    BithumbOrderListRequest,
    BithumbKrwDeposit,
    BithumbKrwDepositsRequest,
    BithumbKrwTransferRequest,
    BithumbKrwWithdrawal,
    BithumbKrwWithdrawalsRequest,
    BithumbMarketAlert,
    BithumbNotice,
    BithumbPendingOrdersRequest,
    BithumbTwapOrder,
    BithumbTwapOrderRequest,
    BithumbTwapOrdersRequest,
    BithumbWithdrawalAddress,
    Candle,
    CandleRequest,
    Exchange,
    Feature,
    FundingPayment,
    FundingRate,
    HyperliquidAssetContext,
    HyperliquidAccountEvent,
    HyperliquidCandleSnapshot,
    HyperliquidL2Book,
    HyperliquidRecentTrade,
    HyperliquidFundingHistoryEntry,
    HyperliquidUserFunding,
    HyperliquidSpotClearinghouseState,
    HyperliquidSpotMeta,
    HyperliquidSpotMetaAndAssetContexts,
    HyperliquidLedgerEntry,
    HyperliquidMidPrice,
    HyperliquidMarketEvent,
    HyperliquidOpenOrder,
    HyperliquidOrderInfo,
    HyperliquidOrderReference,
    HyperliquidOrderStatusResponse,
    HyperliquidPortfolioPeriod,
    HyperliquidReferral,
    HyperliquidSubAccount,
    HyperliquidUserFees,
    HyperliquidUserFill,
    HyperliquidUserRateLimit,
    HyperliquidUserRole,
    HyperliquidVaultEquity,
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
    UpbitApiKey,
    UpbitDepositInfo,
    UpbitWithdrawalAddress,
    UpbitKrwDeposit,
    UpbitKrwTransferRequest,
    UpbitKrwWithdrawal,
    UpbitTravelRuleVasp,
    UpbitTravelRuleVerification,
    UpbitBatchCancelRequest,
    UpbitCancelAndNewOrderRequest,
    UpbitCancelAndNewOrderResult,
    Trade,
    UpbitMarketEvent,
    UpbitSubscriptionList,
    UpbitOrderBookInstrument,
    UpbitClosedOrder,
    UpbitClosedOrdersRequest,
    UpbitOrderDetail,
    UpbitOrderDetailRequest,
    UpbitPocket,
    UpbitPocketApiKeyGroup,
    UpbitPocketApiKeysRequest,
    UpbitPocketBalance,
    UpbitPocketTransfer,
    UpbitPocketTransferQuery,
    UpbitPocketTransferRequest,
    UpbitPocketUniversalTransferRequest,
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


def _decode_hyperliquid_market_event(value: dict[str, Any]) -> HyperliquidMarketEvent:
    return _model_from_wire("HyperliquidMarketEvent", value)


def _decode_hyperliquid_account_event(value: dict[str, Any]) -> HyperliquidAccountEvent:
    return _model_from_wire("HyperliquidAccountEvent", value)


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
    def __init__(
        self,
        source: Any,
        *,
        account: bool,
        native: Any,
        decode: Optional[Callable[[dict[str, Any]], Any]] = None,
    ) -> None:
        self._source = source.__aiter__()
        self._account = account
        self._native_module = native
        self._decode = decode

    def __aiter__(self) -> _DecodedStream:
        return self

    async def __anext__(self) -> Any:
        value = await self._source.__anext__()
        if self._decode is not None:
            if value["kind"] == "error":
                return StreamError(_api_module()._error_from_wire(value["error"]))
            return StreamEvent(self._decode(value["event"]))
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

    def _hyperliquid_market_stream(self, source: Any) -> HyperliquidMarketStream:
        return HyperliquidMarketStream(
            _DecodedStream(
                source,
                account=False,
                native=self._native_module,
                decode=_decode_hyperliquid_market_event,
            )
        )

    def _hyperliquid_account_stream(self, source: Any) -> HyperliquidAccountStream:
        return HyperliquidAccountStream(
            _DecodedStream(
                source,
                account=True,
                native=self._native_module,
                decode=_decode_hyperliquid_account_event,
            )
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

    async def list_subscriptions(
        self,
        subscription: Subscription,
    ) -> UpbitSubscriptionList:
        value = await self._call(self._handle.list_subscriptions, subscription.to_wire())
        return _model_from_wire("UpbitSubscriptionList", value)

    async def withdrawal_addresses(self) -> list[UpbitWithdrawalAddress]:
        values = await self._call(self._handle.withdrawal_addresses)
        return [_model_from_wire("UpbitWithdrawalAddress", value) for value in values]

    async def test_order(self, request: OrderRequest) -> Order:
        """Validate an Upbit order without creating it.

        The returned order is a dry-run result. Its ID cannot be queried or
        cancelled, and its status does not represent a live order.
        """
        value = await self._call(self._handle.test_order, request.to_wire())
        return _model_from_wire("Order", value)

    async def order_detail(self, request: UpbitOrderDetailRequest) -> UpbitOrderDetail:
        value = await self._call(self._handle.order_detail, request.to_wire())
        return _model_from_wire("UpbitOrderDetail", value)

    async def closed_orders(
        self,
        request: UpbitClosedOrdersRequest,
    ) -> list[UpbitClosedOrder]:
        """Return closed-order summary rows without individual trades."""
        values = await self._call(self._handle.closed_orders, request.to_wire())
        return [_model_from_wire("UpbitClosedOrder", value) for value in values]

    async def deposit_info(
        self,
        asset: str,
        network: Network,
    ) -> UpbitDepositInfo:
        """Return Upbit's non-real-time deposit availability metadata."""
        value = await self._call(self._handle.deposit_info, asset, network)
        return _model_from_wire("UpbitDepositInfo", value)

    async def travel_rule_vasps(self) -> list[UpbitTravelRuleVasp]:
        """List supported Travel Rule VASPs for Upbit Korea or Singapore."""
        values = await self._call(self._handle.travel_rule_vasps)
        return [_model_from_wire("UpbitTravelRuleVasp", value) for value in values]

    async def verify_travel_rule_by_uuid(
        self,
        deposit_uuid: str,
        vasp_uuid: str,
    ) -> UpbitTravelRuleVerification:
        """Request Travel Rule verification by Upbit deposit UUID."""
        value = await self._call(
            self._handle.verify_travel_rule_by_uuid,
            deposit_uuid,
            vasp_uuid,
        )
        return _model_from_wire("UpbitTravelRuleVerification", value)

    async def verify_travel_rule_by_txid(
        self,
        txid: str,
        vasp_uuid: str,
        currency: str,
        net_type: str,
    ) -> UpbitTravelRuleVerification:
        """Request Travel Rule verification by transaction ID."""
        value = await self._call(
            self._handle.verify_travel_rule_by_txid,
            txid,
            vasp_uuid,
            currency,
            net_type,
        )
        return _model_from_wire("UpbitTravelRuleVerification", value)

    async def batch_cancel_open_orders(
        self,
        request: UpbitBatchCancelRequest,
    ) -> CancelOrdersResult:
        """Cancel matching Upbit `wait` orders with an explicit market scope.

        `UpbitBatchCancelScope.all()` selects every eligible market, but Upbit
        still applies the request count (default 20, maximum 300). The result
        can contain both completed and failed cancellations when orders change
        state in flight.
        """
        value = await self._call(
            self._handle.batch_cancel_open_orders,
            request.to_wire(),
        )
        return _model_from_wire("CancelOrdersResult", value)

    async def cancel_and_new_order(
        self,
        request: UpbitCancelAndNewOrderRequest,
    ) -> UpbitCancelAndNewOrderResult:
        """Cancel an existing order and conditionally place its replacement.

        This is a financial write. Upbit can return no replacement UUID when
        the previous order changes state during the cancellation race.
        """
        value = await self._call(
            self._handle.cancel_and_new_order,
            request.to_wire(),
        )
        return _model_from_wire("UpbitCancelAndNewOrderResult", value)

    async def deposit_krw(
        self,
        request: UpbitKrwTransferRequest,
    ) -> UpbitKrwDeposit:
        """Request an Upbit Korea KRW deposit; this is a financial write."""
        value = await self._call(self._handle.deposit_krw, request.to_wire())
        return _model_from_wire("UpbitKrwDeposit", value)

    async def withdraw_krw(
        self,
        request: UpbitKrwTransferRequest,
    ) -> UpbitKrwWithdrawal:
        """Request an Upbit Korea KRW withdrawal; this is a financial write."""
        value = await self._call(self._handle.withdraw_krw, request.to_wire())
        return _model_from_wire("UpbitKrwWithdrawal", value)

    async def api_keys(self) -> list[UpbitApiKey]:
        values = await self._call(self._handle.api_keys)
        return [_model_from_wire("UpbitApiKey", value) for value in values]

    async def list_pockets(self) -> list[UpbitPocket]:
        values = await self._call(self._handle.list_pockets)
        return [_model_from_wire("UpbitPocket", value) for value in values]

    async def list_pocket_api_keys(
        self,
        request: UpbitPocketApiKeysRequest,
    ) -> list[UpbitPocketApiKeyGroup]:
        values = await self._call(
            self._handle.list_pocket_api_keys,
            request.to_wire(),
        )
        return [_model_from_wire("UpbitPocketApiKeyGroup", value) for value in values]

    async def sub_pocket_balances(self, pocket_uuid: str) -> list[UpbitPocketBalance]:
        values = await self._call(self._handle.sub_pocket_balances, pocket_uuid)
        return [_model_from_wire("UpbitPocketBalance", value) for value in values]

    async def universal_transfer(
        self,
        request: UpbitPocketUniversalTransferRequest,
    ) -> UpbitPocketTransfer:
        """Move assets between Upbit pockets; this is a financial write."""
        value = await self._call(self._handle.universal_transfer, request.to_wire())
        return _model_from_wire("UpbitPocketTransfer", value)

    async def universal_transfers(
        self,
        request: UpbitPocketTransferQuery,
    ) -> list[UpbitPocketTransfer]:
        values = await self._call(self._handle.universal_transfers, request.to_wire())
        return [_model_from_wire("UpbitPocketTransfer", value) for value in values]

    async def sub_pocket_transfer(
        self,
        request: UpbitPocketTransferRequest,
    ) -> UpbitPocketTransfer:
        """Move assets from an Upbit sub-pocket; this is a financial write."""
        value = await self._call(self._handle.sub_pocket_transfer, request.to_wire())
        return _model_from_wire("UpbitPocketTransfer", value)

    async def sub_pocket_transfers(
        self,
        request: UpbitPocketTransferQuery,
    ) -> list[UpbitPocketTransfer]:
        values = await self._call(self._handle.sub_pocket_transfers, request.to_wire())
        return [_model_from_wire("UpbitPocketTransfer", value) for value in values]


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

    async def withdrawal_addresses(self) -> list[BithumbWithdrawalAddress]:
        values = await self._call(self._handle.withdrawal_addresses)
        return [
            _model_from_wire("BithumbWithdrawalAddress", value) for value in values
        ]

    async def order_detail(
        self,
        request: BithumbOrderDetailRequest,
    ) -> BithumbOrderDetail:
        value = await self._call(self._handle.order_detail, request.to_wire())
        return _model_from_wire("BithumbOrderDetail", value)

    async def order_list(
        self,
        request: BithumbOrderListRequest,
    ) -> list[BithumbOrderListItem]:
        values = await self._call(self._handle.order_list, request.to_wire())
        return [_model_from_wire("BithumbOrderListItem", value) for value in values]

    async def krw_withdrawals(
        self,
        request: BithumbKrwWithdrawalsRequest,
    ) -> list[BithumbKrwWithdrawal]:
        values = await self._call(self._handle.krw_withdrawals, request.to_wire())
        return [_model_from_wire("BithumbKrwWithdrawal", value) for value in values]

    async def withdraw_krw(
        self,
        request: BithumbKrwTransferRequest,
    ) -> BithumbKrwWithdrawal:
        """Request a KRW withdrawal; this is a financial write."""
        value = await self._call(self._handle.withdraw_krw, request.to_wire())
        return _model_from_wire("BithumbKrwWithdrawal", value)

    async def krw_deposits(
        self,
        request: BithumbKrwDepositsRequest,
    ) -> list[BithumbKrwDeposit]:
        values = await self._call(self._handle.krw_deposits, request.to_wire())
        return [_model_from_wire("BithumbKrwDeposit", value) for value in values]

    async def deposit_krw(
        self,
        request: BithumbKrwTransferRequest,
    ) -> BithumbKrwDeposit:
        """Request a KRW deposit; this is a financial write."""
        value = await self._call(self._handle.deposit_krw, request.to_wire())
        return _model_from_wire("BithumbKrwDeposit", value)

    async def pending_orders(
        self, request: BithumbPendingOrdersRequest
    ) -> Page[Order]:
        value = await self._call(self._handle.pending_orders, request.to_wire())
        return Page(
            [_model_from_wire("Order", item) for item in value["items"]],
            Cursor(value["next"]) if value.get("next") is not None else None,
        )

    async def closed_orders(
        self, request: BithumbClosedOrdersRequest
    ) -> Page[BithumbClosedOrder]:
        value = await self._call(self._handle.closed_orders, request.to_wire())
        return Page(
            [_model_from_wire("BithumbClosedOrder", item) for item in value["items"]],
            Cursor(value["next"]) if value.get("next") is not None else None,
        )

    async def twap_orders(
        self, request: BithumbTwapOrdersRequest
    ) -> Page[BithumbTwapOrder]:
        value = await self._call(self._handle.twap_orders, request.to_wire())
        return Page(
            [_model_from_wire("BithumbTwapOrder", item) for item in value["items"]],
            Cursor(value["next"]) if value.get("next") is not None else None,
        )

    async def create_twap_order(self, request: BithumbTwapOrderRequest) -> str:
        """Create a TWAP order; this submits a financial write to Bithumb."""
        return await self._call(self._handle.create_twap_order, request.to_wire())

    async def cancel_twap_order(self, algo_order_id: str) -> str:
        """Cancel a TWAP order; this submits a financial write to Bithumb."""
        return await self._call(self._handle.cancel_twap_order, algo_order_id)

    async def batch_orders(
        self,
        request: BithumbBatchOrdersRequest,
    ) -> BithumbBatchOrdersResult:
        """Submit up to 20 independent Bithumb orders.

        This is a financial write. The result preserves accepted and rejected
        outcomes in the provider response order.
        """
        value = await self._call(self._handle.batch_orders, request.to_wire())
        return _model_from_wire("BithumbBatchOrdersResult", value)

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

    async def spot_average_price(self, market: Market) -> BinanceSpotAveragePrice:
        value = await self._call(self._handle.spot_average_price, market)
        return BinanceSpotAveragePrice.from_wire(value)

    async def spot_account_information(self) -> BinanceSpotAccountInformation:
        value = await self._call(self._handle.spot_account_information)
        return _model_from_wire("BinanceSpotAccountInformation", value)

    async def spot_cancel_all_open_orders(
        self, market: Market
    ) -> BinanceSpotCancelAllOpenOrders:
        value = await self._call(self._handle.spot_cancel_all_open_orders, market)
        return _model_from_wire("BinanceSpotCancelAllOpenOrders", value)

    async def spot_exchange_info(self) -> BinanceExchangeInfo:
        value = await self._call(self._handle.spot_exchange_info)
        return _model_from_wire("BinanceExchangeInfo", value)

    async def usd_m_account_information(self) -> BinanceUsdMAccountInformation:
        value = await self._call(self._handle.usd_m_account_information)
        return _model_from_wire("BinanceUsdMAccountInformation", value)

    async def usd_m_exchange_info(self) -> BinanceExchangeInfo:
        value = await self._call(self._handle.usd_m_exchange_info)
        return _model_from_wire("BinanceExchangeInfo", value)

    async def usd_m_position_information(
        self, market: Optional[Market] = None
    ) -> list[BinanceUsdMPositionInformation]:
        values = await self._call(self._handle.usd_m_position_information, market)
        return [
            _model_from_wire("BinanceUsdMPositionInformation", value)
            for value in values
        ]

    async def all_coins_information(self) -> list[BinanceCoinInformation]:
        values = await self._call(self._handle.all_coins_information)
        return [_model_from_wire("BinanceCoinInformation", value) for value in values]

    async def api_key_permissions(self) -> BinanceApiKeyPermissions:
        value = await self._call(self._handle.api_key_permissions)
        return _model_from_wire("BinanceApiKeyPermissions", value)

    async def deposit_history(
        self, request: BinanceDepositHistoryRequest
    ) -> BinanceDepositHistory:
        value = await self._call(self._handle.deposit_history, request.to_wire())
        return _model_from_wire("BinanceDepositHistory", value)

    async def questionnaire_requirements(self) -> BinanceQuestionnaireRequirements:
        value = await self._call(self._handle.questionnaire_requirements)
        return _model_from_wire("BinanceQuestionnaireRequirements", value)

    async def withdraw_address_list(self) -> list[BinanceWithdrawalAddress]:
        values = await self._call(self._handle.withdraw_address_list)
        return [_model_from_wire("BinanceWithdrawalAddress", value) for value in values]

    async def withdraw_history(
        self, request: BinanceWithdrawHistoryRequest
    ) -> BinanceWithdrawHistory:
        value = await self._call(self._handle.withdraw_history, request.to_wire())
        return _model_from_wire("BinanceWithdrawHistory", value)

    async def mark_price(self, market: Market) -> BinanceMarkPrice:
        value = await self._call(self._handle.mark_price, market)
        return BinanceMarkPrice.from_wire(value)

    async def mark_prices(self) -> list[BinanceMarkPrice]:
        values = await self._call(self._handle.mark_prices)
        return [BinanceMarkPrice.from_wire(value) for value in values]

    async def open_interest(self, market: Market) -> BinanceOpenInterest:
        value = await self._call(self._handle.open_interest, market)
        return BinanceOpenInterest.from_wire(value)

    async def aggregate_trades(
        self,
        request: BinanceAggregateTradesRequest,
    ) -> list[BinanceAggregateTrade]:
        values = await self._call(
            self._handle.aggregate_trades,
            request.to_wire(),
        )
        return [BinanceAggregateTrade.from_wire(value) for value in values]

    async def account_trades(
        self,
        request: HistoryRequest,
    ) -> Page[BinanceAccountTrade]:
        value = await self._call(self._handle.account_trades, request.to_wire())
        return Page(
            [_model_from_wire("BinanceAccountTrade", item) for item in value["items"]],
            Cursor(value["next"]) if value.get("next") is not None else None,
        )

    async def c2c_trade_history(
        self,
        request: BinanceC2cTradeHistoryRequest,
    ) -> BinanceC2cTradeHistoryPage:
        value = await self._call(self._handle.c2c_trade_history, request.to_wire())
        return _model_from_wire("BinanceC2cTradeHistoryPage", value)

    async def test_order(self, request: BinanceTestOrderRequest) -> BinanceTestOrder:
        """Validate a Binance order without creating it."""
        value = await self._call(self._handle.test_order, request.to_wire())
        return _model_from_wire("BinanceTestOrder", value)

    async def cancel_all_open_orders(self, market: Market) -> None:
        """Cancel all open orders for one Binance market; this is a financial write."""
        await self._call(self._handle.cancel_all_open_orders, market)

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

    async def candle_snapshot(
        self,
        market: Market,
        interval: str,
        from_ns: int,
        to_ns: Optional[int] = None,
    ) -> list[HyperliquidCandleSnapshot]:
        values = await self._call(
            self._handle.candle_snapshot, market, interval, from_ns, to_ns
        )
        return [_model_from_wire("HyperliquidCandleSnapshot", value) for value in values]

    async def l2_book(self, market: Market) -> HyperliquidL2Book:
        value = await self._call(self._handle.l2_book, market)
        return _model_from_wire("HyperliquidL2Book", value)

    async def recent_trades(self, market: Market) -> list[HyperliquidRecentTrade]:
        values = await self._call(self._handle.recent_trades, market)
        return [_model_from_wire("HyperliquidRecentTrade", value) for value in values]

    async def funding_history(
        self, market: Market, from_ns: int, to_ns: Optional[int] = None
    ) -> list[HyperliquidFundingHistoryEntry]:
        values = await self._call(
            self._handle.funding_history, market, from_ns, to_ns
        )
        return [
            _model_from_wire("HyperliquidFundingHistoryEntry", value)
            for value in values
        ]

    async def user_funding(
        self, from_ns: int, to_ns: Optional[int] = None
    ) -> list[HyperliquidUserFunding]:
        values = await self._call(self._handle.user_funding, from_ns, to_ns)
        return [_model_from_wire("HyperliquidUserFunding", value) for value in values]

    async def spot_clearinghouse_state(self) -> HyperliquidSpotClearinghouseState:
        value = await self._call(self._handle.spot_clearinghouse_state)
        return _model_from_wire("HyperliquidSpotClearinghouseState", value)

    async def spot_meta(self) -> HyperliquidSpotMeta:
        value = await self._call(self._handle.spot_meta)
        return _model_from_wire("HyperliquidSpotMeta", value)

    async def spot_meta_and_asset_contexts(
        self,
    ) -> HyperliquidSpotMetaAndAssetContexts:
        value = await self._call(self._handle.spot_meta_and_asset_contexts)
        return _model_from_wire("HyperliquidSpotMetaAndAssetContexts", value)

    async def all_mids(self) -> list[HyperliquidMidPrice]:
        values = await self._call(self._handle.all_mids)
        return [HyperliquidMidPrice.from_wire(value) for value in values]

    async def subscribe_detailed(
        self,
        subscription: Subscription,
    ) -> HyperliquidMarketStream:
        source = await self._call(self._handle.subscribe_detailed, subscription.to_wire())
        return self._hyperliquid_market_stream(source)

    async def subscribe_detailed_with(
        self,
        subscription: Subscription,
        config: StreamConfig,
    ) -> HyperliquidMarketStream:
        source = await self._call(
            self._handle.subscribe_detailed_with,
            subscription.to_wire(),
            config.to_wire(),
        )
        return self._hyperliquid_market_stream(source)

    async def subscribe_detailed_account(self) -> HyperliquidAccountStream:
        source = await self._call(self._handle.subscribe_detailed_account)
        return self._hyperliquid_account_stream(source)

    async def subscribe_detailed_account_with(
        self,
        config: StreamConfig,
    ) -> HyperliquidAccountStream:
        source = await self._call(
            self._handle.subscribe_detailed_account_with,
            config.to_wire(),
        )
        return self._hyperliquid_account_stream(source)

    async def basic_open_orders(self) -> list[HyperliquidOpenOrder]:
        values = await self._call(self._handle.basic_open_orders)
        return [_model_from_wire("HyperliquidOpenOrder", value) for value in values]

    async def order_status(
        self,
        reference: HyperliquidOrderReference,
    ) -> HyperliquidOrderStatusResponse:
        value = await self._call(self._handle.order_status, reference.to_wire())
        return HyperliquidOrderStatusResponse.from_wire(value)

    async def historical_orders(self) -> list[HyperliquidOrderInfo]:
        values = await self._call(self._handle.historical_orders)
        return [_model_from_wire("HyperliquidOrderInfo", value) for value in values]

    async def user_fills(
        self, aggregate_by_time: bool
    ) -> list[HyperliquidUserFill]:
        values = await self._call(self._handle.user_fills, aggregate_by_time)
        return [_model_from_wire("HyperliquidUserFill", value) for value in values]

    async def user_fills_by_time(
        self,
        from_ns: int,
        to_ns: Optional[int] = None,
        aggregate_by_time: bool = False,
    ) -> list[HyperliquidUserFill]:
        values = await self._call(
            self._handle.user_fills_by_time,
            from_ns,
            to_ns,
            aggregate_by_time,
        )
        return [_model_from_wire("HyperliquidUserFill", value) for value in values]

    async def user_rate_limit(self) -> HyperliquidUserRateLimit:
        value = await self._call(self._handle.user_rate_limit)
        return _model_from_wire("HyperliquidUserRateLimit", value)

    async def user_role(self) -> HyperliquidUserRole:
        value = await self._call(self._handle.user_role)
        return HyperliquidUserRole.from_wire(value)

    async def referral(self) -> HyperliquidReferral:
        value = await self._call(self._handle.referral)
        return _model_from_wire("HyperliquidReferral", value)

    async def user_fees(self) -> HyperliquidUserFees:
        value = await self._call(self._handle.user_fees)
        return _model_from_wire("HyperliquidUserFees", value)

    async def portfolio(self) -> list[HyperliquidPortfolioPeriod]:
        values = await self._call(self._handle.portfolio)
        return [
            _model_from_wire("HyperliquidPortfolioPeriod", value) for value in values
        ]

    async def sub_accounts(self) -> list[HyperliquidSubAccount]:
        values = await self._call(self._handle.sub_accounts)
        return [_model_from_wire("HyperliquidSubAccount", value) for value in values]

    async def user_vault_equities(self) -> list[HyperliquidVaultEquity]:
        values = await self._call(self._handle.user_vault_equities)
        return [
            _model_from_wire("HyperliquidVaultEquity", value) for value in values
        ]


__all__ = [
    "BinanceAdapter",
    "BinanceListenKey",
    "BithumbAdapter",
    "HyperliquidAdapter",
    "UpbitAdapter",
]
