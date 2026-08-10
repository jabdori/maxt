# maxt for Python

[English](README.md) | [한국어](README.ko.md)

One async Python API for the same operations, models, errors, and streams as
the Rust contract. Common operations and exchange-specific operations remain
available together. Generated contracts are checked against the compiled
native API.

## Install

GIL-enabled CPython 3.9 or newer is required. PyPy and free-threaded CPython
are not supported in 0.1.0. Prebuilt wheels cover glibc 2.17 or newer Linux
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
- Private perpetuals: `positions()`, `margin_summary()`, `set_margin()`, and
  `funding_payments()` on Binance USD-M and Hyperliquid.

Public calls need no credentials. Private calls require both credential fields.
Use `client.supports(feature)` before optional operations when the adapter or
credential state is dynamic.

## Exchange-specific API

Exchange-specific methods remain available through `client.adapter`.

| Adapter | Construction | Additional methods |
| --- | --- | --- |
| `UpbitAdapter` | `UpbitAdapter()` or `UpbitAdapter(region=...)` | `order_books()`, `tickers()`, `market_events()` |
| `BithumbAdapter` | `BithumbAdapter()` | `market_warnings()`, `market_alerts()` |
| `BinanceAdapter` | `BinanceAdapter.spot()` | `spot_symbol_filters()`; authenticated: `spot_order()` |
| `BinanceAdapter` | `BinanceAdapter.usd_m_futures()` | Authenticated: `usd_m_create_listen_key()`, `usd_m_keepalive_listen_key()`, `usd_m_close_listen_key()` |
| `HyperliquidAdapter` | `HyperliquidAdapter()` or `HyperliquidAdapter.testnet()` | `asset_context()`, `non_funding_ledger()` |

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
