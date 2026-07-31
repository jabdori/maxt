# maxt

`maxt` provides one async Python API for public market data, account state,
orders, margin, history, and streams across Upbit, Bithumb, Binance, and
Hyperliquid. Exact decimal values cross the native boundary as strings and
timestamps use Unix epoch nanoseconds.

```bash
pip install maxt
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

Project source and provider notes are available at
[github.com/jabdori/maxt](https://github.com/jabdori/maxt).
