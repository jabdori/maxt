"""Read Hyperliquid public data and optional address-scoped Info data."""

from __future__ import annotations

import asyncio
import os

from maxt import Exchange, HyperliquidAdapter, Market


async def main() -> None:
    market = Market.perpetual(Exchange.HYPERLIQUID, "BTC", "USDC")
    adapter = HyperliquidAdapter()
    mids = await adapter.all_mids()
    book = await adapter.l2_book(market)
    trades = await adapter.recent_trades(market)
    print(f"{len(mids)} mid prices; {len(book.bids) + len(book.asks)} levels; {len(trades)} trades")

    address = os.getenv("HYPERLIQUID_ADDRESS")
    if address:
        orders = await HyperliquidAdapter(address=address).basic_open_orders()
        print(f"{len(orders)} address-scoped open orders")
    else:
        print("set HYPERLIQUID_ADDRESS for address-scoped Info reads; a private key is not needed")


if __name__ == "__main__":
    asyncio.run(main())
