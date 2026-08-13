import unittest
from decimal import Decimal
from importlib import import_module
from types import SimpleNamespace
from unittest.mock import patch

from maxt import (
    Adapter,
    AuthError,
    BithumbAdapter,
    BithumbAlertStep,
    BithumbApiKey,
    BithumbAssetFee,
    BithumbClosedOrder,
    BithumbClosedOrderState,
    BithumbClosedOrdersRequest,
    BithumbKrwDepositsRequest,
    BithumbKrwTransferRequest,
    BithumbKrwWithdrawalsRequest,
    BithumbMarketAlert,
    BithumbNetworkFee,
    BithumbNotice,
    BithumbOrderDetail,
    BithumbOrderDirection,
    BithumbOrderDetailRequest,
    BithumbOrderListItem,
    BithumbOrderListRequest,
    BithumbOrderListState,
    BithumbPendingOrderState,
    BithumbPendingOrdersRequest,
    BithumbTwapOrder,
    BithumbTwapOrderDirection,
    BithumbTwapOrderRequest,
    BithumbTwapOrdersRequest,
    BithumbTwapState,
    BithumbWithdrawalAddress,
    BinanceAdapter,
    BinanceAccountTrade,
    BinanceC2cTradeHistoryPage,
    BinanceC2cTradeHistoryRequest,
    BinanceC2cTradeType,
    BinanceListenKey,
    BinanceMarket,
    BinanceMarkPrice,
    BinanceOpenInterest,
    BinanceSpotAveragePrice,
    BinanceSpotOrderDetail,
    BinanceSymbolFilters,
    BinanceTestOrder,
    BinanceTestOrderRequest,
    Client,
    Cursor,
    Exchange,
    Feature,
    HyperliquidAdapter,
    HyperliquidAssetContext,
    HyperliquidDailyVolume,
    HyperliquidLedgerEntry,
    HyperliquidLedgerKind,
    HyperliquidMidPrice,
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
    InvalidRequestError,
    Market,
    MarketEvent,
    Network,
    OrderRequest,
    OrderStatus,
    Side,
    Size,
    StreamError,
    StreamEvent,
    StreamConfig,
    Subscription,
    UnsupportedError,
    Feed,
    Trade,
    UpbitAdapter,
    UpbitApiKey,
    UpbitBatchCancelRequest,
    UpbitBatchCancelScope,
    UpbitMarketEvent,
    UpbitKrwDeposit,
    UpbitKrwTransferRequest,
    UpbitKrwTwoFactorType,
    UpbitKrwWithdrawal,
    UpbitOrderBookInstrument,
    UpbitClosedOrder,
    UpbitClosedOrderState,
    UpbitClosedOrdersRequest,
    UpbitOrderDetail,
    UpbitOrderDetailRequest,
    UpbitOrderDirection,
    UpbitPocket,
    UpbitPocketApiKeysRequest,
    UpbitPocketBalance,
    UpbitPocketTransfer,
    UpbitPocketTransferQuery,
    UpbitPocketTransferRequest,
    UpbitPocketTransferState,
    UpbitPocketUniversalTransferRequest,
    UpbitRegion,
    UpbitTravelRuleVasp,
    UpbitTravelRuleVerification,
    UpbitYearCandle,
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

    async def tickers_by_quote(self, quote_currencies):
        self.quote_currencies = quote_currencies
        return [
            {
                "market": MARKET_WIRE,
                "timestamp": 1_700_000_000_000_000_012,
                "last_trade_time": 1_700_000_000_000_000_011,
                "last_price": "50000000.0100",
                "change": None,
                "change_rate": None,
                "high": None,
                "low": None,
                "volume": None,
                "quote_volume": None,
            }
        ]

    async def order_books_at_level(self, markets, level, depth=None):
        self.order_book_level_args = (markets, level, depth)
        return [
            {
                "market": MARKET_WIRE,
                "timestamp": 1_700_000_000_000_000_013,
                "bids": [{"price": "50000000", "quantity": "0.1"}],
                "asks": [{"price": "50001000", "quantity": "0.2"}],
            }
        ]

    async def year_candles(self, market, to=None, count=None):
        self.year_candle_args = (market, to, count)
        return [
            {
                "market": MARKET_WIRE,
                "open_time": 1_767_225_600_000_000_000,
                "korea_open_time": 1_767_225_600_000_000_000,
                "timestamp": 1_786_467_753_786_000_000,
                "open": "128000000.00000000",
                "high": "143050000.00000000",
                "low": "88770000.00000000",
                "close": "89587000.00000000",
                "volume": "348666.78732189",
                "quote_volume": "37189906239683.17623000",
                "first_day_of_period": "2026-01-01",
            }
        ]

    async def orderbook_instruments(self, markets):
        self.instrument_markets = markets
        return [
            {
                "market": MARKET_WIRE,
                "quote_currency": "KRW",
                "tick_size": "1000",
                "supported_levels": ["0", "10000"],
            }
        ]

    async def test_order(self, request):
        self.test_order_request = request
        return {
            "id": "dry-run-order",
            "market": MARKET_WIRE,
            "side": "buy",
            "status": "accepted",
            "filled_quantity": "0",
            "remaining_quantity": "0.01",
            "price": "100000000",
            "created_at": 1_700_000_000_000_000_014,
        }

    async def order_detail(self, request):
        self.order_detail_request = request
        market = request["market"]
        return {
            "market": market,
            "uuid": request.get("uuid") or "upbit-order-detail-1",
            "side": "bid",
            "order_type": "limit",
            "price": "100000000",
            "state": "done",
            "created_at": 1_700_000_000_000_000_014,
            "volume": "0.01",
            "remaining_volume": "0",
            "executed_volume": "0.01",
            "reserved_fee": "5000",
            "remaining_fee": "0",
            "paid_fee": "5000",
            "locked": "0",
            "trades_count": 1,
            "prevented_volume": "0",
            "prevented_locked": "0",
            "time_in_force": "ioc",
            "identifier": request.get("identifier"),
            "smp_type": None,
            "trades": [
                {
                    "market": market,
                    "uuid": "upbit-fill-1",
                    "price": "100000000",
                    "volume": "0.01",
                    "funds": "1000000",
                    "trend": "up",
                    "created_at": 1_700_000_000_000_000_015,
                    "side": "bid",
                }
            ],
        }

    async def closed_orders(self, request):
        self.closed_orders_request = request
        return [
            {
                "market": request["market"] or MARKET_WIRE,
                "uuid": "upbit-closed-order-1",
                "side": "bid",
                "ord_type": "limit",
                "state": "done",
                "created_at": 1_700_000_000_000_000_016,
                "volume": "0.01",
                "price": "100000000",
                "remaining_volume": "0",
                "executed_volume": "0.01",
                "executed_funds": "1000000",
                "reserved_fee": "5000",
                "remaining_fee": "0",
                "paid_fee": "5000",
                "locked": "0",
                "trades_count": 1,
                "prevented_volume": "0",
                "prevented_locked": "0",
                "time_in_force": "ioc",
                "identifier": "closed-client-1",
                "smp_type": None,
            }
        ]

    async def deposit_info(self, asset, network):
        self.deposit_info_args = (asset, network)
        return {
            "asset": "BTC",
            "network": "bitcoin",
            "provider_network": "BTC",
            "is_deposit_possible": True,
            "deposit_impossible_reason": None,
            "minimum_deposit_amount": "0.0005",
            "minimum_deposit_confirmations": 18_446_744_073_709_551_615,
            "decimal_precision": 18_446_744_073_709_551_615,
        }

    async def travel_rule_vasps(self):
        return [
            {
                "vasp_name": "Upbit Singapore",
                "vasp_uuid": "vasp-1",
                "depositable": True,
                "withdrawable": False,
            }
        ]

    async def verify_travel_rule_by_uuid(self, deposit_uuid, vasp_uuid):
        self.travel_rule_uuid_args = (deposit_uuid, vasp_uuid)
        return {
            "deposit_uuid": deposit_uuid,
            "deposit_state": "ACCEPTED",
            "verification_result": "verified",
        }

    async def verify_travel_rule_by_txid(self, txid, vasp_uuid, currency, net_type):
        self.travel_rule_txid_args = (txid, vasp_uuid, currency, net_type)
        return {
            "deposit_uuid": "deposit-from-txid",
            "deposit_state": "PROCESSING",
            "verification_result": "pending",
        }

    async def batch_cancel_open_orders(self, request):
        self.batch_cancel_open_orders_request = request
        return {
            "cancelled": [
                {
                    "order_id": "done-1",
                    "client_id": "client-1",
                    "market": MARKET_WIRE,
                    "cancelled_at": None,
                }
            ],
            "failed": [
                {
                    "order_id": "failed-1",
                    "client_id": None,
                    "market": MARKET_WIRE,
                    "code": None,
                    "message": None,
                }
            ],
        }

    async def deposit_krw(self, request):
        self.deposit_krw_request = request
        return {
            "transfer_type": "deposit",
            "uuid": "upbit-krw-deposit-1",
            "currency": "KRW",
            "net_type": None,
            "txid": "upbit-krw-deposit-txid",
            "state": "PROCESSING",
            "created_at": 1_700_000_000_000_000_015,
            "done_at": None,
            "amount": "20000",
            "fee": "0",
            "transaction_type": "default",
        }

    async def withdraw_krw(self, request):
        self.withdraw_krw_request = request
        return {
            "transfer_type": "withdraw",
            "uuid": "upbit-krw-withdrawal-1",
            "currency": "KRW",
            "net_type": None,
            "txid": None,
            "state": "PROCESSING",
            "created_at": 1_700_000_000_000_000_016,
            "done_at": None,
            "amount": "20000",
            "fee": "0",
            "transaction_type": "default",
            "is_cancelable": True,
        }

    async def api_keys(self):
        return [
            {
                "access_key": "upbit-access-key-1",
                "expires_at": 1_812_672_000_000_000_000,
            }
        ]

    async def list_pockets(self):
        return [{"uuid": "pocket-main", "name": "Main", "kind": "main"}]

    async def list_pocket_api_keys(self, request):
        self.pocket_api_keys_request = request
        return [
            {
                "uuid": "pocket-main",
                "keys": [
                    {
                        "access_key": "pocket-access-key-1",
                        "permissions": ["View Accounts"],
                        "allowed_ips": ["203.0.113.1"],
                        "created_at": 1_700_000_000_000_000_017,
                        "expired_at": 1_812_672_000_000_000_000,
                    }
                ],
            }
        ]

    async def sub_pocket_balances(self, pocket_uuid):
        self.sub_pocket_uuid = pocket_uuid
        return [
            {
                "currency": "BTC",
                "balance": "0.1",
                "locked": "0.01",
                "avg_buy_price": "50000000",
                "avg_buy_price_modified": False,
                "unit_currency": "KRW",
            }
        ]

    @staticmethod
    def _pocket_transfer(request):
        return {
            "uuid": "pocket-transfer-1",
            "identifier": request.get("identifier"),
            "from": request.get("from") or "pocket-main",
            "to": request["to"],
            "state": "done",
            "currency": request.get("currency", "BTC"),
            "amount": request.get("amount", "0.01"),
            "created_at": 1_700_000_000_000_000_018,
        }

    async def universal_transfer(self, request):
        self.universal_transfer_request = request
        return self._pocket_transfer(request)

    async def universal_transfers(self, request):
        self.universal_transfers_request = request
        return [self._pocket_transfer({"to": "pocket-sub", "amount": "0.01"})]

    async def sub_pocket_transfer(self, request):
        self.sub_pocket_transfer_request = request
        return self._pocket_transfer(request)

    async def sub_pocket_transfers(self, request):
        self.sub_pocket_transfers_request = request
        return [self._pocket_transfer({"to": "pocket-main", "amount": "0.01"})]

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

    async def notices(self, count=None):
        self.notice_count = count
        return [
            {
                "categories": ["입출금"],
                "title": "네트워크 점검 안내",
                "url": "https://feed.bithumb.com/notice/1654458",
                "published_at": 1_700_000_000_000_000_009,
                "modified_at": 1_700_000_000_000_000_010,
            }
        ]

    async def transfer_fees(self, currency):
        self.fee_currency = currency
        return [
            {
                "display_name": "비트코인",
                "asset": "BTC",
                "networks": [
                    {
                        "network": "bitcoin",
                        "provider_name": "Bitcoin",
                        "deposit_fee": "0",
                        "minimum_deposit": "0",
                        "withdrawal_fee": {"kind": "fixed", "value": "0.0002"},
                        "minimum_withdrawal": "0.001",
                    }
                ],
            }
        ]

    async def api_keys(self):
        return [
            {
                "access_key": "example-access-key-1",
                "expires_at": 1_812_672_000_000_000_000,
            }
        ]

    async def withdrawal_addresses(self):
        return [
            {
                "currency": "BTC",
                "net_type": "BTC",
                "network_name": "Bitcoin",
                "withdraw_address": "bc1example",
                "secondary_address": None,
                "exchange_name": "Bithumb",
                "owner_type": "individual",
                "owner_ko_name": "홍길동",
                "owner_en_name": "Hong Gildong",
                "owner_corp_ko_name": None,
                "owner_corp_en_name": None,
            }
        ]

    async def order_detail(self, request):
        self.order_detail_request = request
        market = request["market"]
        return {
            "uuid": request.get("uuid") or "order-detail-1",
            "client_order_id": request.get("client_order_id"),
            "side": "bid",
            "order_type": "limit",
            "price": "50000000",
            "state": "done",
            "market": market,
            "created_at": 1_700_000_000_000_000_019,
            "volume": "0.1",
            "remaining_volume": "0",
            "reserved_fee": "2500",
            "remaining_fee": "0",
            "paid_fee": "2500",
            "locked": "0",
            "executed_volume": "0.1",
            "executed_funds": "5000000",
            "trades_count": 1,
            "trades": [
                {
                    "market": market,
                    "uuid": "trade-detail-1",
                    "price": "50000000",
                    "volume": "0.1",
                    "funds": "5000000",
                    "side": "bid",
                    "created_at": 1_700_000_000_000_000_020,
                }
            ],
            "stp_type": None,
            "cancel_type": None,
            "canceling_uuid": None,
            "time_in_force": "ioc",
        }

    async def order_list(self, request):
        self.order_list_request = request
        return [
            {
                "uuid": "order-list-1",
                "client_order_id": "client-list-1",
                "side": "bid",
                "order_type": "limit",
                "price": "50000000",
                "state": "wait",
                "market": request["market"] or {**MARKET_WIRE, "exchange": "bithumb"},
                "created_at": 1_700_000_000_000_000_021,
                "volume": "0.1",
                "remaining_volume": "0.1",
                "reserved_fee": "2500",
                "remaining_fee": "2500",
                "paid_fee": "0",
                "locked": "5002500",
                "executed_volume": "0",
                "executed_funds": "0",
                "trades_count": 0,
                "stp_type": None,
                "time_in_force": "post_only",
            }
        ]

    async def krw_withdrawals(self, request):
        self.krw_withdrawals_request = request
        return [self._krw_transfer("withdraw")]

    async def withdraw_krw(self, request):
        self.withdraw_krw_request = request
        return self._krw_transfer("withdraw")

    async def krw_deposits(self, request):
        self.krw_deposits_request = request
        return [self._krw_transfer("deposit")]

    async def deposit_krw(self, request):
        self.deposit_krw_request = request
        return self._krw_transfer("deposit")

    @staticmethod
    def _krw_transfer(transfer_type):
        return {
            "transfer_type": transfer_type,
            "uuid": "krw-transfer-1",
            "currency": "KRW",
            "net_type": None,
            "txid": None,
            "state": "PROCESSING",
            "created_at": 1_700_000_000_000_000_000,
            "done_at": None,
            "amount": "10000",
            "fee": "0",
            "transaction_type": "default",
        }

    async def pending_orders(self, request):
        self.pending_request = request
        return {"items": [], "next": "page+/=="}

    async def closed_orders(self, request):
        self.closed_request = request
        return {
            "items": [
                {
                    "order_id": "closed-order-1",
                    "side": "bid",
                    "order_type": "limit",
                    "price": "50000000",
                    "state": "cancel",
                    "market": {**MARKET_WIRE, "exchange": "bithumb"},
                    "created_at": 1_700_000_000_000_000_022,
                    "volume": "0.1",
                    "remaining_volume": "0.1",
                    "reserved_fee": "2500",
                    "remaining_fee": "2500",
                    "paid_fee": "0",
                    "locked": "5002500",
                    "executed_volume": "0",
                    "executed_funds": "0",
                    "trades_count": 0,
                    "client_order_id": "client-closed-1",
                    "stp_type": "cancel_maker",
                    "time_in_force": "ioc",
                    "cancel_type": "user",
                    "canceling_order_id": "canceling-order-1",
                }
            ],
            "next": "closed+/==",
        }

    async def twap_orders(self, request):
        self.twap_request = request
        return {
            "items": [
                {
                    "id": "twap-1",
                    "side": "buy",
                    "price": "50000000.0000",
                    "state": "progress",
                    "market": {**MARKET_WIRE, "exchange": "bithumb"},
                    "created_at": 1_700_000_000_000_000_011,
                    "volume": "0.1",
                    "finished_at": None,
                    "total_order_count": 10,
                    "total_trades_count": 2,
                    "progress_count": 3,
                    "total_executed_amount": "1000000.00",
                    "total_executed_volume": "0.02",
                    "avg_trade_price": "50000000.0000",
                    "wallet_id": "wallet-1",
                    "canceled_at": None,
                    "cancel_type": None,
                }
            ],
            "next": None,
        }

    async def create_twap_order(self, request):
        self.create_twap_request = request
        return "twap-created"

    async def cancel_twap_order(self, algo_order_id):
        self.cancel_twap_id = algo_order_id
        return "twap-cancelled"


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

    async def spot_average_price(self, market):
        return {
            "market": market.to_wire(),
            "minutes": 7,
            "price": "9.357518340000000000",
            "close_time": 1_694_061_154_503_000_000,
        }

    async def mark_price(self, market):
        return {
            "market": market.to_wire(),
            "mark_price": "50001.25",
            "index_price": "50000.75",
            "estimated_settle_price": None,
            "last_funding_rate": "0.0001",
            "interest_rate": "0.0001",
            "next_funding_time": 1_700_000_000_000_000_011,
            "time": 1_700_000_000_000_000_012,
        }

    async def mark_prices(self):
        return [
            await self.mark_price(
                Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
            )
        ]

    async def open_interest(self, market):
        return {
            "market": market.to_wire(),
            "open_interest": "1234.5",
            "time": 1_700_000_000_000_000_013,
        }

    async def account_trades(self, request):
        self.account_trades_request = request
        return {
            "items": [
                {
                    "market": request["market"],
                    "id": "account-trade-1",
                    "order_id": "order-1",
                    "timestamp": 1_700_000_000_000_000_014,
                    "side": "buy",
                    "maker": False,
                    "best_match": True,
                    "order_list_id": None,
                    "price": "50000",
                    "quantity": "0.1",
                    "quote_quantity": "5000",
                    "commission": "0.005",
                    "commission_asset": "BNB",
                    "realized_pnl": None,
                    "position_side": None,
                    "pair": None,
                    "base_quantity": None,
                    "margin_asset": None,
                }
            ],
            "next": None,
        }

    async def c2c_trade_history(self, request):
        self.c2c_trade_history_request = request
        return {
            "code": "000000",
            "message": None,
            "data": [
                {
                    "order_number": "order-c2c-1",
                    "adv_no": "adv-1",
                    "trade_type": "BUY",
                    "asset": "usdt",
                    "fiat": "KRW",
                    "fiat_symbol": "₩",
                    "amount": "100",
                    "total_price": "150000",
                    "unit_price": "1500",
                    "order_status": "COMPLETED",
                    "created_at": 1_700_000_000_000_000_021,
                    "commission": "0",
                    "counterparty_nickname": "counterparty",
                    "pay_method_name": "Bank",
                    "additional_kyc_verify": 2,
                    "taker_commission_rate": None,
                    "taker_commission": None,
                    "taker_amount": None,
                    "advertisement_role": "TAKER",
                }
            ],
            "total": 1,
            "success": True,
        }

    async def test_order(self, request):
        self.test_order_request = request
        return {"response_json": "{}"}

    async def cancel_all_open_orders(self, market):
        self.cancel_all_open_orders_market = market

    async def usd_m_create_listen_key(self):
        return FakeNativeBinanceListenKey()

    async def usd_m_keepalive_listen_key(self):
        self.kept_alive = True

    async def usd_m_close_listen_key(self):
        self.closed = True


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

    async def all_mids(self):
        return [
            {
                "market": {
                    "exchange": "hyperliquid",
                    "kind": "perpetual",
                    "base": "BTC",
                    "quote": "USDC",
                },
                "price": "50000.5",
            }
        ]

    async def basic_open_orders(self):
        return [
            {
                "coin": "ETH",
                "limit_price": "3500.2500",
                "order_id": 42,
                "side": "B",
                "size": "0.1000",
                "timestamp": 1_700_000_000_000_000_013,
                "raw_json": '{"coin":"ETH"}',
            }
        ]

    async def order_status(self, reference):
        self.order_status_reference = reference
        return {"kind": "order", "value": self._order_info()}

    async def historical_orders(self):
        return [self._order_info()]

    @staticmethod
    def _order_info():
        return {
            "order": {
                "coin": "ETH",
                "side": "B",
                "limit_price": "3500.2500",
                "size": "0.1000",
                "order_id": 42,
                "timestamp": 1_700_000_000_000_000_013,
                "trigger_condition": "N/A",
                "is_trigger": False,
                "trigger_price": "0",
                "children_json": "[]",
                "is_position_tpsl": False,
                "reduce_only": True,
                "order_type": "Limit",
                "original_size": "0.1000",
                "time_in_force": "Gtc",
                "client_order_id": "0x0123456789abcdef0123456789abcdef",
                "raw_json": '{"oid":42}',
            },
            "status": "open",
            "status_timestamp": 1_700_000_000_000_000_014,
            "raw_json": '{"status":"open"}',
        }

    async def user_fills(self, aggregate_by_time):
        self.user_fills_args = (aggregate_by_time,)
        return [self._user_fill()]

    async def user_fills_by_time(self, from_ns, to_ns=None, aggregate_by_time=False):
        self.user_fills_by_time_args = (from_ns, to_ns, aggregate_by_time)
        return [self._user_fill()]

    @staticmethod
    def _user_fill():
        return {
            "coin": "ETH",
            "price": "3500.25",
            "size": "0.1",
            "side": "B",
            "time": 1_700_000_000_000_000_012,
            "start_position": "0",
            "direction": "Open Long",
            "closed_pnl": "0",
            "hash": "0xfill",
            "order_id": 42,
            "crossed": False,
            "fee": "0.01",
            "builder_fee": None,
            "trade_id": 7,
            "fee_token": "USDC",
            "twap_id": None,
            "raw_json": "{}",
        }

    async def user_rate_limit(self):
        return {
            "cumulative_volume": "1000",
            "requests_used": 12,
            "requests_cap": 1000,
            "requests_surplus": 5,
        }

    async def user_role(self):
        return {"kind": "agent", "user": "0xuser"}

    async def referral(self):
        return {
            "referred_by": {"address": "0xreferrer", "code": "REF"},
            "cumulative_volume": "1000",
            "unclaimed_rewards": "1",
            "claimed_rewards": "2",
            "builder_rewards": "3",
            "referrer_state_json": "{}",
            "reward_history_json": "[]",
            "token_to_state_json": "{}",
        }

    async def user_fees(self):
        return {
            "daily_volumes": [
                {
                    "date": "2026-08-12",
                    "user_cross": "1",
                    "user_add": "2",
                    "exchange": "3",
                }
            ],
            "fee_schedule_json": "{}",
            "user_cross_rate": "0.0004",
            "user_add_rate": "0.0001",
            "user_spot_cross_rate": None,
            "user_spot_add_rate": None,
            "active_referral_discount": None,
            "details_json": "{}",
        }

    async def portfolio(self):
        return [
            {
                "period": "day",
                "account_value_history": [
                    {"time": 1_700_000_000_000_000_000, "value": "100"}
                ],
                "pnl_history": [
                    {"time": 1_700_000_000_000_000_000, "value": "2"}
                ],
                "volume": "10",
            }
        ]

    async def sub_accounts(self):
        return [
            {
                "name": "sub",
                "user": "0xsub",
                "master": "0xmaster",
                "perpetual_state_json": "{}",
                "spot_state_json": "{}",
            }
        ]

    async def user_vault_equities(self):
        return [
            {
                "vault_address": "0xvault",
                "equity": "25",
                "locked_until": None,
            }
        ]


class BuiltinAdapterTests(unittest.IsolatedAsyncioTestCase):
    async def test_native_client_rejects_empty_subscriptions_before_dispatch(self) -> None:
        native = SimpleNamespace(NativeUpbitAdapter=FakeNativeUpbitAdapter)
        with patch("maxt._api._load_native", return_value=native):
            client = Client(UpbitAdapter())
            market = Market.spot(Exchange.UPBIT, "BTC", "KRW")

            with self.assertRaises(InvalidRequestError) as empty_markets:
                await client.adapter.subscribe(Subscription((), ()), StreamConfig())
            with self.assertRaises(InvalidRequestError) as empty_feeds:
                await client.adapter.subscribe(
                    Subscription((market,), ()),
                    StreamConfig(),
                )

        self.assertEqual(empty_markets.exception.field, "markets")
        self.assertEqual(empty_feeds.exception.field, "feeds")

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
            quote_tickers = await adapter.tickers_by_quote(["KRW"])
            aggregated_books = await adapter.order_books_at_level(
                [market], Decimal("100000.0"), depth=2
            )
            annual = await adapter.year_candles(market, to=1_767_225_600_000_000_000, count=2)
            policies = await adapter.orderbook_instruments([market])
            test_order = await adapter.test_order(
                OrderRequest.limit_order(
                    market,
                    Side.BUY,
                    Size.base(Decimal("0.01")),
                    Decimal("100000000"),
                )
            )
            order_detail = await adapter.order_detail(
                UpbitOrderDetailRequest(
                    market,
                    uuid="upbit-order-detail-1",
                    identifier="upbit-client-1",
                )
            )
            closed_orders = await adapter.closed_orders(
                UpbitClosedOrdersRequest(
                    market,
                    None,
                    [UpbitClosedOrderState.DONE, UpbitClosedOrderState.CANCEL],
                    start_time=1_700_000_000_000_000_000,
                    end_time=1_700_001_000_000_000_000,
                    limit=1000,
                    order_by=UpbitOrderDirection.ASCENDING,
                )
            )
            deposit_info = await adapter.deposit_info("BTC", Network.BITCOIN)
            vasps = await adapter.travel_rule_vasps()
            verification_by_uuid = await adapter.verify_travel_rule_by_uuid(
                "deposit-1", "vasp-1"
            )
            verification_by_txid = await adapter.verify_travel_rule_by_txid(
                "tx-1", "vasp-1", "BTC", "BTC"
            )
            batch_result = await adapter.batch_cancel_open_orders(
                UpbitBatchCancelRequest(
                    scope=UpbitBatchCancelScope.quote_currencies(["KRW"]),
                    excluded_pairs=[market],
                    side=Side.BUY,
                    count=20,
                    order_by=UpbitOrderDirection.ASCENDING,
                )
            )
            krw_request = UpbitKrwTransferRequest(
                Decimal("20000"),
                UpbitKrwTwoFactorType.KAKAO,
            )
            krw_deposit = await adapter.deposit_krw(krw_request)
            krw_withdrawal = await adapter.withdraw_krw(krw_request)
            api_keys = await adapter.api_keys()
            pockets = await adapter.list_pockets()
            pocket_api_keys = await adapter.list_pocket_api_keys(
                UpbitPocketApiKeysRequest(["pocket-main"], False)
            )
            balances = await adapter.sub_pocket_balances("pocket-sub")
            query = UpbitPocketTransferQuery(
                from_=None,
                to=None,
                direction=None,
                states=[UpbitPocketTransferState.DONE],
                uuids=[],
                identifiers=[],
            )
            universal_transfer = await adapter.universal_transfer(
                UpbitPocketUniversalTransferRequest(
                    from_="pocket-main",
                    to="pocket-sub",
                    currency="BTC",
                    amount=Decimal("0.01"),
                    identifier="universal-1",
                )
            )
            universal_transfers = await adapter.universal_transfers(query)
            sub_transfer = await adapter.sub_pocket_transfer(
                UpbitPocketTransferRequest(
                    to="pocket-main",
                    currency="BTC",
                    amount=Decimal("0.01"),
                    identifier="sub-1",
                )
            )
            sub_transfers = await adapter.sub_pocket_transfers(query)

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
        self.assertEqual(adapter._handle.quote_currencies, ["KRW"])
        self.assertEqual(quote_tickers[0].last_price, Decimal("50000000.0100"))
        self.assertEqual(adapter._handle.order_book_level_args[1], "100000.0")
        self.assertEqual(adapter._handle.order_book_level_args[2], 2)
        self.assertEqual(aggregated_books[0].asks[0].price, Decimal("50001000"))
        self.assertIsInstance(annual[0], UpbitYearCandle)
        self.assertEqual(annual[0].korea_open_time, annual[0].open_time)
        self.assertEqual(annual[0].quote_volume, Decimal("37189906239683.17623000"))
        self.assertIsInstance(policies[0], UpbitOrderBookInstrument)
        self.assertEqual(policies[0].supported_levels, [Decimal("0"), Decimal("10000")])
        self.assertEqual(adapter._handle.test_order_request["order_type"], "limit")
        self.assertEqual(test_order.id, "dry-run-order")
        self.assertIs(test_order.status, OrderStatus.ACCEPTED)
        self.assertEqual(adapter._handle.order_detail_request["uuid"], "upbit-order-detail-1")
        self.assertIsInstance(order_detail, UpbitOrderDetail)
        self.assertEqual(order_detail.trades[0].funds, Decimal("1000000"))
        self.assertEqual(adapter._handle.closed_orders_request["states"], ["done", "cancel"])
        self.assertEqual(adapter._handle.closed_orders_request["order_by"], "asc")
        self.assertIsInstance(closed_orders[0], UpbitClosedOrder)
        self.assertEqual(closed_orders[0].executed_funds, Decimal("1000000"))
        self.assertFalse(hasattr(closed_orders[0], "trades"))
        self.assertEqual(adapter._handle.deposit_info_args, ("BTC", Network.BITCOIN))
        self.assertTrue(deposit_info.is_deposit_possible)
        self.assertEqual(deposit_info.minimum_deposit_amount, Decimal("0.0005"))
        self.assertEqual(
            deposit_info.minimum_deposit_confirmations,
            18_446_744_073_709_551_615,
        )
        self.assertEqual(deposit_info.decimal_precision, 18_446_744_073_709_551_615)
        self.assertIsInstance(vasps[0], UpbitTravelRuleVasp)
        self.assertEqual(vasps[0].vasp_uuid, "vasp-1")
        self.assertIsInstance(verification_by_uuid, UpbitTravelRuleVerification)
        self.assertEqual(verification_by_uuid.deposit_state, "ACCEPTED")
        self.assertEqual(
            adapter._handle.travel_rule_uuid_args, ("deposit-1", "vasp-1")
        )
        self.assertEqual(
            adapter._handle.travel_rule_txid_args,
            ("tx-1", "vasp-1", "BTC", "BTC"),
        )
        self.assertEqual(verification_by_txid.verification_result, "pending")
        self.assertEqual(
            adapter._handle.batch_cancel_open_orders_request["scope"],
            {"kind": "quote_currencies", "values": ["KRW"]},
        )
        self.assertEqual(batch_result.cancelled[0].order_id, "done-1")
        self.assertEqual(batch_result.failed[0].order_id, "failed-1")
        self.assertIsInstance(krw_deposit, UpbitKrwDeposit)
        self.assertIsInstance(krw_withdrawal, UpbitKrwWithdrawal)
        self.assertIsInstance(api_keys[0], UpbitApiKey)
        self.assertEqual(adapter._handle.deposit_krw_request["amount"], "20000")
        self.assertEqual(
            adapter._handle.withdraw_krw_request["two_factor_type"],
            "kakao",
        )
        self.assertIsInstance(pockets[0], UpbitPocket)
        self.assertEqual(pockets[0].uuid, "pocket-main")
        self.assertEqual(adapter._handle.pocket_api_keys_request["uuids"], ["pocket-main"])
        self.assertEqual(pocket_api_keys[0].keys[0].allowed_ips, ["203.0.113.1"])
        self.assertEqual(adapter._handle.sub_pocket_uuid, "pocket-sub")
        self.assertIsInstance(balances[0], UpbitPocketBalance)
        self.assertEqual(balances[0].balance, Decimal("0.1"))
        self.assertEqual(adapter._handle.universal_transfer_request["from"], "pocket-main")
        self.assertEqual(universal_transfer.identifier, "universal-1")
        self.assertEqual(adapter._handle.universal_transfers_request["states"], ["done"])
        self.assertIsInstance(universal_transfers[0], UpbitPocketTransfer)
        self.assertEqual(adapter._handle.sub_pocket_transfer_request["to"], "pocket-main")
        self.assertEqual(sub_transfer.identifier, "sub-1")
        self.assertEqual(adapter._handle.sub_pocket_transfers_request["states"], ["done"])
        self.assertEqual(sub_transfers[0].state, "done")

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
            notices = await adapter.notices(1)
            fees = await adapter.transfer_fees("BTC")
            api_keys = await adapter.api_keys()
            pending = await adapter.pending_orders(
                BithumbPendingOrdersRequest(
                    market=Market.spot(Exchange.BITHUMB, "BTC", "KRW"),
                    state=BithumbPendingOrderState.WATCH,
                    limit=25,
                    order_by=BithumbOrderDirection.ASCENDING,
                    cursor=Cursor("page+/=="),
                )
            )
            closed = await adapter.closed_orders(
                BithumbClosedOrdersRequest(
                    market=Market.spot(Exchange.BITHUMB, "BTC", "KRW"),
                    state=None,
                    states=[BithumbClosedOrderState.DONE, BithumbClosedOrderState.CANCEL],
                    start_time=1_700_000_000_000_000_000,
                    end_time=1_700_000_001_000_000_000,
                    limit=25,
                    order_by=BithumbOrderDirection.ASCENDING,
                    cursor=Cursor("page+/=="),
                )
            )
            twap = await adapter.twap_orders(
                BithumbTwapOrdersRequest(
                    market=None,
                    uuids=["twap-1"],
                    state=BithumbTwapState.PROGRESS,
                    cursor=Cursor("page+/=="),
                    limit=50,
                    order_by=BithumbTwapOrderDirection.DESCENDING,
                )
            )
            created = await adapter.create_twap_order(
                BithumbTwapOrderRequest(
                    market=Market.spot(Exchange.BITHUMB, "BTC", "KRW"),
                    side=Side.BUY,
                    volume=Decimal("0.1"),
                    price=None,
                    duration=300,
                    frequency=15,
                )
            )
            cancelled = await adapter.cancel_twap_order("twap-1")
            krw_withdrawals = await adapter.krw_withdrawals(
                BithumbKrwWithdrawalsRequest(uuids=["krw-transfer-1"])
            )
            krw_withdrawal = await adapter.withdraw_krw(
                BithumbKrwTransferRequest(Decimal("10000"))
            )
            krw_deposits = await adapter.krw_deposits(BithumbKrwDepositsRequest())
            krw_deposit = await adapter.deposit_krw(
                BithumbKrwTransferRequest(Decimal("20000"))
            )
            withdrawal_addresses = await adapter.withdrawal_addresses()
            order_detail = await adapter.order_detail(
                BithumbOrderDetailRequest(
                    Market.spot(Exchange.BITHUMB, "BTC", "KRW"),
                    uuid="order-detail-1",
                    client_order_id="client-detail-1",
                )
            )
            order_list = await adapter.order_list(
                BithumbOrderListRequest(
                    market=Market.spot(Exchange.BITHUMB, "BTC", "KRW"),
                    state=None,
                    states=[BithumbOrderListState.WAIT],
                    uuids=[],
                    client_order_ids=["client-list-1"],
                    page=2,
                    limit=25,
                    order_by=BithumbOrderDirection.ASCENDING,
                )
            )

        self.assertEqual(adapter.exchange, Exchange.BITHUMB)
        self.assertTrue(adapter.authenticated)
        self.assertEqual(warnings[0][1], "CAUTION")
        self.assertIsInstance(alerts[0][1], BithumbMarketAlert)
        self.assertEqual(alerts[0][1].step, BithumbAlertStep.DANGER)
        self.assertEqual(alerts[0][1].ends_at, 1_700_000_000_000_000_008)
        self.assertEqual(adapter._handle.notice_count, 1)
        self.assertIsInstance(notices[0], BithumbNotice)
        self.assertEqual(notices[0].categories, ["입출금"])
        self.assertEqual(notices[0].url, "https://feed.bithumb.com/notice/1654458")
        self.assertEqual(adapter._handle.fee_currency, "BTC")
        self.assertIsInstance(fees[0], BithumbAssetFee)
        self.assertIsInstance(fees[0].networks[0], BithumbNetworkFee)
        self.assertEqual(fees[0].networks[0].withdrawal_fee.value, Decimal("0.0002"))
        self.assertIsInstance(api_keys[0], BithumbApiKey)
        self.assertEqual(api_keys[0].access_key, "example-access-key-1")
        self.assertEqual(api_keys[0].expires_at, 1_812_672_000_000_000_000)
        self.assertEqual(adapter._handle.pending_request["state"], "watch")
        self.assertEqual(adapter._handle.pending_request["order_by"], "asc")
        self.assertEqual(adapter._handle.pending_request["cursor"], "page+/==")
        self.assertEqual(str(pending.next), "page+/==")
        self.assertEqual(adapter._handle.closed_request["states"], ["done", "cancel"])
        self.assertEqual(adapter._handle.closed_request["cursor"], "page+/==")
        self.assertIsInstance(closed.items[0], BithumbClosedOrder)
        self.assertEqual(closed.items[0].price, Decimal("50000000"))
        self.assertEqual(closed.items[0].created_at, 1_700_000_000_000_000_022)
        self.assertEqual(closed.items[0].trades_count, 0)
        self.assertEqual(closed.items[0].client_order_id, "client-closed-1")
        self.assertEqual(closed.items[0].cancel_type, "user")
        self.assertEqual(closed.items[0].canceling_order_id, "canceling-order-1")
        self.assertEqual(str(closed.next), "closed+/==")
        self.assertIsInstance(twap.items[0], BithumbTwapOrder)
        self.assertEqual(twap.items[0].state, BithumbTwapState.PROGRESS)
        self.assertEqual(adapter._handle.twap_request["order_by"], "desc")
        self.assertEqual(adapter._handle.twap_request["uuids"], ["twap-1"])
        self.assertEqual(adapter._handle.create_twap_request["duration"], 300)
        self.assertEqual(created, "twap-created")
        self.assertEqual(adapter._handle.cancel_twap_id, "twap-1")
        self.assertEqual(cancelled, "twap-cancelled")
        self.assertEqual(
            adapter._handle.krw_withdrawals_request["uuids"], ["krw-transfer-1"]
        )
        self.assertEqual(krw_withdrawals[0].currency, "KRW")
        self.assertEqual(krw_withdrawal.amount, Decimal("10000"))
        self.assertEqual(adapter._handle.withdraw_krw_request["amount"], "10000")
        self.assertEqual(krw_deposits[0].transfer_type, "deposit")
        self.assertEqual(krw_deposit.amount, Decimal("10000"))
        self.assertEqual(adapter._handle.deposit_krw_request["amount"], "20000")
        self.assertIsInstance(withdrawal_addresses[0], BithumbWithdrawalAddress)
        self.assertEqual(withdrawal_addresses[0].withdraw_address, "bc1example")
        self.assertEqual(adapter._handle.order_detail_request["uuid"], "order-detail-1")
        self.assertEqual(
            adapter._handle.order_detail_request["client_order_id"],
            "client-detail-1",
        )
        self.assertIsInstance(order_detail, BithumbOrderDetail)
        self.assertEqual(order_detail.trades[0].funds, Decimal("5000000"))
        self.assertEqual(adapter._handle.order_list_request["states"], ["wait"])
        self.assertEqual(adapter._handle.order_list_request["page"], 2)
        self.assertIsInstance(order_list[0], BithumbOrderListItem)
        self.assertEqual(order_list[0].time_in_force, "post_only")

    async def test_bithumb_pending_order_limit_uses_the_public_error_contract(self) -> None:
        adapter = BithumbAdapter(access_key="key", secret_key="secret")

        for limit in (-1, 4_294_967_296):
            with self.assertRaises(InvalidRequestError) as error:
                await adapter.pending_orders(BithumbPendingOrdersRequest(limit=limit))

            self.assertEqual(error.exception.field, "limit")

    async def test_pocket_models_keep_required_destinations_and_strict_wire_fields(self) -> None:
        with self.assertRaisesRegex(TypeError, "missing 1 required positional argument: 'to'"):
            UpbitPocketTransferRequest(  # type: ignore[call-arg]
                currency="BTC",
                amount=Decimal("0.01"),
            )

        with self.assertRaisesRegex(ValueError, "does not accept unexpected"):
            UpbitPocketTransferQuery.from_wire(
                {
                    "from": None,
                    "to": None,
                    "direction": None,
                    "states": [],
                    "uuids": [],
                    "identifiers": [],
                    "unexpected": True,
                }
            )

    async def test_c2c_page_keeps_a_nullable_provider_envelope(self) -> None:
        page = BinanceC2cTradeHistoryPage.from_wire(
            {
                "code": None,
                "message": None,
                "data": None,
                "total": None,
                "success": None,
            }
        )

        self.assertIsNone(page.code)
        self.assertIsNone(page.data)
        self.assertIsNone(page.success)

    async def test_native_provider_preflight_returns_public_errors_before_network(self) -> None:
        try:
            import_module("maxt._native")
        except ImportError:
            self.skipTest("maxt._native is not built")

        singapore = UpbitAdapter(region=UpbitRegion.SINGAPORE)
        with self.assertRaises(InvalidRequestError) as region_error:
            await singapore.list_pockets()
        self.assertEqual(region_error.exception.field, "region")

        korea = UpbitAdapter()
        with self.assertRaises(AuthError):
            await korea.list_pockets()

        market = Market.spot(Exchange.BITHUMB, "BTC", "KRW")
        with self.assertRaises(AuthError):
            await BithumbAdapter().order_detail(BithumbOrderDetailRequest(market))

        with self.assertRaises(AuthError):
            await BithumbAdapter().order_list(
                BithumbOrderListRequest(None, None, [], [], [])
            )

        with self.assertRaises(AuthError):
            await UpbitAdapter().order_detail(
                UpbitOrderDetailRequest(
                    Market.spot(Exchange.UPBIT, "BTC", "KRW"), uuid="upbit-order-1"
                )
            )

        closed_request = UpbitClosedOrdersRequest(None, None, [])
        self.assertFalse(
            UpbitAdapter(access_key=" ", secret_key="secret").authenticated
        )
        self.assertFalse(
            UpbitAdapter(access_key="access", secret_key=" \t").authenticated
        )
        self.assertTrue(
            UpbitAdapter(access_key="access", secret_key="secret").authenticated
        )
        with self.assertRaises(AuthError):
            await UpbitAdapter().closed_orders(closed_request)

        with self.assertRaises(AuthError):
            await UpbitAdapter(access_key=" ", secret_key="secret").closed_orders(
                closed_request
            )

        authenticated_upbit = UpbitAdapter(access_key="key", secret_key="secret")
        with self.assertRaises(InvalidRequestError) as state_conflict:
            await authenticated_upbit.closed_orders(
                UpbitClosedOrdersRequest(
                    None,
                    UpbitClosedOrderState.DONE,
                    [UpbitClosedOrderState.CANCEL],
                )
            )
        self.assertEqual(state_conflict.exception.field, "states")

        with self.assertRaises(InvalidRequestError) as window_error:
            await authenticated_upbit.closed_orders(
                UpbitClosedOrdersRequest(
                    None,
                    None,
                    [],
                    start_time=0,
                    end_time=604_800_001_000_000,
                )
            )
        self.assertEqual(window_error.exception.field, "end_time")

        with self.assertRaises(AuthError):
            await HyperliquidAdapter().user_fills(False)

        with self.assertRaises(AuthError):
            await HyperliquidAdapter().basic_open_orders()

        with self.assertRaises(AuthError):
            await HyperliquidAdapter().historical_orders()

        with self.assertRaises(AuthError):
            await HyperliquidAdapter().order_status(
                HyperliquidOrderReference.client_order_id("not-a-cloid")
            )

        with self.assertRaises(InvalidRequestError) as order_reference_error:
            await HyperliquidAdapter(
                address="0x14791697260e4c9a71f18484c9f997b308e59325"
            ).order_status(HyperliquidOrderReference.client_order_id("not-a-cloid"))
        self.assertEqual(order_reference_error.exception.field, "client_order_id")

        with self.assertRaises(UnsupportedError) as venue_error:
            await BinanceAdapter.usd_m_futures().c2c_trade_history(
                BinanceC2cTradeHistoryRequest(BinanceC2cTradeType.BUY)
            )
        self.assertEqual(venue_error.exception.exchange, Exchange.BINANCE)

        with self.assertRaises(UnsupportedError) as venue_error:
            await BinanceAdapter.usd_m_futures().spot_average_price(
                Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
            )
        self.assertEqual(venue_error.exception.exchange, Exchange.BINANCE)
        self.assertEqual(venue_error.exception.feature, Feature.TICKER)

        with self.assertRaises(InvalidRequestError) as market_error:
            await BinanceAdapter.spot().spot_average_price(
                Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
            )
        self.assertEqual(market_error.exception.field, "market")

    async def test_upbit_batch_cancel_scope_does_not_drop_restrictions(self) -> None:
        with self.assertRaisesRegex(ValueError, "does not accept values"):
            UpbitBatchCancelScope("all", values=["KRW"]).to_wire()
        with self.assertRaisesRegex(ValueError, "does not accept pairs"):
            UpbitBatchCancelScope.from_wire({"kind": "all", "pairs": ["KRW-BTC"]})
        with self.assertRaisesRegex(ValueError, "does not accept pairs"):
            UpbitBatchCancelRequest.from_wire(
                {"scope": {"kind": "all"}, "pairs": ["KRW-BTC"]}
            )

        request = UpbitBatchCancelRequest(scope={"kind": "all", "values": ["KRW"]})
        adapter = UpbitAdapter(access_key="key", secret_key="secret")

        with self.assertRaises(InvalidRequestError) as raised:
            await adapter.batch_cancel_open_orders(request)

        self.assertEqual(raised.exception.field, "upbit_batch_cancel_scope")

        class RawRequest:
            def to_wire(self):
                return {"scope": {"kind": "all"}, "pairs": ["KRW-BTC"]}

        with self.assertRaises(InvalidRequestError) as raised:
            await adapter.batch_cancel_open_orders(RawRequest())

        self.assertEqual(raised.exception.field, "upbit_batch_cancel_request")

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
            average_price = await spot.spot_average_price(market)
            futures_market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
            mark_price = await futures.mark_price(futures_market)
            mark_prices = await futures.mark_prices()
            open_interest = await futures.open_interest(futures_market)
            account_trades = await spot.account_trades(HistoryRequest(market))
            c2c_history = await spot.c2c_trade_history(
                BinanceC2cTradeHistoryRequest(
                    BinanceC2cTradeType.BUY,
                    page=2,
                    rows=20,
                )
            )
            test_order = await spot.test_order(
                BinanceTestOrderRequest(
                    OrderRequest.limit_order(
                        market,
                        Side.BUY,
                        Size.base(Decimal("0.1")),
                        Decimal("50000"),
                    ),
                    False,
                )
            )
            await spot.cancel_all_open_orders(market)
            key = await futures.usd_m_create_listen_key()
            await futures.usd_m_keepalive_listen_key()
            await futures.usd_m_close_listen_key()

        self.assertEqual(spot.venue, BinanceMarket.SPOT)
        self.assertEqual(futures.venue, BinanceMarket.USD_M_FUTURES)
        self.assertIsInstance(filters, BinanceSymbolFilters)
        self.assertEqual(filters.tick_size, Decimal("0.0100"))
        self.assertIsInstance(order, BinanceSpotOrderDetail)
        self.assertEqual(order.filled_quote_quantity, Decimal("12500"))
        self.assertIsInstance(average_price, BinanceSpotAveragePrice)
        self.assertEqual(average_price.market, market)
        self.assertEqual(average_price.minutes, 7)
        self.assertEqual(average_price.price, Decimal("9.357518340000000000"))
        self.assertEqual(average_price.close_time, 1_694_061_154_503_000_000)
        self.assertIsInstance(mark_price, BinanceMarkPrice)
        self.assertEqual(mark_price.mark_price, Decimal("50001.25"))
        self.assertIsInstance(mark_prices[0], BinanceMarkPrice)
        self.assertIsInstance(open_interest, BinanceOpenInterest)
        self.assertEqual(open_interest.open_interest, Decimal("1234.5"))
        self.assertIsInstance(account_trades.items[0], BinanceAccountTrade)
        self.assertEqual(account_trades.items[0].commission_asset, "BNB")
        self.assertIsInstance(c2c_history, BinanceC2cTradeHistoryPage)
        self.assertEqual(c2c_history.data[0].asset, "USDT")
        self.assertEqual(c2c_history.data[0].additional_kyc_verify, 2)
        self.assertEqual(spot._handle.c2c_trade_history_request["trade_type"], "BUY")
        self.assertEqual(spot._handle.c2c_trade_history_request["page"], 2)
        self.assertIsInstance(test_order, BinanceTestOrder)
        self.assertEqual(test_order.response_json, "{}")
        self.assertEqual(
            spot._handle.cancel_all_open_orders_market,
            market,
        )
        self.assertIsInstance(key, BinanceListenKey)
        self.assertEqual(key.value, "listen-key")
        self.assertTrue(futures._handle.kept_alive)
        self.assertTrue(futures._handle.closed)

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
            mids = await adapter.all_mids()
            open_orders = await adapter.basic_open_orders()
            order_status = await adapter.order_status(
                HyperliquidOrderReference.order_id(42)
            )
            historical_orders = await adapter.historical_orders()
            fills = await adapter.user_fills(True)
            fills_by_time = await adapter.user_fills_by_time(
                1_700_000_000_000_000_000,
                1_700_000_100_000_000_000,
                True,
            )
            rate_limit = await adapter.user_rate_limit()
            role = await adapter.user_role()
            referral = await adapter.referral()
            fees = await adapter.user_fees()
            portfolio = await adapter.portfolio()
            sub_accounts = await adapter.sub_accounts()
            vault_equities = await adapter.user_vault_equities()

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
        self.assertIsInstance(mids[0], HyperliquidMidPrice)
        self.assertEqual(mids[0].price, Decimal("50000.5"))
        self.assertIsInstance(open_orders[0], HyperliquidOpenOrder)
        self.assertEqual(open_orders[0].limit_price, Decimal("3500.2500"))
        self.assertIsInstance(order_status, HyperliquidOrderStatusResponse)
        self.assertEqual(order_status.kind, "order")
        self.assertEqual(adapter._handle.order_status_reference, {"kind": "order_id", "value": 42})
        self.assertIsInstance(historical_orders[0], HyperliquidOrderInfo)
        self.assertTrue(historical_orders[0].order.reduce_only)
        self.assertEqual(adapter._handle.user_fills_args, (True,))
        self.assertIsInstance(fills[0], HyperliquidUserFill)
        self.assertEqual(fills[0].price, Decimal("3500.25"))
        self.assertEqual(
            adapter._handle.user_fills_by_time_args,
            (1_700_000_000_000_000_000, 1_700_000_100_000_000_000, True),
        )
        self.assertIsInstance(fills_by_time[0], HyperliquidUserFill)
        self.assertIsInstance(rate_limit, HyperliquidUserRateLimit)
        self.assertEqual(rate_limit.requests_cap, 1000)
        self.assertIsInstance(role, HyperliquidUserRole)
        self.assertEqual(role.user, "0xuser")
        self.assertIsInstance(referral, HyperliquidReferral)
        self.assertEqual(referral.referred_by.code, "REF")
        self.assertIsInstance(fees, HyperliquidUserFees)
        self.assertIsInstance(fees.daily_volumes[0], HyperliquidDailyVolume)
        self.assertIsInstance(portfolio[0], HyperliquidPortfolioPeriod)
        self.assertIsInstance(sub_accounts[0], HyperliquidSubAccount)
        self.assertIsInstance(vault_equities[0], HyperliquidVaultEquity)


if __name__ == "__main__":
    unittest.main()
