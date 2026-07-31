# maxt

`maxt` provides one async Python API for public market data, account state,
orders, margin, history, and streams across Upbit, Bithumb, Binance, and
Hyperliquid. Public numeric values use `decimal.Decimal`; timestamps are Unix
epoch nanosecond integers. Decimal strings preserve exact values across the
Rust boundary.

After the first PyPI release:

```bash
pip install maxt
```

From a repository checkout:

```bash
mise install
mise exec -- uv sync --project bindings/python --frozen
```

```python
import asyncio

from maxt import Client, Exchange, Market, UpbitAdapter


async def main() -> None:
    client = Client(UpbitAdapter())
    market = Market.spot(Exchange.UPBIT, "BTC", "KRW")
    book = await client.order_book(market, 5)
    print(book)


asyncio.run(main())
```

## API surface

`Client` provides markets, trades, order books, tickers, candles, public and
account streams, balances, orders, positions, margin, and funding history.

| Adapter | Provider-specific API |
| --- | --- |
| `UpbitAdapter` | `region`, `order_books`, `tickers`, `market_events` |
| `BithumbAdapter` | `market_warnings`, `market_alerts` |
| `BinanceAdapter` | `venue`, `spot_symbol_filters`, `spot_order`, `usd_m_create_listen_key`, `usd_m_keepalive_listen_key`, `usd_m_close_listen_key` |
| `HyperliquidAdapter` | `is_testnet`, `non_funding_ledger`, `asset_context` |

Calls raise structured `InvalidRequestError`, `UnsupportedError`,
`AdapterError`, `AuthError`, `ExchangeError`, `TransportError`, or
`DecodeError` values. `ExchangeError` preserves the provider code, status, and
retry classification.

## Streams

Stream items are `StreamEvent` or non-terminal `StreamError` values. Use
`async with stream` or `await stream.aclose()` to await native cleanup; garbage
collection is best-effort only.

Project source and provider notes are available at
[github.com/jabdori/maxt](https://github.com/jabdori/maxt).
