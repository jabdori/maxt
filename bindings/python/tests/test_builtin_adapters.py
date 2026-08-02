import unittest
from decimal import Decimal
from types import SimpleNamespace
from unittest.mock import patch

from maxt import (
    Adapter,
    BithumbAdapter,
    BithumbAlertStep,
    BithumbMarketAlert,
    BinanceAdapter,
    BinanceListenKey,
    BinanceMarket,
    BinanceSpotOrderDetail,
    BinanceSymbolFilters,
    Client,
    Exchange,
    Feature,
    HyperliquidAdapter,
    HyperliquidAssetContext,
    HyperliquidLedgerEntry,
    HyperliquidLedgerKind,
    Market,
    MarketEvent,
    StreamError,
    StreamEvent,
    Subscription,
    Feed,
    Trade,
    UpbitAdapter,
    UpbitMarketEvent,
    UpbitRegion,
)


MARKET_WIRE = {
    "exchange": "upbit",
    "kind": "spot",
    "base": "BTC",
    "quote": "KRW",
}


class FakeNativeClient:
    exchange = "upbit"

    def supports(self, feature):
        value = getattr(feature, "value", feature)
        return value in {"trades", "balances", "trade_stream"}

    async def trades(self, market, limit=None):
        return [
            {
                "market": MARKET_WIRE,
                "timestamp": 1_700_000_000_000_000_007,
                "price": "50000000.0100",
                "quantity": "0.001",
                "taker_side": "buy",
                "id": "native-trade",
            }
        ]

    async def subscribe(self, subscription, config):
        trade = (await self.trades(None))[0]
        return FakeNativeStream(
            [
                {"kind": "event", "event": {"kind": "trade", "value": trade}},
                {
                    "kind": "error",
                    "error": {"kind": "decode", "detail": "bad frame"},
                },
                None,
                {"kind": "event", "event": {"kind": "reconnected", "value": None}},
            ]
        )


class FakeNativeStream:
    def __init__(self, items):
        self.items = iter(items)
        self.closed = False

    def __aiter__(self):
        return self

    async def __anext__(self):
        try:
            return next(self.items)
        except StopIteration:
            raise StopAsyncIteration

    async def aclose(self):
        self.closed = True


class FakeNativeUpbitAdapter:
    def __init__(
        self,
        *,
        region="korea",
        access_key=None,
        secret_key=None,
    ):
        self.region = region
        self.authenticated = access_key is not None and secret_key is not None

    def client(self):
        return FakeNativeClient()

    async def market_events(self):
        return [
            {
                "market": MARKET_WIRE,
                "warning": True,
                "cautions": ["PRICE_FLUCTUATION"],
            }
        ]


class FakeNativeBithumbAdapter:
    authenticated = False

    def __init__(self, *, access_key=None, secret_key=None):
        self.authenticated = access_key is not None and secret_key is not None

    def client(self):
        client = FakeNativeClient()
        client.exchange = "bithumb"
        return client

    async def market_warnings(self):
        return [{"market": {**MARKET_WIRE, "exchange": "bithumb"}, "warning": "CAUTION"}]

    async def market_alerts(self):
        return [
            {
                "market": {**MARKET_WIRE, "exchange": "bithumb"},
                "kind": "PRICE_FLUCTUATION",
                "step": "danger",
                "ends_at": 1_700_000_000_000_000_008,
            }
        ]


class FakeNativeBinanceListenKey:
    value = "listen-key"


class FakeNativeBinanceAdapter:
    authenticated = True

    def __init__(self, *, venue="spot", api_key=None, secret_key=None):
        self.venue = venue
        self.kept_alive = None
        self.closed = None

    def client(self):
        client = FakeNativeClient()
        client.exchange = "binance"
        return client

    async def spot_symbol_filters(self, market):
        return {
            "symbol": "BTCUSDT",
            "tick_size": "0.0100",
            "min_price": "0.01",
            "max_price": None,
            "step_size": "0.0001",
            "min_quantity": "0.0001",
            "max_quantity": None,
            "min_notional": "5",
        }

    async def spot_order(self, market, order_id):
        return {
            "order": {
                "id": order_id,
                "market": {**MARKET_WIRE, "exchange": "binance", "quote": "USDT"},
                "side": "buy",
                "status": "filled",
                "filled_quantity": "0.25",
                "remaining_quantity": "0",
                "price": "50000",
                "created_at": 1_700_000_000_000_000_009,
            },
            "client_order_id": "client-42",
            "order_type": "LIMIT_MAKER",
            "time_in_force": "GTC",
            "filled_quote_quantity": "12500",
            "updated_at": 1_700_000_000_000_000_010,
        }

    async def usd_m_create_listen_key(self):
        return FakeNativeBinanceListenKey()

    async def usd_m_keepalive_listen_key(self, key):
        self.kept_alive = key

    async def usd_m_close_listen_key(self, key):
        self.closed = key


class FakeNativeHyperliquidAdapter:
    def __init__(self, *, testnet=False, address=None, private_key=None):
        self.is_testnet = testnet
        self.authenticated = address is not None and private_key is not None
        self.ledger_args = None

    def client(self):
        client = FakeNativeClient()
        client.exchange = "hyperliquid"
        return client

    async def non_funding_ledger(self, from_ns=None, to_ns=None, cursor=None, limit=None):
        self.ledger_args = (from_ns, to_ns, cursor, limit)
        return {
            "items": [
                {
                    "kind": "future_ledger_type",
                    "time": 1_700_000_000_000_000_011,
                    "hash": "0xabc",
                    "asset": "USDC",
                    "amount": "10.2500",
                    "fee": "0.01",
                    "counterparty": None,
                }
            ],
            "next": "ledger-cursor",
        }

    async def asset_context(self, market):
        return {
            "mid_price": "3500.25",
            "mark_price": "3500.20",
            "oracle_price": "3499.99",
            "funding_rate": "0.0001",
            "open_interest": "1250.5",
            "size_decimals": 4,
            "price_decimals": 2,
        }


class BuiltinAdapterTests(unittest.IsolatedAsyncioTestCase):
    async def test_binance_listen_keys_only_come_from_the_provider_api(self) -> None:
        with self.assertRaisesRegex(
            TypeError,
            "BinanceListenKey values come from "
            "BinanceAdapter.usd_m_create_listen_key",
        ):
            BinanceListenKey()

    async def test_upbit_wraps_native_common_and_provider_apis(self) -> None:
        native = SimpleNamespace(NativeUpbitAdapter=FakeNativeUpbitAdapter)
        with patch("maxt._api._load_native", return_value=native):
            adapter = UpbitAdapter(
                region=UpbitRegion.SINGAPORE,
                access_key="key",
                secret_key="secret",
            )
            client = Client(adapter)
            market = Market.spot(Exchange.UPBIT, "BTC", "KRW")

            trades = await client.trades(market)
            events = await adapter.market_events()

        self.assertIsInstance(adapter, Adapter)
        self.assertIs(client.adapter, adapter)
        self.assertEqual(adapter.exchange, Exchange.UPBIT)
        self.assertEqual(adapter.region, UpbitRegion.SINGAPORE)
        self.assertTrue(adapter.authenticated)
        self.assertIn(Feature.TRADES, adapter.features)
        self.assertIsInstance(trades[0], Trade)
        self.assertEqual(trades[0].price, Decimal("50000000.0100"))
        self.assertEqual(events[0][0], market)
        self.assertIsInstance(events[0][1], UpbitMarketEvent)
        self.assertTrue(events[0][1].warning)

    async def test_native_stream_items_are_decoded_and_only_termination_ends(self) -> None:
        native = SimpleNamespace(NativeUpbitAdapter=FakeNativeUpbitAdapter)
        with patch("maxt._api._load_native", return_value=native):
            adapter = UpbitAdapter()
            market = Market.spot(Exchange.UPBIT, "BTC", "KRW")
            stream = await Client(adapter).subscribe(
                Subscription((market,), (Feed.TRADES,))
            )

            first = await stream.__anext__()
            error = await stream.__anext__()
            with self.assertRaises(TypeError):
                await stream.__anext__()
            reconnected = await stream.__anext__()
            with self.assertRaises(StopAsyncIteration):
                await stream.__anext__()

        self.assertIsInstance(first, StreamEvent)
        self.assertIsInstance(first.event, MarketEvent)
        self.assertIsInstance(first.event.value, Trade)
        self.assertIsInstance(error, StreamError)
        self.assertEqual(error.error.detail, "bad frame")
        self.assertEqual(reconnected.event.kind, "reconnected")

    async def test_bithumb_exposes_warning_and_alert_rows(self) -> None:
        native = SimpleNamespace(NativeBithumbAdapter=FakeNativeBithumbAdapter)
        with patch("maxt._api._load_native", return_value=native):
            adapter = BithumbAdapter(access_key="key", secret_key="secret")
            warnings = await adapter.market_warnings()
            alerts = await adapter.market_alerts()

        self.assertEqual(adapter.exchange, Exchange.BITHUMB)
        self.assertTrue(adapter.authenticated)
        self.assertEqual(warnings[0][1], "CAUTION")
        self.assertIsInstance(alerts[0][1], BithumbMarketAlert)
        self.assertEqual(alerts[0][1].step, BithumbAlertStep.DANGER)
        self.assertEqual(alerts[0][1].ends_at, 1_700_000_000_000_000_008)

    async def test_binance_exposes_spot_details_and_usd_m_listen_keys(self) -> None:
        native = SimpleNamespace(NativeBinanceAdapter=FakeNativeBinanceAdapter)
        with patch("maxt._api._load_native", return_value=native):
            spot = BinanceAdapter.spot(api_key="key", secret_key="secret")
            futures = BinanceAdapter.usd_m_futures(
                api_key="key",
                secret_key="secret",
            )
            market = Market.spot(Exchange.BINANCE, "BTC", "USDT")
            filters = await spot.spot_symbol_filters(market)
            order = await spot.spot_order(market, "42")
            key = await futures.usd_m_create_listen_key()
            await futures.usd_m_keepalive_listen_key(key)
            await futures.usd_m_close_listen_key(key)

        self.assertEqual(spot.venue, BinanceMarket.SPOT)
        self.assertEqual(futures.venue, BinanceMarket.USD_M_FUTURES)
        self.assertIsInstance(filters, BinanceSymbolFilters)
        self.assertEqual(filters.tick_size, Decimal("0.0100"))
        self.assertIsInstance(order, BinanceSpotOrderDetail)
        self.assertEqual(order.filled_quote_quantity, Decimal("12500"))
        self.assertIsInstance(key, BinanceListenKey)
        self.assertEqual(key.value, "listen-key")
        self.assertIs(futures._handle.kept_alive, key._handle)
        self.assertIs(futures._handle.closed, key._handle)

    async def test_hyperliquid_exposes_ledger_and_asset_context(self) -> None:
        native = SimpleNamespace(
            NativeHyperliquidAdapter=FakeNativeHyperliquidAdapter
        )
        with patch("maxt._api._load_native", return_value=native):
            adapter = HyperliquidAdapter.testnet(
                address="0xaccount",
                private_key="0xkey",
            )
            market = Market.perpetual(Exchange.HYPERLIQUID, "ETH", "USDC")
            ledger = await adapter.non_funding_ledger(
                from_=1_700_000_000_000_000_000,
                cursor="cursor",
                limit=50,
            )
            context = await adapter.asset_context(market)

        self.assertTrue(adapter.is_testnet)
        self.assertTrue(adapter.authenticated)
        self.assertIsInstance(ledger.items[0], HyperliquidLedgerEntry)
        self.assertEqual(
            ledger.items[0].kind.value,
            "future_ledger_type",
        )
        self.assertEqual(ledger.items[0].amount, Decimal("10.2500"))
        self.assertEqual(
            adapter._handle.ledger_args,
            (1_700_000_000_000_000_000, None, "cursor", 50),
        )
        self.assertIsInstance(context, HyperliquidAssetContext)
        self.assertEqual(context.mid_price, Decimal("3500.25"))


if __name__ == "__main__":
    unittest.main()
