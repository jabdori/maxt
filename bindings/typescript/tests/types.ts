import {
  BinanceAdapter,
  BinanceAggregateTradesRequest,
  type BinanceAccountTrade,
  type BinanceC2cTradeHistoryPage,
  BinanceC2cTradeHistoryRequest,
  BinanceC2cTradeType,
  type BinanceAggregateTrade,
  type BinanceMarkPrice,
  type BinanceOpenInterest,
  BinanceTestOrderRequest,
  type BinanceTestOrder,
  ChainDestination,
  Client,
  CancelOrdersRequest,
  Decimal,
  type DepositAddressEntry,
  DepositAddressRequest,
  Exchange,
  Feed,
  HistoryRequest,
  Market,
  Network,
  OrderIdKind,
  OrderRequest,
  OrderLookupRequest,
  TransferDestination,
  TransferError,
  TransferErrorKind,
  TransferHistoryRequest,
  TransferLookupRequest,
  Timestamp,
  WithdrawRequest,
  UpbitAdapter,
  type UpbitClosedOrder,
  UpbitClosedOrdersRequest,
  UpbitClosedOrderState,
  type UpbitOrderDetail,
  UpbitOrderDetailRequest,
  type UpbitApiKey,
  type UpbitPocket,
  type UpbitPocketApiKeyGroup,
  UpbitPocketApiKeysRequest,
  type UpbitPocketBalance,
  type UpbitPocketTransfer,
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
  UpbitOrderDirection,
  UpbitOrderReference,
  UpbitOrderVolume,
  UpbitSmpType,
  UpbitKrwTransferRequest,
  UpbitKrwTwoFactorType,
  type UpbitKrwDeposit,
  type UpbitKrwWithdrawal,
  BithumbAdapter,
  BithumbBatchOrdersRequest,
  type BithumbClosedOrder,
  BithumbClosedOrderState,
  BithumbClosedOrdersRequest,
  BithumbKrwDepositsRequest,
  BithumbKrwTransferRequest,
  BithumbKrwWithdrawalsRequest,
  type BithumbBatchOrdersResult,
  type BithumbKrwDeposit,
  type BithumbKrwWithdrawal,
  BithumbOrderDirection,
  type BithumbOrderListItem,
  BithumbOrderListRequest,
  BithumbOrderListState,
  BithumbPendingOrderState,
  BithumbPendingOrdersRequest,
  BithumbTwapOrder,
  BithumbTwapOrderRequest,
  BithumbTwapOrdersRequest,
  Cursor,
  HyperliquidAdapter,
  type HyperliquidAccountStream,
  type HyperliquidMarketStream,
  type HyperliquidMidPrice,
  type HyperliquidOpenOrder,
  type HyperliquidOrderInfo,
  type HyperliquidOrderReference,
  type HyperliquidOrderStatusResponse,
  type HyperliquidPortfolioPeriod,
  type HyperliquidReferral,
  type HyperliquidSubAccount,
  type HyperliquidUserFees,
  type HyperliquidUserFill,
  type HyperliquidUserRateLimit,
  type HyperliquidUserRole,
  type HyperliquidVaultEquity,
  type AssetNetwork,
  type Deposit,
  type Page,
  type Ticker,
  type Withdrawal,
  type WithdrawalQuote,
  type UpbitOrderBookInstrument,
  type UpbitDepositInfo,
  type UpbitTravelRuleVasp,
  type UpbitTravelRuleVerification,
  type UpbitYearCandle,
  type BithumbApiKey,
  type BithumbAssetFee,
  type BithumbNotice,
  type BithumbWithdrawalAddress,
  type BithumbOrderDetail,
  BithumbOrderDetailRequest,
  type Order,
  Side,
  Size,
  Subscription,
  TimeInForce,
} from "../src/node.js";

type NodeExports = typeof import("../src/node.js");
type BrowserExports = typeof import("../src/browser.js");
type Assert<T extends true> = T;
type _BrowserContainsExactlyNodeExports = Assert<
  BrowserExports extends NodeExports
    ? NodeExports extends BrowserExports ? true : false
    : false
>;

const adapter = BinanceAdapter.spot();
const client = new Client(adapter);
const market = Market.spot(Exchange.Binance, "BTC", "USDT");
const ticker: Promise<Ticker> = client.ticker(market);
const filters = client.adapter.spotSymbolFilters(market);
const usdM = BinanceAdapter.usdMFutures();
const usdMMarket = Market.perpetual(Exchange.Binance, "BTC", "USDT");
const markPrice: Promise<BinanceMarkPrice> = usdM.markPrice(usdMMarket);
const markPrices: Promise<readonly BinanceMarkPrice[]> = usdM.markPrices();
const openInterest: Promise<BinanceOpenInterest> = usdM.openInterest(usdMMarket);
const aggregateTrades: Promise<readonly BinanceAggregateTrade[]> = usdM.aggregateTrades(
  new BinanceAggregateTradesRequest(usdMMarket, 100n),
);
const accountTrades: Promise<Page<BinanceAccountTrade>> = adapter.accountTrades(
  new HistoryRequest(market),
);
const c2cTradeHistory: Promise<BinanceC2cTradeHistoryPage> = adapter.c2cTradeHistory(
  new BinanceC2cTradeHistoryRequest(BinanceC2cTradeType.Buy),
);
const binanceTestOrder: Promise<BinanceTestOrder> = adapter.testOrder(
  new BinanceTestOrderRequest(OrderRequest.limit(
    market, Side.Buy, Size.base(Decimal.parse("0.01")), Decimal.parse("100000"),
  )),
);
const cancelAllOpenOrders: Promise<void> = adapter.cancelAllOpenOrders(market);
const hyperliquid = new HyperliquidAdapter();
const allMids: Promise<readonly HyperliquidMidPrice[]> = hyperliquid.allMids();
const hyperliquidMarket = Market.perpetual(Exchange.Hyperliquid, "BTC", "USDC");
const detailedMarket: Promise<HyperliquidMarketStream> = hyperliquid.subscribeDetailed(
  new Subscription([hyperliquidMarket], [Feed.Trades]),
);
const detailedAccount: Promise<HyperliquidAccountStream> =
  hyperliquid.subscribeDetailedAccount();
const userRateLimit: Promise<HyperliquidUserRateLimit> = hyperliquid.userRateLimit();
const userRole: Promise<HyperliquidUserRole> = hyperliquid.userRole();
const referral: Promise<HyperliquidReferral> = hyperliquid.referral();
const userFees: Promise<HyperliquidUserFees> = hyperliquid.userFees();
const userFills: Promise<readonly HyperliquidUserFill[]> = hyperliquid.userFills(true);
const userFillsByTime: Promise<readonly HyperliquidUserFill[]> = hyperliquid.userFillsByTime(
  Timestamp.zero, null, false,
);
const portfolio: Promise<readonly HyperliquidPortfolioPeriod[]> = hyperliquid.portfolio();
const subAccounts: Promise<readonly HyperliquidSubAccount[]> = hyperliquid.subAccounts();
const vaultEquities: Promise<readonly HyperliquidVaultEquity[]> = hyperliquid.userVaultEquities();
const basicOpenOrders: Promise<readonly HyperliquidOpenOrder[]> = hyperliquid.basicOpenOrders();
const orderStatus: Promise<HyperliquidOrderStatusResponse> = hyperliquid.orderStatus(
  { kind: "order_id", value: 1n } satisfies HyperliquidOrderReference,
);
const historicalOrders: Promise<readonly HyperliquidOrderInfo[]> = hyperliquid.historicalOrders();
const upbit = new UpbitAdapter();
const upbitMarket = Market.spot(Exchange.Upbit, "BTC", "KRW");
const upbitOrderDetail: Promise<UpbitOrderDetail> = upbit.orderDetail(
  new UpbitOrderDetailRequest(upbitMarket, "order-1"),
);
const closedOrders: Promise<readonly UpbitClosedOrder[]> = upbit.closedOrders(
  new UpbitClosedOrdersRequest(null, UpbitClosedOrderState.Done),
);
const quoteTickers: Promise<readonly Ticker[]> = upbit.tickersByQuote(["KRW"]);
const aggregatedBooks = upbit.orderBooksAtLevel(
  [upbitMarket],
  Decimal.parse("100000"),
);
const yearCandles: Promise<readonly UpbitYearCandle[]> = upbit.yearCandles(upbitMarket, null, 1);
const instruments: Promise<readonly UpbitOrderBookInstrument[]> =
  upbit.orderbookInstruments([upbitMarket]);
const testOrder: Promise<Order> = upbit.testOrder(
  OrderRequest.limit(
    upbitMarket,
    Side.Buy,
    Size.base(Decimal.parse("0.01")),
    Decimal.parse("100000000"),
  ),
);
const depositInfo: Promise<UpbitDepositInfo> = upbit.depositInfo("BTC", Network.Bitcoin);
const batchCancellation = upbit.batchCancelOpenOrders(
  new UpbitBatchCancelRequest(
    UpbitBatchCancelScope.all(),
    null,
    null,
    20,
    UpbitOrderDirection.Ascending,
  ),
);
const cancelAndNew = upbit.cancelAndNewOrder(
  new UpbitCancelAndNewOrderRequest(
    UpbitOrderReference.identifier("old-client-id"),
    UpbitCancelAndNewOrder.limit(
      UpbitOrderVolume.amount(Decimal.parse("0.01")),
      Decimal.parse("100000000"),
      TimeInForce.ImmediateOrCancel,
    ),
    "new-client-id",
    UpbitSmpType.Reduce,
  ),
);
const travelRuleVasps: Promise<readonly UpbitTravelRuleVasp[]> = upbit.travelRuleVasps();
const travelRuleByUuid: Promise<UpbitTravelRuleVerification> = upbit.verifyTravelRuleByUuid(
  "deposit-1",
  "vasp-1",
);
const travelRuleByTxid: Promise<UpbitTravelRuleVerification> = upbit.verifyTravelRuleByTxid(
  "tx-1",
  "vasp-1",
  "BTC",
  "BTC",
);
const upbitDeposit: Promise<UpbitKrwDeposit> = upbit.depositKrw(
  new UpbitKrwTransferRequest(Decimal.parse("10000"), UpbitKrwTwoFactorType.Kakao),
);
const upbitWithdrawal: Promise<UpbitKrwWithdrawal> = upbit.withdrawKrw(
  new UpbitKrwTransferRequest(Decimal.parse("10000"), UpbitKrwTwoFactorType.Hana),
);
const upbitApiKeys: Promise<readonly UpbitApiKey[]> = upbit.apiKeys();
const pockets: Promise<readonly UpbitPocket[]> = upbit.listPockets();
const pocketApiKeys: Promise<readonly UpbitPocketApiKeyGroup[]> = upbit.listPocketApiKeys(
  new UpbitPocketApiKeysRequest(),
);
const pocketBalances: Promise<readonly UpbitPocketBalance[]> = upbit.subPocketBalances("pocket-1");
const universalTransfer: Promise<UpbitPocketTransfer> = upbit.universalTransfer(
  new UpbitPocketUniversalTransferRequest(null, "pocket-1", "BTC", Decimal.parse("0.01")),
);
const universalTransfers: Promise<readonly UpbitPocketTransfer[]> = upbit.universalTransfers(
  new UpbitPocketTransferQuery(
    null, null, UpbitPocketTransferDirection.All, [UpbitPocketTransferState.Done], [], [],
    null, null, null, 20, UpbitPocketTransferOrder.Ascending,
  ),
);
const subPocketTransfer: Promise<UpbitPocketTransfer> = upbit.subPocketTransfer(
  new UpbitPocketTransferRequest("pocket-1", "BTC", Decimal.parse("0.01")),
);
const subPocketTransfers: Promise<readonly UpbitPocketTransfer[]> = upbit.subPocketTransfers(
  new UpbitPocketTransferQuery(),
);
const bithumb = new BithumbAdapter();
const notices: Promise<readonly BithumbNotice[]> = bithumb.notices();
const transferFees: Promise<readonly BithumbAssetFee[]> = bithumb.transferFees("BTC");
const apiKeys: Promise<readonly BithumbApiKey[]> = bithumb.apiKeys();
const withdrawalAddresses: Promise<readonly BithumbWithdrawalAddress[]> = bithumb.withdrawalAddresses();
const orderDetail: Promise<BithumbOrderDetail> = bithumb.orderDetail(
  new BithumbOrderDetailRequest(Market.spot(Exchange.Bithumb, "BTC", "KRW"), "order-1"),
);
const orderList: Promise<readonly BithumbOrderListItem[]> = bithumb.orderList(
  new BithumbOrderListRequest(
    Market.spot(Exchange.Bithumb, "BTC", "KRW"), BithumbOrderListState.Done,
  ),
);
const pendingOrders: Promise<Page<Order>> = bithumb.pendingOrders(
  new BithumbPendingOrdersRequest(
    Market.spot(Exchange.Bithumb, "BTC", "KRW"),
    BithumbPendingOrderState.Watch,
    25,
    BithumbOrderDirection.Ascending,
    new Cursor("page+/=="),
  ),
);
const bithumbClosedOrders: Promise<Page<BithumbClosedOrder>> = bithumb.closedOrders(
  new BithumbClosedOrdersRequest(null, BithumbClosedOrderState.Done),
);
const batchOrders: Promise<BithumbBatchOrdersResult> = bithumb.batchOrders(
  new BithumbBatchOrdersRequest([]),
);
const twapOrders: Promise<Page<BithumbTwapOrder>> = bithumb.twapOrders(
  new BithumbTwapOrdersRequest(),
);
const twapOrder: Promise<string> = bithumb.createTwapOrder(
  new BithumbTwapOrderRequest(
    Market.spot(Exchange.Bithumb, "BTC", "KRW"),
    Side.Buy,
    null,
    Decimal.one,
    300,
    30,
  ),
);
const twapCancellation: Promise<string> = bithumb.cancelTwapOrder("twap-1");
const krwWithdrawals: Promise<readonly BithumbKrwWithdrawal[]> = bithumb.krwWithdrawals(
  new BithumbKrwWithdrawalsRequest(),
);
const krwWithdrawal: Promise<BithumbKrwWithdrawal> = bithumb.withdrawKrw(
  new BithumbKrwTransferRequest(Decimal.parse("10000")),
);
const krwDeposits: Promise<readonly BithumbKrwDeposit[]> = bithumb.krwDeposits(
  new BithumbKrwDepositsRequest(),
);
const krwDeposit: Promise<BithumbKrwDeposit> = bithumb.depositKrw(
  new BithumbKrwTransferRequest(Decimal.parse("10000")),
);
const destination = TransferDestination.chain(
  new ChainDestination("BTC", Network.Bitcoin, "bc1qdestination"),
);
const withdrawRequest = new WithdrawRequest("BTC", Network.Bitcoin, Decimal.one, destination);
const networks: Promise<readonly AssetNetwork[]> = client.assetNetworks("BTC");
const addresses: Promise<readonly DepositAddressEntry[]> = client.depositAddresses();
const address = client.depositAddress(new DepositAddressRequest("BTC", Network.Bitcoin));
const quote: Promise<WithdrawalQuote> = client.prepareWithdrawal(withdrawRequest);
const withdrawal: Promise<Withdrawal> = client.withdraw(withdrawRequest);
const deposit = client.deposit(new TransferLookupRequest("BTC", "deposit-1"));
const withdrawalLookup = client.withdrawal(new TransferLookupRequest("BTC", "withdrawal-1"));
const cancellation: Promise<void> = client.cancelWithdrawal("withdrawal-1");
const deposits: Promise<Page<Deposit>> = client.deposits(new TransferHistoryRequest());
const withdrawals: Promise<Page<Withdrawal>> = client.withdrawals(new TransferHistoryRequest());
const orders = client.ordersByIds(new OrderLookupRequest(OrderIdKind.Exchange, ["order-1"], market));
const cancellations = client.cancelOrders(
  new CancelOrdersRequest(OrderIdKind.Exchange, ["order-1"]),
);
const transferError = new TransferError(TransferErrorKind.NetworkMismatch, "chains differ");
void ticker;
void filters;
void markPrice;
void markPrices;
void openInterest;
void aggregateTrades;
void accountTrades;
void c2cTradeHistory;
void binanceTestOrder;
void cancelAllOpenOrders;
void allMids;
void userRateLimit;
void userRole;
void referral;
void userFees;
void userFills;
void userFillsByTime;
void portfolio;
void subAccounts;
void vaultEquities;
void basicOpenOrders;
void orderStatus;
void historicalOrders;
void quoteTickers;
void upbitOrderDetail;
void closedOrders;
void aggregatedBooks;
void yearCandles;
void instruments;
void testOrder;
void depositInfo;
void batchCancellation;
void cancelAndNew;
void travelRuleVasps;
void travelRuleByUuid;
void travelRuleByTxid;
void upbitDeposit;
void upbitWithdrawal;
void upbitApiKeys;
void pockets;
void pocketApiKeys;
void pocketBalances;
void universalTransfer;
void universalTransfers;
void subPocketTransfer;
void subPocketTransfers;
void notices;
void transferFees;
void apiKeys;
void withdrawalAddresses;
void orderDetail;
void orderList;
void pendingOrders;
void bithumbClosedOrders;
void batchOrders;
void twapOrders;
void twapOrder;
void twapCancellation;
void krwWithdrawals;
void krwWithdrawal;
void krwDeposits;
void krwDeposit;
void networks;
void addresses;
void address;
void quote;
void withdrawal;
void deposit;
void withdrawalLookup;
void cancellation;
void deposits;
void withdrawals;
void orders;
void cancellations;
void transferError;
void (null as unknown as _BrowserContainsExactlyNodeExports);
