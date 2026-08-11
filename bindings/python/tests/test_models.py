import unittest
from decimal import Decimal

import maxt
import maxt.models as maxt_models
from maxt import (
    Balance,
    CancelOrdersRequest,
    CancelOrdersResult,
    CancelledOrder,
    ChainDestination,
    Candle,
    CandleRequest,
    Decimal as MaxtDecimal,
    Exchange,
    Feature,
    FundingPayment,
    FundingRate,
    HistoryRequest,
    Interval,
    Level,
    MarketEvent,
    MarketInfo,
    MarketKind,
    MarketStatus,
    Market,
    MarginSummary,
    MarginMode,
    MarginRequest,
    Network,
    Order,
    OrderAccount,
    OrderBook,
    OrderHistoryRequest,
    OrderCancelFailure,
    OrderIdKind,
    OrderLookupRequest,
    OrderOption,
    OrderRequest,
    OrderRules,
    OrderStatus,
    OrderType,
    Position,
    Page,
    Side,
    Ticker,
    TimeInForce,
    Timestamp,
    Trade,
    Size,
    StreamConfig,
    TransferDestination,
    TransferHistoryRequest,
    TravelRuleRequirement,
    WithdrawalFee,
    WithdrawRequest,
)
from maxt.models import _model_from_wire, _model_to_wire


class WireModelTests(unittest.TestCase):
    def test_wire_conversion_helpers_are_not_public(self) -> None:
        for name in ("_model_from_wire", "_model_to_wire"):
            with self.subTest(name=name):
                self.assertNotIn(name, maxt.__all__)
                self.assertFalse(hasattr(maxt, name))
                self.assertNotIn(name, maxt_models.__all__)

    def test_enum_helpers_match_the_rust_value_types(self) -> None:
        self.assertEqual(
            {exchange: exchange.display_name() for exchange in Exchange},
            {
                Exchange.UPBIT: "Upbit",
                Exchange.BITHUMB: "Bithumb",
                Exchange.BINANCE: "Binance",
                Exchange.HYPERLIQUID: "Hyperliquid",
            },
        )
        self.assertEqual(
            {feature for feature in Feature if feature.needs_credentials()},
            {
                Feature.BALANCES,
                Feature.ASSET_NETWORKS,
                Feature.DEPOSIT_ADDRESSES,
                Feature.DEPOSIT_HISTORY,
                Feature.DEPOSIT_LOOKUP,
                Feature.WITHDRAWAL_QUOTES,
                Feature.WITHDRAWALS,
                Feature.WITHDRAWAL_HISTORY,
                Feature.WITHDRAWAL_LOOKUP,
                Feature.WITHDRAWAL_CANCELLATION,
                Feature.OPEN_ORDERS,
                Feature.ORDER_HISTORY,
                Feature.ACCOUNT_STREAM,
                Feature.TRADING,
                Feature.POSITIONS,
                Feature.MARGIN,
                Feature.FUNDING_PAYMENTS,
                Feature.MARGIN_CONFIG,
                Feature.REDUCE_ONLY_ORDERS,
            },
        )
        self.assertEqual(
            {feature for feature in Feature if feature.is_derivatives_only()},
            {
                Feature.POSITIONS,
                Feature.MARGIN,
                Feature.FUNDING_RATES,
                Feature.FUNDING_PAYMENTS,
                Feature.MARGIN_CONFIG,
                Feature.REDUCE_ONLY_ORDERS,
            },
        )
        self.assertFalse(MarketKind.SPOT.is_derivative())
        self.assertTrue(MarketKind.PERPETUAL.is_derivative())
        self.assertIs(Side.BUY.flip(), Side.SELL)
        self.assertIs(Side.SELL.flip(), Side.BUY)

    def test_order_history_request_defaults_to_all_final_orders(self) -> None:
        request = OrderHistoryRequest()

        self.assertIsNone(request.market)
        self.assertEqual(request.statuses, [])

    def test_order_lookup_request_preserves_identifier_namespace(self) -> None:
        market = Market.spot(Exchange.UPBIT, "BTC", "KRW")
        request = OrderLookupRequest(OrderIdKind.EXCHANGE, ["order-1", "order-2"], market)

        self.assertEqual(
            request.to_wire(),
            {
                "kind": "exchange",
                "ids": ["order-1", "order-2"],
                "market": market.to_wire(),
            },
        )

    def test_batch_cancel_models_preserve_partial_failures(self) -> None:
        request = CancelOrdersRequest(OrderIdKind.CLIENT, ["client-1"])
        result = CancelOrdersResult(
            [CancelledOrder("order-1", "client-1", None, 123)],
            [OrderCancelFailure(None, "missing-1", None, "not_found", "missing")],
        )

        self.assertEqual(request.to_wire(), {"kind": "client", "ids": ["client-1"]})
        self.assertEqual(result.to_wire()["failed"][0]["code"], "not_found")

    def test_order_rules_preserve_known_and_future_provider_options(self) -> None:
        market = Market.spot(Exchange.UPBIT, "BTC", "KRW")
        rules = OrderRules(
            market,
            "BTC/KRW",
            MarketStatus.ACTIVE,
            Decimal("0.001"),
            Decimal("0.001"),
            Decimal("0.0005"),
            Decimal("0.0005"),
            [Side.BUY, Side.SELL],
            [OrderOption("limit_ioc", OrderType.LIMIT, TimeInForce.IMMEDIATE_OR_CANCEL)],
            [OrderOption("future_order")],
            None,
            None,
            Decimal("5000"),
            Decimal("5000"),
            Decimal("1000000000"),
            OrderAccount(Balance("krw", Decimal("10000"), Decimal("0")), Decimal("0"), False, "KRW"),
            OrderAccount(Balance("btc", Decimal("1"), Decimal("0")), Decimal("95000000"), False, "KRW"),
        )

        wire = rules.to_wire()
        self.assertEqual(wire["quote_account"]["balance"]["asset"], "KRW")
        self.assertEqual(wire["buy_options"][0]["time_in_force"], "immediate_or_cancel")
        self.assertIsNone(wire["sell_options"][0]["order_type"])
        self.assertIsNone(wire["buy_price_unit"])

    def test_intervals_report_fixed_lengths_and_advance_without_overflow(self) -> None:
        self.assertEqual(
            {interval: interval.as_secs() for interval in Interval},
            {
                Interval.SEC1: 1,
                Interval.MIN1: 60,
                Interval.MIN3: 180,
                Interval.MIN5: 300,
                Interval.MIN10: 600,
                Interval.MIN15: 900,
                Interval.MIN30: 1_800,
                Interval.HOUR1: 3_600,
                Interval.HOUR2: 7_200,
                Interval.HOUR4: 14_400,
                Interval.HOUR6: 21_600,
                Interval.HOUR8: 28_800,
                Interval.HOUR12: 43_200,
                Interval.DAY1: 86_400,
                Interval.DAY3: 259_200,
                Interval.WEEK1: 604_800,
                Interval.MONTH1: None,
            },
        )

        at = 1_700_000_000_123_456_789
        self.assertEqual(
            Interval.MIN1.advance(at, 2),
            1_700_000_120_123_456_789,
        )
        self.assertEqual(
            Interval.WEEK1.advance(at, -1),
            at - 604_800_000_000_000,
        )

        late = 9_220_000_000_000_000_000
        self.assertIsNone(Interval.MONTH1.advance(late, 12))
        self.assertIsNone(Interval.WEEK1.advance(late, (1 << 63) - 1))

    def test_month_interval_uses_the_utc_calendar_and_preserves_nanoseconds(self) -> None:
        nanoseconds = 123_456_789
        january_31_2024 = 1_706_659_200_000_000_000 + nanoseconds
        february_29_2024 = 1_709_164_800_000_000_000 + nanoseconds

        self.assertEqual(
            Interval.MONTH1.advance(january_31_2024, 1),
            february_29_2024,
        )

    def test_market_event_smart_constructors_preserve_the_payload(self) -> None:
        market = Market.spot(Exchange.UPBIT, "BTC", "KRW")
        order_book = OrderBook(market, 1, [Level(Decimal("1"), Decimal("2"))], [])
        ticker = Ticker(
            market,
            2,
            None,
            Decimal("1"),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        candle = Candle(
            market,
            Interval.MIN1,
            3,
            Decimal("1"),
            Decimal("2"),
            Decimal("0.5"),
            Decimal("1.5"),
            Decimal("10"),
            None,
            True,
        )

        for event, kind, value in (
            (MarketEvent.order_book(order_book), "order_book", order_book),
            (MarketEvent.ticker(ticker), "ticker", ticker),
            (MarketEvent.candle(candle), "candle", candle),
        ):
            with self.subTest(kind=kind):
                self.assertEqual(event.kind, kind)
                self.assertIs(event.value, value)

    def test_decimal_and_nanosecond_timestamp_round_trip_exactly(self) -> None:
        self.assertIs(MaxtDecimal, Decimal)
        self.assertIs(Timestamp, int)
        payload = {
            "market": {
                "exchange": "upbit",
                "kind": "spot",
                "base": "BTC",
                "quote": "KRW",
            },
            "timestamp": 1_700_000_000_123_456_789,
            "price": "12345678901234567890.12345678",
            "quantity": "0.00000001",
            "taker_side": "buy",
            "id": "trade-1",
        }

        trade = _model_from_wire("Trade", payload)

        self.assertIsInstance(trade, Trade)
        self.assertIsInstance(trade.price, Decimal)
        self.assertEqual(trade.price, Decimal("12345678901234567890.12345678"))
        self.assertIs(type(trade.timestamp), int)
        self.assertEqual(trade.timestamp, 1_700_000_000_123_456_789)
        self.assertEqual(_model_to_wire(trade), payload)

    def test_request_wire_values_use_uppercase_assets_and_exact_timestamps(self) -> None:
        market = Market.spot(Exchange.UPBIT, "btc", "krw")
        request = CandleRequest(
            market=market,
            interval=Interval.MIN1,
            from_=1_700_000_000_123_456_789,
            limit=10,
        )

        self.assertEqual(market.base, "BTC")
        self.assertEqual(market.quote, "KRW")
        self.assertEqual(
            request.to_wire(),
            {
                "market": {
                    "exchange": "upbit",
                    "kind": "spot",
                    "base": "BTC",
                    "quote": "KRW",
                },
                "interval": "min1",
                "from": 1_700_000_000_123_456_789,
                "to": None,
                "limit": 10,
            },
        )

    def test_asset_normalization_uppercases_ascii_only(self) -> None:
        for value, expected in (
            ("éth", "éTH"),
            ("ıbtc", "ıBTC"),
            ("σbtc", "σBTC"),
            ("ßtc", "ßTC"),
        ):
            with self.subTest(value=value):
                market = Market.spot(Exchange.BINANCE, value, value)
                balance = Balance(value, Decimal("1"), Decimal("0"))
                margin = MarginSummary(value, None, None, None)

                self.assertEqual(market.base, expected)
                self.assertEqual(market.quote, expected)
                self.assertEqual(balance.asset, expected)
                self.assertEqual(margin.asset, expected)

    def test_stream_config_rejects_values_rust_unsigned_fields_cannot_hold(self) -> None:
        fields = (
            "max_reconnect_attempts",
            "initial_reconnect_delay_ms",
            "max_reconnect_delay_ms",
            "idle_timeout_ms",
            "buffer_size",
        )

        for field_name in fields:
            with self.subTest(field=field_name):
                with self.assertRaisesRegex(
                    ValueError,
                    f"^{field_name} must be non-negative$",
                ):
                    StreamConfig(**{field_name: -1})

    def test_public_market_data_wire_objects_become_typed_values(self) -> None:
        market_wire = {
            "exchange": "binance",
            "kind": "spot",
            "base": "BTC",
            "quote": "USDT",
        }
        info = _model_from_wire(
            "MarketInfo",
            {
                "market": market_wire,
                "native_symbol": "BTCUSDT",
                "status": "active",
                "english_name": "Bitcoin",
            },
        )
        book = _model_from_wire(
            "OrderBook",
            {
                "market": market_wire,
                "timestamp": 1_700_000_000_000_000_001,
                "bids": [{"price": "100.10", "quantity": "2.5"}],
                "asks": [{"price": "100.30", "quantity": "1.5"}],
            },
        )
        ticker = _model_from_wire(
            "Ticker",
            {
                "market": market_wire,
                "timestamp": 1_700_000_000_000_000_002,
                "last_trade_time": None,
                "last_price": "100.20",
                "change": "0.20",
                "change_rate": "0.002",
                "high": None,
                "low": None,
                "volume": "20",
                "quote_volume": None,
            },
        )
        candle = _model_from_wire(
            "Candle",
            {
                "market": market_wire,
                "interval": "min1",
                "open_time": 1_700_000_000_000_000_003,
                "open": "100",
                "high": "101",
                "low": "99",
                "close": "100.5",
                "volume": "12.25",
                "quote_volume": None,
                "closed": True,
            },
        )

        self.assertIsInstance(info, MarketInfo)
        self.assertEqual(info.status, MarketStatus.ACTIVE)
        self.assertIsNone(info.korean_name)
        self.assertIsInstance(book, OrderBook)
        self.assertEqual(book.spread(), Decimal("0.2"))
        self.assertIsInstance(ticker, Ticker)
        self.assertEqual(ticker.last_price, Decimal("100.20"))
        self.assertIsInstance(candle, Candle)
        self.assertEqual(candle.open_time, 1_700_000_000_000_000_003)

    def test_private_order_position_and_margin_wire_objects_are_typed(self) -> None:
        market_wire = {
            "exchange": "binance",
            "kind": "perpetual",
            "base": "BTC",
            "quote": "USDT",
        }
        order = _model_from_wire(
            "Order",
            {
                "id": "42",
                "market": market_wire,
                "side": "buy",
                "status": "partially_filled",
                "filled_quantity": "0.25",
                "remaining_quantity": "0.75",
                "price": "50000.125",
                "created_at": 1_700_000_000_000_000_004,
            },
        )
        position = _model_from_wire(
            "Position",
            {
                "market": market_wire,
                "side": "sell",
                "quantity": "1.5",
                "entry_price": "51000",
                "mark_price": "50500",
                "notional": "75750",
                "unrealized_pnl": "750",
                "leverage": "3",
                "margin_mode": "isolated",
            },
        )
        margin = _model_from_wire(
            "MarginSummary",
            {
                "asset": "USDT",
                "equity": "1000.01",
                "margin_balance": "900",
                "available_balance": None,
            },
        )

        self.assertIsInstance(order, Order)
        self.assertEqual(order.status, OrderStatus.PARTIALLY_FILLED)
        self.assertEqual(order.price, Decimal("50000.125"))
        self.assertIsInstance(position, Position)
        self.assertEqual(position.quantity, Decimal("1.5"))
        self.assertFalse(position.is_flat())
        self.assertIsInstance(margin, MarginSummary)
        self.assertEqual(margin.equity, Decimal("1000.01"))

    def test_history_pages_keep_typed_items_and_opaque_cursors(self) -> None:
        market_wire = {
            "exchange": "hyperliquid",
            "kind": "perpetual",
            "base": "ETH",
            "quote": "USDC",
        }
        rates = _model_from_wire(
            "FundingRatePage",
            {
                "items": [
                    {
                        "market": market_wire,
                        "timestamp": 1_700_000_000_000_000_005,
                        "rate": "0.0001",
                        "mark_price": "3500.25",
                    }
                ],
                "next": "opaque-rate-cursor",
            },
        )
        payments = _model_from_wire(
            "FundingPaymentPage",
            {
                "items": [
                    {
                        "market": market_wire,
                        "timestamp": 1_700_000_000_000_000_006,
                        "amount": "-1.25",
                        "rate": None,
                        "id": "payment-1",
                    }
                ],
                "next": None,
            },
        )

        self.assertIsInstance(rates, Page)
        self.assertIsInstance(rates.items[0], FundingRate)
        self.assertEqual(rates.next.as_str(), "opaque-rate-cursor")
        self.assertTrue(rates.has_more())
        self.assertIsInstance(payments.items[0], FundingPayment)
        self.assertFalse(payments.has_more())

        request = HistoryRequest(
            market=rates.items[0].market,
            from_=1_700_000_000_000_000_000,
            cursor=rates.next,
            limit=100,
        )
        self.assertEqual(request.to_wire()["cursor"], "opaque-rate-cursor")
        self.assertEqual(request.to_wire()["from"], 1_700_000_000_000_000_000)

    def test_order_and_margin_requests_preserve_decimal_scale_on_wire(self) -> None:
        market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
        market_order = OrderRequest.market_order(
            market,
            Side.BUY,
            Size.quote(Decimal("100.00")),
            reduce_only=True,
        )
        limit_order = OrderRequest.limit_order(
            market,
            Side.SELL,
            Size.base(Decimal("0.25")),
            Decimal("50000.2500"),
            time_in_force=TimeInForce.POST_ONLY,
        )
        best_order = OrderRequest.best_order(
            Market.spot(Exchange.BITHUMB, "BTC", "KRW"),
            Side.BUY,
            Size.quote(Decimal("10000")),
            TimeInForce.IMMEDIATE_OR_CANCEL,
            client_id="client-1",
        )
        margin = MarginRequest(
            market,
            leverage=Decimal("3.0"),
            margin_mode=MarginMode.ISOLATED,
        )

        self.assertEqual(market_order.order_type, OrderType.MARKET)
        self.assertEqual(
            market_order.to_wire(),
            {
                "market": market.to_wire(),
                "side": "buy",
                "order_type": "market",
                "size": {"kind": "quote", "value": "100.00"},
                "price": None,
                "time_in_force": None,
                "reduce_only": True,
                "client_id": None,
            },
        )
        self.assertEqual(limit_order.to_wire()["price"], "50000.2500")
        self.assertEqual(limit_order.to_wire()["time_in_force"], "post_only")
        self.assertEqual(best_order.to_wire()["order_type"], "best")
        self.assertEqual(best_order.to_wire()["client_id"], "client-1")
        self.assertEqual(margin.to_wire()["leverage"], "3.0")

        with self.assertRaisesRegex(ValueError, "time_in_force"):
            OrderRequest(
                Market.spot(Exchange.UPBIT, "BTC", "KRW"),
                Side.BUY,
                OrderType.BEST,
                Size.quote(Decimal("10000")),
            )

    def test_generated_wallet_models_round_trip_tagged_values(self) -> None:
        destination = TransferDestination.chain(
            ChainDestination("éth", Network.ARBITRUM, "0xabc")
        )
        request = WithdrawRequest(
            "eth",
            Network.ARBITRUM,
            Decimal("1.25"),
            destination,
        )

        self.assertEqual(request.asset, "ETH")
        self.assertEqual(
            request.to_wire()["destination"],
            {
                "kind": "chain",
                "value": {
                    "asset": "éTH",
                    "network": "arbitrum",
                    "address": "0xabc",
                    "memo": None,
                },
            },
        )
        history = TransferHistoryRequest(asset="btc", network=Network.other("custom"))
        self.assertEqual(history.asset, "BTC")
        self.assertEqual(history.network.value, "custom")
        fee = WithdrawalFee.from_wire(
            {"kind": "rate", "rate": "0.01", "minimum": None, "maximum": "1"}
        )
        self.assertEqual(fee.to_wire()["maximum"], "1")
        self.assertEqual(
            TravelRuleRequirement.required("https://example.test").to_wire()["kind"],
            "required",
        )


if __name__ == "__main__":
    unittest.main()
