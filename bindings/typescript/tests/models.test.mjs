import assert from "node:assert/strict";
import test from "node:test";

import {
  Balance,
  BinanceMarket,
  CandleRequest,
  Decimal,
  Exchange,
  Feed,
  HyperliquidLedgerKind,
  Interval,
  Level,
  Market,
  OrderBook,
  Overflow,
  Page,
  Side,
  StreamConfig,
  Subscription,
  Timestamp,
} from "../dist/models.js";

test("Decimal preserves its exact text and rejects unrepresentable inputs", () => {
  const value = Decimal.parse("1.2300");
  assert.equal(value.coefficient, 12300n);
  assert.equal(value.scale, 4);
  assert.equal(value.toString(), "1.2300");
  assert.equal(Decimal.parse("+12.30e+2").toString(), "+12.30e+2");
  assert.throws(() => Decimal.parse("2.5e-28"), RangeError);
  assert.throws(() => Decimal.parse("0.00000000000000000000000000001"), RangeError);
  assert.throws(() => Decimal.parse("79228162514264337593543950336"), RangeError);
  assert.throws(() => new Decimal(1n, 29), RangeError);
  assert.throws(() => Number(value), TypeError);
});

test("Decimal compares numerically and arithmetic uses half-even rounding", () => {
  assert.equal(Decimal.parse("1.0").equals(Decimal.parse("1.00")), true);
  assert.equal(Decimal.parse("-2").compareTo(Decimal.parse("-1.999")), -1);
  assert.equal(Decimal.parse("1.20").add(Decimal.parse("2.30")).toString(), "3.5");
  assert.equal(Decimal.parse("1.20").subtract(Decimal.parse("2.30")).toString(), "-1.1");
  assert.equal(Decimal.parse("1").divideByInteger(2n).toString(), "0.5");
  assert.equal(
    Decimal.parse("0.0000000000000000000000000005").divideByInteger(2n).toString(),
    "0.0000000000000000000000000002",
  );
  assert.equal(
    Decimal.parse("0.0000000000000000000000000015").divideByInteger(2n).toString(),
    "0.0000000000000000000000000008",
  );
});

test("Timestamp preserves signed i64 nanoseconds and saturates scaled constructors", () => {
  const minimum = Timestamp.fromNanoseconds(-9223372036854775808n);
  const maximum = Timestamp.fromNanoseconds(9223372036854775807n);
  assert.equal(minimum.nanosecondsSinceEpoch, -9223372036854775808n);
  assert.equal(maximum.nanosecondsSinceEpoch, 9223372036854775807n);
  assert.throws(() => Timestamp.fromNanoseconds(9223372036854775808n), RangeError);
  assert.equal(Timestamp.fromSeconds(9223372036854775807n).equals(maximum), true);
  assert.equal(Timestamp.fromMilliseconds(-9223372036854775808n).equals(minimum), true);
  assert.equal(Timestamp.fromNanoseconds(-1999999999n).millisecondsSinceEpoch, -1999n);
  assert.equal(Timestamp.fromNanoseconds(-1999999999n).secondsSinceEpoch, -1n);
  assert.equal(Timestamp.fromNanoseconds(1500000n).toDate().getTime(), 1);
});

test("string variants are stable singleton values in Rust declaration order", () => {
  assert.deepEqual(Exchange.values, [
    Exchange.Upbit,
    Exchange.Bithumb,
    Exchange.Binance,
    Exchange.Hyperliquid,
  ]);
  assert.equal(Exchange.Binance.id, "binance");
  assert.deepEqual(BinanceMarket.values, [BinanceMarket.Spot, BinanceMarket.UsdMFutures]);
  assert.equal(HyperliquidLedgerKind.other("futureKind").id, "futureKind");
});

test("public records normalize ASCII assets, preserve nulls, and freeze collections", () => {
  const market = Market.spot(Exchange.Binance, "éth", "usdt");
  assert.equal(market.base, "éTH");
  assert.equal(market.quote, "USDT");
  assert.equal(market.toString(), "binance:éTH/USDT");

  const bids = [new Level(Decimal.parse("100.10"), Decimal.one)];
  const asks = [new Level(Decimal.parse("100.30"), Decimal.one)];
  const book = new OrderBook(market, Timestamp.zero, bids, asks);
  bids.push(new Level(Decimal.zero, Decimal.zero));
  assert.equal(book.bids.length, 1);
  assert.equal(book.spread?.toString(), "0.2");
  assert.equal(book.midPrice?.toString(), "100.2");
  assert.equal(Object.isFrozen(book), true);
  assert.equal(Object.isFrozen(book.bids), true);

  const balance = new Balance("ıbtc", Decimal.parse("1.25"), Decimal.parse("0.75"));
  assert.equal(balance.asset, "ıBTC");
  assert.equal(balance.total.toString(), "2");
  const page = new Page([balance], null);
  assert.equal(page.hasMore, false);
  assert.equal(Object.hasOwn(page, "next"), true);
});

test("request models reject values outside their Rust unsigned integer boundaries", () => {
  const market = Market.spot(Exchange.Upbit, "BTC", "KRW");
  assert.throws(
    () => new CandleRequest(market, Interval.Min1, null, null, 4294967296),
    RangeError,
  );
  assert.throws(() => new StreamConfig({ maxReconnectAttempts: -1 }), RangeError);
  assert.throws(() => new StreamConfig({ idleTimeoutMs: Number.MAX_SAFE_INTEGER + 1 }), RangeError);
  assert.throws(() => new StreamConfig({ bufferSize: 1.5 }), RangeError);

  const config = new StreamConfig({ overflow: Overflow.DropNewest });
  assert.equal(config.maxReconnectAttempts, null);
  assert.equal(config.bufferSize, 4096);
  assert.equal(config.overflow, Overflow.DropNewest);
});

test("feeds and subscriptions keep immutable snapshots", () => {
  const market = Market.spot(Exchange.Upbit, "BTC", "KRW");
  const markets = [market];
  const feeds = [Feed.Trades, Feed.candles(Interval.Min1)];
  const subscription = new Subscription(markets, feeds);
  markets.length = 0;
  feeds.length = 0;
  assert.deepEqual(subscription.markets, [market]);
  assert.equal(subscription.feeds[1]?.interval, Interval.Min1);
  assert.equal(Object.isFrozen(subscription.markets), true);
  assert.equal(subscription.withFeed(Feed.Ticker).feeds.length, 3);
});
