import assert from "node:assert/strict";
import test from "node:test";

import {
  Balance,
  BinanceMarket,
  BithumbApiKey,
  BithumbAssetFee,
  BithumbNetworkFee,
  BithumbOrderDirection,
  BithumbPendingOrderState,
  BithumbPendingOrdersRequest,
  CancelOrdersRequest,
  CancelOrdersResult,
  CancelledOrder,
  Cursor,
  ChainDestination,
  CandleRequest,
  Decimal,
  DepositAddressEntry,
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
  MarketStatus,
  Network,
  OrderBook,
  OrderAccount,
  OrderCancelFailure,
  OrderIdKind,
  OrderLookupRequest,
  OrderOption,
  OrderRequest,
  OrderRules,
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
  TransferLookupRequest,
  TravelRuleRequirement,
  UpbitOrderBookInstrument,
  UpbitDepositInfo,
  UpbitYearCandle,
  WithdrawRequest,
  WithdrawalFee,
  WithdrawalStatus,
} from "../dist/models.js";
import { InvalidRequestError } from "../dist/errors.js";
import {
  assetNetworkFromWire,
  assetNetworkToWire,
  bithumbPendingOrdersRequestFromWire,
  bithumbPendingOrdersRequestToWire,
  cancelOrdersRequestFromWire,
  cancelOrdersRequestToWire,
  cancelOrdersResultFromWire,
  cancelOrdersResultToWire,
  depositAddressEntryFromWire,
  depositAddressEntryToWire,
  depositFromWire,
  depositToWire,
  orderRequestFromWire,
  orderRequestToWire,
  orderLookupRequestFromWire,
  orderLookupRequestToWire,
  orderRulesFromWire,
  orderRulesToWire,
  streamConfigFromWire,
  transferHistoryRequestFromWire,
  transferHistoryRequestToWire,
  transferLookupRequestFromWire,
  transferLookupRequestToWire,
  upbitDepositInfoFromWire,
  upbitDepositInfoToWire,
  upbitOrderBookInstrumentFromWire,
  upbitOrderBookInstrumentToWire,
  upbitYearCandleFromWire,
  upbitYearCandleToWire,
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

test("Upbit yearly candles and orderbook policy preserve provider-only fields", () => {
  const market = Market.spot(Exchange.Upbit, "BTC", "KRW");
  const annual = new UpbitYearCandle(
    market,
    Timestamp.fromNanoseconds(1767225600000000000n),
    Timestamp.fromNanoseconds(1767225600000000000n),
    Timestamp.fromNanoseconds(1786467753786000000n),
    Decimal.parse("128000000.00000000"),
    Decimal.parse("143050000.00000000"),
    Decimal.parse("88770000.00000000"),
    Decimal.parse("89587000.00000000"),
    Decimal.parse("348666.78732189"),
    Decimal.parse("37189906239683.17623000"),
    "2026-01-01",
  );
  const instrument = new UpbitOrderBookInstrument(
    market,
    "KRW",
    Decimal.parse("1000"),
    [Decimal.zero, Decimal.parse("10000")],
  );

  assert.deepEqual(upbitYearCandleToWire(upbitYearCandleFromWire(upbitYearCandleToWire(annual))), upbitYearCandleToWire(annual));
  assert.deepEqual(
    upbitOrderBookInstrumentToWire(
      upbitOrderBookInstrumentFromWire(upbitOrderBookInstrumentToWire(instrument)),
    ),
    upbitOrderBookInstrumentToWire(instrument),
  );
  assert.equal(Object.isFrozen(instrument.supportedLevels), true);
});

test("Upbit deposit information preserves nullable network metadata and policy", () => {
  const deposit = new UpbitDepositInfo(
    "btc",
    Network.Bitcoin,
    "BTC",
    true,
    null,
    Decimal.parse("0.0005"),
    18_446_744_073_709_551_615n,
    18_446_744_073_709_551_615n,
  );

  assert.equal(deposit.asset, "BTC");
  assert.equal(deposit.network, Network.Bitcoin);
  assert.equal(deposit.minimumDepositAmount.toString(), "0.0005");
  assert.equal(deposit.minimumDepositConfirmations, 18_446_744_073_709_551_615n);
  assert.equal(deposit.decimalPrecision, 18_446_744_073_709_551_615n);
  assert.equal(Object.isFrozen(deposit), true);
  assert.deepEqual(
    upbitDepositInfoToWire(upbitDepositInfoFromWire(upbitDepositInfoToWire(deposit))),
    upbitDepositInfoToWire(deposit),
  );
});

test("Bithumb transfer fees preserve fixed and rate rules per network", () => {
  const fixed = new BithumbNetworkFee(
    Network.Bitcoin, "Bitcoin", Decimal.zero, Decimal.zero,
    WithdrawalFee.fixed(Decimal.parse("0.0002")), Decimal.parse("0.001"),
  );
  const rate = new BithumbNetworkFee(
    Network.Arbitrum, "Arbitrum One", Decimal.parse("0.01"), Decimal.parse("2"),
    WithdrawalFee.rate(Decimal.parse("0.01"), Decimal.one, Decimal.parse("100")),
    Decimal.parse("10"),
  );
  const fee = new BithumbAssetFee("비트코인", "btc", [fixed, rate]);

  assert.equal(fee.asset, "BTC");
  assert.equal(fee.networks[0].withdrawalFee.kind, "fixed");
  assert.equal(fee.networks[1].withdrawalFee.kind, "rate");
  assert.throws(() => fee.networks.push(fixed), TypeError);
});

test("Bithumb API keys preserve their identifier and expiry", () => {
  const key = new BithumbApiKey("example-access-key-1", Timestamp.fromSeconds(1812672000n));

  assert.equal(key.accessKey, "example-access-key-1");
  assert.equal(key.expiresAt.nanosecondsSinceEpoch, 1812672000000000000n);
  assert.equal(Object.isFrozen(key), true);
});

test("Bithumb pending-order requests preserve filters and opaque cursors", () => {
  const request = new BithumbPendingOrdersRequest(
    Market.spot(Exchange.Bithumb, "BTC", "KRW"),
    BithumbPendingOrderState.Watch,
    25,
    BithumbOrderDirection.Ascending,
    new Cursor("page+/=="),
  );
  const wire = bithumbPendingOrdersRequestToWire(request);

  assert.deepEqual(wire, {
    market: { exchange: "bithumb", kind: "spot", base: "BTC", quote: "KRW" },
    state: "watch",
    limit: 25,
    order_by: "asc",
    cursor: "page+/==",
  });
  const decoded = bithumbPendingOrdersRequestFromWire(wire);
  assert.equal(decoded.state, BithumbPendingOrderState.Watch);
  assert.equal(decoded.orderBy, BithumbOrderDirection.Ascending);
  assert.equal(decoded.cursor.value, "page+/==");
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

test("bulk order lookup preserves one identifier namespace", () => {
  const request = new OrderLookupRequest(
    OrderIdKind.Client,
    ["client-1", "client-2"],
    Market.spot(Exchange.Bithumb, "BTC", "KRW"),
  );
  const wire = orderLookupRequestToWire(request);

  assert.equal(wire.kind, "client");
  assert.deepEqual(wire.ids, ["client-1", "client-2"]);
  assert.deepEqual(orderLookupRequestToWire(orderLookupRequestFromWire(wire)), wire);
});

test("batch cancellation preserves per-order outcomes and immutable inputs", () => {
  const ids = ["client-1", "missing-1"];
  const request = new CancelOrdersRequest(OrderIdKind.Client, ids);
  const result = new CancelOrdersResult(
    [new CancelledOrder("order-1", "client-1", null, Timestamp.fromNanoseconds(1n))],
    [new OrderCancelFailure(null, "missing-1", null, "order_not_found", "not found")],
  );
  ids.length = 0;

  const requestWire = cancelOrdersRequestToWire(request);
  const resultWire = cancelOrdersResultToWire(result);
  assert.deepEqual(request.ids, ["client-1", "missing-1"]);
  assert.equal(Object.isFrozen(request.ids), true);
  assert.equal(Object.isFrozen(result.cancelled), true);
  assert.deepEqual(cancelOrdersRequestToWire(cancelOrdersRequestFromWire(requestWire)), requestWire);
  assert.deepEqual(cancelOrdersResultToWire(cancelOrdersResultFromWire(resultWire)), resultWire);
});

test("order rules preserve typed and future provider options", () => {
  const market = Market.spot(Exchange.Upbit, "BTC", "KRW");
  const rules = new OrderRules(
    market,
    "BTC/KRW",
    MarketStatus.Active,
    Decimal.parse("0.001"),
    Decimal.parse("0.001"),
    Decimal.parse("0.0005"),
    Decimal.parse("0.0005"),
    [Side.Buy, Side.Sell],
    [new OrderOption("limit_ioc", OrderType.Limit, TimeInForce.ImmediateOrCancel)],
    [new OrderOption("future_order", null, null)],
    null,
    null,
    Decimal.parse("5000"),
    Decimal.parse("5000"),
    Decimal.parse("1000000000"),
    new OrderAccount(new Balance("KRW", Decimal.parse("10000"), Decimal.zero), Decimal.zero, false, "KRW"),
    new OrderAccount(new Balance("BTC", Decimal.one, Decimal.zero), Decimal.parse("95000000"), false, "KRW"),
  );
  const wire = orderRulesToWire(rules);
  const restored = orderRulesFromWire(wire);

  assert.equal(Object.isFrozen(rules.sides), true);
  assert.equal(Object.isFrozen(rules.buyOptions), true);
  assert.equal(restored.buyOptions[0].timeInForce, TimeInForce.ImmediateOrCancel);
  assert.equal(restored.sellOptions[0].providerId, "future_order");
  assert.equal(restored.sellOptions[0].orderType, null);
  assert.equal(restored.buyPriceUnit, null);
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
  const transferLookupWire = { asset: "BTC", id: "deposit-1", tx_id: null };
  assert.deepEqual(
    transferLookupRequestToWire(transferLookupRequestFromWire(transferLookupWire)),
    transferLookupWire,
  );
  assert.equal(new TransferLookupRequest("btc", null, "tx-1").asset, "BTC");
  const depositAddressEntryWire = {
    exchange: "binance",
    asset: "XRP",
    network: null,
    provider_network: null,
    address: null,
    memo: "tag-7",
  };
  assert.deepEqual(
    depositAddressEntryToWire(depositAddressEntryFromWire(depositAddressEntryWire)),
    depositAddressEntryWire,
  );
  assert.equal(
    new DepositAddressEntry(Exchange.Binance, "xrp", null, null, null, "tag-7").asset,
    "XRP",
  );
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
