import assert from "node:assert/strict";
import { createRequire } from "node:module";
import test from "node:test";

import * as maxt from "../dist/node.js";
import {
  Adapter,
  AuthError,
  BinanceAdapter,
  BinanceMarket,
  BithumbAdapter,
  CancelOrdersRequest,
  CancelOrdersResult,
  CancelledOrder,
  Client,
  Decimal,
  Exchange,
  Feature,
  InvalidRequestError,
  Market,
  MarketStatus,
  Order,
  OrderAccount,
  OrderCancelFailure,
  OrderHistoryRequest,
  OrderIdKind,
  OrderLookupRequest,
  OrderOption,
  OrderRequest,
  OrderRules,
  OrderStatus,
  OrderType,
  Page,
  Side,
  Size,
  Ticker,
  Timestamp,
  TimeInForce,
} from "../dist/node.js";
import {
  RAW_NATIVE_CLIENT_MEMBERS,
  RAW_NATIVE_EXPORTS,
  RAW_PROVIDER_MEMBERS,
  PROVIDER_CONSTRUCTORS,
} from "../dist/generated/contract.js";

const raw = createRequire(import.meta.url)("../native.cjs");

test("generated inventories match the loaded native module exactly", () => {
  assert.deepEqual(Object.keys(raw).sort(), [...RAW_NATIVE_EXPORTS].sort());
  assert.deepEqual(
    Object.getOwnPropertyNames(raw.NativeClient.prototype)
      .filter((name) => name !== "constructor")
      .sort(),
    [...RAW_NATIVE_CLIENT_MEMBERS].sort(),
  );
  for (const [exchange, constructor] of Object.entries({
    upbit: raw.NativeUpbit,
    bithumb: raw.NativeBithumb,
    binance: raw.NativeBinance,
    hyperliquid: raw.NativeHyperliquid,
  })) {
    assert.deepEqual(
      Object.getOwnPropertyNames(constructor.prototype)
        .filter((name) => name !== "constructor")
        .sort(),
      [...RAW_PROVIDER_MEMBERS[exchange]].sort(),
    );
  }
});

test("every generated provider adapter is exported publicly", () => {
  assert.deepEqual(
    Object.keys(maxt).filter((name) => name === "Adapter" || name.endsWith("Adapter")).sort(),
    ["Adapter", "BinanceAdapter", "BithumbAdapter", "HyperliquidAdapter", "UpbitAdapter"],
  );

  for (const [exchange, constructor] of Object.entries({
    upbit: maxt.UpbitAdapter,
    bithumb: maxt.BithumbAdapter,
    binance: maxt.BinanceAdapter,
    hyperliquid: maxt.HyperliquidAdapter,
  })) {
    assert.deepEqual(
      Object.getOwnPropertyNames(constructor)
        .filter((name) => !["length", "name", "prototype"].includes(name))
        .sort(),
      PROVIDER_CONSTRUCTORS[exchange]
        .filter((name) => name !== "constructor")
        .sort(),
    );
  }
});

test("built-in adapters load through the generated Node backend", () => {
  const spot = BinanceAdapter.spot();
  assert.equal(spot.exchange, Exchange.Binance);
  assert.equal(spot.venue, BinanceMarket.Spot);

  const futures = BinanceAdapter.usdMFutures();
  assert.equal(futures.venue, BinanceMarket.UsdMFutures);

  assert.throws(
    () => new BithumbAdapter({ accessKey: "incomplete" }),
    InvalidRequestError,
  );
});

test("Upbit orderbook aggregation carries Decimal through the Node boundary", async () => {
  await maxt.initialize();
  const upbit = new maxt.UpbitAdapter();
  const market = Market.spot(Exchange.Upbit, "BTC", "KRW");

  await assert.rejects(
    upbit.orderBooksAtLevel([market], Decimal.parse("-1")),
    (error) => error instanceof InvalidRequestError && error.field === "level",
  );
});

test("Upbit test orders reject missing credentials before a network request", async () => {
  await maxt.initialize();
  const market = Market.spot(Exchange.Upbit, "BTC", "KRW");

  await assert.rejects(
    new maxt.UpbitAdapter().testOrder(OrderRequest.limit(
      market,
      Side.Buy,
      Size.base(Decimal.parse("0.01")),
      Decimal.parse("100000000"),
    )),
    (error) => error instanceof AuthError,
  );
});

test("Upbit deposit information rejects missing credentials before a network request", async () => {
  await maxt.initialize();

  await assert.rejects(
    new maxt.UpbitAdapter().depositInfo("BTC", maxt.Network.Bitcoin),
    (error) => error instanceof AuthError,
  );
});

test("Travel Rule and KRW transfer history reject unsupported or unauthenticated calls", async () => {
  await maxt.initialize();

  await assert.rejects(
    new maxt.UpbitAdapter({ region: maxt.UpbitRegion.Indonesia }).travelRuleVasps(),
    (error) => error instanceof maxt.UnsupportedError,
  );
  await assert.rejects(
    new maxt.UpbitAdapter().travelRuleVasps(),
    (error) => error instanceof AuthError,
  );
  await assert.rejects(
    new maxt.BithumbAdapter().krwWithdrawals(new maxt.BithumbKrwWithdrawalsRequest()),
    (error) => error instanceof AuthError,
  );
});

test("Upbit conditional batch cancellation rejects missing credentials before a network request", async () => {
  await maxt.initialize();

  await assert.rejects(
    new maxt.UpbitAdapter().batchCancelOpenOrders(
      new maxt.UpbitBatchCancelRequest(maxt.UpbitBatchCancelScope.all()),
    ),
    (error) => error instanceof AuthError,
  );
});

test("Bithumb notices reject invalid counts before the Node boundary", async () => {
  await maxt.initialize();
  const bithumb = new maxt.BithumbAdapter();

  for (const count of [0, Number.NaN, Number.POSITIVE_INFINITY]) {
    await assert.rejects(
      bithumb.notices(count),
      (error) => error instanceof InvalidRequestError && error.field === "count",
    );
  }
});

test("Bithumb transfer fees reject an empty currency before the Node boundary", async () => {
  await maxt.initialize();
  const bithumb = new maxt.BithumbAdapter();

  await assert.rejects(
    bithumb.transferFees(" "),
    (error) => error instanceof InvalidRequestError && error.field === "currency",
  );
});

test("Bithumb API keys reject missing credentials before a network request", async () => {
  await maxt.initialize();

  await assert.rejects(
    new maxt.BithumbAdapter().apiKeys(),
    (error) => error instanceof AuthError,
  );
});

test("Bithumb pending orders validate the provider limit before a network request", async () => {
  await maxt.initialize();
  const bithumb = new maxt.BithumbAdapter({ accessKey: "key", secretKey: "secret" });

  await assert.rejects(
    bithumb.pendingOrders(new maxt.BithumbPendingOrdersRequest(null, null, 101)),
    (error) => error instanceof InvalidRequestError && error.field === "limit",
  );
});

test("custom Adapter calls round-trip through Rust without losing values", async () => {
  const market = Market.spot(Exchange.Binance, "BTC", "USDT");
  const expected = new Ticker(
    market,
    Timestamp.fromNanoseconds(123n),
    null,
    Decimal.parse("100.25"),
    null,
    null,
    null,
    null,
    Decimal.parse("2.5"),
    null,
  );
  const expectedOrder = new Order(
    "order-1",
    market,
    Side.Buy,
    OrderStatus.Filled,
    Decimal.parse("1"),
    Decimal.zero,
    Decimal.parse("100.25"),
    Timestamp.fromNanoseconds(124n),
  );
  const expectedRules = new OrderRules(
    market,
    "BTC/USDT",
    MarketStatus.Active,
    Decimal.parse("0.001"),
    Decimal.parse("0.001"),
    Decimal.parse("0.0005"),
    Decimal.parse("0.0005"),
    [Side.Buy, Side.Sell],
    [new OrderOption("limit_ioc", OrderType.Limit, TimeInForce.ImmediateOrCancel)],
    [new OrderOption("future_order", null, null)],
    Decimal.parse("0.1"),
    Decimal.parse("0.1"),
    Decimal.parse("10"),
    Decimal.parse("10"),
    Decimal.parse("1000000"),
    new OrderAccount(new maxt.Balance("USDT", Decimal.parse("100"), Decimal.zero), Decimal.zero, false, "USDT"),
    new OrderAccount(new maxt.Balance("BTC", Decimal.one, Decimal.zero), Decimal.parse("50000"), false, "USDT"),
  );

  class FixtureAdapter extends Adapter {
    exchange = Exchange.Binance;
    features = new Set([Feature.Ticker, Feature.OrderHistory, Feature.Trading]);
    async ticker(requested) {
      assert.equal(requested.toString(), market.toString());
      return expected;
    }
    async orderRules(requested) {
      assert.equal(requested.toString(), market.toString());
      return expectedRules;
    }
    async order(requested, orderId) {
      assert.equal(requested.toString(), market.toString());
      assert.equal(orderId, "order-1");
      return expectedOrder;
    }
    async orderByClientId(requested, clientId) {
      assert.equal(requested.toString(), market.toString());
      assert.equal(clientId, "client-1");
      return expectedOrder;
    }
    async ordersByIds(request) {
      assert.equal(request.kind, OrderIdKind.Exchange);
      assert.deepEqual(request.ids, ["order-1", "order-2"]);
      assert.equal(request.market?.toString(), market.toString());
      return [expectedOrder];
    }
    async orderHistory(request) {
      assert.equal(request.market?.toString(), market.toString());
      assert.deepEqual(request.statuses, [OrderStatus.Filled]);
      return new Page([expectedOrder], null);
    }
    async cancelOrders(request) {
      assert.equal(request.kind, OrderIdKind.Client);
      assert.deepEqual(request.ids, ["client-1", "missing-1"]);
      return new CancelOrdersResult(
        [new CancelledOrder("order-1", "client-1", market, Timestamp.fromNanoseconds(125n))],
        [new OrderCancelFailure(null, "missing-1", null, "order_not_found", "not found")],
      );
    }
  }

  const client = new Client(new FixtureAdapter());
  const actual = await client.ticker(market);
  assert.equal(actual.lastPrice.toString(), "100.25");
  assert.equal(actual.timestamp.nanosecondsSinceEpoch, 123n);
  assert.equal(actual.volume?.toString(), "2.5");
  const rules = await client.orderRules(market);
  assert.equal(rules.baseAccount.balance.asset, "BTC");
  assert.equal(rules.buyOptions[0].timeInForce, TimeInForce.ImmediateOrCancel);
  assert.equal(rules.sellOptions[0].orderType, null);
  assert.equal((await client.order(market, "order-1")).id, "order-1");
  assert.equal((await client.orderByClientId(market, "client-1")).id, "order-1");
  assert.equal(
    (await client.ordersByIds(
      new OrderLookupRequest(OrderIdKind.Exchange, ["order-1", "order-2"], market),
    ))[0].id,
    "order-1",
  );
  const history = await client.orderHistory(
    new OrderHistoryRequest(market, [OrderStatus.Filled]),
  );
  assert.equal(history.items[0].id, "order-1");
  const cancelled = await client.cancelOrders(
    new CancelOrdersRequest(OrderIdKind.Client, ["client-1", "missing-1"]),
  );
  assert.equal(cancelled.cancelled[0].cancelledAt.nanosecondsSinceEpoch, 125n);
  assert.equal(cancelled.failed[0].code, "order_not_found");
});
