import assert from "node:assert/strict";
import test from "node:test";

import {
  Balance,
  BinanceMarket,
  BinanceAggregateTrade,
  BinanceAggregateTradesRequest,
  BinanceAccountTrade,
  BinanceC2cTrade,
  BinanceC2cTradeHistoryPage,
  BinanceC2cTradeHistoryRequest,
  BinanceC2cTradeType,
  BinanceTestOrder,
  BinanceTestOrderRequest,
  BinanceMarkPrice,
  BinanceOpenInterest,
  BithumbApiKey,
  BithumbAssetFee,
  BithumbBatchOrder,
  BithumbBatchOrderFailure,
  BithumbBatchOrderOutcome,
  BithumbBatchOrdersRequest,
  BithumbBatchOrdersResult,
  BithumbClosedOrder,
  BithumbClosedOrderState,
  BithumbClosedOrdersRequest,
  BithumbKrwDeposit,
  BithumbKrwDepositsRequest,
  BithumbKrwTransferRequest,
  BithumbKrwWithdrawal,
  BithumbKrwWithdrawalsRequest,
  BithumbNetworkFee,
  BithumbWithdrawalAddress,
  BithumbOrderDetail,
  BithumbOrderDetailRequest,
  BithumbOrderDetailTrade,
  BithumbOrderDirection,
  BithumbOrderListItem,
  BithumbOrderListRequest,
  BithumbOrderListState,
  BithumbPendingOrderState,
  BithumbPendingOrdersRequest,
  BithumbTwapOrderRequest,
  BithumbTwapOrdersRequest,
  BithumbTwapOrderDirection,
  BithumbTwapState,
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
  HyperliquidDailyVolume,
  HyperliquidMidPrice,
  HyperliquidOpenOrder,
  HyperliquidOrderDetail,
  HyperliquidOrderInfo,
  HyperliquidPortfolioPeriod,
  HyperliquidPortfolioPoint,
  HyperliquidReferral,
  HyperliquidReferrer,
  HyperliquidSubAccount,
  HyperliquidUserFees,
  HyperliquidUserFill,
  HyperliquidUserRateLimit,
  HyperliquidVaultEquity,
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
  Order,
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
  UpbitClosedOrder,
  UpbitClosedOrdersRequest,
  UpbitClosedOrderState,
  UpbitOrderDetail,
  UpbitOrderDetailRequest,
  UpbitOrderDetailTrade,
  UpbitApiKey,
  UpbitPocket,
  UpbitPocketApiKey,
  UpbitPocketApiKeyGroup,
  UpbitPocketApiKeysRequest,
  UpbitPocketBalance,
  UpbitPocketTransfer,
  UpbitPocketTransferDirection,
  UpbitPocketTransferOrder,
  UpbitPocketTransferQuery,
  UpbitPocketTransferRequest,
  UpbitPocketTransferState,
  UpbitPocketUniversalTransferRequest,
  UpbitBatchCancelRequest,
  UpbitBatchCancelScope,
  UpbitCancelAndNewOrder,
  UpbitCancelAndNewOrderRequest,
  UpbitCancelAndNewOrderResult,
  UpbitDepositInfo,
  UpbitTravelRuleVasp,
  UpbitTravelRuleVerification,
  UpbitOrderDirection,
  UpbitOrderReference,
  UpbitOrderVolume,
  UpbitSmpType,
  UpbitKrwDeposit,
  UpbitKrwTransferRequest,
  UpbitKrwTwoFactorType,
  UpbitKrwWithdrawal,
  UpbitYearCandle,
  WithdrawRequest,
  WithdrawalFee,
  WithdrawalStatus,
} from "../dist/models.js";
import { InvalidRequestError } from "../dist/errors.js";
import {
  assetNetworkFromWire,
  assetNetworkToWire,
  binanceMarkPriceFromWire,
  binanceMarkPriceToWire,
  binanceOpenInterestFromWire,
  binanceOpenInterestToWire,
  binanceAggregateTradeFromWire,
  binanceAggregateTradeToWire,
  binanceAggregateTradesRequestFromWire,
  binanceAggregateTradesRequestToWire,
  binanceAccountTradeFromWire,
  binanceAccountTradeToWire,
  binanceC2cTradeFromWire,
  binanceC2cTradeHistoryPageFromWire,
  binanceC2cTradeHistoryPageToWire,
  binanceC2cTradeHistoryRequestFromWire,
  binanceC2cTradeHistoryRequestToWire,
  binanceC2cTradeToWire,
  binanceTestOrderFromWire,
  binanceTestOrderToWire,
  binanceTestOrderRequestFromWire,
  binanceTestOrderRequestToWire,
  bithumbBatchOrdersRequestFromWire,
  bithumbBatchOrdersRequestToWire,
  bithumbBatchOrdersResultFromWire,
  bithumbBatchOrdersResultToWire,
  bithumbKrwDepositFromWire,
  bithumbKrwDepositToWire,
  bithumbKrwDepositsRequestFromWire,
  bithumbKrwDepositsRequestToWire,
  bithumbKrwTransferRequestFromWire,
  bithumbKrwTransferRequestToWire,
  bithumbKrwWithdrawalFromWire,
  bithumbKrwWithdrawalToWire,
  bithumbKrwWithdrawalsRequestFromWire,
  bithumbKrwWithdrawalsRequestToWire,
  bithumbPendingOrdersRequestFromWire,
  bithumbPendingOrdersRequestToWire,
  bithumbTwapOrderRequestFromWire,
  bithumbTwapOrderRequestToWire,
  bithumbTwapOrdersRequestFromWire,
  bithumbTwapOrdersRequestToWire,
  bithumbWithdrawalAddressFromWire,
  bithumbWithdrawalAddressToWire,
  bithumbOrderDetailFromWire,
  bithumbOrderDetailRequestFromWire,
  bithumbOrderDetailRequestToWire,
  bithumbOrderDetailToWire,
  bithumbOrderDetailTradeFromWire,
  bithumbOrderDetailTradeToWire,
  bithumbOrderListItemFromWire,
  bithumbOrderListItemToWire,
  bithumbOrderListRequestFromWire,
  bithumbOrderListRequestToWire,
  cancelOrdersRequestFromWire,
  cancelOrdersRequestToWire,
  cancelOrdersResultFromWire,
  cancelOrdersResultToWire,
  depositAddressEntryFromWire,
  depositAddressEntryToWire,
  depositFromWire,
  depositToWire,
  hyperliquidMidPriceFromWire,
  hyperliquidMidPriceToWire,
  hyperliquidOpenOrderFromWire,
  hyperliquidOpenOrderToWire,
  hyperliquidOrderDetailFromWire,
  hyperliquidOrderDetailToWire,
  hyperliquidOrderInfoFromWire,
  hyperliquidOrderInfoToWire,
  hyperliquidOrderReferenceToWire,
  hyperliquidOrderStatusResponseFromWire,
  hyperliquidOrderStatusResponseToWire,
  hyperliquidPortfolioPeriodFromWire,
  hyperliquidPortfolioPeriodToWire,
  hyperliquidReferralFromWire,
  hyperliquidReferralToWire,
  hyperliquidUserFeesFromWire,
  hyperliquidUserFeesToWire,
  hyperliquidUserFillFromWire,
  hyperliquidUserFillToWire,
  hyperliquidUserRateLimitFromWire,
  hyperliquidUserRateLimitToWire,
  hyperliquidVaultEquityFromWire,
  hyperliquidVaultEquityToWire,
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
  upbitTravelRuleVaspFromWire,
  upbitTravelRuleVaspToWire,
  upbitTravelRuleVerificationFromWire,
  upbitTravelRuleVerificationToWire,
  upbitBatchCancelRequestFromWire,
  upbitBatchCancelRequestToWire,
  upbitBatchCancelScopeToWire,
  upbitCancelAndNewOrderRequestFromWire,
  upbitCancelAndNewOrderRequestToWire,
  upbitCancelAndNewOrderResultFromWire,
  upbitCancelAndNewOrderResultToWire,
  upbitOrderBookInstrumentFromWire,
  upbitOrderBookInstrumentToWire,
  upbitClosedOrderFromWire,
  upbitClosedOrderToWire,
  upbitClosedOrdersRequestFromWire,
  upbitClosedOrdersRequestToWire,
  bithumbClosedOrderFromWire,
  bithumbClosedOrdersRequestFromWire,
  bithumbClosedOrdersRequestToWire,
  upbitOrderDetailFromWire,
  upbitOrderDetailRequestFromWire,
  upbitOrderDetailRequestToWire,
  upbitOrderDetailToWire,
  upbitOrderDetailTradeFromWire,
  upbitOrderDetailTradeToWire,
  upbitYearCandleFromWire,
  upbitYearCandleToWire,
  upbitApiKeyFromWire,
  upbitApiKeyToWire,
  upbitPocketApiKeyFromWire,
  upbitPocketApiKeyGroupFromWire,
  upbitPocketApiKeyGroupToWire,
  upbitPocketApiKeyToWire,
  upbitPocketApiKeysRequestFromWire,
  upbitPocketApiKeysRequestToWire,
  upbitPocketBalanceFromWire,
  upbitPocketBalanceToWire,
  upbitPocketFromWire,
  upbitPocketToWire,
  upbitPocketTransferFromWire,
  upbitPocketTransferQueryFromWire,
  upbitPocketTransferQueryToWire,
  upbitPocketTransferRequestFromWire,
  upbitPocketTransferRequestToWire,
  upbitPocketTransferToWire,
  upbitPocketUniversalTransferRequestFromWire,
  upbitPocketUniversalTransferRequestToWire,
  upbitKrwDepositFromWire,
  upbitKrwDepositToWire,
  upbitKrwTransferRequestFromWire,
  upbitKrwTransferRequestToWire,
  upbitKrwWithdrawalFromWire,
  upbitKrwWithdrawalToWire,
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

test("Upbit closed orders preserve nullable values and omit detail-only trades", () => {
  const market = Market.spot(Exchange.Upbit, "BTC", "KRW");
  const request = new UpbitClosedOrdersRequest(
    market,
    UpbitClosedOrderState.Done,
    [UpbitClosedOrderState.Cancel],
    Timestamp.fromNanoseconds(-1n),
    Timestamp.fromNanoseconds(1n),
    1_000,
    UpbitOrderDirection.Ascending,
  );
  const order = new UpbitClosedOrder(
    market, "order-1", "bid", "best", "future_state", Timestamp.fromNanoseconds(-1n),
    null, Decimal.parse("100000.0000"), Decimal.zero, Decimal.parse("0.0100"), null,
    Decimal.zero, Decimal.zero, Decimal.zero, Decimal.zero, 0, Decimal.zero, Decimal.zero,
    "future_tif", "client-1", "future_smp",
  );

  assert.deepEqual(
    upbitClosedOrdersRequestToWire(upbitClosedOrdersRequestFromWire(
      upbitClosedOrdersRequestToWire(request),
    )),
    upbitClosedOrdersRequestToWire(request),
  );
  const wire = upbitClosedOrderToWire(order);
  assert.equal("trades" in wire, false);
  assert.deepEqual(upbitClosedOrderToWire(upbitClosedOrderFromWire(wire)), wire);
  assert.equal(Object.isFrozen(request.states), true);
  assert.equal(order.state, "future_state");
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

test("Upbit Travel Rule and Bithumb KRW records preserve provider values", () => {
  const vasp = new UpbitTravelRuleVasp("Example VASP", "vasp-1", true, false);
  const verification = new UpbitTravelRuleVerification("deposit-1", "ACCEPTED", "verified");
  const withdrawals = new BithumbKrwWithdrawalsRequest(
    "DONE", ["withdrawal-1"], ["tx-1"], 2, 20, BithumbOrderDirection.Descending,
  );
  const deposits = new BithumbKrwDepositsRequest();
  const transfer = new BithumbKrwTransferRequest(Decimal.parse("10000.00"));
  const withdrawal = new BithumbKrwWithdrawal(
    "withdraw", "withdrawal-1", "KRW", null, "tx-1", "DONE",
    Timestamp.fromMilliseconds(1_760_000_000_000n), null,
    Decimal.parse("10000.00"), Decimal.zero, "default",
  );
  const deposit = new BithumbKrwDeposit(
    "deposit", "deposit-1", "KRW", null, null, "ACCEPTED",
    Timestamp.fromMilliseconds(1_760_000_000_000n), null,
    Decimal.parse("10000.00"), Decimal.zero, null,
  );

  assert.deepEqual(upbitTravelRuleVaspToWire(upbitTravelRuleVaspFromWire(
    upbitTravelRuleVaspToWire(vasp),
  )), upbitTravelRuleVaspToWire(vasp));
  assert.deepEqual(upbitTravelRuleVerificationToWire(upbitTravelRuleVerificationFromWire(
    upbitTravelRuleVerificationToWire(verification),
  )), upbitTravelRuleVerificationToWire(verification));
  assert.deepEqual(bithumbKrwWithdrawalsRequestToWire(bithumbKrwWithdrawalsRequestFromWire(
    bithumbKrwWithdrawalsRequestToWire(withdrawals),
  )), bithumbKrwWithdrawalsRequestToWire(withdrawals));
  assert.deepEqual(new BithumbKrwDepositsRequest().uuids, []);
  assert.deepEqual(bithumbKrwDepositsRequestToWire(bithumbKrwDepositsRequestFromWire(
    bithumbKrwDepositsRequestToWire(deposits),
  )), bithumbKrwDepositsRequestToWire(deposits));
  assert.deepEqual(bithumbKrwTransferRequestToWire(bithumbKrwTransferRequestFromWire(
    bithumbKrwTransferRequestToWire(transfer),
  )), bithumbKrwTransferRequestToWire(transfer));
  assert.deepEqual(bithumbKrwWithdrawalToWire(bithumbKrwWithdrawalFromWire(
    bithumbKrwWithdrawalToWire(withdrawal),
  )), bithumbKrwWithdrawalToWire(withdrawal));
  assert.deepEqual(bithumbKrwDepositToWire(bithumbKrwDepositFromWire(
    bithumbKrwDepositToWire(deposit),
  )), bithumbKrwDepositToWire(deposit));
});

test("provider account records preserve exact transfers, JSON, and nullable metadata", () => {
  const upbitTransfer = new UpbitKrwTransferRequest(
    Decimal.parse("10000.00"),
    UpbitKrwTwoFactorType.Kakao,
  );
  const upbitDeposit = new UpbitKrwDeposit(
    "deposit", "deposit-1", "KRW", null, "tx-1", "ACCEPTED",
    Timestamp.fromMilliseconds(1760000000000n), null,
    Decimal.parse("10000.00"), Decimal.zero, "default",
  );
  const upbitWithdrawal = new UpbitKrwWithdrawal(
    "withdraw", "withdrawal-1", "KRW", null, null, "PROCESSING",
    Timestamp.fromMilliseconds(1760000000000n), null,
    Decimal.parse("10000.00"), Decimal.zero, "default", true,
  );
  const upbitKey = new UpbitApiKey("access-key-1", Timestamp.fromSeconds(1812672000n));
  const bithumbAddress = new BithumbWithdrawalAddress(
    "XRP", "XRP", null, "rAddress", "1234", null, null, null, null, null, null,
  );

  for (const [toWire, fromWire, value] of [
    [upbitKrwTransferRequestToWire, upbitKrwTransferRequestFromWire, upbitTransfer],
    [upbitKrwDepositToWire, upbitKrwDepositFromWire, upbitDeposit],
    [upbitKrwWithdrawalToWire, upbitKrwWithdrawalFromWire, upbitWithdrawal],
    [upbitApiKeyToWire, upbitApiKeyFromWire, upbitKey],
    [bithumbWithdrawalAddressToWire, bithumbWithdrawalAddressFromWire, bithumbAddress],
  ]) {
    assert.deepEqual(toWire(fromWire(toWire(value))), toWire(value));
  }

  const market = Market.spot(Exchange.Binance, "BTC", "USDT");
  const trade = new BinanceAccountTrade(
    market, "1", "2", Timestamp.fromMilliseconds(1760000000000n), Side.Buy, true, true,
    "-1", Decimal.parse("100000.00"), Decimal.parse("0.0100"),
    Decimal.parse("1000.0000"), Decimal.parse("0.0010"), "BNB", null, null, null, null, null,
  );
  const testOrder = new BinanceTestOrder('{"standardCommissionForOrder":{"maker":"0.001"}}');
  const testOrderRequest = new BinanceTestOrderRequest(
    OrderRequest.limit(market, Side.Buy, Size.base(Decimal.parse("0.01")), Decimal.parse("100000")),
    true,
  );
  for (const [toWire, fromWire, value] of [
    [binanceAccountTradeToWire, binanceAccountTradeFromWire, trade],
    [binanceTestOrderToWire, binanceTestOrderFromWire, testOrder],
    [binanceTestOrderRequestToWire, binanceTestOrderRequestFromWire, testOrderRequest],
  ]) {
    assert.deepEqual(toWire(fromWire(toWire(value))), toWire(value));
  }
  assert.equal(Object.isFrozen(trade), true);
});

test("pocket, order-detail, and C2C records preserve wire values", () => {
  const timestamp = Timestamp.fromMilliseconds(1760000000000n);
  const upbitMarket = Market.spot(Exchange.Upbit, "BTC", "KRW");
  const upbitRequest = new UpbitOrderDetailRequest(upbitMarket, "order-1", "client-1");
  const upbitTrade = new UpbitOrderDetailTrade(
    upbitMarket, "fill-1", Decimal.parse("100000.00"), Decimal.parse("0.0100"),
    Decimal.parse("1000.0000"), "up", timestamp, "bid",
  );
  const upbitDetail = new UpbitOrderDetail(
    upbitMarket, "order-1", "bid", "limit", Decimal.parse("100000.00"), "done", timestamp,
    Decimal.parse("0.0100"), Decimal.zero, Decimal.parse("0.0100"), Decimal.zero,
    Decimal.zero, Decimal.zero, Decimal.zero, 1, Decimal.zero, Decimal.zero, "ioc", "client-1",
    "reduce", [upbitTrade],
  );
  for (const [toWire, fromWire, value] of [
    [upbitOrderDetailRequestToWire, upbitOrderDetailRequestFromWire, upbitRequest],
    [upbitOrderDetailTradeToWire, upbitOrderDetailTradeFromWire, upbitTrade],
    [upbitOrderDetailToWire, upbitOrderDetailFromWire, upbitDetail],
  ]) {
    assert.deepEqual(toWire(fromWire(toWire(value))), toWire(value));
  }

  const upbitPocket = new UpbitPocket("pocket-1", "trading", "user_spot_trading");
  const upbitKey = new UpbitPocketApiKey(
    "access-key", ["View Account"], ["127.0.0.1"], timestamp, timestamp,
  );
  const upbitKeyGroup = new UpbitPocketApiKeyGroup("pocket-1", [upbitKey]);
  const upbitKeysRequest = new UpbitPocketApiKeysRequest(["pocket-1"], true);
  const upbitBalance = new UpbitPocketBalance(
    "BTC", Decimal.parse("1.2300"), Decimal.zero, Decimal.parse("100000.00"), false, "KRW",
  );
  const upbitQuery = new UpbitPocketTransferQuery(
    "main", "pocket-1", UpbitPocketTransferDirection.Incoming,
    [UpbitPocketTransferState.Done], ["transfer-1"], ["client-1"], timestamp, null,
    "BTC", 20, UpbitPocketTransferOrder.Ascending,
  );
  const upbitUniversalRequest = new UpbitPocketUniversalTransferRequest(
    "main", "pocket-1", "BTC", Decimal.parse("0.0100"), "client-1",
  );
  const upbitTransferRequest = new UpbitPocketTransferRequest(
    "pocket-2", "BTC", Decimal.parse("0.0100"), "client-2",
  );
  const upbitTransfer = new UpbitPocketTransfer(
    "transfer-1", "client-1", "main", "pocket-1", "done", "BTC", Decimal.parse("0.0100"), timestamp,
  );
  for (const [toWire, fromWire, value] of [
    [upbitPocketToWire, upbitPocketFromWire, upbitPocket],
    [upbitPocketApiKeyToWire, upbitPocketApiKeyFromWire, upbitKey],
    [upbitPocketApiKeyGroupToWire, upbitPocketApiKeyGroupFromWire, upbitKeyGroup],
    [upbitPocketApiKeysRequestToWire, upbitPocketApiKeysRequestFromWire, upbitKeysRequest],
    [upbitPocketBalanceToWire, upbitPocketBalanceFromWire, upbitBalance],
    [upbitPocketTransferQueryToWire, upbitPocketTransferQueryFromWire, upbitQuery],
    [upbitPocketUniversalTransferRequestToWire, upbitPocketUniversalTransferRequestFromWire, upbitUniversalRequest],
    [upbitPocketTransferRequestToWire, upbitPocketTransferRequestFromWire, upbitTransferRequest],
    [upbitPocketTransferToWire, upbitPocketTransferFromWire, upbitTransfer],
  ]) {
    assert.deepEqual(toWire(fromWire(toWire(value))), toWire(value));
  }

  assert.equal(Object.isFrozen(upbitKey.permissions), true);

  const bithumbMarket = Market.spot(Exchange.Bithumb, "BTC", "KRW");
  const bithumbRequest = new BithumbOrderDetailRequest(bithumbMarket, "order-1", "client-1");
  const bithumbTrade = new BithumbOrderDetailTrade(
    bithumbMarket, "fill-1", Decimal.parse("100000.00"), Decimal.parse("0.0100"),
    Decimal.parse("1000.0000"), "bid", timestamp,
  );
  const bithumbDetail = new BithumbOrderDetail(
    "order-1", "client-1", "bid", "limit", Decimal.parse("100000.00"), "done", bithumbMarket,
    timestamp, Decimal.parse("0.0100"), Decimal.zero, Decimal.zero, Decimal.zero, Decimal.zero,
    Decimal.zero, Decimal.parse("0.0100"), Decimal.parse("1000.0000"), 1, [bithumbTrade],
    null, null, null, "ioc",
  );
  for (const [toWire, fromWire, value] of [
    [bithumbOrderDetailRequestToWire, bithumbOrderDetailRequestFromWire, bithumbRequest],
    [bithumbOrderDetailTradeToWire, bithumbOrderDetailTradeFromWire, bithumbTrade],
    [bithumbOrderDetailToWire, bithumbOrderDetailFromWire, bithumbDetail],
  ]) {
    assert.deepEqual(toWire(fromWire(toWire(value))), toWire(value));
  }

  const bithumbListRequest = new BithumbOrderListRequest(
    bithumbMarket, BithumbOrderListState.Done, [BithumbOrderListState.Done], ["order-1"],
    ["client-1"], 1, 100, BithumbOrderDirection.Ascending,
  );
  const bithumbListItem = new BithumbOrderListItem(
    "order-1", "client-1", "bid", "limit", Decimal.parse("100000.00"), "done", bithumbMarket,
    timestamp, Decimal.parse("0.0100"), Decimal.zero, Decimal.zero, Decimal.zero, Decimal.zero,
    Decimal.zero, Decimal.parse("0.0100"), Decimal.parse("1000.0000"), 1, "cancel_maker", "ioc",
  );
  for (const [toWire, fromWire, value] of [
    [bithumbOrderListRequestToWire, bithumbOrderListRequestFromWire, bithumbListRequest],
    [bithumbOrderListItemToWire, bithumbOrderListItemFromWire, bithumbListItem],
  ]) {
    assert.deepEqual(toWire(fromWire(toWire(value))), toWire(value));
  }

  const c2cRequest = new BinanceC2cTradeHistoryRequest(
    BinanceC2cTradeType.Buy, timestamp, null, 1, 100, 60000n,
  );
  const c2cTrade = new BinanceC2cTrade(
    "order-1", "adv-1", "BUY", "BTC", "KRW", "₩", Decimal.parse("0.0100"),
    Decimal.parse("1000.00"), Decimal.parse("100000.00"), "COMPLETED", timestamp,
    Decimal.zero, "trader", "BANK", 2, Decimal.zero, Decimal.zero, Decimal.parse("0.0100"), "TAKER",
  );
  const c2cPage = new BinanceC2cTradeHistoryPage("000000", null, [c2cTrade], 1n, true);
  for (const [toWire, fromWire, value] of [
    [binanceC2cTradeHistoryRequestToWire, binanceC2cTradeHistoryRequestFromWire, c2cRequest],
    [binanceC2cTradeToWire, binanceC2cTradeFromWire, c2cTrade],
    [binanceC2cTradeHistoryPageToWire, binanceC2cTradeHistoryPageFromWire, c2cPage],
  ]) {
    assert.deepEqual(toWire(fromWire(toWire(value))), toWire(value));
  }
  assert.throws(
    () => new BinanceC2cTrade(
      null, null, null, null, null, null, null, null, null, null, null, null, null, null,
      -1, null, null, null, null,
    ),
    RangeError,
  );
});

test("Hyperliquid account records preserve u64, exact decimals, nested JSON, and frozen histories", () => {
  const timestamp = Timestamp.fromMilliseconds(1760000000000n);
  const rateLimit = new HyperliquidUserRateLimit(
    Decimal.parse("1.2300"), 18_446_744_073_709_551_615n, 2n, 0n,
  );
  const referral = new HyperliquidReferral(
    new HyperliquidReferrer("0xreferrer", "CODE"), Decimal.parse("1.2300"), Decimal.zero,
    Decimal.parse("2.00"), Decimal.zero, "{}", "[]", "{\"USDC\":{}}",
  );
  const userFees = new HyperliquidUserFees(
    [new HyperliquidDailyVolume("2026-08-12", Decimal.parse("1.00"), Decimal.parse("2.00"), Decimal.parse("3.00"))],
    "{}", Decimal.parse("0.00045"), Decimal.parse("0.00015"), null, null, null, "{}",
  );
  const userFill = new HyperliquidUserFill(
    "BTC", Decimal.parse("100000.00"), Decimal.parse("0.0100"), "B", timestamp,
    Decimal.parse("0.0100"), "Open Long", Decimal.zero, "0xfill",
    18_446_744_073_709_551_615n, true, Decimal.parse("0.0100"), null,
    18_446_744_073_709_551_614n, "USDC", 1n, "{\"future\":true}",
  );
  const period = new HyperliquidPortfolioPeriod(
    "day", [new HyperliquidPortfolioPoint(Timestamp.fromMilliseconds(1760000000000n), Decimal.parse("1.2300"))],
    [], Decimal.parse("9.00"),
  );
  const vault = new HyperliquidVaultEquity(
    "0xvault", Decimal.parse("42.0000"), Timestamp.fromMilliseconds(1760000000000n),
  );
  const subAccount = new HyperliquidSubAccount("sub", "0xuser", "0xmaster", "{}", "[]");

  for (const [toWire, fromWire, value] of [
    [hyperliquidUserRateLimitToWire, hyperliquidUserRateLimitFromWire, rateLimit],
    [hyperliquidReferralToWire, hyperliquidReferralFromWire, referral],
    [hyperliquidUserFeesToWire, hyperliquidUserFeesFromWire, userFees],
    [hyperliquidUserFillToWire, hyperliquidUserFillFromWire, userFill],
    [hyperliquidPortfolioPeriodToWire, hyperliquidPortfolioPeriodFromWire, period],
    [hyperliquidVaultEquityToWire, hyperliquidVaultEquityFromWire, vault],
  ]) {
    assert.deepEqual(toWire(fromWire(toWire(value))), toWire(value));
  }
  assert.equal(rateLimit.requestsUsed, 18_446_744_073_709_551_615n);
  assert.equal(userFill.orderId, 18_446_744_073_709_551_615n);
  assert.equal(userFill.rawJson, "{\"future\":true}");
  assert.equal(Object.isFrozen(userFees.dailyVolumes), true);
  assert.equal(Object.isFrozen(period.accountValueHistory), true);
  assert.equal(subAccount.perpetualStateJson, "{}");
});

test("Hyperliquid order queries preserve u64, timestamps, decimals, raw JSON, and status unions", () => {
  const timestamp = Timestamp.fromMilliseconds(1760000000000n);
  const detail = new HyperliquidOrderDetail(
    "BTC", "B", Decimal.parse("100000.00"), Decimal.parse("0.0100"),
    18_446_744_073_709_551_615n, timestamp, "N/A", false, Decimal.zero,
    "[]", false, true, "Limit", Decimal.parse("0.0200"), "Gtc",
    "0x0123456789abcdef0123456789abcdef", "{\"future\":true}",
  );
  const openOrder = new HyperliquidOpenOrder(
    "BTC", Decimal.parse("100000.00"), 18_446_744_073_709_551_615n, "B",
    Decimal.parse("0.0100"), timestamp, "{\"future\":true}",
  );
  const info = new HyperliquidOrderInfo(detail, "filled", timestamp, "{\"status\":\"filled\"}");

  for (const [toWire, fromWire, value] of [
    [hyperliquidOpenOrderToWire, hyperliquidOpenOrderFromWire, openOrder],
    [hyperliquidOrderDetailToWire, hyperliquidOrderDetailFromWire, detail],
    [hyperliquidOrderInfoToWire, hyperliquidOrderInfoFromWire, info],
  ]) {
    assert.deepEqual(toWire(fromWire(toWire(value))), toWire(value));
  }
  assert.deepEqual(hyperliquidOrderReferenceToWire({
    kind: "order_id", value: 18_446_744_073_709_551_615n,
  }), { kind: "order_id", value: "18446744073709551615" });
  for (const value of [
    { kind: "order", value: info },
    { kind: "unknown_order" },
    { kind: "other", status: "future", rawJson: "{\"future\":true}" },
  ]) {
    assert.deepEqual(
      hyperliquidOrderStatusResponseToWire(hyperliquidOrderStatusResponseFromWire(
        hyperliquidOrderStatusResponseToWire(value),
      )),
      hyperliquidOrderStatusResponseToWire(value),
    );
  }
});

test("Upbit conditional batch cancellation keeps its explicit scope and filters", () => {
  const market = Market.spot(Exchange.Upbit, "BTC", "KRW");
  const request = new UpbitBatchCancelRequest(
    UpbitBatchCancelScope.quoteCurrencies(["KRW"]),
    [market],
    Side.Buy,
    20,
    UpbitOrderDirection.Ascending,
  );

  assert.equal(request.scope.kind, "quote_currencies");
  assert.equal(request.count, 20);
  assert.equal(Object.isFrozen(request.excludedPairs), true);
  assert.deepEqual(
    upbitBatchCancelRequestToWire(upbitBatchCancelRequestFromWire(upbitBatchCancelRequestToWire(request))),
    upbitBatchCancelRequestToWire(request),
  );
});

test("Upbit batch cancellation rejects a restriction on the all scope", () => {
  const unsafeScope = { kind: "all", values: ["KRW"] };

  assert.throws(
    () => upbitBatchCancelScopeToWire(unsafeScope),
    (error) => error instanceof InvalidRequestError
      && error.field === "upbitBatchCancelScope.values",
  );

  const unsafeRequest = {
    scope: { kind: "all" },
    excludedPairs: null,
    side: null,
    count: 300,
    orderBy: null,
    pairs: ["KRW-BTC"],
  };
  assert.throws(
    () => upbitBatchCancelRequestToWire(unsafeRequest),
    (error) => error instanceof InvalidRequestError
      && error.field === "upbitBatchCancelRequest.pairs",
  );
});

test("Upbit cancel-and-new preserves the previous-order race outcome", () => {
  const market = Market.spot(Exchange.Upbit, "BTC", "KRW");
  const request = new UpbitCancelAndNewOrderRequest(
    UpbitOrderReference.identifier("old-client-id"),
    UpbitCancelAndNewOrder.limit(
      UpbitOrderVolume.amount(Decimal.parse("0.01")),
      Decimal.parse("100000000"),
      TimeInForce.ImmediateOrCancel,
    ),
    "replacement-client-id",
    UpbitSmpType.Reduce,
  );
  const result = new UpbitCancelAndNewOrderResult(
    new Order(
      "old-order",
      market,
      Side.Buy,
      OrderStatus.Filled,
      Decimal.parse("0.02"),
      Decimal.zero,
      Decimal.parse("100000000"),
      Timestamp.fromNanoseconds(1n),
    ),
    null,
    null,
  );

  assert.deepEqual(
    upbitCancelAndNewOrderRequestToWire(upbitCancelAndNewOrderRequestFromWire(
      upbitCancelAndNewOrderRequestToWire(request),
    )),
    upbitCancelAndNewOrderRequestToWire(request),
  );
  assert.deepEqual(
    upbitCancelAndNewOrderResultToWire(upbitCancelAndNewOrderResultFromWire(
      upbitCancelAndNewOrderResultToWire(result),
    )),
    upbitCancelAndNewOrderResultToWire(result),
  );
  assert.equal(result.replacementCreated, false);
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

test("Bithumb closed orders preserve nullable values and cursors", () => {
  const request = new BithumbClosedOrdersRequest(
    Market.spot(Exchange.Bithumb, "BTC", "KRW"),
    BithumbClosedOrderState.Done,
    [BithumbClosedOrderState.Cancel],
    Timestamp.fromNanoseconds(1n),
    Timestamp.fromNanoseconds(2n),
    25,
    BithumbOrderDirection.Ascending,
    new Cursor("next+/=="),
  );
  assert.deepEqual(bithumbClosedOrdersRequestToWire(request), {
    market: { exchange: "bithumb", kind: "spot", base: "BTC", quote: "KRW" },
    state: "done", states: ["cancel"], start_time: "1", end_time: "2", limit: 25,
    order_by: "asc", cursor: "next+/==",
  });
  assert.equal(
    bithumbClosedOrdersRequestFromWire(bithumbClosedOrdersRequestToWire(request)).cursor.value,
    "next+/==",
  );

  const order = bithumbClosedOrderFromWire({
    order_id: "order-1", side: "bid", order_type: "limit", price: null, state: "done",
    market: { exchange: "bithumb", kind: "spot", base: "BTC", quote: "KRW" },
    created_at: null, volume: "1", remaining_volume: "0", reserved_fee: "0.1",
    remaining_fee: "0", paid_fee: "0.1", locked: "0", executed_volume: "1",
    executed_funds: "100", trades_count: 4294967295, client_order_id: null, stp_type: null,
    time_in_force: null, cancel_type: null, canceling_order_id: null,
  });
  assert.ok(order instanceof BithumbClosedOrder);
  assert.equal(order.price, null);
  assert.equal(order.createdAt, null);
  assert.equal(order.tradesCount, 4294967295);
});

test("Bithumb batch requests retain partial outcomes and provider fields", () => {
  const market = Market.spot(Exchange.Bithumb, "BTC", "KRW");
  const request = new BithumbBatchOrdersRequest([
    OrderRequest.limit(market, Side.Buy, Size.base(Decimal.parse("0.01")), Decimal.parse("1000")),
  ]);
  const result = new BithumbBatchOrdersResult([
    BithumbBatchOrderOutcome.accepted(new BithumbBatchOrder(
      "order-1",
      "client-1",
      market,
      Side.Buy,
      OrderType.Limit,
      "post_only",
      "cancel_maker",
      Timestamp.fromNanoseconds(2n),
    )),
    BithumbBatchOrderOutcome.rejected(new BithumbBatchOrderFailure(
      "second",
      "ioc",
      "cross_trading",
      "rejected",
    )),
  ]);

  assert.deepEqual(
    bithumbBatchOrdersRequestToWire(bithumbBatchOrdersRequestFromWire(
      bithumbBatchOrdersRequestToWire(request),
    )),
    bithumbBatchOrdersRequestToWire(request),
  );
  assert.deepEqual(
    bithumbBatchOrdersResultToWire(bithumbBatchOrdersResultFromWire(
      bithumbBatchOrdersResultToWire(result),
    )),
    bithumbBatchOrdersResultToWire(result),
  );
});

test("Bithumb TWAP requests preserve defaults, filters, and u32 boundaries", () => {
  const query = new BithumbTwapOrdersRequest(
    null,
    [],
    BithumbTwapState.Progress,
    new Cursor("page+/=="),
    25,
    BithumbTwapOrderDirection.Ascending,
  );
  const request = new BithumbTwapOrderRequest(
    Market.spot(Exchange.Bithumb, "BTC", "KRW"),
    Side.Buy,
    null,
    Decimal.parse("10000"),
    300,
    30,
  );

  assert.deepEqual(new BithumbTwapOrdersRequest().uuids, []);
  assert.deepEqual(
    bithumbTwapOrdersRequestToWire(bithumbTwapOrdersRequestFromWire(
      bithumbTwapOrdersRequestToWire(query),
    )),
    bithumbTwapOrdersRequestToWire(query),
  );
  assert.deepEqual(
    bithumbTwapOrderRequestToWire(bithumbTwapOrderRequestFromWire(
      bithumbTwapOrderRequestToWire(request),
    )),
    bithumbTwapOrderRequestToWire(request),
  );
  assert.throws(
    () => new BithumbTwapOrderRequest(request.market, Side.Buy, null, Decimal.one, 2 ** 32, 30),
    RangeError,
  );
});

test("Binance USD-M and Hyperliquid market snapshots preserve provider values", () => {
  const binanceMarket = Market.perpetual(Exchange.Binance, "BTC", "USDT");
  const markPrice = new BinanceMarkPrice(
    binanceMarket,
    Decimal.parse("100001.25"),
    Decimal.parse("100000.75"),
    null,
    Decimal.parse("0.0001"),
    Decimal.zero,
    Timestamp.fromMilliseconds(1760000000000n),
    Timestamp.fromMilliseconds(1759999999000n),
  );
  const openInterest = new BinanceOpenInterest(
    binanceMarket,
    Decimal.parse("123.456"),
    Timestamp.fromMilliseconds(1759999999000n),
  );
  const midPrice = new HyperliquidMidPrice(
    Market.perpetual(Exchange.Hyperliquid, "ETH", "USD"),
    Decimal.parse("3500.125"),
  );

  assert.deepEqual(
    binanceMarkPriceToWire(binanceMarkPriceFromWire(binanceMarkPriceToWire(markPrice))),
    binanceMarkPriceToWire(markPrice),
  );
  assert.deepEqual(
    binanceOpenInterestToWire(binanceOpenInterestFromWire(binanceOpenInterestToWire(openInterest))),
    binanceOpenInterestToWire(openInterest),
  );
  assert.deepEqual(
    hyperliquidMidPriceToWire(hyperliquidMidPriceFromWire(hyperliquidMidPriceToWire(midPrice))),
    hyperliquidMidPriceToWire(midPrice),
  );

  const aggregateRequest = new BinanceAggregateTradesRequest(
    binanceMarket,
    18_446_744_073_709_551_615n,
    Timestamp.fromMilliseconds(1759996400000n),
    Timestamp.fromMilliseconds(1759999999000n),
    1000,
  );
  const aggregateTrade = new BinanceAggregateTrade(
    binanceMarket,
    18_446_744_073_709_551_615n,
    18_446_744_073_709_551_614n,
    18_446_744_073_709_551_615n,
    Timestamp.fromMilliseconds(1759999999000n),
    Decimal.parse("100001.25"),
    Decimal.parse("0.01"),
    null,
    null,
    Side.Sell,
    "{}",
  );
  assert.deepEqual(
    binanceAggregateTradesRequestToWire(binanceAggregateTradesRequestFromWire(
      binanceAggregateTradesRequestToWire(aggregateRequest),
    )),
    binanceAggregateTradesRequestToWire(aggregateRequest),
  );
  assert.deepEqual(
    binanceAggregateTradeToWire(binanceAggregateTradeFromWire(
      binanceAggregateTradeToWire(aggregateTrade),
    )),
    binanceAggregateTradeToWire(aggregateTrade),
  );
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
  assert.equal(Feature.TravelRule.needsCredentials, true);
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
