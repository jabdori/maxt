"""Read an Upbit account when configured and build safe request objects."""

from __future__ import annotations

import asyncio
import os
from decimal import Decimal

from maxt import (
    Client,
    Exchange,
    Market,
    Network,
    OrderRequest,
    Side,
    Size,
    TransferHistoryRequest,
    UpbitAdapter,
)


async def main() -> None:
    market = Market.spot(Exchange.UPBIT, "BTC", "KRW")
    draft = OrderRequest.limit_order(
        market,
        Side.BUY,
        Size.base(Decimal("0.0001")),
        Decimal("100000"),
        client_id="docs-example-only",
    )
    history = TransferHistoryRequest(asset="BTC", network=Network.BITCOIN, limit=20)
    print(f"order draft only; it was not sent: {draft}")
    print(f"transfer-history request: {history}")

    access_key = os.getenv("UPBIT_ACCESS_KEY")
    secret_key = os.getenv("UPBIT_SECRET_KEY")
    if not access_key or not secret_key:
        print("set UPBIT_ACCESS_KEY and UPBIT_SECRET_KEY to run the read-only account section")
        return

    client = Client(UpbitAdapter(access_key=access_key, secret_key=secret_key))
    balances = await client.balances()
    orders = await client.open_orders()
    print(f"{len(balances)} balances and {len(orders)} open orders")


if __name__ == "__main__":
    asyncio.run(main())
