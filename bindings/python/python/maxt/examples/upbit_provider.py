"""Read Upbit Korea provider-specific quotation data."""

from __future__ import annotations

import asyncio

from maxt import Exchange, Market, UpbitAdapter


async def main() -> None:
    adapter = UpbitAdapter()
    market = Market.spot(Exchange.UPBIT, "BTC", "KRW")
    tickers = await adapter.tickers([market])
    instruments = await adapter.orderbook_instruments([market])
    print(f"region={adapter.region}; ticker rows={len(tickers)}")
    print(f"order-book instrument rows={len(instruments)}")


if __name__ == "__main__":
    asyncio.run(main())
