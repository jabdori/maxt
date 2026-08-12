# maxt for Python

[English](README.md) | [한국어](README.ko.md)

One async Python API for the same operations, models, errors, and streams as
the Rust contract. Common operations and exchange-specific operations remain
available together. Generated contracts are checked against the compiled
native API.

## Install

GIL-enabled CPython 3.9 or newer is required. PyPy and free-threaded CPython
are not currently supported. Prebuilt wheels cover glibc 2.17 or newer Linux
(x64 and ARM64),
macOS (x64 and ARM64), and Windows (x64). Other platforms build from the source
distribution and require Rust and a native compiler toolchain.

```sh
python -m pip install maxt
```

Python has no separate initialization function. Constructing a built-in
adapter loads the native module.

## Supported exchanges

- Upbit Spot: Korea, Singapore, Indonesia, and Thailand
- Bithumb Spot
- Binance Spot and USD-M perpetual futures
- Hyperliquid Spot and perpetual futures on mainnet and testnet

Binance testnet constructors are not exposed. Hyperliquid HIP-3 perpetual DEXs
and outcome assets are not exposed.

## Common API

`Client` provides the same method names for every built-in adapter:

- Public REST: `markets()`, `trades()`, `order_book()`, `ticker()`, and
  `candles()`.
- Public streams: `subscribe()` and `subscribe_with()` for trades, order books,
  tickers, and candles. Bithumb does not support candle streams.
- Public funding history: `funding_rates()` on Binance USD-M and Hyperliquid
  perpetual markets.
- Private Spot: `balances()`, `open_orders()`, `place_order()`,
  `cancel_order()`, and `subscribe_account()` on every exchange.
- Private order lookup: `order()`, `order_by_client_id()`, `orders_by_ids()`,
  and `order_history()` on Upbit and Bithumb.
- Private order rules: `order_rules()` on Upbit and Bithumb.
- Private batch cancellation: `cancel_orders()` on Upbit and Bithumb.
- Private wallet lookup and cancellation: `deposit()`, `withdrawal()`, and
  `cancel_withdrawal()` on Upbit and Bithumb. Lookups require an asset and one
  exchange ID or transaction ID; cancellation must be followed by a lookup.
- Private perpetuals: `positions()`, `margin_summary()`, `set_margin()`, and
  `funding_payments()` on Binance USD-M and Hyperliquid.

Public calls need no credentials. Private calls require both credential fields.
Use `client.supports(feature)` before optional operations when the adapter or
credential state is dynamic.

## Exchange-specific API

Exchange-specific methods remain available through `client.adapter`.

| Adapter | Construction | Additional methods |
| --- | --- | --- |
| `UpbitAdapter` | `UpbitAdapter()` or `UpbitAdapter(region=...)` | `order_books()`, `order_books_at_level()`, `tickers()`, `tickers_by_quote()`, `year_candles()`, `orderbook_instruments()`, `market_events()`; authenticated: `test_order()`, `deposit_info()`, `travel_rule_vasps()`, `verify_travel_rule_by_uuid()`, `verify_travel_rule_by_txid()`, `batch_cancel_open_orders()`, `cancel_and_new_order()` |
| `BithumbAdapter` | `BithumbAdapter()` | `market_warnings()`, `market_alerts()`, `notices()`, `transfer_fees()`; authenticated: `api_keys()`, `krw_withdrawals()`, `withdraw_krw()`, `krw_deposits()`, `deposit_krw()`, `pending_orders()`, `batch_orders()`, `twap_orders()`, `create_twap_order()`, `cancel_twap_order()` |
| `BinanceAdapter` | `BinanceAdapter.spot()` | `spot_symbol_filters()`; authenticated: `spot_order()` |
| `BinanceAdapter` | `BinanceAdapter.usd_m_futures()` | Public: `mark_price()`, `mark_prices()`, `open_interest()`, `aggregate_trades()`; authenticated: `usd_m_create_listen_key()`, `usd_m_keepalive_listen_key()`, `usd_m_close_listen_key()` |
| `HyperliquidAdapter` | `HyperliquidAdapter()` or `HyperliquidAdapter.testnet()` | Public: `all_mids()`; `asset_context()`, `non_funding_ledger()` |

`UpbitAdapter.test_order()` validates an order without creating it. The returned
`Order` is a dry-run result: do not query or cancel its `id`, and do not treat
its status as a live order.

`UpbitAdapter.deposit_info(asset, network)` returns the provider's deposit
availability, minimum amount, confirmation, and precision metadata. Upbit may
delay this information by several minutes; it is not a real-time service-status signal.

`UpbitAdapter.travel_rule_vasps()` lists VASPs for Travel Rule verification.
The verification methods are financial writes and are available only in Korea
and Singapore; Indonesia and Thailand fail before a network request. These
paths are fixture-verified only.

`UpbitAdapter.batch_cancel_open_orders(request)` is a financial write.
`UpbitBatchCancelScope.all()` explicitly selects every eligible market; Upbit
still applies the request count (default 20, maximum 300 `wait` orders), and
the result preserves partial failures.

`UpbitAdapter.cancel_and_new_order(request)` is a financial write using the JSON
endpoint. The replacement keeps the original market and side; `post_only` and
SMP cannot be combined. A successful HTTP response may still have no new order
when the previous order fills before cancellation completes. This path is
fixture-verified only.

`BithumbAdapter.batch_orders(request)` accepts 1–20 orders and can return HTTP
200 with per-item failures; inspect every `BithumbBatchOrderOutcome`. Accepted
items preserve `time_in_force` and `stp_type`; rejected items preserve returned
`time_in_force`. This is a fixture-verified financial write only.

`BithumbAdapter.twap_orders(request)` is an authenticated, read-only history
query for Bithumb's KRW markets. `create_twap_order()` and
`cancel_twap_order()` are financial writes; do not call them in a read-only
verification.

`BithumbAdapter.krw_withdrawals()` and `krw_deposits()` read KRW transfer
history. `withdraw_krw()` and `deposit_krw()` are financial writes. Bithumb
requires its registered account and Kakao second-factor flow; maxt neither
accepts nor stores those credentials. These paths are fixture-verified only.

```python
from maxt import (
    BithumbAdapter,
    BithumbTwapOrdersRequest,
    Client,
    Exchange,
    Market,
)

async def read_twap_history() -> None:
    client = Client(BithumbAdapter(access_key=access_key, secret_key=secret_key))
    market = Market.spot(Exchange.BITHUMB, "BTC", "KRW")
    page = await client.adapter.twap_orders(
        BithumbTwapOrdersRequest(market=market, limit=20)
    )
```

The Bithumb TWAP API accepts `progress`, `done`, or `cancel` states and uses a
page size from 1 through 100. Creation uses a 300–43,200 second duration and a
15/20/30/60/120 second interval; buys require `price`, sells require `volume`.

`BinanceAdapter.usd_m_futures()` exposes `mark_price()`, `mark_prices()`, and
`open_interest()` as public, read-only USD-M perpetual market-data calls. These
methods are fixture-verified; they have not been live-read verified.
`aggregate_trades(request)` is also a public USD-M read. It uses an inclusive
`from_id` cursor or inclusive time bounds (not both), with a time window shorter
than one hour and `limit` from 1 through 1,000 (`None` defaults to 500). Binance
only retains the latest 48 hours; this method is fixture-verified only.
`HyperliquidAdapter.all_mids()` is also public and read-only. It returns the
default perpetual DEX mids and first-DEX spot mids; Hyperliquid falls back to
the last trade price when a book is empty. This method is fixture-verified and
has not been live-read verified.

## Binance common and exchange-specific APIs

```python
import asyncio

from maxt import BinanceAdapter, Client, Exchange, Market


async def main() -> None:
    client = Client(BinanceAdapter.spot())
    market = Market.spot(Exchange.BINANCE, "BTC", "USDT")

    ticker = await client.ticker(market)
    filters = await client.adapter.spot_symbol_filters(market)

    print(ticker.last_price)
    print(filters.tick_size)


asyncio.run(main())
```

`ticker()` is common. `spot_symbol_filters()` is Binance Spot-specific and is
available through `client.adapter`.

## Streams

```python
from maxt import Feed, StreamError, StreamEvent, Subscription

subscription = Subscription((market,), (Feed.TRADES,))
async with await client.subscribe(subscription) as stream:
    async for item in stream:
        if isinstance(item, StreamEvent):
            print(item.event)
        elif isinstance(item, StreamError):
            print(item.error)
```

`StreamError` does not terminate iteration. Use `async with` or
`await stream.aclose()` to await native cleanup.

## Custom adapters

Subclass `Adapter`, implement `exchange` and `features`, then override every
advertised operation. Wrap the instance with `Client(adapter)`. Default
methods raise `UnsupportedError`.

For custom streams, return `MarketStream` or `AccountStream` over an async
iterator. Emit `StreamEvent` and `StreamError`; implement the iterator's
`aclose()` when cleanup is required.

## Contracts

- `decimal.Decimal`: exact 96-bit coefficient, scale `0..=28`; no rounding at the native boundary.
- `Timestamp`: Unix epoch nanoseconds as `int`.
- Errors: `InvalidRequestError`, `UnsupportedError`, `AdapterError`, `AuthError`, `ExchangeError`, `TransportError`, `DecodeError`.
- `ExchangeError`: preserves provider code, HTTP status, and retry classification.

See the [common data and pagination contracts](../../docs/common-api.md) and
[provider limits and data semantics](../../docs/providers.md).

## License

MIT
