const assert = require("node:assert/strict");
const test = require("node:test");

const { createCustomClient } = require("../../native.cjs");

const market = { exchange: "binance", kind: "spot", base: "BTC", quote: "USDT" };
const config = {
  max_reconnect_attempts: null,
  initial_reconnect_delay_ms: "100",
  max_reconnect_delay_ms: "1000",
  idle_timeout_ms: "30000",
  buffer_size: "32",
  overflow: "backpressure",
};
const order = {
  id: "order-1",
  market,
  side: "buy",
  status: "open",
  filled_quantity: "0",
  remaining_quantity: "1",
  price: null,
  created_at: null,
};
const depositAddressRequest = {
  asset: "BTC",
  network: "bitcoin",
  amount: null,
};
const transferDestination = {
  kind: "chain",
  value: {
    asset: "BTC",
    network: "bitcoin",
    address: "bc1qdestination",
    memo: null,
  },
};
const withdrawRequest = {
  asset: "BTC",
  network: "bitcoin",
  amount: "1.00",
  destination: transferDestination,
  client_id: "client-1",
};
const transferHistory = {
  asset: null,
  network: null,
  cursor: null,
  limit: null,
};
const ok = (value) => JSON.stringify({ ok: true, value });
const nativeValue = async (promise) => {
  const result = await promise;
  assert.equal(result.ok, true, JSON.stringify(result));
  return result.value;
};
const eventually = async (predicate) => {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (predicate()) return;
    await new Promise((resolve) => setImmediate(resolve));
  }
  assert.fail("condition was not met");
};

test("custom client dispatches every Adapter operation and owns stream cleanup", async () => {
  const calls = [];
  const streamCalls = [];
  const closed = [];
  const dispatch = async (text) => {
    const call = JSON.parse(text);
    calls.push(call);
    const empty = [
      "markets", "trades", "candles", "balances", "asset_networks", "open_orders", "positions",
    ];
    if (empty.includes(call.kind)) return ok({ kind: call.kind, value: [] });
    switch (call.kind) {
      case "order_book":
        return ok({
          kind: "order_book",
          value: { market, timestamp: "1", bids: [], asks: [] },
        });
      case "ticker":
        return ok({
          kind: "ticker",
          value: {
            market,
            timestamp: "1",
            last_trade_time: null,
            last_price: "1",
            change: null,
            change_rate: null,
            high: null,
            low: null,
            volume: null,
            quote_volume: null,
          },
        });
      case "subscribe":
        return ok({ kind: "market_stream", stream_id: call.stream_id });
      case "subscribe_account":
        return ok({ kind: "account_stream", stream_id: call.stream_id });
      case "deposit_address":
        return ok({
          kind: "deposit_address",
          value: {
            exchange: "binance",
            asset: call.request.asset,
            network: call.request.network,
            address: "bc1qdestination",
            memo: null,
          },
        });
      case "prepare_withdrawal":
        return ok({
          kind: "withdrawal_quote",
          value: {
            fee: "0.0001",
            expected_receive: "0.9999",
            minimum_amount: null,
            maximum_amount: null,
            address_allowed: true,
            travel_rule: { kind: "not_required" },
            expires_at: null,
          },
        });
      case "withdraw":
        return ok({
          kind: "withdrawal",
          value: {
            id: "withdrawal-1",
            asset: call.request.asset,
            network: call.request.network,
            provider_network: "BTC",
            amount: call.request.amount,
            fee: "0.0001",
            destination: call.request.destination,
            status: "pending",
            provider_status: "accepted",
            tx_id: null,
            created_at: null,
          },
        });
      case "deposits":
      case "withdrawals":
        return ok({ kind: call.kind, value: { items: [], next: null } });
      case "place_order":
        return ok({ kind: "place_order", value: order });
      case "cancel_order":
      case "cancel_order_by_client_id":
        return ok({ kind: "unit" });
      case "margin_summary":
        return ok({
          kind: "margin_summary",
          value: {
            asset: "USDT",
            equity: null,
            margin_balance: null,
            available_balance: null,
          },
        });
      case "funding_rates":
      case "funding_payments":
        return ok({ kind: call.kind, value: { items: [], next: null } });
      case "set_margin":
        return ok({ kind: "unit" });
      default:
        throw new Error(`unhandled call ${call.kind}`);
    }
  };
  const streamNext = async (id) => {
    streamCalls.push(id);
    if (streamCalls.filter((value) => value === id).length > 1) return ok(null);
    return ok({ kind: "event", event: { kind: "reconnected" } });
  };
  const streamClose = async (id) => {
    closed.push(id);
    return ok(null);
  };
  const client = createCustomClient(
    "binance",
    ["markets", "ticker", "trade_stream", "account_stream"],
    dispatch,
    streamNext,
    streamClose,
  );

  assert.equal(client.exchange(), "binance");
  assert.equal(client.supports("ticker"), true);
  assert.equal(client.supports("trades"), false);
  await nativeValue(client.markets(JSON.stringify("spot")));
  await nativeValue(client.trades(JSON.stringify(market), "null"));
  await nativeValue(client.orderBook(JSON.stringify(market), "null"));
  await nativeValue(client.ticker(JSON.stringify(market)));
  await nativeValue(client.candles(JSON.stringify({
    market,
    interval: "min1",
    from: null,
    to: null,
    limit: null,
  })));
  await nativeValue(client.balances());
  await nativeValue(client.assetNetworks(JSON.stringify("BTC")));
  await nativeValue(client.depositAddress(JSON.stringify(depositAddressRequest)));
  await nativeValue(client.prepareWithdrawal(JSON.stringify(withdrawRequest)));
  await nativeValue(client.withdraw(JSON.stringify(withdrawRequest)));
  await nativeValue(client.deposits(JSON.stringify(transferHistory)));
  await nativeValue(client.withdrawals(JSON.stringify(transferHistory)));
  await nativeValue(client.openOrders());
  await nativeValue(client.openOrdersOn(JSON.stringify(market)));
  await nativeValue(client.placeOrder(JSON.stringify({
    market,
    side: "buy",
    order_type: "market",
    size: { kind: "base", value: "1" },
    price: null,
    time_in_force: null,
    reduce_only: false,
  })));
  await nativeValue(client.cancelOrder(JSON.stringify(market), JSON.stringify("order-1")));
  await nativeValue(client.cancelOrderByClientId(JSON.stringify(market), JSON.stringify("client-1")));
  await nativeValue(client.positions());
  await nativeValue(client.positionsOn(JSON.stringify(market)));
  await nativeValue(client.marginSummary());
  const history = JSON.stringify({ market, from: null, to: null, cursor: null, limit: null });
  await nativeValue(client.fundingRates(history));
  await nativeValue(client.fundingPayments(history));
  await nativeValue(client.setMargin(JSON.stringify({
    market,
    leverage: null,
    margin_mode: null,
  })));

  const subscription = JSON.stringify({ markets: [market], feeds: [{ kind: "trades" }] });
  const marketHandle = await nativeValue(client.subscribeWith(subscription, JSON.stringify(config)));
  assert.deepEqual(await nativeValue(client.streamNext(JSON.stringify(marketHandle.id))), {
    kind: "event",
    event: { kind: "reconnected" },
  });
  await nativeValue(client.streamClose(JSON.stringify(marketHandle.id)));

  const accountHandle = await nativeValue(client.subscribeAccountWith(JSON.stringify(config)));
  assert.deepEqual(await nativeValue(client.streamNext(JSON.stringify(accountHandle.id))), {
    kind: "event",
    event: { kind: "reconnected" },
  });
  await nativeValue(client.streamClose(JSON.stringify(accountHandle.id)));

  assert.deepEqual(
    [...new Set(calls.map(({ kind }) => kind))].sort(),
    [
      "asset_networks", "balances", "cancel_order", "cancel_order_by_client_id", "candles", "deposit_address", "deposits",
      "funding_payments", "funding_rates", "margin_summary", "markets", "open_orders",
      "order_book", "place_order", "positions", "prepare_withdrawal", "set_margin", "subscribe",
      "subscribe_account", "ticker", "trades", "withdraw", "withdrawals",
    ].sort(),
  );
  assert.deepEqual(streamCalls, ["1", "2"]);
  assert.deepEqual(closed, ["1", "2"]);
});

test("custom client keeps structured errors and rejects forged stream IDs", async () => {
  const transport = createCustomClient(
    "binance",
    ["ticker"],
    async () => JSON.stringify({
      ok: false,
      error: { kind: "transport", detail: "offline" },
    }),
    async () => ok(null),
    async () => ok(null),
  );
  assert.deepEqual(await transport.ticker(JSON.stringify(market)), {
    ok: false,
    error: { kind: "transport", detail: "offline" },
  });

  const forgedClosed = [];
  const forged = createCustomClient(
    "binance",
    ["trade_stream"],
    async () => ok({ kind: "market_stream", stream_id: "forged" }),
    async () => ok(null),
    async (id) => {
      forgedClosed.push(id);
      return ok(null);
    },
  );
  const result = await forged.subscribe(JSON.stringify({
    markets: [market],
    feeds: [{ kind: "trades" }],
  }));
  assert.equal(result.ok, false);
  assert.equal(result.error.kind, "adapter");
  assert.match(result.error.detail, /allocated stream/);
  await eventually(() => forgedClosed.length === 1);
  assert.deepEqual(forgedClosed, ["1"]);
});

test("custom client owns callback references after factory arguments are released", async () => {
  const client = createCustomClient(
    "binance",
    ["markets"],
    async () => ok({ kind: "markets", value: [] }),
    async () => ok(null),
    async () => ok(null),
  );
  global.gc?.();
  assert.deepEqual(await nativeValue(client.markets(JSON.stringify("spot"))), []);
});

test("closing a custom stream wakes a pending pull and closes once", async () => {
  let closeCount = 0;
  const client = createCustomClient(
    "binance",
    ["trade_stream"],
    async (text) => {
      const call = JSON.parse(text);
      return ok({ kind: "market_stream", stream_id: call.stream_id });
    },
    async () => new Promise(() => {}),
    async () => {
      closeCount += 1;
      return ok(null);
    },
  );
  const handle = await nativeValue(client.subscribe(JSON.stringify({
    markets: [market],
    feeds: [{ kind: "trades" }],
  })));
  const pending = client.streamNext(JSON.stringify(handle.id));
  await new Promise((resolve) => setImmediate(resolve));

  await nativeValue(client.streamClose(JSON.stringify(handle.id)));
  assert.equal(await nativeValue(pending), null);
  await nativeValue(client.streamClose(JSON.stringify(handle.id)));
  assert.equal(closeCount, 1);
});

test("natural custom stream exhaustion awaits close once", async () => {
  let closeCount = 0;
  const client = createCustomClient(
    "binance",
    ["trade_stream"],
    async (text) => {
      const call = JSON.parse(text);
      return ok({ kind: "market_stream", stream_id: call.stream_id });
    },
    async () => ok(null),
    async () => {
      closeCount += 1;
      return ok(null);
    },
  );
  const handle = await nativeValue(client.subscribe(JSON.stringify({
    markets: [market],
    feeds: [{ kind: "trades" }],
  })));

  assert.equal(await nativeValue(client.streamNext(JSON.stringify(handle.id))), null);
  await nativeValue(client.streamClose(JSON.stringify(handle.id)));
  assert.equal(closeCount, 1);
});
