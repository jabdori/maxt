import assert from "node:assert/strict";
import test from "node:test";

import {
  Balance,
  BinanceMarket,
  ChainDestination,
  CandleRequest,
  Decimal,
  DepositAddressRequest,
  DepositStatus,
  Exchange,
  Feed,
  Feature,
  HyperliquidLedgerKind,
  Interval,
  Level,
  Market,
  MarketKind,
  Network,
  OrderBook,
  OrderRequest,
  OrderStatus,
  OrderType,
  Overflow,
  Page,
  Side,
  Size,
  StreamConfig,
  Subscription,
  Timestamp,
  TimeInForce,
  TransferDestination,
  TransferHistoryRequest,
  TravelRuleRequirement,
  WithdrawRequest,
  WithdrawalFee,
  WithdrawalStatus,
} from "../dist/models.js";
import { InvalidRequestError } from "../dist/errors.js";
import {
  assetNetworkFromWire,
  assetNetworkToWire,
  depositFromWire,
  depositToWire,
  orderRequestFromWire,
  orderRequestToWire,
  streamConfigFromWire,
  transferHistoryRequestFromWire,
  transferHistoryRequestToWire,
  withdrawRequestFromWire,
  withdrawRequestToWire,
  withdrawalFromWire,
  withdrawalToWire,
} from "../dist/generated/codec.js";

test("Decimal preserves its exact text and rejects unrepresentable inputs", () => {
  const value = Decimal.parse("1.2300");
  assert.equal(value.coefficient, 12300n);
  assert.equal(value.scale, 4);
  assert.equal(value.toString(), "1.2300");
  assert.equal(Decimal.parse("+12.30e+2").toString(), "+12.30e+2");
  const scientificZero = Decimal.parse("0e+30");
  assert.equal(scientificZero.coefficient, 0n);
  assert.equal(scientificZero.scale, 0);
  assert.equal(scientificZero.toString(), "0e+30");
  assert.equal(Decimal.parse(".0e+64").toString(), ".0e+64");
  assert.throws(() => Decimal.parse(".0e+65"), RangeError);
  assert.throws(() => Decimal.parse("0e100"), RangeError);
  assert.throws(() => Decimal.parse("0e9223372036854775807"), RangeError);
  assert.throws(() => Decimal.parse("1e+30"), RangeError);
  assert.throws(() => Decimal.parse("2.5e-28"), RangeError);
  assert.throws(() => Decimal.parse("0.00000000000000000000000000001"), RangeError);
  assert.throws(() => Decimal.parse("79228162514264337593543950336"), RangeError);
  assert.throws(() => Decimal.parse("9".repeat(10_000)), RangeError);
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
  assert.equal(
    new Decimal(17040610785213832950n, 1).divideByInteger(-184146665451776816n).toString(),
    "-9.253825337215417658162271385",
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
  assert.equal(HyperliquidLedgerKind.other("deposit"), HyperliquidLedgerKind.Deposit);
  assert.equal(Network.other("bitcoin"), Network.Bitcoin);
  assert.equal(Network.other("future_chain").id, "future_chain");
  assert.equal(Feature.Balances.needsCredentials, true);
  assert.equal(Feature.AssetNetworks.needsCredentials, true);
  assert.equal(Feature.FundingRates.needsCredentials, false);
  assert.equal(Feature.FundingRates.isDerivativesOnly, true);
  assert.equal(MarketKind.Perpetual.isDerivative, true);
  assert.equal(Interval.Hour4.seconds, 14_400);
  assert.equal(Interval.Month1.seconds, null);
  assert.equal(Side.Buy.flipped, Side.Sell);
  assert.equal(OrderStatus.PartiallyFilled.isLive, true);
  assert.equal(OrderStatus.Filled.isLive, false);
});

test("best orders preserve time in force and client id across the wire", () => {
  const request = OrderRequest.best(
    Market.spot(Exchange.Bithumb, "BTC", "KRW"),
    Side.Buy,
    Size.quote(Decimal.parse("10000")),
    TimeInForce.ImmediateOrCancel,
    { clientId: "client-1" },
  );
  const wire = orderRequestToWire(request);

  assert.equal(request.orderType, OrderType.Best);
  assert.equal(wire.client_id, "client-1");
  assert.deepEqual(orderRequestToWire(orderRequestFromWire(wire)), wire);
});

test("wallet unions, statuses, open networks, and pages preserve the wire contract", () => {
  const assetNetworkWire = {
    exchange: "binance",
    asset: "btc",
    network: "future_chain",
    provider_id: "FUTURE",
    deposit_enabled: true,
    withdrawal_enabled: false,
    withdrawal_fee: { kind: "rate", rate: "0.001", minimum: "0.0001", maximum: null },
    minimum_withdrawal: "0.01",
    maximum_withdrawal: null,
    memo_required: true,
  };
  const network = assetNetworkFromWire(assetNetworkWire);
  assert.equal(network.asset, "BTC");
  assert.equal(network.network.id, "future_chain");
  assert.equal(network.withdrawalFee.kind, "rate");
  assert.deepEqual(assetNetworkToWire(network), { ...assetNetworkWire, asset: "BTC" });

  const destination = TransferDestination.chain(new ChainDestination(
    "btc", Network.Bitcoin, "bc1qdestination",
  ));
  const request = new WithdrawRequest(
    "btc", Network.Bitcoin, Decimal.parse("1.00"), destination, "client-1",
  );
  assert.deepEqual(withdrawRequestToWire(withdrawRequestFromWire(withdrawRequestToWire(request))), {
    asset: "BTC",
    network: "bitcoin",
    amount: "1.00",
    destination: {
      kind: "chain",
      value: { asset: "BTC", network: "bitcoin", address: "bc1qdestination", memo: null },
    },
    client_id: "client-1",
  });

  const history = new TransferHistoryRequest("btc", Network.Bitcoin, null, 100);
  assert.deepEqual(
    transferHistoryRequestToWire(transferHistoryRequestFromWire(transferHistoryRequestToWire(history))),
    { asset: "BTC", network: "bitcoin", cursor: null, limit: 100 },
  );

  const withdrawalWire = {
    id: "withdrawal-1",
    asset: "BTC",
    network: "bitcoin",
    provider_network: "BTC",
    amount: "1.00",
    fee: "0.0001",
    destination: withdrawRequestToWire(request).destination,
    status: WithdrawalStatus.Processing.id,
    provider_status: "processing",
    tx_id: null,
    created_at: null,
  };
  assert.deepEqual(withdrawalToWire(withdrawalFromWire(withdrawalWire)), withdrawalWire);

  const depositWire = {
    id: "deposit-1",
    asset: "BTC",
    network: "bitcoin",
    provider_network: "BTC",
    amount: "0.99",
    address: null,
    memo: null,
    status: DepositStatus.Completed.id,
    provider_status: "credited",
    tx_id: "tx-1",
    created_at: null,
  };
  assert.deepEqual(depositToWire(depositFromWire(depositWire)), depositWire);
  assert.equal(WithdrawalFee.fixed(Decimal.one).kind, "fixed");
  assert.equal(TravelRuleRequirement.NotRequired.kind, "not_required");
  assert.equal(new DepositAddressRequest("btc", Network.Bitcoin).asset, "BTC");
});

test("wire unsigned integers reject malformed and unsafe values", () => {
  const wire = {
    max_reconnect_attempts: null,
    initial_reconnect_delay_ms: "1000",
    max_reconnect_delay_ms: "30000",
    idle_timeout_ms: "30000",
    buffer_size: "4096",
    overflow: "backpressure",
  };
  assert.equal(streamConfigFromWire(wire).bufferSize, 4096);
  assert.throws(
    () => streamConfigFromWire({ ...wire, buffer_size: "1.5" }),
    InvalidRequestError,
  );
  assert.throws(
    () => streamConfigFromWire({ ...wire, buffer_size: "9007199254740992" }),
    InvalidRequestError,
  );
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

  const boundary = new StreamConfig({
    initialReconnectDelayMs: 4294967296,
    maxReconnectDelayMs: Number.MAX_SAFE_INTEGER,
    idleTimeoutMs: Number.MAX_SAFE_INTEGER,
    bufferSize: Number.MAX_SAFE_INTEGER,
  });
  assert.equal(boundary.initialReconnectDelayMs, 4294967296);
  assert.equal(boundary.maxReconnectDelayMs, Number.MAX_SAFE_INTEGER);
  assert.equal(boundary.idleTimeoutMs, Number.MAX_SAFE_INTEGER);
  assert.equal(boundary.bufferSize, Number.MAX_SAFE_INTEGER);
});

test("feeds and subscriptions keep immutable snapshots", () => {
  const market = Market.spot(Exchange.Upbit, "BTC", "KRW");
  const duplicateMarket = Market.spot(Exchange.Upbit, "btc", "krw");
  const candleFeed = Feed.candles(Interval.Min1);
  const duplicateCandleFeed = Feed.candles(Interval.Min1);
  const markets = [market, duplicateMarket, market];
  const feeds = [Feed.Trades, candleFeed, Feed.Trades, duplicateCandleFeed];
  const subscription = new Subscription(markets, feeds);
  markets.length = 0;
  feeds.length = 0;
  assert.deepEqual(subscription.markets, [market]);
  assert.deepEqual(subscription.feeds, [Feed.Trades, candleFeed]);
  assert.equal(subscription.markets[0], market);
  assert.equal(subscription.feeds[1], candleFeed);
  assert.equal(Object.isFrozen(subscription.markets), true);
  assert.equal(subscription.withFeed(Feed.Ticker).feeds.length, 3);
  assert.equal(subscription.withMarket(duplicateMarket).markets.length, 1);
  assert.equal(subscription.withFeed(duplicateCandleFeed).feeds.length, 2);
});

test("subscription market deduplication compares structural identity without display collisions", () => {
  const slashInBase = Market.spot(Exchange.Upbit, "A/B", "C");
  const slashInQuote = Market.spot(Exchange.Upbit, "A", "B/C");
  assert.equal(slashInBase.toString(), slashInQuote.toString());

  const subscription = new Subscription([slashInBase, slashInQuote, slashInBase]);

  assert.deepEqual(subscription.markets, [slashInBase, slashInQuote]);
});
