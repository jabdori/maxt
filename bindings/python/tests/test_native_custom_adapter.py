import asyncio
import io
import os
import sys
import unittest
from contextlib import redirect_stderr
from decimal import Decimal
from importlib import import_module

from maxt import (
    AccountEvent,
    AccountStream,
    Adapter,
    AdapterError,
    Balance,
    Candle,
    CandleRequest,
    Client,
    Cursor,
    DecodeError,
    Exchange,
    Feature,
    Feed,
    FundingPayment,
    FundingRate,
    HistoryRequest,
    InvalidRequestError,
    Interval,
    Level,
    MarginMode,
    MarginRequest,
    MarginSummary,
    Market,
    MarketEvent,
    MarketInfo,
    MarketKind,
    MarketStatus,
    MarketStream,
    Order,
    OrderBook,
    OrderRequest,
    OrderStatus,
    Page,
    Position,
    Side,
    Size,
    StreamConfig,
    StreamError,
    StreamEvent,
    Subscription,
    Ticker,
    Trade,
    TransferError,
    TransferErrorKind,
)


def _native_available() -> bool:
    try:
        import_module("maxt._native")
    except ImportError as error:
        if os.environ.get("MAXT_REQUIRE_NATIVE_TESTS") == "1":
            raise RuntimeError("MAXT_REQUIRE_NATIVE_TESTS needs maxt._native") from error
        return False
    return True


NATIVE_AVAILABLE = _native_available()
NATIVE_TIMEOUT = 2.0


class NativeReplayAdapter(Adapter):
    def __init__(self, market: Market) -> None:
        self.market = market
        self.received = []
        self.pending_started = asyncio.Event()
        self.pending_cleanup_started = asyncio.Event()
        self.pending_closed = asyncio.Event()
        self.trade = Trade(
            market,
            1_700_000_000_123_456_789,
            Decimal("50000.0100"),
            Decimal("0.0010"),
            Side.BUY,
            "trade-1",
        )
        self.balance = Balance("usdt", Decimal("100.2500"), Decimal("2.5000"))
        self.order = Order(
            "order-1",
            market,
            Side.BUY,
            OrderStatus.OPEN,
            Decimal("0.2500"),
            Decimal("0.7500"),
            Decimal("50000.0100"),
            1_700_000_000_123_456_790,
        )
        self.position = Position(
            market,
            Side.BUY,
            Decimal("0.1250"),
            entry_price=Decimal("49000.00"),
            mark_price=Decimal("50000.00"),
            leverage=Decimal("5"),
            margin_mode=MarginMode.CROSS,
        )

    @property
    def exchange(self) -> Exchange:
        return Exchange.BINANCE

    @property
    def features(self) -> frozenset[Feature]:
        return frozenset(Feature)

    async def markets(self, kind: MarketKind) -> list[MarketInfo]:
        self.received.append(("markets", kind))
        return [MarketInfo(self.market, "BTCUSDT", MarketStatus.ACTIVE, None, "Bitcoin")]

    async def trades(self, market: Market, limit=None) -> list[Trade]:
        self.received.append(("trades", market, limit, market is self.market))
        return [self.trade]

    async def order_book(self, market: Market, depth=None) -> OrderBook:
        self.received.append(("order_book", market, depth))
        return OrderBook(
            market,
            1_700_000_000_123_456_791,
            [Level(Decimal("50000.00"), Decimal("1.25"))],
            [Level(Decimal("50000.02"), Decimal("0.75"))],
        )

    async def ticker(self, market: Market) -> Ticker:
        self.received.append(("ticker", market))
        return Ticker(
            market,
            1_700_000_000_123_456_792,
            None,
            Decimal("50000.01"),
            None,
            None,
            None,
            None,
            None,
            None,
        )

    async def candles(self, request: CandleRequest) -> list[Candle]:
        self.received.append(("candles", request))
        return [
            Candle(
                request.market,
                request.interval,
                1_700_000_000_000_000_000,
                Decimal("49000"),
                Decimal("51000"),
                Decimal("48000"),
                Decimal("50000"),
                Decimal("12.5000"),
                None,
                True,
            )
        ]

    async def balances(self) -> list[Balance]:
        self.received.append(("balances",))
        return [self.balance]

    async def open_orders(self, market=None) -> list[Order]:
        self.received.append(("open_orders", market))
        return [self.order]

    async def place_order(self, request: OrderRequest) -> Order:
        self.received.append(("place_order", request))
        return self.order

    async def cancel_order(self, market: Market, order_id: str) -> Order:
        self.received.append(("cancel_order", market, order_id))
        return self.order

    async def positions(self, market=None) -> list[Position]:
        self.received.append(("positions", market))
        return [Position(self.market, None, Decimal("0")), self.position]

    async def margin_summary(self) -> MarginSummary:
        self.received.append(("margin_summary",))
        return MarginSummary(
            "USDT",
            Decimal("1000.00"),
            Decimal("400.00"),
            Decimal("600.00"),
        )

    async def funding_rates(self, request: HistoryRequest) -> Page[FundingRate]:
        self.received.append(("funding_rates", request))
        return Page(
            [FundingRate(self.market, 1_700_000_000_123_456_793, Decimal("0.0001"), None)],
            Cursor("rate-next"),
        )

    async def funding_payments(self, request: HistoryRequest) -> Page[FundingPayment]:
        self.received.append(("funding_payments", request))
        return Page(
            [
                FundingPayment(
                    self.market,
                    1_700_000_000_123_456_794,
                    Decimal("-0.1250"),
                    Decimal("0.0001"),
                    "funding-1",
                )
            ],
            None,
        )

    async def set_margin(self, request: MarginRequest) -> None:
        self.received.append(("set_margin", request))

    async def subscribe(self, subscription, config):
        if subscription.feeds == (Feed.TICKER,):
            async def pending():
                self.pending_started.set()
                try:
                    await asyncio.Event().wait()
                    yield StreamEvent(MarketEvent.reconnected())
                finally:
                    self.pending_cleanup_started.set()
                    await asyncio.sleep(0.05)
                    self.pending_closed.set()

            return MarketStream(pending())

        async def replay():
            yield StreamEvent(MarketEvent.trade(self.trade))
            yield StreamError(DecodeError("recorded corrupt market frame"))
            yield StreamEvent(MarketEvent.reconnected())

        return MarketStream(replay())

    async def subscribe_account(self, config):
        async def replay():
            yield StreamEvent(AccountEvent.balance(self.balance))
            yield StreamError(DecodeError("recorded corrupt account frame"))
            yield StreamEvent(AccountEvent.order(self.order))

        return AccountStream(replay())


class UnknownFieldErrorAdapter(NativeReplayAdapter):
    async def ticker(self, market: Market) -> Ticker:
        raise InvalidRequestError("custom_field", "bad")


class TransferFailureAdapter(NativeReplayAdapter):
    async def ticker(self, market: Market) -> Ticker:
        raise TransferError(TransferErrorKind.NETWORK_MISMATCH, "bad chain")


class CloseFailureSource:
    def __init__(self) -> None:
        self.started = asyncio.Event()

    def __aiter__(self):
        return self

    async def __anext__(self):
        self.started.set()
        await asyncio.Event().wait()

    async def aclose(self) -> None:
        raise RuntimeError("close boom")


class CloseFailureAdapter(NativeReplayAdapter):
    def __init__(self, market: Market) -> None:
        super().__init__(market)
        self.source = CloseFailureSource()

    async def subscribe(self, subscription, config):
        return MarketStream(self.source)


class DropCloseSource:
    def __init__(self) -> None:
        self.closed = asyncio.Event()

    def __aiter__(self):
        return self

    async def __anext__(self):
        await asyncio.Event().wait()

    async def aclose(self) -> None:
        self.closed.set()


class DropCloseAdapter(NativeReplayAdapter):
    def __init__(self, market: Market) -> None:
        super().__init__(market)
        self.source = DropCloseSource()

    async def subscribe(self, subscription, config):
        return MarketStream(self.source)


class NaturalEndSource:
    def __init__(self) -> None:
        self.closed = asyncio.Event()

    def __aiter__(self):
        return self

    async def __anext__(self):
        raise StopAsyncIteration

    async def aclose(self) -> None:
        await asyncio.sleep(0.25)
        self.closed.set()


class NaturalEndAdapter(NativeReplayAdapter):
    def __init__(self, market: Market) -> None:
        super().__init__(market)
        self.source = NaturalEndSource()

    async def subscribe(self, subscription, config):
        return MarketStream(self.source)


@unittest.skipUnless(NATIVE_AVAILABLE, "maxt._native is not built")
class NativeCustomAdapterTests(unittest.IsolatedAsyncioTestCase):
    async def test_custom_invalid_request_fields_survive_rust(self) -> None:
        market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
        client = Client(UnknownFieldErrorAdapter(market))

        with self.assertRaises(InvalidRequestError) as raised:
            await client.ticker(market)

        self.assertEqual(raised.exception.field, "custom_field")
        self.assertEqual(raised.exception.detail, "bad")

    async def test_custom_transfer_error_kind_survives_rust(self) -> None:
        market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
        client = Client(TransferFailureAdapter(market))

        with self.assertRaises(TransferError) as raised:
            await client.ticker(market)

        self.assertEqual(
            raised.exception.transfer_kind,
            TransferErrorKind.NETWORK_MISMATCH,
        )
        self.assertEqual(raised.exception.detail, "bad chain")

    async def test_source_close_errors_cross_rust_without_unraisable_output(self) -> None:
        market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
        adapter = CloseFailureAdapter(market)
        stream = await Client(adapter).subscribe(Subscription((market,), (Feed.TRADES,)))
        pending = asyncio.create_task(stream.__anext__())
        await asyncio.wait_for(adapter.source.started.wait(), NATIVE_TIMEOUT)
        stderr = io.StringIO()
        raised = None

        with redirect_stderr(stderr):
            try:
                await asyncio.wait_for(stream.aclose(), NATIVE_TIMEOUT)
            except AdapterError as error:
                raised = error
            with self.assertRaises(StopAsyncIteration):
                await asyncio.wait_for(pending, NATIVE_TIMEOUT)
            await asyncio.sleep(0.05)

        self.assertIsInstance(raised, AdapterError)
        self.assertIn("close boom", raised.detail)
        self.assertEqual(stderr.getvalue(), "")

    async def test_close_after_natural_end_waits_for_source_cleanup(self) -> None:
        market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
        adapter = NaturalEndAdapter(market)
        stream = await Client(adapter).subscribe(Subscription((market,), (Feed.TRADES,)))
        stderr = io.StringIO()

        with redirect_stderr(stderr):
            with self.assertRaises(StopAsyncIteration):
                await stream.__anext__()
            await asyncio.wait_for(stream.aclose(), NATIVE_TIMEOUT)
            closed_on_return = adapter.source.closed.is_set()
            await asyncio.wait_for(adapter.source.closed.wait(), NATIVE_TIMEOUT)

        self.assertTrue(closed_on_return)
        self.assertEqual(stderr.getvalue(), "")

    async def test_rest_private_order_margin_and_history_cross_rust(self) -> None:
        market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
        adapter = NativeReplayAdapter(market)
        client = Client(adapter)
        candle_request = CandleRequest(market, Interval.MIN1, limit=1)
        history_request = HistoryRequest(market, limit=1)
        order_request = OrderRequest.limit_order(
            market,
            Side.BUY,
            Size.base(Decimal("1.0000")),
            Decimal("50000.0100"),
        )
        margin_request = MarginRequest(market, Decimal("5"), MarginMode.CROSS)

        self.assertIs(client.adapter, adapter)
        self.assertEqual(client.exchange(), Exchange.BINANCE)
        self.assertTrue(client.supports(Feature.TRADES))
        self.assertEqual((await client.markets(MarketKind.SPOT))[0].market, market)
        self.assertEqual((await client.trades(market, 1))[0].price, Decimal("50000.0100"))
        self.assertEqual((await client.order_book(market, 1)).spread(), Decimal("0.02"))
        self.assertEqual((await client.ticker(market)).last_price, Decimal("50000.01"))
        self.assertEqual((await client.candles(candle_request))[0].volume, Decimal("12.5000"))
        self.assertEqual((await client.balances())[0].total(), Decimal("102.7500"))
        self.assertEqual((await client.open_orders())[0].id, "order-1")
        self.assertEqual((await client.open_orders_on(market))[0].id, "order-1")
        self.assertEqual((await client.place_order(order_request)).id, "order-1")
        self.assertEqual((await client.cancel_order(market, "order-1")).id, "order-1")
        self.assertEqual(await client.positions(), [adapter.position])
        self.assertEqual(await client.positions_on(market), [adapter.position])
        self.assertEqual((await client.margin_summary()).equity, Decimal("1000.00"))
        self.assertEqual((await client.funding_rates(history_request)).next, Cursor("rate-next"))
        self.assertEqual(
            (await client.funding_payments(history_request)).items[0].amount,
            Decimal("-0.1250"),
        )
        self.assertIsNone(await client.set_margin(margin_request))

        trade_call = next(call for call in adapter.received if call[0] == "trades")
        self.assertEqual(trade_call[1], market)
        self.assertFalse(trade_call[3])
        placed = next(call[1] for call in adapter.received if call[0] == "place_order")
        self.assertEqual(placed.to_wire(), order_request.to_wire())
        self.assertIsNot(placed, order_request)

    async def test_market_and_account_stream_items_cross_rust(self) -> None:
        market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
        client = Client(NativeReplayAdapter(market))
        subscription = Subscription((market,), (Feed.TRADES,))

        async with await client.subscribe(subscription) as market_stream:
            market_items = [await market_stream.__anext__() for _ in range(3)]
            with self.assertRaises(StopAsyncIteration):
                await market_stream.__anext__()

        async with await client.subscribe_account() as account_stream:
            account_items = [await account_stream.__anext__() for _ in range(3)]
            with self.assertRaises(StopAsyncIteration):
                await account_stream.__anext__()

        self.assertEqual([item.kind for item in market_items], ["event", "error", "event"])
        self.assertEqual(market_items[0].event.value.id, "trade-1")
        self.assertEqual(market_items[1].error.detail, "recorded corrupt market frame")
        self.assertEqual(market_items[2].event.kind, "reconnected")
        self.assertEqual([item.kind for item in account_items], ["event", "error", "event"])
        self.assertEqual(account_items[0].event.value.asset, "USDT")
        self.assertEqual(account_items[1].error.detail, "recorded corrupt account frame")
        self.assertEqual(account_items[2].event.value.id, "order-1")

    async def test_pending_market_next_closes_without_unraisable_output(self) -> None:
        market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
        adapter = NativeReplayAdapter(market)
        client = Client(adapter)
        stream = await client.subscribe(Subscription((market,), (Feed.TICKER,)))
        stderr = io.StringIO()

        with redirect_stderr(stderr):
            pending = asyncio.create_task(stream.__anext__())
            await asyncio.wait_for(adapter.pending_started.wait(), NATIVE_TIMEOUT)
            await asyncio.wait_for(stream.aclose(), NATIVE_TIMEOUT)
            closed_on_return = adapter.pending_closed.is_set()
            with self.assertRaises(StopAsyncIteration):
                await asyncio.wait_for(pending, NATIVE_TIMEOUT)
            await asyncio.wait_for(adapter.pending_closed.wait(), NATIVE_TIMEOUT)
            self.assertTrue(closed_on_return)
            await stream.aclose()
            await asyncio.sleep(0)

        self.assertEqual(stderr.getvalue(), "")

    async def test_cancelled_native_close_can_be_awaited_again(self) -> None:
        market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
        adapter = NativeReplayAdapter(market)
        stream = await Client(adapter).subscribe(Subscription((market,), (Feed.TICKER,)))
        pending = asyncio.create_task(stream.__anext__())
        await asyncio.wait_for(adapter.pending_started.wait(), NATIVE_TIMEOUT)
        first_close = asyncio.create_task(stream.aclose())
        await asyncio.wait_for(adapter.pending_cleanup_started.wait(), NATIVE_TIMEOUT)

        first_close.cancel()
        with self.assertRaises(asyncio.CancelledError):
            await first_close
        await asyncio.wait_for(stream.aclose(), NATIVE_TIMEOUT)
        with self.assertRaises(StopAsyncIteration):
            await asyncio.wait_for(pending, NATIVE_TIMEOUT)

        self.assertTrue(adapter.pending_closed.is_set())

    async def test_pending_close_process_stderr_is_empty(self) -> None:
        script = f"""
import asyncio
import sys
sys.path.insert(0, {os.path.dirname(__file__)!r})
from maxt import Client, Exchange, Feed, Market, Subscription
from test_native_custom_adapter import NativeReplayAdapter

async def main():
    market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
    adapter = NativeReplayAdapter(market)
    stream = await Client(adapter).subscribe(Subscription((market,), (Feed.TICKER,)))
    pending = asyncio.create_task(stream.__anext__())
    await asyncio.wait_for(adapter.pending_started.wait(), {NATIVE_TIMEOUT!r})
    await asyncio.wait_for(stream.aclose(), {NATIVE_TIMEOUT!r})
    closed_on_return = adapter.pending_closed.is_set()
    try:
        await asyncio.wait_for(pending, {NATIVE_TIMEOUT!r})
    except StopAsyncIteration:
        pass
    else:
        raise AssertionError("pending __anext__ did not stop")
    await asyncio.wait_for(adapter.pending_closed.wait(), {NATIVE_TIMEOUT!r})
    if not closed_on_return:
        raise AssertionError("aclose returned before source cleanup")

asyncio.run(main())
"""
        process = await asyncio.create_subprocess_exec(
            sys.executable,
            "-c",
            script,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        stdout, stderr = await asyncio.wait_for(process.communicate(), 5)

        self.assertEqual(process.returncode, 0, stderr.decode())
        self.assertEqual(stdout, b"")
        self.assertEqual(stderr, b"")

    async def test_unpolled_drop_is_best_effort_without_stderr(self) -> None:
        script = f"""
import asyncio
import gc
import sys
import threading
sys.path.insert(0, {os.path.dirname(__file__)!r})
from maxt import Client, Exchange, Feed, Market, Subscription
from test_native_custom_adapter import DropCloseAdapter

async def main():
    market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
    adapter = DropCloseAdapter(market)
    stream = await Client(adapter).subscribe(Subscription((market,), (Feed.TRADES,)))
    loop = asyncio.get_running_loop()
    call_soon_threadsafe = loop.call_soon_threadsafe
    scheduled = threading.Event()
    discarded = []

    def discard(callback, *args, **kwargs):
        discarded.append(callback)
        scheduled.set()

    loop.call_soon_threadsafe = discard
    try:
        del stream
        gc.collect()

        async def wait_until_scheduled():
            while not scheduled.is_set():
                await asyncio.sleep(0)

        await asyncio.wait_for(wait_until_scheduled(), {NATIVE_TIMEOUT!r})
    finally:
        loop.call_soon_threadsafe = call_soon_threadsafe

    discarded.clear()
    gc.collect()
    await asyncio.sleep(0)

asyncio.run(main())
"""
        process = await asyncio.create_subprocess_exec(
            sys.executable,
            "-c",
            script,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        stdout, stderr = await asyncio.wait_for(process.communicate(), 5)

        self.assertEqual(process.returncode, 0, stderr.decode())
        self.assertEqual(stdout, b"")
        self.assertEqual(stderr, b"")


if __name__ == "__main__":
    unittest.main()
