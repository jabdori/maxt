"""Read public Binance USD-M perpetual funding and market data."""

from __future__ import annotations

import asyncio

from maxt import BinanceAdapter, Client, Exchange, HistoryRequest, Market


async def main() -> None:
    market = Market.perpetual(Exchange.BINANCE, "BTC", "USDT")
    adapter = BinanceAdapter.usd_m_futures()
    mark = await adapter.mark_price(market)
    interest = await adapter.open_interest(market)
    print(f"mark={mark.mark_price}; open interest={interest.open_interest}")

    funding = await Client(adapter).funding_rates(HistoryRequest(market, limit=5))
    print(f"{len(funding.items)} funding rows; next={funding.next}")


if __name__ == "__main__":
    asyncio.run(main())
