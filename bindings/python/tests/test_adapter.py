import unittest

from maxt import (
    Adapter,
    AdapterError,
    AuthError,
    Exchange,
    ExchangeError,
    ExchangeErrorKind,
    Feed,
    Feature,
    InvalidRequestError,
    Market,
    StreamConfig,
    Subscription,
    TransportError,
    UnsupportedError,
)
from maxt._api import _error_from_wire


class MinimalAdapter(Adapter):
    @property
    def exchange(self) -> Exchange:
        return Exchange.UPBIT

    @property
    def features(self) -> frozenset[Feature]:
        return frozenset({Feature.MARKETS})


class AdapterContractTests(unittest.IsolatedAsyncioTestCase):
    async def test_optional_methods_raise_the_exact_unsupported_error(self) -> None:
        adapter = MinimalAdapter()

        with self.assertRaises(UnsupportedError) as raised:
            await adapter.balances()

        self.assertEqual(raised.exception.exchange, Exchange.UPBIT)
        self.assertEqual(raised.exception.feature, Feature.BALANCES)
        self.assertEqual(
            str(raised.exception),
            "upbit has no endpoint for balances",
        )

    async def test_every_optional_method_names_its_unsupported_feature(self) -> None:
        adapter = MinimalAdapter()
        value = object()
        subscription = Subscription(
            (Market.spot(Exchange.UPBIT, "BTC", "KRW"),),
            (Feed.TRADES,),
        )
        calls = [
            (Feature.MARKETS, lambda: adapter.markets(value)),
            (Feature.TRADES, lambda: adapter.trades(value)),
            (Feature.ORDER_BOOK, lambda: adapter.order_book(value)),
            (Feature.TICKER, lambda: adapter.ticker(value)),
            (Feature.CANDLES, lambda: adapter.candles(value)),
            (
                Feature.TRADE_STREAM,
                lambda: adapter.subscribe(subscription, StreamConfig()),
            ),
            (Feature.BALANCES, adapter.balances),
            (Feature.OPEN_ORDERS, adapter.open_orders),
            (Feature.ACCOUNT_STREAM, lambda: adapter.subscribe_account(value)),
            (Feature.TRADING, lambda: adapter.place_order(value)),
            (Feature.TRADING, lambda: adapter.cancel_order(value, "order-id")),
            (Feature.POSITIONS, adapter.positions),
            (Feature.MARGIN, adapter.margin_summary),
            (Feature.FUNDING_RATES, lambda: adapter.funding_rates(value)),
            (Feature.FUNDING_PAYMENTS, lambda: adapter.funding_payments(value)),
            (Feature.MARGIN_CONFIG, lambda: adapter.set_margin(value)),
        ]

        for feature, call in calls:
            with self.subTest(feature=feature):
                with self.assertRaises(UnsupportedError) as raised:
                    await call()
                self.assertEqual(raised.exception.feature, feature)

    async def test_default_subscribe_rejects_empty_markets_before_feeds(self) -> None:
        adapter = MinimalAdapter()

        with self.assertRaises(InvalidRequestError) as raised:
            await adapter.subscribe(Subscription((), ()), StreamConfig())

        self.assertEqual(raised.exception.field, "markets")
        self.assertEqual(
            raised.exception.detail,
            "a subscription needs at least one market",
        )

    async def test_default_subscribe_rejects_empty_feeds(self) -> None:
        adapter = MinimalAdapter()
        market = Market.spot(Exchange.UPBIT, "BTC", "KRW")

        with self.assertRaises(InvalidRequestError) as raised:
            await adapter.subscribe(Subscription((market,), ()), StreamConfig())

        self.assertEqual(raised.exception.field, "feeds")
        self.assertEqual(
            raised.exception.detail,
            "a subscription needs at least one feed",
        )

    async def test_structured_errors_preserve_the_core_error_fields(self) -> None:
        invalid = InvalidRequestError("limit", "must be positive")
        auth = AuthError("missing API key")
        exchange = ExchangeError(
            Exchange.BINANCE,
            "-1003",
            "too many requests",
            429,
            ExchangeErrorKind.RATE_LIMITED,
        )
        transport = TransportError("connection reset")
        adapter = AdapterError("custom adapter crashed")

        self.assertEqual(
            invalid.to_wire(),
            {
                "kind": "invalid_request",
                "field": "limit",
                "detail": "must be positive",
            },
        )
        self.assertEqual(auth.to_wire(), {"kind": "auth", "detail": "missing API key"})
        self.assertEqual(exchange.to_wire()["exchange_kind"], "rate_limited")
        self.assertTrue(exchange.is_retryable())
        self.assertTrue(exchange.is_rate_limited())
        self.assertTrue(transport.is_retryable())
        self.assertFalse(adapter.is_retryable())

    async def test_exchange_error_retryability_comes_from_its_kind(self) -> None:
        retryable = {
            kind for kind in ExchangeErrorKind if kind.is_retryable()
        }
        self.assertEqual(
            retryable,
            {
                ExchangeErrorKind.RATE_LIMITED,
                ExchangeErrorKind.UNAVAILABLE,
            },
        )

        for kind in ExchangeErrorKind:
            with self.subTest(kind=kind):
                error = ExchangeError(
                    Exchange.BINANCE,
                    "code",
                    "message",
                    None,
                    kind,
                )
                self.assertEqual(error.is_retryable(), kind.is_retryable())

    async def test_native_error_wires_preserve_variant_details(self) -> None:
        adapter = _error_from_wire(
            {
                "kind": "adapter",
                "message": "adapter failed: original detail",
                "detail": "original detail",
            }
        )
        exchange = _error_from_wire(
            {
                "kind": "exchange",
                "exchange": "binance",
                "code": "-1003",
                "provider_message": "too many requests",
                "status": 429,
                "exchange_kind": "rate_limited",
            }
        )
        exchange_without_status = _error_from_wire(
            {
                "kind": "exchange",
                "exchange": "binance",
                "code": "-1003",
                "provider_message": "too many requests",
                "exchange_kind": "rate_limited",
            }
        )
        empty_adapter = _error_from_wire(
            {
                "kind": "adapter",
                "message": "adapter failed: ",
                "detail": "",
            }
        )
        empty_exchange = _error_from_wire(
            {
                "kind": "exchange",
                "exchange": "binance",
                "code": "-1",
                "message": "binance returned -1: ",
                "provider_message": "",
                "status": None,
                "exchange_kind": "unknown",
            }
        )

        self.assertIsInstance(adapter, AdapterError)
        self.assertEqual(adapter.detail, "original detail")
        self.assertIsInstance(exchange, ExchangeError)
        self.assertEqual(exchange.message, "too many requests")
        self.assertIsInstance(exchange_without_status, ExchangeError)
        self.assertIsNone(exchange_without_status.status)
        self.assertEqual(empty_adapter.detail, "")
        self.assertEqual(empty_exchange.message, "")


if __name__ == "__main__":
    unittest.main()
