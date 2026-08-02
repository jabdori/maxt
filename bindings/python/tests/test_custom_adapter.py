import asyncio
import unittest
from decimal import Decimal
from unittest.mock import patch

from maxt import (
    Adapter,
    Balance,
    Client,
    DecodeError,
    Exchange,
    Feature,
    Feed,
    Market,
    MarketEvent,
    MarketStream,
    Side,
    StreamError,
    StreamEvent,
    Subscription,
    Trade,
)


class ReplayAdapter(Adapter):
    def __init__(self, trades):
        self._trades = trades

    @property
    def exchange(self) -> Exchange:
        return Exchange.UPBIT

    @property
    def features(self) -> frozenset[Feature]:
        return frozenset(
            {Feature.TRADES, Feature.BALANCES, Feature.TRADE_STREAM}
        )

    async def trades(self, market, limit=None):
        return self._trades if limit is None else self._trades[:limit]

    async def balances(self):
        return [Balance("KRW", Decimal("100000"), Decimal("2500"))]

    async def subscribe(self, subscription, config):
        async def replay():
            yield StreamEvent(MarketEvent.trade(self._trades[0]))
            yield StreamError(DecodeError("recorded corrupt frame"))
            yield StreamEvent(MarketEvent.trade(self._trades[1]))

        return MarketStream(replay())


class SlowCloseSource:
    def __init__(self):
        self.started = asyncio.Event()
        self.release = asyncio.Event()
        self.closed = asyncio.Event()

    def __aiter__(self):
        return self

    async def __anext__(self):
        await asyncio.Event().wait()

    async def aclose(self):
        self.started.set()
        await self.release.wait()
        self.closed.set()


class CustomAdapterTests(unittest.IsolatedAsyncioTestCase):
    async def test_cancelled_close_can_be_awaited_again(self) -> None:
        source = SlowCloseSource()
        stream = MarketStream(source)
        first_close = asyncio.create_task(stream.aclose())
        await source.started.wait()

        first_close.cancel()
        with self.assertRaises(asyncio.CancelledError):
            await first_close
        source.release.set()
        await asyncio.wait_for(stream.aclose(), 0.5)

        self.assertTrue(source.closed.is_set())
        with self.assertRaises(StopAsyncIteration):
            await stream.__anext__()

    async def test_replay_adapter_serves_rest_private_and_stream_results(self) -> None:
        market = Market.spot(Exchange.UPBIT, "BTC", "KRW")
        trades = [
            Trade(
                market,
                1_700_000_000_123_456_789,
                Decimal("50000000.01"),
                Decimal("0.001"),
                Side.BUY,
                "first",
            ),
            Trade(
                market,
                1_700_000_000_123_456_790,
                Decimal("50000000.02"),
                Decimal("0.002"),
                Side.SELL,
                "second",
            ),
        ]
        adapter = ReplayAdapter(trades)
        with patch("maxt.adapters._client_delegate", return_value=adapter):
            client = Client(adapter)

        self.assertEqual(await client.trades(market, 1), trades[:1])
        self.assertEqual((await client.balances())[0].total(), Decimal("102500"))

        subscription = Subscription((market,), (Feed.TRADES,))
        async with await client.subscribe(subscription) as stream:
            first = await stream.__anext__()
            error = await stream.__anext__()
            second = await stream.__anext__()

            self.assertEqual(first.kind, "event")
            self.assertEqual(first.event.value.id, "first")
            self.assertEqual(error.kind, "error")
            self.assertIsInstance(error.error, DecodeError)
            self.assertEqual(second.kind, "event")
            self.assertEqual(second.event.value.id, "second")
            with self.assertRaises(StopAsyncIteration):
                await stream.__anext__()

        await stream.aclose()


if __name__ == "__main__":
    unittest.main()
