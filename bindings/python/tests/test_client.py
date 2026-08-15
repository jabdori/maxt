import unittest
from decimal import Decimal
from types import SimpleNamespace
from unittest.mock import patch

from maxt import (
    Adapter,
    Client,
    Exchange,
    Feature,
    Market,
    Position,
    Side,
    StreamConfig,
    Trade,
)


class NativeClientProbe:
    adapter = None

    def __init__(self, adapter):
        self._adapter = adapter

    @classmethod
    def from_adapter(cls, adapter):
        cls.adapter = adapter
        return cls(adapter)

    @property
    def exchange(self):
        return self._adapter.exchange.value

    def supports(self, feature):
        value = Feature(getattr(feature, "value", feature))
        return self._adapter.supports(value)

    async def trades(self, market, limit=None):
        return [value.to_wire() for value in await self._adapter.trades(market, limit)]


class BridgeProbeAdapter(Adapter):
    def __init__(self, trade):
        self.trade = trade
        self.calls = []

    @property
    def exchange(self) -> Exchange:
        return Exchange.BINANCE

    @property
    def features(self) -> frozenset[Feature]:
        return frozenset({Feature.TRADES})

    async def trades(self, market, limit=None):
        self.calls.append((market, limit))
        return [self.trade]


class RecordingAdapter(Adapter):
    def __init__(self) -> None:
        self.calls = []

    @property
    def exchange(self) -> Exchange:
        return Exchange.BINANCE

    @property
    def features(self) -> frozenset[Feature]:
        return frozenset({Feature.TRADES, Feature.TRADE_STREAM})

    async def trades(self, market, limit=None):
        self.calls.append(("trades", market, limit))
        return ["trade"]

    async def subscribe(self, subscription, config):
        self.calls.append(("subscribe", subscription, config))
        return "stream"

    async def positions(self, market=None):
        self.calls.append(("positions", market))
        position_market = market or Market.perpetual(
            Exchange.BINANCE,
            "BTC",
            "USDT",
        )
        return [
            Position(position_market, None, Decimal("0")),
            Position(position_market, Side.BUY, Decimal("0.125")),
        ]

    async def _record(self, name, *args):
        self.calls.append((name, *args))
        return name

    async def markets(self, kind):
        return await self._record("markets", kind)

    async def order_book(self, market, depth=None):
        return await self._record("order_book", market, depth)

    async def ticker(self, market):
        return await self._record("ticker", market)

    async def candles(self, request):
        return await self._record("candles", request)

    async def balances(self):
        return await self._record("balances")

    async def open_orders(self, market=None):
        return await self._record("open_orders", market)

    async def subscribe_account(self, config):
        return await self._record("subscribe_account", config)

    async def place_order(self, request):
        return await self._record("place_order", request)

    async def cancel_order(self, market, order_id):
        await self._record("cancel_order", market, order_id)

    async def margin_summary(self):
        return await self._record("margin_summary")

    async def funding_rates(self, request):
        return await self._record("funding_rates", request)

    async def funding_payments(self, request):
        return await self._record("funding_payments", request)

    async def set_margin(self, request):
        return await self._record("set_margin", request)


class ClientContractTests(unittest.IsolatedAsyncioTestCase):
    async def test_custom_adapter_calls_cross_the_native_client_bridge(self) -> None:
        market = Market.spot(Exchange.BINANCE, "BTC", "USDT")
        trade = Trade(
            market,
            1_700_000_000_123_456_789,
            Decimal("50000.0100"),
            Decimal("0.001"),
            Side.BUY,
            "bridge-trade",
        )
        adapter = BridgeProbeAdapter(trade)
        NativeClientProbe.adapter = None
        native = SimpleNamespace(NativeClient=NativeClientProbe)

        with patch("maxt._api._load_native", return_value=native):
            client = Client(adapter)
            result = await client.trades(market, 1)

        self.assertIs(NativeClientProbe.adapter, adapter)
        self.assertIs(client.adapter, adapter)
        self.assertEqual(result, [trade])
        self.assertEqual(adapter.calls, [(market, 1)])

    async def test_client_preserves_the_adapter_and_rust_defaults(self) -> None:
        adapter = RecordingAdapter()
        with patch("maxt.adapters._client_delegate", return_value=adapter):
            client = Client(adapter)
        market = Market.spot(Exchange.BINANCE, "BTC", "USDT")
        subscription = object()

        self.assertIs(client.adapter, adapter)
        self.assertIs(client.into_adapter(), adapter)
        self.assertEqual(client.exchange(), Exchange.BINANCE)
        self.assertTrue(client.supports(Feature.TRADES))
        self.assertEqual(await client.trades(market), ["trade"])
        self.assertEqual(await client.subscribe(subscription), "stream")

        self.assertEqual(adapter.calls[0], ("trades", market, None))
        self.assertEqual(adapter.calls[1][0:2], ("subscribe", subscription))
        self.assertEqual(adapter.calls[1][2], StreamConfig())

    async def test_positions_remove_flat_rows_for_global_and_market_reads(self) -> None:
        adapter = RecordingAdapter()
        with patch("maxt.adapters._client_delegate", return_value=adapter):
            client = Client(adapter)
        market = Market.perpetual(Exchange.BINANCE, "ETH", "USDT")

        all_positions = await client.positions()
        market_positions = await client.positions_on(market)

        self.assertEqual([row.quantity for row in all_positions], [Decimal("0.125")])
        self.assertEqual([row.quantity for row in market_positions], [Decimal("0.125")])
        self.assertEqual(adapter.calls, [("positions", None), ("positions", market)])

    async def test_public_private_order_margin_and_history_calls_are_exposed(self) -> None:
        adapter = RecordingAdapter()
        with patch("maxt.adapters._client_delegate", return_value=adapter):
            client = Client(adapter)
        market = Market.spot(Exchange.BINANCE, "BTC", "USDT")
        kind = object()
        request = object()
        config = StreamConfig(buffer_size=8)

        self.assertEqual(await client.markets(kind), "markets")
        self.assertEqual(await client.order_book(market), "order_book")
        self.assertEqual(await client.ticker(market), "ticker")
        self.assertEqual(await client.candles(request), "candles")
        self.assertEqual(await client.balances(), "balances")
        self.assertEqual(await client.open_orders(), "open_orders")
        self.assertEqual(await client.open_orders_on(market), "open_orders")
        self.assertEqual(await client.subscribe_account(), "subscribe_account")
        self.assertEqual(
            await client.subscribe_account_with(config),
            "subscribe_account",
        )
        self.assertEqual(await client.place_order(request), "place_order")
        self.assertIsNone(await client.cancel_order(market, "42"))
        self.assertEqual(await client.margin_summary(), "margin_summary")
        self.assertEqual(await client.funding_rates(request), "funding_rates")
        self.assertEqual(await client.funding_payments(request), "funding_payments")
        self.assertEqual(await client.set_margin(request), "set_margin")

        self.assertEqual(adapter.calls[0], ("markets", kind))
        self.assertEqual(adapter.calls[1], ("order_book", market, None))
        self.assertEqual(adapter.calls[5], ("open_orders", None))
        self.assertEqual(adapter.calls[6], ("open_orders", market))
        self.assertEqual(adapter.calls[7], ("subscribe_account", StreamConfig()))
        self.assertEqual(adapter.calls[8], ("subscribe_account", config))


if __name__ == "__main__":
    unittest.main()
